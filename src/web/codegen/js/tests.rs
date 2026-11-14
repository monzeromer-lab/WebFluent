use super::*;
use crate::syntax::parser::ast::{
    Assignment, EmitStmt, EventDecl, MatchArm, PropDecl, SlotDecl, SlotFill,
};

fn compile(src: &str) -> String {
    let program = crate::syntax::parse_source(src, "<t>").expect("parse");
    JsCodegen::new().generate(&crate::check::sema::lower(program))
}

#[test]
fn a_state_starts_from_a_derived_value_defined_before_it() {
    let out = compile(
        "component Level(start: String) {\n    state level: Number = number\n    state other = 1\n    derived unused = other + 1\n    derived number: Number = Number(raw)\n    derived raw = start\n    Text(\"{level}\")\n}\n",
    );
    let at = |s: &str| out.find(s).unwrap_or_else(|| panic!("{s} in {out}"));
    // What `level` reads comes first, and what that reads before it;
    // what no state reads stays where it was.
    assert!(at("const _raw = WF.computed") < at("const _number = WF.computed"));
    assert!(at("const _number = WF.computed") < at("const _level = WF.signal(_number())"));
    assert!(at("const _level =") < at("const _other ="));
    assert!(at("const _frag") < at("const _unused = WF.computed"));
    assert_eq!(out.matches("const _number =").count(), 1);
    // A page too.
    let page = compile(
        "page Home(path: \"/\") {\n    state shown = first\n    derived first = \"a\"\n    Text(shown)\n}\n",
    );
    assert!(
        page.find("const _first =").unwrap() < page.find("const _shown =").unwrap(),
        "{page}"
    );
}

fn compile_ssg(src: &str) -> String {
    let program = crate::syntax::parse_source(src, "<t>").expect("parse");
    let mut codegen = JsCodegen::new();
    codegen.set_ssg(true);
    codegen.generate(&crate::check::sema::lower(program))
}

#[test]
fn a_script_loaded_after_the_page_is_asked_for_by_the_runtime() {
    let program =
        crate::syntax::parse_source("page P(path: \"/\") { Text(\"x\") }", "<t>").expect("parse");
    let mut codegen = JsCodegen::new();
    codegen.set_later_scripts(vec![
        (
            "https://tags.example/t.js".to_string(),
            Some("sha384-abc".to_string()),
        ),
        ("/widget.js".to_string(), None),
    ]);
    let out = codegen.generate(&crate::check::sema::lower(program));
    assert!(
            out.contains(r#"WF.loadAfter([{ src: "https://tags.example/t.js", integrity: "sha384-abc" }, { src: "/widget.js" }]);"#),
            "{out}"
        );
    assert!(
        out.contains("function loadAfter("),
        "the runtime module comes with it"
    );
}

#[test]
fn a_script_loaded_on_consent_waits_for_the_readers_yes() {
    let program =
        crate::syntax::parse_source("page P(path: \"/\") { Text(\"x\") }", "<t>").expect("parse");
    let mut codegen = JsCodegen::new();
    codegen.set_consent_scripts(vec![("/count.js".to_string(), None)]);
    let out = codegen.generate(&crate::check::sema::lower(program));
    assert!(
        out.contains(r#"WF.consentScripts([{ src: "/count.js" }]);"#),
        "{out}"
    );
    assert!(out.contains("function consentScripts("), "{out}");
}

/// A static build draws its app beside the pre-rendered one and takes
/// that over (runtime/modules/hydrate.js); a single-page build still
/// clears the root.
#[test]
fn a_static_build_draws_beside_the_painted_page_and_names_what_its_closures_hold() {
    let src = concat!(
        "app { Router }\n",
        "page P(path: \"/\") {\n",
        "  state pct = 30\n",
        "  Heading(\"Kept\").h1\n",
        "  Stack { style { width: {pct}% } }\n",
        "  Tabs { Tabs.Page(\"One\") { Text(\"1\") } Tabs.Page(\"Two\") { Text(\"2\") } }\n",
        "}",
    );
    let ssg = compile_ssg(src);
    assert!(
        ssg.contains("const _app = WF.hydrating(document.getElementById('app'));"),
        "{ssg}"
    );
    assert!(!ssg.contains("_app.innerHTML = ''"), "{ssg}");
    let live = ssg
        .lines()
        .find(|l| l.trim_start().starts_with("WF.live("))
        .expect("a WF.live line");
    // The bar whose width follows `pct`, and the tab pages an effect shows.
    let held = live.matches("_e").count();
    assert!(held >= 3, "{live}");
    let spa = compile(src);
    assert!(spa.contains("_app.innerHTML = '';"), "{spa}");
    assert!(
        !spa.contains("WF.live("),
        "a single-page build takes nothing over: {spa}"
    );
    assert!(!spa.contains("WF.hydrating("), "{spa}");
}

#[test]
fn a_services_hook_runs_what_it_says_with_its_parameter_bound() {
    let out = compile(concat!(
        "store L { state last = \"\"\n action note(s: String) { last = s } }\n",
        "api B(base: \"/api\") {\n",
        "  on request(r) { r.headers[\"X-Id\"] = \"abc\"  L.note(r.url) }\n",
        "  get me() -> Map\n}\n",
        "page P(path: \"/\") { resource m = B.me()\n Text(L.last)\n",
        "  match m { ready(v) { Text(\"{v}\") } else { Text(\"…\") } } }",
    ));
    // The body is emitted at all, and `r` is the parameter, not a signal.
    assert!(out.contains("r.headers[\"X-Id\"] = \"abc\""), "{out}");
    assert!(out.contains("L.note(r.url)"), "{out}");
    assert!(!out.contains("_r()"), "{out}");
}

#[test]
fn a_match_over_a_connection_reads_the_handle_not_a_signal() {
    let out = compile(concat!(
        "page P(path: \"/\") {\n",
        "  socket chat = ws(\"wss://e.com/c\")\n",
        "  match chat { open { Text(\"on\") } else { Text(\"off\") } } }",
    ));
    assert!(out.contains("_chat.state()"), "{out}");
    assert!(!out.contains("_chat()"), "{out}");
}

#[test]
fn an_if_let_in_an_imperative_body_binds_a_plain_name() {
    let out = compile(concat!(
        "page P(path: \"/\") {\n",
        "  state price = 0\n",
        "  stream ticks = sse(\"/events\")\n",
        "  effect { if let p = ticks.last(\"price\") { price = p } }\n",
        "  Text(\"{price}\") }",
    ));
    assert!(out.contains("_price.set(p)"), "{out}");
    assert!(!out.contains("_p()"), "{out}");
}

#[test]
fn a_sockets_handler_reads_what_arrived_by_name() {
    let out = compile(concat!(
        "store F { state items = []\n action add(m: Map) { items = items.concat([m]) } }\n",
        "page P(path: \"/\") {\n",
        "  socket chat = ws(\"wss://e.com/c\") { on message(m) { F.add(m) } }\n",
        "  Text(\"{F.items.length}\")\n",
        "  match chat { open { Text(\"on\") } else { Text(\"off\") } } }",
    ));
    assert!(out.contains("onMessage: (m) => { F.add(m); }"), "{out}");
}

// ─── Nodes the new grammar produces, built by hand until it parses ───

fn stmt(kind: StatementKind) -> Statement {
    Statement::new(kind, Span::dummy())
}

fn element(name: &str, args: Vec<Arg>, children: Vec<Statement>) -> UIElement {
    UIElement {
        component: ComponentRef::BuiltIn(name.to_string()),
        args,
        modifiers: Vec::new(),
        children,
        style_block: None,
        transition_block: None,
        events: Vec::new(),
        slot_fills: Vec::new(),
        span: Span::dummy(),
        paren_span: None,
        body_span: None,
        style_span: None,
        arg_spans: Vec::new(),
        modifier_spans: Vec::new(),
    }
}

fn text(expr: Expr) -> Statement {
    stmt(StatementKind::UIElement(element(
        "Text",
        vec![Arg::Positional(expr)],
        Vec::new(),
    )))
}

fn page(name: &str, body: Vec<Statement>) -> Declaration {
    Declaration::Page(PageDecl {
        name: name.to_string(),
        path: "/".to_string(),
        title: None,
        title_expr: None,
        description_expr: None,
        image_expr: None,
        guard: None,
        redirect: None,
        description: None,
        image: None,
        image_alt: None,
        page_type: None,
        noindex: false,
        lang: None,
        translates: None,
        layout: None,
        params: Vec::new(),
        head: Vec::new(),
        paths: None,
        body,
        span: Span::dummy(),
        header_span: Span::dummy(),
        body_span: Span::dummy(),
    })
}

fn generate(declarations: Vec<Declaration>) -> String {
    JsCodegen::new().generate(&Program { declarations })
}

#[test]
fn a_resource_is_declared_once_and_a_match_renders_its_states() {
    let ident = |n: &str| Expr::Identifier(n.to_string());
    let arm = |pattern, binding: Option<&str>, body| MatchArm {
        pattern,
        binding: binding.map(str::to_string),
        bindings: Vec::new(),
        body,
        span: Span::dummy(),
    };
    let out = generate(vec![page(
        "Home",
        vec![
            stmt(StatementKind::Resource(ResourceDecl {
                name: "rows".to_string(),
                ty: None,
                url: Expr::StringLiteral("/api/rows".to_string()),
                options: Vec::new(),
            })),
            stmt(StatementKind::Match(MatchStmt {
                scrutinee: ident("rows"),
                arms: vec![
                    arm(
                        ArmPattern::Loading,
                        None,
                        vec![text(Expr::StringLiteral("…".into()))],
                    ),
                    arm(
                        ArmPattern::Error,
                        Some("e"),
                        vec![text(Expr::PropertyAccess(
                            Box::new(ident("e")),
                            "message".to_string(),
                        ))],
                    ),
                    arm(
                        ArmPattern::Ready,
                        Some("list"),
                        vec![text(Expr::PropertyAccess(
                            Box::new(ident("list")),
                            "length".to_string(),
                        ))],
                    ),
                ],
            })),
            stmt(StatementKind::UIElement(element(
                "Button",
                vec![Arg::Positional(Expr::StringLiteral("Reload".into()))],
                vec![stmt(StatementKind::ExprStatement(Expr::MethodCall(
                    Box::new(ident("rows")),
                    "reload".to_string(),
                    Vec::new(),
                )))],
            ))),
        ],
    )]);
    assert!(
        out.contains("const _rows = WF.resource(\"/api/rows\", null);"),
        "{out}"
    );
    assert!(out.contains("WF.match(_root, () => _rows.state(), () => _rows.state() === \"error\" ? _rows.error() : _rows.data(), {"), "{out}");
    assert!(out.contains("loading: ($$value, $$arm) => {"), "{out}");
    // An arm is handed its value and an accessor of it, and reads its
    // binding through the accessor, so what it drew follows a refresh.
    assert!(
        out.contains("error: ($$value, $e) => {") && out.contains("() => $e().message"),
        "{out}"
    );
    assert!(
        out.contains("ready: ($$value, $list) => {") && out.contains("() => $list().length"),
        "{out}"
    );
    // A reference to the resource is the object, not a signal read.
    assert!(out.contains("_rows.reload()"), "{out}");
    assert!(!out.contains("_rows()"), "{out}");
}

#[test]
fn a_match_over_an_enum_picks_the_arm_by_case() {
    let out = generate(vec![page(
        "Home",
        vec![
            stmt(StatementKind::State(StateDecl {
                name: "tone".to_string(),
                ty: None,
                value: Expr::EnumCase("danger".to_string()),
                persist: false,
                policy: None,
                doc: None,
            })),
            stmt(StatementKind::Match(MatchStmt {
                scrutinee: Expr::Identifier("tone".to_string()),
                arms: vec![
                    MatchArm {
                        pattern: ArmPattern::Case("danger".to_string()),
                        binding: None,
                        bindings: Vec::new(),
                        body: vec![text(Expr::StringLiteral("red".into()))],
                        span: Span::dummy(),
                    },
                    MatchArm {
                        pattern: ArmPattern::Else,
                        binding: None,
                        bindings: Vec::new(),
                        body: vec![text(Expr::StringLiteral("calm".into()))],
                        span: Span::dummy(),
                    },
                ],
            })),
        ],
    )]);
    assert!(
        out.contains("WF.signal(\"danger\")"),
        "an enum case is its name: {out}"
    );
    assert!(
        out.contains("WF.match(_root, () => WF.caseOf(_tone()), () => _tone(), {"),
        "{out}"
    );
    assert!(
        out.contains("danger: ($$value, $$arm) => {")
            && out.contains("else: ($$value, $$arm) => {"),
        "{out}"
    );
}

#[test]
fn a_declared_event_is_emitted_to_the_handler_the_caller_passed() {
    let component = Declaration::Component(ComponentDecl {
        name: "Row".to_string(),
        props: vec![PropDecl {
            name: "label".to_string(),
            prop_type: TypeRef::String,
            optional: false,
            default: None,
            positional: true,
            doc: None,
            span: Span::dummy(),
        }],
        events: vec![EventDecl {
            name: "pick".to_string(),
            params: Vec::new(),
            doc: None,
            span: Span::dummy(),
        }],
        slots: vec![SlotDecl {
            params: Vec::new(),
            name: Some("trailing".to_string()),
            span: Span::dummy(),
        }],
        parts: Vec::new(),
        doc: None,
        body: vec![
            stmt(StatementKind::UIElement({
                let mut b = element(
                    "Button",
                    vec![Arg::Positional(Expr::Identifier("label".to_string()))],
                    Vec::new(),
                );
                b.events.push(EventHandler {
                    pace: None,
                    event: "click".to_string(),
                    params: vec!["ev".to_string()],
                    key: None,
                    body: vec![stmt(StatementKind::Emit(EmitStmt {
                        event: "pick".to_string(),
                        args: vec![Expr::Identifier("label".to_string())],
                    }))],
                    span: Span::dummy(),
                });
                b
            })),
            stmt(StatementKind::UIElement(element(
                "Children",
                vec![Arg::Named(
                    "slot".to_string(),
                    Expr::StringLiteral("trailing".to_string()),
                )],
                Vec::new(),
            ))),
        ],
        span: Span::dummy(),
        header_span: Span::dummy(),
        body_span: Span::dummy(),
    });
    let mut call = UIElement {
        component: ComponentRef::UserDefined("Row".to_string()),
        ..element(
            "Row",
            vec![Arg::Positional(Expr::StringLiteral("Hi".into()))],
            Vec::new(),
        )
    };
    call.events.push(EventHandler {
        pace: None,
        event: "pick".to_string(),
        params: vec!["which".to_string()],
        key: None,
        body: vec![stmt(StatementKind::Log(Expr::Identifier(
            "which".to_string(),
        )))],
        span: Span::dummy(),
    });
    call.events.push(EventHandler {
        pace: None,
        event: "mouseenter".to_string(),
        params: Vec::new(),
        key: None,
        body: vec![stmt(StatementKind::Log(Expr::StringLiteral("in".into())))],
        span: Span::dummy(),
    });
    call.slot_fills.push(SlotFill {
        params: Vec::new(),
        name: "trailing".to_string(),
        body: vec![text(Expr::StringLiteral("→".into()))],
        span: Span::dummy(),
        body_span: Span::dummy(),
    });
    let out = generate(vec![
        component,
        page("Home", vec![stmt(StatementKind::UIElement(call))]),
    ]);
    // Inside the component: the handler's own parameter, and the emit.
    assert!(
        out.contains("addEventListener(\"click\", (ev) => { WF.emit(_p, \"pick\", _p.label); })"),
        "{out}"
    );
    assert!(
        out.contains(
            "if (typeof _slots.trailing === 'function') _frag.appendChild(_slots.trailing());"
        ),
        "{out}"
    );
    // At the call: the positional prop by name, the declared event as a
    // handler in the props, the DOM event on the root, the named fill.
    assert!(
        out.contains(
            "Component_Row({ label: \"Hi\", on: { pick: (which) => { console.log(which); } } }, {"
        ),
        "{out}"
    );
    assert!(out.contains("trailing: () => {"), "{out}");
    assert!(
        out.contains("WF.onRoot(_e2, \"mouseenter\", (event) => { console.log(\"in\"); });"),
        "{out}"
    );
}

#[test]
fn a_router_without_routes_takes_every_page_most_specific_first() {
    let out = compile(
        "page Deploy(path: \"/deploys/:id\", id: String) { Text(id) }\npage NotFound(path: \"*\") { Text(\"?\") }\npage Home(path: \"/\") { Text(\"h\") }\npage Deploys(path: \"/deploys\") { Text(\"d\") }\napp { Router }",
    );
    let table = out.find("const _routes = [").unwrap();
    let order: Vec<usize> = ["\"/deploys\"", "\"/\"", "\"/deploys/:id\"", "\"*\""]
        .iter()
        .map(|p| out[table..].find(&format!("path: {p}")).unwrap())
        .collect();
    assert!(
        order[0] < order[1] && order[1] < order[2] && order[2] < order[3],
        "{out}"
    );
    assert!(
        out.contains("Page_Deploy(params)") && out.contains("params.id"),
        "{out}"
    );
}

/// A layout is called with the route's parameters, so its arguments may
/// read them. Compiled without them in scope, a parameter named after a
/// browser value — `hash` — became `location.hash`.
#[test]
fn a_layout_argument_reads_the_routes_parameters() {
    let out = compile(
        r#"
            component Shell(crumb: String) { slot  Text(crumb)  children }
            page Build(path: "/b/:hash", hash: String, title: "Build", layout: Shell(crumb: "Build {hash}")) {
                Text(hash)
            }
            app { Router }
            "#,
    );
    assert!(out.contains("`Build ${params.hash}`"), "{out}");
    assert!(!out.contains("WF.hash()"), "{out}");
}

#[test]
fn a_page_with_a_layout_is_rendered_inside_it() {
    let out = compile(
        "component Shell(crumb: String) { slot  Text(crumb)  children }\npage Home(path: \"/\", layout: Shell(crumb: \"Home\")) { Text(\"h\") }\napp { Router }",
    );
    assert!(
            out.contains("layout: (page, params) => Component_Shell({ crumb: \"Home\" }, { children: () => page(params) }), render: (params) => Page_Home(params)"),
            "{out}"
        );
}

#[test]
fn if_let_binds_the_value_and_for_by_passes_the_key() {
    let out = generate(vec![page(
        "Home",
        vec![
            stmt(StatementKind::State(StateDecl {
                name: "hint".to_string(),
                ty: None,
                value: Expr::Null,
                persist: false,
                policy: None,
                doc: None,
            })),
            stmt(StatementKind::State(StateDecl {
                name: "items".to_string(),
                ty: None,
                value: Expr::ListLiteral(Vec::new()),
                persist: false,
                policy: None,
                doc: None,
            })),
            stmt(StatementKind::If(IfStmt {
                condition: Expr::Identifier("hint".to_string()),
                binding: Some("h".to_string()),
                animate: None,
                animate_span: None,
                then_body: vec![text(Expr::Identifier("h".to_string()))],
                else_if_branches: Vec::new(),
                else_body: None,
            })),
            stmt(StatementKind::For(ForStmt {
                item: "it".to_string(),
                index: None,
                iterable: Expr::Identifier("items".to_string()),
                key: Some(Expr::PropertyAccess(
                    Box::new(Expr::Identifier("it".to_string())),
                    "id".to_string(),
                )),
                animate: None,
                animate_span: None,
                body: vec![text(Expr::Identifier("it".to_string()))],
            })),
            stmt(StatementKind::Action(ActionDecl {
                name: "load".to_string(),
                params: Vec::new(),
                body: vec![stmt(StatementKind::Assignment(Assignment {
                    target: Expr::Identifier("hint".to_string()),
                    value: Expr::Await(Box::new(Expr::FunctionCall(
                        "fetch".to_string(),
                        vec![Expr::StringLiteral("/x".into())],
                    ))),
                }))],
                concurrent: false,
                doc: None,
            })),
        ],
    )]);
    assert!(out.contains("() => _hint() != null"), "{out}");
    // The binding follows the value: a new hint reaches the branch.
    assert!(out.contains("const $h = WF.held(() => _hint());"), "{out}");
    assert!(out.contains("() => $h()"), "{out}");
    assert!(
        out.contains("{ key: (it) => it.id, live: true }"),
        "the key function is in the list's options: {out}"
    );
    assert!(
        out.contains("($$item, $$index, $it) => {") && out.contains("() => $it()"),
        "a keyed item is read live: {out}"
    );
    assert!(
        out.contains("function load() {")
            && out.contains("return WF.sameCall(_load_pending, [], async () => {")
            && out.contains("(await WF.request(\"/x\"))"),
        "{out}"
    );
}

#[test]
fn motion_off_a_branch_root_is_a_marker_and_the_router_takes_a_transition() {
    let out = compile(
        "component Chip(_ label: String) { Badge(label) }\npage P(path: \"/\") {\n  Card { Text(\"x\", exit: .fadeOut, delay: \"100ms\")  Chip(\"y\", exit: .fadeOut).fadeIn.fast }\n}\napp { Router(transition: .slide, duration: \"150ms\") }",
    );
    assert!(
        out.contains("\"data-wf-exit\": \"fadeOut\", \"data-wf-delay\": \"100ms\""),
        "{out}"
    );
    assert!(
            out.contains("{ \"data-wf-exit\": \"fadeOut\", \"data-wf-animate\": \"fadeIn\", \"data-wf-duration\": \"150ms\" });"),
            "{out}"
        );
    assert!(
        out.contains(
            "WF.router(_routes, _routerEl, { transition: \"slide\", duration: \"150ms\", play: WF.playRoute });"
        ),
        "{out}"
    );
}

/// `WF.each` hands the body its item as a plain callback parameter, so
/// every reference to it must stay plain.
///
/// The identifier path treated anything that was not a prop or a store as
/// state, so a loop over `tasks` bound `task` and then read `_task()` —
/// `ReferenceError` on the first non-empty list. `wf init -t spa` shipped it.
#[test]
fn a_component_receives_its_callers_block_as_children() {
    let out = compile(
        r#"
            component Panel(title: String) {
                Card { Heading(title).h3 children }
            }
            page P(path: "/") {
                state n = 1
                Panel(title: "Keys") { Text("count {n}") }
                Panel(title: "Empty")
            }
            
            "#,
    );
    assert!(
        out.contains("function Component_Panel(_p, _slots)"),
        "{out}"
    );
    assert!(
        out.contains("Component_Panel({ title: \"Keys\" }, {") && out.contains("children: () => {"),
        "{out}"
    );
    // The block is compiled in the caller's scope.
    assert!(out.contains("_n()"), "{out}");
    assert!(
        out.contains("Component_Panel({ title: \"Empty\" });"),
        "{out}"
    );
}

#[test]
fn a_store_derived_may_build_on_derived_values_actions_and_if_expressions() {
    let out = compile(
        r#"
            store Pricing {
                state seats = 6
                state annual = true
                derived rate = if annual { 14 } else { 18 }
                derived cost = Math.round(seats * rate)
                derived label = "{cost} units"
                derived share = pctOf(cost)
                action pctOf(part: Number) {
                    if cost == 0 { return 0 }
                    return Math.round(part / cost * 100)
                }
                action bump() { seats = seats + 1  log(share) }
            }
            page P(path: "/") { use Pricing  Text("{Pricing.label}") }
            
            "#,
    );
    assert!(
        out.contains("rate: (store) => (store.annual ? 14 : 18)"),
        "{out}"
    );
    assert!(
        out.contains("cost: (store) => Math.round((store.seats * store.rate))"),
        "{out}"
    );
    assert!(
        out.contains("label: (store) => `${store.cost} units`"),
        "{out}"
    );
    assert!(
        out.contains("share: (store) => store.pctOf(store.cost)"),
        "{out}"
    );
    assert!(
        out.contains("return Math.round(((part / store.cost) * 100));"),
        "{out}"
    );
    assert!(!out.contains("_part()"), "{out}");
}

#[test]
fn an_input_keeps_its_binding_beside_the_authors_input_handler() {
    let out = compile(
        r#"
            store S { state q = ""  action set(v: String) { q = v } }
            page P(path: "/") {
                use S
                state email = ""
                Input(email, bind: email, placeholder: "e") { on input { S.set(email) } }
                Form { on submit { S.set("sent") } Text("f")  }
            }
            
            "#,
    );
    assert!(
        out.contains("\"on:input\": ($$e) => { _email.set(WF.bound($$e.target)); }"),
        "{out}"
    );
    assert!(
        out.contains(".addEventListener(\"input\", (event) => { S.set(_email()); });"),
        "{out}"
    );
    assert!(
        out.contains("\"on:submit\": (e) => e.preventDefault()"),
        "{out}"
    );
    assert!(
        out.contains(".addEventListener(\"submit\", (event) => { S.set(\"sent\"); });"),
        "{out}"
    );
}

#[test]
fn a_buttons_block_may_mix_content_and_actions() {
    let out = compile(
        r#"
            page P(path: "/") {
                state open = false
                Button("", aria-expanded: open) {
                    on click {
                        open = !open
                    }
                    Text("Details")
                    Icon("chevron-down")
                }
            }
            
            "#,
    );
    assert!(
        out.contains(".addEventListener(\"click\", (event) => { _open.set(!_open()); });"),
        "{out}"
    );
    // The assignment is not executed at render time.
    assert!(!out.contains("\n  _open.set(!_open());"), "{out}");
    assert!(out.contains("\"Details\""), "{out}");
}

#[test]
fn a_sub_component_keeps_its_style_block_attributes_and_handlers() {
    let out = compile(
        r#"
            page P(path: "/") {
                state n = 0
                List { List.Item(id: "first", aria-current: "true") { style { padding: 0; &:hover { color: red } } on click { n = n + 1 } Text("a") } }
            }
            
            "#,
    );
    assert!(out.contains("WF.el(\"li\", { className: \"wf-list__item\", id: \"first\", \"aria-current\": \"true\" })"), "{out}");
    // `padding: "0"` is a literal: hoisted into the scoped class, not inline.
    assert!(!out.contains(".style.padding"), "{out}");
    assert!(out.contains(".classList.add(\"wf-s"), "{out}");
    assert!(
        out.contains(".addEventListener(\"click\", (event) => { _n.set((_n() + 1)); });"),
        "{out}"
    );
}

#[test]
fn a_map_literal_returned_from_a_lambda_is_an_object_not_a_block() {
    let out = compile(
        r#"
            store S {
                state items = [1, 2]
                derived pairs = items.map(i => { n: i, twice: i * 2 })
                derived counts = { all: items.length }
            }
            page P(path: "/") { use S  Text("x") }
            
            "#,
    );
    assert!(
        out.contains("items.map(((i) => ({ n: i, twice: (i * 2) })))"),
        "{out}"
    );
    assert!(
        out.contains("counts: (store) => ({ all: store.items.length })"),
        "{out}"
    );
}

#[test]
fn a_store_action_declares_its_locals() {
    let out = compile(
        r#"
            store S {
                state secret = ""
                action headers() {
                    h = {}
                    h["Authorization"] = "Bearer " + secret
                    h = h
                    return h
                }
            }
            page P(path: "/") { use S  Text("x") }
            
            "#,
    );
    assert!(out.contains("let h = ({  });"), "{out}");
    assert!(
        out.contains("h[\"Authorization\"] = (\"Bearer \" + store.secret);"),
        "{out}"
    );
    assert_eq!(out.matches("let h = ").count(), 1, "{out}");
}

#[test]
fn an_action_parameter_shadows_a_store_member_of_the_same_name() {
    let out = compile(
        r#"
            store S {
                state at = 0
                action step(n: Number) { return n + 1 }
                action move(step: Number) { at = at + step }
            }
            page P(path: "/") { use S  Text("x") }
            
            "#,
    );
    assert!(out.contains("store.at = (store.at + step);"), "{out}");
    assert!(!out.contains("store.step"), "{out}");
}

#[test]
fn a_local_shadows_a_store_member_of_the_same_name_within_its_scope() {
    let out = compile(
        r#"
            store S {
                state refs = ["a"]
                state count = 0
                derived sizes = refs.map(refs => refs.length)
                action add(x: String) {
                    let refs = [x, x]
                    count = refs.length
                    refs = [x]
                    if count > 0 {
                        let count = 5
                        log(count)
                    }
                    count = count + refs.length
                }
                action each() {
                    for refs in [[1, 2, 3]] {
                        let count = refs.length
                        log(count)
                    }
                    let n = [[4]].map(refs => refs.length)
                    try { log(n) } catch refs { log(refs) }
                    count = count + refs.length
                }
            }
            page P(path: "/") { use S  Text("{S.sizes.length}") }
            "#,
    );
    let add = &out[out.find("add: (store, x)").unwrap()..];
    let add = &add[..add.find("each:").unwrap()];
    assert!(add.contains("let refs = [x, x];"), "{add}");
    assert!(add.contains("store.count = refs.length;"), "{add}");
    assert!(
        add.contains("\n      refs = [x];"),
        "the local is assigned, not the store: {add}"
    );
    assert!(
        add.contains("let count = 5;") && add.contains("console.log(count);"),
        "{add}"
    );
    assert!(
        add.contains("store.count = (store.count + refs.length);"),
        "a block's local ends with it: {add}"
    );
    let each = &out[out.find("each: (store)").unwrap()..];
    assert!(each.contains("let count = refs.length;"), "{each}");
    assert!(each.contains("((refs) => refs.length)"), "{each}");
    assert!(each.contains("console.log(refs);"), "{each}");
    assert!(
        each.contains("store.count = (store.count + store.refs.length);"),
        "past the loop, the store's again: {each}"
    );
    assert!(
        out.contains("sizes: (store) => store.refs.map(((refs) => refs.length))"),
        "{out}"
    );
}

#[test]
fn a_local_shadows_a_prop_of_the_same_name_within_its_block() {
    let out = compile(
        r#"
            component Box(items: [String] = [], label: String = "L") {
                Text("{items.map(label => label.length)} {label}")
                Button("Box") { on click {
                    let label = "mine"
                    log(label)
                    for items in [["q"]] { log(items.length) }
                    log(items.length)
                } }
            }
            page P(path: "/") { Box(items: ["a"]) }
            "#,
    );
    assert!(
        out.contains("_p.items.map(((label) => label.length))"),
        "{out}"
    );
    assert!(
        out.contains("const _label = WF.signal(\"mine\"); console.log(_label());"),
        "{out}"
    );
    assert!(
            out.contains("for (const items of [[\"q\"]]) { console.log(items.length); } console.log(_p.items.length);"),
            "{out}"
        );
}

#[test]
fn a_known_attribute_that_reads_state_follows_it() {
    let out = compile(
        r#"
            store S { state busy = false  derived hint = if busy { "wait" } else { "type" } }
            page P(path: "/") {
                use S
                Input(placeholder: S.hint, disabled: S.busy)
                Input(placeholder: "fixed")
            }
            
            "#,
    );
    assert!(out.contains("placeholder: () => S.hint"), "{out}");
    assert!(out.contains("disabled: () => S.busy"), "{out}");
    assert!(out.contains("placeholder: \"fixed\""), "{out}");
}

#[test]
fn a_slot_inside_a_select_is_passed_with_the_element() {
    let out = compile(
        r#"
            component Picker(value: String) { Select(value: value) { children } }
            page P(path: "/") { Picker(value: "b") { Select.Option("A", value: "a")  Select.Option("B", value: "b") } }
            
            "#,
    );
    assert!(
            out.contains("WF.el(\"select\", { className: \"wf-select\", value: () => _p.value }, (typeof _slots.children === 'function' ? _slots.children() : null))"),
            "{out}"
        );
    assert!(!out.contains("appendChild(_slots.children())"), "{out}");
}

#[test]
fn an_else_if_chain_with_a_final_else_keeps_every_branch() {
    let out = compile(
        r#"
            page P(path: "/") {
                state v = "no"
                if v == "yes" { Text("Y") } else if v == "no" { Text("N") } else { Text("other") }
            }
            
            "#,
    );
    assert!(out.contains("(_v() === \"yes\")"), "{out}");
    assert!(out.contains("(_v() === \"no\")"), "{out}");
    assert!(out.contains("\"other\""), "{out}");
}

#[test]
fn a_sliders_own_input_handler_runs_after_the_binding() {
    let out = compile(
        r#"
            store S { state n = 0  action set(v: Number) { n = v } }
            page P(path: "/") {
                use S
                state req = 4
                Slider(bind: req, min: 0, max: 8, step: 1, aria-label: "Requests") { on input { S.set(req) } }
            }
            
            "#,
    );
    assert_eq!(out.matches("\"on:input\"").count(), 1, "{out}");
    assert!(
        out.contains("_req.set(Number(event.target.value)); S.set(_req());"),
        "{out}"
    );
}

#[test]
fn a_slider_carries_aria_attributes_and_defers_to_aria_valuetext() {
    let out = compile(
        r#"
            page P(path: "/") {
                state req = 4
                derived shown = "5M"
                Slider(bind: req, min: 0, max: 8, step: 1, aria-labelledby: "d-req", aria-valuetext: shown)
                Slider(bind: req, min: 0, max: 8, step: 1, aria-label: "Seats")
            }
            
            "#,
    );
    assert!(out.contains("\"aria-labelledby\": \"d-req\""), "{out}");
    assert!(out.contains("\"aria-valuetext\": () => _shown()"), "{out}");
    assert_eq!(out.matches("wf-slider__value").count(), 1, "{out}");
}

#[test]
fn a_prop_that_reads_state_is_passed_as_a_getter_and_read_live() {
    let out = compile(
        r#"
            component Chip(label: String, pressed: Bool = false) {
                derived bg = if pressed { "a" } else { "b" }
                Button(label, aria-pressed: pressed) { style { background: {bg} } }
            }
            page P(path: "/") {
                state on = true
                Chip(label: "Errors", pressed: on) { on click { on = !on } }
            }
            "#,
    );
    assert!(
        out.contains("Component_Chip({ label: \"Errors\", get pressed() { return _on(); } })"),
        "{out}"
    );
    assert!(
        out.contains("WF.computed(() => (_p.pressed ? \"a\" : \"b\"))"),
        "{out}"
    );
    assert!(out.contains("\"aria-pressed\": () => _p.pressed"), "{out}");
}

#[test]
fn a_prop_that_reads_a_store_named_like_a_browser_global_is_a_getter_too() {
    // `History` is the browser's, so the store is `History$` in the
    // generated code; a prop that read only it was copied once, and a
    // row made before the store changed kept the old value.
    let out = compile(
        r#"
            store History {
                state lanes = 1
            }
            store Deploys {
                state wide = 1
            }
            component Graph(wide: Number = 1) { Text("{wide}") }
            page P(path: "/") {
                use History
                use Deploys
                Graph(wide: History.lanes)
                Graph(wide: Deploys.wide)
            }
            "#,
    );
    assert!(
        out.contains("get wide() { return History$.lanes; }"),
        "{out}"
    );
    assert!(out.contains("get wide() { return Deploys.wide; }"), "{out}");
}

#[test]
fn a_declared_prop_default_reaches_the_spa() {
    let out = compile(
        r#"
            component Badge(label: String, tone: String = "neutral", dot: Bool = true) { Text(label) }
            page P(path: "/") { Badge(label: "x") }
            
            "#,
    );
    assert!(
        out.contains("_p = WF.props(_p, { tone: \"neutral\", dot: true });"),
        "{out}"
    );
}

#[test]
fn a_custom_property_is_set_by_name_and_follows_state() {
    let out = compile(
        r#"
            page P(path: "/") {
                state tone = "red"
                Card { style { --hover-bg: {tone}; --edge: 1px; &:hover { background: var(--hover-bg) } } }
            }
            
            "#,
    );
    assert!(
        out.contains(".style.setProperty(\"--hover-bg\", _tone()); });"),
        "{out}"
    );
    // The literal custom property is hoisted with the rest of the block.
    assert!(!out.contains("--edge"), "{out}");
    assert!(out.contains(".classList.add(\"wf-s"), "{out}");
}

#[test]
fn handlers_on_a_component_call_attach_to_its_root() {
    let out = compile(
        r#"
            component Save(label: String) { Button(label) }
            page P(path: "/") {
                state n = 0
                Save(label: "Go") { on click { n = n + 1 } }
                Save(label: "Hover") { on mouseenter { n = 9 } }
            }
            
            "#,
    );
    assert!(out.contains("WF.onRoot(_e"), "{out}");
    assert!(
        out.contains(", \"click\", (event) => { _n.set((_n() + 1)); });"),
        "{out}"
    );
    assert!(
        out.contains(", \"mouseenter\", (event) => { _n.set(9); });"),
        "{out}"
    );
    // The action block is not mistaken for slot children.
    assert!(
        !out.contains("Component_Save({ label: \"Go\" }, () =>"),
        "{out}"
    );
}

#[test]
fn every_route_carries_its_pages_title() {
    let out = compile(
        r#"
            page Home(path: "/", title: "Home") { Text("h") }
            page Docs(path: "/docs", title: "Routing \"rules\"") { Text("d") }
            app { Router }
            
            "#,
    );
    assert!(
        out.contains("{ path: \"/\", title: \"Home\", render: (params) => Page_Home(params) },"),
        "{out}"
    );
    assert!(
        out.contains("title: \"Routing \\\"rules\\\"\", render: (params) => Page_Docs(params)"),
        "{out}"
    );
}

#[test]
fn hyphenated_named_arguments_become_attributes_reactive_when_they_read_state() {
    let out = compile(
        r#"
            page P(path: "/") {
                state on = true
                Button("Errors", aria-pressed: on, data-tone: "danger")
                Text(a - b)
            }
            
            "#,
    );
    assert!(out.contains("\"aria-pressed\": () => _on()"), "{out}");
    assert!(out.contains("\"data-tone\": \"danger\""), "{out}");
    // `a - b` is still an expression, not a named argument.
    assert!(out.contains("(_a() - _b())"), "{out}");
}

#[test]
fn a_style_value_that_reads_state_follows_it() {
    let out = compile(
        r##"
            store S { state tone = "#fff" }
            page P(path: "/") {
                use S
                state pct = 40
                Card {
                    style {
                        width: {pct}%
                        background: {S.tone}
                        padding: 1rem
                    }
                }
            }
            
            "##,
    );
    // Element numbering is process-wide, so match on the tail of each line.
    assert!(out.contains(".style.width = `${_pct()}%`; });"), "{out}");
    assert!(out.contains("WF.effect(() => { _e"), "{out}");
    assert!(out.contains(".style.background = S.tone; });"), "{out}");
    // The literal is in the stylesheet, under the element's class.
    assert!(!out.contains(".style.padding"), "{out}");
    assert!(out.contains(".classList.add(\"wf-s"), "{out}");
}

#[test]
fn a_value_that_reads_state_is_reactive() {
    let out = compile(
        r#"
            store S { state pct = 5 }
            page P(path: "/") {
                use S
                state done = 40
                Progress(value: done, max: 100)
                Progress(value: S.pct, max: 100)
                Progress(value: 75, max: 100)
            }
            
            "#,
    );
    assert!(out.contains("value: () => _done()"), "{out}");
    assert!(out.contains("value: () => S.pct"), "{out}");
    assert!(out.contains("value: 75"), "{out}");
}

#[test]
fn a_lambda_parameter_is_a_plain_binding_not_a_signal() {
    let out = compile(
        r#"
            page P(path: "/") {
                state items = []
                state limit = 3
                Text("{items.filter(x => x.done && x.n < limit).length}")
            }
            
            "#,
    );
    assert!(
        out.contains("((x) => (x.done && (x.n < _limit())))"),
        "{out}"
    );
    assert!(!out.contains("_x()"), "{out}");
}

#[test]
fn the_app_wrapper_around_the_router_gets_layout_classes() {
    let out = compile(
        r#"
            page Home(path: "/") { Text("hi") }
            app {
                Row(gap: .lg, align: .center) {
                    Router
                }
            }
            
            "#,
    );
    assert!(out.contains("wf-row wf-gap--lg wf-align--center"), "{out}");
    assert!(!out.contains("--gap-lg"), "{out}");
}

#[test]
fn a_loop_variable_is_a_plain_binding_not_a_signal() {
    let js = compile(
        "page P(path: \"/\") {\n    state items = []\n    for item in items { Text(item.title) }\n}",
    );
    assert!(
        js.contains("item.title"),
        "the loop body should read the binding directly:\n{js}"
    );
    assert!(
        !js.contains("_item()"),
        "the loop variable was emitted as a signal:\n{js}"
    );
}

/// The same has to hold inside an event handler nested in the loop, which is
/// where the pattern actually shows up — a row with a button acting on it.
#[test]
fn a_loop_variable_survives_into_a_nested_event_handler() {
    let js = compile(
        "store S { state rows = []\n action pick(id: Number) { } }\npage P(path: \"/\") {\n    use S\n    for row in S.rows { Button(\"Pick\") { on click { S.pick(row.id) } } }\n}",
    );
    assert!(
        js.contains("S.pick(row.id)"),
        "handler lost the binding:\n{js}"
    );
    assert!(
        !js.contains("_row()"),
        "handler treated the binding as state:\n{js}"
    );
}

/// An index binding is a parameter too.
#[test]
fn a_loop_index_is_a_plain_binding() {
    let js = compile(
        "page P(path: \"/\") {\n    state items = []\n    for item, i in items { Text(\"{i}: {item.name}\") }\n}",
    );
    assert!(
        !js.contains("_i()"),
        "the index was emitted as a signal:\n{js}"
    );
    assert!(
        !js.contains("_item()"),
        "the item was emitted as a signal:\n{js}"
    );
}

/// A nested loop must not erase the outer binding when it finishes.
#[test]
fn nested_loops_each_keep_their_own_binding() {
    let js = compile(
        "page P(path: \"/\") {\n    state groups = []\n    for group in groups {\n        for member in group.members { Text(member.name) }\n        Text(group.title)\n    }\n}",
    );
    assert!(
        !js.contains("_member()"),
        "inner binding became a signal:\n{js}"
    );
    assert!(
        !js.contains("_group()"),
        "the outer binding was lost after the inner loop closed:\n{js}"
    );
    assert!(
        js.contains("group.title"),
        "outer binding missing after nesting:\n{js}"
    );
}

/// State that is genuinely state must still be read as a signal — the fix
/// above must not make everything plain.
#[test]
fn state_outside_a_loop_is_still_a_signal() {
    let js = compile("page P(path: \"/\") {\n    state count = 0\n    Text(\"{count}\")\n}");
    assert!(
        js.contains("_count()"),
        "state stopped being reactive:\n{js}"
    );
}

#[test]
fn a_for_loop_in_an_action_runs_once_in_order() {
    let out = compile(
        "page P(path: \"/\") { state items = [1, 2]\n state n = 0\n action all() { for x in items { n = n + x }\n for x, i in items { log(i) } }\n Text(\"{n}\") }",
    );
    assert!(out.contains("for (const x of _items()) {"), "{out}");
    assert!(
        out.contains("_n.set((_n() + x));"),
        "the item is a plain name: {out}"
    );
    assert!(
        out.contains("for (const [i, x] of Array.from(_items()).entries()) {"),
        "{out}"
    );
}

#[test]
fn optional_chaining_is_javascripts_own() {
    let out = compile(
        "page P(path: \"/\") { state user = null\n derived name = user?.profile?.name ?? \"anon\"\n derived first = user?.tags?.[0]\n derived up = user?.name?.toUpperCase()\n Text(name) }",
    );
    assert!(out.contains("_user()?.profile?.name ?? \"anon\""), "{out}");
    assert!(out.contains("_user()?.tags?.[0]"), "{out}");
    assert!(out.contains("_user()?.name?.toUpperCase()"), "{out}");
}

#[test]
fn push_on_a_state_sets_a_new_list_so_readers_repaint() {
    let out = compile(
        "store Cart { state items: [String] = []\n action add(x: String) { items.push(x) } }\npage P(path: \"/\") { use Cart\n state tags: [String] = []\n state form = { tags: [\"a\"] }\n action go() { tags.push(\"x\")  Cart.items.push(\"y\")  form.tags.push(\"z\") }\n Text(\"{tags.length}\") }",
    );
    assert!(out.contains("_tags.set([..._tags(), \"x\"])"), "{out}");
    assert!(
        out.contains("(Cart.items = [...Cart.items, \"y\"])"),
        "{out}"
    );
    assert!(out.contains("(store.items = [...store.items, x])"), "{out}");
    // A nested list is the writer's own object: pushed in place.
    assert!(out.contains("_form().tags.push(\"z\")"), "{out}");
}

#[test]
fn a_wrapped_control_carries_its_aria_data_and_name_attributes_on_the_input() {
    let out = compile(
        "page P(path: \"/\") { state on = false\n state pick = \"a\"\n Checkbox(bind: on, aria-label: \"Done\", name: \"done\", disabled: true)\n Radio(bind: pick, value: \"a\", label: \"A\", aria-describedby: \"h\")\n Switch(bind: on, label: \"S\", data-kind: \"x\") }",
    );
    assert!(
        out.contains(
            "type: \"checkbox\", \"aria-label\": \"Done\", name: \"done\", disabled: true, checked:"
        ),
        "{out}"
    );
    assert!(
        out.contains("type: \"radio\", \"aria-describedby\": \"h\", name: \"pick\", checked:"),
        "{out}"
    );
    assert!(
        out.contains("role: \"switch\"") && out.contains("\"data-kind\": \"x\" })"),
        "{out}"
    );
}

#[test]
fn a_store_action_keeps_its_own_names_inside_try_for_and_else_if() {
    let out = compile(
        "store S { state user: Map? = null\n state error = \"\"\n state n = 0\n action login(email: String, rows: [Map]) { try { let r = await fetch(\"/x\")\n user = r.user\n for row, i in rows { if row.ok { n = n + i } else if row.bad { log(row) } else { n = 0 } } } catch e { error = e.message } } }\npage P(path: \"/\") { use S\n Text(\"x\") }",
    );
    let i = out.find("login: async").unwrap();
    let action = &out[i..out[i..].find("},\n").map_or(out.len(), |j| i + j)];
    for expected in [
        "let r = (await WF.request(\"/x\"));",
        "store.user = r.user;",
        "for (const [i, row] of Array.from(rows).entries()) {",
        "if (row.ok) {",
        "store.n = (store.n + i);",
        "} else if (row.bad) {",
        "console.log(row);",
        "} catch (e) {",
        "store.error = e.message;",
    ] {
        assert!(action.contains(expected), "{expected} in {action}");
    }
    assert!(
        !action.contains("_user") && !action.contains("WF.signal("),
        "{action}"
    );
}

#[test]
fn a_dialogs_visible_may_be_a_state_a_store_member_or_a_condition() {
    let out = compile(
        "store Ui { state palette = false\n state confirmId: String? = null }\npage P(path: \"/\") { use Ui\n state open = false\n Modal(visible: open, title: \"a\") { Text(\"x\") }\n Modal(visible: Ui.palette, title: \"b\") { Text(\"y\") }\n Dialog(visible: Ui.confirmId != null, title: \"c\") { Text(\"z\") } }",
    );
    assert!(
        out.contains("() => _open(), (v) => _open.set(v));"),
        "{out}"
    );
    assert!(
        out.contains("() => Ui.palette, (v) => { Ui.palette = v; });"),
        "{out}"
    );
    assert!(
        out.contains("() => (Ui.confirmId != null), null);"),
        "{out}"
    );
}

#[test]
fn an_icon_buttons_class_joins_the_engines_and_its_icon_follows_state() {
    let out = compile(
        "page P(path: \"/\") { state dark = false\n IconButton(icon: if dark { \"sun\" } else { \"moon\" }, label: \"Theme\", class: \"site-icon-button\") { on click { dark = !dark } } }",
    );
    assert!(
        out.contains("className: \"wf-icon-btn site-icon-button\""),
        "{out}"
    );
    assert!(
        out.contains("\"data-icon\": () => (_dark() ? \"sun\" : \"moon\")"),
        "{out}"
    );
    assert!(!out.contains("class: \"site-icon-button\""), "{out}");
}

#[test]
fn a_part_with_a_destination_inside_a_loop_is_a_link() {
    let out = compile(
        "page P(path: \"/\") { state names = [\"a\"]\n Sidebar { for n in names by n { Sidebar.Item(to: \"/x/{n}\") { Text(n) } } } }",
    );
    assert!(
        out.contains("WF.el(\"a\", { className: \"wf-sidebar__item\", href: () => `/x/${$n()}`"),
        "{out}"
    );
    assert!(out.contains("WF.navigate(`/x/${$n()}`)"), "{out}");
    assert!(out.contains("WF.activeLink("), "{out}");
    assert!(!out.contains("to: `/x/${"), "{out}");
}

#[test]
fn an_arm_binding_named_like_its_resource_is_the_data_not_the_handle() {
    // The guide's own example. `users` inside the arm read the
    // resource's handle — `_users.length`, a list of nothing — because a
    // resource's name was looked up before the arm's binding.
    let out = compile(
        "page P(path: \"/\") {\n resource users = fetch(\"/api/users\")\n match users {\n  loading { Text(\"…\") }\n  error(e) { Text(e.message) }\n  ready(users) { Text(\"{users.length} users\")  for user in users by user.id { Text(user.name) } }\n }\n}",
    );
    assert!(out.contains("ready: ($$value, $users) => {"), "{out}");
    assert!(out.contains("() => `${$users().length} users`"), "{out}");
    assert!(out.contains("() => $users(),"), "{out}");
    assert!(!out.contains("_users.length"), "{out}");
}

#[test]
fn a_live_binding_never_stands_in_for_a_name_the_compiler_uses() {
    // A component's props are `_p`, a page's root `_root`: a binding
    // spelled `_p` would hide them inside its body.
    let out = compile(
        "type Post { id: String, title: String }\ncomponent List(posts: [Post], heading: String) {\n Text(heading)\n for p in posts by p.id { Text(\"{p.title} in {heading}\") }\n}\npage P(path: \"/\") { List(posts: [], heading: \"All\") }",
    );
    assert!(out.contains("($$item, $$index, $p) => {"), "{out}");
    assert!(out.contains("`${$p().title} in ${_p.heading}`"), "{out}");
}

#[test]
fn an_address_resource_takes_on_and_cache_as_policies() {
    // `on: .focus` was handed over as the word "focus", so a resource
    // over an address never looked again.
    let out = compile(
        "page P(path: \"/\") {\n resource rows = fetch(\"/api/rows\", on: .focus, cache: .swr(60000))\n resource ticks = fetch(\"/api/ticks\", on: [.focus, .interval(30000)])\n Text(\"{rows.state} {ticks.state}\")\n}",
    );
    assert!(
            out.contains("WF.resource(\"/api/rows\", { on: { focus: true }, cache: { kind: \"swr\", ttl: 60000 } })"),
            "{out}"
        );
    assert!(
        out.contains("WF.resource(\"/api/ticks\", { on: { focus: true, interval: 30000 } })"),
        "{out}"
    );
}

#[test]
fn whether_a_resource_is_refreshing_is_read_live() {
    let out = compile(
        "page P(path: \"/\") {\n resource rows = fetch(\"/api/rows\")\n if rows.refreshing { Text(\"Refreshing\") }\n Button(\"Reload\", disabled: rows.pending) { on click { rows.reload() } }\n}",
    );
    assert!(out.contains("() => _rows.refreshing()"), "{out}");
    assert!(out.contains("disabled: () => _rows.pending()"), "{out}");
}

#[test]
fn a_bound_field_of_a_live_item_spelled_like_the_placeholder_is_written() {
    let out = compile(
        "page P(path: \"/\") {\n state rows = [{ id: \"1\", name: \"a\" }]\n for Vendor in rows by Vendor.id { Input(bind: Vendor.name, label: \"Name\") }\n}",
    );
    assert!(
        out.contains("$Vendor().name = WF.bound($$e.target); _rows.set(_rows().slice())"),
        "{out}"
    );
}
