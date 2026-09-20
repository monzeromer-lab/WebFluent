//! Abstract Syntax Tree (AST) node types for the WebFluent language.
//!
//! The AST is produced by the [`crate::parser::Parser`] and consumed by the
//! code generators in [`crate::codegen`].

// ─── Source spans ────────────────────────────────────────

/// A source span: a byte range into the original `.wf` source plus the
/// 1-based line/column of its start.
///
/// `start`/`end` are **byte** offsets (`end` exclusive), so `&source[start..end]`
/// slices the exact text the node was parsed from. `line`/`col` describe `start`
/// and exist for diagnostics.
///
/// Spans are additive metadata: they power the studio's click-to-code and
/// structured-edit features (see the STUDIO_INTEGRATION_PLAN). Every code
/// generator (html/css/js/ssg/pdf/slides) ignores them, and nodes built outside
/// the parser can use [`Span::dummy`] / `..Default::default()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    /// Byte offset of the first byte of the node in the source.
    pub start: u32,
    /// Byte offset one past the last byte of the node (exclusive).
    pub end: u32,
    /// 1-based line number of `start`.
    pub line: u32,
    /// 1-based column number of `start`.
    pub col: u32,
}

impl Span {
    pub fn new(start: u32, end: u32, line: u32, col: u32) -> Self {
        Self {
            start,
            end,
            line,
            col,
        }
    }

    /// A placeholder span (all zeros) for nodes constructed outside the parser.
    pub fn dummy() -> Self {
        Self::default()
    }

    /// Slice the original `source` this span was taken from.
    ///
    /// Panics if `source` is not the exact string the span was produced against.
    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start as usize..self.end as usize]
    }
}

/// The root AST node — a complete WebFluent program.
///
/// Contains all top-level declarations: pages, components, stores, and the app block.
#[derive(Debug, Clone)]
pub struct Program {
    /// Top-level declarations in source order.
    pub declarations: Vec<Declaration>,
}

/// A top-level declaration in a WebFluent program.
#[derive(Debug, Clone)]
pub enum Declaration {
    Page(PageDecl),
    Component(ComponentDecl),
    Store(StoreDecl),
    App(AppDecl),
    Theme(ThemeDecl),
    /// A record type: `type Todo { id: String, done: Bool = false }`.
    Type(TypeDecl),
    /// An enumeration: `enum Tone { neutral, info, danger }`.
    Enum(EnumDecl),
}

// ─── Types ───────────────────────────────────────────────

/// A record type declaration. Its fields are typed like a component's props.
#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    /// The `///` comment above it.
    pub doc: Option<String>,
    pub span: Span,
    pub header_span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub ty: TypeRef,
    pub default: Option<Expr>,
    pub doc: Option<String>,
    pub span: Span,
}

/// An enumeration declaration: the cases a value of the type can be.
#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub cases: Vec<String>,
    pub doc: Option<String>,
    pub span: Span,
    pub header_span: Span,
}

/// A reference to a type, as written after a colon.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    String,
    Number,
    Bool,
    Map,
    /// Unannotated, or anything: never checked.
    Any,
    /// `[T]`; the original grammar's bare `List` is `List(Any)`.
    List(Box<TypeRef>),
    /// `T?`.
    Optional(Box<TypeRef>),
    /// A declared `type` or `enum`, by name.
    Named(String),
}

// ─── Themes ──────────────────────────────────────────────

/// A design system, declared in the language: `Theme Brand { token … }`.
///
/// The engine used to ship four named palettes as Rust maps, so the only way to
/// change a project's design was a JSON file that named one of them or listed
/// overrides. A theme is design work, and design work belongs next to the code
/// it dresses — in the same language, under the same review, in the same file
/// tree as the `style { }` blocks it sits behind.
#[derive(Debug, Clone)]
pub struct ThemeDecl {
    pub name: String,
    pub tokens: Vec<ThemeToken>,
    pub span: Span,
}

/// One design token: `token color-primary: "#0F766E"`.
#[derive(Debug, Clone)]
pub struct ThemeToken {
    /// The token name without the `--` prefix, e.g. `color-primary`.
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

// ─── Pages ───────────────────────────────────────────────

/// A page declaration: `Page Name (path: "/route", title: "Title") { ... }`.
#[derive(Debug, Clone)]
pub struct PageDecl {
    pub name: String,
    pub path: String,
    pub title: Option<String>,
    pub guard: Option<Expr>,
    pub redirect: Option<String>,

    // ── Search and sharing ──
    /// The snippet a search result and a link preview show. Falls back to the
    /// project description; a page with neither has its snippet written for it
    /// by whatever crawled it.
    pub description: Option<String>,
    /// The image a shared link previews with, as a site-relative or absolute URL.
    pub image: Option<String>,
    /// `og:type` — `website` for most pages, `article` for a post.
    pub page_type: Option<String>,
    /// Keep this page out of search results (`robots: noindex`).
    pub noindex: bool,
    /// The component that frames the page: `layout: AppShell(crumb: "x")`.
    pub layout: Option<LayoutRef>,

    pub body: Vec<Statement>,

    // ── Source spans (additive) ──
    /// Whole declaration, `Page` keyword through the closing `}`.
    pub span: Span,
    /// The header — `Page Name (…)` — up to (excluding) the body's opening `{`.
    pub header_span: Span,
    /// Interior of the `{ … }` body, exclusive of the braces.
    pub body_span: Span,
}

/// The layout a page names in its header: a component call whose default
/// slot is the page.
#[derive(Debug, Clone)]
pub struct LayoutRef {
    pub name: String,
    pub args: Vec<Arg>,
    pub span: Span,
}

// ─── Components ──────────────────────────────────────────

/// A reusable component: `Component Name (props...) { ... }`.
#[derive(Debug, Clone)]
pub struct ComponentDecl {
    pub name: String,
    pub props: Vec<PropDecl>,
    /// The events it declares (`event toggle(id: String)`), fired with `emit`.
    pub events: Vec<EventDecl>,
    /// The slots it declares (`slot`, `slot trailing`); `None` names the
    /// default slot.
    pub slots: Vec<SlotDecl>,
    /// The `///` comment above it.
    pub doc: Option<String>,
    pub body: Vec<Statement>,

    // ── Source spans (additive) ──
    /// Whole declaration, `Component` keyword through the closing `}`.
    pub span: Span,
    /// The header — `Component Name (…)` — up to (excluding) the body's `{`.
    pub header_span: Span,
    /// Interior of the `{ … }` body, exclusive of the braces.
    pub body_span: Span,
}

/// A component property declaration with type, optionality, and default value.
#[derive(Debug, Clone)]
pub struct PropDecl {
    pub name: String,
    pub prop_type: TypeRef,
    pub optional: bool,
    pub default: Option<Expr>,
    /// Whether it is the one prop a call may pass positionally (`_ label:`).
    pub positional: bool,
    pub doc: Option<String>,
    pub span: Span,
}

/// An event a component declares: `event toggle(id: String)`.
#[derive(Debug, Clone)]
pub struct EventDecl {
    pub name: String,
    pub params: Vec<ParamDecl>,
    pub doc: Option<String>,
    pub span: Span,
}

/// A slot a component declares: `slot` (the default) or `slot trailing`.
#[derive(Debug, Clone)]
pub struct SlotDecl {
    pub name: Option<String>,
    pub span: Span,
}

// ─── Stores ──────────────────────────────────────────────

/// A shared state store: `Store Name { state ..., derived ..., action ... }`.
#[derive(Debug, Clone)]
pub struct StoreDecl {
    pub name: String,
    pub body: Vec<Statement>,

    // ── Source spans (additive) ──
    /// Whole declaration, `Store` keyword through the closing `}`.
    pub span: Span,
    /// The header — `Store Name` — up to (excluding) the body's opening `{`.
    pub header_span: Span,
    /// Interior of the `{ … }` body, exclusive of the braces.
    pub body_span: Span,
}

// ─── App ─────────────────────────────────────────────────

/// The root app declaration: `App { Navbar, Router, Footer }`.
#[derive(Debug, Clone)]
pub struct AppDecl {
    pub body: Vec<Statement>,
}

// ─── Types ───────────────────────────────────────────────

// ─── Statements ──────────────────────────────────────────

/// A statement plus its source span.
///
/// The span covers the whole statement — from the first token that belongs to
/// it through the last — so `&source[stmt.span]` slices its exact text. Code
/// generators match on [`Statement::kind`]; the span is additive metadata.
#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

impl Statement {
    pub fn new(kind: StatementKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    State(StateDecl),
    Derived(DerivedDecl),
    Effect(EffectDecl),
    Action(ActionDecl),
    UIElement(UIElement),
    If(IfStmt),
    For(ForStmt),
    Show(ShowStmt),
    Fetch(FetchDecl),
    Assignment(Assignment),
    MethodCall(MethodCallStmt),
    Use(UseDecl),
    EventHandler(EventHandler),
    Navigate(Expr),
    Log(Expr),
    Animate(AnimateStmt),
    ExprStatement(Expr),
    Return(Option<Expr>),
    /// `resource rows = fetch("/api")`: an async value, rendered with `match`.
    Resource(ResourceDecl),
    /// `match rows { loading { … } error(e) { … } ready(v) { … } }`.
    Match(MatchStmt),
    /// `emit toggle(id)`: fire a declared event.
    Emit(EmitStmt),
}

/// An async resource: the request, declared once and rendered anywhere.
#[derive(Debug, Clone)]
pub struct ResourceDecl {
    pub name: String,
    pub ty: Option<TypeRef>,
    pub url: Expr,
    pub options: Vec<FetchOption>,
}

/// A `match` over a resource's states or an enum's cases.
#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub scrutinee: Expr,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: ArmPattern,
    /// The name the arm binds: the error in `error(e)`, the value in
    /// `ready(v)`.
    pub binding: Option<String>,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArmPattern {
    Loading,
    Error,
    Ready,
    /// `.case { … }` over an enum.
    Case(String),
    /// `else { … }`.
    Else,
}

#[derive(Debug, Clone)]
pub struct EmitStmt {
    pub event: String,
    pub args: Vec<Expr>,
}

// ─── State declarations ─────────────────────────────────

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: String,
    /// The declared type, when one is written (`state items: [Todo] = []`).
    pub ty: Option<TypeRef>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct DerivedDecl {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct EffectDecl {
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ActionDecl {
    pub name: String,
    pub params: Vec<ParamDecl>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub name: String,
    pub param_type: TypeRef,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub store_name: String,
}

// ─── UI Elements ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UIElement {
    pub component: ComponentRef,
    pub args: Vec<Arg>,
    pub modifiers: Vec<String>,
    pub children: Vec<Statement>,
    pub style_block: Option<StyleBlock>,
    pub transition_block: Option<TransitionBlock>,
    pub events: Vec<EventHandler>,
    /// The named slots a call fills: `trailing { … }` in the element's block.
    pub slot_fills: Vec<SlotFill>,

    // ── Source spans (additive; every code generator ignores these) ──
    /// Whole-node span: the component name through the closing `}` — or through
    /// the end of the args / component name when there is no block.
    pub span: Span,
    /// The `( … )` argument + modifier group, parentheses included. `None` when
    /// the element is written with no parentheses.
    pub paren_span: Option<Span>,
    /// Interior of the `{ … }` body, exclusive of the braces. `None` when the
    /// element has no block.
    pub body_span: Option<Span>,
    /// The `style { … }` block: the `style` keyword through its closing `}`.
    /// `None` when the element has no style block.
    pub style_span: Option<Span>,
    /// Per-argument spans, parallel to and index-aligned with `args`.
    pub arg_spans: Vec<Span>,
    /// Per-modifier spans, parallel to and index-aligned with `modifiers`.
    pub modifier_spans: Vec<Span>,
}

impl UIElement {
    /// The slot this element stands for, when it is a slot use: the pseudo
    /// element `Children` names the default slot, or a named one through
    /// its `slot:` argument.
    pub fn slot_name(&self) -> Option<&str> {
        if !matches!(&self.component, ComponentRef::BuiltIn(n) if n == "Children") {
            return None;
        }
        Some(
            self.args
                .iter()
                .find_map(|a| match a {
                    Arg::Named(k, Expr::StringLiteral(name)) if k == "slot" => Some(name.as_str()),
                    _ => None,
                })
                .unwrap_or("children"),
        )
    }
}

/// A named slot filled at a call site: `header { Badge("Beta") }`.
#[derive(Debug, Clone)]
pub struct SlotFill {
    pub name: String,
    pub body: Vec<Statement>,
    pub span: Span,
    pub body_span: Span,
}

#[derive(Debug, Clone)]
pub enum ComponentRef {
    // Built-in components like Container, Row, Button, etc.
    BuiltIn(String),
    // Sub-components like Navbar.Brand, Card.Header
    SubComponent(String, String),
    // User-defined components referenced by name
    UserDefined(String),
}

#[derive(Debug, Clone)]
pub enum Arg {
    Positional(Expr),
    Named(String, Expr),
}

#[derive(Debug, Clone)]
pub struct StyleBlock {
    pub properties: Vec<StyleProperty>,
    pub media_queries: Vec<MediaQuery>,
    /// `hover { … }`, `focus { … }` and the other pseudo-state blocks, in
    /// source order. Compiled to stylesheet rules, since an inline style
    /// cannot express a state.
    pub pseudo_blocks: Vec<PseudoBlock>,
    /// Interior of the `style { … }` block (between the braces, exclusive) —
    /// where a new property is inserted. Additive; codegen ignores it.
    pub body_span: Span,
}

#[derive(Debug, Clone)]
pub struct StyleProperty {
    pub name: String,
    pub value: Expr,
    /// Whole `name: value` span (for removal).
    pub span: Span,
    /// The value expression's span (for value replacement).
    pub value_span: Span,
}

#[derive(Debug, Clone)]
pub struct MediaQuery {
    pub condition: String,
    pub properties: Vec<StyleProperty>,
    /// The whole block, `@media (…) { … }`.
    pub span: Span,
}

/// A pseudo-state block inside `style { }`: the state's name as written
/// (`hover`, `focus`, `active`, `disabled`, `placeholder`, `focus-within`)
/// and the declarations that apply while the element is in it.
#[derive(Debug, Clone)]
pub struct PseudoBlock {
    pub state: String,
    pub properties: Vec<StyleProperty>,
    /// The whole block, `hover { … }`.
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub event: String, // click, submit, input, change, etc.
    /// The handler's named event parameter (`on click(e)`); `None` for the
    /// original grammar, whose handlers read an implicit `event`.
    pub param: Option<String>,
    pub body: Vec<Statement>,
    /// The whole handler, from `on` to the closing brace.
    pub span: Span,
}

// ─── Animation ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AnimateConfig {
    pub enter: String,
    pub exit: Option<String>,
    pub duration: Option<String>,
    pub delay: Option<String>,
    pub stagger: Option<String>,
    pub easing: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TransitionBlock {
    pub properties: Vec<TransitionProperty>,
    /// The whole block, `transition { … }`.
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TransitionProperty {
    pub property: String,
    pub duration: String,
    pub easing: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AnimateStmt {
    pub target: String,
    pub animation: String,
    pub duration: Option<String>,
}

// ─── Control Flow ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    /// `if let name = condition { … }`: the name bound to the condition's
    /// value inside the branch, which runs when the value is not null.
    pub binding: Option<String>,
    pub animate: Option<AnimateConfig>,
    /// The `, animate(…)` clause as written, when there is one.
    pub animate_span: Option<Span>,
    pub then_body: Vec<Statement>,
    pub else_if_branches: Vec<(Expr, Vec<Statement>)>,
    pub else_body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub item: String,
    pub index: Option<String>,
    pub iterable: Expr,
    /// `for x in xs by key`: the expression that identifies an item across
    /// renders.
    pub key: Option<Expr>,
    pub animate: Option<AnimateConfig>,
    /// The `, animate(…)` clause as written, when there is one.
    pub animate_span: Option<Span>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ShowStmt {
    pub condition: Expr,
    pub animate: Option<AnimateConfig>,
    /// The `, animate(…)` clause as written, when there is one.
    pub animate_span: Option<Span>,
    pub body: Vec<Statement>,
}

// ─── Data Fetching ───────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FetchDecl {
    pub variable: String,
    pub url: Expr,
    pub options: Vec<FetchOption>,
    pub loading_block: Option<Vec<Statement>>,
    pub error_block: Option<(String, Vec<Statement>)>, // (error_var_name, body)
    pub success_block: Option<Vec<Statement>>,
    /// The URL expression, and the options list with its parentheses.
    pub url_span: Span,
    pub options_span: Option<Span>,
    /// Each clause as written — `loading { … }`, `error(e) { … }`,
    /// `success { … }` — for a tool that rewrites them.
    pub loading_span: Option<Span>,
    pub error_span: Option<Span>,
    pub success_span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct FetchOption {
    pub key: String,
    pub value: Expr,
}

// ─── Assignments ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: Expr,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct MethodCallStmt {
    pub object: Expr,
    pub method: String,
    pub args: Vec<Expr>,
}

// ─── Expressions ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    StringLiteral(String),
    InterpolatedString(Vec<StringPart>),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Null,

    // Identifiers & access
    Identifier(String),
    PropertyAccess(Box<Expr>, String),
    IndexAccess(Box<Expr>, Box<Expr>),

    // Operations
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),

    // Calls
    MethodCall(Box<Expr>, String, Vec<Expr>),
    FunctionCall(String, Vec<Expr>),

    // Collections
    ListLiteral(Vec<Expr>),
    MapLiteral(Vec<(String, Expr)>),

    // Lambda
    Lambda(String, Box<Expr>),

    /// `.case`: a member of whichever enum the position expects.
    EnumCase(String),
    /// `$name`: a design token, `var(--name)` on the web.
    Token(String),
    /// `await expr`, inside an action or a handler.
    Await(Box<Expr>),
}

impl Expr {
    /// The expressions directly inside this one.
    pub fn children(&self) -> Vec<&Expr> {
        match self {
            Expr::InterpolatedString(parts) => parts
                .iter()
                .filter_map(|p| match p {
                    StringPart::Expression(e) => Some(e),
                    StringPart::Literal(_) => None,
                })
                .collect(),
            Expr::PropertyAccess(base, _) => vec![base],
            Expr::IndexAccess(base, index) => vec![base, index],
            Expr::BinaryOp(l, _, r) => vec![l, r],
            Expr::UnaryOp(_, e) | Expr::Lambda(_, e) | Expr::Await(e) => vec![e],
            Expr::MethodCall(obj, _, args) => std::iter::once(&**obj).chain(args).collect(),
            Expr::FunctionCall(_, args) | Expr::ListLiteral(args) => args.iter().collect(),
            Expr::MapLiteral(pairs) => pairs.iter().map(|(_, v)| v).collect(),
            Expr::StringLiteral(_)
            | Expr::NumberLiteral(_)
            | Expr::BoolLiteral(_)
            | Expr::Null
            | Expr::Identifier(_)
            | Expr::EnumCase(_)
            | Expr::Token(_) => Vec::new(),
        }
    }

    /// Whether this expression, at any depth, is or holds an `await`.
    pub fn contains_await(&self) -> bool {
        matches!(self, Expr::Await(_)) || self.children().iter().any(|c| c.contains_await())
    }
}

impl StatementKind {
    /// The expressions this statement holds directly (not those of nested
    /// statements).
    pub fn exprs(&self) -> Vec<&Expr> {
        match self {
            StatementKind::State(s) => vec![&s.value],
            StatementKind::Derived(d) => vec![&d.value],
            StatementKind::If(i) => std::iter::once(&i.condition)
                .chain(i.else_if_branches.iter().map(|(c, _)| c))
                .collect(),
            StatementKind::For(f) => std::iter::once(&f.iterable).chain(f.key.iter()).collect(),
            StatementKind::Show(s) => vec![&s.condition],
            StatementKind::Fetch(f) => std::iter::once(&f.url)
                .chain(f.options.iter().map(|o| &o.value))
                .collect(),
            StatementKind::Assignment(a) => vec![&a.target, &a.value],
            StatementKind::MethodCall(m) => std::iter::once(&m.object).chain(&m.args).collect(),
            StatementKind::Navigate(e)
            | StatementKind::Log(e)
            | StatementKind::ExprStatement(e) => {
                vec![e]
            }
            StatementKind::Return(e) => e.iter().collect(),
            StatementKind::Resource(r) => std::iter::once(&r.url)
                .chain(r.options.iter().map(|o| &o.value))
                .collect(),
            StatementKind::Match(m) => vec![&m.scrutinee],
            StatementKind::Emit(e) => e.args.iter().collect(),
            StatementKind::UIElement(el) => el
                .args
                .iter()
                .map(|a| match a {
                    Arg::Positional(e) | Arg::Named(_, e) => e,
                })
                .collect(),
            StatementKind::Effect(_)
            | StatementKind::Action(_)
            | StatementKind::Use(_)
            | StatementKind::EventHandler(_)
            | StatementKind::Animate(_) => Vec::new(),
        }
    }
}

/// Whether any statement in `stmts`, at any depth, awaits something.
pub fn awaits(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| {
        stmt.kind.exprs().iter().any(|e| e.contains_await())
            || match &stmt.kind {
                StatementKind::If(i) => {
                    awaits(&i.then_body)
                        || i.else_if_branches.iter().any(|(_, b)| awaits(b))
                        || i.else_body.as_deref().is_some_and(awaits)
                }
                StatementKind::For(f) => awaits(&f.body),
                StatementKind::Show(s) => awaits(&s.body),
                StatementKind::Match(m) => m.arms.iter().any(|a| awaits(&a.body)),
                _ => false,
            }
    })
}

#[derive(Debug, Clone)]
pub enum StringPart {
    Literal(String),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    /// `a ?? b`: `b` when `a` is null.
    NullCoalesce,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
    Neg,
}
