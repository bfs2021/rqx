# Changelog

Notable changes, newest first, in the shape
[keep a changelog](https://keepachangelog.com/en/1.1.0/) suggests: Added, Changed,
Removed, Fixed. Dates are when the release was tagged.

User-visible changes are recorded under `## Unreleased` in the change that introduces
them. `tools/release.sh` turns that section into the release notes.

## Unreleased

### Fixed

- **`dead-store` recognises a parenthesised assignment target.** `(a) = 2` is a write to
  `a`, so an earlier write to the same local is now reported as dead, as it is without
  the parentheses. A parenthesised read such as `use((a))` still counts as a read.

## 0.5.3 — 2026-11-19

### Changed

- **`--no-mouse` is listed in `--help`, the manual pages and the shell completions.**
  The flag has worked since 0.5.2, and `docs/interface.md` described it, but it was
  hidden from the generated reference.

- **`rqx fix` no longer deletes `suppressed-diagnostic` comments.** Whether an
  `eslint-disable`, `# noqa` or `# type: ignore` is still needed depends on what the
  suppressed tool reports for that line, which rqx cannot check. The rule still reports
  the comment, and `rqx fix --plan` lists it as a decision. The rule's documentation no
  longer marks it as auto-fixable.

- **`unused-parameter` reports a parameter of a public function at `heuristic`
  confidence.** Public means: a Python function at module level whose name has no
  leading underscore, or one listed in `__all__`; an exported TypeScript or JavaScript
  function, or a method that is not `private` or `#private`; a Java method that is not
  `private`; a C++ function other than a `static` free function, one in an unnamed
  namespace, or a lambda; a Go function with a capitalised name. Code outside the
  project can call these, so `rqx fix` does not change their signatures.

### Fixed

- **The JSON `skipped` entries are relative to the scanned directory.** They were
  relative to the directory rqx was started from, so `rqx scan .` printed
  `./app/big.js: ...` and `rqx scan project` printed `project/app/big.js: ...`, which
  matched neither the paths of the findings nor the `skipped_by` paths. The MCP
  server's summary and the language server's scan answer name them the same way.

- **`rqx config` and the configuration schema say that all six languages are enabled by
  default.** They said five.

- **`unreachable-code` no longer reports, and `rqx fix` no longer deletes, code after an
  exit that still runs or still changes the program.** This covers a label that a
  `goto`, `break` or `continue` names (Go, C++, JavaScript, TypeScript, Java); a Python
  `yield` that makes its function a generator; a Python `global` or `nonlocal`
  statement, or the only binding of a name the rest of the function reads; a
  JavaScript or TypeScript `var` or function declaration, which is hoisted; a C++ call
  spelled as a macro or a preprocessor line; and a block that does not parse cleanly,
  such as `ASSERT_DEATH(f(); return, "")`. On klauspost/compress 1.17.9, `fix --write`
  removed the `emitDict:` and `emitMatch:` labels from `s2/encode_all.go` and the
  package stopped compiling.

- **`rqx fix` keeps build constraints, interpreter lines and tool directives.** No fix
  removes a comment that holds `//go:build`, `// +build`, `//go:generate`,
  `//go:embed` or another `//go:` directive, a shebang, an encoding declaration,
  `eslint-*`, `@ts-*`, `/// <reference`, `prettier-ignore`, `istanbul ignore`,
  `c8 ignore`, `noqa`, `type:`, `pylint:`, `pragma:`, `NOLINT`, `nolint`,
  `SuppressWarnings`, `clang-format off` or `on`, or a fall-through marker, or any
  comment above a Go file's `package` clause. The list is `TOOL_DIRECTIVES` in
  `crates/rqx-lang/src/vocabulary.rs`. `//go:build ignore` is no longer reported as a
  `suppressed-diagnostic`.

- **The `unused-parameter` fix updates every call or none.** A private method called
  through `this` (TypeScript, JavaScript, Java, C++) or by its bare name (Java, C++)
  now loses the argument at each call along with the parameter. In 0.5.2 only the
  declaration changed, and `tsc` and `javac` rejected the calls. The fix is withheld
  when the name is mentioned anywhere else outside a comment, such as a call through
  another receiver, an overload, a callback or a string, or when a call passes a
  keyword or spread argument.

- **`unused-parameter` no longer reports parameters whose list is fixed elsewhere.**
  Java and C++ `main`, a Java method with an annotation, a TypeScript method with a
  decorator or `override`, a C++ method declared `virtual`, `override` or `final`, and a
  pytest test function, whose parameters name the fixtures pytest sets up.

- **`rqx fix` checks each edited file before writing it.** An edit is withheld, with the
  reason, when it removes a protected comment, a label or the last jump to one, the
  `yield` that makes a Python function a generator, a Python `global` or `nonlocal`
  statement or a use of a name one declares, or a C++ macro call; when it leaves a Go
  import or local unused; or when it changes the signature of a public function. Each
  finding's edit is checked on its own first, so one refused edit does not hold back
  the rest of the file.

- **The `unused-local` fix leaves a declaration whose value is read by index.** An
  index can raise, and in CPython the unread binding keeps the value alive until the
  function returns, which `functools.lru_cache` relies on. In 0.5.2, `fix --write`
  also removed `full = False` from `functools._lru_cache_wrapper`, which a nested
  function declares `nonlocal`, and `functools.py` stopped compiling.

- **The `unused-local` fix leaves a declaration whose value is read from an attribute
  or property.** `loop = self._loop`, `x = obj.field` and `v = a.b.c` stay in every
  language, because reading a property can run a getter with side effects. The
  finding is still reported; only the deletion is withheld.

- **The `unused-local` fix leaves a declaration whose name appears again in the same
  function.** In pprof's `legacy_profile.go`, a later `case` declares its own `err`,
  so `var err error` was reported as unused although an earlier `case` assigns and
  reads it, and `fix --write` deleted it and broke `go build`. The finding is still
  reported; only the deletion is withheld.

## 0.5.2 — 2026-11-18

### Changed

- **The `/` menu in the session scrolls and lists every match.** It used to show the
  first six, and the others could be reached only by typing more. It now shows up to
  10 rows (fewer in a short terminal) that follow the highlight. `↑` and `↓` still wrap
  at the ends, `pgup` and `pgdn` move by a page, and `home` and `end` jump to the first
  and last row while the menu is open. A bar on the right edge and an `↑ N more` /
  `↓ N more` line under the rows show where the window sits in the list. The
  descriptions keep one column while it scrolls.

- **The mouse wheel scrolls the `/` menu while it is open.** The session asks the
  terminal to report the mouse when the menu opens and gives it back when the menu
  closes, a row is taken, the line runs, or the session ends or crashes, so text can
  be selected as before whenever the menu is shut. While it is open, selecting text
  usually needs `shift` held. Each notch moves the highlight one row. Clicks are
  ignored. `--no-mouse`, which was accepted and ignored, now keeps the session from
  capturing the mouse at all.

- **The session owl moves with what the session is doing.** While a command runs, the
  owl reads: its pupils sweep along a line and jump back to the next, in place of the
  left and right glance. When the first scan finishes, the header and the worst
  findings stay in the redrawn rows above the prompt until the session prints anything
  else, then move into scrollback where they were printed before. Until then the owl
  in the header wakes up, reacts to the health score (a smile at 80 or more, a look
  down from 50 to 79, a worried face below 50), rests on that face and makes a small
  move every 4 to 10 seconds. A key press returns it to its resting face. A scan that
  finishes later (`/rescan`, or the rescan after `/waive` or `/fix --write`) prints its
  summary at once and keeps the busy owl on screen for the same reaction, in the colour
  of the new score, with a one-line result beside it in place of the spinner. Any key
  ends it and is handled as usual. With `--motion off`, or on a window too short for the
  redrawn rows, the owl is still and the header goes straight into scrollback as before.

- **The terminal report sums up skipped files in one line.** For example:
  `skipped 412 files: 398 in foreign submodule design/ (github.com/other-org), 14
  minified · -v lists them`. The report used to list up to ten skipped files. `-v`
  lists every skipped file with its reason, under a line naming the settings that
  bring them back, such as `scan.skip_submodules: none`. The paths are relative to the
  scanned directory, as finding paths are. `docs/configuration.md` describes what a scan leaves out and how
  `ignore.patterns` covers vendored code it does not recognise.

- **Git submodules from another organisation are no longer scanned by default.** The
  scan reads `.gitmodules` in the scanned directory, and at the repository root when a
  subdirectory is scanned. It reduces each submodule's URL and the project's `origin`
  remote to a host and an organisation, the first segment of the path, so GitLab
  subgroups of one group count as one organisation. A submodule on another host or
  under another organisation is not descended into. It is reported once, with its host
  and organisation and the number of source files in it. A relative URL, or a project
  without an `origin`, counts as the project's own and is scanned, and so does a
  submodule that holds the scanned directory. Credentials in a URL are never printed.
  `scan.skip_submodules: none` scans every submodule and `all` leaves out every one.

- **Minified files are no longer analysed by default.** Only a file whose non-blank
  lines are at least half over 300 characters can be skipped as minified, so a
  component that embeds one or two long SVG paths among ordinary lines is analysed.
  Such a file is skipped when its longest line is over 4,000 characters and lines over
  1,000 characters hold at least 80% of it, when it has at least 5 lines averaging
  over 300 characters, or when it has fewer than 5 lines averaging over 1,000
  characters. The default `ignore.patterns` also
  gain `**/*.min.js`, `**/*.min.mjs`, `**/*.min.css` and `**/*.bundle.js`.
  `scan.skip_minified: false` analyses minified files again. A list in `rqx.yaml`
  replaces the default list, so a project that sets its own `ignore.patterns` gets the
  four new patterns only if it adds them.

- **Files `.gitattributes` marks as generated or vendored are no longer analysed.** A
  file with `linguist-generated` or `linguist-vendored` set, alone or as `=true`, is
  skipped and reported as marked in `.gitattributes`. `-linguist-generated`, `!`, and
  `=false` unset the attribute, and a later line overrides an earlier one, as in git.
  The file at the scanned directory and the one at the repository root are read, and
  the nearer one wins. `scan.respect_gitattributes: false` analyses these files again.

- **More directories are pruned by default.** `scan.prune_dirs` also lists
  `vendored`, `charting_library`, `highcharts`, `lightweight-charts`, `tradingview`,
  `jquery`, `bower_components`, `jspm_packages`, `.next`, `.nuxt`, `.svelte-kit`,
  `.turbo`, `.expo`, `storybook-static` and `_next`. The match is on the exact
  directory name, as before. A project that sets its own `scan.prune_dirs` replaces
  the default list, so its scan does not change.

- **`scan.skip_generated` recognises more generator headers.** Besides `@generated`,
  `Code generated by` and Prisma's header, a file is skipped when a comment line in its
  first 4 KB contains `DO NOT EDIT` or `@auto-generated`, starts with `Automatically
  generated`, or says the file or code is, was or has been automatically generated.
  Webpack, esbuild and rollup bundles are skipped too, recognised by the runtime
  definitions those bundlers put at the top of their output. The same words past the
  first 4 KB, outside a comment, or in lower case (`do not edit`) do not count.

### Added

- **The session folds long output.** Output taller than the rows above the prompt
  prints its first rows and a line with the number of rows left out. `ctrl-o` or
  `/open-last` opens the last folded output in a full-height viewer with paging and
  search (`/`, `n`, `N`), and `q` returns to the prompt as it was. See
  `docs/interface.md`.

- **`rqx scan` and `rqx check` page a long report.** On a terminal outside CI, a
  `terminal` report taller than the window goes through `$RQX_PAGER`, `$PAGER` or
  `less -FRX`. `--no-pager` or an empty `RQX_PAGER` prints it directly. Pipes, files,
  document reporters and `--watch` are not paged, and the exit code is the same with
  and without the pager. See `docs/ci.md`.

- **`.rqx/` keeps the cache out of git.** When `.rqx/` has no `.gitignore`, saving the
  scan cache or recording a trend snapshot there writes one listing `cache.bin`,
  `cache.json`, `.cache.bin.rqx-tmp` and `trend.ndjson`. rqx does not change a
  `.gitignore` that already exists. `baseline.json` is left out of the list because it
  is meant to be committed. The `rqx init` template comment that said to add `.rqx/` to
  `.gitignore` now describes this.

- **The JSON summary has a `skipped_by` object.** It counts the skipped files per
  reason: `submodule`, `minified`, `generated`, `gitattributes`, `too-large`,
  `unreadable`, `parse-timeout`, `unsupported` and `walk`. Each value is an object with a `files`
  count, and the `submodule` entry also has `paths`, the skipped submodules relative to
  the scanned directory. The `skipped` list keeps its `"path: reason"` strings. The MCP
  `scan` tool result carries the same object, and the language server's `rqx/report`
  notification carries it as `skippedBy`.

- **`scan.skip_submodules`, `scan.skip_minified` and `scan.respect_gitattributes`
  settings.** `scan.skip_submodules` takes `auto` (the default), `all` or `none`, and
  reads `true` as `all` and `false` as `none`. The other two default to `true`, and
  `false` analyses those files as 0.5.1 did. Both are part of the cache key, so
  changing one makes the next scan analyse every file again.

### Fixed

- **`rqx config validate` shows config paths relative to the working directory on Windows,
  with `/` between the parts.** The working directory and the config file's path can be
  two spellings of one directory there (`RUNNER~1` and `runneradmin`), so the file was
  shown as a long absolute path in the report and in `--reporter json`.

- **The session keeps the backslashes of a Windows path.** A command typed at the
  prompt, such as `/trend C:\work\trend.ndjson`, treated each backslash as an escape and
  read the path as `C:worktrend.ndjson`. On Windows a backslash is now part of the
  word, as in cmd and PowerShell; elsewhere it still escapes the next character.

- **`rqx mcp --http` could be made to use all of a machine's memory.** rmcp 3.2, the
  MCP library behind the server, kept the tool name of every `tools/call` request in
  a map with no size limit, including names that match no tool. A client that could
  reach the endpoint could send requests with distinct tool names until the process
  ran out of memory (RUSTSEC-2026-0461). rqx now uses rmcp 3.11, which looks the tool
  up on each request and keeps no map. The endpoint listens on `127.0.0.1` unless
  `--http` names another address, and a non-loopback address already requires
  `RQX_MCP_TOKEN`, so the exposure was local processes on a loopback server without a
  token, and token holders otherwise. `rqx mcp` over stdio was not affected. The same
  update carries rmcp's fixes for its OAuth and HTTP client code (RUSTSEC-2026-0457
  and RUSTSEC-2026-0462), which rqx does not use.

- **tokio is updated to 1.54.** Versions before 1.53.3 let safe code build an
  `AsyncFd` from a file descriptor it did not own (RUSTSEC-2026-0386). rqx does not
  call `AsyncFd`, so the update clears the advisory without changing behavior.

- **SARIF files read with `--sarif-in` resolve `file://` URIs for network shares.**
  `file://server/share/a.py` is read as the path `//server/share/a.py`. It used to
  be read as the relative path `server/share/a.py`, which could attach a result to
  the wrong file. A drive path such as `file:///C:/work/a.py` is read as before.

- **`self-asserting-test` compares the two sides of a check as wholes.** In 0.5.1 it
  reported any check that held the same call twice anywhere in it, so
  `expect(window(at(8), at(12))).toEqual([{ start: at(8) }])`, where a helper builds
  part of both values, was reported. A check is now reported only when the value
  checked and the value expected are the same expression, or when one is a spread or
  clone of the other (`[...x]`, `{ ...x }`, `structuredClone(x)`, `copy.deepcopy(x)`,
  `x.clone()`). The sides are read per framework: `expect(a).toBe(b)` and other
  matcher chains, `assertThat(a).isEqualTo(b)`, `assertEquals(a, b)` (skipping JUnit
  4's leading message), `require.Equal(t, a, b)`, `EXPECT_EQ(a, b)` and `assert a == b`.
  A negated comparison such as `x !== x` checks for NaN and is not reported. A check
  that compares a literal with itself, such as `assertEquals(1, 1)`, is now reported.
  Identical sides that contain a call, such as `expect(add(1, 1)).toBe(add(1, 1))`,
  `new Money(1)` twice or `await load()` twice, are no longer reported: each side runs
  the code, so the check verifies that it gives the same answer twice. A copy is
  reported only when the original contains no call. `expect(total).toBe(total)`,
  `order.total` against `order.total`, `[...x]` against `x` and `assert x == x` are
  still reported. The fix hint now reads "write the value the code should produce as a
  literal on the expected side".

- **`mutated-while-iterated` follows each language's iteration rules.** In 0.5.1 it
  reported `map.delete(key)` inside `for (const [key] of map)`. The JavaScript
  specification defines that case: a Map or Set entry removed before the walk reaches
  it is skipped. In JavaScript and TypeScript, `delete` and `clear` on the walked
  collection are no longer reported, and `splice`, `push`, `shift` and the other array
  changers still are. Assigning a new value to the walked name is no longer reported in
  JavaScript, TypeScript, Java or Python, because the loop keeps the collection it
  started with. Python's `items += more` changes the list in place and is still
  reported. In Go, `delete(m, k)` during `range` is not reported, as before;
  `s = append(s, x)` is no longer reported, and `s = append(s[:i], s[i+1:]...)`, which
  moves the elements the range reads, still is. Python's `del items[key]` inside
  `for key in items` is now reported. C++ is unchanged.

- **`dropped-promise` reports only calls that reach one of the file's own async
  functions.** In 0.5.1 it matched the last segment of the call's name, so
  `this.eventLoop.stop()` was reported in any file that declared an async `stop`. It
  now reports `f()` when `f` is an async function declared in the file, `this.f()` or
  `self.f()` when `f` is an async method of the enclosing class, and `Type.f()` when
  `f` is an async method of class `Type` in the file. Calls on any other receiver are
  not reported, because the file does not show what that receiver's method returns.

- **A rule reports the same findings whichever other rules are enabled.** The scan
  built the scope resolver and the collectors only when an enabled rule declared the
  `Semantics` or `Modules` analysis, and most rules declared `Syntax` read those
  collectors too. A configuration that switched the `Semantics` and `Modules` rules off
  lost findings of rules that were still on, with exit code 0 and no warning. On cobra 1.10.2,
  disabling every `Semantics` and `Modules` rule and the pattern advisories cut the
  findings of `Syntax` rules from 401 to 108, and `rules.categories: [flow]` reported
  nothing at a health of 100. Each rule now names the facts it reads, and the scan
  builds every fact that an enabled rule or advisory reads. A file in an `overrides`
  subtree also builds what rules elsewhere read from every file, such as its imports
  for the import graph, so a file imported only from that subtree is no longer
  reported as unused. The default configuration already built every fact, and its
  output is unchanged. Configurations that disable rules, set `categories`, or use
  `overrides` can report findings they did not report before.

- **`overrides` apply to findings that need the whole project.** The rules decided
  after the walk, such as `unused-export`, `unused-file`, `wide-fan-in`, `wide-fan-out`,
  `circular-dependency`, `deep-inheritance`, `oversized-module`, `stale-todo`,
  `hotspot` and `single-author-file`, read the root configuration. A subtree that
  disabled one of them, changed its severity or raised its threshold still got its
  findings at the root's settings. On cobra 1.10.2, an override on `doc/**` that
  disabled `unused-export` and set `fan_out: 100` still reported 15 `unused-export`
  and 11 `wide-fan-out` findings there. Every finding is now admitted, and its
  severity set, under the configuration of the file it is reported in, and these
  rules read their thresholds from that file's configuration. A finding that involves
  more than one file, such as a duplicated block or an import cycle, follows the
  configuration of the file it is reported at. A rule enabled only in a subtree now
  also reports there. Only configurations with `overrides` see a change.

- **Editing a custom rule invalidates the cache.** The cache key covered thresholds,
  rule and pattern settings, but not the `rule` entries a project writes. After a
  custom rule's query or message changed, files that had not changed were served from
  the cache with the findings of the old query. The custom rules are now part of the
  key, so the first scan after an edit analyses every file again. Projects without
  custom rules see no change.

- **`empty-catch` no longer reports a catch clause that holds a comment.** In 0.5.1 a
  comment counted as part of an empty body, so
  `catch (error) { /* the default applies */ }` was reported, although the rule's fix
  hint asks for the reason to be written down. A catch clause with a comment anywhere in it is no longer reported in
  JavaScript, TypeScript, Python, Java or C++, including Python's
  `except ValueError:  # optional` with the comment on the `except` line. A clause with
  no comment and an empty body, such as `{}`, `{ ; }` or a bare `pass`, is still
  reported.

- **`focused-test` and `skipped-test` report `fit`, `fdescribe`, `xit` and `xdescribe`
  only where they register a test.** In 0.5.1 any call named `fit` or `xit` was
  reported, including a project's own `fit(width, limit)`. Such a call also marked the
  file as a test file, which turned off `ambient-dependency` and `import-time-work` for
  the whole file. These names now count as a test registration only when the first
  argument is a string or template string, or an argument is a function, and the file
  does not declare the name itself. An imported name, as in
  `import { fit } from "@jest/globals"`, still counts. `it.only`, `test.only`,
  `describe.only`, `it.skip`, `it.todo` and dotted forms such as `fit.each` are
  reported as before.

- **`no-base-case` and `branching-recursion` no longer give a callback the name of the
  variable its caller's result is stored in.** In 0.5.1 a declaration or class field
  passed its name to the first function anywhere in its initialiser. In
  `const unsubscribe = bus.subscribe(() => { unsubscribe(); })` the callback was named
  `unsubscribe` and reported as recursion with no base case, and
  `private stop = bus.on(() => this.stop())` was reported the same way. A function now
  takes the name only when it is the value itself, with or without parentheses. Other
  function findings on such a callback report it as unnamed, at its first line.
  `const spin = (n) => spin(n)` and recursive functions and methods are reported as
  before.

- **rqx no longer overflows the stack at startup on Windows.** On Windows the program
  exited without printing anything, with the status `0xC00000FD`, because the command
  line parser needs more than the 1 MB that Windows gives the main thread. rqx now runs
  on a thread with an 8 MB stack on every platform. Exit codes, the behaviour on a
  closed pipe and the message printed on a panic are unchanged.

## 0.5.1 — 2026-11-18


### Added

- **A finding can carry a `focus`.** It is the exact range that is the problem, inside
  the statement or declaration that `location` covers. `discarded-error` focuses on the
  name that discards the value, `ambient-dependency` and `import-time-work` on the
  call's name without its arguments, `redundant-else` on the `else` keyword, and
  `passthrough-function`, `static-only-class` and `many-collaborators` on the
  declaration's name. `location` is unchanged, so baselines and waivers keep matching.
  In `--reporter json` the key `focus` is present only when a rule set it. In SARIF the
  focus is `physicalLocation.properties.focus`, and `region` is unchanged.

- **`thresholds.test_decisions` sets how many decisions a test body may make before
  `branching-test` reports it.** The default is 2. The key can also be written as
  `rules.branching-test.test_decisions`. An `if` whose arms only fail or skip the test
  (`if got != want { t.Errorf(...) }`) is not counted, and neither is a loop over a table
  of cases written in the test (`for _, tt := range tests`, `for (const row of [...])`) or
  a loop driven by a benchmark's handle (`for i := 0; i < b.N; i++`).

- **`gate.baseline_match`.** Sets how a scan's findings are matched against the
  baseline: `count` (the default) or `set`. With `count`, a baseline entry covers as
  many findings with its identity as the baseline recorded, and any more are new. With
  `set`, an entry covers every finding with its identity, which is how rqx 0.5.0 and
  earlier matched. See `docs/baseline.md`.

- **`fix.min_confidence`.** The lowest finding confidence whose fix `rqx fix` applies:
  `verified` (the default), `heuristic` or `low`. It applies to `rqx fix`, `rqx check
  --fix`, the editor's code actions and the MCP `fix_preview` tool. The preview counts the
  fixes it left out on a `Floor` row, the JSON summary as `below_floor`, and `fix_preview`
  as `below_floor`. See `docs/configuration.md`.

- **A package check in `rqx fix`.** Before writing, `rqx fix` compares what each edited
  file declares before and after the edit. An edit that removes a name, or changes the
  signature of a function, that another file still uses is withheld, and the report names
  the name and the file. The search covers every file the scan read and every file of the
  same language in the edited file's directory, so a Go file excluded by a build tag or a
  `_windows.go` suffix is included. A Go edit that changes any function signature is
  withheld.

- **`rqx fix --verify-build`.** Runs `go vet ./...` on a temporary copy of each Go module
  with an edit, before and after the edits. A module that fails after the edits, or
  already fails before them, has all of its edits withheld. Off by default; needs `go` on
  `PATH`.

### Changed

- **Finding messages name the subject and the numbers.** About a hundred rules have a
  sentence of their own in place of the generic one. `discarded-error` says which call's
  result is assigned to an ignored name, `magic-number` says how often the literal
  appears, `branching-test` and `many-exits` count in the singular when the count is 1,
  and `no-base-case` and `branching-recursion` no longer share one sentence. The sentence
  appears in the terminal, SARIF, GitLab, GitHub, JUnit, Markdown and HTML reports, in
  editor diagnostics and in the MCP tools. The `evidence` object in JSON is unchanged,
  and so are the fingerprints in the GitLab report and in SARIF `partialFingerprints`,
  which still hash the evidence's own text. `--search` matches both texts.

- **Every finding in the terminal has a reason.** The whole rationale for a rule prints
  under its first finding, as before. The later findings of that rule print a one-line
  form of it instead of nothing. The catalogue entry takes an optional `short`; without
  one, the short form is the first clause of `why`. `--detail full` still prints the
  whole rationale every time.

- **Thirty-five rules name a different fix technique.** The earlier pairings were
  mismatches: `discarded-error` suggested Decompose Conditional, `sequential-await`
  suggested Replace Algorithm, `loop-concatenation` suggested Replace Algorithm where
  Accumulate in Place exists, and `many-locals` suggested Extract Variable, which adds a
  local. The `fixes` array in JSON, the technique names in SARIF and GitLab help text,
  and the technique under each finding change with them. Six techniques are new, in the
  existing groups and a new `Errors` group: Handle the Failure, Narrow the Handler,
  Replace Exit with Error, Declare Immutable, Narrow the Type and Share Ownership. A
  test lists, for each technique, the rules it applies to, so a new pairing has to be
  added there on purpose.

- **The terminal marks the focus, not the whole statement.** A finding with a focus
  underlines only that range and shows the rest of its statement as a count of lines.
  A multi-line finding with no focus shows its first line and `⋮ N more lines` instead of
  one row of carets per line.

- **`branching-test` reports at `info` instead of `warning`, and only when a test makes
  more decisions than `thresholds.test_decisions`.** In 0.5.0 a single `if`, loop or
  switch anywhere in a test was reported. Set `rules.branching-test.severity: warning` to
  keep the previous severity, and `thresholds.test_decisions: 0` to report every counted
  decision.

- **`rqx --help` lists the commands before the global options.** The command groups
  come first, then an `Options:` heading with the global options, then the notes, the
  examples and the exit codes. The text of each is unchanged.

- **`-h` lists the common flags of a command, and `--help` lists all of them.** Under
  `-h`, `rqx scan` shows 10 of its 39 flags, such as `--rule`, `--severity`, `--path`
  and `--fail-on`, and its help flag says that `--help` lists every option. `check`,
  `fix`, `verify`, `comment`, `perf`, `plan` and most other analysis commands do the
  same with their own set. The global `--motion`, `--no-progress`, `--progress` and
  `--hyperlinks` are also left out of `-h`. Every flag is still accepted and still in
  `--help`, the man pages and the shell completion scripts.

- **The session's `/` menu offers flags when asked, not after every space.** A command
  followed by a space leaves the menu closed, so `enter` runs the line. Flags are
  listed once a `-` is typed, or when `tab` is pressed. The flags that `-h` shows come
  first, a flag already on the line is left out unless it can be repeated, and `esc`
  keeps the menu closed until the next `-` or `tab`. The values a flag takes are still
  listed right after the flag.

- **`rqx fix` applies only the fixes of `verified` findings by default.** Fixes for
  `heuristic` and `low` findings, such as `banner-comment`, `commented-out-code` and
  `redundant-comment`, are counted and left in place. `fix.min_confidence: low` restores
  the 0.5.0 behaviour. The `autofixable` flag in JSON and editor diagnostics, and the
  findings `rqx fix --plan` lists, follow the same setting.

### Fixed

- **`assertionless-test` recognises the standard assertions of each supported test
  framework.** In 0.5.0 only the first segment of a call was compared against a short
  list, so `t.Errorf`, `self.assertIn` and `EXPECT_NE` were not assertions.
  - Go: `Error`, `Errorf`, `Fatal`, `Fatalf`, `Fail` and `FailNow` on the test's
    `*testing.T`, `*testing.B`, `*testing.F` or `testing.TB`, and on `s.T()`; any call
    that passes that handle on, such as `checkOutput(t, got)`; a subtest passed by name,
    `t.Run(name, tc.test)`; Gomega's `Expect`, `Eventually` and `Consistently`;
    testify's `Fail` and `FailNow`, and suite assertions written through `s.Require()` and
    `s.Assert()`. Benchmarks and fuzz targets are not reported.
  - Python: every unittest `self.assert*` method and `self.fail`, mock's `assert_*`
    methods, `pytest.raises`, `pytest.warns`, `pytest.deprecated_call`, `pytest.fail`, and
    `ThreadingMock`'s `wait_until_called` and `wait_until_any_call_with`.
  - Java: every JUnit `assert*` method and `fail`, AssertJ's `assertThat*`, Mockito's
    `verify*`, and JUnit 4's `@Test(expected = ...)`. A constructor of a class whose name
    starts with `Test` is no longer read as a test.
  - TypeScript and JavaScript: Chai's `should` chains, `sinon.assert`, `t.assert`, `fail`,
    and functions named `expect*` or `assert*`.
  - C++: every GoogleTest and GoogleMock `EXPECT_*` and `ASSERT_*` macro, including
    `ASSERT_THROW` with a type argument; `FAIL`, `ADD_FAILURE` and `GTEST_FAIL`; Catch2
    and doctest `REQUIRE*` and `CHECK*`; Boost.Test `BOOST_CHECK*`, `BOOST_REQUIRE*`,
    `BOOST_TEST`, `BOOST_FAIL` and `BOOST_ERROR`; and `static_assert`.
  - Every language: a call to a function declared in the same file that asserts counts as
    an assertion, if the test calls it bare, through `self` or `this`, or from inside the
    function that declares it.

- **A call that does not pass a function body is no longer analysed as a test
  registration.** In Python, `test.assertEqual(3, 2)` was read as a call to `test(...)`
  registering a test and was reported by `assertionless-test` and `empty-test`.
  `it("name", handler)` in TypeScript and JavaScript was reported the same way.

- **A baseline records how many findings share an identity.** An identity is made of
  the rule, the file and the subject, so two findings of one rule about the same name
  in one file have the same identity. The baseline stored each identity once and
  matched it as a set: a new finding with the same identity as an existing one was
  marked `known` and did not count against the gate. On cobra 1.10.2 the baseline held
  1,267 entries for 1,671 findings. Each entry now has a `count` field, written when it
  is greater than 1. When a file has more findings with an identity than the baseline
  recorded, the extra findings are new; findings are matched in source order, so the
  last ones in the file are reported. A baseline file written by 0.5.0 still loads, and
  each of its entries is read as one finding. **A gate can fail right after upgrading**
  if the baseline was hiding repeated findings. Run `rqx baseline --update` to record
  the counts, or set `gate.baseline_match: set` to restore the previous matching.
  `rqx baseline` now prints the number of findings it recorded instead of the number of
  entries, and `--update` compares finding totals.

- **`oversized-module` and `cross-language-duplicate` findings are matched against the
  baseline.** Findings built after the per-file analysis were matched only when their
  rule was on a fixed list, and these two rules were missing from it, so right after
  `rqx baseline` their findings in unchanged files were reported as new. Every finding
  is now matched in one place, whichever stage built it.

- **`rqx verify` and the resolved findings in `rqx comment` pair repeated findings one
  to one.** When findings shared an identity, every one of them at the base revision
  was paired with the same finding in the working tree. Two findings reduced to one
  were reported as two unchanged instead of one unchanged and one resolved. Each
  finding at the base is now paired with at most one finding in the working tree, in
  source order. `rqx verify` also keys a finding from a SARIF file on its own rule id,
  so findings from two rules of another tool are no longer paired with each other.

- **`rqx fix --write` could delete licence header lines and package-level declarations
  still used by other files.** On cobra v1.10.2 it removed the Apache licence URL line
  from 31 files, deleted `var preExecHookFn` from two build-tagged files that
  `command.go` reads, and deleted result types from Go functions, after which `go vet`
  failed. The summary still said every fix was re-analysed clean. The causes:
  - `banner-comment` matched any comment that was one punctuated word, which included
    URLs, licence lines, encoding declarations and commented-out code. It now reports a
    divider (`// ======`) or a heading of one or two words fenced by one
    (`// --- helpers ---`), and never a comment in a licence or copyright block.
  - No fix removes a comment block that carries licence or copyright text (licence,
    copyright, SPDX, "all rights reserved", Apache, MIT, GPL, BSD, Mozilla and similar
    words), or a comment that holds a URL.
  - `unused-local` reported names declared at package, module, namespace or class level,
    which other files can read. It now reports only names declared inside a function, and
    its fix removes only those. Go package-level variables and C++ globals declared
    `extern` in a header are no longer reported.
  - `unused-parameter` read the last parameter list of a Go function, which is the result
    list when the function returns values. It reported result types as unused parameters,
    and its fix deleted them along with an argument at every call site, which produced
    `completions.go:187: too many return values`. The fix now edits only the list the
    grammar names as the parameters.

- **`dead-store` reported package-level and class-level names.** A call between the two
  writes can read such a name. The rule now judges only names the function declares or
  takes as parameters.

- **A fix that added a second copy of an existing finding passed verification.**
  Verification compared finding identities as a set, so a new finding with the same rule,
  file and subject as an existing one was not seen. It now compares the count of each
  identity.

- **`output-parameter` reports only assignments to a parameter.** It reported any
  assignment whose target had the name of a parameter anywhere in the file, so a local
  declared with `:=`, a class-level annotation in Python, and a local that shared a name
  with another function's parameter were all reported. The target is now resolved
  through the file's scopes and must be a parameter of an enclosing function. A Go
  receiver, a named result in a Go signature and a name inside a function type in a
  signature are not counted, and each parameter is reported once.

- **`unused-parameter` reads only a function's parameter list.** The rule took the
  last parameter list in a signature, which in Go is the result list when the results
  are in parentheses, and in a C++ trailing return type or a TypeScript return type
  can be the list of a function type. Result types such as `ShellCompDirective`, named
  results, and the parameters of a returned function type were reported as unused
  parameters. The list the grammar marks as the parameters is now read, and a
  parameter written as a bare type, such as Go's `func(int)`, has no name to report.
  The `TEST(Suite, Name)` macros of GoogleTest are no longer read as two unused
  parameters either.

- **Python parameters without a type annotation are parameters.** In `def f(x)` the
  name sits directly in the parameter list, and only typed or defaulted parameters
  were recognised, so `unused-parameter` and `output-parameter` did not see most
  Python code. Both now do, which adds findings on Python projects. To keep them to
  parameters a caller chooses, `unused-parameter` leaves out `self` and `cls`,
  `*args` and `**kwargs`, names starting with `_`, dunder methods other than
  `__init__` and `__new__`, functions under a decorator other than `staticmethod` or
  `classmethod`, method bodies that only document or refuse (`...`, `pass`, a
  docstring, a lone `raise`, or a closing `raise NotImplementedError`), and functions
  the file passes or returns by name. `output-parameter` leaves out `self` and `cls`.
  The name in a keyword argument, `limit` in `StreamReader(limit=limit)`, is no longer
  read as a new name, and a parameter's default value is no longer read as a
  parameter.
  `unused-parameter` checks module-level Python functions only, not methods: a base
  class's default method that returns a fixed answer cannot yet be told apart from one
  that ignores its argument.

- **`long-parameter-list` and `flag-argument` read only the parameters.** They used
  the same last list as `unused-parameter`, so a Go function with six named results
  was reported as taking six parameters, and a `found bool` result as a boolean flag
  argument.

- **`discarded-error` reads the error position of a Go assignment with several
  targets.** It looked only at the first target, so `_, err := w.Write(b)`, which keeps
  the error and drops the byte count, was reported. With several targets the rule now
  reads the last one, where Go returns the error, and the message says that the error
  is the result being thrown away. In Go, `var _ io.Writer = (*Sink)(nil)` checks that
  a type satisfies an interface and is no longer reported, and neither is a call to a
  function declared in the same file whose results include no `error`.

- **`ambient-dependency` no longer reports a parameter default or a test file.** A
  default such as `random: () => number = Math.random` is the injection the rule asks
  for, and it was reported. A file that registers tests with `describe`, `it` or `test`
  was checked outside the test bodies, so setup callbacks such as `beforeEach`, mock
  factories and helper functions were reported for reading the clock or the
  environment. Such files are now skipped, as `import-time-work` already skips them.

- **`mixed-language-file` recognises SQL by the shape of a statement.** Any string
  containing a keyword such as `update ` in any case counted as SQL, so an error message
  like `"Failed to update user"` was reported, once for each string. A string now counts
  as SQL when it opens with the statement keyword in capitals and contains the clause
  that statement needs: `SELECT … FROM`, `UPDATE … SET`, `INSERT INTO … VALUES` or
  `… SELECT`, `DELETE FROM`, `WITH … AS`, or `CREATE TABLE`, `INDEX` or `VIEW`. A file
  is reported once per language, at its first such string. Files whose name marks a
  data-access layer (`users.repository.ts`, `OrderDao.java`, `UserRepo.go`,
  `report.sql.ts`, `queries.py`) and files that register tests are not reported.

- **`distant-declaration` and `short-name-wide-scope` count only locals inside a
  body.** Both rules read every local binding, including those declared at the top of
  a module, so a module constant read near the bottom of its file was reported as
  declared far from its use. A binding in the file's own scope is now left out, as the
  `distant-declaration` documentation already said.

- **`distant-declaration` measures from the end of the declaration.** The distance ran
  from the declared name, so a table or a command literal written across sixty lines
  and used on the next line was reported as declared sixty lines before its use. It
  now runs from the end of the declaration.

- **`imprecise-type` reports a chain of casts once.** `value as unknown as Row` is two
  nested casts and produced two findings on one line. A cast whose value is another
  cast of the same kind is now reported once, at the outermost cast. The same applies
  to `(Row) (Object) value` in Java and C++.

- **`imprecise-type` reports `any` only where it is a type.** Any node spelled `any`
  counted, so the matcher `expect.any(Function)`, a variable named `any` in TypeScript
  or Go, and `from typing import Any` in Python were reported. A member name is no
  longer read as a type, and a plain identifier counts only inside a type annotation.

- **`empty-method` no longer reports a constructor that initialises members outside
  its braces.** A C++ constructor with a member initialiser list, `Meter(int factor) :
  factor_(factor) {}`, and a TypeScript constructor with parameter properties,
  `constructor(private readonly label: string) {}`, do their work before the body and
  were reported as empty.

- **`hungarian-notation` no longer reads a constant or an acronym as an interface
  prefix.** `IDLE_TIMEOUT_SECONDS`, a Go field named `ID`, `IOError` and `IPAddress`
  were reported as interface names with an `I` prefix. A name with no lowercase letter
  is now never reported, and the `I` prefix needs a word after it, as in `IOrder`.

- **`never-reassigned` sees a `final` or `const` that follows a parenthesis.** The
  marker was looked for among words separated by spaces, so in `for (final String item
  : items)` and `catch (final IOException error)` it was read as `(final` and the
  already-final variable was reported.

- **`magic-number` leaves out the value of a declaration.** The rule describes itself
  as counting literals outside a declaration, but `const MAX_RETRIES = 3` counted
  toward the repeats of `3`, so naming a number, which is the fix the rule recommends,
  could still produce the finding. A literal that is the whole value of a variable,
  constant or field declaration, optionally with a sign or parentheses, is no longer
  counted.

- **`import-time-work` no longer reports registrations or entry points.** A top-level
  `router.get('/x', handler)`, `app.use(auth)`, `emitter.on('e', () => …)` or
  `atexit.register(flush)` only hands over a handler, and it was reported as work done
  on load. A top-level call is now skipped when one of its arguments is a function
  written in place or declared in the file, or, for a method named like a registration
  (`use`, `get`, `post`, `on`, `command`, `register` and similar), any argument other
  than a plain literal, such as a router or a handler built by a factory call. Files
  configured in `entrypoints.paths` and files named `index`, `main`, `server`, `app`,
  `cli` or `__main__` are skipped, since running top-level statements is what they are
  for. Calls such as `fs.readFileSync(path)` or `connect()` at the top of a module are
  still reported.

- **`no-base-case` and `branching-recursion` count a call as recursion only when it
  reaches the same function.** A Go method that calls the package function it is named
  after, such as `(c *Command) MarkFlagRequired` calling `MarkFlagRequired`, was read
  as calling itself, and so were methods in Python, TypeScript and JavaScript that call
  a module function of the same name. In those languages a method now recurses only
  through its receiver (`self`, `this`, or the Go receiver's own name), which also
  means a Go method that recurses through its receiver is now found. In Java a call
  with a different number of arguments is another overload, and in C++ `::name()` is
  the global function.

- **`closure-over-loop-variable` leaves out closures that run before the loop moves
  on, and Go modules on 1.22 or later.** A function literal passed to a call that runs
  it before returning, such as Go's `t.Run`, `sort.Slice` or `strings.Map`, a
  JavaScript `sort` or `replace`, or Python's `sorted` or `re.sub`, was reported as
  if it ran after the loop had finished. A `t.Run` whose case calls `t.Parallel()` is
  still reported. In a Go module whose `go.mod` declares Go 1.22 or later each
  iteration has its own loop variables, so the rule does not report there at all.

- **`closure-over-loop-variable` reports a JavaScript or TypeScript loop only when its
  variable is shared between passes.** A loop that declares its variable with `let` or
  `const`, as in `for (const row of rows)` or `for (let i = 0; …)`, gives each pass a
  binding of its own, and a closure keeps the value of its pass. Only a `var` loop, or a
  loop over a variable declared before it, is reported now.

- **A line waiver covers only its own line and the next line.** In 0.5.0 an `rqx-ignore`
  comment also covered matching findings on every line above it, so one waiver at the end
  of a file silenced every finding of the rule it named in that file: on cobra 1.10.2, one
  `// rqx-ignore discarded-error` comment at the end of `args_test.go` hid all 36 of that
  file's `discarded-error` findings. Findings that a waiver was hiding this way are now
  reported and count towards the health score and the gate. `rqx-ignore-file` is
  unchanged. A waiver that names something other than a rule, a family, a custom rule or
  another tool's rule (`tool/rule-id`) waives nothing under that name, and is now
  reported as `unjustified-suppression` with the closest rule name when there is one.

- **A policy's locked keys hold against flags and environment variables.** In 0.5.0
  `--fail-on`, `--allow-findings`, `--min-health`, `--min-grade`, `--fail-on-incomplete`,
  `--threads`, `--max-findings`, `--coverage`, `--profile`, `--sarif-in` and
  `rqx hotspots --since`, and the `RQX_*` variables behind them, replaced the value of a
  key the policy locks, so `--fail-on never --min-health 0` passed a gate the policy had
  fixed. Giving a locked key a different value from a flag or a variable now exits 2, and
  the error names the key, the policy file and the flag or variable. Giving the locked
  value is still accepted.

- **`rqx baseline --update` refuses when the count of any rule rises.** It compared only
  the totals, so an update that added findings of one rule and removed more findings of
  another was accepted. The error now lists each rule whose count rose, with the recorded
  and the new count. `--force` still records the new set. For a baseline file written by
  0.5.0, which has no counts, each rule is compared by its number of entries.

- **Findings are reported in the same order on every run.** Two findings of one rule on
  the same span with the same hint, such as two `repeated-computation-in-loop` findings
  on one line of go-redis, could change places between runs and between thread counts.
  Their evidence is now part of the sort key, so the terminal, JSON and SARIF output of a
  scan is the same for any run and any `--threads` value.

- **`oversized-module` and `cross-language-duplicate` findings counted as project-wide.** The
  health projection after `rqx fix` and the resolved credit in `rqx comment` no longer drop
  them when only some files are re-analysed.

## 0.5.0 — 2026-11-18

### Added

- **`rqx perf` estimates the time, space, I/O and allocation cost of each function.**
  The estimate is read from source and nothing is run, so every order is an estimate,
  and each row says how sure it is: `verified`, `heuristic` or `unknown`. A loop is sized
  by what it runs over, a library call is priced from a table of its cost for the
  receiver's type (one table per language, in `crates/rqx-lang/data/cost/`), and a call
  to a function the scan has read costs what that function costs, followed through
  imports and re-exports into other files. A recursion is solved from how its argument
  changes: one call on a smaller argument, two calls with or without a cache, a halved
  argument, a copied slice, and functions that call each other. The table gains `time`,
  `space`, `io`, `alloc` and `conf` columns beside `order`, and `--sort time` and
  `--sort space`. A loop over three items and a loop over a million rows both read as
  `O(n)`. See `docs/perf.md`.
- **`rqx perf --explain FUNCTION`.** Prints how a function's cost was derived: each loop
  and where its bound came from, each call with the callee's cost, the library table
  entry that priced each library call, the recursion shape, and the reasons for the
  confidence. `FUNCTION` is a name, `Class.method` or `file:line`. A name that several
  functions share lists them and exits 2. It works as `/perf --explain` in a session and
  takes `--json`.
- **`rqx perf --json` and the MCP `perf` tool gain keys.** Each row has `time`, `space`,
  `io_order`, `alloc`, `confidence` and `cost` (`expr`, `trace`, `vars`). The existing
  keys are unchanged: `order` is the syntactic loop order and `io` is still the count of
  round trips inside loops.
- **A Performance section in the HTML report.** It ranks up to ten functions with their
  time, space, io and confidence, links each to its file, and holds the derivation in a
  `<details>` element under the row. The page still uses no script.
- **Fifteen performance rules.** `linear-op-in-loop`, `sort-in-loop`, `lock-in-loop`,
  `unbounded-concurrency`, `unbounded-cache`, `defer-in-loop` (Go), `boxing-in-loop`
  (Java), `copy-in-range-for` and `large-by-value` (C++) read one function at a time.
  `quadratic-via-call`, `deep-recursion` and `repeated-computation-in-loop` read the
  cost of a function composed with the functions it calls, across files.
  `allocation-in-loop`, `missing-capacity` and `pair-scan-could-hash` are off by default;
  the `perf` preset switches them on. The rules report as `warning` or `info`, so a gate that fails on
  errors is not affected, but a scan of an existing project can report new findings and
  move its health score. The four cross-file rules need every file of the project and
  are skipped when a scan names single files. See `docs/perf.md`.
- **Complexity regressions in `rqx check` and `rqx comment`.** A function whose time
  order grew against the base revision is reported with its orders before and after, such
  as `reconcileBatch  O(n) → O(n²)`, and the trace that explains it. A caller that nobody
  edited is reported when a function it calls got slower. The base revision is not
  scanned: only the files the change touched are read as they were at the base, and the
  rest reuse the facts of the scan. A renamed function is new, not a regression. The check
  runs when `gate.complexity_regression` is `warning` or `error`, or for one run with the
  new `--regressions` flag; with neither, no output changes. The regressions appear as a
  `Complexity` row in the terminal summary, as a `Complexity regressions` section in the
  merge request comment, as a `regressions` array on the JSON summary line, and as
  `complexity-regression` results in SARIF. See `docs/ci.md`.
- **Complexity settings.** `gate.complexity_regression` (`off`, `warning` or `error`,
  default `off`) sets how a growth in order counts against `rqx check`: `error` fails the
  run and the verdict names the functions. `performance.max_order` sets the largest time
  order a reviewed function may have, and a function above it fails the review; costs
  marked `unknown` are not counted, and it is unset by default. `performance.costs`
  adds library calls to the cost tables. The keys are validated, with the line and column
  of a mistake. See `docs/configuration.md`.
- **`--profile` reads speedscope and JFR profiles.** A speedscope JSON file is detected
  by its `$schema` or its `shared.frames` array, in both its `sampled` and `evented`
  forms. The text from `jfr print --events jdk.ExecutionSample` is read as one sample
  per event, and a frame's class gives its file. async-profiler collapsed stacks with
  Java frames in slash or dot form are matched to the same file, and to the line when
  they carry one. A binary `.jfr` file is skipped with an error that names the
  `jfr print` conversion. See `docs/perf.md`.
- **Measured precision of the cost analysis.** A corpus of annotated functions, written
  for the purpose and copied from five open source projects, is run in `cargo test`
  against a committed baseline that no per-language or per-rule metric may fall below.
  On the 437 written functions the time and space orders were exact for 0.91; on the 31
  functions copied from real projects they were exact for 0.48. The results are in
  `docs/perf-precision.md`, generated by `rqx docs`: precision and recall per language
  for the written and the real functions, and the right, false and missed counts of each
  rule. `rqx explain RULE`, the MCP `explain` tool and the editor rule page print a line
  such as `Measured precision: 0.85 (29 of 34 on the rqx corpus, synthetic + real)` for
  each rule the corpus scores, and the MCP answer gains a `measuredPrecision` object for
  those rules.

### Changed

- **`rqx perf` terminal output.** The column that counted round trips inside loops,
  `io`, is now `trips`, and `io` holds the order of the calls that leave the process.
  `--sort order` ranks by the composed time order, and the default weight grows with it.
  The `order` and `loops` columns and the JSON keys of the same names are unchanged.
- **`accumulate-by-copy` reports declared text at `verified` confidence.** A name
  declared as `String`, `string` or `str` that a loop adds to with `+=` is reported as
  the rule's typed case. The rule id is the same.
- **The `perf` preset covers the new rules.** It reports each of them as an error and
  switches on the three that are off by default. Projects that do not extend `perf` are
  unaffected.
- **The scan cache is rebuilt once after the upgrade.** Cache format 3 holds the cost
  facts of each file, and a cache written by an earlier version is a miss. The cache key
  also covers the library cost tables and `performance.costs`.

### Deprecated

- **The `html` reporter.** It will be removed in 0.6.0. `--reporter html` still writes
  the report and prints a notice on stderr. For a report kept as a pipeline artifact,
  use `json` or `sarif`. `--trend` charts only in the HTML report and goes with it;
  `rqx trend` keeps printing the series.

## 0.4.0 — 2026-11-17

### Added

- **`rqx check --staged`.** Reviews what `git commit` would record. The staged files are
  read from the index and analysed in a temporary copy of the repository, so unstaged
  edits do not change the verdict. Every other file is read from the working tree.
  It cannot be combined with `--base`, `--files-from`, `--fix` or `--write`.
- **A pre-commit hook definition.** `.pre-commit-hooks.yaml` declares `rqx-check`, which
  runs `rqx check --staged` for the [pre-commit](https://pre-commit.com) framework.
  `docs/ci.md` also has the plain git hook, lefthook and husky equivalents.
- **`rqx setup agent`.** Prints, or with `--write` adds, the instructions that make a
  coding agent run `rqx check --staged`, apply the named refactoring and confirm it with
  `rqx verify`. `--for claude|codex|cursor|generic` picks the file: `CLAUDE.md` or
  `AGENTS.md`. The instructions sit between `<!-- rqx:begin -->` and `<!-- rqx:end -->`,
  appended once and replaced on later runs, and the rest of the file is kept. For
  `claude`, `--write` also merges a Stop hook into `.claude/settings.json`.
- **MCP `fix_preview` tool.** Returns the edits `rqx fix` would make to one file as a
  unified diff, with the findings they resolve. An optional `rule` narrows it to one
  rule. It reuses the fixer's planning and verification and never writes a file.
- **`rqx badge` writes the health score as an SVG badge.** The badge reads
  `rqx | health 82 fair`, coloured by the grade's band with the rqx palette, and has no
  external references. `--output PATH` writes a file; without it the SVG goes to standard
  output. `docs/ci.md` has a GitHub Actions workflow that keeps the badge on a branch.
- **`rqx hotspots` ranks files by churn and by what is wrong with them.** The score is
  the commits in the churn window multiplied by the file's weight: the severity weight
  of its findings that count toward the health score, plus one point per ten points of
  cognitive complexity. A column states each factor. `--top` sets the length, `--since`
  the window in days, and `--reporter terminal|json` the output. `/hotspots` runs it in
  the interactive session. See `docs/hotspots.md`.
- **`rqx check` and `rqx comment` report the debt a change paid off.** A finding that
  the base revision has and the change removed is listed in a `Resolved` section of the
  pull request comment and in a `Resolved` row of the terminal summary, with the health
  score before and after. The JSON summary line gains `resolved`, `resolved_scope` and
  `health_delta` keys, absent from `rqx scan` and from `--files-from` reviews. A change
  that adds a function has its base scanned whole (`resolved_scope: "project"`, with a
  health delta). Any other change is credited from the files it touched, read at the
  base with `git show` and analysed on their own (`"changed-files"`, no health delta,
  and no finding that needs more than one file). See `docs/ci.md`.
- **`reimplemented-existing` reports new functions that repeat existing ones.**
  `rqx check` and `rqx comment` scan the revision the change forked from and compare
  the bodies of the functions the change adds with the functions that exist at both
  revisions, using the fingerprints of the clone detector. A match is reported with the
  existing function and its location as the one to call. Confidence is `verified` for
  the same normalised syntax, `heuristic` for the same control flow and constants in one
  language, and `low` for a match across languages. Renamed and moved functions are not
  reported. The base is scanned only when the change adds a function, and it reads the
  working tree's scan cache without writing it. `rqx scan` and `--files-from` do not
  report the rule. See `docs/ci.md`.
- **`--sarif-in` reads other tools' SARIF results.** `rqx scan`, `rqx check` and
  `rqx comment` accept SARIF 2.1.0 files from ruff, Biome, ESLint, Semgrep or any tool
  that writes the format, and report each result as a finding in the new `external`
  family. Rule ids carry the tool name (`eslint/no-unused-vars`). Baselines,
  `gate.fail_on`, the reporters and the pull request comment cover these findings; the
  health score does not. `scan.sarif_in` in `rqx.yaml` sets the files. See
  `docs/ci.md`.
- **Ownership from CODEOWNERS.** rqx reads `.github/CODEOWNERS`, `.gitlab/CODEOWNERS`,
  `CODEOWNERS` or `docs/CODEOWNERS` (the first that holds a rule), in GitHub's format
  or GitLab's, including `[Section]` and `^[Optional]` headers. The last matching rule
  decides. Each JSON finding line gains an `owners` array, and the JSON summary gains an
  `owners` object with `files`, `findings`, `health` and `grade` per owner. Both appear
  only when a CODEOWNERS file exists. See `docs/ownership.md`.
- **`--group-by owner`** on `scan` and `check` lists findings under the owners of their
  file. A file with several owners is listed under each, and a file no rule names is
  listed under `(unowned)`.
- **Health per owner** in the terminal summary, in the HTML report as an Owners section
  after the summary, and in the JSON summary.
- **`rqx owners`** prints the owners ranked by health, as a table or, with
  `--reporter json`, one object per owner. `/owners` prints the same output in the
  session.
- **`rqx comment` names the owners** of files with new findings, with a count each. The
  names are written in code spans, which the forge does not notify.
  `comment.mention_owners: true` in `rqx.yaml` writes them as real mentions.
- **`gate.owners`** sets `min_health` and `fail_on` per owner in `rqx.yaml`. The verdict
  lists each owner that fell short. Both checks are off until set, and the check runs
  only when `gate.owners` is not empty.
- **`--record [PATH]` on `scan` and `check`** appends one NDJSON snapshot to
  `.rqx/trend.ndjson`, or to `PATH`: commit, time, rqx version, health, grade, file
  count, counts by severity, findings per rule and, with a CODEOWNERS file, health per
  owner. The file is only appended to.
- **`rqx trend [PATH]`** prints the snapshots as series for health and findings, each
  with its first and last value, the change, a direction and a block-character
  sparkline, then the rules or, with `--by owner`, the owners that moved most.
  `--last N` limits it to the newest N snapshots, and `--reporter json` writes one
  object per series. `/trend` prints the same output in the session.
- **`--trend PATH` on `scan` and `check`** adds a Trend section with an inline SVG
  chart of the health score to the HTML report. `docs/trends.md` describes the format
  and `docs/ci.md` has a recipe for keeping the file between CI runs.

## 0.3.0 — 2026-11-17

### Added

- **The GitHub Action keeps the scan cache between runs.** A new `cache` input, on by
  default, restores and saves `.rqx/cache.bin` with `actions/cache`. The cache key
  combines the rqx version, the `rqx.yaml` files and the OS, so a pull request analyses
  only the files it changed. `cache: "false"` turns it off. `docs/ci.md` has the GitLab
  `cache:` recipe.
- **`--reporter` on every command that prints.** `fix`, `verify`, `plan`,
  `suppressions`, `complexity`, `perf`, `recommend`, `slop`, `health`, `patterns`,
  `graph`, `config` and `config validate` accept `--reporter terminal|json`.
  `--reporter json=out.json` writes the document to a file while the console keeps the
  readable form. `--json` still works and prints the same output. `graph` also accepts
  `text`, `dot` and `mermaid`, and `--format` is unchanged. As on `scan`, a JSON
  document cannot share stdout with another reporter, and the run reports this before
  writing anything.
- **The gate can be set from the environment.** `RQX_FAIL_ON`, `RQX_ALLOW_FINDINGS`,
  `RQX_MIN_HEALTH`, `RQX_MIN_GRADE` and `RQX_FAIL_ON_INCOMPLETE` correspond to the flags
  of the same names on `scan`, `check` and `comment`. A flag takes precedence over its
  variable, and a variable takes precedence over `rqx.yaml`. As with a flag, setting
  one makes `rqx scan` gate. A variable set to an empty string counts as not set.
- **`rqx check` accepts the scan's options.** `--threads` (or `RQX_THREADS`),
  `--coverage` and `--profile` work as they do on `rqx scan`, and `rqx comment`
  accepts `--coverage` and `--profile`. `rqx check --max-findings N` shortens the
  listing only. The gate counts every finding, so a cap never lets a branch through.
- **`/waive`, `/watch` and `/open` in the session.** `/waive src/a.ts:42 rule` is
  `rqx waive`, and prompts for the reason when `--reason` is missing. `/watch on`
  re-analyses files as they are saved and prints what changed, such as
  `src/a.ts  −1 finding · health 82 → 83`, with `watching` in the status line.
  `/open 2` or `/open src/a.ts:42` opens a finding in `$VISUAL` or `$EDITOR` at its
  line, and the session resumes when the editor exits.
- **`rqx waive FILE:LINE RULE --reason TEXT`.** Writes the `rqx-ignore` comment above
  the line, using the file's indentation and comment style. `--until YYYY-MM-DD` sets
  an expiry date, `--whole-file` waives the rule for the whole file, and `--dry-run`
  prints the diff and changes nothing. An unknown rule is refused with the closest
  known rule, and a blank reason is refused, since the scan would not honour either.
- **Clickable `file:line` in the terminal report.** On iTerm2, WezTerm, VS Code,
  Ghostty, kitty, Windows Terminal and newer VTE terminals, the file and line of each
  finding open in the editor. `--hyperlinks auto|always|never` (or `RQX_HYPERLINKS`)
  overrides the detection, and `RQX_EDITOR_URL`, such as
  `vscode://file/{path}:{line}:{column}`, sets where a click goes. Output to a pipe, a
  file or a report without colour is unchanged.
- **The terminal report wraps its prose to the window.** The evidence, the reason, the
  fix and the cost of a finding break at word boundaries with the same hanging indent
  as before, and never run wider than 100 columns. Only a report written to a terminal
  is wrapped. A file or a pipe receives the same bytes as before.
- **`-v` and `-vv`.** `-v` adds a short block to the end of the terminal report: the
  configuration files in effect, cache hits and misses, and the run time. It also lists
  every skipped file instead of the first ten. `-vv` also prints the stage timings and
  every `git` command the scan ran, on stderr, so a document on stdout stays
  parseable.
- **`--progress auto|bar|plain|none`** (or `RQX_PROGRESS`). `plain` prints a line to
  stderr every two seconds and when the scan moves to its next phase, such as
  `rqx: scanning 1,204/3,456 files · 31 findings · 4.2s`, so a long CI job does not stay
  silent. `auto` keeps the previous behaviour with one change: it also stays quiet under
  CI when stderr is a terminal. `--no-progress` still works and means `none`.
- **`rqx update --version X.Y.Z` and `rqx update --dry-run`.** `--version` installs the
  given release instead of the newest, which also allows moving to an older release.
  `--dry-run` prints how the update would be done, the download URLs and the file it
  would replace, and changes nothing.
- **Man pages.** `man rqx` and one page per command, such as `man rqx-scan`, are
  generated from the same parser as `docs/cli.md` and checked in under `docs/man`.
  Release archives include them in a `man/` directory beside the binary, which stays at
  the archive root, so the installers and the action extract the archive as before.
- **`--reporter html=PATH` writes a report that opens in a browser.** The output is one
  file with its own styles and no scripts or network requests, laid out like an rqx
  terminal session: the owl and the gate's verdict at the top, a summary with the grade
  distribution and the findings by family and by rule as block bars, then every file,
  worst grade first, with each finding's syntax-highlighted code frame, its reason and
  the technique that fixes it, and waived findings listed separately. The page is dark,
  like the CLI, and prints in a light theme. Like `json` and `sarif` it is a document,
  so on stdout it cannot share the stream with another report. `docs/ci.md` has the
  artifact upload for GitHub Actions and GitLab.
- **JavaScript is analysed.** `.js`, `.jsx`, `.mjs` and `.cjs` files are now read, and
  176 of the 185 rules apply to them. This changes what a scan reports: a repository
  that contains JavaScript shows more findings after upgrading, and `rqx check` can
  fail on code it did not read before. JavaScript is read as TypeScript without its
  types, so the eight rules that read type syntax (`imprecise-type`, `flag-argument`,
  `boolean-not-a-question`, `catch-all-handler`, `primitive-obsession`,
  `abstract-nothing`, `speculative-generality` and `inappropriate-static`) and the
  pattern advisories that depend on a declared type do not apply. TypeScript and
  JavaScript share one module graph. An import written `./util.js` now resolves to
  `util.ts`, which also fixes TypeScript projects that use that convention being
  reported as having an unused module. To keep the previous behaviour, list the
  languages in `languages.enabled: [typescript, python, go, java, cpp]` in `rqx.yaml`,
  or skip the files with `ignore.patterns: ["**/*.js"]`.
- **`--timings` reports what the scan did.** Beneath the stage times it now prints
  counts: files read, cache hits and misses, walks of the tree, `git` processes and
  blames. A count is the same on every machine, so a warm run that stopped using the
  cache appears as a changed number and not as timing noise. The stages gain cache
  load, cache save and render.
- **`rqx config validate [PATH] [--strict]`.** Resolves the configuration the way a
  scan does and reports its problems with the file, line and column. An error (bad
  YAML, an unknown key, a key the policy locks, a retired `RQX.toml`) exits 2. A
  warning (a rule name that is not in the catalogue, a threshold that only disabled
  rules read, an `overrides` entry with no `paths`) prints and exits 0, or 1 under
  `--strict`. A scan still refuses unknown names, so a warning here describes a scan
  that would stop. It prints `✓ config is valid (N files)` when nothing stops it.
- **`rqx config --json` and `rqx config validate --json`.** `rqx config --json` writes
  one line per setting (`key`, `value`, `from`, `default`) and a summary line listing
  the files read, so a script can assert on the settings a run will use.
  `config validate --json` writes one line per problem (`severity`, `file`, `line`,
  `column`, `message`) and a summary whose `valid` field matches the exit code. Both
  formats are in `docs/json.md`.
- **`--files-from FILE` on `scan` and `check`.** Names the files to examine, one path
  per line, or separated by NUL when the input contains one, so `git diff -z
  --name-only` works. `-` reads standard input. Paths are relative to the scanned
  directory or absolute, and a path that is not a file under it is reported once on
  stderr and ignored. On `scan` it narrows the report the way `--since` and `--changed`
  do, and combines with them. The whole project is still analysed, so cross-file
  findings remain correct. On `check` the list replaces the git diff as the files under
  review (files that import them are still added), so a review works in a clone with no
  history to diff against. `--base` and `--files-from` cannot be given together.

### Changed

- **Rule explanations and fix hints were reworded.** The `why` text of all 185 rules,
  the `metric` text of some, the fix hints of many and the pattern advisory text now
  use a plainer, more technical wording. The new text appears in `rqx explain`,
  `docs/rules`, the terminal report, `docs/rqx.schema.json` and the `why`, `metric` and
  `fix_hint` fields of the JSON, SARIF, GitLab, JUnit and GitHub reports. Field names,
  rule ids, severities, locations, counts and evidence text are unchanged, so finding
  identities and GitLab fingerprints do not change.
- **Help text and runtime messages use the YAML configuration syntax.** The `rqx query`
  help and the notices about generated files, a disabled cache, an empty scan, a policy
  lock and custom rules name keys as they are written in `rqx.yaml`, such as
  `scan.skip_generated: false`, instead of the retired TOML form.
- **`rqx scan` always prints the report.** In a terminal it previously opened the
  interactive interface, which dropped `--severity`, `--not-*`, `--confidence`,
  `--fixable`, `--search`, `--since`, `--changed`, `--group-by`, `--sort`, `--top`,
  `--detail` and `--context` without a message. They now apply there as they do
  everywhere else. `--no-tui`, `--no-mouse` and `--motion` are still accepted.
- **Behaviour change: bare `rqx` outside a terminal scans the working directory and
  exits with the scan's code.** It previously printed help and exited 0, which made a
  misconfigured CI job appear to pass. When piped, redirected, under CI, on a dumb
  terminal or with `--no-tui`, `rqx` now means `rqx scan`, including the defaults set
  by `RQX_LEVEL` and `RQX_THREADS`. In a terminal it still opens the interface.
- **`rqx update` downloads the release instead of building it.** A binary installed
  with `install.sh`, `install.ps1` or from a release page is now replaced by the new
  release's binary, which takes seconds where a build took about a minute. The archive
  is checked against the release's `SHA256SUMS` first. If the checksum is missing or
  does not match, nothing is installed and the exit code is 2. Installs under
  `~/.cargo/bin` (or `$CARGO_HOME/bin`) are still rebuilt with `cargo install`, and
  installs marked `RQX_MANAGED` are still handed to their package manager. Downloading
  requires `curl` and `tar`, which Windows 10 and later include.
- **Import resolution is faster.** Resolving an import now checks the files the scan
  already walked instead of querying the filesystem for each candidate, and a
  specifier that many files in one directory write is resolved once. The graph and
  every finding are unchanged.
- **Per-file analysis reads the syntax tree once for its hygiene checks.** Twelve checks
  each walked the whole tree and built their own list of its nodes. They now share one
  list per file. Findings are unchanged.
- **rqx uses the mimalloc allocator.** A scan allocates many small short-lived values
  on every worker thread, and mimalloc handles that with about 4% less processor time
  and about 3% more peak memory on a synthetic 4,400-file corpus. It is the `mimalloc`
  cargo feature, on by default. `cargo install rqx --no-default-features` builds with
  the system allocator.
- **Bare `rqx` opens an inline session instead of the full-screen interface.** It
  scans the working directory, prints a header and the worst findings into the
  terminal's own scrollback, and keeps a prompt at the bottom. The session does not take
  over the screen or capture the mouse, so selecting and scrolling work as usual and the
  printed output remains after exit. It needs a window of at least 50x14. A smaller
  window, a pipe, CI or `--no-tui` gets the plain scan. `--no-mouse` is still accepted
  and has no effect.
- **The terminal library is built with only the features rqx uses.** `ratatui` was
  built with its calendar widget, its macros and a layout cache, none of which rqx uses,
  and with the `time` crates the calendar needs. Building rqx now compiles 9 fewer
  crates (255 to 246), and `termwiz` and `terminfo`, the one dependency under the WTFPL
  licence, no longer appear in `Cargo.lock`.
- **A scan starts reading files immediately.** Counting the files for the progress bar
  used to finish before the first file was opened. It now runs alongside the scan, so
  the bar shows no percentage until the count arrives and the scan no longer waits for
  it.
- **The cache is under a quarter of the size.** It was one JSON document of 37 MB on a
  4,440-file tree. Each file's entry is now compressed separately, and the cache is
  8.4 MB, in `.rqx/cache.bin`. A cache from an earlier version is not read, so the
  first scan after upgrading is a cold one, and it removes the old `.rqx/cache.json`.
  A warm scan decodes only the entries it uses, on the threads that walk the tree,
  instead of reading the whole document before the first file is opened.
- **A warm scan does not open unchanged files.** The cache now records each file's
  length and modification time, and a file that still has both is served from the
  cache without being read. On a 4,440-file tree a warm scan reads no files where it
  previously read all of them. A file edited in the two seconds before the cache was
  written is still read and compared by content, because an edit that keeps the length
  cannot be distinguished by its time. Baselines are applied to cached files without
  their text. `scan.skip_generated` now counts as a setting for the cache, so changing
  it rebuilds the cache.
- **Blame does not run when `stale-todo` is off, and it no longer waits in batches.**
  `git blame` costs one process per file that contains a note. A project that had
  switched `stale-todo` off still paid for it and discarded the result. When the rule is
  on, blames now run eight at a time as each one finishes, instead of in groups of eight
  where one slow file delayed the other seven.
- **Each file's line index is built once.** Placing a project-wide finding rebuilt the
  line index of its file every time, so a module with dozens of dead exports built the
  same index dozens of times.
- **The stages after the walk run in parallel.** Duplicate detection, the import graph
  and git history no longer wait for one another, so the 250 ms a scan spent reading
  history on a 4,440-file tree now overlaps the import graph. Results are unchanged and
  come out in the same order. With `--timings`, the stage times show what the scan was
  waiting on at that point, so a stage that finished while an earlier one was still
  running shows no time of its own.
- **`--group-by rule|family|severity` opens with the same header as the default
  listing.** The directory, the file count, the finding count and the health score now
  head every arrangement instead of only the one grouped by file.

### Removed

- **The full-screen interface.** The tabbed screens, the command palette and the
  mouse handling are removed, replaced by the inline session that bare `rqx` opens,
  which prints into the terminal's scrollback instead of taking over the window.
  Nothing a pipeline reads changes: `rqx scan` and every other command printed their
  reports without it. `--no-mouse` is still accepted, hidden from `--help`, and has no
  effect.

### Fixed

- **The GitHub Action no longer fails when `config` is left blank.** It passed its
  inputs to rqx in `RQX_CONFIG` and `RQX_FAIL_ON`, so a blank `config` arrived as an
  `RQX_CONFIG` that pointed at nothing, and rqx refused to run. The inputs now use
  names rqx does not read.
- **`scan --watch` no longer writes a clear-screen sequence into a file or a pipe.**
  It cleared the screen before every run regardless of where stdout went, so a watch
  redirected to a log filled it with escape characters. It now clears only on a
  terminal, and otherwise prints a `── rescanned after HH:MM:SS ──` line between
  reports.
- **Windows reports no longer print `//?/C:/…` paths.** Scanning a subdirectory made
  the SARIF, GitLab, HTML, Markdown and terminal reports name files with the `\\?\`
  prefix that Windows adds when it resolves a path, because the working directory never
  carries it. Every path rqx resolves now drops that prefix first.
- **Finding paths use `/` on Windows.** A file in a nested directory was reported as
  `domain\billing\invoice.ts`, so an architecture glob such as `domain/**` never
  matched it, and the same scan printed different paths on Windows and elsewhere.
  Paths in findings, reports, glob matches and the cache are now always written with
  `/`.
- **`overrides` apply however the files are reached.** An override's `paths` were
  compared with the path as the scan wrote it, so they missed when `rqx scan` was given
  a directory other than the one holding `rqx.yaml`, when it ran from inside the
  subtree, and when the project was reached through a symlink (macOS temporary
  directories are symlinks). The path is now resolved first and matched relative to the
  config file's directory, which `docs/configuration.md` now states. `rqx verify`
  rebases the same way for a relative scan root.

## 0.2.1 — 2026-11-17

### Added

- **One line installs rqx.** `curl -fsSL …/install.sh | sh` on macOS and Linux, and
  `irm …/install.ps1 | iex` on Windows, download the release binary for the machine,
  check it against `SHA256SUMS` and install it into `~/.local/bin` or
  `%LOCALAPPDATA%\rqx\bin`. Neither uses sudo or edits shell files. `--version`
  pins a release and `--dry-run` shows every step without changing anything.
- **`--no-progress` turns the progress bar off, and `--quiet` does too.** The bar
  already stayed away from anything but a terminal; on a terminal there was no way to
  silence it short of redirecting stderr. `--no-progress` (or `RQX_NO_PROGRESS=1`) is a
  global flag, and `-q` on `scan` and `check` now implies it.
- **JUnit and GitHub annotations carry the gate.** A gated run adds a
  `<testsuite name="rqx gate">` to the JUnit report with one test that fails when the
  gate does, and ends the GitHub annotations with `::error title=rqx gate::` and the
  failures, or `::notice title=rqx gate::gate passed`. A provider that shows only
  tests or only annotations no longer has to infer a failed gate from a red exit
  code. A run with no gate writes exactly what it wrote before.
- **The SARIF run records its verdict.** `runs[0].invocations[0]` carries the exit
  code the run ends with and one error notification per gate failure, and
  `runs[0].properties.rqx` carries `schema_version` and, for a gated run, the gate. A
  viewer that only has the document can now tell a failed gate from a clean run. Two
  more things are available and off by default, because they change how GitHub
  matches the alerts it already holds: `RQX_SARIF_FINGERPRINTS=1` adds
  `partialFingerprints`, and `RQX_SARIF_CATEGORY=<id>` adds `automationDetails`.
  `docs/ci.md` says what each one does to existing alerts.
- **A gated scan says whether it passed.** The JSON summary line gains
  `gate: { "passed": bool, "failures": [...] }` whenever there is a verdict: a scan
  with a gate flag such as `--fail-on` or `--min-health`, `check`, and `comment`. Before, a pipeline reading the JSON had to rebuild the gate from the
  counts, or trust the exit code alone. `review` is unchanged.
- **Every JSON line says what it is.** Finding lines in `--reporter json` now carry
  `"kind": "finding"`, so a consumer switches on `kind` for every line instead of
  telling findings apart by the keys they lack. The summary line gains
  `schema_version` (`1`) and `rqx_version`, and the summaries of `fix --json`,
  `verify --json` and `plan --json` gain `schema_version`. `docs/json.md` is new and
  states the promise: keys are never removed or renamed, and a consumer ignores keys
  it does not know. The camelCase keys in `verify` and `plan` stay as they are.

### Fixed

- **The action's `findings` output counts what was reported.** It counted every
  `"ruleId"` in the SARIF file, and SARIF lists a finding a waiver silenced too, marked
  suppressed, so a project that waived findings saw a number larger than the report
  showed. The count now leaves suppressed results out and needs `jq`, which the
  GitHub-hosted runners have.
- **`check` counts every finding, not the 500 kept for display.** The review gate
  counted the findings retained under `max_findings` (500 by default), so a branch
  with more findings than that was counted as if it had 500, and a limit above 500
  could never fail. `[gate] max_findings`, `--allow-findings` and `fail_on` now
  count from a per-file tally taken before the cap, the way `rqx scan` already did.
  A config such as `[gate] max_findings = 600` that used to pass `rqx check` on a large
  change now fails when the change holds more than 600.
- **`fail_on = "never"` no longer switches off the health and grade floors.** A
  project that set `fail_on = "never"` together with `min_health` or `min_grade` got a
  gate that enforced neither in `rqx check` and `rqx comment`, and not the health floor
  in `rqx scan`: the setting meant "no finding fails the run by severity" and was read
  as "no gate". `min_health` and `min_grade` now fail a run regardless of `fail_on`. A
  config with `fail_on = "never"` and a floor that used to pass because of this now
  fails when the score or a file's grade is below it.
- **`check` and `comment` honour `ratchet` and `fail_on_incomplete`.** Both settings
  were read only by `rqx scan`, so a pipeline that gated merge requests with
  `rqx check` or `rqx comment` passed branches that grew the baseline's total or left
  a design pattern half applied, however the gate was configured. A project with
  `[gate] ratchet = true` or `fail_on_incomplete = true` that used to pass `check` now
  fails when the total outgrows the baseline, or when a half applied pattern sits in a
  file the review covers. The ratchet compares the project's total, because the
  baseline records one number; the pattern check looks only at the reviewed files.
- **Every `--group-by` ends with the summary and the gate.** Grouping by rule, family
  or severity stopped after the last group, so a gated run printed no summary and no
  `gate failed` line and the exit code was the only sign. `--stats` printed no gate
  either. All of them now end the way the per-file listing does: cycles,
  architecture, summary, hotspots, skipped files, notes, gate.
- **A document on stdout is the only thing on stdout.** `--timings`, `scan --advice`
  and the fix report of `check --fix` wrote their lines to stdout beside the JSON,
  SARIF, JUnit, GitLab or Markdown document, so a pipeline that parsed the output
  failed on the first extra line. They now go to stderr whenever a document is
  printed to stdout; the terminal and GitHub reports keep them where they were.
- **A cache hit is kept.** After one file changed, the cache was rewritten holding
  only that file, so the next run analysed everything else again, and the run after
  paid the same price for the next edit. The cache now keeps every file the run saw,
  hit or miss, and a run that hit every file no longer rewrites it.
- **A blame that crashed no longer shifts the dates of every file after it.** The
  answers were collected without the crashed one, so the files after it picked up the
  neighbouring file's blame: `TODO` ages and waiver ages could land on the wrong
  file. A file whose blame failed now reports no dates and the rest stay in place.
- **A comma or colon no longer cuts a GitHub annotation short.** A path or title with
  a `,` or `:` in it ended the annotation's properties early, so GitHub dropped the
  line number or pinned the warning to the wrong file. Properties now escape both, and
  a rule the project wrote is titled with its own name instead of `custom`.

## 0.2.0 — 2026-11-17

### Added

- **Prebuilt binaries on every release.** Each GitHub release now carries a binary
  for macOS, Linux and Windows on x64 and arm64, with a `SHA256SUMS` file beside them.
  Installing needs no compiler and no Rust toolchain, so a CI job that spent minutes
  building rqx now downloads it in seconds. The Linux build is static musl: one binary
  runs on Alpine, Debian and RHEL with no glibc floor. An npm package wrapping the
  binaries is built and dry-run by every release but not published yet; setting the
  repository variable `PUBLISH_NPM` to `true` turns publishing on.
- **Configuration is YAML.** `rqx init` writes `rqx.yaml`, and rqx also reads
  `.rqx.yml`, `rqx.yml` and `.rqx.yaml`; two in one directory is an error that names
  both. The keys and structure are the ones `RQX.toml` had, with each table now a
  mapping, `[[override]]` written as an `overrides:` list, and the user config, policy
  files and `--config` all YAML. A mistake in a file is reported as `rqx.yaml:12:3`,
  and a misspelt key lists the keys it could have been.
- **`npm install --save-dev @rqx/cli`.** Every release now carries a prebuilt binary
  for macOS, Linux and Windows on x64 and arm64, behind an npm package that resolves
  the right one at install time. Nothing to compile and no Rust toolchain: a CI job
  that spent minutes building rqx now spends seconds downloading it. The Linux build
  is static musl, so one binary covers Alpine, Debian and RHEL with no glibc floor.
  The tarballs are on the GitHub release with a `SHA256SUMS` beside them, and the npm
  packages publish with Sigstore provenance.

- **`rqx config explain KEY`.** Prints a setting's description, type, default and
  current value, and the file or preset that set it: `rqx config explain
  rules.long-function.severity`. It reads the same schema an editor does, so the
  words match the hover text, and a misspelt key is answered with the closest real one.
- **Per-rule options.** A rule's slug is a key under `rules`, with `severity`,
  `enabled`, `paths`, `confidence` and the thresholds that rule reads:
  `rules: {long-parameter-list: {severity: error, max_params: 6, paths: ["src/domain/**"]}}`.
  The same keys work inside an entry of `overrides`. A threshold written under a rule
  is moved into `thresholds` as the file is read, so layering, policy locks and the
  `from` column treat it as the setting it is. A key under `rules` that is not a rule,
  a list name or an option of that rule is refused with the closest name; the JSON
  Schema lists every rule and its own thresholds. The `perf` preset now reports every
  performance rule as an error.
- **Presets and `extends`.** `extends: [legacy-adoption, ci-gate]` builds a config on
  named sets of settings: `recommended`, `strict`, `legacy-adoption` (relaxed
  thresholds, a gate that fails only on errors, the ratchet on), `ci-gate` (no errors,
  health of at least 70, no critical file) and `perf`. An entry may also be a path to
  another YAML file, relative to the one naming it. Everything extended goes under
  every file rqx reads, so a preset never overrides a policy or a project's own key,
  and `rqx config` shows `preset:ci-gate` in the `from` column for a value it set.
  See `docs/presets.md`.
- **A JSON Schema for the config.** `rqx config schema` prints it and
  `docs/rqx.schema.json` is the checked-in copy, regenerated by `rqx docs`. It is
  derived from the config types, so it lists every key, its default and its
  description, and refuses the keys rqx would. `rqx init` writes a
  `# yaml-language-server: $schema=` line at the top of `rqx.yaml`, which is enough
  for VS Code's YAML extension and others to complete and validate it.
- **`rqx config migrate [PATH] [--dry-run]`.** Converts every `RQX.toml` under a
  directory, the policy files they point at, and the user config to YAML. Comments
  above keys and sections come across, `[[override]]` becomes `overrides:`, and a
  policy pointer is rewritten to the new file. The original is kept as `RQX.toml.bak`,
  and the YAML is parsed back and compared with the TOML before anything is written,
  so a conversion that would change a value writes nothing.

### Removed

- **TOML configuration.** rqx no longer reads `RQX.toml` or `rqx.toml`, in a
  directory, as the user config, as a policy, or through `--config` and `RQX_CONFIG`.
  Finding one stops the run with exit 2 and a message saying so, because ignoring it
  could quietly loosen a CI gate. `rqx config migrate` converts it.

### Changed

- **The action downloads rqx rather than building it.** `jadhamwi21/rqx@v0.2.0` reads the
- **`rqx config set` refuses a value the config would refuse.** It edits the YAML in
  place, keeping comments and layout, and checks the result before writing, so
  `rqx config set thresholds.max_params many` leaves the file as it was and says why.
  Per-rule keys such as `rules.long-function.severity` work like any other.
- **The action downloads rqx rather than building it.** `jadhamwi21/rqx@v1` reads the
  version out of the checkout, fetches that release's binary for the runner and
  verifies it against `SHA256SUMS`. A ref with no published binary — a fork, or
  `@main` ahead of the last release — still compiles from the checkout as before.
- **`rqx update` hands a managed install back to its manager.** An install marked
  with `RQX_MANAGED` gets the release notes and a pointer to the package manager that
  owns it. Building over that file would leave a second binary on the path.
- The published binary is built with fat LTO, one codegen unit and no symbols.
  Nobody waits for a release build, so it takes the opposite side of the trade
  `Cargo.toml` makes for source installs — smaller to download and faster to run.
- **A clean install takes 68 seconds rather than 228.** `codegen-units = 1` compiled
  the binary crate as one unit no core could share — 183 of those 228 seconds, all
  of it serialised at the end. Sixteen units cost 2.4% of scan time and 4MB of
  binary, measured on a 3,456-file corpus, with findings byte-identical.

## 0.1.1 — 2026-09-06

### Added

- **`rqx update`.** rqx publishes no binaries; a tag is the release, and updating
  builds it the same way `cargo install --git` installed it. A scan prints one dim
  line when a newer version has been tagged, with the changelog behind
  `rqx update --notes`. The check runs on a thread nothing waits for, caches for a
  day, and is skipped under CI, off a terminal, with `RQX_NO_UPDATE_CHECK`, or with
  `[update] check = false`.

### Fixed

- **The update check could print into a build log, and never completed.** Its gate
  asked whether the reporter was a coloured kind — true for the default reporter
  piped into a log — and leaned on `CI`, which Jenkins, TeamCity and Azure Pipelines
  do not reliably set. It now requires stdout and stderr to be terminals and names
  those systems too. Separately the worker was spawned after the report and
  abandoned, but `process::exit` does not wait for a thread, so `curl` was killed
  before answering; the check now starts with the scan and is collected at the end.
- crates.io publishing skips with a notice when no token is configured, rather than
  failing the last job of a release that had already published.

### Removed

- **The published binaries.** No tarballs, no `SHA256SUMS`, no five-target build
  matrix — installing and updating both build from source, which is one thing that
  can go wrong instead of five platform artifacts and the pipeline that uploads
  them. The GitHub Action builds from the checked-out commit and caches it.
- **The editor extensions.** The VS Code, Zed and JetBrains clients are gone, along
  with the marketplace publishing that went with them. Every editor rqx supports now
  talks to `rqx lsp` through stock LSP configuration, which `rqx setup <editor>`
  prints — nine of them, none needing anything installed beyond the binary.

## 0.1.0 — 2026-09-06

### Added

- **Baseline and ratchet.** `rqx baseline` records the debt a project already has;
  `[gate] baseline` counts only findings it does not hold, and `[gate] ratchet`
  refuses to let the total grow. [docs](docs/baseline.md)
- **Waivers.** `// rqx-ignore <rule>: <reason>` waives a finding, and the reason is
  not optional — without one the finding stands and `unjustified-suppression`
  reports the comment. `rqx suppressions` audits them. [docs](docs/suppressions.md)
- **Rules you write.** `[[rule]]` blocks holding tree-sitter queries, with
  `rqx query` to try one first. [docs](docs/custom-rules.md)
- **Policy.** `policy = "…"` reads a config from outside the repository, layered
  under the project so a service may still tighten it, with `[locked] keys` naming
  what it may not change. [docs](docs/policy.md)
- **`rqx config set/unset/files`** — change a setting from the command line, in the
  right file, without losing the comments.
- **`rqx complexity`**, **`rqx health`**, **`rqx slop`**, **`rqx recommend`**,
  **`rqx graph`**, **`rqx cache`**, **`rqx completions`**.
- **The `noise` family** — five rules for text that costs a reader and buys nothing.
- `--stats` for counts without findings, `--timings` for where a scan spent itself.
- Four more MCP tools: `complexity`, `health`, `graph`, `recommend`.
- **The interface takes a mouse.** The wheel scrolls, a click selects, a second
  click opens, and the tab bar and key hints are buttons. `--no-mouse` turns
  capture off where drag-select matters more. [docs](docs/interface.md)
- **A command palette** on `ctrl-p`: every tab, command, file and rule, narrowing
  as you type.
- **A menu that holds every command**, in five groups — Analyse, Browse, Report,
  Set up and Session — rather than the five flat entries it had. `RQX.toml`, a
  baseline and clearing the cache can be run from it.
- **Three new screens** — Complexity (and bottlenecks), Graph (the cycle rings and
  the imports) and Health (and slop) — from data the scan already produced.
- **`rqx scan` opens the interface** when it has a terminal and nothing about the
  invocation says otherwise. `--no-tui` prints the report instead.
- **Filters that subtract**: `--not-rule`, `--not-category`, `--not-path`, plus
  `--confidence`, `--fixable` and `--search`.
- **`--detail terse|normal|full`** and a Cycles section in the report.
- `rqx rules --slugs`, for a script or a shell completion.
- **`rqx comment` posts the review onto the merge request.** GitLab and GitHub, one
  comment edited on every re-run rather than repeated, and a commit status carrying
  the grade so the branch shows it without anyone opening a job log. `--dry-run`
  prints what it would post. [docs](docs/ci.md)
- **A scan screen that says what is happening**: the phase it is in, the ones behind
  it and what each took, files against total, lines a second, an estimate, a bar per
  language filling as files land, and the paths going past.
- **Grouped help.** `rqx --help` puts twenty-eight commands under five headings, and
  a test fails the build until a new one is placed. `s`, `c`, `f` and `v` are
  aliases for the four you type all day.
- One dim line each when a run has no `RQX.toml`, and when it is not in a git
  repository and the churn rules therefore found nothing.

### Removed

- **The browser dashboard.** `rqx serve`, the `html` reporter, the `rqx.dashboard`
  LSP command and the editor buttons that opened it are all gone. A second surface
  cost more to keep honest than it returned, and every table it drew is a table the
  terminal interface draws. `rqx` on its own is the interface now.
- `rqx tui` — the interface opens on `rqx` and on `rqx scan` in a terminal; nothing
  is left for the subcommand to name.

### Changed

- **The cache is on by default.** It was off because a warm run was no faster than a
  cold one; that was measured while the collector waited for `git log` before
  draining its channel. With that fixed, a warm scan of 3,440 files costs 298ms
  against 2,023ms cold.
- **A cycle is reported as a ring** — `a.ts → b.ts → c.ts → a.ts` — anchored on the
  import that opens it rather than on byte zero of whichever file sorted first.
- `verify` recognises an Extract Method that stayed in the file it came from.
- **`rqx` scans on launch and opens on your own code**, rather than on a menu. The
  menu is gone; the palette shows the same five groups when nothing is typed and
  ranks matches when something is, and `esc` steps back through where you have been.
- **One key, one meaning.** `s` was severity, sort and slop on three screens; it is
  sort everywhere, `m` flips a screen's other mode, `g` groups, `>` raises the
  severity floor. A test holds every screen's key table to one vocabulary.
- `esc` on the scan screen stops the scan instead of leaving it running behind you.
- The parser gives up on a file after five seconds, so one pathological minified
  file cannot hold a worker forever.

### Fixed

- **Findings could differ between runs** on any project with more than
  `max_findings` of them. The heap that chose which to keep ranked only on severity,
  confidence and rule, so ties were broken by whichever worker finished first.
- **`Identity` is FNV-1a rather than `DefaultHasher`.** Its values are persisted in
  GitLab reports and now in baselines, and the standard library promises nothing
  across versions. GitLab fingerprints change once.
- Release URLs, the VS Code extension's download base and the Action's binary base
  all named a repository that does not exist.
- The VS Code extension registered six commands the language client already
  registers from the server, so it failed to start at all.
- **A monochrome palette printed escape codes.** `--reporter terminal=<path>` wrote
  29 of them into the file and the reading pane showed `[1m` as text: bold is an
  effect rather than a colour, so an empty style still rendered one. Every reporter
  is now asserted to emit no `\x1b` at all when colour is off.
- **`rqx scan .` on this repository took 5.05s; it takes 0.36s.** Resolving imports
  walked the filesystem five levels deep ignoring `prune_dirs`, which on a build
  directory is 209,803 `stat` calls. A cold scan of a 3,456-file TypeScript project
  went from 6.05s to 2.60s: text is sliced rather than re-validated, node kinds are
  compared as integers, the walker carries its own ancestry instead of asking
  tree-sitter to re-descend for a parent, git blames run in parallel, and the cache
  loads beside the walk rather than before it. Findings are byte-identical.
- **The unused-fixture check was cubic** in the size of a test file. A 6,899-line
  pytest file took 36s to scan and now takes 0.47s.
- **The commit status named the wrong commit on GitHub.** `GITHUB_SHA` on a
  pull_request event is the merge commit, which is on no branch. The token also went
  to `curl` as an argument, where `ps` showed it to every other job on the runner.
- `--config` said it read a file "instead of discovering `RQX.toml`" while layering
  it over what it discovered, which is what `docs/configuration.md` describes and
  what it still does. The flag now says so.
