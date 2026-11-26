use std::fmt;

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
pub enum TokenType {
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

    // Special
    Eof,

    // Design tokens, separators, documentation and CSS
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
            TokenType::StringLiteral(s) => write!(f, "\"{}\"", s.spelling),
            TokenType::TemporalLiteral(t) => write!(f, "@{t}"),
            TokenType::ColorLiteral(c) => write!(f, "#{c}"),
            TokenType::MoneyLiteral(symbol, amount) => write!(f, "{symbol}{amount}"),
            TokenType::NumberLiteral(n) => write!(f, "{n}"),
            TokenType::BoolLiteral(b) => write!(f, "{b}"),
            TokenType::Null => write!(f, "null"),
            TokenType::Identifier(s) => write!(f, "{s}"),
            TokenType::Eof => write!(f, "EOF"),
            other => write!(f, "{other:?}"),
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
            // known (see `LexerV2::tokenize`); callers using `new` directly get 0.
            offset: 0,
            end: 0,
        }
    }
}

/// Every built-in component name the parser accepts as an element, in the
/// language reference's order.
pub const ALL_COMPONENT_NAMES: &[&str] = &[
    // Layout
    "Container",
    "Row",
    "Column",
    "Grid",
    "Stack",
    "Spacer",
    "Divider",
    "Splitter",
    "VirtualList",
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
    "AlertDialog",
    "Sheet",
    "Popover",
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
    "Frame",
    "Icon",
    "Carousel",
    // Typography
    "Text",
    "Heading",
    "Code",
    "Blockquote",
    "Markdown",
    // Content
    "Callout",
    "Kbd",
    "EmptyState",
    "Stat",
    "CodeBlock",
    "Chip",
    "VisuallyHidden",
    "CopyButton",
    "ThemeToggle",
    "LocaleSwitch",
    "Accordion",
    "Field",
    "Fieldset",
    "InputGroup",
    "OnThisPage",
    // Choosing
    "Toggle",
    "ToggleGroup",
    "RadioGroup",
    "Pagination",
    "Stepper",
    "NumberInput",
    "Calendar",
    // Collections
    "Listbox",
    "Tree",
    "DataTable",
    "Combobox",
    "CommandPalette",
    "AppShell",
    // More form controls
    "CheckboxGroup",
    "InputOTP",
    "TagInput",
    "Rating",
    "DateRangePicker",
    // More navigation
    "NavigationMenu",
    "Menubar",
    "ContextMenu",
    "BottomNavigation",
    "PrevNext",
    // More overlays
    "HoverCard",
    "LoadingOverlay",
    "Lightbox",
    "Tour",
    "ConsentBanner",
    "ConsentSettings",
    // Display and layout, the second group
    "DescriptionList",
    "Timeline",
    "AvatarGroup",
    "Item",
    "Gallery",
    "Masonry",
    "CircularProgress",
    "Meter",
    "Sparkline",
    "RelativeTime",
    "AspectRatio",
    "ScrollArea",
    "Center",
    "Sticky",
    "PanelGroup",
    // Tier 3
    "ColorPicker",
    "TransferList",
    "Editable",
    "Fab",
    "BackToTop",
    "Popconfirm",
    "Highlight",
    "ClientOnly",
    // A node handed to somebody else's code, with a lifetime.
    "Host",
    // A custom element a script defines, by its tag.
    "Element",
    // A block drawn into a node somebody else's code owns.
    "Portal",
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
    "Background",
    "Watermark",
    "TableOfContents",
    "Chart",
    "QrCode",
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

