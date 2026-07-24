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
        Self { start, end, line, col }
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
    pub body: Vec<Statement>,

    // ── Source spans (additive) ──
    /// Whole declaration, `Page` keyword through the closing `}`.
    pub span: Span,
    /// The header — `Page Name (…)` — up to (excluding) the body's opening `{`.
    pub header_span: Span,
    /// Interior of the `{ … }` body, exclusive of the braces.
    pub body_span: Span,
}

// ─── Components ──────────────────────────────────────────

/// A reusable component: `Component Name (props...) { ... }`.
#[derive(Debug, Clone)]
pub struct ComponentDecl {
    pub name: String,
    pub props: Vec<PropDecl>,
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
    pub prop_type: WfType,
    pub optional: bool,
    pub default: Option<Expr>,
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

/// WebFluent's type system for component props.
#[derive(Debug, Clone, PartialEq)]
pub enum WfType {
    String,
    Number,
    Bool,
    List,
    Map,
}

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
}

// ─── State declarations ─────────────────────────────────

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: String,
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
    pub param_type: WfType,
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
}

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub event: String, // click, submit, input, change, etc.
    pub body: Vec<Statement>,
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
    pub animate: Option<AnimateConfig>,
    pub then_body: Vec<Statement>,
    pub else_if_branches: Vec<(Expr, Vec<Statement>)>,
    pub else_body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub item: String,
    pub index: Option<String>,
    pub iterable: Expr,
    pub animate: Option<AnimateConfig>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ShowStmt {
    pub condition: Expr,
    pub animate: Option<AnimateConfig>,
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
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
    Neg,
}
