use std::fmt;

/// All token types produced by the WebFluent lexer.
/// How a string literal was written, which decides what its text means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringKind {
    /// `"…"` — escapes, and `{…}` splices.
    Plain,
    /// `#"…"#` — neither. The text is what is written, which is how a
    /// sample of code goes in a string without being spelled twice.
    Raw,
    /// `"""…"""` — escapes and splices, with the indentation the source
    /// gave it removed.
    Block,
}

/// A string literal as it was written: the spelling between its
/// delimiters, with nothing resolved, and which delimiters they were.
///
/// The escapes are resolved by the parser, in the one pass that also
/// splits the splices out — so an escaped brace is a brace from the
/// moment it is understood, and nothing downstream has to know it was
/// ever written `\{`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringLit {
    pub spelling: String,
    pub kind: StringKind,
}

impl StringLit {
    pub fn new(spelling: impl Into<String>, kind: StringKind) -> Self {
        StringLit {
            spelling: spelling.into(),
            kind,
        }
    }

    /// A `"…"` literal.
    pub fn plain(spelling: impl Into<String>) -> Self {
        StringLit::new(spelling, StringKind::Plain)
    }
}

impl From<&str> for StringLit {
    fn from(s: &str) -> Self {
        StringLit::plain(s)
    }
}

impl From<String> for StringLit {
    fn from(s: String) -> Self {
        StringLit::plain(s)
    }
}

/// All token types produced by the WebFluent lexer.
#[derive(Debug, Clone, PartialEq)]
// `EOF` is the conventional spelling in a lexer, and this enum is public.
#[allow(clippy::upper_case_acronyms)]
pub enum TokenType {
    // Top-level declarations
    Page,
    Component,
    Store,
    App,

    // Routing
    Router,
    Route,

    // State & logic
    State,
    Derived,
    Effect,
    Action,
    Use,
    Fetch,
    From,
    Navigate,
    Log,
    Return,

    // Control flow
    If,
    Else,
    For,
    In,
    Show,

    // Fetch blocks
    Loading,
    Error,
    Success,

    // Types
    TypeString,
    TypeNumber,
    TypeBool,
    TypeList,
    TypeMap,

    // Literals
    StringLiteral(StringLit),
    /// `/pattern/flags`: a regular expression, pattern and flags.
    RegexLiteral(String, String),
    /// `@2026-03-14`, `@09:30`, `@2026-03-14T09:30Z` — a date, a time or
    /// both. Which one it is, the parser decides from the text.
    TemporalLiteral(String),
    /// `#0F766E` — a colour, written as CSS writes one.
    ColorLiteral(String),
    /// `€12.99` — an amount and the currency its symbol names.
    MoneyLiteral(String, String),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Null,

    // Layout components
    Container,
    Row,
    Column,
    Grid,
    Stack,
    Spacer,
    Divider,

    // Navigation components
    Navbar,
    Sidebar,
    Breadcrumb,
    Link,
    Menu,
    Tabs,
    TabPage,

    // Data display components
    Card,
    Table,
    Thead,
    Tbody,
    Trow,
    Tcell,
    List,
    Badge,
    Avatar,
    Tooltip,
    Tag,

    // Data input components
    Input,
    Select,
    Option,
    Checkbox,
    Radio,
    Switch,
    Slider,
    Textarea,
    DatePicker,
    FileUpload,
    Form,

    // Feedback components
    Alert,
    Toast,
    Modal,
    Dialog,
    Spinner,
    Progress,
    Skeleton,

    // Action components
    Button,
    IconButton,
    ButtonGroup,
    Dropdown,

    // Media components
    Image,
    Video,
    Audio,
    Icon,
    Carousel,

    // Typography components
    Text,
    Heading,
    Code,
    Blockquote,
    Markdown,

    // Document components (PDF)
    Document,
    Section,
    Paragraph,
    PageBreak,
    Header,
    Footer,

    // Slides components (PDF slide deck)
    Presentation,
    Slide,
    TitleSlide,
    SectionSlide,
    TwoColumn,
    ImageSlide,

    // Style & Animation
    Style,
    Theme,
    Token,
    Animate,
    Transition,

    // Identifiers
    Identifier(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equals,
    DoubleEquals,
    NotEquals,
    StrictNotEqual,
    BitwiseAnd,
    LessThan,
    GreaterThan,
    LessEquals,
    GreaterEquals,
    And,
    Or,
    Not,
    Dot,
    Arrow, // =>

    // Punctuation
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Colon,
    Comma,
    QuestionMark,
    /// `?.`: a member read that is `null` when its base is.
    OptionalChain,
    /// `...`: a spread into a list or a map.
    Ellipsis,
    /// `..`: a range, exclusive of its end.
    DotDot,
    /// `..=`: a range, inclusive of its end.
    DotDotEq,

    // Events
    Event(String), // on:click, on:submit, etc.

    // Special
    Children,
    EOF,

    // ─── WebFluent 3 (the `v2` lexer) ───
    /// `$name`: a design token.
    DesignToken(String),
    /// `??`.
    NullCoalesce,
    /// `;`.
    Semicolon,
    /// A `///` comment, without the slashes.
    DocComment(String),
    /// The property name of a declaration inside a style block.
    StyleProp(String),
    /// The raw CSS text of a declaration's value.
    RawValue(String),
    /// The raw selector of a nested rule inside a style block, before its `{`.
    RawSelector(String),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Page => write!(f, "Page"),
            TokenType::Component => write!(f, "Component"),
            TokenType::Store => write!(f, "Store"),
            TokenType::App => write!(f, "App"),
            TokenType::Router => write!(f, "Router"),
            TokenType::Route => write!(f, "Route"),
            TokenType::State => write!(f, "state"),
            TokenType::Derived => write!(f, "derived"),
            TokenType::Effect => write!(f, "effect"),
            TokenType::Action => write!(f, "action"),
            TokenType::Use => write!(f, "use"),
            TokenType::Fetch => write!(f, "fetch"),
            TokenType::From => write!(f, "from"),
            TokenType::Navigate => write!(f, "navigate"),
            TokenType::Log => write!(f, "log"),
            TokenType::Return => write!(f, "return"),
            TokenType::If => write!(f, "if"),
            TokenType::Else => write!(f, "else"),
            TokenType::For => write!(f, "for"),
            TokenType::In => write!(f, "in"),
            TokenType::Show => write!(f, "show"),
            TokenType::Loading => write!(f, "loading"),
            TokenType::Error => write!(f, "error"),
            TokenType::Success => write!(f, "success"),
            TokenType::StringLiteral(s) => write!(f, "\"{}\"", s.spelling),
            TokenType::TemporalLiteral(t) => write!(f, "@{t}"),
            TokenType::ColorLiteral(c) => write!(f, "#{c}"),
            TokenType::MoneyLiteral(symbol, amount) => write!(f, "{symbol}{amount}"),
            TokenType::NumberLiteral(n) => write!(f, "{}", n),
            TokenType::BoolLiteral(b) => write!(f, "{}", b),
            TokenType::Null => write!(f, "null"),
            TokenType::Identifier(s) => write!(f, "{}", s),
            TokenType::Event(s) => write!(f, "on:{}", s),
            TokenType::EOF => write!(f, "EOF"),
            TokenType::TypeList => write!(f, "List"),
            other => write!(f, "{:?}", other),
        }
    }
}

/// A token with its type and source location.
#[derive(Debug, Clone)]
pub struct Token {
    /// The token type and payload.
    pub token_type: TokenType,
    /// 1-based line number in the source file.
    pub line: usize,
    /// 1-based column number in the source file.
    pub column: usize,
    /// Byte offset of the token's first byte in the source.
    pub offset: usize,
    /// Byte offset one past the token's last byte (exclusive).
    pub end: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self {
            token_type,
            line,
            column,
            // Byte offsets are stamped by the lexer once the token's extent is
            // known (see `Lexer::tokenize`); callers using `new` directly get 0.
            offset: 0,
            end: 0,
        }
    }
}

/// The built-in component this token names, if it names one.
///
/// Built-in component names are lexed as dedicated keyword tokens, which is right
/// in *element* position but wrong everywhere a plain name is expected: a page
/// called `Menu` could be declared but never routed to, because
/// `Route(path: "/menu", page: Menu)` hit the expression parser and failed with
/// "Expected expression, got Menu". Every builtin name was affected — `Menu`,
/// `Card`, `List`, `Table`, `Form`, `Alert`, `Modal`, `Link`, `Text`, `Image`,
/// `Icon`, `Footer`, `Badge` — and `Menu` is the likeliest page name a restaurant
/// site will ever have.
///
/// Every built-in component name the parser accepts as an element, in the
/// language reference's order. `List` is spelled by the `TypeList` token.
pub const ALL_COMPONENT_NAMES: &[&str] = &[
    // Layout
    "Container",
    "Row",
    "Column",
    "Grid",
    "Stack",
    "Spacer",
    "Divider",
    // Navigation
    "Navbar",
    "Sidebar",
    "Breadcrumb",
    "Link",
    "Menu",
    "Tabs",
    "TabPage",
    // Data display
    "Card",
    "Table",
    "Thead",
    "Tbody",
    "Trow",
    "Tcell",
    "List",
    "Badge",
    "Avatar",
    "Tooltip",
    "Tag",
    // Form
    "Input",
    "Select",
    "Option",
    "Checkbox",
    "Radio",
    "Switch",
    "Slider",
    "DatePicker",
    "FileUpload",
    "Textarea",
    "Form",
    // Feedback
    "Alert",
    "Toast",
    "Modal",
    "Dialog",
    "Spinner",
    "Progress",
    "Skeleton",
    // Actions
    "Button",
    "IconButton",
    "ButtonGroup",
    "Dropdown",
    // Media
    "Image",
    "Video",
    "Audio",
    "Icon",
    "Carousel",
    // Typography
    "Text",
    "Heading",
    "Code",
    "Blockquote",
    "Markdown",
    // A node handed to somebody else's code, with a lifetime.
    "Host",
    // `Unsafe.Html(markup)`: the one element whose content is markup.
    // The part lowers to `UnsafeHtml`, which is an IR name, not one a
    // program writes.
    "Unsafe",
    // Document
    "Document",
    "Section",
    "Paragraph",
    "PageBreak",
    "Header",
    "Footer",
    // Slides
    "Presentation",
    "Slide",
    "TitleSlide",
    "SectionSlide",
    "TwoColumn",
    "ImageSlide",
    // Routing
    "Router",
    "Route",
];

/// The parser uses this to accept such a token where a name is expected. Only
/// component keywords are listed: control-flow and declaration keywords (`if`,
/// `for`, `Page`, `style`) stay reserved.
pub fn component_name(token: &TokenType) -> Option<&'static str> {
    // Written as the inverse of the component arms of `keyword_or_identifier`
    // below; the test at the bottom of this file asserts the two stay in step.
    let name = match token {
        // Layout
        TokenType::Container => "Container",
        TokenType::Row => "Row",
        TokenType::Column => "Column",
        TokenType::Grid => "Grid",
        TokenType::Stack => "Stack",
        TokenType::Spacer => "Spacer",
        TokenType::Divider => "Divider",
        // Navigation
        TokenType::Navbar => "Navbar",
        TokenType::Sidebar => "Sidebar",
        TokenType::Breadcrumb => "Breadcrumb",
        TokenType::Link => "Link",
        TokenType::Menu => "Menu",
        TokenType::Tabs => "Tabs",
        TokenType::TabPage => "TabPage",
        // Data display
        TokenType::Card => "Card",
        TokenType::Table => "Table",
        TokenType::Thead => "Thead",
        TokenType::Tbody => "Tbody",
        TokenType::Trow => "Trow",
        TokenType::Tcell => "Tcell",
        TokenType::Badge => "Badge",
        TokenType::Avatar => "Avatar",
        TokenType::Tooltip => "Tooltip",
        TokenType::Tag => "Tag",
        // Data input
        TokenType::Input => "Input",
        TokenType::Select => "Select",
        TokenType::Option => "Option",
        TokenType::Checkbox => "Checkbox",
        TokenType::Radio => "Radio",
        TokenType::Switch => "Switch",
        TokenType::Slider => "Slider",
        TokenType::DatePicker => "DatePicker",
        TokenType::FileUpload => "FileUpload",
        TokenType::Textarea => "Textarea",
        TokenType::Form => "Form",
        // Feedback
        TokenType::Alert => "Alert",
        TokenType::Toast => "Toast",
        TokenType::Modal => "Modal",
        TokenType::Dialog => "Dialog",
        TokenType::Spinner => "Spinner",
        TokenType::Progress => "Progress",
        TokenType::Skeleton => "Skeleton",
        // Actions
        TokenType::Button => "Button",
        TokenType::IconButton => "IconButton",
        TokenType::ButtonGroup => "ButtonGroup",
        TokenType::Dropdown => "Dropdown",
        // Media
        TokenType::Image => "Image",
        TokenType::Video => "Video",
        TokenType::Audio => "Audio",
        TokenType::Icon => "Icon",
        TokenType::Carousel => "Carousel",
        // Typography
        TokenType::Text => "Text",
        TokenType::Heading => "Heading",
        TokenType::Code => "Code",
        TokenType::Blockquote => "Blockquote",
        TokenType::Markdown => "Markdown",
        // Document / slides
        TokenType::Document => "Document",
        TokenType::Section => "Section",
        TokenType::Paragraph => "Paragraph",
        TokenType::PageBreak => "PageBreak",
        TokenType::Header => "Header",
        TokenType::Footer => "Footer",
        TokenType::Presentation => "Presentation",
        TokenType::Slide => "Slide",
        TokenType::TitleSlide => "TitleSlide",
        TokenType::SectionSlide => "SectionSlide",
        TokenType::TwoColumn => "TwoColumn",
        TokenType::ImageSlide => "ImageSlide",
        // `List` and `Map` lex as TYPE tokens (TypeList/TypeMap) and are resolved
        // by context in the parser, so they are not listed here.
        _ => return None,
    };
    Some(name)
}

pub fn keyword_or_identifier(word: &str) -> TokenType {
    match word {
        // Top-level
        "Page" => TokenType::Page,
        "Component" => TokenType::Component,
        "Store" => TokenType::Store,
        "App" => TokenType::App,

        // Routing
        "Router" => TokenType::Router,
        "Route" => TokenType::Route,

        // State & logic
        "state" => TokenType::State,
        "derived" => TokenType::Derived,
        "effect" => TokenType::Effect,
        "action" => TokenType::Action,
        "use" => TokenType::Use,
        "fetch" => TokenType::Fetch,
        "from" => TokenType::From,
        "navigate" => TokenType::Navigate,
        "log" => TokenType::Log,
        "return" => TokenType::Return,

        // Control flow
        "if" => TokenType::If,
        "else" => TokenType::Else,
        "for" => TokenType::For,
        "in" => TokenType::In,
        "show" => TokenType::Show,

        // Fetch blocks
        "loading" => TokenType::Loading,
        "error" => TokenType::Error,
        "success" => TokenType::Success,

        // Types
        "String" => TokenType::TypeString,
        "Number" => TokenType::TypeNumber,
        "Bool" => TokenType::TypeBool,
        "List" => TokenType::TypeList,
        "Map" => TokenType::TypeMap,

        // Literals
        "true" => TokenType::BoolLiteral(true),
        "false" => TokenType::BoolLiteral(false),
        "null" => TokenType::Null,

        // Layout components
        "Container" => TokenType::Container,
        "Row" => TokenType::Row,
        "Column" => TokenType::Column,
        "Grid" => TokenType::Grid,
        "Stack" => TokenType::Stack,
        "Spacer" => TokenType::Spacer,
        "Divider" => TokenType::Divider,

        // Navigation components
        "Navbar" => TokenType::Navbar,
        "Sidebar" => TokenType::Sidebar,
        "Breadcrumb" => TokenType::Breadcrumb,
        "Link" => TokenType::Link,
        "Menu" => TokenType::Menu,
        "Tabs" => TokenType::Tabs,
        "TabPage" => TokenType::TabPage,

        // Data display components
        "Card" => TokenType::Card,
        "Table" => TokenType::Table,
        "Thead" => TokenType::Thead,
        "Tbody" => TokenType::Tbody,
        "Trow" => TokenType::Trow,
        "Tcell" => TokenType::Tcell,
        "Badge" => TokenType::Badge,
        "Avatar" => TokenType::Avatar,
        "Tooltip" => TokenType::Tooltip,
        "Tag" => TokenType::Tag,

        // Data input components
        "Input" => TokenType::Input,
        "Select" => TokenType::Select,
        "Option" => TokenType::Option,
        "Checkbox" => TokenType::Checkbox,
        "Radio" => TokenType::Radio,
        "Switch" => TokenType::Switch,
        "Slider" => TokenType::Slider,
        "Textarea" => TokenType::Textarea,
        "DatePicker" => TokenType::DatePicker,
        "FileUpload" => TokenType::FileUpload,
        "Form" => TokenType::Form,

        // Feedback components
        "Alert" => TokenType::Alert,
        "Toast" => TokenType::Toast,
        "Modal" => TokenType::Modal,
        "Dialog" => TokenType::Dialog,
        "Spinner" => TokenType::Spinner,
        "Progress" => TokenType::Progress,
        "Skeleton" => TokenType::Skeleton,

        // Action components
        "Button" => TokenType::Button,
        "IconButton" => TokenType::IconButton,
        "ButtonGroup" => TokenType::ButtonGroup,
        "Dropdown" => TokenType::Dropdown,

        // Media components
        "Image" => TokenType::Image,
        "Video" => TokenType::Video,
        "Audio" => TokenType::Audio,
        "Icon" => TokenType::Icon,
        "Carousel" => TokenType::Carousel,

        // Typography components
        "Text" => TokenType::Text,
        "Heading" => TokenType::Heading,
        "Code" => TokenType::Code,
        "Blockquote" => TokenType::Blockquote,
        "Markdown" => TokenType::Markdown,

        // Document components (PDF)
        "Document" => TokenType::Document,
        "Section" => TokenType::Section,
        "Paragraph" => TokenType::Paragraph,
        "PageBreak" => TokenType::PageBreak,
        "Header" => TokenType::Header,
        "Footer" => TokenType::Footer,

        // Slides components (PDF slide deck)
        "Presentation" => TokenType::Presentation,
        "Slide" => TokenType::Slide,
        "TitleSlide" => TokenType::TitleSlide,
        "SectionSlide" => TokenType::SectionSlide,
        "TwoColumn" => TokenType::TwoColumn,
        "ImageSlide" => TokenType::ImageSlide,

        // Style & Animation
        "style" => TokenType::Style,
        "Theme" => TokenType::Theme,
        "token" => TokenType::Token,
        "animate" => TokenType::Animate,
        "transition" => TokenType::Transition,

        // Children
        "children" => TokenType::Children,

        // List (as component, not type - handled by context)
        // "List" handled above as TypeList, we'll use context in parser
        _ => TokenType::Identifier(word.to_string()),
    }
}
