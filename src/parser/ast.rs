/// All AST node types for the WebFluent language.

#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Page(PageDecl),
    Component(ComponentDecl),
    Store(StoreDecl),
    App(AppDecl),
}

// ─── Pages ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PageDecl {
    pub name: String,
    pub path: String,
    pub title: Option<String>,
    pub guard: Option<Expr>,
    pub redirect: Option<String>,
    pub body: Vec<Statement>,
}

// ─── Components ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ComponentDecl {
    pub name: String,
    pub props: Vec<PropDecl>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct PropDecl {
    pub name: String,
    pub prop_type: WfType,
    pub optional: bool,
    pub default: Option<Expr>,
}

// ─── Stores ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StoreDecl {
    pub name: String,
    pub body: Vec<Statement>,
}

// ─── App ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AppDecl {
    pub body: Vec<Statement>,
}

// ─── Types ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum WfType {
    String,
    Number,
    Bool,
    List,
    Map,
}

// ─── Statements ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Statement {
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
}

#[derive(Debug, Clone)]
pub struct StyleProperty {
    pub name: String,
    pub value: Expr,
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
