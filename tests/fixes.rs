//! C10: the decided fix of each diagnostic family (DIAGNOSTICS §2), held to
//! what the source reads once the fix is applied — through the build's own
//! pipeline, as `wf check`, the language server and the editor see it —
//! and to the program then being free of the finding.

use webfluent::diagnostics::Diagnostic;
use webfluent::diagnostics::fixes::{apply, fix_all};

/// Every finding for one file `t.wf` holding `src`, with `public/` holding
/// `files`, through the build's pipeline.
fn findings_with(src: &str, files: &[&str]) -> Vec<Diagnostic> {
    // A folder of its own: the tests run side by side, and two sources of
    // one length used to share one, each removing it under the other.
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "wf-fixes-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("public")).unwrap();
    for f in files {
        let at = dir.join("public").join(f);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, "x").unwrap();
    }
    let (program, mut out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}\n{src}");
    let source = src.to_string();
    out.extend(
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| Some(source.clone()),
            dir: Some(&dir),
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete: false,
        })
        .diagnostics,
    );
    let _ = std::fs::remove_dir_all(&dir);
    out
}

fn findings(src: &str) -> Vec<Diagnostic> {
    findings_with(src, &[])
}

/// `src` with the first fix of its first `code` finding applied, and the
/// fix's title.
fn fixed_with(src: &str, code: &str, files: &[&str]) -> (String, String) {
    let all = findings_with(src, files);
    let d = all.iter().find(|d| d.code == code).unwrap_or_else(|| {
        panic!(
            "no {code} in:\n{src}\nfound: {:?}",
            all.iter().map(|d| (d.code, &d.message)).collect::<Vec<_>>()
        )
    });
    let fix = d
        .fixes
        .first()
        .unwrap_or_else(|| panic!("{code} has no fix: {}\n{src}", d.message));
    (apply(src, fix), fix.title.clone())
}

fn fixed(src: &str, code: &str) -> (String, String) {
    fixed_with(src, code, &[])
}

/// The fix applied, and the finding gone from what it leaves.
fn fixes_it(src: &str, code: &str, want: &str) -> String {
    let (out, title) = fixed(src, code);
    assert_eq!(out, want, "{code}'s fix, `{title}`");
    let left: Vec<_> = findings(&out)
        .into_iter()
        .filter(|d| d.code == code)
        .collect();
    assert!(
        left.is_empty(),
        "{code} is still there after its fix:\n{out}\n{left:?}"
    );
    title
}

fn page(body: &str) -> String {
    format!(
        "page P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n{body}\n}}\n"
    )
}

// ─── T: types ──────────────────────────────────────────────────────────

#[test]
fn t02_the_nearest_case() {
    let head = "enum Tone { calm, loud }\n";
    let src = format!(
        "{head}{}",
        page("    state t: Tone = .clam\n    Text(\"{t}\")")
    );
    let title = fixes_it(&src, "T02", &src.replace(".clam", ".calm"));
    assert_eq!(title, "Change to `.calm`");
    // In a match arm too.
    let src = format!(
        "{head}{}",
        page(
            "    state t: Tone = .calm\n    match t {\n        .calm { Text(\"c\") }\n        .lod { Text(\"l\") }\n    }"
        )
    );
    fixes_it(&src, "T02", &src.replace(".lod", ".loud"));
}

#[test]
fn t17_await_where_await_may_be_written() {
    let src = "store S {\n    state n = \"\"\n    action load() {\n        let r = await fetch(\"/x\")\n        return r\n    }\n    action show() {\n        n = load().name\n    }\n}\npage P(path: \"/\", title: \"T\", description: \"D\") { Heading(\"T\").h1  Button(\"Go\") { on click { S.show() } }  Text(S.n) }\n";
    let (out, title) = fixed(src, "T17");
    assert_eq!(title, "Await it");
    assert!(out.contains("n = (await load()).name"), "{out}");
}

#[test]
fn t21_an_error_arm() {
    let src = page(
        "    resource r = fetch(\"/x\")\n    match r {\n        loading { Spinner }\n        ready(v) { Text(\"{v}\") }\n    }",
    );
    let want = src.replace(
        "        ready(v) { Text(\"{v}\") }\n    }",
        "        ready(v) { Text(\"{v}\") }\n        error(e) { Alert(e.message).danger }\n    }",
    );
    assert_eq!(fixes_it(&src, "T21", &want), "Add an `error` arm");
}

// ─── C: components ─────────────────────────────────────────────────────

#[test]
fn c02_c04_the_nearest_prop() {
    let comp = "component Panel(title: String) {\n    Text(title)\n}\n";
    let src = format!("{comp}{}", page("    Panel(titel: \"x\")"));
    assert_eq!(
        fixes_it(&src, "C02", &src.replace("titel:", "title:")),
        "Change to `title`"
    );
    let src = page("    Image(src: \"/a.png\", alt: \"a\", sizez: \"100vw\")");
    let (out, _) = fixed(&src, "C04");
    assert!(out.contains("sizes: \"100vw\""), "{out}");
}

#[test]
fn c05_an_argument_named_for_the_one_prop_it_can_be() {
    let src = page("    Tooltip(\"Save\") { Button(\"Save\") }\n    Badge(\"x\")");
    // `Tooltip` takes its text positionally; a component with one text prop
    // of its own and no positional one is named for it.
    let found = findings(&src);
    assert!(found.iter().all(|d| d.code != "C05"), "{found:?}");
    let src = page(
        "    Table(\"Deploys\") {\n        Table.Head { Table.Row { Table.Cell(\"Region\") } }\n    }",
    );
    let title = fixes_it(
        &src,
        "C05",
        &src.replace("Table(\"Deploys\")", "Table(caption: \"Deploys\")"),
    );
    assert_eq!(title, "Name it `caption:`");
    // Three text props: which one is not certain, so nothing is offered.
    let src = page("    Avatar(\"AL\")");
    let d = findings(&src)
        .into_iter()
        .find(|d| d.code == "C05")
        .expect("C05");
    assert!(d.fixes.is_empty(), "{:?}", d.fixes);
}

#[test]
fn c06_the_prop_is_marked_positional_in_its_declaration() {
    let src = format!(
        "component Panel(title: String) {{\n    Text(title)\n}}\n{}",
        page("    Panel(\"x\")")
    );
    let title = fixes_it(
        &src,
        "C06",
        &src.replace(
            "component Panel(title: String)",
            "component Panel(_ title: String)",
        ),
    );
    assert_eq!(title, "Mark `title` positional in `Panel`");
}

// ─── R: routes ─────────────────────────────────────────────────────────

const ABOUT: &str =
    "page About(path: \"/about\", title: \"A\", description: \"D\") { Heading(\"A\").h1 }\n";

#[test]
fn r01_the_nearest_route_and_the_root() {
    let src = format!("{ABOUT}{}", page("    Link(\"About\", to: \"/abuot\")"));
    assert_eq!(
        fixes_it(&src, "R01", &src.replace("\"/abuot\"", "\"/about\"")),
        "Change to `/about`"
    );
    let src = format!("{ABOUT}{}", page("    Link(\"About\", to: \"about\")"));
    fixes_it(&src, "R01", &src.replace("to: \"about\"", "to: \"/about\""));
    // A link with a value spliced in is not guessed at.
    let src = format!(
        "{ABOUT}{}",
        page("    state id = 1\n    Link(\"x\", to: \"/abuot/{id}\")")
    );
    let d = findings(&src)
        .into_iter()
        .find(|d| d.code == "R01")
        .expect("R01");
    assert!(d.fixes.is_empty(), "{:?}", d.fixes);
}

#[test]
fn r03_a_router_where_there_is_none() {
    let src = format!("{ABOUT}app {{\n    Navbar {{ Text(\"x\") }}\n}}\n");
    let title = fixes_it(
        &src,
        "R03",
        &src.replace(
            "    Navbar { Text(\"x\") }\n}",
            "    Navbar { Text(\"x\") }\n    Router\n}",
        ),
    );
    assert_eq!(title, "Add `Router`");
    let two = format!("{ABOUT}app {{\n    Router\n    Router\n}}\n");
    let d = findings(&two)
        .into_iter()
        .find(|d| d.code == "R03")
        .expect("R03");
    assert!(d.fixes.is_empty(), "with two, neither is guessed");
}

// ─── F: forms ──────────────────────────────────────────────────────────

#[test]
fn f04_bind_in_place_of_the_one_way_prop() {
    let src = page("    state on = false\n    Checkbox(checked: on, label: \"On\")");
    assert_eq!(
        fixes_it(&src, "F04", &src.replace("checked: on", "bind: on")),
        "Change to `bind:`"
    );
    let src = page("    state n = 0\n    Checkbox(checked: n > 1, label: \"Many\")");
    let d = findings(&src)
        .into_iter()
        .find(|d| d.code == "F04")
        .expect("F04");
    assert!(d.fixes.is_empty(), "a condition cannot be bound");
}

// ─── X: state ──────────────────────────────────────────────────────────

#[test]
fn x05_the_nearest_store() {
    let src = format!(
        "store Cart {{\n    state n = 0\n}}\n{}",
        page("    use Crat\n    Text(\"{Cart.n}\")")
    );
    fixes_it(&src, "X05", &src.replace("use Crat", "use Cart"));
}

// ─── U: never used ─────────────────────────────────────────────────────

#[test]
fn u09_keyed_by_the_id_the_body_reads() {
    let src = page(
        "    state rows: [Map] = []\n    for r in rows {\n        Input(placeholder: r.id)\n    }",
    );
    let title = fixes_it(
        &src,
        "U09",
        &src.replace("for r in rows {", "for r in rows by r.id {"),
    );
    assert_eq!(title, "Key it `by r.id`");
    let src = page(
        "    state rows: [Map] = []\n    for r in rows {\n        Input(placeholder: r.name)\n    }",
    );
    let d = findings(&src)
        .into_iter()
        .find(|d| d.code == "U09")
        .expect("U09");
    assert!(d.fixes.is_empty(), "no `id` read: not certain it has one");
}

#[test]
fn u06_the_dead_line_goes() {
    let src = "store S {\n    state n = 0\n    action go() {\n        return null\n        n = 1\n    }\n}\npage P(path: \"/\", title: \"T\", description: \"D\") { Heading(\"T\").h1  Button(\"Go\") { on click { S.go() } }  Text(\"{S.n}\") }\n";
    let want = src.replace(
        "        return null\n        n = 1\n",
        "        return null\n",
    );
    fixes_it(src, "U06", &want);
}

// ─── V: vocabulary and styles ──────────────────────────────────────────

#[test]
fn v07_a_length_gets_its_unit() {
    let src = page(
        "    state w = 40\n    Container {\n        style { width: {w} }\n        Text(\"x\")\n    }",
    );
    assert_eq!(
        fixes_it(&src, "V07", &src.replace("width: {w}", "width: {w}px")),
        "Add `px`"
    );
}

#[test]
fn v08_v09_the_nearest_icon_and_event() {
    let src = page("    Icon(\"settigns\")");
    fixes_it(&src, "V08", &src.replace("settigns", "settings"));
    let src = page("    Button(\"Go\") {\n        on clik { log(1) }\n    }");
    assert_eq!(
        fixes_it(&src, "V09", &src.replace("on clik", "on click")),
        "Change to `click`"
    );
}

// ─── A: accessibility ──────────────────────────────────────────────────

/// What a finding of `code` offers, and that it is not offered empty.
fn no_fix(src: &str, code: &str) {
    let d = findings(src)
        .into_iter()
        .find(|d| d.code == code)
        .unwrap_or_else(|| panic!("no {code}"));
    assert!(d.fixes.is_empty(), "{code} offers {:?}", d.fixes);
}

#[test]
fn a02_a03_a04_a06_a08_a_name_drafted_from_the_code() {
    let src = page("    IconButton(icon: \"arrow-left\")");
    assert_eq!(
        fixes_it(
            &src,
            "A02",
            &src.replace(
                "IconButton(icon: \"arrow-left\")",
                "IconButton(icon: \"arrow-left\", label: \"Arrow left\")"
            )
        ),
        "Add `label: \"Arrow left\"`"
    );
    let src = page("    state emailAddress = \"\"\n    Input(bind: emailAddress).email");
    fixes_it(
        &src,
        "A03",
        &src.replace(
            "Input(bind: emailAddress)",
            "Input(bind: emailAddress, label: \"Email address\")",
        ),
    );
    let src = page("    state reuse_cache = false\n    Switch(bind: reuse_cache)");
    fixes_it(
        &src,
        "A04",
        &src.replace(
            "Switch(bind: reuse_cache)",
            "Switch(bind: reuse_cache, label: \"Reuse cache\")",
        ),
    );
    let src = page("    Link(to: \"/docs/getting-started\") { on click { log(1) } }");
    fixes_it(
        &src,
        "A06",
        &src.replace(
            "Link(to: \"/docs/getting-started\")",
            "Link(to: \"/docs/getting-started\", aria-label: \"Getting started\")",
        ),
    );
    let src = page(
        "    state confirmDelete = false\n    Modal(visible: confirmDelete) { Text(\"Sure?\") }",
    );
    fixes_it(
        &src,
        "A08",
        &src.replace(
            "Modal(visible: confirmDelete)",
            "Modal(visible: confirmDelete, title: \"Confirm delete\")",
        ),
    );
}

#[test]
fn a_name_with_nothing_to_draft_it_from_is_left_to_a_person() {
    // An empty name would silence the warning and name nothing.
    no_fix(&page("    Input(id: \"q\").text"), "A03");
    no_fix(&page("    Button { Icon(\"close\") }"), "A05");
    no_fix(&page("    Heading(\"\").h2"), "A07");
    no_fix(&page("    Video(src: \"/a.mp4\", controls: true)"), "A09");
}

#[test]
fn a09_controls_and_a11_the_level_that_mends_the_outline() {
    let src = page("    Video(src: \"/a.mp4\", captions: \"/a.vtt\")");
    let (out, title) = fixed(&src, "A09");
    assert_eq!(title, "Add `controls: true`");
    assert!(
        out.contains("Video(src: \"/a.mp4\", captions: \"/a.vtt\", controls: true)"),
        "{out}"
    );
    let src = page("    Heading(\"Part\").h4");
    assert_eq!(
        fixes_it(&src, "A11", &src.replace(".h4", ".h2")),
        "Change to `.h2`"
    );
}

#[test]
fn a15_the_label_starts_with_what_it_shows() {
    let src = page("    Button(\"Save\", aria-label: \"Store the draft\")");
    let title = fixes_it(
        &src,
        "A15",
        &src.replace("\"Store the draft\"", "\"Save: Store the draft\""),
    );
    assert_eq!(title, "Start it with \"Save\"");
}

// ─── U: removing what nothing uses ─────────────────────────────────────

#[test]
fn u01_u02_u05_remove_first_and_keep_as_the_second() {
    let src = page("    state unused = 1\n    derived twice = 2\n    action never() { log(1) }");
    let all = findings(&src);
    for (code, name) in [("U01", "unused"), ("U02", "twice"), ("U05", "never")] {
        let d = all
            .iter()
            .find(|d| d.code == code)
            .unwrap_or_else(|| panic!("no {code}"));
        let titles: Vec<&str> = d.fixes.iter().map(|f| f.title.as_str()).collect();
        assert_eq!(
            titles,
            [format!("Remove `{name}`"), format!("Rename to `_{name}`")],
            "{code}"
        );
    }
    let (out, _) = fixed(&src, "U01");
    assert!(!out.contains("state unused"), "{out}");
    assert!(out.contains("derived twice = 2"), "only it goes: {out}");
    let (out, _) = fixed(&src, "U05");
    assert!(
        !out.contains("action never"),
        "the action and its block: {out}"
    );
}

#[test]
fn u03_u04_a_component_and_a_store_member_nothing_uses() {
    let src = format!(
        "component Lonely(title: String) {{\n    Text(title)\n}}\nstore S {{\n    state kept = 1\n    state dead = 2\n}}\n{}",
        page("    Text(\"{S.kept}\")")
    );
    let (out, title) = fixed(&src, "U03");
    assert_eq!(title, "Remove `Lonely`");
    assert!(
        !out.contains("component Lonely") && !out.contains("Text(title)"),
        "{out}"
    );
    let (out, title) = fixed(&src, "U04");
    assert_eq!(title, "Remove `S.dead`");
    assert!(
        !out.contains("state dead") && out.contains("state kept = 1"),
        "{out}"
    );
}

#[test]
fn u07_an_allow_that_allows_nothing_goes() {
    let src = page("    // wf-allow(U01)\n    Text(\"x\")");
    assert_eq!(
        fixes_it(&src, "U07", &src.replace("    // wf-allow(U01)\n", "")),
        "Remove the allow"
    );
    let src = page("    state n = 1\n    Text(\"{n}\") // wf-allow(U01)");
    fixes_it(&src, "U07", &src.replace(" // wf-allow(U01)", ""));
    // One code of two allowed nothing: the other stays.
    let src = page("    // wf-allow(U01, A11)\n    state n = 1");
    let (out, title) = fixed(&src, "U07");
    assert_eq!(title, "Take `A11` out of the allow");
    assert!(out.contains("// wf-allow(U01)\n    state n = 1"), "{out}");
}

#[test]
fn v05_v06_the_nearest_property_and_token() {
    let src = page("    Card {\n        style { colr: red }\n        Text(\"x\")\n    }");
    assert_eq!(
        fixes_it(&src, "V05", &src.replace("colr: red", "color: red")),
        "Change to `color`"
    );
    let src = page("    Card {\n        style { backgroundColr: red }\n        Text(\"x\")\n    }");
    fixes_it(
        &src,
        "V05",
        &src.replace("backgroundColr", "background-color"),
    );
    let src =
        page("    Card {\n        style { background: $colr-primary }\n        Text(\"x\")\n    }");
    let (out, _) = fixed(&src, "V06");
    assert!(out.contains("$color-primary"), "{out}");
}

// ─── Fix-all ───────────────────────────────────────────────────────────

#[test]
fn a_family_in_a_file_is_fixed_as_one_change_set() {
    let src = page(
        "    state w = 40\n    Container {\n        style { width: {w} }\n        Text(\"a\")\n    }\n    Container {\n        style { height: {w} }\n        Text(\"b\")\n    }",
    );
    let all = findings(&src);
    let change = fix_all(&all, "V07", "t.wf").expect("a change set");
    assert_eq!(change.title, "Fix all 2 V07 in t.wf");
    let out = apply(&src, &change);
    assert!(
        out.contains("width: {w}px") && out.contains("height: {w}px"),
        "{out}"
    );
    assert!(findings(&out).iter().all(|d| d.code != "V07"));
}
