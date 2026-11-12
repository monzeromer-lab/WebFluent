//! The front end on input that used to hang it, overflow its stack or lose
//! what it found: a name that starts with a symbol, a program that nests
//! past any reasonable depth, a `.wfx` file of many lines, and several
//! errors in one file.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use webfluent::build::{BuildOptions, check};
use webfluent::lexer::{LexerV2, TokenType};
use webfluent::parser::MAX_NESTING;
use webfluent::parser::v2::{ParserV2, parse_v2};
use webfluent::vfs::FsVfs;
use webfluent::{Template, WebFluentError};

/// The stack a test thread gets: 2 MB — what a server's worker thread has —
/// in a release build; a debug build's frames are several times larger, so
/// there it is what `wf`'s own main thread has, twice over.
const STACK: usize = if cfg!(debug_assertions) { 16 << 20 } else { 2 << 20 };

/// Run `f` on a thread with [`STACK`] and fail if it does not finish in
/// `secs`. A hang or a runaway allocation would otherwise take the whole
/// test binary with it, so a timeout ends the process.
fn within<T: Send + 'static>(secs: u64, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(move || {
            let _ = tx.send(f());
        })
        .unwrap();
    match rx.recv_timeout(Duration::from_secs(secs)) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("did not finish within {secs}s (or overflowed its stack)");
            std::process::exit(1);
        }
    }
}

fn codes(err: &WebFluentError) -> Vec<&'static str> {
    err.diagnostics().iter().map(|d| d.code).collect()
}

fn parse_codes(src: &str, file: &str) -> Result<(), Vec<&'static str>> {
    parse_v2(src, file).map(|_| ()).map_err(|e| codes(&e))
}

// ─── B1: a name that starts with a symbol ────────────────────────────

/// Every character that may start a name (UAX #31) but is no letter to
/// `char::is_alphanumeric` — `℘` and `℮` among them.
fn starts_but_not_alphanumeric() -> Vec<char> {
    (0..=0x10FFFFu32)
        .filter_map(char::from_u32)
        .filter(|c| !c.is_ascii() && unicode_ident::is_xid_start(*c) && !c.is_alphanumeric())
        .collect()
}

#[test]
fn a_name_may_start_with_any_character_uax31_lets_one_start_with() {
    let odd = starts_but_not_alphanumeric();
    assert!(odd.contains(&'℘') && odd.contains(&'℮'), "{odd:?}");
    let found = within(30, move || {
        odd.iter()
            .map(|c| {
                let src = format!("page P(path: \"/\") {{ Text({c}) }}");
                let tokens = LexerV2::new(&src, "x.wf").tokenize().unwrap();
                let names: Vec<String> = tokens
                    .iter()
                    .filter_map(|t| match &t.token_type {
                        TokenType::Identifier(w) => Some(w.clone()),
                        _ => None,
                    })
                    .collect();
                (*c, names)
            })
            .collect::<Vec<_>>()
    });
    for (c, names) in found {
        assert!(names.iter().all(|n| !n.is_empty()), "{c:?} made an empty name: {names:?}");
        assert!(names.iter().any(|n| n.starts_with(c)), "{c:?} is not a name: {names:?}");
    }
}

#[test]
fn a_name_that_starts_with_a_symbol_goes_on_with_letters() {
    let tokens = within(10, || LexerV2::new("℘abc ℮1", "x.wf").tokenize().unwrap());
    let names: Vec<_> = tokens.iter().map(|t| t.token_type.clone()).collect();
    assert_eq!(names[0], TokenType::Identifier("℘abc".into()));
    assert_eq!(names[1], TokenType::Identifier("℮1".into()));
}

#[test]
fn the_reviewers_page_checks_and_renders_without_hanging() {
    let src = "page P(path: \"/\") { Text(℘) }";
    within(10, move || {
        // The name is undeclared, so the checker says so; nothing hangs.
        let _ = parse_v2(src, "x.wf").unwrap();
        let _ = Template::from_str(src);
    });
}

#[test]
fn every_character_lexes_or_is_refused_in_every_position() {
    // A character in each place a token may begin — a name, a value, a
    // style value — either lexes or is an error; none hangs.
    let samples: Vec<char> = (0..=0x10FFFFu32)
        .step_by(97)
        .filter_map(char::from_u32)
        .chain(['℘', '℮', '\u{0300}', '\u{200D}', '\u{FEFF}', '·', '゛'])
        .collect();
    within(60, move || {
        for c in samples {
            for src in [
                format!("{c}"),
                format!("page P(path: \"/\") {{ Text({c}) }}"),
                format!("page P(path: \"/\") {{ Text(\"x\") {{ style {{ color: {c} }} }} }}"),
                format!("a{c}b"),
            ] {
                let _ = LexerV2::new(&src, "x.wf").tokenize();
                let _ = LexerV2::new(&src, "x.wfx").tokenize();
            }
        }
    });
}

// ─── B13: `.wfx` lexing in linear time ───────────────────────────────

fn wfx_lines(n: usize) -> String {
    let mut src = String::from("page P(path: \"/\")\n");
    for i in 0..n {
        src.push_str(&format!("    Row\n        Text(\"line {i}\")\n"));
    }
    src
}

#[test]
fn a_long_wfx_file_lexes_in_linear_time() {
    // 40,000 implied blocks: quadratic was minutes, linear is well under
    // a second even in a debug build.
    let src = wfx_lines(20_000);
    let tokens = within(20, move || LexerV2::for_file(&src, "x.wfx").tokenize().unwrap());
    let opens = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::OpenBrace)
        .count();
    assert_eq!(opens, 20_001);
}

#[test]
fn an_implied_brace_sits_after_the_last_token_of_its_line() {
    // The column of a synthetic brace is the column after the token it
    // follows — names in Arabic and strings with splices included.
    let src = "page P(path: \"/\")\n    Row\n        Text(\"عدد {x}\")\n    عمود\n";
    let tokens = LexerV2::for_file(src, "x.wfx").tokenize().unwrap();
    let braces: Vec<(usize, usize)> = tokens
        .iter()
        .filter(|t| matches!(t.token_type, TokenType::OpenBrace | TokenType::CloseBrace))
        .filter(|t| t.offset == t.end)
        .map(|t| (t.line, t.column))
        .collect();
    // `{` after `)` on line 1 (column 18), `{` after `Row` on line 2,
    // `}` after `)` on line 3, then the closes at the end of the file.
    assert_eq!(braces[0], (1, 18), "{braces:?}");
    assert_eq!(braces[1], (2, 8), "{braces:?}");
    assert_eq!(braces[2], (3, 24), "{braces:?}");
    assert_eq!(braces.last().unwrap().0, 4, "{braces:?}");
}

#[test]
fn every_implied_brace_is_where_the_source_says() {
    // The column of each synthetic brace is the column of the token before
    // it plus that token's width in characters, read from the source — what
    // the quadratic walk computed — for every `.wfx` file of the examples.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    let mut stack = vec![root.join("examples"), root.join("tests"), root.join("site")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "wfx") {
                files.push(path);
            }
        }
    }
    files.push(PathBuf::from("<inline>"));
    let mut checked = 0;
    for file in files {
        let src = if file.to_str() == Some("<inline>") {
            "page P(path: \"/\")\n    Row\n        Text(\"عدد {x}\")\n    /// docs\n    عمود\n".to_string()
        } else {
            std::fs::read_to_string(&file).unwrap()
        };
        let Ok(tokens) = LexerV2::for_file(&src, "x.wfx").tokenize() else {
            continue;
        };
        for pair in tokens.windows(2) {
            let (before, brace) = (&pair[0], &pair[1]);
            if brace.offset != brace.end
                || !matches!(brace.token_type, TokenType::OpenBrace | TokenType::CloseBrace)
            {
                continue;
            }
            let width = src[before.offset..before.end].chars().count();
            assert_eq!(
                (brace.line, brace.column),
                (before.line, before.column + width),
                "{}",
                file.display()
            );
            checked += 1;
        }
    }
    assert!(checked > 0);
}

// ─── B14: every error, and a parser with no tokens ───────────────────

#[test]
fn a_file_with_several_errors_reports_each_of_them() {
    let src = "page A(path: \"/a\") { Text( }\npage B(path: \"/b\") { Row( }\n";
    let err = parse_v2(src, "x.wf").unwrap_err();
    assert!(matches!(err, WebFluentError::Diagnostics(_)), "{err:?}");
    let found = err.diagnostics();
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!((found[0].line, found[1].line), (1, 2));
}

#[test]
fn a_file_with_one_error_is_still_one_parse_error() {
    let err = parse_v2("page A(path: \"/a\") { Text( }\n", "x.wf").unwrap_err();
    assert!(matches!(err, WebFluentError::ParseError(_)), "{err:?}");
}

#[test]
fn a_parser_given_no_tokens_parses_nothing() {
    let program = ParserV2::new(Vec::new(), "x.wf").parse().unwrap();
    assert!(program.declarations.is_empty());
    let mut parser = ParserV2::new(Vec::new(), "x.wf");
    assert!(parser.parse_expression().is_err());
}

#[test]
fn a_parser_given_tokens_without_an_end_finds_one() {
    let mut tokens = LexerV2::new("page P(path: \"/\") { Text(\"x\") }", "x.wf")
        .tokenize()
        .unwrap();
    tokens.pop(); // the EOF
    let program = ParserV2::new(tokens, "x.wf").parse().unwrap();
    assert_eq!(program.declarations.len(), 1);
}

// ─── B2: nesting, in the parser ──────────────────────────────────────

fn page_with(expr: &str) -> String {
    format!("page P(path: \"/\", title: \"T\", description: \"D\") {{\n    derived x = {expr}\n    Heading(\"H\").h1\n    Text(\"{{x}}\")\n}}\n")
}

/// Each shape of nesting the grammar has, `n` deep.
fn shapes(n: usize) -> Vec<(&'static str, String)> {
    vec![
        ("not", format!("{}true", "!".repeat(n))),
        ("neg", format!("{}1", "-".repeat(n))),
        ("await", format!("{}1", "await ".repeat(n))),
        ("parens", format!("{}1{}", "(".repeat(n), ")".repeat(n))),
        ("lists", format!("{}1{}", "[".repeat(n), "]".repeat(n))),
        ("maps", format!("{}1{}", "{a: ".repeat(n), "}".repeat(n))),
        ("calls", format!("{}1{}", "f(".repeat(n), ")".repeat(n))),
        ("plus", format!("1{}", " + 1".repeat(n))),
        ("and", format!("true{}", " && true".repeat(n))),
        ("coalesce", format!("a{}", " ?? a".repeat(n))),
        ("members", format!("a{}", ".b".repeat(n))),
        ("indexes", format!("a{}", "[0]".repeat(n))),
        ("optional", format!("a{}", "?.b".repeat(n))),
        ("methods", format!("a{}", ".m()".repeat(n))),
        ("ifs", format!("{}1{}", "if true { ".repeat(n), " } else { 0 }".repeat(n))),
        ("lambdas", format!("{}1", "x => ".repeat(n))),
    ]
}

#[test]
fn every_shape_of_nesting_parses_just_under_the_limit() {
    // Comfortably under: what a program might reach is far less.
    let found = within(60, || {
        shapes(MAX_NESTING - 8)
            .into_iter()
            .map(|(name, e)| (name, parse_codes(&page_with(&e), "x.wf")))
            .collect::<Vec<_>>()
    });
    for (name, result) in found {
        assert!(result.is_ok(), "{name}: {result:?}");
    }
}

#[test]
fn every_shape_of_nesting_past_the_limit_is_e120() {
    let found = within(60, || {
        shapes(MAX_NESTING + 1)
            .into_iter()
            .map(|(name, e)| (name, parse_codes(&page_with(&e), "x.wf")))
            .collect::<Vec<_>>()
    });
    for (name, result) in found {
        assert_eq!(result, Err(vec!["E120"]), "{name}");
    }
}

#[test]
fn the_reviewers_two_hundred_thousand_nots_are_e120_not_an_abort() {
    let src = page_with(&format!("{}true", "!".repeat(200_000)));
    let result = within(60, move || parse_codes(&src, "x.wf"));
    assert_eq!(result, Err(vec!["E120"]));
}

#[test]
fn a_long_chain_past_the_limit_is_e120_not_a_deep_tree() {
    // Built in a loop, a chain makes a tree as deep as it is long, which
    // every later pass — and dropping it — walks by recursion.
    for e in [
        format!("a{}", ".b".repeat(100_000)),
        format!("1{}", " + 1".repeat(100_000)),
        format!("a{}", "[0]".repeat(100_000)),
    ] {
        let src = page_with(&e);
        let result = within(60, move || parse_codes(&src, "x.wf"));
        assert_eq!(result, Err(vec!["E120"]));
    }
}

#[test]
fn the_limit_is_exact_for_a_bracket() {
    // The derived value's own expression is one level, each `(` another.
    let at = |n: usize| parse_codes(&page_with(&format!("{}1{}", "(".repeat(n), ")".repeat(n))), "x.wf");
    let (ok, over) = within(30, move || {
        let mut last_ok = 0;
        for n in (MAX_NESTING - 20)..(MAX_NESTING + 5) {
            if at(n).is_ok() {
                last_ok = n;
            }
        }
        (last_ok, at(last_ok + 1))
    });
    assert!(ok >= MAX_NESTING - 8, "{ok}");
    assert_eq!(over, Err(vec!["E120"]));
}

#[test]
fn nesting_through_string_splices_is_counted() {
    // `"a{("a{(…)}b")}b"` — each splice parses its text with a parser of
    // its own, which starts where the one around it is.
    let mut e = String::from("1");
    for _ in 0..(MAX_NESTING + 1) {
        e = format!("\"a{{({e})}}b\"");
    }
    let deep = page_with(&e);
    let mut e = String::from("1");
    for _ in 0..20 {
        e = format!("\"a{{({e})}}b\"");
    }
    let shallow = page_with(&e);
    let (deep, shallow) = within(60, move || {
        (parse_codes(&deep, "x.wf"), parse_codes(&shallow, "x.wf"))
    });
    assert_eq!(deep, Err(vec!["E120"]));
    assert!(shallow.is_ok(), "{shallow:?}");
}

#[test]
fn nesting_through_style_splices_is_counted() {
    let inner = format!("{}1{}", "(".repeat(MAX_NESTING + 1), ")".repeat(MAX_NESTING + 1));
    let src = format!(
        "page P(path: \"/\") {{\n    Text(\"x\") {{\n        style {{\n            width: {{{inner}}}px\n        }}\n    }}\n}}\n"
    );
    let result = within(30, move || parse_codes(&src, "x.wf"));
    assert_eq!(result, Err(vec!["E120"]));
}

#[test]
fn nesting_blocks_and_elements_is_counted() {
    let n = MAX_NESTING + 1;
    let elements = format!("page P(path: \"/\") {{ {}Text(\"x\"){} }}", "Row { ".repeat(n), " }".repeat(n));
    let ifs = format!("page P(path: \"/\") {{ {}Text(\"x\"){} }}", "if true { ".repeat(n), " }".repeat(n));
    let handlers = format!(
        "page P(path: \"/\") {{ state a = 0\n Button(\"b\") {{ on click {{ {}a = 1{} }} }} }}",
        "if true { ".repeat(n),
        " }".repeat(n)
    );
    let types = format!("component C(x: {}String{}) {{ Text(\"x\") }}", "[".repeat(n), "]".repeat(n));
    let found = within(60, move || {
        [elements, ifs, handlers, types]
            .iter()
            .map(|s| parse_codes(s, "x.wf"))
            .collect::<Vec<_>>()
    });
    for result in found {
        assert_eq!(result, Err(vec!["E120"]));
    }
}

#[test]
fn nesting_in_wfx_is_counted() {
    let n = MAX_NESTING + 1;
    let mut src = String::from("page P(path: \"/\")\n");
    for i in 0..n {
        src.push_str(&"    ".repeat(i + 1));
        src.push_str("Row\n");
    }
    let result = within(60, move || parse_codes(&src, "x.wfx"));
    assert_eq!(result, Err(vec!["E120"]));
}

#[test]
fn the_parser_recovers_after_e120_and_reads_the_next_declaration() {
    let src = format!(
        "{}\npage Q(path: \"/q\") {{ Text( }}\n",
        page_with(&format!("{}true", "!".repeat(MAX_NESTING + 1)))
    );
    let err = within(30, move || parse_v2(&src, "x.wf").unwrap_err());
    assert_eq!(codes(&err), vec!["E120", "E002"]);
}

#[test]
fn nesting_generated_at_random_never_overflows() {
    // Many shapes mixed: each either parses or is refused, on a 2 MB stack.
    let pieces: [(&str, &str); 8] = [
        ("(", ")"),
        ("[", "]"),
        ("{a: ", "}"),
        ("f(", ")"),
        ("!", ""),
        ("-", ""),
        ("\"{", "}\""),
        ("if true { ", " } else { 0 }"),
    ];
    let mut seed: u64 = 0x5eed;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut cases = Vec::new();
    for _ in 0..120 {
        let depth = (next() % 600) as usize;
        let mut open = String::new();
        let mut close = Vec::new();
        for _ in 0..depth {
            let (o, c) = pieces[(next() % pieces.len() as u64) as usize];
            open.push_str(o);
            close.push(c);
        }
        let tail = ".b".repeat((next() % 300) as usize);
        let e = format!("{open}a{tail}{}", close.iter().rev().copied().collect::<String>());
        cases.push(page_with(&e));
    }
    let found = within(120, move || {
        cases
            .iter()
            .map(|c| parse_codes(c, "x.wf"))
            .collect::<Vec<_>>()
    });
    for result in found {
        if let Err(codes) = result {
            assert!(codes.iter().all(|c| *c == "E120" || *c == "E002"), "{codes:?}");
        }
    }
}

// ─── B2: nesting, in the checker ─────────────────────────────────────

fn project(name: &str, page: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-frontend-fixes-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "deep", "build": { "output": "./build" } }"#,
    )
    .unwrap();
    std::fs::write(dir.join("src/pages/Home.wf"), page).unwrap();
    dir
}

/// Each finding's code.
fn findings(dir: &Path) -> Vec<&'static str> {
    let report = match check(&BuildOptions::new(dir, &FsVfs)) {
        Ok(report) => report,
        Err(failure) => *failure.report,
    };
    report.diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn a_program_just_under_the_limit_checks_on_a_small_stack() {
    // What the parser accepts, every later pass must walk: the checker,
    // the linters, the code generators.
    let n = MAX_NESTING - 8;
    let cases = vec![
        ("parens", format!("{}1{}", "(".repeat(n), ")".repeat(n))),
        ("not", format!("{}true", "!".repeat(n))),
        ("methods", format!("\"a\"{}", ".trim()".repeat(n))),
        ("plus", format!("1{}", " + 1".repeat(n))),
        ("ifs", format!("{}1{}", "if true { ".repeat(n), " } else { 0 }".repeat(n))),
        ("lists", format!("{}1{}", "[".repeat(n), "]".repeat(n))),
        ("lambdas", format!("[1].map({}1)", "x => ".repeat(n))),
    ];
    for (name, e) in cases {
        let dir = project(name, &page_with(&e));
        let found = within(120, move || findings(&dir));
        assert!(!found.contains(&"E120"), "{name}: {found:?}");
    }
}

#[test]
fn a_tree_deeper_than_the_parser_allows_is_e120_in_the_checker() {
    // A tree the parser would refuse, made by hand: the checker says so
    // once and goes on, rather than recursing past its stack.
    use webfluent::parser::{Declaration, Expr, StatementKind, UnaryOp};
    let found = within(60, || {
        let mut program = parse_v2(&page_with("true"), "x.wf").unwrap();
        let Declaration::Page(page) = &mut program.declarations[0] else {
            unreachable!()
        };
        let StatementKind::Derived(d) = &mut page.body[0].kind else {
            unreachable!()
        };
        let mut e = Expr::BoolLiteral(true);
        for _ in 0..(MAX_NESTING * 3) {
            e = Expr::UnaryOp(UnaryOp::Not, Box::new(e));
        }
        d.value = e;
        let info = webfluent::sema::types::check(&program, &|_| "x.wf".to_string());
        info.findings
            .errors
            .iter()
            .map(|d| d.code)
            .collect::<Vec<_>>()
    });
    assert_eq!(found, vec!["E120"]);
}

// ─── B12: a number the grammar cannot hold is refused, not clamped ───

fn api_with(status: &str) -> String {
    format!(
        "api B(base: \"/api\") {{\n    post make(body: Map) -> Map\n        errors {{ {status} -> Map }}\n}}\n"
    )
}

#[test]
fn a_status_outside_http_s_range_or_with_a_fraction_is_refused() {
    for bad in ["70000", "404.5", "99", "600", "0"] {
        let err = parse_v2(&api_with(bad), "x.wf").unwrap_err();
        let found = err.diagnostics();
        assert_eq!(found.len(), 1, "{bad}: {found:?}");
        assert!(found[0].message.contains("is not an HTTP status"), "{bad}: {found:?}");
    }
    for good in ["100", "422", "599"] {
        parse_v2(&api_with(good), "x.wf").unwrap_or_else(|e| panic!("{good}: {e}"));
    }
}

#[test]
fn a_persisted_version_too_large_to_hold_is_refused() {
    let with = |v: &str| {
        format!("page P(path: \"/\") {{\n    persist n: Number = 0 {{ version: {v} }}\n    Text(\"{{n}}\")\n}}\n")
    };
    for bad in ["4294967296", "10000000000000", "0", "1.5"] {
        let err = parse_v2(&with(bad), "x.wf").unwrap_err();
        assert!(err.to_string().contains("is not a version"), "{bad}: {err}");
    }
    for good in ["1", "4294967295"] {
        parse_v2(&with(good), "x.wf").unwrap_or_else(|e| panic!("{good}: {e}"));
    }
}

// ─── B10: the editor's scope inside a connection's handler ───────────

#[test]
fn a_socket_s_handler_has_its_message_in_scope() {
    let src = "page P(path: \"/\") {\n    state last = \"\"\n    socket chat = ws(\"wss://example.com/chat\") {\n        on message(m) {\n            last = m.text\n        }\n    }\n    Text(last)\n}\n";
    let program = parse_v2(src, "x.wf").unwrap();
    let offset = src.find("m.text").unwrap();
    let names: Vec<String> = webfluent::scope::scope_at(&program.declarations[0], offset)
        .into_iter()
        .map(|b| b.name)
        .collect();
    assert!(names.contains(&"m".to_string()), "{names:?}");
    // Outside the handler, it is not.
    let outside = src.find("Text(last)").unwrap();
    let names: Vec<String> = webfluent::scope::scope_at(&program.declarations[0], outside)
        .into_iter()
        .map(|b| b.name)
        .collect();
    assert!(!names.contains(&"m".to_string()), "{names:?}");
}

#[test]
fn child_bodies_names_every_body_the_ast_has() {
    use webfluent::parser::StatementKind;
    let src = "page P(path: \"/\") {\n    socket chat = ws(\"wss://x.example/c\") {\n        on message(m) { log(m) }\n        on close(c) { log(c) }\n    }\n    every(1000) { log(1) }\n    effect { log(2)\n cleanup { log(3) } }\n    Text(\"x\")\n}\n";
    let program = parse_v2(src, "x.wf").unwrap();
    let webfluent::parser::Declaration::Page(page) = &program.declarations[0] else {
        unreachable!()
    };
    for stmt in &page.body {
        let mut want = stmt.kind.bodies().len();
        if let StatementKind::UIElement(el) = &stmt.kind {
            want = 1 + el.events.len() + el.slot_fills.len();
        }
        assert_eq!(webfluent::scope::child_bodies(stmt).len(), want, "{:?}", stmt.kind);
    }
    let StatementKind::Connection(c) = &page.body[0].kind else {
        unreachable!()
    };
    assert_eq!(c.handlers.len(), 2);
    assert_eq!(webfluent::scope::child_bodies(&page.body[0]).len(), 2);
}

// ─── A service's cache, forgotten whole ──────────────────────────────

#[test]
fn a_service_may_be_invalidated_whole() {
    let with = |call: &str| {
        format!(
            "api Backend(base: \"/api\") {{\n    get users() -> [Map]\n}}\npage P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"H\").h1\n    Button(\"Log out\") {{ on click {{ {call} }} }}\n}}\n"
        )
    };
    let errors = |call: &str| {
        let program = parse_v2(&with(call), "x.wf").unwrap();
        webfluent::sema::types::check(&program, &|_| "x.wf".to_string())
            .findings
            .errors
            .iter()
            .map(|d| d.code)
            .collect::<Vec<_>>()
    };
    assert_eq!(errors("Backend.invalidate()"), Vec::<&str>::new());
    assert_eq!(errors("Backend.users.invalidate()"), Vec::<&str>::new());
    assert_eq!(errors("Backend.invalidate(1)"), vec!["T10"]);
    assert_eq!(errors("Backend.forget()"), vec!["T06"]);
}

// ─── B9, B11: findings that are the same every run, in the right place ──

#[test]
fn an_action_two_stores_share_is_named_the_same_way_every_run() {
    let page = "store Beta { state n = 0\n action reset() { n = 0 } }\nstore Alpha { state n = 0\n action reset() { n = 0 } }\npage P(path: \"/\", title: \"T\", description: \"D\") {\n    Heading(\"H\").h1\n    Button(\"b\") { on click { reset() } }\n}\n";
    let dir = project("b9", page);
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..12 {
        let report = match check(&BuildOptions::new(&dir, &FsVfs)) {
            Ok(report) => report,
            Err(failure) => *failure.report,
        };
        for d in report.diagnostics.iter().filter(|d| d.code == "T13") {
            seen.insert(d.message.clone());
        }
    }
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert!(seen.iter().next().unwrap().contains("`Alpha`"), "{seen:?}");
}

#[test]
fn a_handler_s_extra_name_is_placed_by_characters_on_a_line_of_arabic() {
    let line = "    Button(\"حفظ الملف\") { on click(e, زائد) { log(\"x\") } }";
    let page = format!(
        "page P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"H\").h1\n{line}\n}}\n"
    );
    let dir = project("b11", &page);
    let report = match check(&BuildOptions::new(&dir, &FsVfs)) {
        Ok(report) => report,
        Err(failure) => *failure.report,
    };
    let t22 = report
        .diagnostics
        .iter()
        .find(|d| d.code == "T22")
        .expect("a T22");
    let want = line.chars().take_while(|c| *c != 'ز').count() + 1;
    assert_eq!((t22.line, t22.column), (3, want), "{t22:?}");
}
