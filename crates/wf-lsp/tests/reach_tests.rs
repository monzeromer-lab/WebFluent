//! What the server reaches of the language: every word the parser reads,
//! every name the compiler gives a program, a component's props, events and
//! slots as things to rename, and the requests, connections and records a
//! call can make — each read from the compiler's own tables where it has
//! them.

use lsp_types::*;
use wf_lsp::completion::provide_completions;
use wf_lsp::definition::find_definition;
use wf_lsp::hover::provide_hover;
use wf_lsp::project::Project;
use wf_lsp::symbols::document_symbols;

fn project(src: &str) -> Project {
    Project::single(Url::parse("file:///test.wf").unwrap(), src)
}

/// A buffer mid-edit: `broken` is the text, `valid` the last parse of it that
/// succeeded — what the server holds while the user types.
fn mid_edit(valid: &str, broken: &str) -> Project {
    let mut project = project(valid);
    project.files[0].source = broken.into();
    project.files[0].index = wf_lsp::line_index::LineIndex::new(broken);
    project.files[0].parsed = Project::single(Url::parse("file:///x.wf").unwrap(), broken)
        .files
        .remove(0)
        .parsed;
    project.files[0].stale = project.files[0].parsed.is_err();
    project
}

/// Completion at `‸` in `broken`, the server holding `valid`. Labels come
/// back without a prop's `:` or a token's `$`.
fn offered(valid: &str, broken: &str) -> Vec<String> {
    let at = broken.find('‸').expect("a ‸ marks the cursor");
    let text = broken.replacen('‸', "", 1);
    let project = mid_edit(valid, &text);
    let pos = project.files[0].index.offset_to_position(&text, at);
    provide_completions(&project, 0, pos)
        .into_iter()
        .map(|c| {
            c.label
                .trim_end_matches(':')
                .trim_start_matches('$')
                .to_string()
        })
        .collect()
}

#[track_caller]
fn offers(labels: &[String], want: &[&str]) {
    let missing: Vec<&&str> = want
        .iter()
        .filter(|w| !labels.iter().any(|l| l == *w))
        .collect();
    assert!(missing.is_empty(), "missing {missing:?} in {labels:?}");
}

#[track_caller]
fn lacks(labels: &[String], unwanted: &[&str]) {
    let there: Vec<&&str> = unwanted
        .iter()
        .filter(|w| labels.iter().any(|l| l == *w))
        .collect();
    assert!(there.is_empty(), "{there:?} offered in {labels:?}");
}

const HEAD: &str =
    "page Home(path: \"/\", title: \"H\", description: \"D\") {\n    Heading(\"H\").h1\n";

fn in_page(valid_body: &str, broken_body: &str) -> Vec<String> {
    offered(
        &format!("{HEAD}{valid_body}\n}}\n"),
        &format!("{HEAD}{broken_body}\n}}\n"),
    )
}

fn hover_at(src: &str, needle: &str, nth: usize) -> Option<String> {
    let project = project(src);
    let offset = src.match_indices(needle).nth(nth).unwrap().0 + 1;
    let pos = project.files[0].index.offset_to_position(src, offset);
    provide_hover(&project, 0, pos).map(|h| match h.contents {
        HoverContents::Markup(m) => m.value,
        _ => panic!("markdown expected"),
    })
}

fn apply(src: &str, edit: &WorkspaceEdit) -> String {
    let project = project(src);
    let file = &project.files[0];
    let mut edits: Vec<&TextEdit> = edit.changes.as_ref().unwrap().values().flatten().collect();
    edits.sort_by_key(|e| (e.range.start.line, e.range.start.character));
    let mut out = src.to_string();
    for e in edits.into_iter().rev() {
        let s = file.index.position_to_offset(src, e.range.start).unwrap();
        let t = file.index.position_to_offset(src, e.range.end).unwrap();
        out.replace_range(s..t, &e.new_text);
    }
    out
}

/// `src` with the name at the `nth` `needle` renamed to `to`.
fn renamed(src: &str, needle: &str, nth: usize, to: &str) -> String {
    let project = project(src);
    let offset = src.match_indices(needle).nth(nth).unwrap().0 + 1;
    let pos = project.files[0].index.offset_to_position(src, offset);
    assert!(
        wf_lsp::rename::prepare_rename(&project, 0, pos).is_some(),
        "{needle} #{nth} cannot be renamed"
    );
    apply(src, &wf_lsp::rename::rename(&project, 0, pos, to).unwrap())
}

fn definition_line(src: &str, needle: &str, nth: usize) -> Option<u32> {
    let project = project(src);
    let offset = src.match_indices(needle).nth(nth).unwrap().0 + 1;
    let pos = project.files[0].index.offset_to_position(src, offset);
    match find_definition(&project, 0, pos)? {
        GotoDefinitionResponse::Scalar(l) => Some(l.range.start.line + 1),
        _ => None,
    }
}

// ─── A component's props, events and slots ───────────────────────────────

const CARD: &str = "component UserCard(_ name: String, title: String, done: Bool = false) {\n    event toggle(id: String)\n    slot trailing\n    Row {\n        Text(title)\n        Checkbox(checked: done) { on change { emit toggle(name) } }\n        trailing\n    }\n}\npage P(path: \"/\") {\n    state picked = \"\"\n    UserCard(\"Ada\", title: \"Lead\") {\n        on toggle(id) { picked = id }\n        trailing { Badge(\"New\") }\n    }\n}\n";

#[test]
fn a_named_argument_of_a_component_is_its_prop() {
    // From the call, the body's read and the header: the header's line.
    assert_eq!(definition_line(CARD, "title:", 1), Some(1));
    assert_eq!(definition_line(CARD, "title)", 0), Some(1));
    let out = renamed(CARD, "title:", 1, "role");
    assert!(
        out.contains("UserCard(_ name: String, role: String,")
            && out.contains("Text(role)")
            && out.contains("UserCard(\"Ada\", role: \"Lead\")"),
        "{out}"
    );
    // At the declaration, too.
    let out = renamed(CARD, "title", 0, "role");
    assert!(out.contains("role: \"Lead\""), "{out}");
}

#[test]
fn an_event_renames_where_it_is_declared_fired_and_handled() {
    assert_eq!(definition_line(CARD, "toggle(id)", 0), Some(2));
    assert_eq!(definition_line(CARD, "toggle(name)", 0), Some(2));
    let out = renamed(CARD, "toggle(id) {", 0, "flip");
    assert!(
        out.contains("event flip(id: String)")
            && out.contains("emit flip(name)")
            && out.contains("on flip(id) {"),
        "{out}"
    );
}

#[test]
fn a_slot_renames_where_it_is_declared_placed_and_filled() {
    assert_eq!(definition_line(CARD, "trailing {", 0), Some(3));
    let out = renamed(CARD, "trailing {", 0, "aside");
    assert!(
        out.contains("slot aside\n")
            && out.contains("        aside\n    }")
            && out.contains("aside { Badge"),
        "{out}"
    );
}

// ─── Keywords, by the parser's tables ────────────────────────────────────

#[test]
fn a_body_is_offered_the_words_the_parser_takes_there() {
    let body = in_page("    Text(\"a\")", "    ‸");
    offers(&body, &["worker", "on", "state", "head", "else"]);
    lacks(
        &body,
        &[
            "navigate",
            "log",
            "style",
            "transition",
            "migrate",
            "emit",
            "let",
        ],
    );
    // In an element's block: the element's own words.
    let el = in_page(
        "    Card {\n        Text(\"a\")\n    }",
        "    Card {\n        ‸\n    }",
    );
    offers(&el, &["style", "transition", "on"]);
    // A store holds what it keeps open, too.
    let store = offered(
        "store S {\n    state n = 0\n    state m = 1\n}\n",
        "store S {\n    state n = 0\n    ‸\n}\n",
    );
    offers(
        &store,
        &[
            "state", "use", "resource", "socket", "stream", "channel", "peer", "worker", "action",
        ],
    );
    lacks(&store, &["if", "head", "Button"]);
}

#[test]
fn a_block_s_own_words_are_offered_in_it() {
    let seq = "    sequence {\n        step { Text(\"a\") }\n    }";
    offers(
        &in_page(seq, &seq.replace("step { Text(\"a\") }", "‸")),
        &["step", "step(after:)"],
    );
    let effect = "    state n = 0\n    effect {\n        log(n)\n        log(n + 1)\n    }";
    offers(
        &in_page(effect, &effect.replace("log(n + 1)", "‸")),
        &["cleanup", "let"],
    );
    let action = "    action a() {\n        try {\n            log(1)\n        } catch e {\n            log(2)\n        }\n    }";
    let after_try = in_page(
        action,
        &action.replace(" catch e {\n            log(2)\n        }", " ‸"),
    );
    offers(&after_try, &["catch"]);
    // A connection's handler does something: what an action says.
    let socket = "    socket s = ws(\"wss://x\") {\n        on message(m) {\n            log(m)\n        }\n    }";
    let handler = in_page(socket, &socket.replace("log(m)", "‸"));
    offers(&handler, &["let", "return", "navigate"]);
    lacks(&handler, &["Button", "state"]);
}

#[test]
fn on_offers_the_events_the_place_hears() {
    let page = in_page("    on keydown(e) { log(e) }", "    on ‸");
    offers(&page, &["key", "keydown", "resize", "online", "storage"]);
    lacks(&page, &["click", "submit"]);
    let el = in_page(
        "    Button(\"x\") { on click { log(1) } }",
        "    Button(\"x\") { on ‸ }",
    );
    offers(&el, &["click", "submit"]);
    let worker = "    worker w = spawn(\"/w.js\") {\n        on message(m) { log(m) }\n    }";
    offers(
        &in_page(worker, &worker.replace("on message(m) { log(m) }", "on ‸")),
        &["message", "restart", "error"],
    );
    let socket = "    socket s = ws(\"wss://x\") {\n        on message(m) { log(m) }\n    }";
    let in_socket = in_page(socket, &socket.replace("on message(m) { log(m) }", "‸"));
    offers(&in_socket, &["on", "send", "receive"]);
}

// ─── Names the compiler gives a program ──────────────────────────────────

#[test]
fn every_name_the_compiler_gives_is_offered_and_spawn_where_it_opens_a_worker() {
    let expr = in_page("    Text(\"a\")", "    Text(‸)");
    offers(
        &expr,
        &["env", "params", "locale", "dir", "viewport", "format"],
    );
    lacks(&expr, &["spawn"]);
    for name in wf_lsp::reference::LANGUAGE_VALUES {
        assert!(
            webfluent::sema::types::language_name(name).is_some(),
            "`{name}` is not the language's"
        );
    }
    let worker = "    worker w = spawn(\"/w.js\")";
    assert_eq!(
        in_page(worker, &worker.replace("spawn(\"/w.js\")", "‸")),
        vec!["spawn"]
    );
    let socket = "    socket s = ws(\"wss://x\")";
    assert_eq!(
        in_page(socket, &socket.replace("ws(\"wss://x\")", "‸")),
        vec!["ws"]
    );
}

#[test]
fn hover_on_a_type_the_language_brings_says_what_it_can_do() {
    let src = format!("{HEAD}    state d: Date = @2026-03-14\n    state m: Money = €12.99\n}}\n");
    let date = hover_at(&src, "Date", 0).expect("hover on Date");
    assert!(
        date.contains("**Date**") && date.contains(".plus()"),
        "{date}"
    );
    let money = hover_at(&src, "Money", 0).expect("hover on Money");
    assert!(
        money.contains("minor units") && money.contains(".amount"),
        "{money}"
    );
}

#[test]
fn type_in_a_test_is_the_step_and_at_the_top_level_the_declaration() {
    let src = "type Row { id: String }\ntest \"t\" {\n    Input(label: \"Name\")\n    type \"Ada\" into \"Name\"\n    expect \"Ada\"\n}\n";
    let top = hover_at(src, "type", 0).unwrap();
    assert!(top.contains("record type"), "{top}");
    let step = hover_at(src, "type", 1).unwrap();
    assert!(step.contains("Type text into"), "{step}");
}

#[test]
fn a_type_with_a_condition_reads_as_written() {
    let src = "component Nights(n: Number(1..=30)) { Text(\"{n}\") }\npage P(path: \"/\") {\n    Nights(n: 3)\n}\n";
    let p = project(src);
    let DocumentSymbolResponse::Nested(symbols) = document_symbols(&p, 0, true) else {
        panic!()
    };
    let prop = &symbols[0].children.as_ref().unwrap()[0];
    assert_eq!(prop.detail.as_deref(), Some("prop · Number(1..=30)"));
    let at = src.find("n: 3").unwrap() + 3;
    let pos = p.files[0].index.offset_to_position(src, at);
    let help = wf_lsp::signature::signature_help(&p, 0, pos).unwrap();
    assert_eq!(help.signatures[0].label, "Nights(n: Number(1..=30))");
}

// ─── Requests, connections and records ───────────────────────────────────

#[test]
fn a_request_s_options_and_their_cases_are_offered() {
    let fetch = "    resource r = fetch(\"/x\", method: \"GET\")";
    offers(
        &in_page(fetch, &fetch.replace("method: \"GET\"", "‸")),
        &["method", "body", "on", "cache", "paginate"],
    );
    offers(
        &in_page(fetch, &fetch.replace("method: \"GET\"", "on: .‸")),
        &["focus", "reconnect", "interval"],
    );
    let api =
        "api Backend(base: \"/api\") {\n    get users(page: Number = 1, q: String?) -> [Map]\n}\n";
    let v = format!("{api}{HEAD}    resource r = Backend.users(page: 1)\n}}\n");
    let call = offered(&v, &v.replace("page: 1", "‸"));
    offers(&call, &["page", "q", "cache", "as", "on", "paginate"]);
    offers(
        &offered(&v, &v.replace("page: 1", "cache: .‸")),
        &["swr", "none", "forever"],
    );
    for (decl, want) in [
        (
            "    socket s = ws(\"wss://x\", keep: 10)",
            &["protocols", "heartbeat"][..],
        ),
        ("    stream s = sse(\"/e\", keep: 10)", &["events"][..]),
        (
            "    worker w = spawn(\"/w.js\", keep: 10)",
            &["restart", "timeout", "type"][..],
        ),
        (
            "    peer p = rtc(keep: 10)",
            &["signal", "initiator", "ice"][..],
        ),
    ] {
        offers(&in_page(decl, &decl.replace("keep: 10", "‸")), want);
    }
}

#[test]
fn a_record_built_offers_its_fields_and_shows_its_signature() {
    let decls = "type Post { id: String, title: String = \"\" }\nenum Status { idle, failed(reason: String) }\n";
    let v = format!("{decls}{HEAD}    state p = Post(id: \"1\")\n}}\n");
    let fields = offered(&v, &v.replace("Post(id: \"1\")", "Post(‸)"));
    offers(&fields, &["id", "title"]);
    let sig = |valid: &str, broken: &str| {
        let at = broken.find('‸').unwrap();
        let text = broken.replacen('‸', "", 1);
        let p = mid_edit(valid, &text);
        let pos = p.files[0].index.offset_to_position(&text, at);
        wf_lsp::signature::signature_help(&p, 0, pos).map(|h| h.signatures[0].label.clone())
    };
    assert_eq!(
        sig(&v, &v.replace("Post(id: \"1\")", "Post(‸)")).as_deref(),
        Some("Post(id: String, title: String = …)")
    );
    let v = format!("{decls}{HEAD}    state s: Status = .failed(\"x\")\n}}\n");
    assert_eq!(
        sig(&v, &v.replace(".failed(\"x\")", ".failed(‸)")).as_deref(),
        Some(".failed(reason: String)")
    );
    let c = "component C {\n    event pick(id: String, at: Number)\n    Button(\"x\") { on click { emit pick(\"a\", 1) } }\n}\n";
    assert_eq!(
        sig(c, &c.replace("emit pick(\"a\", 1)", "emit pick(‸)")).as_deref(),
        Some("emit pick(id: String, at: Number)")
    );
}

#[test]
fn a_match_offers_an_enum_s_cases_and_a_connection_s_states() {
    let e = "enum Tone { calm, loud, odd(n: Number) }\n";
    let v = format!(
        "{e}{HEAD}    state t: Tone = .calm\n    match t {{\n        .calm {{ Text(\"c\") }}\n    }}\n}}\n"
    );
    let arms = offered(
        &v,
        &v.replace(
            "        .calm { Text(\"c\") }\n    }",
            "        .calm { Text(\"c\") }\n‸    }",
        ),
    );
    offers(&arms, &[".loud", ".odd", "else"]);
    lacks(&arms, &[".calm", "loading", "ready"]);
    let v = format!(
        "{HEAD}    socket s = ws(\"wss://x\")\n    match s {{\n        open {{ Text(\"o\") }}\n    }}\n}}\n"
    );
    let arms = offered(
        &v,
        &v.replace(
            "        open { Text(\"o\") }\n    }",
            "        open { Text(\"o\") }\n‸    }",
        ),
    );
    offers(&arms, &["connecting", "closed", "error", "else"]);
    lacks(&arms, &["open", "loading", "ready"]);
}

// ─── The outline ─────────────────────────────────────────────────────────

#[test]
fn the_outline_lists_events_slots_parts_connections_and_rules() {
    let src = "component Panel(_ title: String) {\n    event close()\n    slot trailing\n    part Header(_ text: String) { Heading(text).h3 }\n    Card { Text(title)  children }\n}\npage P(path: \"/\") {\n    state e = \"\"\n    validate e { required }\n    socket s = ws(\"wss://x\") {\n        on message(m) { log(m) }\n    }\n    Input(bind: e, label: \"E\")\n    Panel(\"x\")\n}\n";
    let p = project(src);
    let DocumentSymbolResponse::Nested(symbols) = document_symbols(&p, 0, true) else {
        panic!()
    };
    let names = |s: &DocumentSymbol| -> Vec<String> {
        s.children
            .iter()
            .flatten()
            .map(|c| c.name.clone())
            .collect()
    };
    assert_eq!(symbols.len(), 2, "the part is inside its owner");
    let panel = names(&symbols[0]);
    for want in ["title", "close", "trailing", "Header"] {
        assert!(panel.contains(&want.to_string()), "{want} in {panel:?}");
    }
    let page = names(&symbols[1]);
    for want in ["e", "validate e", "s"] {
        assert!(page.contains(&want.to_string()), "{want} in {page:?}");
    }
}

// ─── Fixes ───────────────────────────────────────────────────────────────

#[test]
fn every_fix_in_a_file_is_one_source_fix_all() {
    let uri = Url::parse("file:///p/src/App.wf").unwrap();
    let finding = |code: &str, line: u32, text: &str| {
        let edit = TextEdit {
            range: Range::new(Position::new(line, 4), Position::new(line, 8)),
            new_text: text.into(),
        };
        Diagnostic {
            range: edit.range,
            code: Some(NumberOrString::String(code.into())),
            message: "x".into(),
            data: Some(serde_json::json!([{ "title": "fix", "edits": [edit] }])),
            ..Default::default()
        }
    };
    let all = vec![finding("V01", 1, "a"), finding("T05", 3, "b")];
    let Some(CodeActionOrCommand::CodeAction(action)) =
        wf_lsp::code_actions::source_fix_all_action(&uri, &all)
    else {
        panic!("one action");
    };
    assert_eq!(action.kind, Some(CodeActionKind::SOURCE_FIX_ALL));
    assert_eq!(
        action.edit.unwrap().changes.unwrap()[&uri].len(),
        2,
        "both families"
    );
    assert!(wf_lsp::code_actions::source_fix_all_action(&uri, &[]).is_none());
}

#[test]
fn an_extracted_component_keeps_a_language_type() {
    let src = "page P(path: \"/\") {\n    state due: Date = @2026-03-14\n    Card {\n        Text(\"{due}\")\n    }\n}\n";
    let p = project(src);
    let file = &p.files[0];
    let start = src.find("Card {").unwrap();
    let end = src.find("\n}\n").unwrap();
    let range = Range {
        start: file.index.offset_to_position(src, start),
        end: file.index.offset_to_position(src, end),
    };
    let actions = wf_lsp::code_actions::extract_component_action(&p, 0, range);
    let Some(CodeActionOrCommand::CodeAction(action)) = actions.first() else {
        panic!("offered")
    };
    let out = apply(src, action.edit.as_ref().unwrap());
    assert!(out.contains("component Extracted(due: Date)"), "{out}");
}

// ─── Tokens, handles, colours ────────────────────────────────────────────

#[test]
fn the_compiler_s_tokens_and_every_handle_member_are_offered() {
    let style = "    Card {\n        style {\n            color: red\n        }\n    }";
    let tokens = in_page(style, &style.replace("color: red", "color: $‸"));
    offers(
        &tokens,
        &[
            "screen-md",
            "ease-spring",
            "animation-duration-fast",
            "color-primary",
        ],
    );
    let channel = "    channel c = broadcast(\"x\")\n    Text(c.state)";
    offers(
        &in_page(channel, &channel.replace("c.state", "c.‸")),
        &["state", "last", "error", "closure", "post", "messages"],
    );
    let stream = "    stream s = sse(\"/e\")\n    Text(s.state)";
    offers(
        &in_page(stream, &stream.replace("s.state", "s.‸")),
        &["closure", "last"],
    );
}

#[test]
fn a_word_only_a_block_makes_a_keyword_is_a_name_elsewhere() {
    use wf_lsp::semantic::{TYPES, classify};
    let src = format!(
        "{HEAD}    state step = 1\n    worker w = spawn(\"/w.js\")\n    Text(\"{{step}}\")\n    Button(\"x\") {{ on click {{ step = step + 1 }} }}\n}}\n"
    );
    let p = project(&src);
    let classified = classify(&p, 0);
    let kind = |needle: &str, nth: usize| {
        let offset = src.match_indices(needle).nth(nth).unwrap().0;
        let pos = p.files[0].index.offset_to_position(&src, offset);
        classified
            .iter()
            .find(|c| c.start == pos)
            .map(|c| TYPES[c.kind as usize].clone())
    };
    assert_eq!(kind("worker", 0), None, "a keyword");
    assert_eq!(kind("step + 1", 0), Some(SemanticTokenType::VARIABLE));
}

#[test]
fn an_animation_offers_its_keyframes_and_their_properties() {
    let v = "animation Pop {\n    from { opacity: 0 }\n    to { opacity: 1 }\n}\n";
    offers(
        &offered(
            v,
            &v.replace("    to { opacity: 1 }\n", "    to { opacity: 1 }\n    ‸\n"),
        ),
        &["from", "to", "50%"],
    );
    let props = offered(
        v,
        &v.replace(
            "from { opacity: 0 }",
            "from {\n        opacity: 0\n        ‸\n    }",
        ),
    );
    offers(&props, &["opacity", "transform"]);
    lacks(&props, &["@media", "Button"]);
}

#[test]
fn completion_in_an_indented_file_that_does_not_lex_keeps_its_blocks() {
    let src = "page Home(path: \"/\", title: \"H\", description: \"D\")\n    Heading(\"H\").h1\n    Card\n        Text(\"x\")\n        \n    Text(\"a\") ¤\n";
    let project = Project::single(Url::parse("file:///t.wfx").unwrap(), src);
    assert!(wf_lsp::analysis::tokens_of(&project.files[0]).is_none());
    let at = src.find("        \n").unwrap() + 8;
    let pos = project.files[0].index.offset_to_position(src, at);
    let labels: Vec<String> = provide_completions(&project, 0, pos)
        .into_iter()
        .map(|c| c.label)
        .collect();
    offers(&labels, &["Button", "style"]);
}

#[test]
fn range_formatting_changes_only_the_lines_asked_for() {
    let src = "page P(path: \"/\") {\n  Text(\"a\")\n      Text(\"b\")\n}\n";
    let p = project(src);
    let edits = wf_lsp::formatting::format_range(
        &p,
        0,
        Range::new(Position::new(2, 0), Position::new(2, 3)),
    )
    .unwrap();
    assert_eq!(edits.len(), 1, "{edits:?}");
    assert_eq!(edits[0].range.start.line, 2);
    assert_eq!(edits[0].new_text, "    Text(\"b\")");
    let whole = wf_lsp::formatting::format_range(
        &p,
        0,
        Range::new(Position::new(0, 0), Position::new(4, 0)),
    )
    .unwrap();
    assert_eq!(whole.len(), 2, "{whole:?}");
}
