//! A parenthesised assignment target is still a write (`(a) = 2`), so an earlier
//! write to the same local is dead just as it is without the parentheses.

use rqx_core::Language;
use rqx_core::config::RqxConfig;
use rqx_core::rule::RuleId;

fn dead_store_count(language: Language, display: &str, source: &str) -> usize {
    rqx_lang::analyze_as(language, display, source, &RqxConfig::default())
        .expect("the snippet parses")
        .findings
        .iter()
        .filter(|finding| finding.rule == RuleId::DeadStore)
        .count()
}

#[test]
fn parenthesised_target_is_a_write() {
    let cases: &[(Language, &str, &str)] = &[
        (
            Language::TypeScript,
            "a.ts",
            "export function one(): number {\n  let a = 1;\n  (a) = 2;\n  a = 3;\n  return a;\n}\n",
        ),
        (
            Language::Cpp,
            "a.cpp",
            "int one() {\n  int a = 1;\n  (a) = 2;\n  a = 3;\n  return a;\n}\n",
        ),
    ];
    for &(language, display, source) in cases {
        assert!(
            dead_store_count(language, display, source) >= 1,
            "a parenthesised target should be a write in {display}"
        );
    }
}

#[test]
fn parenthesised_read_is_still_a_read() {
    let source = "export function one(): number {\n  let a = 1;\n  use((a));\n  a = 3;\n  return a;\n}\n";
    assert_eq!(dead_store_count(Language::TypeScript, "a.ts", source), 0);
}
