//! The hygiene collector: the `Walker`, its traversal and the entry points.

use std::collections::{HashMap, HashSet};

use rqx_core::facts::{Hygiene, Limits, Spot};
use rqx_core::rule::RuleId;
use rqx_core::span::Span;
use tree_sitter::Node;

use crate::names::Names;
use crate::profile::LanguageProfile;
use crate::scope::{BindingKind, Scopes};
use crate::tree::{Nodes, descendants, kind_of, operator_of, span_of, text_of};
use crate::vocabulary::{
    configuration_literal, foreign_language, query_prefix, same_family, vague_verb, verb_and_object,
};

use comments::scan_comments;
use naming::{distant_declarations, note_conventions, short_names};
use parameters::reassigned_parameters;

mod asynchrony;
mod comments;
mod errors;
mod flow;
mod modules;
mod naming;
mod parameters;
mod performance;
mod testing;
mod types;
mod values;

/// One function literal's view of the loops around it. A callback handed to
/// `forEach` runs per element, so it inherits the loops of its caller; anything else
/// starts clear of them.
#[derive(Default)]
struct PerfFrame {
    is_async: bool,
    loops: u32,
    /// Loops written as loops. A callback is repetition the caller chose, and
    /// awaiting inside one is how concurrency is spelled rather than a mistake.
    statement_loops: u32,
}

/// Functions in one file that assert, each with the stretch of source that can call
/// it: the whole file for one declared at the top or in a class, its enclosing
/// function for one declared inside a function.
#[derive(Default)]
struct Helpers {
    known: Vec<(String, Option<std::ops::Range<usize>>)>,
}

impl Helpers {
    fn reaches(&self, name: &str, call: Node<'_>) -> bool {
        self.known.iter().any(|(known, scope)| {
            known == name
                && scope
                    .as_ref()
                    .is_none_or(|scope| scope.contains(&call.start_byte()))
        })
    }

    fn is_empty(&self) -> bool {
        self.known.is_empty()
    }
}

/// An async function the file declares, with the class it is a method of. A call
/// reaches it by its name and, for a method, through that class.
struct AsyncFunction {
    name: String,
    class: Option<String>,
}

struct Walker<'a> {
    profile: &'static LanguageProfile,
    source: &'a str,
    nodes: &'a Nodes<'a>,
    limits: Limits,
    extra: &'a rqx_core::config::Performance,
    out: Hygiene,
    numbers: HashMap<String, Vec<Span>>,
    boundaries: HashMap<String, Vec<Span>>,
    conventions: Vec<(String, Span)>,
    verbs: Vec<(String, String, Span)>,
    /// What each local name was introduced as, so a rule can ask whether a value is a
    /// list, a string, or something already keyed.
    declared: HashMap<String, String>,
    /// The names whose declaration spells a text type, as opposed to one that merely
    /// started out holding a piece of text.
    text_declared: HashSet<String>,
    frames: Vec<PerfFrame>,
    test_depth: u32,
    /// Whether anything in this file turned out to be a test. Known only once the
    /// walk is over, so the rules that need it run after it.
    holds_tests: bool,
    block_depth: u32,
    /// Functions in this file that assert, so a test calling one asserts through it.
    asserting_helpers: Helpers,
    /// Where a name such as `fit` or `xit` means something this file declared, so a
    /// call on it is that, not the test runner's.
    local_marker_names: HashSet<(u32, u32)>,
}

impl<'a> Walker<'a> {
    fn names(&self) -> Names<'a> {
        Names::new(self.profile, self.source)
    }

    fn text(&self, node: Node<'a>) -> &'a str {
        text_of(node, self.source)
    }

    fn see_through(&self, node: Node<'a>) -> Node<'a> {
        self.names().see_through(node)
    }

    fn callee_name(&self, node: Node<'a>) -> Option<String> {
        self.names().callee_name(node)
    }

    fn member_property(&self, node: Node<'a>) -> Option<Node<'a>> {
        self.names().member_property(node)
    }

    fn called_name(&self, call: Node<'a>) -> Option<String> {
        self.names().called_name(call)
    }

    /// What a call is named by, receiver included and arguments left out: the part of
    /// a call a finding is about when the call is the whole statement.
    fn callee_span(&self, call: Node<'a>) -> Option<Span> {
        let callee = call.child_by_field_name(self.profile.callee_field)?;
        (self.profile.is_call(call) && callee.end_byte() > call.start_byte())
            .then(|| Span::new(call.start_byte() as u32, callee.end_byte() as u32))
    }

    /// The name a declaration is introduced by, as the part of it a finding is about.
    fn focus_on_name(&mut self, rule: RuleId, declaration: Node<'a>) {
        if let Some(name) = self.function_name(declaration) {
            self.out.focus(rule, span_of(declaration), span_of(name));
        }
    }

    /// Counts the decisions packed into one expression. Nothing here is named, so each
    /// one is a step the reader has to hold rather than read.
    fn operations(&self, node: Node<'a>) -> u32 {
        descendants(node)
            .into_iter()
            .filter(|inner| {
                self.profile.binary_kinds.contains(&kind_of(inner))
                    || self.profile.is_call(*inner)
                    || kind_of(inner) == "ternary_expression"
                    || kind_of(inner) == "conditional_expression"
            })
            .count() as u32
    }

    /// Both sides of `a, b := x, y` as lists, and either side of `a = x` as a list of one.
    fn listed(&self, node: Node<'a>) -> Vec<Node<'a>> {
        if self.profile.expression_list_kinds.contains(&kind_of(node)) {
            let mut cursor = node.walk();
            node.named_children(&mut cursor).collect()
        } else {
            vec![node]
        }
    }

    /// A query that writes is only interesting when the write outlives the call, so
    /// assignments to a local are not counted — a field or a captured name is.
    fn body_mutates(&self, body: Node<'a>) -> bool {
        descendants(body)
            .into_iter()
            .filter(|node| self.profile.assignment_kinds.contains(&kind_of(node)))
            .any(|assignment| {
                assignment
                    .child_by_field_name("left")
                    .is_some_and(|target| {
                        let target = if self
                            .profile
                            .expression_list_kinds
                            .contains(&kind_of(target))
                        {
                            target.named_child(0).unwrap_or(target)
                        } else {
                            target
                        };
                        self.profile.is_member_access(target)
                    })
            })
    }

    /// Shared by plain functions and by methods, which in most grammars are a
    /// different node kind and would otherwise be invisible to both rules.
    fn name_the_body(&mut self, name: &str, span: Span, body: Option<Node<'a>>) {
        if vague_verb(name) {
            self.out.vague_names.push(Spot {
                span,
                detail: name.to_owned(),
                count: 1,
            });
        }
        if let Some((verb, object)) = verb_and_object(name) {
            self.verbs.push((verb.to_owned(), object, span));
        }
        if query_prefix(name) && body.is_some_and(|body| self.body_mutates(body)) {
            self.out.side_effecting_queries.push(Spot {
                span,
                detail: name.to_owned(),
                count: 1,
            });
        }
    }

    fn function_name(&self, node: Node<'a>) -> Option<Node<'a>> {
        self.names().function_name(node)
    }

    fn function_body(&self, node: Node<'a>) -> Option<Node<'a>> {
        self.names().function_body(node)
    }

    fn declared_type(&self, parameter: Node<'a>) -> Option<&'a str> {
        let annotation = descendants(parameter)
            .into_iter()
            .find(|child| self.profile.type_annotation_kinds.contains(&kind_of(child)))?;
        let inner = annotation.named_child(0).unwrap_or(annotation);
        Some(self.text(inner))
    }

    fn annotations(&self, function: Node<'a>) -> Vec<String> {
        let mut found = Vec::new();
        if let Some(parent) = function.parent()
            && self.profile.decorated_kinds.contains(&kind_of(parent))
        {
            let mut cursor = parent.walk();
            found.extend(
                parent
                    .named_children(&mut cursor)
                    .filter(|child| self.profile.decorator_kinds.contains(&kind_of(child)))
                    .map(|child| self.text(child).to_owned()),
            );
        }
        let mut cursor = function.walk();
        for container in function.named_children(&mut cursor).filter(|child| {
            self.profile
                .modifier_container_kinds
                .contains(&kind_of(child))
        }) {
            found.extend(
                descendants(container)
                    .into_iter()
                    .filter(|child| self.profile.decorator_kinds.contains(&kind_of(child)))
                    .map(|child| self.text(child).to_owned()),
            );
        }
        found
    }

    /// The run of statements a loop repeats. Go puts a `statement_list` inside the
    /// `block` a loop's body field names, so the statements are one level deeper
    /// than that field suggests.
    fn loop_statements(&self, loop_node: Node<'a>) -> Option<Node<'a>> {
        let profile = self.profile;
        let body = self.function_body(loop_node).or_else(|| {
            loop_node
                .named_children(&mut loop_node.walk())
                .find(|child| profile.is_block(*child))
        })?;
        Some(
            body.named_children(&mut body.walk())
                .find(|child| profile.is_block(*child))
                .unwrap_or(body),
        )
    }

    /// One line of a node's text, short enough to read inside a finding.
    fn snippet(&self, node: Node<'a>) -> String {
        let line = self.text(node).lines().next().unwrap_or("").trim();
        match line.char_indices().nth(48) {
            Some((at, _)) => format!("{}…", &line[..at]),
            None => line.to_owned(),
        }
    }

    /// A catch whose body does nothing and says nothing. A comment is the reason the
    /// rule asks for; Python's sits beside the block rather than in it, so the whole
    /// clause is searched.
    fn swallows_silently(&self, catch: Node<'a>) -> bool {
        let profile = self.profile;
        let does_nothing = catch
            .child_by_field_name("body")
            .or_else(|| catch.named_child(catch.named_child_count().saturating_sub(1)))
            .is_some_and(|body| {
                body.named_children(&mut body.walk())
                    .all(|child| profile.empty_body_kinds.contains(&kind_of(child)))
            });
        does_nothing
            && !descendants(catch)
                .into_iter()
                .any(|node| profile.is_comment(node))
    }

    /// The call a statement is, whether it is returned or stands on its own. Only the
    /// statement's own value counts: a call buried inside a larger expression means
    /// the body is doing something with the answer.
    fn only_call(&self, statement: Node<'a>) -> Option<Node<'a>> {
        let mut value = self.see_through(statement.named_child(0)?);
        // Go wraps a returned value in a list of one, which is not a step the author
        // wrote.
        while self.profile.expression_list_kinds.contains(&kind_of(value))
            && value.named_child_count() == 1
        {
            value = self.see_through(value.named_child(0)?);
        }
        self.profile.is_call(value).then_some(value)
    }

    fn enclosing_class_name(&self, node: Node<'a>) -> Option<String> {
        let profile = self.profile;
        let mut parent = node.parent();
        while let Some(current) = parent {
            if profile.is_class(current) {
                return self.class_name(current);
            }
            parent = current.parent();
        }
        None
    }

    /// The class a function is a method of: `Some(None)` for a function of its own,
    /// and None for one no call can name, a method of an unnamed class or of an
    /// object literal.
    fn owning_class(&self, function: Node<'a>) -> Option<Option<String>> {
        let profile = self.profile;
        let mut parent = function.parent();
        while let Some(current) = parent {
            if profile.is_class(current) {
                return self.class_name(current).map(Some);
            }
            if profile.is_function(current) {
                return Some(None);
            }
            if kind_of(current) == "object" {
                return None;
            }
            parent = current.parent();
        }
        Some(None)
    }

    fn class_body(&self, class: Node<'a>) -> Option<Node<'a>> {
        let profile = self.profile;
        class.child_by_field_name("body").or_else(|| {
            class
                .named_children(&mut class.walk())
                .find(|child| profile.is_block(*child))
        })
    }

    fn class_name(&self, class: Node<'a>) -> Option<String> {
        let profile = self.profile;
        class
            .child_by_field_name("name")
            .filter(|node| {
                profile.is_identifier(*node)
                    || profile.binding_identifier_kinds.contains(&kind_of(node))
            })
            .map(|node| self.text(node).to_owned())
    }

    /// The marker is an anonymous keyword in most grammars and a decorator in
    /// Python, so the unnamed children count as well as the named ones.
    fn is_static(&self, member: Node<'a>) -> bool {
        let profile = self.profile;
        let holder = member
            .parent()
            .filter(|parent| profile.decorated_kinds.contains(&kind_of(parent)))
            .unwrap_or(member);
        let mut cursor = holder.walk();
        let mut direct: Vec<Node<'a>> = holder.children(&mut cursor).collect();
        let containers: Vec<Node<'a>> = direct
            .iter()
            .copied()
            .filter(|node| profile.modifier_container_kinds.contains(&kind_of(node)))
            .collect();
        for container in containers {
            let mut inner = container.walk();
            direct.extend(container.children(&mut inner));
        }
        direct
            .into_iter()
            .any(|node| profile.static_markers.contains(&self.text(node)))
            || self.annotations(member).iter().any(|annotation| {
                profile
                    .static_markers
                    .iter()
                    .any(|marker| annotation.contains(marker))
            })
    }

    fn enclosing_function(&self, node: Node<'a>) -> Option<Node<'a>> {
        let mut parent = node.parent();
        while let Some(current) = parent {
            if self.profile.is_function(current) {
                return Some(current);
            }
            parent = current.parent();
        }
        None
    }

    /// Whether a name spelled like an imprecise type is used as one. `expect.any(…)`,
    /// a variable called `any` and `from typing import Any` name something else: a
    /// member name never counts, and a plain identifier counts only inside a type
    /// annotation, which is how Python writes its types.
    fn is_in_type_position(&self, node: Node<'a>) -> bool {
        const MEMBER_NAMES: [&str; 3] = [
            "property_identifier",
            "field_identifier",
            "shorthand_property_identifier",
        ];
        if MEMBER_NAMES.contains(&kind_of(node)) {
            return false;
        }
        if kind_of(node) != "identifier" {
            return true;
        }
        let mut current = node.parent();
        for _ in 0..3 {
            let Some(parent) = current else {
                return false;
            };
            let kind = kind_of(parent);
            if kind == "type" || kind.ends_with("_type") || kind.ends_with("type_annotation") {
                return true;
            }
            current = parent.parent();
        }
        false
    }

    /// A field of the loop's header, looking inside a range clause where the grammar
    /// puts one between the loop and its parts.
    fn loop_header(&self, loop_node: Node<'a>, field: &str) -> Option<Node<'a>> {
        loop_node.child_by_field_name(field).or_else(|| {
            loop_node
                .named_children(&mut loop_node.walk())
                .find(|child| kind_of(child) == "range_clause")
                .and_then(|clause| clause.child_by_field_name(field))
        })
    }

    /// Every mention of a name inside one body, in order, and whether the mention
    /// wrote to it. Three rules ask the same question of this and get different
    /// answers out of it.
    fn touches(&self, body: Node<'a>) -> HashMap<&'a str, Vec<Touch>> {
        let profile = self.profile;
        let mut seen: HashMap<&'a str, Vec<Touch>> = HashMap::new();
        for node in descendants(body) {
            let names_something =
                profile.is_identifier(node) || profile.property_kinds.contains(&kind_of(node));
            if !names_something {
                continue;
            }
            // Asked once and passed along: `Node::parent` walks down from the root
            // of the tree to find it, and this loop wanted it five times per node.
            let above = node.parent();
            let declares = above.is_some_and(|parent| {
                crate::tree::is_binding_position_under(
                    parent,
                    node,
                    &[],
                    profile.borrowed_name_kinds,
                    profile.local_declaring_kinds,
                    profile.expression_list_kinds,
                )
            });
            // The left of an assignment is a write even where it declares nothing —
            // and `+=` is a read as well, which is what stops it looking like a
            // second write over a value nobody looked at.
            // A language that writes `a, b = x, y` puts the target inside a list even
            // when there is only one of them.
            let holder = above.filter(|parent| {
                profile.expression_list_kinds.contains(&kind_of(parent))
                    && parent.named_child_count() == 1
            });
            // A parenthesised target such as `(a) = 2` puts a wrapper between the
            // name and the assignment. Climb through those so the write is seen.
            let mut statement = holder.and_then(|list| list.parent()).or(above);
            while statement.is_some_and(|parent| profile.transparent_kinds.contains(&kind_of(parent))) {
                statement = statement.and_then(|parent| parent.parent());
            }
            let target = |kinds: &[&str]| {
                statement.is_some_and(|parent| {
                    kinds.contains(&kind_of(parent))
                        && parent.child_by_field_name("left").is_some_and(|left| {
                            left.id() == node.id()
                                || self.see_through(left).id() == node.id()
                                || Some(left.id()) == holder.map(|l| l.id())
                                || holder.is_some_and(|h| self.see_through(left).id() == h.id())
                        })
                })
            };
            // Some grammars keep the parenthesised target as a chain of wrappers
            // below the `left` field. Walk from the name up to the assignment so a
            // wrapped target is recognised as a write in those too.
            let wrapped_target = |kinds: &[&str]| {
                let mut parent = above;
                while let Some(p) = parent {
                    if kinds.contains(&kind_of(p))
                        && p.child_by_field_name("left").is_some_and(|left| {
                            let mut inner = left;
                            loop {
                                if inner.id() == node.id() {
                                    return true;
                                }
                                let next = self.see_through(inner);
                                if next.id() == inner.id() {
                                    return false;
                                }
                                inner = next;
                            }
                        })
                    {
                        return true;
                    }
                    if !profile.transparent_kinds.contains(&kind_of(p)) {
                        break;
                    }
                    parent = p.parent();
                }
                false
            };
            // A method's name sits in the same field a declaration's does in some
            // grammars, so `rows.add(x)` would read as a write to `add`.
            let reaching = above.is_some_and(|parent| {
                profile.is_call(parent)
                    || profile.is_member_access(parent)
                    || profile.markup_kinds.contains(&kind_of(parent))
            });
            let assigns = target(profile.assignment_kinds)
                || wrapped_target(profile.assignment_kinds);
            let updates = (target(profile.compound_assignment_kinds)
                || wrapped_target(profile.compound_assignment_kinds))
                && operator_of(above.unwrap_or(node), self.source)
                    .is_some_and(|operator| operator.len() > 1 && operator.ends_with('='));
            let wrote = (declares || assigns || updates) && !reaching;
            // A write lands after everything the right-hand side read, even though
            // the name being written is the first thing on the line.
            let at = if assigns || updates {
                statement.map_or(node.end_byte(), |parent| parent.end_byte())
            } else {
                node.start_byte()
            };
            let mut block = body.id();
            let mut parent = above;
            while let Some(current) = parent {
                if current.id() == body.id() {
                    break;
                }
                if profile.is_block(current) {
                    block = current.id();
                    break;
                }
                parent = current.parent();
            }
            seen.entry(self.text(node)).or_default().push(Touch {
                at,
                block,
                wrote,
                read: !wrote || updates,
                declared: declares && !reaching,
            });
        }
        for mentions in seen.values_mut() {
            mentions.sort_unstable_by_key(|touch| touch.at);
        }
        seen
    }

    /// A function's name where it has one, so a finding lands on the identifier a
    /// reader searches for rather than on the whole body.
    fn name_span(&self, function: Node<'a>) -> Span {
        self.function_name(function)
            .map_or_else(|| span_of(function), span_of)
    }

    fn loop_depth(&self) -> u32 {
        self.frames.last().map_or(0, |frame| frame.loops)
    }

    fn in_statement_loop(&self) -> bool {
        self.frames
            .last()
            .is_some_and(|frame| frame.statement_loops > 0)
    }

    fn in_async(&self) -> bool {
        self.frames.last().is_some_and(|frame| frame.is_async)
    }

    fn push_loop(&mut self, counts_as_written: bool) {
        if let Some(frame) = self.frames.last_mut() {
            frame.loops += 1;
            if counts_as_written {
                frame.statement_loops += 1;
            }
        }
    }

    fn pop_loop(&mut self, counted_as_written: bool) {
        if let Some(frame) = self.frames.last_mut() {
            frame.loops = frame.loops.saturating_sub(1);
            if counted_as_written {
                frame.statement_loops = frame.statement_loops.saturating_sub(1);
            }
        }
    }

    fn note(&mut self, into: fn(&mut Hygiene) -> &mut Vec<Spot>, span: Span, detail: String) {
        let count = self.loop_depth().max(1);
        into(&mut self.out).push(Spot {
            span,
            detail,
            count,
        });
    }

    /// `new RegExp(…)` names what it builds in a field of its own, which the callee
    /// walk has no reason to look in.
    fn constructed_name(&self, call: Node<'a>) -> Option<String> {
        let field = self.profile.constructor_field?;
        self.callee_name(call.child_by_field_name(field)?)
    }

    /// What a name was introduced as. The first declaration wins: a later assignment
    /// is the name being used, and `total = total + part` would otherwise erase the
    /// `""` that says what `total` is.
    fn note_declaration(&mut self, node: Node<'a>) {
        let named = node
            .child_by_field_name("name")
            .or_else(|| node.child_by_field_name("declarator"))
            .or_else(|| node.child_by_field_name("left"));
        let Some(named) = named else { return };
        let named = self.see_through(named);
        let name = if self.profile.is_identifier(named) {
            self.text(named).to_owned()
        } else {
            let Some(inner) = descendants(named)
                .into_iter()
                .find(|child| self.profile.is_identifier(*child))
            else {
                return;
            };
            self.text(inner).to_owned()
        };
        let declared = node
            .child_by_field_name("type")
            .or_else(|| {
                node.parent()
                    .and_then(|parent| parent.child_by_field_name("type"))
            })
            .map(|kind| self.text(kind).to_owned())
            .or_else(|| self.declared_type(node).map(str::to_owned))
            .unwrap_or_default();
        let value = node
            .child_by_field_name("value")
            .or_else(|| node.child_by_field_name("right"))
            .map(|value| self.text(value))
            .unwrap_or_default();
        let head: String = value.chars().take(40).collect();
        if self
            .profile
            .perf
            .string_types
            .contains(&declared.trim_start_matches("final ").trim())
        {
            self.text_declared.insert(name.clone());
        }
        self.declared
            .entry(name)
            .or_insert_with(|| format!("{declared} {head}").trim().to_owned());
    }

    fn walk(&mut self, node: Node<'a>) {
        let kind = kind_of(node);
        let profile = self.profile;
        let mut entered_test = false;

        // A function's own braces are not a level of nesting a reader chose; the
        // blocks written inside them are.
        let nests = profile.is_block(node)
            && node
                .parent()
                .is_some_and(|parent| !profile.is_function(parent));
        if profile.is_block(node) {
            self.note_unreachable(node);
        }
        if nests {
            self.block_depth += 1;
        }
        let entered_frame = profile.is_function(node);
        if entered_frame {
            let inherited = if profile.is_loop_callback(node, self.source) {
                self.loop_depth() + 1
            } else {
                0
            };
            self.frames.push(PerfFrame {
                is_async: profile.is_async(node),
                loops: inherited,
                statement_loops: 0,
            });
        }
        let repeats = profile.loop_kinds.contains(&kind);
        if repeats {
            self.push_loop(true);
        }
        let clauses = if profile.comprehension_kinds.contains(&kind) {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .filter(|child| kind_of(child) == "for_in_clause")
                .count()
                .max(1) as u32
        } else {
            0
        };
        for _ in 0..clauses {
            self.push_loop(true);
        }
        if kind == "program"
            || kind == "module"
            || kind == "source_file"
            || kind == "translation_unit"
        {
            self.note_unreachable(node);
            self.note_module_state(node);
        }
        if profile.is_function(node) {
            self.note_flag_parameters(node);
            self.note_unquestioning_name(node);
            self.note_declared_test(node);
            if let Some(name) = self.function_name(node) {
                let text = self.text(name).to_owned();
                self.name_the_body(&text, span_of(name), self.function_body(node));
            }
        }
        if profile.catch_kinds.contains(&kind) && self.swallows_silently(node) {
            self.out.empty_catches.push(Spot {
                span: span_of(node),
                detail: "catch clause".to_owned(),
                count: 1,
            });
        }
        if kind == profile.if_kind || profile.else_clause_kinds.contains(&kind) {
            self.note_condition(node);
        }
        if kind == profile.if_kind {
            self.note_branch_shape(node);
            self.note_assignment_in_condition(node);
            self.note_duplicate_condition(node);
        }
        if profile.switch_kinds.contains(&kind) {
            self.note_switch_width(node);
            self.note_missing_default(node);
            self.note_silent_fallthrough(node);
        }
        if profile.catch_kinds.contains(&kind) {
            self.note_rethrow_only_catch(node);
            self.note_catch_all(node);
        }
        if profile.finally_kinds.contains(&kind) {
            self.note_exit_from_finally(node);
        }
        if kind == "try_statement" {
            self.note_wide_try(node);
            self.note_blanket_try(node);
            self.note_shadowed_handler(node);
        }
        if profile.throw_kinds.contains(&kind) {
            self.note_generic_throw(node);
        }
        self.note_wildcard_import(node);
        if profile.qualified_name_kinds.contains(&kind)
            && self.test_depth == 0
            && let Some(writer) = profile
                .debug_writers
                .iter()
                .find(|writer| **writer == self.text(node).replace("::", "."))
        {
            self.out.debug_output.push(Spot {
                span: span_of(node),
                detail: (*writer).to_owned(),
                count: 1,
            });
        }
        if profile.loop_kinds.contains(&kind) {
            self.note_redundant_jump(node);
        }
        if profile.loop_kinds.contains(&kind) {
            self.note_single_pass_loop(node);
            self.note_mutation_while_iterating(node);
        }
        self.note_production_assert(node);
        if profile.ternary_kinds.contains(&kind) {
            self.note_nested_ternary(node);
        }
        if profile.negation_kinds.contains(&kind) {
            self.note_negated_comparison(node);
        }
        if profile.is_class(node) {
            self.note_class_shape(node);
            self.note_empty_abstraction(node);
            self.note_split_cohesion(node);
            self.note_assign_once_fields(node);
            self.note_lonely_fields(node);
            self.note_many_collaborators(node);
        }
        self.note_declared_name(node);
        self.note_named_shape(node);
        self.note_ambient_dependency(node);
        if profile.is_member_access(node) {
            self.note_private_reach(node);
        }
        if profile.is_function(node) {
            self.note_empty_method(node);
            self.note_loop_capture(node);
            self.note_passthrough_override(node);
            self.note_stores(node);
            self.note_instance_free_method(node);
            self.note_over_logging(node);
            self.note_sequential_await(node);
            self.note_guard_cascade(node);
            self.note_fresh_local_return(node);
            self.note_many_locals(node);
            self.note_working_constructor(node);
            self.note_mixed_return(node);
            self.note_placeholder_body(node);
            self.note_passthrough(node);
            self.note_exits(node);
            self.note_callback_depth(node);
        }
        // The braces too, not only the statement list inside them: a grammar that
        // separates the two emits no list at all when the braces are empty.
        if profile.is_block(node) || profile.structural_block_parents.contains(&kind) {
            self.note_empty_block(node);
        }
        if profile.binary_kinds.contains(&kind) {
            self.note_comparison(node);
            self.note_ambiguous_precedence(node);
            self.note_integer_division(node);
        }
        if kind == "expression_statement" {
            self.note_discarded_expression(node);
        }
        if profile.declarator_kinds.contains(&kind)
            || profile.local_declaring_kinds.contains(&kind)
            || profile.parameter_kinds.contains(&kind)
        {
            self.note_declaration(node);
        }
        if profile.await_kinds.contains(&kind) {
            self.note_await(node);
        }
        if kind == "comparison_operator" {
            self.note_membership_scan(node);
        }
        if kind == "declaration" {
            self.note_declaration(node);
            self.note_regex_declaration(node);
        }
        if profile.assignment_kinds.contains(&kind)
            || profile.local_declaring_kinds.contains(&kind)
            || profile.declarator_kinds.contains(&kind)
        {
            self.note_discarded_error(node);
        }
        if profile.assignment_kinds.contains(&kind) {
            self.note_copy_accumulation(node);
            self.note_self_built_collaborator(node);
        }
        if profile.assignment_kinds.contains(&kind)
            || profile.compound_assignment_kinds.contains(&kind)
        {
            self.note_loop_concatenation(node);
        }
        if profile.is_call(node) {
            self.note_perf_call(node);
            self.note_call(node);
            self.note_sleep(node);
            self.note_nested_continuation(node);
            self.note_library_exit(node);
            self.note_boolean_argument(node);
            if self
                .called_name(node)
                .is_some_and(|name| profile.test.is_test(&name) && self.registers_test(node, &name))
            {
                self.test_depth += 1;
                self.holds_tests = true;
                entered_test = true;
            }
        }
        if profile.is_function(node)
            && self
                .function_name(node)
                .is_some_and(|name| profile.test.names_a_case(self.text(name)))
        {
            self.test_depth += 1;
            entered_test = true;
        }
        if profile.annotation_kinds.contains(&kind)
            && let Some(directive) = profile
                .suppression_directives
                .iter()
                .find(|marker| self.text(node).contains(*marker))
        {
            self.out.suppressions.push(Spot {
                span: span_of(node),
                detail: (*directive).to_owned(),
                count: 1,
            });
        }
        if profile.number_kinds.contains(&kind) {
            self.note_number(node);
        }
        if profile.binary_kinds.contains(&kind) {
            self.note_boundary(node);
        }
        if profile.string_kinds.contains(&kind) {
            let text = self.text(node).trim_matches(['"', '\'', '`']);
            if let Some(language) = foreign_language(text) {
                self.out.mixed_languages.push(Spot {
                    span: span_of(node),
                    detail: language.to_owned(),
                    count: 1,
                });
            }
            if let Some(standin) = crate::vocabulary::placeholder_value(text) {
                self.out.example_values.push(Spot {
                    span: span_of(node),
                    detail: standin.to_owned(),
                    count: 1,
                });
            }
            if self.block_depth >= self.limits.configuration_depth
                && let Some(configuration) = configuration_literal(text)
            {
                self.out.buried_configuration.push(Spot {
                    span: span_of(node),
                    detail: configuration.to_owned(),
                    count: self.block_depth,
                });
            }
        }
        if kind == "return_statement" {
            self.note_many_results(node);
        }
        if kind == "return_statement"
            && let Some(value) = node.named_child(0)
        {
            let operations = self.operations(value);
            if operations > self.limits.expression_operations {
                self.out.obscured_expressions.push(Spot {
                    span: span_of(node),
                    detail: String::new(),
                    count: operations,
                });
            }
        }
        if let Some((_, label)) = profile
            .imprecise_type_kinds
            .iter()
            .find(|(candidate, _)| *candidate == kind)
        {
            let excepted = |cast: Node<'a>| {
                profile
                    .imprecise_type_exceptions
                    .iter()
                    .any(|exception| self.text(cast).trim_end().ends_with(exception))
            };
            // `value as unknown as Row` and `(Row) (Object) value` are one decision
            // written as two casts, so a chain is reported once, at the outermost cast.
            let chained = node.parent().is_some_and(|outer| {
                let cast_value = outer
                    .child_by_field_name("value")
                    .or_else(|| outer.named_child(0));
                kind_of(outer) == kind && cast_value == Some(node) && !excepted(outer)
            });
            if !excepted(node) && !chained {
                self.out.imprecise_types.push(Spot {
                    span: span_of(node),
                    detail: (*label).to_owned(),
                    count: 1,
                });
            }
        }
        if node.named_child_count() == 0
            && profile.imprecise_type_names.contains(&self.text(node))
            && self.is_in_type_position(node)
        {
            self.out.imprecise_types.push(Spot {
                span: span_of(node),
                detail: self.text(node).to_owned(),
                count: 1,
            });
        }
        if profile.assignment_kinds.contains(&kind)
            && let Some(target) = node.child_by_field_name("left")
        {
            let target = if profile.expression_list_kinds.contains(&kind_of(target)) {
                target.named_child(0).unwrap_or(target)
            } else {
                target
            };
            if profile.is_identifier(target) {
                self.out.mutated_parameters.push(Spot {
                    span: span_of(target),
                    detail: self.text(target).to_owned(),
                    count: 1,
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.walk(child);
        }

        for _ in 0..clauses {
            self.pop_loop(true);
        }
        if repeats {
            self.pop_loop(true);
        }
        if entered_frame {
            self.frames.pop();
        }
        if entered_test {
            self.test_depth -= 1;
        }
        if nests {
            self.block_depth -= 1;
        }
    }
}

/// A name an import binds and the file never mentions again. Read from the module
/// facts rather than the scope table: an import binds at file level in every
/// language here, while only some of them give the bound name a node the resolver
/// would recognise as a declaration.
#[deprecated(note = "use `unused_imports_over`, which shares the file's one traversal")]
pub fn unused_imports(
    profile: &'static LanguageProfile,
    source: &str,
    root: Node<'_>,
    markup: bool,
    imports: &[rqx_core::facts::ImportRequest],
) -> Vec<Spot> {
    unused_imports_over(profile, source, &Nodes::of(root), markup, imports)
}

pub fn unused_imports_over(
    profile: &'static LanguageProfile,
    source: &str,
    nodes: &Nodes<'_>,
    markup: bool,
    imports: &[rqx_core::facts::ImportRequest],
) -> Vec<Spot> {
    let brought: Vec<(&rqx_core::facts::ImportRequest, &String)> = imports
        .iter()
        .filter(|request| !request.re_export)
        .flat_map(|request| request.locals.iter().map(move |name| (request, name)))
        .collect();
    // A tree with an error has swallowed some of its identifiers, and every one it
    // swallowed reads here as a name the file never mentions.
    if brought.is_empty() || nodes.root().has_error() {
        return Vec::new();
    }
    let mut mentioned: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for node in nodes.iter() {
        let named = profile.is_identifier(node)
            || profile.binding_identifier_kinds.contains(&kind_of(node))
            || profile.reference_only_kinds.contains(&kind_of(node))
            || profile.property_kinds.contains(&kind_of(node));
        if !named {
            continue;
        }
        let span = span_of(node);
        // The import statement spells the name it binds, so counting it there would
        // make every import look read.
        if imports
            .iter()
            .any(|request| request.span.start <= span.start && span.end <= request.span.end)
        {
            continue;
        }
        mentioned.insert(text_of(node, source));
    }
    let mut spots: Vec<Spot> = brought
        .into_iter()
        .filter(|(_, name)| {
            !name.starts_with('_')
                && !mentioned.contains(name.as_str())
                && !(markup && profile.markup_runtimes.contains(&name.as_str()))
        })
        .map(|(request, name)| Spot {
            span: request.span,
            detail: name.clone(),
            count: 1,
        })
        .collect();
    spots.sort_by_key(|spot| spot.span.start);
    spots
}

/// Only names a function declares. A package, module or class level name may be read
/// by another file, including one a build tag or platform suffix leaves out of this
/// scan, so finding no reference in this file does not show that it is unused.
fn unused_locals(scopes: &Scopes) -> Vec<Spot> {
    let mut spots: Vec<Spot> = scopes
        .of_kind(BindingKind::Local)
        .filter(|binding| scopes.is_local(binding.scope))
        .filter(|binding| binding.references.is_empty() && !binding.name.starts_with('_'))
        .map(|binding| Spot {
            span: binding.span,
            detail: binding.name.clone(),
            count: 1,
        })
        .collect();
    spots.sort_by_key(|spot| spot.span.start);
    spots
}

/// Whitespace-insensitive text, so two branches written the same way compare equal
/// however they are indented.
fn squashed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The waivers alone, for a configuration that asks for no semantic analysis: a
/// developer's `rqx-ignore` has to be honoured whichever rules are switched on.
#[deprecated(note = "use `waivers_over`, which shares the file's one traversal")]
pub fn waivers(
    profile: &'static LanguageProfile,
    source: &str,
    root: Node<'_>,
) -> Vec<rqx_core::suppress::Suppression> {
    waivers_over(profile, source, &Nodes::of(root))
}

pub fn waivers_over(
    profile: &'static LanguageProfile,
    source: &str,
    nodes: &Nodes<'_>,
) -> Vec<rqx_core::suppress::Suppression> {
    nodes
        .iter()
        .filter(|node| profile.is_comment(*node))
        .filter_map(|comment| {
            rqx_core::suppress::Suppression::parse(text_of(comment, source), span_of(comment))
        })
        .collect()
}

#[deprecated(note = "use `collect_over`, which shares the file's one traversal")]
pub fn collect(
    profile: &'static LanguageProfile,
    source: &str,
    root: Node<'_>,
    scopes: &Scopes,
    limits: Limits,
    extra: &rqx_core::config::Performance,
) -> Hygiene {
    collect_over(profile, source, &Nodes::of(root), scopes, limits, extra)
}

pub fn collect_over(
    profile: &'static LanguageProfile,
    source: &str,
    nodes: &Nodes<'_>,
    scopes: &Scopes,
    limits: Limits,
    extra: &rqx_core::config::Performance,
) -> Hygiene {
    let root = nodes.root();
    let mut walker = Walker {
        profile,
        source,
        nodes,
        limits,
        extra,
        out: Hygiene::default(),
        numbers: HashMap::new(),
        boundaries: HashMap::new(),
        conventions: Vec::new(),
        verbs: Vec::new(),
        declared: HashMap::new(),
        text_declared: HashSet::new(),
        frames: vec![PerfFrame::default()],
        test_depth: 0,
        holds_tests: false,
        block_depth: 0,
        asserting_helpers: Helpers::default(),
        local_marker_names: local_marker_names(profile, scopes),
    };
    walker.asserting_helpers = walker.find_asserting_helpers();
    walker.walk(root);
    // Whether the file holds tests is only knowable once it has been read. A test file
    // is a list of registrations, and registering is the whole of what loading it is
    // for. Its setup, mocks and helpers reach for the clock and the environment to
    // arrange a case, which is the test controlling them, not depending on them, and
    // the statements it seeds a database with are fixtures.
    if walker.holds_tests {
        walker.out.ambient_dependencies.clear();
        walker.out.mixed_languages.clear();
    } else {
        walker.note_import_time_work(root);
    }
    walker.out.mixed_languages = once_per_language(std::mem::take(&mut walker.out.mixed_languages));
    // Which names the file shares is only settled once every declaration has been
    // seen, so the functions that hand one back are read afterwards.
    // Which functions answer later is only settled once the file has been read, so
    // the statements that drop one of those answers are found afterwards.
    let awaited: Vec<AsyncFunction> = nodes
        .iter()
        .filter(|node| profile.is_function(*node) && profile.is_async(*node))
        .filter_map(|node| {
            let named = node.child_by_field_name("name")?;
            Some(AsyncFunction {
                name: text_of(named, source).to_owned(),
                class: walker.owning_class(node)?,
            })
        })
        .collect();
    if !awaited.is_empty() {
        let statements: Vec<Node<'_>> = nodes
            .iter()
            .filter(|node| kind_of(node) == "expression_statement")
            .collect();
        for statement in statements {
            walker.note_dropped_promise(statement, &awaited);
        }
    }
    let shared: Vec<String> = walker
        .out
        .module_state
        .iter()
        .map(|spot| spot.detail.clone())
        .collect();
    if !shared.is_empty() {
        for function in nodes.iter().filter(|node| profile.is_function(*node)) {
            walker.note_global_accessor(function, &shared);
        }
    }

    let mut out = std::mem::take(&mut walker.out);
    note_conventions(&scopes.bindings, &mut walker.conventions);

    // A literal used once is usually obvious from context; the same one written three
    // times is a decision somebody will have to change in three places.
    let mut repeated: Vec<(String, Vec<Span>)> = walker
        .numbers
        .into_iter()
        .filter(|(_, spans)| spans.len() as u32 >= limits.magic_number_repeats)
        .collect();
    repeated.sort_by(|a, b| a.1[0].start.cmp(&b.1[0].start));
    for (raw, spans) in repeated {
        out.magic_numbers.push(Spot {
            span: spans[0],
            detail: raw,
            count: spans.len() as u32,
        });
    }

    let mut boundaries: Vec<(String, Vec<Span>)> = walker
        .boundaries
        .into_iter()
        .filter(|(_, spans)| spans.len() as u32 >= limits.boundary_repeats)
        .collect();
    boundaries.sort_by(|a, b| a.1[0].start.cmp(&b.1[0].start));
    for (subject, spans) in boundaries {
        out.boundary_arithmetic.push(Spot {
            span: spans[0],
            detail: subject,
            count: spans.len() as u32,
        });
    }

    // One finding per file, at the first name that broke the majority convention.
    let camel = walker
        .conventions
        .iter()
        .filter(|(style, _)| style == "camelCase")
        .count();
    let snake = walker
        .conventions
        .iter()
        .filter(|(style, _)| style == "snake_case")
        .count();
    if camel > 0 && snake > 0 {
        let minority = if camel < snake {
            "camelCase"
        } else {
            "snake_case"
        };
        if let Some((_, span)) = walker
            .conventions
            .iter()
            .find(|(style, _)| style == minority)
        {
            out.mixed_conventions.push(Spot {
                span: *span,
                detail: minority.to_owned(),
                count: camel.min(snake) as u32,
            });
        }
    }

    // One finding per concept: the point is that a reader meets it twice, not that a
    // particular name is wrong.
    let mut by_object: HashMap<String, Vec<(String, Span)>> = HashMap::new();
    for (verb, object, span) in walker.verbs {
        by_object.entry(object).or_default().push((verb, span));
    }
    let mut inconsistent: Vec<Spot> = by_object
        .into_iter()
        .filter_map(|(object, uses)| {
            let (first_verb, first_span) = uses.first()?.clone();
            let clash = uses
                .iter()
                .find(|(verb, _)| *verb != first_verb && same_family(&first_verb, verb))?;
            Some(Spot {
                span: first_span,
                detail: format!("{first_verb}/{} for {object}", clash.0),
                count: uses.len() as u32,
            })
        })
        .collect();
    inconsistent.sort_by_key(|spot| spot.span.start);
    out.inconsistent_vocabulary = inconsistent;

    out.unused_locals = unused_locals(scopes);
    out.short_names = short_names(scopes, source, limits.short_name_scope_lines);
    out.distant_declarations = distant_declarations(
        profile,
        root,
        scopes,
        source,
        limits.declaration_distance_lines,
    );
    scan_comments(profile, source, nodes, limits, &mut out);

    out.mutated_parameters = reassigned_parameters(
        profile,
        root,
        scopes,
        std::mem::take(&mut out.mutated_parameters),
    );
    out
}

/// One finding per foreign language in a file, at its first string, counting every
/// string written in it. Moving the language out is one change, however many strings
/// it touches.
fn once_per_language(strings: Vec<Spot>) -> Vec<Spot> {
    let mut first: Vec<Spot> = Vec::new();
    for spot in strings {
        match first.iter_mut().find(|seen| seen.detail == spot.detail) {
            Some(seen) => seen.count += spot.count,
            None => first.push(spot),
        }
    }
    first
}

/// Every place a name spelled like a marked runner call (`fit`, `xdescribe`) refers to
/// something the file declares itself. An import is the runner's own name brought in
/// explicitly, so it is left out.
fn local_marker_names(profile: &LanguageProfile, scopes: &Scopes) -> HashSet<(u32, u32)> {
    scopes
        .bindings
        .iter()
        .filter(|binding| {
            binding.kind != BindingKind::Import && profile.test.is_prefix_marked(&binding.name)
        })
        .flat_map(|binding| std::iter::once(binding.span).chain(binding.references.iter().copied()))
        .map(|span| (span.start, span.end))
        .collect()
}

/// One identifier or one line of prose as its separate lowercase words.
fn split_words(text: &str) -> Vec<String> {
    let letters: Vec<char> = text.chars().collect();
    let mut words = Vec::new();
    let mut current = String::new();
    for (at, character) in letters.iter().copied().enumerate() {
        if !character.is_alphanumeric() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        // A run of capitals is one word: `userID` is user and id, not user, i and d.
        let opens_word = character.is_uppercase()
            && !current.is_empty()
            && (letters[at - 1].is_lowercase()
                || letters.get(at + 1).is_some_and(|next| next.is_lowercase()));
        if opens_word {
            words.push(std::mem::take(&mut current));
        }
        current.push(character.to_ascii_lowercase());
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// One mention of a name: where it is, and what it did to the value. A compound
/// assignment is both, which is the case the two rules reading this disagree about.
#[derive(Clone, Copy)]
struct Touch {
    at: usize,
    /// The run of statements the mention sits in. Two writes only overwrite each
    /// other when both always happen, and a write inside a branch does not.
    block: usize,
    wrote: bool,
    read: bool,
    declared: bool,
}
