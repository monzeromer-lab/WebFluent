//! What a name at a place can refer to: the scope the editor's completion
//! offers and an inspector's path picker lists — the loop variables and
//! bindings around the place, innermost first, then the declaration's own
//! props, state, derived values and actions, then the stores it `use`s —
//! and, in `paths`, the same list typed, for a picker.

use crate::parser::ast::*;

/// Whether `offset` falls inside `span`.
pub fn contains(span: Span, offset: usize) -> bool {
    (span.start as usize) <= offset && offset < (span.end as usize)
}

/// The statements a declaration's body holds.
pub fn body_of(decl: &Declaration) -> &[Statement] {
    match decl {
        Declaration::Page(p) => &p.body,
        Declaration::Component(c) => &c.body,
        Declaration::Store(s) => &s.body,
        Declaration::App(a) => &a.body,
        Declaration::Theme(_)
        | Declaration::Type(_)
        | Declaration::Enum(_)
        | Declaration::Api(_)
        | Declaration::Script(_)
        | Declaration::Const(_)
        | Declaration::Animation(_)
        | Declaration::Data(_) => &[],
        Declaration::Test(t) => &t.body,
    }
}

/// Every statement list nested in a statement: bodies, branches, handlers.
pub fn child_bodies(stmt: &Statement) -> Vec<&[Statement]> {
    match &stmt.kind {
        StatementKind::UIElement(el) => {
            let mut bodies: Vec<&[Statement]> = vec![&el.children];
            bodies.extend(el.events.iter().map(|e| e.body.as_slice()));
            bodies.extend(el.slot_fills.iter().map(|f| f.body.as_slice()));
            bodies
        }
        StatementKind::If(i) => {
            let mut bodies: Vec<&[Statement]> = vec![&i.then_body];
            bodies.extend(i.else_if_branches.iter().map(|(_, b)| b.as_slice()));
            bodies.extend(i.else_body.as_deref());
            bodies
        }
        StatementKind::For(f) => vec![&f.body],
        StatementKind::Try(t) => vec![&t.body, &t.catch_body],
        StatementKind::Show(s) => vec![&s.body],
        StatementKind::Fetch(f) => {
            let mut bodies: Vec<&[Statement]> = Vec::new();
            bodies.extend(f.loading_block.as_deref());
            bodies.extend(f.error_block.as_ref().map(|(_, b)| b.as_slice()));
            bodies.extend(f.success_block.as_deref());
            bodies
        }
        StatementKind::Match(m) => m.arms.iter().map(|a| a.body.as_slice()).collect(),
        StatementKind::Effect(e) => vec![&e.body, &e.cleanup],
        StatementKind::Timer(t) => vec![&t.body],
        StatementKind::Action(a) => vec![&a.body],
        StatementKind::EventHandler(h) => vec![&h.body],
        _ => Vec::new(),
    }
}

/// The chain of statements from the body down to the innermost one that
/// contains `offset`, outermost first.
pub fn statement_path(stmts: &[Statement], offset: usize) -> Vec<&Statement> {
    let mut path = Vec::new();
    let mut current = stmts;
    'descend: loop {
        for stmt in current {
            if contains(stmt.span, offset) {
                path.push(stmt);
                for body in child_bodies(stmt) {
                    if body.iter().any(|s| contains(s.span, offset)) {
                        current = body;
                        continue 'descend;
                    }
                }
                break 'descend;
            }
        }
        break;
    }
    path
}

/// A name in scope and what declared it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub name: String,
    pub kind: BindingKind,
    /// Byte range of the declaring statement or header, in its own file.
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BindingKind {
    State,
    Derived,
    Action,
    Prop,
    Param,
    LoopItem,
    LoopIndex,
    FetchResult,
    FetchError,
    Store,
    /// A `resource`, rendered with `match`.
    Resource,
    /// The value or error a `match` arm binds, an `if let` name, or a name
    /// a scoped slot's fill gives a value.
    ArmBinding,
    /// A `:param` of the page's route.
    RouteParam,
    /// A connection the page holds open.
    Socket,
    Stream,
    Channel,
    Peer,
    /// `worker css = spawn(…)`: a Web Worker the page or store holds.
    Worker,
    /// `Form(bind: form)`: a handle on the form.
    FormHandle,
    /// `page` and `pages` inside a `Header`, `Footer` or `Background`: the
    /// page being drawn, and how many there are.
    PageNumber,
    /// `ref: name`: a handle on an element.
    ElementHandle,
}

impl BindingKind {
    pub fn label(self) -> &'static str {
        match self {
            BindingKind::State => "state",
            BindingKind::Derived => "derived",
            BindingKind::Action => "action",
            BindingKind::Prop => "prop",
            BindingKind::Param => "parameter",
            BindingKind::LoopItem => "loop item",
            BindingKind::LoopIndex => "loop index",
            BindingKind::FetchResult => "fetch result",
            BindingKind::FetchError => "fetch error",
            BindingKind::Store => "store",
            BindingKind::Resource => "resource",
            BindingKind::ArmBinding => "binding",
            BindingKind::RouteParam => "route parameter",
            BindingKind::Socket => "socket",
            BindingKind::Stream => "stream",
            BindingKind::Channel => "channel",
            BindingKind::Peer => "peer",
            BindingKind::Worker => "worker",
            BindingKind::FormHandle => "form handle",
            BindingKind::PageNumber => "page number",
            BindingKind::ElementHandle => "element handle",
        }
    }
}

/// Everything a name at `offset` inside `decl` can refer to, innermost first:
/// loop variables and fetch bindings of the enclosing statements, then the
/// declaration's own props, state, derived values and actions (hoisted —
/// the compiler resolves them regardless of order), then the stores it
/// `use`s.
pub fn scope_at(decl: &Declaration, offset: usize) -> Vec<Binding> {
    let mut scope = Vec::new();
    let body = body_of(decl);

    let path = statement_path(body, offset);
    let locals = locals_along(&path, offset);
    for (depth, stmt) in path.iter().enumerate().rev() {
        // What an action, a handler, an effect or a timer declares with
        // `let` before the place, in the body around it.
        for b in locals.iter().filter(|(d, _)| *d == depth).map(|(_, b)| b) {
            scope.push(b.clone());
        }
        match &stmt.kind {
            StatementKind::For(f) => {
                scope.push(Binding {
                    name: f.item.clone(),
                    kind: BindingKind::LoopItem,
                    span: stmt.span,
                });
                if let Some(index) = &f.index {
                    scope.push(Binding {
                        name: index.clone(),
                        kind: BindingKind::LoopIndex,
                        span: stmt.span,
                    });
                }
            }
            StatementKind::Fetch(f) => {
                scope.push(Binding {
                    name: f.variable.clone(),
                    kind: BindingKind::FetchResult,
                    span: stmt.span,
                });
                if let Some((err, _)) = &f.error_block {
                    scope.push(Binding {
                        name: err.clone(),
                        kind: BindingKind::FetchError,
                        span: stmt.span,
                    });
                }
            }
            StatementKind::Action(a) => {
                for p in &a.params {
                    scope.push(Binding {
                        name: p.name.clone(),
                        kind: BindingKind::Param,
                        span: stmt.span,
                    });
                }
            }
            StatementKind::If(i) => {
                if let Some(name) = &i.binding {
                    scope.push(Binding {
                        name: name.clone(),
                        kind: BindingKind::ArmBinding,
                        span: stmt.span,
                    });
                }
            }
            StatementKind::Match(m) => {
                for arm in &m.arms {
                    if let Some(name) = &arm.binding
                        && contains(arm.span, offset)
                    {
                        scope.push(Binding {
                            name: name.clone(),
                            kind: BindingKind::ArmBinding,
                            span: arm.span,
                        });
                    }
                }
            }
            StatementKind::EventHandler(h) => {
                for param in &h.params {
                    scope.push(Binding {
                        name: param.clone(),
                        kind: BindingKind::Param,
                        span: h.span,
                    });
                }
            }
            // `catch e { … }`: what was thrown, inside the catch block.
            StatementKind::Try(t) => {
                if let Some(param) = &t.param
                    && t.catch_body
                        .first()
                        .is_some_and(|s| offset >= s.span.start as usize)
                {
                    scope.push(Binding {
                        name: param.clone(),
                        kind: BindingKind::Param,
                        span: stmt.span,
                    });
                }
            }
            // `row(t, i) { … }`: the values a scoped slot hands its fill; and
            // `on menu(key, x, y) { … }`, the names a handler gives its
            // event's values, inside the handler the place is in.
            StatementKind::UIElement(el) => {
                for h in el.events.iter().filter(|h| contains(h.span, offset)) {
                    for param in &h.params {
                        scope.push(Binding {
                            name: param.clone(),
                            kind: BindingKind::Param,
                            span: h.span,
                        });
                    }
                }
                if let ComponentRef::BuiltIn(n) = &el.component
                    && matches!(n.as_str(), "Header" | "Footer" | "Background")
                {
                    for name in ["page", "pages"] {
                        scope.push(Binding {
                            name: name.to_string(),
                            kind: BindingKind::PageNumber,
                            span: stmt.span,
                        });
                    }
                }
                for fill in &el.slot_fills {
                    if contains(fill.span, offset) {
                        for param in &fill.params {
                            scope.push(Binding {
                                name: param.clone(),
                                kind: BindingKind::ArmBinding,
                                span: fill.span,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // A page's route parameters: `page User(path: "/u/:id", id: String)`.
    if let Declaration::Page(p) = decl {
        for param in &p.params {
            scope.push(Binding {
                name: param.name.clone(),
                kind: BindingKind::RouteParam,
                span: p.header_span,
            });
        }
    }

    if let Declaration::Component(c) = decl {
        for p in &c.props {
            scope.push(Binding {
                name: p.name.clone(),
                kind: BindingKind::Prop,
                span: c.header_span,
            });
        }
    }

    hoisted(body, &mut scope);
    scope
}

/// The bodies of a statement that are drawn rather than run: an element's
/// children and slot fills, a branch, a loop — not an action's, a
/// handler's, an effect's or a timer's, whose `let`s are their own.
fn rendered_bodies(stmt: &Statement) -> Vec<&[Statement]> {
    match &stmt.kind {
        StatementKind::Action(_)
        | StatementKind::EventHandler(_)
        | StatementKind::Effect(_)
        | StatementKind::Timer(_) => Vec::new(),
        StatementKind::UIElement(el) => {
            let mut bodies: Vec<&[Statement]> = vec![&el.children];
            bodies.extend(el.slot_fills.iter().map(|f| f.body.as_slice()));
            bodies
        }
        _ => child_bodies(stmt),
    }
}

/// The child body of `stmt` that holds `offset`, and whether it is run
/// rather than drawn.
fn body_at(stmt: &Statement, offset: usize) -> Option<(&[Statement], bool)> {
    let run_all = matches!(
        stmt.kind,
        StatementKind::Action(_)
            | StatementKind::EventHandler(_)
            | StatementKind::Effect(_)
            | StatementKind::Timer(_)
    );
    if let StatementKind::UIElement(el) = &stmt.kind
        && let Some(h) = el.events.iter().find(|h| contains(h.span, offset))
    {
        return Some((&h.body, true));
    }
    child_bodies(stmt)
        .into_iter()
        .find(|b| {
            b.iter().any(|s| contains(s.span, offset))
                || b.iter().any(|s| (s.span.end as usize) <= offset)
        })
        .map(|b| (b, run_all))
}

/// The `let`s in scope at `offset`: in each run body along `path`, the
/// ones begun before it, with the depth of the statement whose body
/// holds them.
fn locals_along(path: &[&Statement], offset: usize) -> Vec<(usize, Binding)> {
    let mut out = Vec::new();
    let mut running = false;
    for (depth, stmt) in path.iter().enumerate() {
        let Some((body, run)) = body_at(stmt, offset) else {
            continue;
        };
        running = running || run;
        if !running {
            continue;
        }
        for s in body.iter().rev() {
            // Begun before the place: its own name, where it is declared,
            // is in scope too.
            if let StatementKind::State(d) = &s.kind
                && (s.span.start as usize) < offset
            {
                out.push((
                    depth,
                    Binding {
                        name: d.name.clone(),
                        kind: BindingKind::State,
                        span: s.span,
                    },
                ));
            }
        }
    }
    out
}

/// The `let`s of the action, handler, effect or timer around `offset` in
/// `decl` that are written before it, innermost first — the names
/// [`scope_at`] lists as `State` that are a run body's own.
pub fn locals_at(decl: &Declaration, offset: usize) -> Vec<Binding> {
    let path = statement_path(body_of(decl), offset);
    let mut out: Vec<(usize, Binding)> = locals_along(&path, offset);
    out.sort_by_key(|(d, _)| std::cmp::Reverse(*d));
    out.into_iter().map(|(_, b)| b).collect()
}

/// The state, derived values, actions and `use`d stores declared anywhere in
/// a body, at any depth — the compiler hoists them all.
pub fn hoisted(stmts: &[Statement], scope: &mut Vec<Binding>) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::State(s) => scope.push(Binding {
                name: s.name.clone(),
                kind: BindingKind::State,
                span: stmt.span,
            }),
            StatementKind::Derived(d) => scope.push(Binding {
                name: d.name.clone(),
                kind: BindingKind::Derived,
                span: stmt.span,
            }),
            StatementKind::Action(a) => scope.push(Binding {
                name: a.name.clone(),
                kind: BindingKind::Action,
                span: stmt.span,
            }),
            StatementKind::Use(u) => scope.push(Binding {
                name: u.store_name.clone(),
                kind: BindingKind::Store,
                span: stmt.span,
            }),
            StatementKind::Resource(r) => scope.push(Binding {
                name: r.name.clone(),
                kind: BindingKind::Resource,
                span: stmt.span,
            }),
            StatementKind::Connection(c) => scope.push(Binding {
                name: c.name.clone(),
                kind: match c.kind {
                    ConnectionKind::Socket => BindingKind::Socket,
                    ConnectionKind::Stream => BindingKind::Stream,
                    ConnectionKind::Channel => BindingKind::Channel,
                    ConnectionKind::Peer => BindingKind::Peer,
                    ConnectionKind::Worker => BindingKind::Worker,
                },
                span: stmt.span,
            }),
            // `Form(bind: form)` and `ref: name`: handles the page names.
            StatementKind::UIElement(el) => {
                for arg in &el.args {
                    match arg {
                        Arg::Named(key, Expr::Identifier(name))
                            if key == "bind"
                                && matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Form") =>
                        {
                            scope.push(Binding {
                                name: name.clone(),
                                kind: BindingKind::FormHandle,
                                span: stmt.span,
                            });
                        }
                        Arg::Named(key, Expr::Identifier(name)) if key == "ref" => {
                            scope.push(Binding {
                                name: name.clone(),
                                kind: BindingKind::ElementHandle,
                                span: stmt.span,
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        for body in rendered_bodies(stmt) {
            hoisted(body, scope);
        }
    }
}

/// The members of a store: its state, derived values and actions.
pub fn store_members(store: &StoreDecl) -> Vec<Binding> {
    let mut members = Vec::new();
    hoisted(&store.body, &mut members);
    members.retain(|b| b.kind != BindingKind::Store);
    members
}

/// The whole extent of a declaration, where it has one.
pub fn decl_span(decl: &Declaration) -> Option<Span> {
    Some(match decl {
        Declaration::Page(p) => p.span,
        Declaration::Component(c) => c.span,
        Declaration::Store(s) => s.span,
        Declaration::Theme(t) => t.span,
        Declaration::Type(t) => t.span,
        Declaration::Enum(e) => e.span,
        Declaration::Api(a) => a.span,
        Declaration::Const(c) => c.span,
        Declaration::Data(d) => d.span,
        Declaration::Animation(a) => a.span,
        Declaration::Test(t) => t.span,
        Declaration::App(a) => {
            let first = a.body.first()?.span;
            let last = a.body.last()?.span;
            Span {
                start: first.start,
                end: last.end,
                ..first
            }
        }
        Declaration::Script(_) => return None,
    })
}

pub mod paths {
    //! The scope as a path picker lists it: what a condition, a loop, a
    //! state picker, an `Any` prop or a template's binding may read at a
    //! place, in the picker's order — the page or component's own `state`
    //! and `derived`, its props (and a page's route parameters), the loop
    //! variables and bindings around the place, innermost first, the
    //! members of every store it `use`s, and in a template the data tree —
    //! each with its type, its shape, where it comes from, and the fields
    //! to go on into.
    //!
    //! It is `scope_at` — the list the editor's
    //! completion offers — with the type checker's types.

    use serde::Serialize;

    use super::{Binding, BindingKind, hoisted, scope_at, store_members};
    use crate::parser::ast::{Declaration, Program, StatementKind, TypeDecl};
    use crate::sema::types::{Scalar, Type, TypeInfo};

    /// Which part of the list a row is in.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Group {
        /// The declaration's own `state` and `derived`, its resources and
        /// the form it binds.
        Own,
        /// A component's props; a page's route parameters.
        Props,
        /// What the statements around the place bind: a loop's item and
        /// index, an `if let`, a `match` arm, a slot's fill, a handler's
        /// parameter.
        Bindings,
        /// A store's state and derived values.
        Stores,
        /// A template's data.
        Data,
    }

    /// What kind of value a row holds, for a picker to offer only what fits:
    /// a condition wants `bool`, a loop a `list`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum Shape {
        Bool,
        Number,
        Text,
        /// A date, a time or a moment.
        Moment,
        List,
        Record,
        Enum,
        /// A `resource`, matched with `loading`, `error` and `ready`.
        Resource,
        /// A `Form(bind:)` handle.
        Form,
        Any,
        Other,
    }

    /// One row.
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct Entry {
        /// What a binding writes: `password`, `Deploys.envs`, `d.name`.
        pub path: String,
        /// The last part of it.
        pub name: String,
        pub kind: Option<BindingKind>,
        pub group: Group,
        /// The type as the checker writes it: `[Deploy]`, `String?`.
        pub ty: String,
        pub shape: Shape,
        /// Whether it may be `null`.
        pub optional: bool,
        /// For a list, the type of an item.
        pub item: Option<String>,
        /// Where it comes from: `state in Signup`, `for d in live`,
        /// `member of Deploys`.
        pub from: String,
        /// The line that declares it, where there is one.
        pub line: Option<u32>,
        /// The fields of a record (or of an object in the data), to go on
        /// into.
        pub members: Vec<Entry>,
    }

    /// How deep a record's fields are followed.
    const DEPTH: usize = 3;

    struct Types<'a> {
        program: &'a Program,
    }

    impl<'a> Types<'a> {
        fn record(&self, name: &str) -> Option<&'a TypeDecl> {
            self.program.declarations.iter().find_map(|d| match d {
                Declaration::Type(t) if t.name == name => Some(t),
                _ => None,
            })
        }

        fn is_enum(&self, name: &str) -> bool {
            self.program
                .declarations
                .iter()
                .any(|d| matches!(d, Declaration::Enum(e) if e.name == name))
        }

        /// A type read from a declaration, as the checker resolves it.
        fn resolve(&self, ty: Type) -> Type {
            match ty {
                Type::Record(n) if self.is_enum(&n) => Type::Enum(n),
                Type::Record(n) if self.record(&n).is_none() && Scalar::of_name(&n).is_some() => {
                    Type::Scalar(Scalar::of_name(&n).expect("just checked"))
                }
                Type::List(inner) => Type::List(Box::new(self.resolve(*inner))),
                Type::Optional(inner) => Type::Optional(Box::new(self.resolve(*inner))),
                other => other,
            }
        }

        fn shape(&self, ty: &Type) -> (Shape, bool, Option<String>) {
            match ty {
                Type::Optional(inner) => {
                    let (s, _, item) = self.shape(inner);
                    (s, true, item)
                }
                Type::Bool => (Shape::Bool, false, None),
                Type::Number => (Shape::Number, false, None),
                Type::String => (Shape::Text, false, None),
                Type::Scalar(Scalar::Date | Scalar::Time | Scalar::DateTime) => {
                    (Shape::Moment, false, None)
                }
                Type::Scalar(Scalar::Duration) => (Shape::Number, false, None),
                Type::Scalar(s) if s.is_text() => (Shape::Text, false, None),
                Type::List(inner) => (Shape::List, false, Some(inner.to_string())),
                Type::Record(_) | Type::Shape(_) | Type::Map | Type::Scalar(Scalar::Money) => {
                    (Shape::Record, false, None)
                }
                Type::Enum(_) | Type::Case(_) => (Shape::Enum, false, None),
                Type::Resource(_) => (Shape::Resource, false, None),
                Type::Any => (Shape::Any, false, None),
                _ => (Shape::Other, false, None),
            }
        }

        /// The fields to go on into from a value of `ty` at `path`.
        fn members(
            &self,
            path: &str,
            ty: &Type,
            group: Group,
            from: &str,
            depth: usize,
        ) -> Vec<Entry> {
            if depth >= DEPTH {
                return Vec::new();
            }
            match ty {
                Type::Optional(inner) | Type::Resource(inner) => {
                    self.members(path, inner, group, from, depth)
                }
                Type::Record(name) => {
                    let Some(decl) = self.record(name) else {
                        return Vec::new();
                    };
                    // A record that holds itself (`parent: Node?`) is
                    // followed as far as any other: the depth stops it.
                    let lookup = |n: &str| self.record(n);
                    decl.all_fields(&lookup)
                        .into_iter()
                        .map(|f| {
                            let t = self.resolve(Type::from_ref(&f.ty));
                            self.entry(
                                &format!("{path}.{}", f.name),
                                &f.name,
                                None,
                                group,
                                t,
                                from,
                                Some(f.span.line),
                                depth + 1,
                            )
                        })
                        .collect()
                }
                Type::Shape(fields) => fields
                    .iter()
                    .map(|(n, t)| {
                        self.entry(
                            &format!("{path}.{n}"),
                            n,
                            None,
                            group,
                            self.resolve(t.clone()),
                            from,
                            None,
                            depth + 1,
                        )
                    })
                    .collect(),
                Type::Scalar(Scalar::Money) => ["amount", "currency"]
                    .iter()
                    .map(|n| {
                        let t = if *n == "amount" {
                            Type::Number
                        } else {
                            Type::String
                        };
                        self.entry(
                            &format!("{path}.{n}"),
                            n,
                            None,
                            group,
                            t,
                            from,
                            None,
                            depth + 1,
                        )
                    })
                    .collect(),
                _ => Vec::new(),
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn entry(
            &self,
            path: &str,
            name: &str,
            kind: Option<BindingKind>,
            group: Group,
            ty: Type,
            from: &str,
            line: Option<u32>,
            depth: usize,
        ) -> Entry {
            let (mut shape, optional, item) = self.shape(&ty);
            if kind == Some(BindingKind::FormHandle) {
                shape = Shape::Form;
            }
            let members = if kind == Some(BindingKind::FormHandle) {
                form_members(path, from)
            } else {
                self.members(path, &ty, group, from, depth)
            };
            Entry {
                path: path.to_string(),
                name: name.to_string(),
                kind,
                group,
                ty: if kind == Some(BindingKind::FormHandle) {
                    "Form".to_string()
                } else {
                    ty.to_string()
                },
                shape,
                optional,
                item,
                from: from.to_string(),
                line,
                members,
            }
        }
    }

    /// What a form handle reads: `form.valid`, `form.errors` …
    fn form_members(path: &str, from: &str) -> Vec<Entry> {
        [
            ("valid", "Bool", Shape::Bool),
            ("pending", "Bool", Shape::Bool),
            ("errors", "Map", Shape::Record),
            ("touched", "Map", Shape::Record),
            ("values", "Map", Shape::Record),
        ]
        .iter()
        .map(|(n, t, s)| Entry {
            path: format!("{path}.{n}"),
            name: n.to_string(),
            kind: None,
            group: Group::Own,
            ty: t.to_string(),
            shape: *s,
            optional: false,
            item: None,
            from: from.to_string(),
            line: None,
            members: Vec::new(),
        })
        .collect()
    }

    fn type_of(info: &TypeInfo, decl: usize, b: &Binding) -> Type {
        info.type_at(decl, &b.name, b.span)
            .cloned()
            .or_else(|| {
                info.bindings
                    .iter()
                    .rev()
                    .find(|t| t.decl == decl && t.name == b.name)
                    .map(|t| t.ty.clone())
            })
            .unwrap_or(Type::Any)
    }

    fn owner_name(d: &Declaration) -> &str {
        match d {
            Declaration::Page(p) => &p.name,
            Declaration::Component(c) => &c.name,
            Declaration::Store(s) => &s.name,
            Declaration::Test(t) => &t.name,
            _ => "the app",
        }
    }

    /// Where a binding comes from, as a row says it.
    fn origin(decl: &Declaration, b: &Binding) -> String {
        let owner = owner_name(decl);
        let statement = super::statement_path(super::body_of(decl), b.span.start as usize)
            .into_iter()
            .rev()
            .find(|s| s.span == b.span);
        match b.kind {
            BindingKind::State => format!("state in {owner}"),
            BindingKind::Derived => format!("derived in {owner}"),
            BindingKind::Resource => format!("resource in {owner}"),
            BindingKind::FormHandle => format!("the form in {owner}"),
            BindingKind::Prop => format!("prop of {owner}"),
            BindingKind::RouteParam => format!("the route's :{}", b.name),
            BindingKind::LoopItem | BindingKind::LoopIndex => match statement.map(|s| &s.kind) {
                Some(StatementKind::For(f)) => {
                    let index = f
                        .index
                        .as_ref()
                        .map(|i| format!(", {i}"))
                        .unwrap_or_default();
                    format!(
                        "for {}{index} in {}",
                        f.item,
                        crate::print::expr(&f.iterable)
                    )
                }
                _ => "a loop".to_string(),
            },
            BindingKind::ArmBinding => match statement.map(|s| &s.kind) {
                Some(StatementKind::If(_)) => format!("if let {}", b.name),
                Some(StatementKind::Match(_)) => "a match arm".to_string(),
                _ => "a slot's fill".to_string(),
            },
            BindingKind::FetchResult | BindingKind::FetchError => "the fetch around it".to_string(),
            BindingKind::Param => "a parameter".to_string(),
            BindingKind::PageNumber => "the page being drawn".to_string(),
            other => other.label().to_string(),
        }
    }

    fn line_of(b: &Binding) -> Option<u32> {
        (b.span.line > 0).then_some(b.span.line)
    }

    /// What a path picker lists at `offset` inside declaration `decl_ix` of
    /// `program`, typed by `info` (the checker's, for the same program).
    pub fn at(program: &Program, info: &TypeInfo, decl_ix: usize, offset: usize) -> Vec<Entry> {
        let Some(decl) = program.declarations.get(decl_ix) else {
            return Vec::new();
        };
        let types = Types { program };
        let mut out: Vec<Entry> = Vec::new();
        let scope = scope_at(decl, offset);
        let locals = super::locals_at(decl, offset);
        // The action or handler a `let` belongs to.
        let running = super::statement_path(super::body_of(decl), offset)
            .into_iter()
            .rev()
            .find_map(|s| match &s.kind {
                StatementKind::Action(a) => Some(format!("let in {}", a.name)),
                StatementKind::Effect(_) => Some("let in an effect".to_string()),
                StatementKind::Timer(_) => Some("let in a timer".to_string()),
                StatementKind::EventHandler(h) => Some(format!("let in on {}", h.event)),
                StatementKind::UIElement(el) => el
                    .events
                    .iter()
                    .find(|h| super::contains(h.span, offset))
                    .map(|h| format!("let in on {}", h.event)),
                _ => None,
            })
            .unwrap_or_else(|| "a let".to_string());
        let mut names: Vec<String> = Vec::new();
        for b in &scope {
            let local = locals.iter().any(|l| l.span == b.span && l.name == b.name);
            let group = match b.kind {
                _ if local => Group::Bindings,
                BindingKind::State
                | BindingKind::Derived
                | BindingKind::Resource
                | BindingKind::FormHandle => Group::Own,
                BindingKind::Prop | BindingKind::RouteParam => Group::Props,
                BindingKind::LoopItem
                | BindingKind::LoopIndex
                | BindingKind::ArmBinding
                | BindingKind::FetchResult
                | BindingKind::FetchError
                | BindingKind::Param
                | BindingKind::PageNumber => Group::Bindings,
                // Actions are called, not read; a store is read through its
                // members; a connection and an element handle are not values.
                _ => continue,
            };
            // The innermost of two of a name is the one a path reads.
            if names.contains(&b.name) {
                continue;
            }
            names.push(b.name.clone());
            let ty = match b.kind {
                BindingKind::LoopIndex | BindingKind::PageNumber => Type::Number,
                _ => types.resolve(type_of(info, decl_ix, b)),
            };
            let from = if local {
                running.clone()
            } else if b.kind == BindingKind::Param && running.starts_with("let in ") {
                running.replacen("let in ", "parameter of ", 1)
            } else {
                origin(decl, b)
            };
            out.push(types.entry(
                &b.name,
                &b.name,
                Some(b.kind),
                group,
                ty,
                &from,
                line_of(b),
                0,
            ));
        }
        // Every store the declaration uses, its readable members.
        let mut used = Vec::new();
        hoisted(super::body_of(decl), &mut used);
        for u in used.iter().filter(|b| b.kind == BindingKind::Store) {
            let Some((store_ix, store)) =
                program
                    .declarations
                    .iter()
                    .enumerate()
                    .find_map(|(i, d)| match d {
                        Declaration::Store(s) if s.name == u.name => Some((i, s)),
                        _ => None,
                    })
            else {
                continue;
            };
            let from = format!("member of {}", store.name);
            for m in store_members(store) {
                if !matches!(
                    m.kind,
                    BindingKind::State | BindingKind::Derived | BindingKind::Resource
                ) {
                    continue;
                }
                let ty =
                    types.resolve(info.member(store_ix, &m.name).cloned().unwrap_or(Type::Any));
                let path = format!("{}.{}", store.name, m.name);
                out.push(types.entry(
                    &path,
                    &m.name,
                    Some(m.kind),
                    Group::Stores,
                    ty,
                    &from,
                    line_of(&m),
                    0,
                ));
            }
        }
        // The picker's order: own, props, bindings, stores; each group in
        // the order it was met (bindings innermost first).
        out.sort_by_key(|e| e.group);
        out
    }

    /// The paths a template's data offers: each key, in name order, with
    /// what it holds and the keys of an object inside it; a list's item is
    /// typed from its first, with the keys any of its objects has.
    pub fn data(value: &serde_json::Value) -> Vec<Entry> {
        fn shape_of(v: &serde_json::Value) -> (Shape, String, Option<String>) {
            use serde_json::Value as V;
            match v {
                V::Bool(_) => (Shape::Bool, "Bool".into(), None),
                V::Number(_) => (Shape::Number, "Number".into(), None),
                V::String(s) => {
                    let moment = s.len() >= 10
                        && s.as_bytes()[4] == b'-'
                        && s.as_bytes()[7] == b'-'
                        && s[..4].chars().all(|c| c.is_ascii_digit());
                    if moment {
                        (Shape::Moment, "Date".into(), None)
                    } else {
                        (Shape::Text, "String".into(), None)
                    }
                }
                V::Array(items) => {
                    let item = items
                        .iter()
                        .find(|i| !i.is_null())
                        .map(|i| shape_of(i).1)
                        .unwrap_or_else(|| "Any".into());
                    (Shape::List, format!("[{item}]"), Some(item))
                }
                V::Object(_) => (Shape::Record, "Map".into(), None),
                V::Null => (Shape::Any, "Any".into(), None),
            }
        }
        fn walk(prefix: &str, v: &serde_json::Value, depth: usize) -> Vec<Entry> {
            let object = match v {
                serde_json::Value::Object(map) => Some(map.clone()),
                // A list's items: every key any of them has.
                serde_json::Value::Array(items) => {
                    let mut merged = serde_json::Map::new();
                    for i in items {
                        if let serde_json::Value::Object(m) = i {
                            for (k, x) in m {
                                merged.entry(k.clone()).or_insert_with(|| x.clone());
                            }
                        }
                    }
                    (!merged.is_empty()).then_some(merged)
                }
                _ => None,
            };
            let Some(map) = object else { return Vec::new() };
            if depth > DEPTH {
                return Vec::new();
            }
            map.iter()
                .map(|(k, x)| {
                    let path = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    let (shape, ty, item) = shape_of(x);
                    Entry {
                        members: walk(&path, x, depth + 1),
                        path,
                        name: k.clone(),
                        kind: None,
                        group: Group::Data,
                        ty,
                        shape,
                        optional: x.is_null(),
                        item,
                        from: "the template's data".into(),
                        line: None,
                    }
                })
                .collect()
        }
        walk("", value, 0)
    }

    /// What a path picker lists at `offset` in `file` (relative to `dir`,
    /// as `src/pages/Signup.wf`), in the project as the editor holds it: an
    /// unsaved edit is the one read.
    pub fn at_via(
        vfs: &dyn crate::vfs::Vfs,
        dir: &std::path::Path,
        file: &str,
        offset: usize,
    ) -> crate::error::Result<Vec<Entry>> {
        let (program, files, _) = crate::build::read_project_collecting_via(vfs, dir, None)?;
        let info =
            crate::sema::types::check(&program, &|i| files.get(i).cloned().unwrap_or_default());
        let decl = program.declarations.iter().enumerate().find_map(|(i, d)| {
            let same = files
                .get(i)
                .is_some_and(|f| f == file || f.ends_with(file) || file.ends_with(f.as_str()));
            let span = super::decl_span(d);
            (same && span.is_some_and(|s| super::contains(s, offset))).then_some(i)
        });
        Ok(decl
            .map(|i| at(&program, &info, i, offset))
            .unwrap_or_default())
    }
}
