//! Deterministic node identity for the studio (Slice 2 of the engine upgrade).
//!
//! Every renderable [`UIElement`] is assigned a stable **path id** — e.g.
//! `Home:2.0.3` — by a single pre-order walk of the program. The id encodes the
//! owning page/component name plus the statement-index chain down to the node.
//!
//! The map is keyed by the element's **source [`Span`]** (from Slice 1), which is
//! unique and deterministic per source node. Both code generators (JS and SSG)
//! consume this one map via [`NodeMap::id_for`] and stamp `data-wf-node="<id>"`
//! only in studio mode — so the two generators are guaranteed to agree on ids by
//! construction, rather than by two hand-written walkers happening to match.
//!
//! The map doubles as the compile's sidecar: `node_id -> { span, path, component }`.
//!
//! Ids are **structural**: inserting or removing a sibling shifts later ids. That
//! is fine for live selection (re-resolved after each edit) but not stable across
//! edits — see the plan's durability note (decision C4).
//!
//! The walk records *every* `UIElement`, so a few builtins that render no
//! persistent DOM root (`Toast`, which is imperative; the `Children`/`_StyleBlock`
//! pseudo-elements; `Router`/`Route`) still receive ids. Those ids resolve to a
//! real source span but to no stamped DOM node — harmless (they're never the
//! target of a `data-wf-node` click), and it keeps the sidecar a faithful index
//! of the source rather than only of the rendered subset.

use std::collections::HashMap;
use crate::parser::ast::{Program, Declaration, Statement, StatementKind, UIElement, Span};

/// A node's path id, e.g. `"Home:2.0.3"`.
pub type NodeId = String;

/// Sidecar information about one identified node.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// The element's whole-node source span.
    pub span: Span,
    /// The node id (same string used as the map key in [`NodeMap::nodes`]).
    pub path: NodeId,
    /// The owning page/component name (`"App"` for the app shell).
    pub component: String,
}

/// The result of the node-identity pre-pass: a bidirectional map between source
/// spans and node ids, plus the sidecar info for each id.
#[derive(Debug, Clone, Default)]
pub struct NodeMap {
    /// node_id -> info. This is the sidecar the studio consumes.
    nodes: HashMap<NodeId, NodeInfo>,
    /// element span -> node_id. The codegen lookup path.
    by_span: HashMap<Span, NodeId>,
}

impl NodeMap {
    /// The id assigned to the element at `span`, if any.
    pub fn id_for(&self, span: Span) -> Option<&str> {
        self.by_span.get(&span).map(String::as_str)
    }

    /// Sidecar info for a node id.
    pub fn info(&self, id: &str) -> Option<&NodeInfo> {
        self.nodes.get(id)
    }

    /// Number of identified nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Iterate `(node_id, info)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &NodeInfo)> {
        self.nodes.iter()
    }

    fn record(&mut self, span: Span, id: NodeId, component: &str) {
        self.by_span.insert(span, id.clone());
        self.nodes.insert(
            id.clone(),
            NodeInfo { span, path: id, component: component.to_string() },
        );
    }
}

/// Build the node-identity map for a whole program in one deterministic pre-order walk.
pub fn build_node_map(program: &Program) -> NodeMap {
    let mut map = NodeMap::default();

    // A Page and a Component may legally share a name (they live in separate JS
    // namespaces, `Page_X` vs `Component_X`), which would otherwise collapse their
    // ids onto the same owner segment. Disambiguate deterministically by
    // declaration kind — never by program order, which is non-deterministic
    // (build.rs merges files in filesystem order). Same-kind duplicate names are
    // already invalid (they collide in codegen too) and are left to collide.
    let mut name_counts: HashMap<&str, u32> = HashMap::new();
    for decl in &program.declarations {
        if let Some(name) = decl_owner_name(decl) {
            *name_counts.entry(name).or_default() += 1;
        }
    }

    for decl in &program.declarations {
        let (body, name, kind) = match decl {
            Declaration::Page(p) => (&p.body, p.name.as_str(), "page"),
            Declaration::Component(c) => (&c.body, c.name.as_str(), "component"),
            Declaration::App(a) => (&a.body, "App", "app"),
            // Stores hold no UI.
            Declaration::Store(_) => continue,
        };
        let owner = if name_counts.get(name).copied().unwrap_or(0) > 1 {
            format!("{}#{}", name, kind)
        } else {
            name.to_string()
        };
        walk_body(body, &owner, None, &mut map);
    }
    map
}

/// The owner name a declaration contributes to the id namespace (`None` for stores).
fn decl_owner_name(decl: &Declaration) -> Option<&str> {
    match decl {
        Declaration::Page(p) => Some(&p.name),
        Declaration::Component(c) => Some(&c.name),
        Declaration::App(_) => Some("App"),
        Declaration::Store(_) => None,
    }
}

/// Walk a statement list, numbering each statement by its position. The first
/// level of a declaration uses `Comp:i`; deeper levels append `.i`.
fn walk_body(stmts: &[Statement], component: &str, prefix: Option<&str>, map: &mut NodeMap) {
    for (i, stmt) in stmts.iter().enumerate() {
        let seg = match prefix {
            None => format!("{}:{}", component, i),
            Some(p) => format!("{}.{}", p, i),
        };
        walk_stmt(stmt, component, &seg, map);
    }
}

/// Record a statement's UIElement (if it is one) and recurse into any child bodies.
///
/// Single-body control flow (`for`/`show`) indexes its body directly under `seg`.
/// Multi-body statements (`if`/`fetch`) insert a short branch discriminator so a
/// then-branch child cannot collide with an else-branch child.
fn walk_stmt(stmt: &Statement, component: &str, seg: &str, map: &mut NodeMap) {
    match &stmt.kind {
        StatementKind::UIElement(ui) => {
            record_element(ui, component, seg, map);
        }
        StatementKind::For(f) => walk_body(&f.body, component, Some(seg), map),
        StatementKind::Show(s) => walk_body(&s.body, component, Some(seg), map),
        StatementKind::If(if_stmt) => {
            walk_body(&if_stmt.then_body, component, Some(&format!("{}.t", seg)), map);
            for (k, (_, body)) in if_stmt.else_if_branches.iter().enumerate() {
                walk_body(body, component, Some(&format!("{}.ei{}", seg, k)), map);
            }
            if let Some(else_body) = &if_stmt.else_body {
                walk_body(else_body, component, Some(&format!("{}.e", seg)), map);
            }
        }
        StatementKind::Fetch(f) => {
            if let Some(b) = &f.loading_block {
                walk_body(b, component, Some(&format!("{}.l", seg)), map);
            }
            if let Some((_, b)) = &f.error_block {
                walk_body(b, component, Some(&format!("{}.err", seg)), map);
            }
            if let Some(b) = &f.success_block {
                walk_body(b, component, Some(&format!("{}.s", seg)), map);
            }
        }
        // State/Derived/Effect/Action/Use/EventHandler/Navigate/Log/Animate/… render no DOM.
        _ => {}
    }
}

/// Record one UIElement's id, then recurse into its children under the same seg.
fn record_element(ui: &UIElement, component: &str, seg: &str, map: &mut NodeMap) {
    map.record(ui.span, seg.to_string(), component);
    walk_body(&ui.children, component, Some(seg), map);
}

#[cfg(test)]
mod tests {
    //! Slice 2 acceptance: ids are deterministic and resolve to the right spans;
    //! studio compiles stamp `data-wf-node`; export compiles do not; and the JS
    //! and SSG generators agree on the id for any node they both stamp.
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::parser::ast::PageDecl;
    use crate::codegen::JsCodegen;
    use crate::codegen::ssg::{render_page_html, render_page_html_studio};
    use crate::config::ProjectConfig;

    fn program(src: &str) -> Program {
        let toks = Lexer::new(src, "<test>").tokenize().expect("lex");
        Parser::new(toks, "<test>").parse().expect("parse")
    }

    fn config() -> ProjectConfig {
        serde_json::from_str(r#"{"name":"test"}"#).expect("config")
    }

    fn first_page(p: &Program) -> &PageDecl {
        p.declarations.iter().find_map(|d| match d {
            Declaration::Page(pg) => Some(pg),
            _ => None,
        }).expect("a page")
    }

    const SRC: &str = "Page Home (path: \"/\") {\n\
                       \x20 Container {\n\
                       \x20   Heading(\"Hi\", h1)\n\
                       \x20   Button(\"Go\", primary)\n\
                       \x20 }\n\
                       }\n";

    #[test]
    fn ids_resolve_to_spans_and_paths() {
        let p = program(SRC);
        let map = build_node_map(&p);
        assert_eq!(map.len(), 3); // Container, Heading, Button

        let container = map.info("Home:0").expect("Home:0");
        assert!(container.span.slice(SRC).starts_with("Container"));
        assert_eq!(container.component, "Home");

        let heading = map.info("Home:0.0").expect("Home:0.0");
        assert_eq!(heading.span.slice(SRC), "Heading(\"Hi\", h1)");

        let button = map.info("Home:0.1").expect("Home:0.1");
        assert_eq!(button.span.slice(SRC), "Button(\"Go\", primary)");

        // Reverse lookup by span matches.
        assert_eq!(map.id_for(heading.span), Some("Home:0.0"));
    }

    #[test]
    fn map_is_deterministic() {
        let p = program(SRC);
        let a = build_node_map(&p);
        let b = build_node_map(&p);
        let mut va: Vec<_> = a.iter().map(|(id, i)| (id.clone(), i.span.start, i.span.end)).collect();
        let mut vb: Vec<_> = b.iter().map(|(id, i)| (id.clone(), i.span.start, i.span.end)).collect();
        va.sort();
        vb.sort();
        assert_eq!(va, vb);
    }

    #[test]
    fn control_flow_uses_branch_tags() {
        let src = "Page P (path: \"/\") {\n\
                   \x20 if (x) {\n\
                   \x20   Text(\"then\")\n\
                   \x20 } else {\n\
                   \x20   Text(\"else\")\n\
                   \x20 }\n\
                   \x20 for item in items {\n\
                   \x20   Text(\"row\")\n\
                   \x20 }\n\
                   }\n";
        let p = program(src);
        let map = build_node_map(&p);
        // if is statement 0: then -> P:0.t.0, else -> P:0.e.0 (no collision).
        assert_eq!(map.info("P:0.t.0").map(|i| i.span.slice(src)), Some("Text(\"then\")"));
        assert_eq!(map.info("P:0.e.0").map(|i| i.span.slice(src)), Some("Text(\"else\")"));
        // for is statement 1: body -> P:1.0
        assert_eq!(map.info("P:1.0").map(|i| i.span.slice(src)), Some("Text(\"row\")"));
    }

    #[test]
    fn js_studio_stamps_export_does_not() {
        let p = program(SRC);
        let map = build_node_map(&p);

        let mut jsgen = JsCodegen::new();
        jsgen.set_studio(map.clone());
        let studio_js = jsgen.generate(&p);
        assert!(studio_js.contains("\"data-wf-node\": \"Home:0.0\""));
        assert!(studio_js.contains("\"data-wf-node\": \"Home:0.1\""));

        let export_js = JsCodegen::new().generate(&p);
        assert!(!export_js.contains("data-wf-node"));
    }

    #[test]
    fn ssg_studio_stamps_export_does_not() {
        let p = program(SRC);
        let map = build_node_map(&p);
        let page = first_page(&p);
        let cfg = config();

        let studio_html = render_page_html_studio(page, &cfg, None, &Default::default(), true, &map);
        assert!(studio_html.contains("data-wf-node=\"Home:0.0\""));
        assert!(studio_html.contains("data-wf-node=\"Home:0.1\""));

        let export_html = render_page_html(page, &cfg, None, &Default::default());
        assert!(!export_html.contains("data-wf-node"));
    }

    #[test]
    fn special_component_roots_are_stamped() {
        // These emitters build their own root and ignore the attrs choke point,
        // so each must be stamped explicitly. Regression guard for that gap.
        let src = "Page S (path: \"/\") {\n\
                   \x20 state on = false\n\
                   \x20 Switch(bind: on, label: \"T\")\n\
                   \x20 Checkbox(bind: on, label: \"A\")\n\
                   \x20 Modal(title: \"H\", visible: on) { Text(\"b\") }\n\
                   \x20 Dropdown(label: \"M\") { Text(\"i\") }\n\
                   \x20 Tabs { TabPage(\"One\") { Text(\"a\") } }\n\
                   }\n";
        let p = program(src);
        let map = build_node_map(&p);
        let mut jsgen = JsCodegen::new();
        jsgen.set_studio(map.clone());
        let js = jsgen.generate(&p);

        for comp in ["Switch", "Checkbox", "Modal", "Dropdown", "Tabs"] {
            let id = map.iter()
                .find(|(_, info)| info.span.slice(src).starts_with(comp))
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| panic!("no map entry for {}", comp));
            assert!(
                js.contains(&format!("\"data-wf-node\": \"{}\"", id)),
                "{} root (id {}) not stamped", comp, id
            );
        }
    }

    #[test]
    fn js_and_ssg_agree_on_ids() {
        let p = program(SRC);
        let map = build_node_map(&p);
        let page = first_page(&p);

        let mut jsgen = JsCodegen::new();
        jsgen.set_studio(map.clone());
        let js = jsgen.generate(&p);
        let html = render_page_html_studio(page, &config(), None, &Default::default(), true, &map);

        // The Heading's id is produced by the one shared map, so both stamp the same string.
        for id in ["Home:0", "Home:0.0", "Home:0.1"] {
            assert!(js.contains(&format!("\"data-wf-node\": \"{}\"", id)), "js missing {}", id);
            assert!(html.contains(&format!("data-wf-node=\"{}\"", id)), "html missing {}", id);
        }
    }

    #[test]
    fn duplicate_names_across_kinds_get_distinct_ids() {
        // A Page and a Component may legally share a name; their ids must not collapse.
        let src = "Component Profile (name: String) { Text(name, bold) }\n\
                   Page Profile (path: \"/me\") { Container { Heading(\"Me\", h1) } }\n";
        let p = program(src);
        let map = build_node_map(&p);
        assert!(map.info("Profile#component:0").is_some(), "component node id");
        assert!(map.info("Profile#page:0").is_some(), "page node id");
        assert!(map.info("Profile#page:0.0").is_some(), "page child id");
        // All ids distinct (no collision-collapse).
        let ids: std::collections::HashSet<_> = map.iter().map(|(id, _)| id.clone()).collect();
        assert_eq!(ids.len(), map.len());
    }

    #[test]
    fn app_shell_router_wrapper_is_stamped() {
        // Regression: the layout element wrapping Router is a real DOM root and
        // was previously left unstamped in both generators.
        let src = "Page Home (path: \"/\") { Text(\"hi\") }\n\
                   App {\n\
                   \x20 Row {\n\
                   \x20   Container { Text(\"nav\") }\n\
                   \x20   Router { Route(path: \"/\", page: Home) }\n\
                   \x20 }\n\
                   }\n";
        let p = program(src);
        let map = build_node_map(&p);
        let row_id = map.iter()
            .find(|(_, i)| i.span.slice(src).starts_with("Row"))
            .map(|(id, _)| id.clone())
            .expect("Row wrapper id");

        let mut jsgen = JsCodegen::new();
        jsgen.set_studio(map.clone());
        let js = jsgen.generate(&p);
        assert!(js.contains(&format!("\"data-wf-node\": \"{}\"", row_id)), "JS app wrapper not stamped");

        let page = first_page(&p);
        let app_body: Vec<Statement> = p.declarations.iter()
            .find_map(|d| if let Declaration::App(a) = d { Some(a.body.clone()) } else { None })
            .expect("app body");
        let html = render_page_html_studio(page, &config(), Some(&app_body), &Default::default(), true, &map);
        assert!(html.contains(&format!("data-wf-node=\"{}\"", row_id)), "SSG app wrapper not stamped");
    }

    #[test]
    fn compound_widget_children_are_stamped_in_js() {
        // Regression: special emitters hand-build sub-element DOM; each of these
        // user-written source elements must still carry its node id in JS.
        let src = "Page P (path: \"/\") {\n\
                   \x20 state on = false\n\
                   \x20 Sidebar { Sidebar.Header { Text(\"B\") } Sidebar.Item(to: \"/a\") { Text(\"A\") } }\n\
                   \x20 Tabs { TabPage(\"One\") { Text(\"a\") } }\n\
                   \x20 Breadcrumb { Breadcrumb.Item(to: \"/x\") { Text(\"X\") } }\n\
                   \x20 Carousel { Carousel.Slide { Text(\"s\") } }\n\
                   \x20 Modal(title: \"H\", visible: on) { Text(\"b\") Modal.Footer { Button(\"OK\") } }\n\
                   }\n";
        let p = program(src);
        let map = build_node_map(&p);
        let mut jsgen = JsCodegen::new();
        jsgen.set_studio(map.clone());
        let js = jsgen.generate(&p);

        for needle in ["Sidebar.Header", "Sidebar.Item", "TabPage", "Breadcrumb.Item", "Carousel.Slide", "Modal.Footer"] {
            let id = map.iter()
                .find(|(_, i)| i.span.slice(src).starts_with(needle))
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| panic!("no map entry for {}", needle));
            assert!(js.contains(&format!("\"data-wf-node\": \"{}\"", id)), "{} not stamped in JS (id {})", needle, id);
        }
    }
}
