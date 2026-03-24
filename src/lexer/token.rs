use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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
    StringLiteral(String),
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
    Icon,
    Carousel,

    // Typography components
    Text,
    Heading,
    Code,
    Blockquote,

    // Document components (PDF)
    Document,
    Section,
    Paragraph,
    PageBreak,
    Header,
    Footer,

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

    // Events
    Event(String), // on:click, on:submit, etc.

    // Special
    Children,
    EOF,
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
            TokenType::If => write!(f, "if"),
            TokenType::Else => write!(f, "else"),
            TokenType::For => write!(f, "for"),
            TokenType::In => write!(f, "in"),
            TokenType::Show => write!(f, "show"),
            TokenType::Loading => write!(f, "loading"),
            TokenType::Error => write!(f, "error"),
            TokenType::Success => write!(f, "success"),
            TokenType::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenType::NumberLiteral(n) => write!(f, "{}", n),
            TokenType::BoolLiteral(b) => write!(f, "{}", b),
            TokenType::Null => write!(f, "null"),
            TokenType::Identifier(s) => write!(f, "{}", s),
            TokenType::Event(s) => write!(f, "on:{}", s),
            TokenType::EOF => write!(f, "EOF"),
            other => write!(f, "{:?}", other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self {
            token_type,
            line,
            column,
        }
    }
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
        "Icon" => TokenType::Icon,
        "Carousel" => TokenType::Carousel,

        // Typography components
        "Text" => TokenType::Text,
        "Heading" => TokenType::Heading,
        "Code" => TokenType::Code,
        "Blockquote" => TokenType::Blockquote,

        // Document components (PDF)
        "Document" => TokenType::Document,
        "Section" => TokenType::Section,
        "Paragraph" => TokenType::Paragraph,
        "PageBreak" => TokenType::PageBreak,
        "Header" => TokenType::Header,
        "Footer" => TokenType::Footer,

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
