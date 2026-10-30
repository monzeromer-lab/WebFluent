//! The checks of `spec/DIAGNOSTICS_PLAN.md`, Part C: what each new code
//! reports, where, and — as importantly — what it leaves alone.

use webfluent::diagnostics::Diagnostic;

/// Every finding for `src`, through the build's pipeline.
fn findings(src: &str) -> Vec<Diagnostic> {
    let (program, mut out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    let source = src.to_string();
    out.extend(
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| Some(source.clone()),
            dir: None,
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete: false,
        })
        .diagnostics,
    );
    out
}

/// The findings with `code`, as `line: message`.
fn with(src: &str, code: &str) -> Vec<String> {
    findings(src)
        .into_iter()
        .filter(|d| d.code == code)
        .map(|d| format!("{}: {}", d.line, d.message))
        .collect()
}

fn page(body: &str) -> String {
    format!(
        "page P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n{body}\n}}\n"
    )
}

/// `src` draws no errors at all.
fn clean(src: &str) {
    let errors: Vec<String> = findings(src)
        .into_iter()
        .filter(|d| d.is_error())
        .map(|d| d.to_string())
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

// ─── C1: X01 ──────────────────────────────────────────────────────────

#[test]
fn x01_a_constant_a_derived_value_and_an_action_cannot_be_assigned() {
    let src = format!(
        "const LIMIT = 3\n{}",
        page(
            "    state n = 1\n    derived double = n * 2\n    action go() { log(1) }\n    Button(\"x\") { on click { LIMIT = 4  double = 3  go = 1  n = 2 } }"
        )
    );
    let found = with(&src, "X01");
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found[0].contains("`LIMIT` is a `const`"), "{found:?}");
    assert!(
        found[1].contains("`double` is a `derived` value"),
        "{found:?}"
    );
    assert!(found[2].contains("`go` is an action"), "{found:?}");
}

#[test]
fn x01_a_prop_a_route_parameter_and_a_loop_variable_cannot_be_assigned() {
    let comp =
        "component Chip(_ label: String) {\n    Button(label) { on click { label = \"x\" } }\n}\n";
    assert!(with(comp, "X01")[0].contains("`label` is a prop"));
    let route = "page U(path: \"/u/:id\", title: \"T\", description: \"D\", id: String) {\n    Heading(id).h1\n    Button(\"x\") { on click { id = \"2\" } }\n}\n";
    assert!(with(route, "X01")[0].contains("`id` is a route parameter"));
    let looped = page(
        "    state items = [1, 2]\n    for it in items { Button(\"x\") { on click { it = 3 } } }",
    );
    assert!(with(&looped, "X01")[0].contains("`it` is a loop variable"));
}

#[test]
fn x01_a_stores_derived_value_cannot_be_assigned_from_anywhere() {
    let src = format!(
        "store Cart {{\n    state items = [1]\n    derived count = items.length\n    action clear() {{ count = 0 }}\n}}\n{}",
        page("    use Cart\n    Button(\"x\") { on click { Cart.count = 5  Cart.items = [] } }")
    );
    let found = with(&src, "X01");
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found.iter().all(|f| f.contains("`derived` value")),
        "{found:?}"
    );
}

#[test]
fn x01_a_null_check_does_not_make_a_derived_value_writable() {
    let src = page(
        "    state xs = [1]\n    derived first = xs.first()\n    Button(\"x\") { on click { if first != null { first = 2 } } }",
    );
    assert_eq!(with(&src, "X01").len(), 1);
}

#[test]
fn x01_leaves_states_lets_and_parameters_alone() {
    clean(&page(
        "    state n = 0\n    persist k = 1\n    action go(v: Number) { let next = v  next = next + 1  v = 2  n = next  k = n }\n    Button(\"x\") { on click { go(1) } }",
    ));
}

// ─── C2: types ───────────────────────────────────────────────────────

#[test]
fn a_derived_values_annotation_is_held_to_what_it_works_out_to() {
    let found = with(
        &page("    state n = 1\n    derived label: String = n * 2\n    Text(label)"),
        "T01",
    );
    assert!(
        found[0].contains("`label` is `Number`, but `String` is wanted"),
        "{found:?}"
    );
}

#[test]
fn a_record_is_held_to_its_fields() {
    let src = format!(
        "type Todo {{ id: String, title: String, done: Bool = false, note: String? }}\n{}",
        page(
            "    state a = Todo(id: \"1\", title: 2)\n    state b = Todo(id: \"1\", title: \"x\", colour: \"red\")\n    state c = Todo(title: \"x\")\n    state d: Todo = { title: \"y\" }\n    Text(\"{a.id}{b.id}{c.id}{d.id}\")"
        )
    );
    assert!(
        with(&src, "T01")
            .iter()
            .any(|f| f.contains("`title` of `Todo` is `String`")),
        "{:?}",
        with(&src, "T01")
    );
    assert!(
        with(&src, "T05")
            .iter()
            .any(|f| f.contains("`Todo` has no field `colour`"))
    );
    let missing = with(&src, "C01");
    assert_eq!(missing.len(), 2, "{missing:?}");
    assert!(
        missing[0].contains("leaves out `id` of `Todo`"),
        "{missing:?}"
    );
    // A field with a default, or one that may be null, may be left out.
    assert!(
        !missing
            .iter()
            .any(|m| m.contains("`done`") || m.contains("`note`"))
    );
}

#[test]
fn a_record_whose_fields_all_have_defaults_is_built_with_none_given() {
    // Wherever an expression is: a prop's default, a field's, a constant,
    // a state, an argument.
    let src = format!(
        "type Offers {{ definition: Bool = false, references: Bool = false }}\ntype Card {{ offers: Offers = Offers(), name: String = \"n\" }}\nconst NONE = Offers()\ncomponent Box(offers: Offers = Offers()) {{\n    Text(\"{{offers.definition}}\")\n}}\n{}",
        page(
            "    state c = Card()\n    Box(offers: Offers())\n    Text(\"{c.name}{NONE.references}\")"
        )
    );
    clean(&src);
    // One with a field that must be given is held to it, as with any
    // argument left out; a function that is not a record is still one.
    let src = format!(
        "type Todo {{ id: String, done: Bool = false }}\n{}",
        page("    state t = Todo()\n    state u = Nobody()\n    Text(\"{t.id}{u}\")")
    );
    let missing = with(&src, "C01");
    assert!(
        missing.len() == 1 && missing[0].contains("leaves out `id` of `Todo`"),
        "{missing:?}"
    );
    let unresolved = with(&src, "T13");
    assert!(
        unresolved.len() == 1 && unresolved[0].contains("`Nobody(…)`"),
        "{unresolved:?}"
    );
}

#[test]
fn a_local_in_a_store_action_is_its_own_not_the_member_it_shadows() {
    // `refs` is a list of the store's and `twice` a derived value; the
    // action's own of the same names are a number, written as it likes.
    let src = format!(
        "type Ref {{ id: String }}\nstore Refs {{\n    state refs: [Ref] = []\n    state count = 0\n    derived twice = count * 2\n    action add(x: Number) {{\n        let refs = x\n        count = refs + 1\n        let twice = refs\n        twice = twice + 1\n        for refs in [\"a\"] {{ log(refs.toUpperCase()) }}\n        count = [1].map(refs => refs + 1).length\n    }}\n}}\n{}",
        page(
            "    use Refs\n    Text(\"{Refs.count} {Refs.twice} {Refs.refs.length}\")\n    Button(\"Go\") { on click { Refs.add(1) } }"
        )
    );
    clean(&src);
}

#[test]
fn a_case_fits_an_enum_only_when_the_enum_has_it_and_a_function_its_arguments() {
    let src = format!(
        "enum Tone {{ calm, loud }}\n{}",
        page("    state t: Tone = .calm\n    Button(\"x\") { on click { t = .quiet } }")
    );
    assert!(with(&src, "T02")[0].contains("`Tone` has no case `.quiet`"));
    // A comparator that takes three arguments where two are given.
    let three = page(
        "    state xs = [3, 1]\n    derived sorted = xs.slice().sort((a, b, c) => a - b)\n    Text(\"{sorted}\")",
    );
    assert!(
        with(&three, "T10")[0].contains("takes 3 arguments, but it is given 2"),
        "{:?}",
        findings(&three)
    );
    clean(&page(
        "    state xs = [3, 1]\n    derived sorted = xs.slice().sort((a, b) => a - b)\n    derived big = xs.filter(x => x > 1)\n    Text(\"{sorted}{big}\")",
    ));
}

#[test]
fn t16_a_method_nothing_has_is_refused_with_the_nearest() {
    let src = page(
        "    state xs = [\"a\"]\n    state s = \"a\"\n    state n = 1\n    Text(xs.joined(\", \"))\n    Text(s.toUpper2())\n    Text(n.toFixd(2))",
    );
    let found = with(&src, "T16");
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(
        findings(&src)
            .iter()
            .any(|d| d.hint.as_deref() == Some("Did you mean `join`?"))
    );
    clean(&page(
        "    state xs = [\"a\"]\n    Text(xs.toSorted().at(0) ?? \"\")\n    Text(\"a\".padStart(3).capitalize())\n    Text(xs.unique().take(1).join(\"\"))",
    ));
}

#[test]
fn t14_a_comparison_that_can_never_change_is_refused() {
    let src = format!(
        "enum Tone {{ calm, loud }}\n{}",
        page(
            "    state n = 0\n    state t: Tone = .calm\n    if n == \"0\" { Text(\"a\") }\n    if t == .quiet { Text(\"b\") }\n    if t != \"calm\" { Text(\"c\") }"
        )
    );
    let found = with(&src, "T14");
    assert_eq!(
        found.len(),
        2,
        "an enum against its own case's name is fine: {found:?}"
    );
    assert!(found[0].contains("always false"), "{found:?}");
}

#[test]
fn t18_arithmetic_takes_numbers() {
    let src = page(
        "    state s = \"12\"\n    state xs = [1]\n    Text(\"{s * 2}\")\n    Text(\"{xs - 1}\")\n    Text(\"{xs + 1}\")",
    );
    assert_eq!(with(&src, "T18").len(), 3, "{:?}", with(&src, "T18"));
    clean(&page(
        "    state s = \"12\"\n    state n = 2\n    Text(\"{s + n}\")\n    Text(\"{n * 2 - 1}\")",
    ));
}

#[test]
fn t17_await_belongs_in_an_async_body_and_a_promise_is_awaited() {
    let derived = page("    derived d = await fetch(\"/x\")\n    Text(\"{d}\")");
    assert!(with(&derived, "T17")[0].contains("a `derived` value cannot `await`"));
    let effect = page(
        "    state n = 0\n    effect { let r = await fetch(\"/x\")  n = r.n }\n    Text(\"{n}\")",
    );
    assert_eq!(with(&effect, "T17").len(), 1);
    let promise = page(
        "    state n = 0\n    action load() { let r = await fetch(\"/x\")  return r.n }\n    Button(\"x\") { on click { n = load() } }",
    );
    assert!(with(&promise, "T17")[0].contains("an async action's result"));
    clean(&page(
        "    state n = 0\n    action load() { let r = await fetch(\"/x\")  return r.n }\n    Button(\"x\", disabled: load.pending) { on click { n = await load() } }",
    ));
}

#[test]
fn event_value_and_key_are_names_only_inside_a_handler() {
    let found = with(&page("    Text(\"{event}\")"), "T13");
    assert_eq!(found.len(), 1, "{found:?}");
    clean(&page(
        "    state k = \"\"\n    Input(bind: k, label: \"K\") { on keydown { k = event.key } }",
    ));
}

#[test]
fn a_refined_state_is_held_to_its_type_at_every_assignment() {
    let found = with(
        &page(
            "    state nights: Number(1..=30) = 12\n    Button(\"x\") { on click { nights = 40 } }\n    Text(\"{nights}\")",
        ),
        "T01",
    );
    assert_eq!(found.len(), 1, "{found:?}");
}

// ─── C3: null ────────────────────────────────────────────────────────

#[test]
fn an_item_at_a_fixed_index_may_not_be_there_unless_a_condition_says_so() {
    let src = page("    state todos = [{ title: \"a\" }]\n    Text(todos[0].title)");
    assert_eq!(with(&src, "T04").len(), 1);
    clean(&page(
        "    state todos = [{ title: \"a\" }]\n    if todos.length > 0 { Text(todos[0].title) }\n    for todo, i in todos { Text(todos[i].title) }",
    ));
}

#[test]
fn a_chain_keeps_each_fields_own_null() {
    let src = format!(
        "type Sel {{ title: String, note: String? }}\n{}",
        page(
            "    state sel: Sel? = null\n    Text(\"{sel?.note.length}\")\n    Text(\"{sel?.title.length}\")"
        )
    );
    let found = with(&src, "T04");
    assert_eq!(found.len(), 1, "only `note` may be null: {found:?}");
    assert!(found[0].contains("`sel?.note` may be null"), "{found:?}");
}

#[test]
fn an_else_and_an_early_return_narrow_out_null() {
    let src = format!(
        "type Sel {{ title: String }}\n{}",
        page(
            "    state sel: Sel? = null\n    action go() {\n        if sel == null { return }\n        log(sel.title)\n    }\n    if sel == null { Text(\"none\") } else { Text(sel.title) }\n    Button(\"x\") { on click { go() } }"
        )
    );
    clean(&src);
}

#[test]
fn t19_a_value_that_may_be_null_shown_as_text_draws_a_warning() {
    let src = format!(
        "type U {{ nick: String? }}\n{}",
        page(
            "    state u = U(nick: null)\n    Text(\"Also {u.nick}\")\n    Text(\"Also {u.nick ?? \\\"-\\\"}\")"
        )
    );
    assert_eq!(with(&src, "T19").len(), 1);
    assert!(
        !findings(&src)
            .iter()
            .any(|d| d.code == "T19" && d.is_error())
    );
}

// ─── C4: match ───────────────────────────────────────────────────────

const STATUS: &str = "enum Status { draft, review, live }\n";

#[test]
fn t15_a_match_that_misses_a_case_or_has_one_twice() {
    let missing = format!(
        "{STATUS}{}",
        page(
            "    state s: Status = .draft\n    match s {\n        .draft { Text(\"d\") }\n        .live { Text(\"l\") }\n    }"
        )
    );
    assert!(
        with(&missing, "T15")[0].contains("no arm for `.review`"),
        "{:?}",
        with(&missing, "T15")
    );
    let twice = format!(
        "{STATUS}{}",
        page(
            "    state s: Status = .draft\n    match s {\n        .draft { Text(\"d\") }\n        .draft { Text(\"again\") }\n        else { Text(\"x\") }\n    }"
        )
    );
    assert!(with(&twice, "T15")[0].contains("`.draft` has two arms"));
    let expr = format!(
        "{STATUS}{}",
        page(
            "    state s: Status = .draft\n    derived l = match s { .draft { \"d\" } .live { \"l\" } }\n    Text(l)"
        )
    );
    assert!(with(&expr, "T15")[0].contains("no arm for `.review`"));
}

#[test]
fn a_match_that_covers_every_case_needs_no_else_and_compiles() {
    let src = format!(
        "{STATUS}{}",
        page(
            "    state s: Status = .draft\n    derived l = match s { .draft { \"d\" } .review { \"r\" } .live { \"l\" } }\n    Text(l)\n    match s {\n        .draft { Text(\"d\") }\n        .review { Text(\"r\") }\n        .live { Text(\"l\") }\n    }"
        )
    );
    clean(&src);
    let program = webfluent::parse_source(&src, "t.wf").unwrap();
    let js = webfluent::codegen::js::JsCodegen::new().generate(&program);
    assert!(!js.contains("__exhaustive"), "the marker compiles to null");
    assert_eq!(webfluent::codegen::jscheck::check_js(&js), Ok(()));
}

#[test]
fn u10_an_else_no_value_reaches_and_t21_a_resource_with_no_error_arm() {
    let src = format!(
        "{STATUS}{}",
        page(
            "    state s: Status = .draft\n    match s {\n        .draft { Text(\"d\") }\n        .review { Text(\"r\") }\n        .live { Text(\"l\") }\n        else { Text(\"?\") }\n    }\n    resource users = fetch(\"/u\")\n    match users {\n        loading { Spinner }\n        ready(list) { Text(\"{list.length}\") }\n    }"
        )
    );
    assert_eq!(with(&src, "U10").len(), 1);
    assert_eq!(with(&src, "T21").len(), 1);
}

#[test]
fn a_match_on_a_value_of_unknown_type_needs_its_else() {
    let src =
        page("    state s = getStatus()\n    derived l = match s { .a { 1 } }\n    Text(\"{l}\")");
    assert!(
        findings(&src)
            .iter()
            .any(|d| d.code == "T15" && d.message.contains("has no `else`")),
        "{:?}",
        findings(&src)
    );
}

// ─── C5: components ──────────────────────────────────────────────────

#[test]
fn c01_a_required_prop_left_out_of_a_call_or_a_layout() {
    let src = format!(
        "enum Tone {{ calm, loud }}\ncomponent Card2(_ title: String, price: Number, tone: Tone, on: Bool, note: String?) {{ Text(title) }}\ncomponent Shell(crumb: String) {{ slot  children }}\npage Q(path: \"/q\", title: \"T\", description: \"D\", layout: Shell) {{ Heading(\"Q\").h1 }}\n{}",
        page("    Card2(\"a\")\n    Card2(\"b\", price: 2).loud")
    );
    let found = with(&src, "C01");
    // The second call gives `price`, and `.loud` gives `tone`; `on` is a
    // `Bool` and `note` may be null, so neither is needed.
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found
            .iter()
            .any(|f| f.contains("`Card2` is placed without `price`, `tone`")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|f| f.contains("`Shell` is placed without `crumb`")),
        "{found:?}"
    );
}

#[test]
fn c02_a_prop_your_component_does_not_declare_is_an_error() {
    let src = format!(
        "component Chip(_ label: String) {{ Text(label) }}\n{}",
        page("    Chip(\"a\", colour: \"red\")")
    );
    assert!(
        findings(&src)
            .iter()
            .any(|d| d.code == "C02" && d.is_error())
    );
}

#[test]
fn c07_a_drag_on_a_component_with_no_one_element() {
    // Each body draws something other than one element.
    for (body, said) in [
        ("if on { Row { Text(name) } }", "an `if`"),
        ("for x in [1, 2] { Text(name) }", "a `for`"),
        ("Text(name)\n    Text(name)", "several elements"),
        ("Many2(name)", "a `for`"),
    ] {
        let src = format!(
            "component Many2(_ name: String) {{ for x in [1, 2] {{ Text(name) }} }}\ncomponent Row2(_ name: String, on: Bool = true) {{\n    {body}\n}}\n{}",
            page("    Row2(\"a\", drag: \"a\", dragGroup: \"g\")")
        );
        let found = with(&src, "C07");
        assert_eq!(found.len(), 1, "{body}: {found:?}");
        assert!(
            found[0].contains("`drag:` on `Row2` has no element to land on")
                && found[0].contains(said),
            "{found:?}"
        );
    }
    // A gesture's handler is the root's too.
    let src = format!(
        "component Row2(_ name: String) {{ if true {{ Text(name) }} }}\n{}",
        page("    Row2(\"a\") { on receive(d) { log(d.value) } }")
    );
    let found = with(&src, "C07");
    assert!(
        found.len() == 1 && found[0].contains("`on receive` on `Row2`"),
        "{found:?}"
    );
    // One element — through another component's call, and beside the state
    // and actions it declares — is a root; a prop or an event the component
    // takes itself is its own.
    clean(&format!(
        "component Inner2(_ name: String) {{ Row {{ Text(name) }} }}\ncomponent Row2(_ name: String) {{\n    state open = false\n    Inner2(name)\n}}\ncomponent Many2(drag: String) {{\n    event receive(d: String)\n    for x in [1, 2] {{ Text(drag) }}\n}}\n{}",
        page(
            "    Row2(\"a\", drag: \"a\", dragGroup: \"g\", dropInside: true) { on receive(d) { log(d.value) } on dragmove(d) { log(d.position) } }\n    Many2(drag: \"b\") { on receive(d) { log(d) } }"
        )
    ));
}

#[test]
fn c03_a_part_outside_its_owner_and_e101_in_every_position() {
    let src = page(
        "    Select.Option(\"a\")\n    Card { Card.Header { Text(\"h\") } }\n    Button(\"x\") { on click { log(1) } }",
    );
    assert_eq!(with(&src, "C03").len(), 1);
    // A part below one of your components may well be inside its owner.
    clean(&format!(
        "component Picker {{ Select {{ children }} }}\n{}",
        page("    Picker { Select.Option(\"a\") }")
    ));
    let fill = format!(
        "component Panel {{ slot trailing  Card {{ trailing }} }}\n{}",
        page("    Panel { trailing { Bdage(\"x\") } }")
    );
    assert!(with(&fill, "E101")[0].contains("`Bdage`"));
}

// ─── C6: routes ──────────────────────────────────────────────────────

#[test]
fn r01_links_and_navigation_are_held_to_the_routes() {
    let src = "page Home(path: \"/\", title: \"T\", description: \"D\") {\n    Heading(\"H\").h1\n    Link(\"a\", to: \"/abuot\")\n    Link(\"b\", to: \"about\")\n    Link(\"c\", to: \"/team/{slug}\")\n    Link(\"d\", to: \"/team\")\n    Link(\"e\", to: \"/report.pdf\")\n    Link(\"f\", to: \"https://example.com/x\")\n    Link(\"g\", to: \"/about?tab=1#top\")\n    Button(\"x\") { on click { navigate(\"/nowhere\") } }\n}\npage About(path: \"/about\", title: \"T\", description: \"D\") { Heading(\"A\").h1 }\npage Team(path: \"/team/:slug\", title: \"T\", description: \"D\", slug: String) { Heading(slug).h1 }\n";
    let found = with(src, "R01");
    assert_eq!(found.len(), 4, "{found:?}");
    assert!(found[0].contains("`/abuot` is not a route"), "{found:?}");
    assert!(found[1].contains("does not begin with `/`"), "{found:?}");
    assert!(found[2].contains("`/team` is not a route"), "{found:?}");
    assert!(found[3].contains("`/nowhere`"), "{found:?}");
}

#[test]
fn r02_r03_r04_route_parameters_the_router_and_relative_urls() {
    let params = "page U(path: \"/u/:id\", title: \"T\", description: \"D\") { Heading(\"u\").h1 }\npage V(path: \"/v\", title: \"T\", description: \"D\", id: String) { Heading(id).h1 }\n";
    assert_eq!(with(params, "R02").len(), 2);
    let no_router = "app { Text(\"chrome\") }\npage P(path: \"/\", title: \"T\", description: \"D\") { Heading(\"p\").h1 }\n";
    assert!(with(no_router, "R03")[0].contains("no `Router`"));
    let two = "app { Router  Router }\npage P(path: \"/\", title: \"T\", description: \"D\") { Heading(\"p\").h1 }\n";
    assert!(with(two, "R03")[0].contains("2 `Router`s"));
    clean("page P(path: \"/\", title: \"T\", description: \"D\") { Heading(\"p\").h1 }\n");
    let nested = "page P(path: \"/blog/a\", title: \"T\", description: \"D\") {\n    Heading(\"p\").h1\n    Image(src: \"img/a.png\", alt: \"\")\n    Image(src: \"/img/b.png\", alt: \"\")\n}\n";
    assert_eq!(with(nested, "R04").len(), 1);
}

#[test]
fn s04_two_pages_on_one_route_is_an_error() {
    let src = "page A(path: \"/\", title: \"T\", description: \"D\") { Heading(\"a\").h1 }\npage B(path: \"/\", title: \"T\", description: \"D\") { Heading(\"b\").h1 }\n";
    assert!(
        findings(src)
            .iter()
            .any(|d| d.code == "S04" && d.is_error())
    );
}

// ─── C7: state ───────────────────────────────────────────────────────

#[test]
fn x02_an_effect_that_writes_what_it_reads_and_x04_a_derived_that_assigns() {
    let feeds = page("    state n = 0\n    effect { n = n + 1 }\n    Text(\"{n}\")");
    assert_eq!(with(&feeds, "X02").len(), 1);
    // Through an action it calls.
    let through = page(
        "    state n = 0\n    action bump() { n = n + 1 }\n    effect { log(n)  bump() }\n    Text(\"{n}\")",
    );
    assert_eq!(with(&through, "X02").len(), 1);
    // A guarded write settles; a write of what it does not read is fine.
    clean(&page(
        "    state n = 0\n    state seen = 0\n    effect { if n < 3 { n = n + 1 } }\n    effect { seen = n }\n    Text(\"{n}{seen}\")",
    ));
    let derived = page(
        "    state n = 0\n    action next() {\n        n = n + 1\n        return n\n    }\n    derived t = next()\n    Text(\"{t}\")",
    );
    assert_eq!(with(&derived, "X04").len(), 1);
}

#[test]
fn x05_use_of_a_store_nothing_declares() {
    let src = format!(
        "store Cart {{ state items = [] }}\n{}",
        page("    use Carts")
    );
    assert!(with(&src, "X05")[0].contains("`use Carts`"));
}

#[test]
fn x06_a_state_that_copies_a_live_value_nothing_changes() {
    let src = String::from(
        "store Clock {\n    state started = now\n}\ncomponent Wide {\n    state wide = viewport.md\n    Text(if wide { \"w\" } else { \"n\" })\n}\n",
    ) + &page(
        "    state clock = now(every: 1.seconds)\n    state tab = if let t = hash { t } else { \"a\" }\n    state dark = theme == \"dark\"\n    state online = !network.online\n    state fresh = update.available\n    state q = query.q ?? \"\"\n    Text(\"{ago(clock)} {tab} {dark} {online} {fresh} {q} {Clock.started}\")\n    Wide",
    );
    let found = with(&src, "X06");
    assert_eq!(
        found,
        vec![
            "2: `started` copies `now` once, when the store is built, and never follows it",
            "5: `wide` copies `viewport` once, when the component is drawn, and never follows it",
            "10: `clock` copies `now` once, when the page opens, and never follows it",
            "11: `tab` copies `hash` once, when the page opens, and never follows it",
            "12: `dark` copies `theme` once, when the page opens, and never follows it",
            "13: `online` copies `network` once, when the page opens, and never follows it",
            "14: `fresh` copies `update` once, when the page opens, and never follows it",
            "15: `q` copies `query` once, when the page opens, and never follows it",
        ]
    );
    assert!(findings(&src).iter().all(|d| !d.is_error()));
}

#[test]
fn x06_leaves_a_live_value_that_is_followed_or_a_state_that_changes() {
    // Worked out again whenever it changes: a derived value, and a lambda
    // that reads it when it is called.
    clean(&page(
        "    derived clock = now(every: 1.seconds)\n    state late = (d) => d.isBefore(now)\n    Text(\"{clock} {late(clock)}\")",
    ));
    // A starting value the page then changes: assigned, bound, changed in
    // place. A `persist` keeps what it was; an action's local is read once.
    let written = page(
        "    state tab = query.tab ?? \"a\"\n    state q = query.q ?? \"\"\n    state seen = [now]\n    persist first = now\n    action stamp() {\n        let at = now\n        seen.push(at)\n    }\n    Input(bind: q, label: \"Search\")\n    Button(\"B\") { on click { tab = \"b\"  stamp() } }\n    Text(\"{tab} {seen.length} {first}\")",
    );
    assert!(
        with(&written, "X06").is_empty(),
        "{:?}",
        with(&written, "X06")
    );
    // A write to a loop's item is a write to its list.
    let item = page(
        "    state tags = query.tags.split(\",\").map(t => { name: t, on: true })\n    for t in tags {\n        Button(t.name) { on click { t.on = !t.on } }\n    }",
    );
    assert!(with(&item, "X06").is_empty(), "{:?}", with(&item, "X06"));
    // A store's state written by its own action, or from a page.
    let store = String::from(
        "store Clock {\n    state last = now\n    state opened = now\n    action touch() { last = now }\n}\n",
    ) + &page(
        "    Text(\"{Clock.last} {Clock.opened}\")\n    Button(\"x\") { on click { Clock.touch()  Clock.opened = now } }",
    );
    assert!(with(&store, "X06").is_empty(), "{:?}", with(&store, "X06"));
}

#[test]
fn x06_offers_derived_and_the_page_it_gives_is_clean() {
    let src = page("    state clock = now(every: 1.seconds)\n    Text(\"Updated {ago(clock)}\")");
    let finding = findings(&src)
        .into_iter()
        .find(|d| d.code == "X06")
        .expect("X06");
    assert_eq!(finding.fixes.len(), 1, "{finding:?}");
    assert_eq!(finding.fixes[0].title, "Change to `derived clock`");
    let fixed = webfluent::diagnostics::fixes::apply(&src, &finding.fixes[0]);
    assert!(
        fixed.contains("    derived clock = now(every: 1.seconds)\n"),
        "{fixed}"
    );
    clean(&fixed);
    assert!(with(&fixed, "X06").is_empty());
}

// ─── C8: forms ───────────────────────────────────────────────────────

#[test]
fn f01_f02_what_bind_names_and_what_the_control_holds() {
    let src = format!(
        "const LIMIT = 3\n{}",
        page(
            "    state first = \"a\"\n    state qty = 1\n    state on = \"yes\"\n    derived up = first.toUpperCase()\n    Input(bind: up, label: \"a\")\n    Input(bind: LIMIT, label: \"b\")\n    Input(bind: \"x\", label: \"c\")\n    Input(bind: qty, label: \"d\")\n    Input(bind: first, label: \"e\").number\n    Checkbox(bind: on, label: \"f\")"
        )
    );
    assert_eq!(with(&src, "F01").len(), 3, "{:?}", with(&src, "F01"));
    assert_eq!(with(&src, "F02").len(), 3, "{:?}", with(&src, "F02"));
    clean(&page(
        "    state name = \"\"\n    state qty = 1\n    state items = [{ t: \"a\" }]\n    Input(bind: name, label: \"a\")\n    Input(bind: qty, label: \"b\").number\n    for it in items by it.t { Input(bind: it.t, label: \"c\") }",
    ));
}

#[test]
fn f03_f04_f05_validation_and_controls_that_cannot_work() {
    let src = page(
        "    state email = \"\"\n    state first = \"a\"\n    derived up = first.toUpperCase()\n    validate email { required }\n    validate up { required }\n    Input(bind: first, label: \"f\")",
    );
    assert_eq!(with(&src, "F03").len(), 2, "{:?}", with(&src, "F03"));
    let unbound = page(
        "    state agree = false\n    Checkbox(checked: agree, label: \"a\")\n    Checkbox(checked: agree, label: \"b\") { on change { agree = !agree } }",
    );
    assert_eq!(with(&unbound, "F04").len(), 1);
    let select = page(
        "    state plan = \"basic\"\n    state ok = \"pro\"\n    Select(bind: plan, label: \"a\") { Select.Option(\"Free\", value: \"free\")  Select.Option(\"Pro\", value: \"pro\") }\n    Select(bind: ok, label: \"b\") { Select.Option(\"Free\", value: \"free\")  Select.Option(\"Pro\", value: \"pro\") }",
    );
    assert_eq!(with(&select, "F05").len(), 1);
}

// ─── C9: secrets ─────────────────────────────────────────────────────

#[test]
fn t12_a_secret_does_not_escape_through_joins_the_browser_or_its_members() {
    let src = page(
        "    state token: Secret = \"\"\n    Button(\"x\") { on click {\n        log(\"/a?t=\" + token)\n        console.log(token)\n        localStorage.setItem(\"t\", token)\n        alert(token)\n        log(token.length)\n    } }",
    );
    let found = with(&src, "T12");
    assert!(found.len() >= 5, "{found:?}");
    assert!(
        found.iter().any(|f| f.contains("`+` would put it in text")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|f| f.contains("`console.log`")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|f| f.contains("has no `length`")),
        "{found:?}"
    );
    // Handed to a request's headers, it is where it belongs.
    clean(&page(
        "    state token: Secret = \"\"\n    resource me = fetch(\"/me\", headers: { \"Authorization\": token })\n    Text(\"{me.state}\")",
    ));
}

// ─── C10: data ───────────────────────────────────────────────────────

#[test]
fn d02_d03_what_persist_cannot_keep_and_what_every_instance_shares() {
    assert_eq!(with(&page("    persist f = (x) => x"), "D02").len(), 1);
    let shared = "component Panel(_ title: String) {\n    persist open = false\n    Text(title)\n}\npage P(path: \"/\", title: \"T\", description: \"D\") { Heading(\"p\").h1  Panel(\"a\")  Panel(\"b\") }\n";
    assert_eq!(with(shared, "D03").len(), 1);
    let keyed = shared.replace(
        "persist open = false",
        "persist open = false { key: title }",
    );
    assert!(with(&keyed, "D03").is_empty());
    // The key reaches the storage key.
    let program = webfluent::parse_source(&keyed, "t.wf").unwrap();
    let js = webfluent::codegen::js::JsCodegen::new().generate(&program);
    assert!(
        js.contains("WF.persist(\"Panel.open\", false, { key: () => _p.title })"),
        "{js}"
    );
}

#[test]
fn t10_an_endpoint_path_parameter_nothing_fills() {
    let src = format!(
        "api B(base: \"/api\") {{\n    get user(userId: String) at \"users/:id\"\n    get ok(id: String) at \"users/:id\"\n}}\n{}",
        page("")
    );
    let found = with(&src, "T10");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("no parameter fills `:id`"), "{found:?}");
}

// ─── C12: literals, styles, text ─────────────────────────────────────

#[test]
fn calendar_dates_money_and_key_combinations_are_held_to_what_they_can_be() {
    let src = page(
        "    state d: Date = @2026-02-30\n    state e: Date = \"2026-13-01\"\n    state ok: Date = @2028-02-29\n    Text(\"{d}{e}{ok}\")",
    );
    let found = with(&src, "T01");
    assert_eq!(found.len(), 2, "{found:?}");
    let money = webfluent::syntax::parse_source(
        &page("    state p = €12.999\n    Text(\"{p.amount}\")"),
        "t.wf",
    )
    .unwrap_err()
    .diagnostics();
    assert_eq!(money[0].code, "T01");
    assert!(
        money[0].message.contains("more decimals than EUR has"),
        "{money:?}"
    );
    let key =
        webfluent::syntax::parse_source(&page("    on key(\"ctrl+shfit+k\") { log(1) }"), "t.wf")
            .unwrap_err()
            .diagnostics();
    assert_eq!(key[0].code, "E117");
    clean(&page(
        "    on key(\"cmd+K\") { log(1) }\n    on key(\"Escape\") { log(2) }\n    on key(\"shift+F2\") { log(3) }",
    ));
}

#[test]
fn v05_v06_v07_styles_that_the_browser_drops() {
    let src = page(
        "    state pct = 40\n    Card { style { colr: red\n background: $brnad\n border: 1px solid $border\n width: {pct}\n height: {pct}%\n --mine: 3px } }",
    );
    assert_eq!(with(&src, "V05").len(), 1);
    assert_eq!(
        with(&src, "V06").len(),
        1,
        "a short token resolves through a border's colour"
    );
    assert_eq!(with(&src, "V07").len(), 1);
}

#[test]
fn t20_a16_a_list_in_text_and_one_id_on_many_elements() {
    let src = page(
        "    state tags = [\"a\"]\n    Text(\"{tags}\")\n    Text(\"{tags.join(\\\", \\\")}\")\n    for t in tags by t { Text(t, id: \"tag\") }",
    );
    assert_eq!(with(&src, "T20").len(), 1);
    assert_eq!(with(&src, "A16").len(), 1);
}

// ─── C13: dead code ──────────────────────────────────────────────────

#[test]
fn u06_u08_u09_code_that_never_runs_and_loops_that_lose_state() {
    let src = page(
        "    state n = 0\n    state rows = [{ id: 1, name: \"a\" }]\n    action go() {\n        return n\n        log(1)\n    }\n    if false { Text(\"x\") }\n    if n == n { Text(\"y\") }\n    for r in rows { Input(bind: r.name, label: \"N\") }\n    for r in rows by r.id { Input(bind: r.name, label: \"M\") }\n    Button(\"go\") { on click { go() } }",
    );
    assert_eq!(with(&src, "U06").len(), 1);
    assert_eq!(with(&src, "U08").len(), 2);
    assert_eq!(with(&src, "U09").len(), 1);
}

#[test]
fn an_image_is_the_asset_the_build_makes_of_it() {
    let src = "image hero = \"hero.jpg\"\npage P(path: \"/\", title: \"T\", description: \"D\") {\n    Heading(\"H\").h1\n    Text(\"{hero.width} by {hero.widht}\")\n}\n";
    let found = with(src, "T05");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("widht"), "{found:?}");
}

// ─── Found building an app ───────────────────────────────────────────

#[test]
fn an_if_let_binding_is_a_name_in_the_whole_of_its_branch() {
    let src = format!(
        "type E {{ reason: String? }}\n\
         component Pill(_ text: String) {{ Badge(text) }}\n\
         component Tile(e: E) {{\n    \
             Stack {{\n        \
                 if let why = e.reason {{\n            \
                     Pill(why)\n            \
                     Row {{ Stack {{ Pill(why) }} }}\n        \
                 }}\n    \
             }}\n\
         }}\n{}",
        page(
            "    resource r: E = fetch(\"/e\")\n    \
             Tile(e: E(reason: null))\n    \
             match r {\n        \
                 ready(e) {\n            \
                     if let why = e.reason { Row { Pill(why) } }\n            \
                     Pill(nope)\n        \
                 }\n        \
                 else { Spinner }\n    \
             }"
        )
    );
    let found = with(&src, "V01");
    // The one word nothing declares, and none of the bindings.
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("`nope`"), "{found:?}");
}

#[test]
fn a_case_s_payload_is_a_name_in_its_arm() {
    let src = format!(
        "enum Status {{ idle, failed(reason: String) }}\n\
         component Pill(_ text: String) {{ Badge(text) }}\n{}",
        page(
            "    state s: Status = .idle\n    \
             match s {\n        \
                 .failed(why) { Row { Pill(why) } }\n        \
                 else { Text(\"Fine\") }\n    \
             }"
        )
    );
    let found = with(&src, "V01");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a05_a_button_whose_block_shows_text_has_a_label() {
    let src = format!(
        "component Glyph(_ name: String) {{ Host(tag: \"span\", class: \"ic\") }}\n\
         component Caption(_ text: String) {{ Text(text) }}\n\
         component Brand {{ Text(\"GitHub\") }}\n{}",
        page(
            "    state who = \"Ada\"\n    \
             Button(class: \"go\") {\n        \
                 on click { log(1) }\n        \
                 Glyph(\"link\")\n        \
                 Text(\"Continue with GitHub\")\n    \
             }\n    \
             Button { Row { Icon(\"user\")  Text(who) } }\n    \
             Button { Caption(\"Save\") }\n    \
             Button { Glyph(\"link\")  Brand }\n    \
             Button(class: \"icon-only\") { Glyph(\"link\") }\n    \
             Button { Text(\"\") }\n    \
             Button { if who == \"\" { Text(\"Sign in\") } else { Text(who) } }\n    \
             Button { Tooltip(\"Delete\") { Icon(\"trash\") } }\n    \
             Button { Element(\"sl-icon\", name: \"gear\") }"
        )
    );
    let found = with(&src, "A05");
    // An icon alone, an empty text, a tooltip's hidden words and a custom
    // element's tag still name nothing.
    let lines: Vec<&str> = found.iter().map(|f| f.split(':').next().unwrap()).collect();
    assert_eq!(lines, ["15", "16", "18", "19"], "{found:?}");
}

// ─── E118: a name the language already has ───────────────────────────

#[test]
fn e118_nothing_at_the_top_level_takes_a_name_the_language_has() {
    // The bundle declares `env` (the build's settings) and `WF` (the
    // runtime) before the program's own values, so a top-level value of
    // either name was a second declaration — refused only by the read-back,
    // as "a bug in WebFluent" (E901). Any other the language has would be
    // hidden on every page.
    for decl in [
        "const env = \"x\"",
        "data env = \"posts.json\"",
        "image env = \"hero.png\"",
        "store env { state a = 1 }",
        "api env(base: \"/api\") { get users() -> Map }",
    ] {
        let found = with(&format!("{decl}\n{}", page("    Text(\"x\")")), "E118");
        assert_eq!(found.len(), 1, "{decl}: {found:?}");
        assert!(
            found[0]
                .starts_with("1: `env` is a name the language already has: the build's settings"),
            "{decl}: {found:?}"
        );
    }
    let found = with(&format!("const WF = 1\n{}", page("")), "E118");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("the runtime the compiled pages call"),
        "{found:?}"
    );
    let found = with(&format!("const format = 1\n{}", page("")), "E118");
    assert!(found[0].contains("`format(…)`"), "{found:?}");
}

#[test]
fn e118_a_store_member_takes_no_name_every_object_has() {
    // A store is an object whose members are its own properties: a member
    // `valueOf` replaced the one JavaScript calls whenever it turns the
    // store into a value, and the page threw as soon as it was drawn.
    for member in [
        "state valueOf = 1",
        "derived toString = 2",
        "action constructor() { }",
        "state hasOwnProperty = 3",
        "state __proto__ = 4",
    ] {
        let found = with(
            &format!("store S {{\n    {member}\n}}\n{}", page("    Text(\"x\")")),
            "E118",
        );
        assert_eq!(found.len(), 1, "{member}: {found:?}");
        assert!(
            found[0].contains("every JavaScript object's, and `S` is one"),
            "{found:?}"
        );
    }
    // A prop of those names is the caller's own and works, except
    // `__proto__`, which in the object a call passes sets its prototype.
    let found = with(
        &format!(
            "component Tile(toString: String = \"b\", valueOf: String) {{ Text(toString) }}\n{}",
            page("    Tile(valueOf: \"a\")")
        ),
        "E118",
    );
    assert!(found.is_empty(), "{found:?}");
    let found = with(
        &format!(
            "component Tile(__proto__: String) {{ Text(\"x\") }}\n{}",
            page("")
        ),
        "E118",
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("`__proto__` is a name the language already has"),
        "{found:?}"
    );
}

#[test]
fn e118_no_name_a_body_declares_takes_one_the_language_has() {
    // A `state env` compiled every read to the build's settings and every
    // write to the state, so the page showed `[object Object]` and `env ==
    // "prod"` was never true; a loop's `t` hid the translation function
    // inside the loop. Every way a body declares a name is refused, where
    // it is declared, once.
    let src = format!(
        "component Tag(env: String) {{ Text(env) }}\n{}",
        page(concat!(
            "    state env = \"a\"\n",
            "    derived query = env + \"?\"\n",
            "    persist theme = \"dark\"\n",
            "    state items = [1, 2]\n",
            "    derived open = items.filter(t => t > 1)\n",
            "    action pick(locale: String) { log(locale) }\n",
            "    for now in items { Text(\"{now}\") }\n",
            "    if let style = items.first() { Text(\"{style}\") }\n",
            "    Button(\"b\") { on click {\n        let params = 1\n        log(params)\n    } }\n",
            "    Text(\"{env} {query} {theme} {open.length}\")\n",
            "    Tag(env: \"b\")",
        ))
    );
    let found = with(&src, "E118");
    let names: Vec<&str> = found.iter().map(|f| f.split('`').nth(1).unwrap()).collect();
    assert_eq!(
        names,
        [
            "env", "env", "query", "theme", "t", "locale", "now", "style", "params"
        ],
        "{found:?}"
    );
    // What a program may name its own: the names a handler brings, the
    // browser's globals, words that only open a top-level declaration, and
    // any name as a field, a map key, an enum case or an endpoint's
    // parameter. A name that only begins like one is a name like any other.
    clean(&format!(
        "enum Stage {{ env, query, t }}\ntype Box {{ env: String, t: Number }}\n\
         api Backend(base: \"/api\") {{ get users(page: Number = 1, query: String?) -> [Map] }}\n\
         const envy = 1\n{}",
        page(concat!(
            "    state value = 1\n",
            "    state key = \"k\"\n",
            "    state history: [Number] = []\n",
            "    state page = 1\n",
            "    state data = { env: \"a\", t: 2 }\n",
            "    state box = Box(env: \"e\", t: 1)\n",
            "    state stage: Stage = .env\n",
            "    Button(\"go\") { on click(e) { value = value + e.detail } }\n",
            "    Text(\"{value} {key} {history.length} {page} {data.env} {box.t} {stage} {envy} {env.PUBLIC_API}\")",
        ))
    ));
}

/// A temporary project with `files` (paths under it), removed when the
/// guard goes.
struct Project(std::path::PathBuf);

impl Project {
    fn new(name: &str, files: &[(&str, &str)]) -> Self {
        let dir = std::env::temp_dir().join(format!("wf-checks-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Project(dir)
    }

    /// `wf <command> <args…> -d <dir>`: its success, and what it printed.
    fn wf(&self, command: &str, args: &[&str]) -> (bool, String, String) {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
            .arg(command)
            .args(args)
            .arg(&self.0)
            .output()
            .expect("run wf");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_build_refuses_a_name_the_language_has_where_it_is_written() {
    // A top-level `env` was refused only as "a bug in WebFluent" (E901);
    // a page's `state env` built, and showed `[object Object]`.
    let page = "page P(path: \"/\", title: \"T\", description: \"D\") {\n";
    for (name, src, at) in [
        (
            "const",
            format!("const env = \"staging\"\n{page}    Heading(\"Deploy to {{env}}\").h1\n}}\n"),
            "src/App.wf:1:1",
        ),
        (
            "data",
            format!("data env = \"posts.json\"\n{page}    Heading(\"Deploy to {{env}}\").h1\n}}\n"),
            "src/App.wf:1:1",
        ),
        (
            "state",
            format!(
                "{page}    state env = \"staging\"\n    Heading(\"Deploy to {{env}}\").h1\n}}\n"
            ),
            "src/App.wf:2:5",
        ),
    ] {
        let project = Project::new(
            &format!("e118-{name}"),
            &[
                ("webfluent.app.json", r#"{ "name": "t" }"#),
                ("posts.json", "[1, 2]"),
                ("src/App.wf", &src),
            ],
        );
        let (ok, stdout, stderr) = project.wf("build", &["-d"]);
        let out = stdout + &stderr;
        assert!(!ok, "{name}: {out}");
        assert!(out.contains("error[E118]"), "{name}: {out}");
        assert!(out.contains(at), "{name}: {out}");
        assert!(!out.contains("E901"), "{name}: {out}");
    }
}

#[test]
fn e118_is_reported_in_every_file_that_declares_the_name() {
    // The same declaration at the same line and column of two files is two
    // findings. One was taken for the other, so the second file's `state
    // query` went unreported until the first was renamed.
    let store = |name: &str| format!("store {name} {{\n    state query = \"\"\n}}\n");
    let project = Project::new(
        "e118-two-files",
        &[
            ("webfluent.app.json", r#"{ "name": "t" }"#),
            ("src/stores/a.wf", &store("A")),
            ("src/stores/b.wf", &store("B")),
            (
                "src/App.wf",
                "page P(path: \"/\", title: \"T\", description: \"D\") {\n    use A\n    use B\n    Heading(\"{A.query}{B.query}\").h1\n}\n",
            ),
        ],
    );
    let (ok, stdout, stderr) = project.wf("check", &["-d"]);
    let out = stdout + &stderr;
    assert!(!ok, "{out}");
    assert_eq!(out.matches("error[E118]").count(), 2, "{out}");
    assert!(out.contains("src/stores/a.wf:2:5"), "{out}");
    assert!(out.contains("src/stores/b.wf:2:5"), "{out}");
}
