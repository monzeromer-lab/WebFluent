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

use crate::parser::ast::{
    Arg, ArmPattern, Declaration, Expr, MatchArm, Program, Span, Statement, StatementKind,
    StringPart, UIElement,
};
use serde::Serialize;
use std::collections::HashMap;

/// A node's path id, e.g. `"Home:2.0.3"`.
pub type NodeId = String;

/// Sidecar information about one identified node.
#[derive(Debug, Clone, Serialize)]
pub struct NodeInfo {
    /// The element's whole-node source span.
    #[serde(skip)]
    pub span: Span,
    /// The node id (the same string the [`NodeMap`] is keyed by).
    pub path: NodeId,
    /// The owning page/component name (`"App"` for the app shell).
    pub component: String,
    /// The file it is written in, when the map was built knowing.
    pub file: Option<String>,
    /// Where it is written in that file: its first byte, one past its
    /// last, and the line it starts on.
    pub start: u32,
    pub end: u32,
    pub line: u32,
    /// What it calls: `Button`, `Card.Header`, `DeployRow`.
    pub element: String,
    /// The element it is directly inside, when there is one.
    pub parent: Option<NodeId>,
    /// The loops and branches it is inside, outermost first.
    pub within: Vec<Within>,
    /// Its arguments whose value is not written out — they read state, a
    /// prop, a loop's item, a store — and what each reads.
    pub bound: Vec<Bound>,
}

impl NodeInfo {
    /// Inside a `for`: one node, drawn once per item.
    pub fn template(&self) -> bool {
        self.within.iter().any(|w| w.kind == "for")
    }

    /// Inside a branch that may not be the one drawn: an `if`'s, a
    /// `show`'s, a `match`'s arm.
    pub fn conditional(&self) -> bool {
        self.within.iter().any(|w| w.kind != "for")
    }

    /// What kind of selection it is (BUILDER §2): `template` inside a
    /// `for`, else `conditional` in a branch, else `bound` when an argument
    /// reads something, else `plain`.
    pub fn kind(&self) -> &'static str {
        if self.template() {
            "template"
        } else if self.conditional() {
            "conditional"
        } else if !self.bound.is_empty() {
            "bound"
        } else {
            "plain"
        }
    }
}

/// A loop or branch a node is inside.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Within {
    /// `for`, `if`, `else if`, `else`, `show`, `match`.
    pub kind: &'static str,
    /// Its head as written: `for d in live by d.id`, `if open`, `.ready(v)`.
    pub head: String,
    pub start: u32,
    pub line: u32,
}

/// An argument whose value is read, not written.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Bound {
    /// The prop's name, or `text` for the positional value.
    pub arg: String,
    pub reads: Vec<Read>,
}

/// A name an argument reads, and what it is.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Read {
    pub name: String,
    /// `state`, `derived`, `prop`, `param` (a route's), `loop` (an item, an
    /// arm's binding, a fill's value), `store`, `const`, `data`, or
    /// `unknown`.
    pub from: &'static str,
}

/// The result of the node-identity pre-pass: a bidirectional map between source
/// spans and node ids, plus the sidecar info for each id.
#[derive(Debug, Clone, Default)]
pub struct NodeMap {
    /// node_id -> info. This is the sidecar the studio consumes.
    nodes: HashMap<NodeId, NodeInfo>,
    /// element span -> every node_id written there. Spans are a file's, so
    /// two files may share one; the owner then tells them apart.
    by_span: HashMap<Span, Vec<NodeId>>,
}

impl NodeMap {
    /// The id assigned to the element at `span`, if exactly one is.
    pub fn id_for(&self, span: Span) -> Option<&str> {
        match self.by_span.get(&span).map(Vec::as_slice) {
            Some([only]) => Some(only.as_str()),
            _ => None,
        }
    }

    /// The id of the element at `span` written in `owner` (a page's, a
    /// component's name, or `App`): what a code generator asks, since it
    /// knows what it is emitting.
    pub fn id_in(&self, owner: &str, span: Span) -> Option<&str> {
        let ids = self.by_span.get(&span)?;
        if let [only] = ids.as_slice() {
            return Some(only);
        }
        let mut found = ids
            .iter()
            .filter(|id| id.split([':', '#']).next() == Some(owner));
        match (found.next(), found.next()) {
            (Some(id), None) => Some(id),
            _ => None,
        }
    }

    /// The id of the element at `span` written in one of `owners`: what
    /// the static paint asks, since a component it expands holds its
    /// caller's fills, and the page sits inside the app's shell.
    pub fn id_among(&self, owners: &[String], span: Span) -> Option<&str> {
        let ids = self.by_span.get(&span)?;
        if let [only] = ids.as_slice() {
            return Some(only);
        }
        let mut found = ids.iter().filter(|id| {
            let owner = id.split([':', '#']).next().unwrap_or("");
            owners.iter().any(|o| o == owner)
        });
        match (found.next(), found.next()) {
            (Some(id), None) => Some(id),
            _ => None,
        }
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

    /// The node the caret at `offset` of `file` is in: the innermost
    /// element whose source holds it. An element the parser makes itself
    /// (a slot's `children`) holds nothing.
    pub fn at(&self, file: &str, offset: usize) -> Option<&NodeInfo> {
        self.nodes
            .values()
            .filter(|n| n.file.as_deref().is_none_or(|f| f == file))
            .filter(|n| n.start < n.end && (n.start as usize) <= offset && offset < n.end as usize)
            .min_by_key(|n| (n.end - n.start, n.start))
    }

    /// The node written exactly at `start..end` of `file`.
    pub fn exactly(&self, file: &str, start: usize, end: usize) -> Option<&NodeInfo> {
        self.nodes.values().find(|n| {
            n.file.as_deref().is_none_or(|f| f == file)
                && n.start as usize == start
                && n.end as usize == end
        })
    }

    /// The nodes directly inside `id`, in source order.
    pub fn children(&self, id: &str) -> Vec<&NodeInfo> {
        let mut out: Vec<&NodeInfo> = self
            .nodes
            .values()
            .filter(|n| n.parent.as_deref() == Some(id))
            .collect();
        out.sort_by_key(|n| (n.start, n.path.clone()));
        out
    }

    /// Every node, in id order, as a tool reads them: `wf build --studio`
    /// writes this as `nodes.json`.
    pub fn to_json(&self) -> serde_json::Value {
        let mut nodes: Vec<&NodeInfo> = self.nodes.values().collect();
        nodes.sort_by(|a, b| (&a.file, a.start, &a.path).cmp(&(&b.file, b.start, &b.path)));
        serde_json::json!({ "nodes": nodes })
    }

    fn record(&mut self, info: NodeInfo) {
        self.by_span
            .entry(info.span)
            .or_default()
            .push(info.path.clone());
        self.nodes.insert(info.path.clone(), info);
    }
}

/// Build the node-identity map for a whole program in one deterministic pre-order walk.
pub fn build_node_map(program: &Program) -> NodeMap {
    build_node_map_with(program, &|_| None)
}

/// [`build_node_map`], each node knowing the file it is written in:
/// `file_of` is the file of the program's `i`th declaration.
pub fn build_node_map_with(
    program: &Program,
    file_of: &dyn Fn(usize) -> Option<String>,
) -> NodeMap {
    let mut map = NodeMap::default();
    walk_program(program, file_of, &mut |ui, id, component, ctx| {
        map.record(NodeInfo {
            span: ui.span,
            path: id.to_string(),
            component: component.to_string(),
            file: ctx.file.clone(),
            start: ui.span.start,
            end: ui.span.end,
            line: ui.span.line,
            element: crate::print::component(&ui.component),
            parent: ctx.parent.clone(),
            within: ctx.within.clone(),
            bound: bound_args(ui, ctx),
        });
    });
    map
}

/// Visit every renderable `UIElement` in the deterministic pre-order that node ids
/// are built from, invoking `visit(element, node_id, component)`.
///
/// This is the single source of truth for the traversal: [`build_node_map`], the
/// codegens (indirectly, via the map), and the edit engine's resolver all go
/// through it, so their ids can never drift apart.
pub fn visit_nodes<'a>(program: &'a Program, visit: &mut dyn FnMut(&'a UIElement, &str, &str)) {
    walk_program(program, &|_| None, &mut |ui, id, component, _| {
        visit(ui, id, component)
    });
}

/// What the walk knows where a node is.
#[derive(Clone, Default)]
pub struct Ctx {
    file: Option<String>,
    parent: Option<NodeId>,
    within: Vec<Within>,
    /// The names in scope that a loop, an arm or a fill brought, and the
    /// declaration's own.
    scope: HashMap<String, &'static str>,
}

fn walk_program<'a>(
    program: &'a Program,
    file_of: &dyn Fn(usize) -> Option<String>,
    visit: &mut dyn FnMut(&'a UIElement, &str, &str, &Ctx),
) {
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
    // What every declaration may read: the program's constants, data and
    // stores.
    let mut global: HashMap<String, &'static str> = HashMap::new();
    for decl in &program.declarations {
        match decl {
            Declaration::Const(c) => {
                global.insert(c.name.clone(), "const");
            }
            Declaration::Data(d) => {
                global.insert(d.name.clone(), "data");
            }
            Declaration::Store(s) => {
                global.insert(s.name.clone(), "store");
            }
            _ => {}
        }
    }

    for (index, decl) in program.declarations.iter().enumerate() {
        let (body, name, kind) = match decl {
            Declaration::Page(p) => (&p.body, p.name.as_str(), "page"),
            Declaration::Component(c) => (&c.body, c.name.as_str(), "component"),
            Declaration::App(a) => (&a.body, "App", "app"),
            // Stores and themes hold no UI.
            Declaration::Store(_)
            | Declaration::Theme(_)
            | Declaration::Type(_)
            | Declaration::Enum(_)
            | Declaration::Api(_)
            | Declaration::Script(_)
            | Declaration::Const(_)
            | Declaration::Animation(_)
            | Declaration::Test(_)
            | Declaration::Data(_) => continue,
        };
        let owner = if name_counts.get(name).copied().unwrap_or(0) > 1 {
            format!("{}#{}", name, kind)
        } else {
            name.to_string()
        };
        let mut scope = global.clone();
        match decl {
            Declaration::Page(p) => {
                for q in &p.params {
                    scope.insert(q.name.clone(), "param");
                }
            }
            Declaration::Component(c) => {
                for q in &c.props {
                    scope.insert(q.name.clone(), "prop");
                }
            }
            _ => {}
        }
        for stmt in body {
            match &stmt.kind {
                StatementKind::State(s) => {
                    scope.insert(s.name.clone(), "state");
                }
                StatementKind::Derived(d) => {
                    scope.insert(d.name.clone(), "derived");
                }
                StatementKind::Resource(r) => {
                    scope.insert(r.name.clone(), "state");
                }
                StatementKind::Connection(c) => {
                    scope.insert(c.name.clone(), "state");
                }
                _ => {}
            }
        }
        let ctx = Ctx {
            file: file_of(index),
            parent: None,
            within: Vec::new(),
            scope,
        };
        walk_body(body, &owner, None, &ctx, visit);
    }
}

/// The owner name a declaration contributes to the id namespace (`None` for stores).
fn decl_owner_name(decl: &Declaration) -> Option<&str> {
    match decl {
        Declaration::Page(p) => Some(&p.name),
        Declaration::Component(c) => Some(&c.name),
        Declaration::App(_) => Some("App"),
        Declaration::Store(_)
        | Declaration::Theme(_)
        | Declaration::Type(_)
        | Declaration::Enum(_)
        | Declaration::Api(_)
        | Declaration::Script(_)
        | Declaration::Const(_)
        | Declaration::Animation(_)
        | Declaration::Test(_)
        | Declaration::Data(_) => None,
    }
}

/// Walk a statement list, numbering each statement by its position. The first
/// level of a declaration uses `Comp:i`; deeper levels append `.i`.
fn walk_body<'a>(
    stmts: &'a [Statement],
    component: &str,
    prefix: Option<&str>,
    ctx: &Ctx,
    visit: &mut dyn FnMut(&'a UIElement, &str, &str, &Ctx),
) {
    for (i, stmt) in stmts.iter().enumerate() {
        let seg = match prefix {
            None => format!("{}:{}", component, i),
            Some(p) => format!("{}.{}", p, i),
        };
        walk_stmt(stmt, component, &seg, ctx, visit);
    }
}

fn within(ctx: &Ctx, kind: &'static str, head: String, stmt: &Statement, names: &[&str]) -> Ctx {
    let mut inner = ctx.clone();
    inner.within.push(Within {
        kind,
        head,
        start: stmt.span.start,
        line: stmt.span.line,
    });
    for n in names {
        inner.scope.insert((*n).to_string(), "loop");
    }
    inner
}

fn arm_head(arm: &MatchArm) -> String {
    let head = match &arm.pattern {
        ArmPattern::Loading => "loading".to_string(),
        ArmPattern::Error => "error".to_string(),
        ArmPattern::Ready => "ready".to_string(),
        ArmPattern::Else => "else".to_string(),
        ArmPattern::State(s) => s.clone(),
        ArmPattern::Case(c) => format!(".{c}"),
    };
    match (&arm.binding, arm.bindings.is_empty()) {
        (Some(b), _) => format!("{head}({b})"),
        (None, false) => format!("{head}({})", arm.bindings.join(", ")),
        (None, true) => head,
    }
}

/// Visit a statement's UIElement (if it is one) and recurse into any child bodies.
///
/// Single-body control flow (`for`/`show`) indexes its body directly under `seg`.
/// Multi-body statements (`if`/`fetch`) insert a short branch discriminator so a
/// then-branch child cannot collide with an else-branch child.
fn walk_stmt<'a>(
    stmt: &'a Statement,
    component: &str,
    seg: &str,
    ctx: &Ctx,
    visit: &mut dyn FnMut(&'a UIElement, &str, &str, &Ctx),
) {
    match &stmt.kind {
        StatementKind::UIElement(ui) => {
            visit(ui, seg, component, ctx);
            let mut inner = ctx.clone();
            inner.parent = Some(seg.to_string());
            walk_body(&ui.children, component, Some(seg), &inner, visit);
            for fill in &ui.slot_fills {
                let mut fill_ctx = inner.clone();
                for p in &fill.params {
                    fill_ctx.scope.insert(p.clone(), "loop");
                }
                walk_body(
                    &fill.body,
                    component,
                    Some(&format!("{}.{}", seg, fill.name)),
                    &fill_ctx,
                    visit,
                );
            }
        }
        StatementKind::For(f) => {
            let mut names = vec![f.item.as_str()];
            names.extend(f.index.as_deref());
            let key = f
                .key
                .as_ref()
                .map(|k| format!(" by {}", crate::print::expr(k)))
                .unwrap_or_default();
            let head = format!(
                "for {} in {}{key}",
                names.join(", "),
                crate::print::expr(&f.iterable)
            );
            let inner = within(ctx, "for", head, stmt, &names);
            walk_body(&f.body, component, Some(seg), &inner, visit)
        }
        StatementKind::Show(s) => {
            let inner = within(
                ctx,
                "show",
                format!("show {}", crate::print::expr(&s.condition)),
                stmt,
                &[],
            );
            walk_body(&s.body, component, Some(seg), &inner, visit)
        }
        StatementKind::If(if_stmt) => {
            let head = match &if_stmt.binding {
                Some(b) => format!("if let {b} = {}", crate::print::expr(&if_stmt.condition)),
                None => format!("if {}", crate::print::expr(&if_stmt.condition)),
            };
            let bound: Vec<&str> = if_stmt.binding.as_deref().into_iter().collect();
            walk_body(
                &if_stmt.then_body,
                component,
                Some(&format!("{}.t", seg)),
                &within(ctx, "if", head, stmt, &bound),
                visit,
            );
            for (k, (cond, body)) in if_stmt.else_if_branches.iter().enumerate() {
                let head = format!("else if {}", crate::print::expr(cond));
                walk_body(
                    body,
                    component,
                    Some(&format!("{}.ei{}", seg, k)),
                    &within(ctx, "else if", head, stmt, &[]),
                    visit,
                );
            }
            if let Some(else_body) = &if_stmt.else_body {
                walk_body(
                    else_body,
                    component,
                    Some(&format!("{}.e", seg)),
                    &within(ctx, "else", "else".into(), stmt, &[]),
                    visit,
                );
            }
        }
        StatementKind::Fetch(f) => {
            let inner = within(ctx, "match", format!("fetch {}", f.variable), stmt, &[]);
            if let Some(b) = &f.loading_block {
                walk_body(b, component, Some(&format!("{}.l", seg)), &inner, visit);
            }
            if let Some((name, b)) = &f.error_block {
                let mut e = inner.clone();
                e.scope.insert(name.clone(), "loop");
                walk_body(b, component, Some(&format!("{}.err", seg)), &e, visit);
            }
            if let Some(b) = &f.success_block {
                walk_body(b, component, Some(&format!("{}.s", seg)), &inner, visit);
            }
        }
        StatementKind::Match(m) => {
            for (k, arm) in m.arms.iter().enumerate() {
                let mut names: Vec<&str> = arm.binding.as_deref().into_iter().collect();
                names.extend(arm.bindings.iter().map(String::as_str));
                let head = format!(
                    "match {} {}",
                    crate::print::expr(&m.scrutinee),
                    arm_head(arm)
                );
                walk_body(
                    &arm.body,
                    component,
                    Some(&format!("{}.m{}", seg, k)),
                    &within(ctx, "match", head, stmt, &names),
                    visit,
                );
            }
        }
        // State/Derived/Effect/Action/Use/EventHandler/Navigate/Log/Animate/… render no DOM.
        _ => {}
    }
}

/// The element's arguments that read something, and what each reads.
fn bound_args(ui: &UIElement, ctx: &Ctx) -> Vec<Bound> {
    let mut out = Vec::new();
    for arg in &ui.args {
        let (name, value) = match arg {
            Arg::Positional(e) => ("text".to_string(), e),
            Arg::Named(k, e) => (k.clone(), e),
        };
        let mut names = Vec::new();
        reads(value, &mut Vec::new(), &mut names);
        names.dedup();
        if names.is_empty() {
            continue;
        }
        out.push(Bound {
            arg: name,
            reads: names
                .into_iter()
                .map(|n| Read {
                    from: ctx.scope.get(&n).copied().unwrap_or("unknown"),
                    name: n,
                })
                .collect(),
        });
    }
    out
}

/// The names an expression reads from its scope, in order: the root of
/// each `a.b.c`, a lambda's own parameters left out.
pub(crate) fn reads(e: &Expr, bound: &mut Vec<String>, out: &mut Vec<String>) {
    match e {
        Expr::Identifier(n) => {
            if !bound.contains(n) && !out.contains(n) {
                out.push(n.clone());
            }
        }
        Expr::Lambda(params, body) => {
            let added: Vec<String> = params
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
            let len = bound.len();
            bound.extend(added);
            reads(body, bound, out);
            bound.truncate(len);
        }
        // `format(x, .date)` and the like: a function is not something read.
        Expr::FunctionCall(_, args) => {
            for a in args {
                reads(a, bound, out);
            }
        }
        Expr::InterpolatedString(parts) => {
            for p in parts {
                if let StringPart::Expression(x) = p {
                    reads(x, bound, out);
                }
            }
        }
        other => {
            for child in other.children() {
                reads(child, bound, out);
            }
        }
    }
}

// ─── Layers ──────────────────────────────────────────────

/// One row of the Layers panel: an element, a loop or a branch, or a
/// statement that is source rather than something drawn.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Layer {
    /// `page`, `component`, `app`; `element`, `slot`; `if`, `else if`,
    /// `else`, `for`, `show`, `match`, `arm`; or `source`.
    pub kind: &'static str,
    /// What it shows: the element's name, the head of a loop or branch,
    /// `state stage`, `use Cart`, `action save`, `on click`.
    pub label: String,
    /// For `source`: what it is — `state`, `derived`, `use`, `action`,
    /// `effect`, `timer`, `resource`, `connection`, `validate`, `handler`,
    /// `statement`.
    pub source: Option<&'static str>,
    /// An element's node id, which the canvas and the caret share.
    pub id: Option<NodeId>,
    pub file: Option<String>,
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub children: Vec<Layer>,
}

/// The Layers panel's tree: each page, component and the app, and inside
/// it every element, loop, branch and source statement in source order.
pub fn layers(program: &Program, file_of: &dyn Fn(usize) -> Option<String>) -> Vec<Layer> {
    let map = build_node_map_with(program, file_of);
    let mut out = Vec::new();
    for (index, decl) in program.declarations.iter().enumerate() {
        let (kind, name, body, span) = match decl {
            Declaration::Page(p) => ("page", p.name.clone(), &p.body, p.span),
            Declaration::Component(c) => ("component", c.name.clone(), &c.body, c.span),
            Declaration::App(a) => ("app", "App".to_string(), &a.body, Span::dummy()),
            _ => continue,
        };
        let file = file_of(index);
        let owners = vec![name.clone()];
        out.push(Layer {
            kind,
            label: name,
            source: None,
            id: None,
            file: file.clone(),
            start: span.start,
            end: span.end,
            line: span.line,
            children: layer_body(body, &map, &owners, &file),
        });
    }
    out
}

fn layer_body(
    stmts: &[Statement],
    map: &NodeMap,
    owners: &[String],
    file: &Option<String>,
) -> Vec<Layer> {
    stmts.iter().map(|s| layer(s, map, owners, file)).collect()
}

fn layer(s: &Statement, map: &NodeMap, owners: &[String], file: &Option<String>) -> Layer {
    let at = |kind: &'static str,
              label: String,
              source: Option<&'static str>,
              id: Option<NodeId>,
              children: Vec<Layer>| Layer {
        kind,
        label,
        source,
        id,
        file: file.clone(),
        start: s.span.start,
        end: s.span.end,
        line: s.span.line,
        children,
    };
    let branch = |kind: &'static str, label: String, body: &[Statement]| Layer {
        kind,
        label,
        source: None,
        id: None,
        file: file.clone(),
        start: body.first().map_or(s.span.start, |b| b.span.start),
        end: body.last().map_or(s.span.end, |b| b.span.end),
        line: body.first().map_or(s.span.line, |b| b.span.line),
        children: layer_body(body, map, owners, file),
    };
    let ex = crate::print::expr;
    match &s.kind {
        StatementKind::UIElement(ui) => {
            if let Some(slot) = ui.slot_name() {
                return at("slot", slot.to_string(), None, None, Vec::new());
            }
            let mut children: Vec<Layer> = Vec::new();
            for h in &ui.events {
                children.push(Layer {
                    kind: "source",
                    label: format!("on {}", h.event),
                    source: Some("handler"),
                    id: None,
                    file: file.clone(),
                    start: h.span.start,
                    end: h.span.end,
                    line: h.span.line,
                    children: Vec::new(),
                });
            }
            for fill in &ui.slot_fills {
                children.push(Layer {
                    kind: "slot",
                    label: fill.name.clone(),
                    source: None,
                    id: None,
                    file: file.clone(),
                    start: fill.span.start,
                    end: fill.span.end,
                    line: fill.span.line,
                    children: layer_body(&fill.body, map, owners, file),
                });
            }
            children.extend(layer_body(&ui.children, map, owners, file));
            let id = map.id_among(owners, ui.span).map(str::to_string);
            at(
                "element",
                crate::print::component(&ui.component),
                None,
                id,
                children,
            )
        }
        StatementKind::If(i) => {
            let head = match &i.binding {
                Some(b) => format!("if let {b} = {}", ex(&i.condition)),
                None => format!("if {}", ex(&i.condition)),
            };
            let mut children = vec![branch("if", head.clone(), &i.then_body)];
            for (c, b) in &i.else_if_branches {
                children.push(branch("else if", format!("else if {}", ex(c)), b));
            }
            if let Some(b) = &i.else_body {
                children.push(branch("else", "else".into(), b));
            }
            at("if", head, None, None, children)
        }
        StatementKind::For(f) => {
            let item = match &f.index {
                Some(i) => format!("{}, {i}", f.item),
                None => f.item.clone(),
            };
            let key = f
                .key
                .as_ref()
                .map(|k| format!(" by {}", ex(k)))
                .unwrap_or_default();
            at(
                "for",
                format!("for {item} in {}{key}", ex(&f.iterable)),
                None,
                None,
                layer_body(&f.body, map, owners, file),
            )
        }
        StatementKind::Show(x) => at(
            "show",
            format!("show {}", ex(&x.condition)),
            None,
            None,
            layer_body(&x.body, map, owners, file),
        ),
        StatementKind::Match(m) => {
            let arms = m
                .arms
                .iter()
                .map(|a| Layer {
                    kind: "arm",
                    label: arm_head(a),
                    source: None,
                    id: None,
                    file: file.clone(),
                    start: a.span.start,
                    end: a.span.end,
                    line: a.span.line,
                    children: layer_body(&a.body, map, owners, file),
                })
                .collect();
            at(
                "match",
                format!("match {}", ex(&m.scrutinee)),
                None,
                None,
                arms,
            )
        }
        StatementKind::State(x) => at(
            "source",
            format!("{} {}", if x.persist { "persist" } else { "state" }, x.name),
            Some("state"),
            None,
            Vec::new(),
        ),
        StatementKind::Derived(x) => at(
            "source",
            format!("derived {}", x.name),
            Some("derived"),
            None,
            Vec::new(),
        ),
        StatementKind::Use(x) => at(
            "source",
            format!("use {}", x.store_name),
            Some("use"),
            None,
            Vec::new(),
        ),
        StatementKind::Action(x) => at(
            "source",
            format!("action {}", x.name),
            Some("action"),
            None,
            Vec::new(),
        ),
        StatementKind::Effect(_) => at("source", "effect".into(), Some("effect"), None, Vec::new()),
        StatementKind::Timer(t) => at(
            "source",
            format!(
                "{}({})",
                if t.every { "every" } else { "after" },
                ex(&t.interval)
            ),
            Some("timer"),
            None,
            Vec::new(),
        ),
        StatementKind::Resource(r) => at(
            "source",
            format!("resource {}", r.name),
            Some("resource"),
            None,
            Vec::new(),
        ),
        StatementKind::Connection(c) => at(
            "source",
            c.name.clone(),
            Some("connection"),
            None,
            Vec::new(),
        ),
        StatementKind::Validate(v) => at(
            "source",
            format!("validate {}", v.name),
            Some("validate"),
            None,
            Vec::new(),
        ),
        StatementKind::EventHandler(h) => at(
            "source",
            format!("on {}", h.event),
            Some("handler"),
            None,
            Vec::new(),
        ),
        _ => at(
            "source",
            crate::print::statement(s, crate::print::Block::Render, 0)
                .lines()
                .next()
                .unwrap_or("")
                .to_string(),
            Some("statement"),
            None,
            Vec::new(),
        ),
    }
}

#[cfg(test)]
mod tests {
    //! Slice 2 acceptance: ids are deterministic and resolve to the right spans;
    //! studio compiles stamp `data-wf-node`; export compiles do not; and the JS
    //! and SSG generators agree on the id for any node they both stamp.
    use super::*;
    use crate::codegen::JsCodegen;
    use crate::codegen::ssg::SiteContext;
    use crate::codegen::ssg::{render_page_html, render_page_html_studio};
    use crate::config::ProjectConfig;
    use crate::parser::ast::PageDecl;

    fn program(src: &str) -> Program {
        crate::syntax::parse_source(src, "<test>")
            .map(crate::sema::lower)
            .expect("parse")
    }

    fn config() -> ProjectConfig {
        serde_json::from_str(r#"{"name":"test"}"#).expect("config")
    }

    fn first_page(p: &Program) -> &PageDecl {
        p.declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Page(pg) => Some(pg),
                _ => None,
            })
            .expect("a page")
    }

    const SRC: &str = "page Home(path: \"/\") {\n  Container {\n    Heading(\"Hi\").h1\n    Button(\"Go\").primary\n  }\n}\n";

    #[test]
    fn ids_resolve_to_spans_and_paths() {
        let p = program(SRC);
        let map = build_node_map(&p);
        assert_eq!(map.len(), 3); // Container, Heading, Button

        let container = map.info("Home:0").expect("Home:0");
        assert!(container.span.slice(SRC).starts_with("Container"));
        assert_eq!(container.component, "Home");

        let heading = map.info("Home:0.0").expect("Home:0.0");
        assert_eq!(heading.span.slice(SRC), "Heading(\"Hi\").h1");

        let button = map.info("Home:0.1").expect("Home:0.1");
        assert_eq!(button.span.slice(SRC), "Button(\"Go\").primary");

        // Reverse lookup by span matches.
        assert_eq!(map.id_for(heading.span), Some("Home:0.0"));
    }

    #[test]
    fn map_is_deterministic() {
        let p = program(SRC);
        let a = build_node_map(&p);
        let b = build_node_map(&p);
        let mut va: Vec<_> = a
            .iter()
            .map(|(id, i)| (id.clone(), i.span.start, i.span.end))
            .collect();
        let mut vb: Vec<_> = b
            .iter()
            .map(|(id, i)| (id.clone(), i.span.start, i.span.end))
            .collect();
        va.sort();
        vb.sort();
        assert_eq!(va, vb);
    }

    #[test]
    fn control_flow_uses_branch_tags() {
        let src = "page P(path: \"/\") {\n  if (x) {\n    Text(\"then\")\n  } else {\n    Text(\"else\")\n  }\n  for item in items {\n    Text(\"row\")\n  }\n}\n";
        let p = program(src);
        let map = build_node_map(&p);
        // if is statement 0: then -> P:0.t.0, else -> P:0.e.0 (no collision).
        assert_eq!(
            map.info("P:0.t.0").map(|i| i.span.slice(src)),
            Some("Text(\"then\")")
        );
        assert_eq!(
            map.info("P:0.e.0").map(|i| i.span.slice(src)),
            Some("Text(\"else\")")
        );
        // for is statement 1: body -> P:1.0
        assert_eq!(
            map.info("P:1.0").map(|i| i.span.slice(src)),
            Some("Text(\"row\")")
        );
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

        // Export must stamp no element. (The runtime's WF.__debug helpers mention
        // `data-wf-node` as a selector string, so check the exact stamp form — a
        // `"data-wf-node": "<id>"` WF.el attrs entry — rather than the bare name.)
        let export_js = JsCodegen::new().generate(&p);
        assert!(!export_js.contains("\"data-wf-node\":"));
    }

    #[test]
    fn ssg_studio_stamps_export_does_not() {
        let p = program(SRC);
        let map = build_node_map(&p);
        let page = first_page(&p);
        let cfg = config();

        let studio_html = render_page_html_studio(page, &SiteContext::bare(&cfg, &p), true, &map);
        assert!(studio_html.contains("data-wf-node=\"Home:0.0\""));
        assert!(studio_html.contains("data-wf-node=\"Home:0.1\""));

        let export_html = render_page_html(page, &SiteContext::bare(&cfg, &p));
        assert!(!export_html.contains("data-wf-node"));
    }

    #[test]
    fn special_component_roots_are_stamped() {
        // These emitters build their own root and ignore the attrs choke point,
        // so each must be stamped explicitly. Regression guard for that gap.
        let src = "page S(path: \"/\") {\n  state on = false\n  Switch(bind: on, label: \"T\")\n  Checkbox(bind: on, label: \"A\")\n  Modal(title: \"H\", visible: on) { Text(\"b\") }\n  Dropdown(label: \"M\") { Text(\"i\") }\n  Tabs { Tabs.Page(\"One\") { Text(\"a\") } }\n}\n";
        let p = program(src);
        let map = build_node_map(&p);
        let mut jsgen = JsCodegen::new();
        jsgen.set_studio(map.clone());
        let js = jsgen.generate(&p);

        for comp in ["Switch", "Checkbox", "Modal", "Dropdown", "Tabs"] {
            let id = map
                .iter()
                .find(|(_, info)| info.span.slice(src).starts_with(comp))
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| panic!("no map entry for {}", comp));
            assert!(
                js.contains(&format!("\"data-wf-node\": \"{}\"", id)),
                "{} root (id {}) not stamped",
                comp,
                id
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
        let html = render_page_html_studio(page, &SiteContext::bare(&config(), &p), true, &map);

        // The Heading's id is produced by the one shared map, so both stamp the same string.
        for id in ["Home:0", "Home:0.0", "Home:0.1"] {
            assert!(
                js.contains(&format!("\"data-wf-node\": \"{}\"", id)),
                "js missing {}",
                id
            );
            assert!(
                html.contains(&format!("data-wf-node=\"{}\"", id)),
                "html missing {}",
                id
            );
        }
    }

    #[test]
    fn duplicate_names_across_kinds_get_distinct_ids() {
        // A Page and a Component may legally share a name; their ids must not collapse.
        let src = "component Profile(name: String) { Text(name).bold }\npage Profile(path: \"/me\") { Container { Heading(\"Me\").h1 } }\n";
        let p = program(src);
        let map = build_node_map(&p);
        assert!(
            map.info("Profile#component:0").is_some(),
            "component node id"
        );
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
        let src = "page Home(path: \"/\") { Text(\"hi\") }\napp {\n  Row {\n    Container { Text(\"nav\") }\n    Router\n  }\n}\n";
        let p = program(src);
        let map = build_node_map(&p);
        let row_id = map
            .iter()
            .find(|(_, i)| i.span.slice(src).starts_with("Row"))
            .map(|(id, _)| id.clone())
            .expect("Row wrapper id");

        let mut jsgen = JsCodegen::new();
        jsgen.set_studio(map.clone());
        let js = jsgen.generate(&p);
        assert!(
            js.contains(&format!("\"data-wf-node\": \"{}\"", row_id)),
            "JS app wrapper not stamped"
        );

        let page = first_page(&p);
        let app_body: Vec<Statement> = p
            .declarations
            .iter()
            .find_map(|d| {
                if let Declaration::App(a) = d {
                    Some(a.body.clone())
                } else {
                    None
                }
            })
            .expect("app body");
        let html = render_page_html_studio(
            page,
            &SiteContext {
                config: &config(),
                app_body: Some(&app_body),
                translations: &Default::default(),
                components: &Default::default(),
                program: &p,
                program_scope: None,
                page_sheets: None,
                node_map: None,
            },
            true,
            &map,
        );
        assert!(
            html.contains(&format!("data-wf-node=\"{}\"", row_id)),
            "SSG app wrapper not stamped"
        );
    }

    #[test]
    fn compound_widget_children_are_stamped_in_js() {
        // Regression: special emitters hand-build sub-element DOM; each of these
        // user-written source elements must still carry its node id in JS.
        let src = "page P(path: \"/\") {\n  state on = false\n  Sidebar { Sidebar.Header { Text(\"B\") } Sidebar.Item(to: \"/a\") { Text(\"A\") } }\n  Tabs { Tabs.Page(\"One\") { Text(\"a\") } }\n  Breadcrumb { Breadcrumb.Item(to: \"/x\") { Text(\"X\") } }\n  Carousel { Carousel.Slide { Text(\"s\") } }\n  Modal(title: \"H\", visible: on) { Text(\"b\") Modal.Footer { Button(\"OK\") } }\n}\n";
        let p = program(src);
        let map = build_node_map(&p);
        let mut jsgen = JsCodegen::new();
        jsgen.set_studio(map.clone());
        let js = jsgen.generate(&p);

        for needle in [
            "Sidebar.Header",
            "Sidebar.Item",
            "Tabs.Page",
            "Breadcrumb.Item",
            "Carousel.Slide",
            "Modal.Footer",
        ] {
            let id = map
                .iter()
                .find(|(_, i)| i.span.slice(src).starts_with(needle))
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| panic!("no map entry for {}", needle));
            assert!(
                js.contains(&format!("\"data-wf-node\": \"{}\"", id)),
                "{} not stamped in JS (id {})",
                needle,
                id
            );
        }
    }

    const PAGE: &str = "store Deploys {\n    state live: [Any] = []\n}\n\npage Home(path: \"/\", title: \"Home\") {\n    use Deploys\n    state open = false\n    Stack(gap: .md) {\n        Heading(\"Deploys\").h1\n        for d in Deploys.live by d.id {\n            Card {\n                Text(d.name)\n            }\n        }\n        if open {\n            Badge(\"open\")\n        } else {\n            Badge(\"shut\")\n        }\n        Text(\"{Deploys.live.length} live\", hidden: !open)\n    }\n}\n";

    fn info_of<'a>(map: &'a NodeMap, src: &str, needle: &str) -> &'a NodeInfo {
        let at = src.find(needle).unwrap_or_else(|| panic!("no `{needle}`"));
        map.at("t.wf", at)
            .unwrap_or_else(|| panic!("nothing at `{needle}`"))
    }

    #[test]
    fn the_caret_finds_the_innermost_node() {
        let p = crate::syntax::parse_source(PAGE, "t.wf").unwrap();
        let map = build_node_map_with(&p, &|_| Some("t.wf".into()));
        assert_eq!(info_of(&map, PAGE, "Heading").element, "Heading");
        assert_eq!(info_of(&map, PAGE, "\"Deploys\").h1").element, "Heading");
        assert_eq!(info_of(&map, PAGE, "d.name").element, "Text");
        // In a block but in no child: the block's own element.
        assert_eq!(info_of(&map, PAGE, "gap: .md").element, "Stack");
        // Between children, inside a loop's head: the element around it.
        assert_eq!(info_of(&map, PAGE, "by d.id").element, "Stack");
        assert!(map.at("t.wf", 0).is_none(), "the store holds no element");
        assert!(map.at("other.wf", PAGE.find("Heading").unwrap()).is_none());
        let heading = info_of(&map, PAGE, "Heading");
        assert_eq!(
            map.exactly("t.wf", heading.start as usize, heading.end as usize)
                .unwrap()
                .path,
            heading.path
        );
        assert_eq!(heading.parent.as_deref(), Some("Home:2"));
        let kids: Vec<&str> = map
            .children("Home:2")
            .iter()
            .map(|n| n.element.as_str())
            .collect();
        assert_eq!(kids, ["Heading", "Card", "Badge", "Badge", "Text"]);
    }

    #[test]
    fn a_node_says_what_it_is_inside_and_what_it_reads() {
        let p = crate::syntax::parse_source(PAGE, "t.wf").unwrap();
        let map = build_node_map_with(&p, &|_| Some("t.wf".into()));
        let heading = info_of(&map, PAGE, "Heading");
        assert_eq!(heading.kind(), "plain");
        let name = info_of(&map, PAGE, "d.name");
        assert_eq!(name.kind(), "template");
        assert_eq!(name.within[0].head, "for d in Deploys.live by d.id");
        assert_eq!(
            name.bound,
            [Bound {
                arg: "text".into(),
                reads: vec![Read {
                    name: "d".into(),
                    from: "loop"
                }]
            }]
        );
        let shut = info_of(&map, PAGE, "\"shut\"");
        assert_eq!(shut.kind(), "conditional");
        assert_eq!(
            (shut.within[0].kind, shut.within[0].head.as_str()),
            ("else", "else")
        );
        let open = info_of(&map, PAGE, "\"open\")");
        assert_eq!(open.within[0].head, "if open");
        let live = info_of(&map, PAGE, "live\", hidden");
        assert_eq!(live.kind(), "bound");
        let reads: Vec<(&str, &str)> = live
            .bound
            .iter()
            .flat_map(|b| b.reads.iter().map(move |r| (b.arg.as_str(), r.from)))
            .collect();
        assert_eq!(reads, [("text", "store"), ("hidden", "state")]);
    }

    #[test]
    fn two_files_with_the_same_layout_keep_their_own_ids() {
        // Each file starts at byte 0, so these two `Text`s are written at
        // the same span; each is still its own component's.
        let a = "component A {\n    Text(\"a\")\n}\n";
        let b = "component B {\n    Text(\"b\")\n}\n";
        let mut p = crate::syntax::parse_source(a, "a.wf").unwrap();
        p.declarations
            .extend(crate::syntax::parse_source(b, "b.wf").unwrap().declarations);
        p.declarations.extend(
            crate::syntax::parse_source("page P(path: \"/\") {\n    A\n    B\n}\n", "p.wf")
                .unwrap()
                .declarations,
        );
        let files = ["a.wf", "b.wf", "p.wf"];
        let map = build_node_map_with(&p, &|i| Some(files[i].to_string()));
        let span = map.info("A:0").unwrap().span;
        assert_eq!(map.info("B:0").unwrap().span, span);
        assert_eq!(map.id_for(span), None, "ambiguous by span alone");
        assert_eq!(map.id_in("A", span), Some("A:0"));
        assert_eq!(map.id_in("B", span), Some("B:0"));
        assert_eq!(map.at("b.wf", 18).unwrap().path, "B:0");
        assert_eq!(map.at("a.wf", 18).unwrap().path, "A:0");
        // And the bundle stamps each with its own.
        let lowered = crate::sema::lower(p.clone());
        let mut js = JsCodegen::new();
        js.set_studio(build_node_map(&lowered));
        let out = js.generate(&lowered);
        assert!(
            out.contains("\"data-wf-node\": \"A:0\"") || out.contains("\"data-wf-node\":\"A:0\""),
            "{out}"
        );
        assert!(
            out.contains("\"data-wf-node\": \"B:0\"") || out.contains("\"data-wf-node\":\"B:0\"")
        );
    }

    #[test]
    fn layers_hold_every_node_and_the_source_beside_them() {
        let p = crate::syntax::parse_source(PAGE, "t.wf").unwrap();
        let tree = layers(&p, &|_| Some("t.wf".into()));
        assert_eq!(tree.len(), 1);
        let home = &tree[0];
        assert_eq!((home.kind, home.label.as_str()), ("page", "Home"));
        let top: Vec<(&str, &str)> = home
            .children
            .iter()
            .map(|l| (l.kind, l.label.as_str()))
            .collect();
        assert_eq!(
            top,
            [
                ("source", "use Deploys"),
                ("source", "state open"),
                ("element", "Stack")
            ]
        );
        let stack = &home.children[2];
        assert_eq!(stack.id.as_deref(), Some("Home:2"));
        let inner: Vec<(&str, &str)> = stack
            .children
            .iter()
            .map(|l| (l.kind, l.label.as_str()))
            .collect();
        assert_eq!(
            inner,
            [
                ("element", "Heading"),
                ("for", "for d in Deploys.live by d.id"),
                ("if", "if open"),
                ("element", "Text")
            ]
        );
        let branches: Vec<&str> = stack.children[2].children.iter().map(|l| l.kind).collect();
        assert_eq!(branches, ["if", "else"]);
        assert_eq!(stack.children[2].children[1].children[0].label, "Badge");
    }

    #[test]
    fn a_selection_follows_an_edit() {
        use crate::reprint::{Patch, follow};
        let p = |s, e, t: &str| Patch {
            start: s,
            end: e,
            text: t.into(),
        };
        assert_eq!(follow(10, 20, &[p(0, 2, "abcd")]), Some((12, 22)));
        assert_eq!(follow(10, 20, &[p(12, 14, "")]), Some((10, 18)));
        assert_eq!(follow(10, 20, &[p(25, 30, "x")]), Some((10, 20)));
        assert_eq!(follow(10, 20, &[p(10, 10, "new\n")]), Some((14, 24)));
        assert_eq!(follow(10, 20, &[p(8, 12, "x")]), None, "an edge rewritten");
        assert_eq!(follow(10, 20, &[p(5, 25, "x")]), None, "the whole of it");
        let before = "abc Text(\"x\") def";
        let after = "abc Text(\"changed\") def";
        assert_eq!(
            follow(4, 13, &[crate::reprint::between(before, after)]),
            Some((4, 19))
        );
    }

    #[test]
    fn an_instance_names_its_calls_innermost_first() {
        let src = "component Inner {\n    Card {\n        Text(\"in\")\n    }\n}\n\ncomponent Outer {\n    Inner\n}\n\npage Home(path: \"/\") {\n    Outer\n}\n";
        let p = program(src);
        let map = build_node_map(&p);
        let cfg = config();
        let components: std::collections::HashMap<String, crate::parser::ast::ComponentDecl> = p
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Component(c) => Some((c.name.clone(), c.clone())),
                _ => None,
            })
            .collect();
        let site = SiteContext {
            components: &components,
            ..SiteContext::bare(&cfg, &p)
        };
        let html = render_page_html_studio(first_page(&p), &site, true, &map);
        assert!(
            html.contains("data-wf-node=\"Inner:0\" data-wf-call=\"Outer:0 Home:0\""),
            "{html}"
        );
    }
}
