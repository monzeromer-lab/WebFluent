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
// A page carries its header, its route parameters and its body; the tree is
// walked, not moved, so the size difference costs nothing.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Declaration {
    /// `const NAME = value`.
    Const(ConstDecl),
    /// `animation Name { … }`.
    Animation(AnimationDecl),
    /// `test "…" { … }`.
    Test(TestDecl),
    /// `data posts = "posts.json"`.
    Data(DataDecl),
    Page(PageDecl),
    Component(ComponentDecl),
    Store(StoreDecl),
    App(AppDecl),
    Theme(ThemeDecl),
    /// A record type: `type Todo { id: String, done: Bool = false }`.
    Type(TypeDecl),
    /// An enumeration: `enum Tone { neutral, info, danger }`.
    Enum(EnumDecl),
    /// A service, described once: `api Backend(base: "/api") { … }`.
    Api(ApiDecl),
    /// Somebody else's code, described so the compiler can check every use
    /// of it: `external Chart from "chart.js" { fn Chart(…) -> Handle }`.
    External(ExternalDecl),
}

// ─── Somebody else's code ────────────────────────────────

/// `external Name from "specifier" { … }` — a module the build imports, or
/// `external element Name("tag-name") { … }` — a custom element the page
/// places.
///
/// Either way it is a **description**: the compiler cannot read the other
/// side, so what is written here is what every call site is checked
/// against. It is the difference between reaching another library and
/// reaching for `window.X` and hoping.
#[derive(Debug, Clone)]
pub struct ExternalDecl {
    pub name: String,
    /// What the declaration is of.
    pub kind: ExternalKind,
    /// The module specifier, or the custom element's tag name.
    pub from: String,
    /// `integrity: "sha384-…"` for a module served from another origin.
    pub integrity: Option<String>,
    /// `fn name(args) -> T` — what the module has.
    pub functions: Vec<ExternalFn>,
    /// `type Handle { m(a: T), field: T }` — the shapes it hands back.
    pub types: Vec<ExternalType>,
    /// `prop name: T` — what a custom element takes.
    pub props: Vec<PropDecl>,
    /// `event name(args)` — what a custom element fires.
    pub events: Vec<EventDecl>,
    pub doc: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalKind {
    /// A module the build imports.
    Module,
    /// A custom element the page places by its tag name.
    Element,
}

/// `fn Chart(canvas: Any, config: Map) -> ChartHandle`
#[derive(Debug, Clone)]
pub struct ExternalFn {
    pub name: String,
    pub params: Vec<PropDecl>,
    pub returns: Option<TypeRef>,
    pub doc: Option<String>,
    pub span: Span,
}

/// `type ChartHandle { update(data: Map), destroy() }`
#[derive(Debug, Clone)]
pub struct ExternalType {
    pub name: String,
    /// The methods it has, and what each gives back.
    pub methods: Vec<ExternalFn>,
    /// The fields it has.
    pub fields: Vec<FieldDecl>,
    pub span: Span,
}

// ─── A service ───────────────────────────────────────────

/// `api Backend(base: env.API) { … }` — where a service is, how it is
/// reached, and what it has.
///
/// One declaration, so every call site is typed, cached and cancellable,
/// and the day the server changes its contract the build says so.
#[derive(Debug, Clone)]
pub struct ApiDecl {
    pub name: String,
    /// `base:`, and the other settings written in the header.
    pub settings: Vec<(String, Expr)>,
    /// `headers { Authorization: "…" }`.
    pub headers: Vec<(String, Expr)>,
    /// `on request(r) { … }`, `on response(r)`, `on error(e)`.
    pub hooks: Vec<EventHandler>,
    pub endpoints: Vec<Endpoint>,
    pub doc: Option<String>,
    pub span: Span,
    /// The file it is read from, when the surface is imported: `api B from
    /// "openapi.json"`.
    pub from: Option<String>,
}

/// One thing a service has: `get users(page: Number = 1) -> [User]`.
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// `get`, `post`, `put`, `patch`, `delete`, `head`, `options`.
    pub method: String,
    pub name: String,
    /// The path, with `:name` where a parameter goes. Written after the
    /// name when it differs from it: `get user(id: String) at "users/:id"`.
    pub path: String,
    pub params: Vec<PropDecl>,
    /// What comes back, and how to read it.
    pub returns: Option<TypeRef>,
    /// `errors { 422 -> ValidationErrors }`: the body of a failure, typed.
    pub errors: Vec<(u16, TypeRef)>,
    /// What the endpoint says about itself: `cache`, `as`, `progress`.
    pub settings: Vec<(String, Expr)>,
    pub doc: Option<String>,
    pub span: Span,
}

// ─── Types ───────────────────────────────────────────────

/// A record type declaration. Its fields are typed like a component's props.
#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    /// `type Admin = User { role: String }`: the record this one extends;
    /// its fields come first, and a field of the same name here replaces it.
    pub extends: Option<String>,
    pub fields: Vec<FieldDecl>,
    /// The `///` comment above it.
    pub doc: Option<String>,
    pub span: Span,
    pub header_span: Span,
}

impl TypeDecl {
    /// Every field, the extended record's first, resolved through `lookup`
    /// (a cycle stops where it started).
    pub fn all_fields<'a>(
        &'a self,
        lookup: &dyn Fn(&str) -> Option<&'a TypeDecl>,
    ) -> Vec<&'a FieldDecl> {
        let mut seen = vec![self.name.as_str()];
        let mut chain: Vec<&'a TypeDecl> = vec![self];
        let mut base = self.extends.as_deref();
        while let Some(name) = base {
            if seen.contains(&name) {
                break;
            }
            seen.push(name);
            match lookup(name) {
                Some(decl) => {
                    chain.push(decl);
                    base = decl.extends.as_deref();
                }
                None => break,
            }
        }
        let mut out: Vec<&'a FieldDecl> = Vec::new();
        for decl in chain.iter().rev() {
            for field in &decl.fields {
                match out.iter().position(|f| f.name == field.name) {
                    Some(at) => out[at] = field,
                    None => out.push(field),
                }
            }
        }
        out
    }
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
    pub cases: Vec<EnumCase>,
    pub doc: Option<String>,
    pub span: Span,
    pub header_span: Span,
}

/// One case of an enum: a bare name, or a name with the payload it
/// carries (`failed(reason: String)`).
#[derive(Debug, Clone)]
pub struct EnumCase {
    pub name: String,
    pub fields: Vec<FieldDecl>,
}

impl EnumDecl {
    /// The case names, in declaration order.
    pub fn case_names(&self) -> Vec<String> {
        self.cases.iter().map(|c| c.name.clone()).collect()
    }

    /// The case called `name`, if there is one.
    pub fn case(&self, name: &str) -> Option<&EnumCase> {
        self.cases.iter().find(|c| c.name == name)
    }
}

/// A reference to a type, as written after a colon.
#[derive(Debug, Clone)]
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
    /// A type with a condition on its values: `Number(0..=100)`,
    /// `String(minLength: 8)`, `Date(after: @2026-01-01)`.
    ///
    /// The condition is what the checker holds a literal to, and what
    /// validation reads to say why a value was refused.
    Refined(Box<TypeRef>, Vec<(String, Expr)>),
}

/// Two types are the same type when they are written the same way; a
/// condition's values are expressions, which compare by how they read.
impl PartialEq for TypeRef {
    fn eq(&self, other: &TypeRef) -> bool {
        match (self, other) {
            (TypeRef::String, TypeRef::String)
            | (TypeRef::Number, TypeRef::Number)
            | (TypeRef::Bool, TypeRef::Bool)
            | (TypeRef::Map, TypeRef::Map)
            | (TypeRef::Any, TypeRef::Any) => true,
            (TypeRef::List(a), TypeRef::List(b)) | (TypeRef::Optional(a), TypeRef::Optional(b)) => {
                a == b
            }
            (TypeRef::Named(a), TypeRef::Named(b)) => a == b,
            (TypeRef::Refined(a, x), TypeRef::Refined(b, y)) => {
                a == b && x.len() == y.len() && x.iter().zip(y).all(|((n, _), (m, _))| n == m)
            }
            _ => false,
        }
    }
}

impl TypeRef {
    /// The type without its condition.
    pub fn base(&self) -> &TypeRef {
        match self {
            TypeRef::Refined(inner, _) => inner.base(),
            other => other,
        }
    }

    /// What it says about its values, innermost last.
    pub fn refinement(&self) -> &[(String, Expr)] {
        match self {
            TypeRef::Refined(_, args) => args,
            _ => &[],
        }
    }
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

/// `const API = "/api"`: a value every page, component and store reads.
#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub ty: Option<TypeRef>,
    pub value: Expr,
    pub doc: Option<String>,
    pub span: Span,
}

/// `data posts = "posts.json"`: a value read from a file at build time —
/// a constant whose value is the file's JSON. The build resolves it into a
/// `const` before anything else runs.
#[derive(Debug, Clone)]
pub struct DataDecl {
    pub name: String,
    pub ty: Option<TypeRef>,
    /// The file, relative to the project (or its `src/`).
    pub file: String,
    pub doc: Option<String>,
    pub span: Span,
    /// Whether it is `image hero = "media/hero.jpg"` rather than `data`:
    /// read as a picture at build time, not as JSON.
    pub is_image: bool,
}

/// `animation Pulse { from { opacity: 1 } to { opacity: 0.5 } }`: keyframes
/// the project declares, played with `animate: .Pulse` or written into a
/// style as `animation: Pulse 1s infinite`.
#[derive(Debug, Clone)]
pub struct AnimationDecl {
    pub name: String,
    pub frames: Vec<KeyFrame>,
    pub doc: Option<String>,
    pub span: Span,
}

/// One keyframe: `from`, `to`, or percentages such as `50%` or `0%, 100%`.
#[derive(Debug, Clone)]
pub struct KeyFrame {
    pub selector: String,
    pub properties: Vec<StyleProperty>,
}

/// `test "name"(data: { … }) { elements  expect "text" }`: a render of the
/// body over the data, held to what it must and must not contain and to a
/// snapshot. `wf test` runs them; a build ignores them.
#[derive(Debug, Clone)]
pub struct TestDecl {
    pub name: String,
    /// The data the body renders over, a map literal.
    pub data: Option<Expr>,
    pub body: Vec<Statement>,
    /// What the test does and what it expects, in the order written.
    ///
    /// A test that only expects is rendered; one that acts is run in a
    /// browser, because a click is not something a renderer can do.
    pub steps: Vec<Step>,
    pub span: Span,
}

impl TestDecl {
    /// Whether this test does something, rather than only looking.
    pub fn acts(&self) -> bool {
        self.steps.iter().any(|s| !matches!(s, Step::Expect { .. }))
    }
}

/// One line of a test: what it does, or what it expects to see.
#[derive(Debug, Clone)]
pub enum Step {
    /// `expect "text"`, or `expect not "text"`.
    Expect {
        text: Expr,
        negated: bool,
        span: Span,
    },
    /// `click "Save"` — whatever carries that name.
    Click { target: Expr, span: Span },
    /// `type "Ada" into "Name"` — into the control that label names.
    Type { text: Expr, into: Expr, span: Span },
    /// `press "Enter"`, or `press "Escape" in "Search"`.
    Press {
        key: Expr,
        target: Option<Expr>,
        span: Span,
    },
}

impl Step {
    pub fn span(&self) -> Span {
        match self {
            Step::Expect { span, .. }
            | Step::Click { span, .. }
            | Step::Type { span, .. }
            | Step::Press { span, .. } => *span,
        }
    }
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
    /// The route's parameters as typed props: `page Deploy(path:
    /// "/deploys/:id", id: String)` binds `:id` to `id`.
    pub params: Vec<PropDecl>,
    /// `head { meta(…) link(…) script(…) }`: tags of the page's own in the
    /// document's head.
    pub head: Vec<HeadTag>,
    /// `paths: posts.map(p => p.slug)` on a page with a `:param` route: the
    /// values the static build renders a page for.
    pub paths: Option<Expr>,

    pub body: Vec<Statement>,

    // ── Source spans (additive) ──
    /// Whole declaration, `Page` keyword through the closing `}`.
    pub span: Span,
    /// The header — `Page Name (…)` — up to (excluding) the body's opening `{`.
    pub header_span: Span,
    /// Interior of the `{ … }` body, exclusive of the braces.
    pub body_span: Span,
}

/// One tag of a page's `head { }`: `meta(name: "x", content: y)`.
#[derive(Debug, Clone)]
pub struct HeadTag {
    pub tag: String,
    pub attrs: Vec<(String, Expr)>,
    pub span: Span,
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
    /// The parts it declares (`part Header(…) { … }`): each is a component
    /// of its own, named `Owner.Part`, declared beside it.
    pub parts: Vec<String>,
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
    /// What the component hands the fill: `slot row(item: Todo)`.
    pub params: Vec<ParamDecl>,
    pub span: Span,
}

// ─── Stores ──────────────────────────────────────────────

/// A shared state store: `Store Name { state ..., derived ..., action ... }`.
#[derive(Debug, Clone)]
pub struct StoreDecl {
    pub name: String,
    /// `store Cart(scope: .route)` — how long what it holds lives.
    pub scope: StoreScope,
    /// `store Cart(eager: true)` — built at boot rather than on first read.
    pub eager: bool,
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
    Timer(TimerStmt),
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
    /// `socket chat = ws(…)`, `stream ticks = sse(…)`, `channel c = broadcast(…)`.
    Connection(ConnectionDecl),
    /// `validate email { required  email }`.
    Validate(ValidateDecl),
    /// `match rows { loading { … } error(e) { … } ready(v) { … } }`.
    Match(MatchStmt),
    /// `emit toggle(id)`: fire a declared event.
    Emit(EmitStmt),
    /// `try { … } catch e { … }` in an action or a handler.
    Try(TryStmt),
}

#[derive(Debug, Clone)]
pub struct TryStmt {
    pub body: Vec<Statement>,
    /// The name the error is bound to in the catch block, when written.
    pub param: Option<String>,
    pub catch_body: Vec<Statement>,
}

/// An async resource: the request, declared once and rendered anywhere.
#[derive(Debug, Clone)]
pub struct ResourceDecl {
    pub name: String,
    pub ty: Option<TypeRef>,
    pub url: Expr,
    pub options: Vec<FetchOption>,
}

/// What a value must be for a form to accept it, declared beside the state
/// it guards.
#[derive(Debug, Clone)]
pub struct ValidateDecl {
    /// The state it guards.
    pub name: String,
    pub rules: Vec<Rule>,
    pub span: Span,
}

/// One rule: `required`, `minLength(8) "Use at least 8"`, `custom "…" { expr }`.
#[derive(Debug, Clone)]
pub struct Rule {
    /// `required`, `email`, `url`, `minLength`, `maxLength`, `min`, `max`,
    /// `pattern`, `matches`, `oneOf`, `custom`, `async`.
    pub name: String,
    pub args: Vec<Expr>,
    /// The message shown when it does not hold; the rule's own when none
    /// is written, which the project's translations may replace.
    pub message: Option<Expr>,
    /// What `custom` and `async` check.
    pub body: Option<Expr>,
    pub span: Span,
}

/// What a connection is: a socket, a stream of events, or a channel every
/// tab of the origin hears.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionKind {
    /// `socket chat = ws("wss://…") { … }`.
    Socket,
    /// `stream ticks = sse("/events")`.
    Stream,
    /// `channel cart = broadcast("cart")`.
    Channel,
    /// `peer link = rtc(signal: send)` — a WebRTC data channel to another
    /// page, signalled over whatever transport the author already has.
    Peer,
}

/// A connection the page holds open, closed when the page leaves.
#[derive(Debug, Clone)]
pub struct ConnectionDecl {
    pub kind: ConnectionKind,
    pub name: String,
    /// The address a socket, a stream or a channel opens; a peer has none —
    /// what reaches it comes through its signalling.
    pub url: Option<Expr>,
    /// `protocols:`, `reconnect:`, `heartbeat:`, `events:`, `resume:`.
    pub options: Vec<(String, Expr)>,
    /// `send Outgoing` and `receive Incoming`, when they are declared.
    pub sends: Option<TypeRef>,
    pub receives: Option<TypeRef>,
    /// `on message(m) { … }`: what arrives, handled where it is opened.
    pub handlers: Vec<EventHandler>,
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
    /// The names a `.case(a, b)` arm binds to the case's payload, in order.
    pub bindings: Vec<String>,
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
    /// A state a connection is in: `connecting`, `open`, `closed(c)`.
    State(String),
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
    /// `persist name = value`: kept in the browser's storage across visits.
    pub persist: bool,
    /// The block a `persist` may carry: where it is written, what version
    /// its shape is, whether other tabs are followed, and how a value an
    /// older build left is brought forward.
    pub policy: Option<PersistPolicy>,
}

/// How long a store's state lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StoreScope {
    /// Built once, for as long as the page is open. The default.
    #[default]
    App,
    /// This tab's: what it keeps defaults to the tab's own storage.
    Session,
    /// The route's: dropped when the route changes, built again empty.
    Route,
}

impl StoreScope {
    pub fn as_str(self) -> &'static str {
        match self {
            StoreScope::App => "app",
            StoreScope::Session => "session",
            StoreScope::Route => "route",
        }
    }
}

/// `persist items = [] { in: .session  version: 2  migrate 1 -> 2 { … } }`
#[derive(Debug, Clone, Default)]
pub struct PersistPolicy {
    /// `.local` (the default, or the store's scope) or `.session`.
    pub storage: Option<String>,
    /// The version of the shape this build writes. Without one, the value
    /// is stored as it stands and no migration is possible.
    pub version: Option<u32>,
    /// Whether a write in another tab is adopted here. On by default for
    /// `.local`, which is shared between tabs by definition.
    pub sync: Option<bool>,
    /// `migrate 1 -> 2 { old.map(…) }`, in the order written.
    pub migrations: Vec<Migration>,
    /// The whole block, for an editor.
    pub span: Span,
}

/// One step forward: what a value of `from` becomes at `to`, reading the
/// old value as `old`.
#[derive(Debug, Clone)]
pub struct Migration {
    pub from: u32,
    pub to: u32,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DerivedDecl {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct EffectDecl {
    pub body: Vec<Statement>,
    /// `cleanup { … }` at the end of the body: run before the effect runs
    /// again, and when what it belongs to leaves.
    pub cleanup: Vec<Statement>,
}

/// `every(ms) { … }` or `after(ms) { … }`: a timer that stops with the
/// page, component, branch or item it is declared in.
#[derive(Debug, Clone)]
pub struct TimerStmt {
    /// `every` repeats; `after` fires once.
    pub every: bool,
    pub interval: Expr,
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
    /// The names the fill gives the slot's values, in the slot's order:
    /// `row(item) { … }`.
    pub params: Vec<String>,
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
    /// `on key("ctrl+k")`: the key the handler answers to.
    pub key: Option<String>,
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
    /// `/pattern/flags`.
    Regex(String, String),
    InterpolatedString(Vec<StringPart>),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Null,

    // Identifiers & access
    Identifier(String),
    PropertyAccess(Box<Expr>, String),
    IndexAccess(Box<Expr>, Box<Expr>),
    /// `a?.b`: `null` when `a` is null, else `a.b`. The chain after it
    /// short-circuits as JavaScript's does.
    OptionalProperty(Box<Expr>, String),
    /// `a?.m(args)`.
    OptionalMethod(Box<Expr>, String, Vec<Expr>),
    /// `a?.[i]`.
    OptionalIndex(Box<Expr>, Box<Expr>),

    // Operations
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),

    // Calls
    MethodCall(Box<Expr>, String, Vec<Expr>),
    FunctionCall(String, Vec<Expr>),

    // Collections
    ListLiteral(Vec<Expr>),
    /// `{ key: value }`; an entry keyed `"..."` is a spread of its value.
    MapLiteral(Vec<(String, Expr)>),
    /// `...list` inside a list literal: its items, in place.
    Spread(Box<Expr>),
    /// `a..b` (exclusive) or `a..=b` (inclusive): the numbers from `a`.
    Range(Box<Expr>, Box<Expr>, bool),
    /// `Todo(id: 1, title: "x")`: a record of a declared `type`, built
    /// with named fields. Emitted as a map; typed by its name.
    Record(String, Vec<(String, Expr)>),

    // Lambda
    Lambda(String, Box<Expr>),

    /// `.case`: a member of whichever enum the position expects.
    EnumCase(String),
    /// `.case(a, b)`: a case carrying its payload, `["case", a, b]` on the
    /// web.
    CaseValue(String, Vec<Expr>),
    /// `$name`: a design token, `var(--name)` on the web.
    Token(String),
    /// `await expr`, inside an action or a handler.
    Await(Box<Expr>),
    /// A literal of one of the language's own types: `@2026-03-14` is a
    /// `Date` carried by `"2026-03-14"`, `3.days` a `Duration` carried by
    /// `259200000`, `€12.99` a `Money` carried by a small map.
    ///
    /// The name is the type; the expression inside is the carrier, and is
    /// what every backend emits — so a scalar is a plain JSON value
    /// wherever it goes, and only the checker knows more.
    Typed(String, Box<Expr>),
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
            Expr::PropertyAccess(base, _) | Expr::OptionalProperty(base, _) => vec![base],
            Expr::IndexAccess(base, index) | Expr::OptionalIndex(base, index) => {
                vec![base, index]
            }
            Expr::BinaryOp(l, _, r) => vec![l, r],
            Expr::UnaryOp(_, e)
            | Expr::Lambda(_, e)
            | Expr::Await(e)
            | Expr::Spread(e)
            | Expr::Typed(_, e) => vec![e],
            Expr::Range(a, b, _) => vec![a, b],
            Expr::MethodCall(obj, _, args) | Expr::OptionalMethod(obj, _, args) => {
                std::iter::once(&**obj).chain(args).collect()
            }
            Expr::FunctionCall(_, args) | Expr::ListLiteral(args) | Expr::CaseValue(_, args) => {
                args.iter().collect()
            }
            Expr::MapLiteral(pairs) | Expr::Record(_, pairs) => {
                pairs.iter().map(|(_, v)| v).collect()
            }
            Expr::StringLiteral(_)
            | Expr::Regex(..)
            | Expr::NumberLiteral(_)
            | Expr::BoolLiteral(_)
            | Expr::Null
            | Expr::Identifier(_)
            | Expr::EnumCase(_)
            | Expr::Token(_) => Vec::new(),
        }
    }

    /// The expressions directly inside this one, to change.
    pub fn children_mut(&mut self) -> Vec<&mut Expr> {
        match self {
            Expr::InterpolatedString(parts) => parts
                .iter_mut()
                .filter_map(|p| match p {
                    StringPart::Expression(e) => Some(e),
                    StringPart::Literal(_) => None,
                })
                .collect(),
            Expr::PropertyAccess(base, _) | Expr::OptionalProperty(base, _) => vec![base],
            Expr::IndexAccess(base, index) | Expr::OptionalIndex(base, index) => {
                vec![base, index]
            }
            Expr::BinaryOp(l, _, r) => vec![l, r],
            Expr::UnaryOp(_, e)
            | Expr::Lambda(_, e)
            | Expr::Await(e)
            | Expr::Spread(e)
            | Expr::Typed(_, e) => vec![e],
            Expr::Range(a, b, _) => vec![a, b],
            Expr::MethodCall(obj, _, args) | Expr::OptionalMethod(obj, _, args) => {
                std::iter::once(&mut **obj).chain(args).collect()
            }
            Expr::FunctionCall(_, args) | Expr::ListLiteral(args) | Expr::CaseValue(_, args) => {
                args.iter_mut().collect()
            }
            Expr::MapLiteral(pairs) | Expr::Record(_, pairs) => {
                pairs.iter_mut().map(|(_, v)| v).collect()
            }
            Expr::StringLiteral(_)
            | Expr::Regex(..)
            | Expr::NumberLiteral(_)
            | Expr::BoolLiteral(_)
            | Expr::Null
            | Expr::Identifier(_)
            | Expr::EnumCase(_)
            | Expr::Token(_) => Vec::new(),
        }
    }

    /// `f` on this expression and then on every one inside it, at any depth.
    pub fn walk_mut(&mut self, f: &mut dyn FnMut(&mut Expr)) {
        f(self);
        for child in self.children_mut() {
            child.walk_mut(f);
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
    /// The statement blocks nested in this statement, each a body of its
    /// own: an element's children are its own affair.
    pub fn bodies(&self) -> Vec<&[Statement]> {
        match self {
            StatementKind::If(i) => {
                let mut bodies: Vec<&[Statement]> = vec![&i.then_body];
                bodies.extend(i.else_if_branches.iter().map(|(_, b)| b.as_slice()));
                bodies.extend(i.else_body.as_deref());
                bodies
            }
            StatementKind::For(f) => vec![&f.body],
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
            StatementKind::Try(t) => vec![&t.body, &t.catch_body],
            // What a connection does with what arrives.
            StatementKind::Connection(c) => c.handlers.iter().map(|h| h.body.as_slice()).collect(),
            _ => Vec::new(),
        }
    }

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
            StatementKind::Connection(c) => c
                .url
                .iter()
                .chain(c.options.iter().map(|(_, v)| v))
                .collect(),
            StatementKind::Validate(v) => v
                .rules
                .iter()
                .flat_map(|r| r.args.iter().chain(r.message.iter()).chain(r.body.iter()))
                .collect(),
            StatementKind::Match(m) => vec![&m.scrutinee],
            StatementKind::Emit(e) => e.args.iter().collect(),
            StatementKind::Try(_) => Vec::new(),
            StatementKind::Timer(t) => vec![&t.interval],
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

/// `f` on every expression under `stmts`, at any depth: a statement's own,
/// an element's arguments, style splices, handler bodies, fills, children.
pub fn walk_exprs_mut(stmts: &mut [Statement], f: &mut dyn FnMut(&mut Expr)) {
    fn style(block: &mut StyleBlock, f: &mut dyn FnMut(&mut Expr)) {
        for p in &mut block.properties {
            p.value.walk_mut(f);
        }
        for mq in &mut block.media_queries {
            for p in &mut mq.properties {
                p.value.walk_mut(f);
            }
        }
        for pb in &mut block.pseudo_blocks {
            for p in &mut pb.properties {
                p.value.walk_mut(f);
            }
        }
    }
    for stmt in stmts.iter_mut() {
        match &mut stmt.kind {
            StatementKind::State(s) => s.value.walk_mut(f),
            StatementKind::Derived(d) => d.value.walk_mut(f),
            StatementKind::Effect(e) => {
                walk_exprs_mut(&mut e.body, f);
                walk_exprs_mut(&mut e.cleanup, f);
            }
            StatementKind::Timer(t) => {
                t.interval.walk_mut(f);
                walk_exprs_mut(&mut t.body, f);
            }
            StatementKind::Action(a) => walk_exprs_mut(&mut a.body, f),
            StatementKind::UIElement(el) => {
                for arg in &mut el.args {
                    match arg {
                        Arg::Positional(e) | Arg::Named(_, e) => e.walk_mut(f),
                    }
                }
                if let Some(block) = &mut el.style_block {
                    style(block, f);
                }
                for h in &mut el.events {
                    walk_exprs_mut(&mut h.body, f);
                }
                for fill in &mut el.slot_fills {
                    walk_exprs_mut(&mut fill.body, f);
                }
                walk_exprs_mut(&mut el.children, f);
            }
            StatementKind::If(i) => {
                i.condition.walk_mut(f);
                walk_exprs_mut(&mut i.then_body, f);
                for (c, b) in &mut i.else_if_branches {
                    c.walk_mut(f);
                    walk_exprs_mut(b, f);
                }
                if let Some(b) = &mut i.else_body {
                    walk_exprs_mut(b, f);
                }
            }
            StatementKind::For(fs) => {
                fs.iterable.walk_mut(f);
                if let Some(k) = &mut fs.key {
                    k.walk_mut(f);
                }
                walk_exprs_mut(&mut fs.body, f);
            }
            StatementKind::Show(s) => {
                s.condition.walk_mut(f);
                walk_exprs_mut(&mut s.body, f);
            }
            StatementKind::Fetch(fd) => {
                fd.url.walk_mut(f);
                for o in &mut fd.options {
                    o.value.walk_mut(f);
                }
                if let Some(b) = &mut fd.loading_block {
                    walk_exprs_mut(b, f);
                }
                if let Some((_, b)) = &mut fd.error_block {
                    walk_exprs_mut(b, f);
                }
                if let Some(b) = &mut fd.success_block {
                    walk_exprs_mut(b, f);
                }
            }
            StatementKind::Assignment(a) => {
                a.target.walk_mut(f);
                a.value.walk_mut(f);
            }
            StatementKind::MethodCall(m) => {
                m.object.walk_mut(f);
                for a in &mut m.args {
                    a.walk_mut(f);
                }
            }
            StatementKind::EventHandler(h) => walk_exprs_mut(&mut h.body, f),
            StatementKind::Navigate(e)
            | StatementKind::Log(e)
            | StatementKind::ExprStatement(e) => e.walk_mut(f),
            StatementKind::Return(e) => {
                if let Some(e) = e {
                    e.walk_mut(f);
                }
            }
            StatementKind::Resource(r) => {
                r.url.walk_mut(f);
                for o in &mut r.options {
                    o.value.walk_mut(f);
                }
            }
            StatementKind::Validate(v) => {
                for rule in &mut v.rules {
                    for e in rule
                        .args
                        .iter_mut()
                        .chain(rule.message.iter_mut())
                        .chain(rule.body.iter_mut())
                    {
                        e.walk_mut(f);
                    }
                }
            }
            StatementKind::Connection(c) => {
                if let Some(url) = &mut c.url {
                    url.walk_mut(f);
                }
                for (_, v) in &mut c.options {
                    v.walk_mut(f);
                }
                for h in &mut c.handlers {
                    walk_exprs_mut(&mut h.body, f);
                }
            }
            StatementKind::Match(m) => {
                m.scrutinee.walk_mut(f);
                for arm in &mut m.arms {
                    walk_exprs_mut(&mut arm.body, f);
                }
            }
            StatementKind::Emit(e) => {
                for a in &mut e.args {
                    a.walk_mut(f);
                }
            }
            StatementKind::Try(t) => {
                walk_exprs_mut(&mut t.body, f);
                walk_exprs_mut(&mut t.catch_body, f);
            }
            StatementKind::Use(_) | StatementKind::Animate(_) => {}
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
                StatementKind::Try(t) => awaits(&t.body) || awaits(&t.catch_body),
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
