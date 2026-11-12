#[test]
fn every_type_name_a_tool_offers_is_one_the_checker_reads() {
    for s in Scalar::ALL {
        assert_eq!(Scalar::of_name(s.name()), Some(s));
    }
    for name in PRIMITIVE_TYPES {
        let src = format!("page P(path: \"/\") {{ state x: {name}? = null\n Text(\"a\") }}");
        let program = crate::syntax::parse_source(&src, "t.wf").unwrap();
        assert!(
            check(&program, &|_| String::new())
                .findings
                .errors
                .is_empty(),
            "{name}"
        );
    }
}

#[test]
fn every_rule_a_tool_offers_is_one_the_checker_reads() {
    for rule in VALIDATE_RULES {
        assert!(
            rule_wants(rule).is_some(),
            "`{rule}` is offered and not a rule"
        );
    }
}

#[test]
fn members_are_the_checker_s_own_tables() {
    let names = |ty: Type| -> Vec<&'static str> {
        members(&ty)
            .into_iter()
            .map(|m| &*Box::leak(m.name.into_boxed_str()))
            .collect()
    };
    let list = names(Type::list(Type::Number));
    for m in [
        "length", "map", "sortBy", "groupBy", "unique", "take", "first", "sum",
    ] {
        assert!(list.contains(&m), "a list's `{m}`: {list:?}");
    }
    let text = names(Type::String);
    for m in [
        "capitalize",
        "truncate",
        "dedent",
        "lines",
        "words",
        "toLowerCase",
    ] {
        assert!(text.contains(&m), "a string's `{m}`: {text:?}");
    }
    let date = names(Type::Scalar(Scalar::Date));
    for m in ["plus", "year", "isBefore", "startOfMonth", "until"] {
        assert!(date.contains(&m), "a date's `{m}`: {date:?}");
    }
    assert!(!date.contains(&"amount") && !date.contains(&"host"));
    let money = names(Type::Scalar(Scalar::Money));
    for m in ["amount", "currency", "plus", "times", "convert"] {
        assert!(money.contains(&m), "money's `{m}`: {money:?}");
    }
    let url = names(Type::Scalar(Scalar::Url));
    assert!(url.contains(&"host") && url.contains(&"with") && url.contains(&"toLowerCase"));
    assert_eq!(url.iter().filter(|m| **m == "length").count(), 1);
    assert!(
        names(Type::Scalar(Scalar::Secret)).is_empty(),
        "a secret is not read apart"
    );
    let viewport = names(browser_value_type("viewport").unwrap());
    assert!(
        viewport.contains(&"md") && viewport.contains(&"width"),
        "{viewport:?}"
    );
    // Every value `members` offers for a scalar is one the checker types.
    for scalar in [
        Scalar::Date,
        Scalar::Time,
        Scalar::DateTime,
        Scalar::Duration,
        Scalar::Money,
        Scalar::Url,
        Scalar::Email,
        Scalar::Color,
        Scalar::File,
        Scalar::Uuid,
    ] {
        for m in members(&Type::Scalar(scalar)) {
            if m.method {
                assert!(
                    scalar_method(scalar, &m.name).is_some(),
                    "{scalar:?}.{}",
                    m.name
                );
            }
        }
    }
    // …and every scalar method the code generator routes is offered.
    for (m, _) in crate::registry::browser::SCALAR_METHODS {
        assert!(
            SCALAR_MEMBER_NAMES.contains(m),
            "`{m}` is routed but not offered"
        );
    }
}

use super::members::SCALAR_MEMBER_NAMES;
use super::*;
use crate::syntax::parser::v2::parse_v2;

fn errors(src: &str) -> Vec<String> {
    let program = parse_v2(src, "<t>").expect("parse");
    let info = check(&program, &|_| "<t>".to_string());
    info.findings
        .errors
        .iter()
        .map(|d| match &d.hint {
            Some(h) => format!("[{}] {}\n  {h}", d.code, d.message),
            None => format!("[{}] {}", d.code, d.message),
        })
        .collect()
}

/// `sort` hands its comparator two elements. Typed like `map`'s
/// `(element, index)`, the second parameter read as a `Number`.
#[test]
fn a_sort_comparator_takes_two_elements() {
    let src = r#"
            type Row { age: Number }
            store S {
                state rows: [Row] = []
                derived oldest = rows.slice().sort((a, b) => b.age - a.age)
            }
            page P(path: "/") { use S  Text("{S.oldest.length}") }
        "#;
    assert_eq!(errors(src), Vec::<String>::new());
}

/// A store's members are mutually visible. Inferring each derived value
/// as it was met meant an action declared below it read as undeclared,
/// though the runtime binds actions first and an action may call one
/// declared after it.
#[test]
fn a_derived_value_may_call_an_action_declared_below_it() {
    let src = r#"
            store S {
                state n = 2
                derived doubled = twice(n)
                action twice(v: Number) { return v * 2 }
            }
            page P(path: "/") { use S  Text("{S.doubled}") }
        "#;
    assert_eq!(errors(src), Vec::<String>::new());
}

fn clean(src: &str) {
    let found = errors(src);
    assert!(found.is_empty(), "{found:#?}");
}

fn has(src: &str, needle: &str) {
    let found = errors(src);
    assert!(
        found.iter().any(|e| e.contains(needle)),
        "no error containing {needle:?} in {found:#?}"
    );
}

fn type_of(src: &str, name: &str) -> String {
    let program = parse_v2(src, "<t>").expect("parse");
    let info = check(&program, &|_| "<t>".to_string());
    info.bindings
        .iter()
        .rev()
        .find(|b| b.name == name)
        .map(|b| b.ty.to_string())
        .unwrap_or_else(|| panic!("{name} has no type; bindings: {:?}", info.bindings))
}

const TODOS: &str = "enum Tone { calm, loud }\ntype Todo { id: String, title: String, done: Bool = false, tone: Tone = .calm, note: String? = null }\n";

#[test]
fn declared_and_inferred_types_of_names() {
    let src = format!(
        "{TODOS}store Todos {{\n  state items: [Todo] = []\n  state draft = \"\"\n  state n = 0\n  state sel = null\n  derived remaining = items.filter(item => !item.done).length\n  derived titles = items.map(item => item.title)\n  derived first = items.find(item => item.done)\n  action add(title: String) {{ items = items.concat([Todo(id: \"1\", title: title)]) }}\n  action count() {{ return items.length }}\n}}\npage P(path: \"/\", id: String) {{\n  use Todos\n  resource rows: [Todo] = fetch(\"/x\")\n  state item = Todo(id: \"a\", title: \"b\")\n  for todo in Todos.items by todo.id {{ Text(todo.title) }}\n  match rows {{ ready(list) {{ for r in list {{ Text(r.title) }} }} else {{ }} }}\n}}"
    );
    assert_eq!(type_of(&src, "items"), "[Todo]");
    assert_eq!(type_of(&src, "draft"), "String");
    assert_eq!(type_of(&src, "n"), "Number");
    assert_eq!(type_of(&src, "sel"), "Any?");
    assert_eq!(type_of(&src, "remaining"), "Number");
    assert_eq!(type_of(&src, "titles"), "[String]");
    assert_eq!(type_of(&src, "first"), "Todo?");
    assert_eq!(type_of(&src, "add"), "action(String)");
    assert_eq!(type_of(&src, "count"), "action() -> Number");
    assert_eq!(type_of(&src, "id"), "String");
    assert_eq!(type_of(&src, "rows"), "resource<[Todo]>");
    assert_eq!(type_of(&src, "item"), "Todo");
    assert_eq!(type_of(&src, "todo"), "Todo");
    assert_eq!(type_of(&src, "list"), "[Todo]");
    assert_eq!(type_of(&src, "r"), "Todo");
    clean(&src);
}

#[test]
fn a_value_of_the_wrong_type_is_reported_where_it_is_given() {
    has(
        &format!("{TODOS}page P(path: \"/\") {{ state n: Number = \"x\" }}"),
        "[T01] `n` is `String`, but `Number` is wanted",
    );
    has(
        &format!(
            "{TODOS}component C(count: Number) {{ Text(count) }}\npage P(path: \"/\") {{ C(count: \"3\") }}"
        ),
        "`count` of `C` is `String`, but `Number` is wanted",
    );
    has(
        &format!(
            "{TODOS}component C(_ label: String) {{ Text(label) }}\npage P(path: \"/\") {{ C(3) }}"
        ),
        "`label` of `C` is `Number`, but `String` is wanted",
    );
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state x = Todo(id: \"a\", title: \"b\")\n Text(x.title) }}\ncomponent D(todo: Todo) {{ Text(todo.title) }}\npage Q(path: \"/q\") {{ state s = \"\"\n D(todo: s) }}"
        ),
        "`todo` of `D` is `String`, but `Todo` is wanted",
    );
    has(
        &format!("{TODOS}type Row {{ n: Number = \"no\" }}"),
        "the default of `n` is `String`, but `Number` is wanted",
    );
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state n = 1\n Button(\"x\") {{ on click {{ n = \"two\" }} }} }}"
        ),
        "`n` is `String`, but `Number` is wanted",
    );
}

#[test]
fn enum_cases_are_checked_against_their_enum() {
    // A case an element argument gets wrong is the resolver's to report;
    // everywhere else it is the checker's.
    clean(&format!(
        "{TODOS}component C(tone: Tone) {{ Text(\"x\") }}\npage P(path: \"/\") {{ C(tone: .angry) }}"
    ));
    has(
        &format!("{TODOS}page P(path: \"/\") {{ state item: Tone = .angry }}"),
        "[T02] `Tone` has no case `.angry`",
    );
    clean(&format!(
        "{TODOS}component C(tone: Tone = .calm) {{ Text(\"x\") }}\npage P(path: \"/\") {{ C(tone: .loud)\n state item: Tone = .calm\n match item {{ .calm {{ Text(\"a\") }} else {{ Text(\"b\") }} }} }}"
    ));
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state item: Tone = .calm\n match item {{ .angry {{ Text(\"a\") }} else {{ }} }} }}"
        ),
        "[T02] `Tone` has no case `.angry`",
    );
}

#[test]
fn fields_members_and_methods_that_do_not_exist() {
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state item = Todo(id: \"a\", title: \"b\")\n Text(item.name) }}"
        ),
        "[T05] `Todo` has no field `name`",
    );
    has(
        "store S { state n = 0 }\npage P(path: \"/\") { use S\n Text(S.count) }",
        "[T06] `S` has no member `count`",
    );
    has(
        "store S { state n = 0 action bump() { n = n + 1 } }\npage P(path: \"/\") { use S\n Button(\"x\") { on click { S.reset() } } }",
        "[T06] `S` has no action `reset`",
    );
    clean(
        "store S { state n = 0 action bump() { n = n + 1 } }\npage P(path: \"/\") { use S\n Text(\"{S.n}\")\n Button(\"x\") { on click { S.bump() } } }",
    );
}

#[test]
fn a_store_is_checked_without_use_and_from_another_store() {
    // `use` says a body depends on a store; the name is reachable either
    // way, so the members are held to what the store has either way.
    has(
        "store S { state n = 0 }\npage P(path: \"/\") { Text(S.count) }",
        "[T06] `S` has no member `count`",
    );
    has(
        "store A { state n = 0 }\nstore B { action go() { A.bmup() } }\npage P(path: \"/\") { Button(\"x\") { on click { B.go() } } }",
        "[T06] `A` has no action `bmup`",
    );
    clean(
        "store A { state n = 0 action bump() { n = n + 1 } }\nstore B { action go() { A.bump() } }\npage P(path: \"/\") { Text(\"{A.n}\")\n Button(\"x\") { on click { B.go() } } }",
    );
}

#[test]
fn a_services_headers_and_hooks_are_checked() {
    has(
        "store M { action record(s: Number) { log(s) } }\napi B(base: \"/api\") { on response(r) { M.recrd(r.status) }\n get me() -> Map }\npage P(path: \"/\") { resource m = B.me()\n match m { ready(v) { Text(\"{v}\") } else { Text(\"…\") } } }",
        "[T06] `M` has no action `recrd`",
    );
    has(
        "store S { state token = \"\" }\napi B(base: \"/api\") { headers { Authorization: \"Bearer {S.tokn}\" }\n get me() -> Map }\npage P(path: \"/\") { resource m = B.me()\n match m { ready(v) { Text(\"{v}\") } else { Text(\"…\") } } }",
        "[T06] `S` has no member `tokn`",
    );
    clean(
        "store S { state token = \"\"\n action refresh() { token = \"x\" } }\napi B(base: \"/api\") { headers { Authorization: \"Bearer {S.token}\" }\n on request(r) { r.headers[\"X-Id\"] = \"1\" }\n on error(e) { await S.refresh()  return \"retry\" }\n get me() -> Map }\npage P(path: \"/\") { resource m = B.me()\n match m { ready(v) { Text(\"{v}\") } else { Text(\"…\") } } }",
    );
}

#[test]
fn a_value_that_may_be_null_must_be_unwrapped() {
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state item = Todo(id: \"a\", title: \"b\")\n Text(item.note.length) }}"
        ),
        "[T04] `item.note` may be null",
    );
    clean(&format!(
        "{TODOS}page P(path: \"/\") {{ state item = Todo(id: \"a\", title: \"b\")\n state sel: Todo? = null\n if let n = item.note {{ Text(n.length) }}\n Text(item.note ?? \"none\")\n if sel != null {{ Text(sel.title) }}\n if sel {{ Text(sel.title) }}\n if sel && item.done {{ Text(sel.title) }} }}"
    ));
    has(
        &format!("{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n Text(sel.title) }}"),
        "[T04] `sel` may be null, so `.title` may fail",
    );
    // The condition of an `if` expression narrows its then-arm too.
    clean(&format!(
        "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived label = if sel != null {{ sel.title }} else {{ \"none\" }}\n Text(label) }}"
    ));
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived label = if true {{ \"x\" }} else {{ sel.title }}\n Text(label) }}"
        ),
        "[T04] `sel` may be null, so `.title` may fail",
    );
}

#[test]
fn calls_and_emits_take_the_declared_arguments() {
    has(
        "page P(path: \"/\") { action add(title: String, n: Number) { }\n Button(\"x\") { on click { add(\"a\") } } }",
        "[T10] `add` takes 2 arguments, but 1 is given",
    );
    has(
        "page P(path: \"/\") { action add(n: Number) { }\n Button(\"x\") { on click { add(\"a\") } } }",
        "argument 1 of `add` is `String`, but `Number` is wanted",
    );
    has(
        "component C { event pick(id: String)\n Button(\"x\") { on click { emit pick(1, 2) } } }",
        "[T09] `emit pick` passes 2 arguments, but the event takes 1",
    );
    has(
        "component C { event pick(id: String)\n Button(\"x\") { on click { emit pick(1) } } }",
        "`id` of `emit pick` is `Number`, but `String` is wanted",
    );
    clean(
        "component C { event pick(id: String)\n Button(\"x\") { on click { emit pick(\"a\") } } }\npage P(path: \"/\") { C { on pick(id) { log(id.length) } } }",
    );
}

#[test]
fn a_handler_names_each_value_its_event_sends() {
    const MENU: &str = "component Entry { event menu(key: String, x: Number, y: Number)\n Button(\"x\") { on click(e) { emit menu(\"a\", e.clientX, e.clientY) } } }\n";
    // Each name is typed as the event declares it.
    clean(&format!(
        "{MENU}page P(path: \"/\") {{ Entry {{ on menu(key, x, y) {{ log(key.length + x + y) }} }} }}"
    ));
    has(
        &format!(
            "{MENU}page P(path: \"/\") {{ state n = 0\n Entry {{ on menu(key, x, y) {{ n = key }} }} }}"
        ),
        "`String`",
    );
    clean(&format!(
        "{MENU}page P(path: \"/\") {{ state n = 0\n Entry {{ on menu(key, x, y) {{ n = x + y }} }} }}"
    ));
    // Fewer names than values is fine: the rest are not read.
    clean(&format!(
        "{MENU}page P(path: \"/\") {{ Entry {{ on menu(key) {{ log(key.length) }} }} }}"
    ));
    has(
        &format!("{MENU}page P(path: \"/\") {{ Entry {{ on menu(key, x, y, z) {{ log(z) }} }} }}"),
        "[T22] `on menu` names 4 values, but `event menu(key: String, x: Number, y: Number)` sends 3 values\n  Name at most 3, in order: `on menu(key, x, y)`",
    );
    has(
        "component C { event close\n Button(\"x\") { on click { emit close() } } }\npage P(path: \"/\") { C { on close(e) { log(e) } } }",
        "[T22] `on close` names 1 value, but `event close()` sends no value\n  Write `on close { … }`",
    );
    // A DOM event sends the event and nothing else, on a built-in, on a
    // component's root, and on the page.
    has(
        "page P(path: \"/\") { Button(\"x\") { on click(e, x) { log(x) } } }",
        "[T22] `on click` names 2 values, but a DOM event sends one value, the event\n  Name the one value, `on click(e)`",
    );
    has(
        &format!("{MENU}page P(path: \"/\") {{ Entry {{ on mouseenter(e, x) {{ log(x) }} }} }}"),
        "[T22] `on mouseenter` names 2 values, but a DOM event sends one value",
    );
    has(
        "page P(path: \"/\") { on keydown(e, k) { log(k) }\n Text(\"x\") }",
        "[T22] `on keydown` names 2 values",
    );
    clean("page P(path: \"/\") { Button(\"x\") { on click(e) { log(e) } } }");
    clean("page P(path: \"/\") { on key(\"Escape\", e) { log(e) }\n Text(\"x\") }");
}

#[test]
fn controls_bind_state_of_their_own_kind() {
    has(
        "page P(path: \"/\") { state name = \"\"\n Checkbox(bind: name, label: \"x\") }",
        "[F02] `bind:` on `Checkbox` holds a `Bool`, but `name` is `String`",
    );
    has(
        "page P(path: \"/\") { state flag = true\n Slider(bind: flag) }",
        "[F02] `bind:` on `Slider` holds a `Number`, but `flag` is `Bool`",
    );
    clean(
        "page P(path: \"/\") { state flag = true\n state n = 0\n state s = \"\"\n Checkbox(bind: flag, label: \"x\")\n Slider(bind: n)\n Input(bind: s, label: \"y\")\n Switch(bind: flag, label: \"z\") }",
    );
}

#[test]
fn loops_conditions_and_matches_need_the_right_shapes() {
    has(
        "page P(path: \"/\") { state name = \"x\"\n for c in name { Text(c) } }",
        "[T08] `for` loops over a list, but `name` is `String`",
    );
    has(
        "page P(path: \"/\") { state items = [1]\n if items { Text(\"some\") } }",
        "[T07] `if` reads `items` as a condition, but a list is always true",
    );
    has(
        "page P(path: \"/\") { state n = 1\n match n { else { Text(\"x\") } } }",
        "[T11] `match` needs a resource or an enum, but `n` is `Number`",
    );
    has(
        "page P(path: \"/\") { resource r = fetch(\"/x\")\n match r { .calm { } else { } } }",
        "`r` is a resource; `.calm` is not one of its states",
    );
    clean(
        "page P(path: \"/\") { state items = [1]\n state n = 0\n state s = \"\"\n if items.length > 0 { Text(\"some\") }\n if n { Text(\"n\") }\n if s { Text(\"s\") }\n show n > 1 { Text(\"x\") } }",
    );
}

#[test]
fn the_layout_and_the_universal_props_are_typed_too() {
    has(
        "component Shell(crumb: String) { slot  children }\npage P(path: \"/\", layout: Shell(crumb: 3)) { Text(\"x\") }",
        "`crumb` of `Shell` is `Number`, but `String` is wanted",
    );
    clean(
        "component Shell(crumb: String) { slot  children }\npage P(path: \"/\", layout: Shell(crumb: \"Home\")) { Text(\"x\")\n Card(exit: .fadeOut, delay: \"100ms\").fadeIn { Text(\"y\") } }",
    );
}

#[test]
fn untyped_programs_check_as_they_always_did() {
    clean(
        "store S { state items = [] state q = \"\" action add(item: Map) { items.push(item) } derived hits = items.filter(i => i.name.includes(q)) }\npage P(path: \"/\") { use S\n state user = null\n derived h = { a: 1 }\n Button(\"x\") { on click { user = { name: \"x\" }  S.add({ name: q })  h.a = 2  log(JSON.stringify(user))  localStorage.setItem(\"k\", \"v\") } }\n if user { Text(user.name) }\n for it in S.hits { Text(it.name) }\n Text(\"{S.q.length}\") }",
    );
}

#[test]
fn a_loop_in_an_action_binds_the_item_and_the_index() {
    clean(&format!(
        "{TODOS}page P(path: \"/\") {{ state items: [Todo] = []\n state n = 0\n action all() {{ for item in items {{ n = n + item.title.length }}\n for item, i in items {{ n = n + i }} }} }}"
    ));
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state items: [Todo] = []\n action all() {{ for item in items {{ item.nam = 1 }} }} }}"
        ),
        "[T05] `Todo` has no field `nam`",
    );
    has(
        "page P(path: \"/\") { state s = \"x\"\n action all() { for c in s { log(c) } } }",
        "[T08] `for` loops over a list, but `s` is `String`",
    );
}

#[test]
fn optional_chaining_reads_through_null_and_its_result_may_be_null() {
    clean(&format!(
        "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived title = sel?.title\n derived n = sel?.title?.length ?? 0\n Text(title ?? \"none\") }}"
    ));
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived title = sel?.nam\n Text(title ?? \"\") }}"
        ),
        "[T05] `Todo` has no field `nam`",
    );
    // The rest of the chain short-circuits with the `?.`: no fault, a
    // value that may be null.
    clean(&format!(
        "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived n = sel?.title.length\n Text(\"{{n}}\") }}"
    ));
    assert_eq!(
        type_of(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived n = sel?.title.length\n Text(\"x\") }}"
            ),
            "n"
        ),
        "Number?"
    );
    // Off the chain, the result is a value that may be null.
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived t = sel?.title\n derived n = t.length\n Text(\"{{n}}\") }}"
        ),
        "[T04] `t` may be null",
    );
    assert_eq!(
        type_of(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived t = sel?.title\n Text(\"x\") }}"
            ),
            "t"
        ),
        "String?"
    );
}

#[test]
fn a_map_literal_has_the_shape_it_was_written_with() {
    clean(
        "page P(path: \"/\") { state user = { name: \"\", age: 0 }\n derived n = user.name.length + user.age\n Text(\"{n}\") }",
    );
    has(
        "page P(path: \"/\") { state user = { name: \"\", age: 0 }\n Text(user.nam) }",
        "[T05] `user` has no field `nam`\n  Did you mean `name`? It was written with `name`, `age`",
    );
    has(
        "page P(path: \"/\") { derived rows = [{ id: 1, label: \"a\" }]\n for r in rows { Text(r.lable) } }",
        "[T05] `r` has no field `lable`",
    );
    assert_eq!(
        type_of(
            "page P(path: \"/\") { state user = { name: \"\", age: 0 }\n Text(\"x\") }",
            "user"
        ),
        "{ name: String, age: Number }"
    );
    // A shape fits a record, a map or another shape that agrees on the
    // shared fields; an empty literal or a spread says nothing about the keys.
    clean(&format!(
        "{TODOS}page P(path: \"/\") {{ state items: [Todo] = []\n state form = {{}}\n state opts = {{ ...form, x: 1 }}\n action add() {{ items.push({{ id: \"1\", title: \"t\" }})  form.anything = 1  opts.other = 2 }} }}"
    ));
    has(
        &format!(
            "{TODOS}page P(path: \"/\") {{ state items: [Todo] = []\n action add() {{ items.push({{ id: 1, title: \"t\" }}) }} }}"
        ),
        "`id` is `Number`, but `String` is wanted",
    );
    // Two shapes join on the fields they share.
    assert_eq!(
        type_of(
            "page P(path: \"/\") { state flag = true\n derived v = if flag { { a: 1, b: \"\" } } else { { a: 2, c: true } }\n Text(\"x\") }",
            "v"
        ),
        "{ a: Number, b: String?, c: Bool? }"
    );
    clean(
        "page P(path: \"/\") { state rows = [{ id: 1 }, { id: 2, other: true }]\n derived n = rows.filter(r => r.other).length\n Text(\"{n}\") }",
    );
}

const STATUS: &str =
    "enum Status { idle, failed(reason: String), done(count: Number, label: String) }\n";

#[test]
fn a_case_carries_the_payload_it_was_declared_with() {
    clean(&format!(
        "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n action go() {{ s = .failed(\"boom\")  s = .done(1, \"one\") }}\n match s {{ .idle {{ Text(\"idle\") }} .failed(r) {{ Text(r.toUpperCase()) }} .done(n, l) {{ Text(\"{{n}} {{l}}\") }} }}\n Text(match s {{ .failed(r) {{ r }} .done {{ \"done\" }} else {{ \"-\" }} }}) }}"
    ));
    assert_eq!(
        type_of(
            &format!(
                "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n derived r = match s {{ .failed(r) {{ r }} else {{ \"\" }} }}\n Text(r) }}"
            ),
            "r"
        ),
        "String"
    );
    has(
        &format!("{STATUS}page P(path: \"/\") {{ state s: Status = .failed  Text(\"x\") }}"),
        "[T02] `.failed` carries a payload; `.failed` alone is not a `Status`\n  Give it: `.failed(reason: String)`",
    );
    has(
        &format!("{STATUS}page P(path: \"/\") {{ state s: Status = .idle(1)  Text(\"x\") }}"),
        "[T02] `.idle` carries nothing; `.idle(…)` is not a `Status`",
    );
    has(
        &format!("{STATUS}page P(path: \"/\") {{ state s: Status = .failed(1)  Text(\"x\") }}"),
        "[T01] argument 1 of `.failed` is `Number`, but `String` is wanted",
    );
    has(
        &format!("{STATUS}page P(path: \"/\") {{ state s: Status = .done(1)  Text(\"x\") }}"),
        "[T10] `.done` takes 2 arguments, but 1 is given",
    );
    has(
        &format!(
            "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n match s {{ .done(n) {{ Text(\"{{n}}\") }} else {{ }} }} }}"
        ),
        "[T10] `.done` carries 2 values, but 1 is bound\n  Bind every part: `.done(count: Number, label: String)`",
    );
    has(
        &format!(
            "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n match s {{ .idle(x) {{ Text(x) }} else {{ }} }} }}"
        ),
        "[T10] `.idle` carries 0 values, but 1 is bound",
    );
    has(
        &format!(
            "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n match s {{ .done(n, l) {{ for x in n {{ Text(\"{{x}}\") }} }} else {{ }} }} }}"
        ),
        "[T08] `for` loops over a list, but `n` is `Number`",
    );
    has(
        &format!(
            "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n derived r = match s {{ .gone {{ 1 }} else {{ 0 }} }}\n Text(\"{{r}}\") }}"
        ),
        "[T02] `Status` has no case `.gone`",
    );
}

#[test]
fn format_and_ago_are_strings_with_a_known_style() {
    clean(
        "page P(path: \"/\") { state n = 1234.5\n state d = \"2024-03-05\"\n derived a = format(n, .currency) + format(n, .currency, \"EUR\") + format(n) + format(d, \"yyyy-MM-dd\") + format(d, .date, \"long\") + ago(d)\n Text(a.toUpperCase()) }",
    );
    has(
        "page P(path: \"/\") { state n = 1\n Text(format(n, .money)) }",
        "[T02] `format` has no style `.money`",
    );
    has(
        "page P(path: \"/\") { state n = 1\n Text(format(n, 2)) }",
        "[T01] the style of `format` is `Number`, but a `.style` or a date pattern is wanted",
    );
    has(
        "page P(path: \"/\") { state n = 1\n Text(format()) }",
        "[T10] `format` takes 1 to 3 arguments, but 0 are given",
    );
    // A page's own `format` would hide the language's: `E118`.
    has(
        "page P(path: \"/\") { state n = 1\n action format(x: Number) { return \"{x}!\" }\n Text(format(n)) }",
        "[E118] `format` is a name the language already has",
    );
}

#[test]
fn a_form_handle_and_an_actions_pending_are_typed() {
    clean(
        "page P(path: \"/\") { state ok = false\n action save() { let r = await fetch(\"/x\")  ok = true }\n Form(bind: f) { Input(name: \"a\")  Button(\"s\", disabled: !f.valid || save.pending) { on click { f.reset()  log(f.values.a) } } } }",
    );
    has(
        "page P(path: \"/\") { action save() { log(1) }\n Text(\"{save.nope}\") }",
        "[T05] `save` is an action; it has no field `nope`",
    );
    has(
        "page P(path: \"/\") { Form(bind: f) { Input(name: \"a\") }\n Text(f.vald) }",
        "[T05] `f` has no field `vald`",
    );
}

#[test]
fn refs_and_timers_are_typed() {
    clean(
        "page P(path: \"/\") { state n = 0\n Input(bind: q, ref: box)\n state q = \"\"\n Button(\"x\") { on click { box.focus()  box.value = \"\" } }\n every(1000) { n = n + 1 }\n effect { log(n)  cleanup { log(n) } } }",
    );
    has(
        "page P(path: \"/\") { state n = 0\n every(\"soon\") { n = n + 1 } }",
        "[T01] `every` takes milliseconds, but `\"soon\"` is `String`",
    );
    has(
        "page P(path: \"/\") { state q = \"\"\n Input(bind: q, ref: \"box\") }",
        "[T01] `ref:` on `Input` names the handle, but `\"box\"` is given",
    );
}

#[test]
fn a_scoped_slot_types_its_values_and_a_fills_names() {
    clean(&format!(
        "{TODOS}component Rows(items: [Todo]) {{ slot row(item: Todo, index: Number)  for it, i in items {{ row(item: it, index: i) }} }}\npage P(path: \"/\") {{ state todos: [Todo] = []\n  Rows(items: todos) {{ row(item, i) {{ Text(\"{{i}}: {{item.title}}\") }} }} }}"
    ));
    has(
        &format!(
            "{TODOS}component Rows(items: [Todo]) {{ slot row(item: Todo, index: Number)  for it, i in items {{ row(item: it, index: \"x\") }} }}\npage P(path: \"/\") {{ Text(\"x\") }}"
        ),
        "[T01] `index` of `row` is `String`, but `Number` is wanted",
    );
    has(
        &format!(
            "{TODOS}component Rows(items: [Todo]) {{ slot row(item: Todo)  for it in items {{ row(item: it) }} }}\npage P(path: \"/\") {{ state todos: [Todo] = []\n  Rows(items: todos) {{ row(item) {{ Text(item.nam) }} }} }}"
        ),
        "[T05] `Todo` has no field `nam`",
    );
}

#[test]
fn an_error_is_placed_at_the_expression_it_names_when_the_source_is_at_hand() {
    let src = format!(
        "{TODOS}page P(path: \"/\") {{\n    state sel: Todo? = null\n    derived n = 1 + sel.title.length\n    Text(\"x\")\n}}"
    );
    let program = crate::syntax::parse_source(&src, "t.wf").unwrap();
    let info = check_in(&program, &|_| "t.wf".to_string(), &|_| Some(src.clone()));
    let error = info
        .findings
        .errors
        .iter()
        .find(|e| e.code == "T04")
        .unwrap();
    // `sel` sits on the derived line, after `derived n = 1 + `.
    let line = src.lines().position(|l| l.contains("derived n")).unwrap() + 1;
    assert_eq!(error.line, line, "{error:?}");
    assert_eq!(error.column, "    derived n = 1 + ".len() + 1, "{error:?}");
}

#[test]
fn a_member_that_does_not_exist_is_found_at_the_member_not_the_store() {
    // The fix renames `fieldz`, and the editor marks it: the build's
    // line and column are the same place, not `S`'s.
    let src = "store S {\n    state fields = 0\n}\npage P(path: \"/\") {\n    use S\n    Text(\"{S.fieldz}\")\n}\n".to_string();
    let program = crate::syntax::parse_source(&src, "t.wf").unwrap();
    let info = check_in(&program, &|_| "t.wf".to_string(), &|_| Some(src.clone()));
    let error = info
        .findings
        .errors
        .iter()
        .find(|e| e.code == "T06")
        .unwrap();
    assert_eq!(
        (error.line, error.column),
        (6, "    Text(\"{S.".len() + 1),
        "{error:?}"
    );
}
