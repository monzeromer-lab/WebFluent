use tower_lsp::lsp_types::*;
use webfluent::parser::ast::*;
use webfluent::parser::vocabulary::MODIFIER_KEYWORDS;

// ---------------------------------------------------------------------------
// Static data
// ---------------------------------------------------------------------------

struct ComponentInfo {
    name: &'static str,
    detail: &'static str,
}

const LAYOUT_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Container", detail: "Responsive centered container" },
    ComponentInfo { name: "Row", detail: "Horizontal flex row" },
    ComponentInfo { name: "Column", detail: "Vertical flex column" },
    ComponentInfo { name: "Grid", detail: "CSS grid layout (cols:, gap:)" },
    ComponentInfo { name: "Stack", detail: "Stacked/overlapping layout" },
    ComponentInfo { name: "Spacer", detail: "Flexible space filler" },
    ComponentInfo { name: "Divider", detail: "Horizontal divider line" },
];

const NAV_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Navbar", detail: "Navigation bar (Navbar.Brand, Navbar.Links, Navbar.Actions)" },
    ComponentInfo { name: "Sidebar", detail: "Side navigation panel (Sidebar.Header, Sidebar.Item, Sidebar.Divider)" },
    ComponentInfo { name: "Breadcrumb", detail: "Breadcrumb navigation trail (Breadcrumb.Item)" },
    ComponentInfo { name: "Link", detail: "Navigation link (to:, href:)" },
    ComponentInfo { name: "Menu", detail: "Dropdown menu (Menu.Item)" },
    ComponentInfo { name: "Tabs", detail: "Tab navigation with TabPage children" },
    ComponentInfo { name: "TabPage", detail: "Individual tab page" },
];

const DATA_DISPLAY_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Card", detail: "Content card container (Card.Header, Card.Body, Card.Footer)" },
    ComponentInfo { name: "Table", detail: "Data table (Thead, Tbody, Trow, Tcell)" },
    ComponentInfo { name: "Thead", detail: "Table header section" },
    ComponentInfo { name: "Tbody", detail: "Table body section" },
    ComponentInfo { name: "Trow", detail: "Table row" },
    ComponentInfo { name: "Tcell", detail: "Table cell" },
    ComponentInfo { name: "List", detail: "Ordered/unordered list" },
    ComponentInfo { name: "Badge", detail: "Status badge / counter" },
    ComponentInfo { name: "Avatar", detail: "User avatar image" },
    ComponentInfo { name: "Tooltip", detail: "Hover tooltip" },
    ComponentInfo { name: "Tag", detail: "Label tag / chip" },
];

const INPUT_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Input", detail: "Text input field (text, email, password, number, bind:)" },
    ComponentInfo { name: "Select", detail: "Dropdown select with Option children (bind:)" },
    ComponentInfo { name: "Option", detail: "Select option (value:)" },
    ComponentInfo { name: "Checkbox", detail: "Checkbox toggle (bind:)" },
    ComponentInfo { name: "Radio", detail: "Radio button (bind:, value:)" },
    ComponentInfo { name: "Switch", detail: "Toggle switch (bind:)" },
    ComponentInfo { name: "Slider", detail: "Range slider (min:, max:, step:, bind:)" },
    ComponentInfo { name: "DatePicker", detail: "Date picker input (bind:)" },
    ComponentInfo { name: "FileUpload", detail: "File upload input (accept:, bind:)" },
    ComponentInfo { name: "Form", detail: "Form wrapper (on:submit:)" },
];

const FEEDBACK_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Alert", detail: "Alert message banner (success, warning, danger, info, dismissible)" },
    ComponentInfo { name: "Toast", detail: "Toast notification" },
    ComponentInfo { name: "Modal", detail: "Modal overlay dialog (Modal.Header, Modal.Body, Modal.Footer, visible:)" },
    ComponentInfo { name: "Dialog", detail: "Confirmation dialog (Dialog.Header, Dialog.Body, Dialog.Footer, visible:)" },
    ComponentInfo { name: "Spinner", detail: "Loading spinner" },
    ComponentInfo { name: "Progress", detail: "Progress bar (value:, max:)" },
    ComponentInfo { name: "Skeleton", detail: "Skeleton loading placeholder" },
];

const ACTION_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Button", detail: "Clickable button (primary, secondary, outlined, danger, large)" },
    ComponentInfo { name: "IconButton", detail: "Icon-only button (icon:, label:)" },
    ComponentInfo { name: "ButtonGroup", detail: "Group of related buttons" },
    ComponentInfo { name: "Dropdown", detail: "Dropdown button with Dropdown.Item" },
];

const MEDIA_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Image", detail: "Responsive image (src:, alt:)" },
    ComponentInfo { name: "Video", detail: "Video player (src:, controls:, autoplay:, loop:)" },
    ComponentInfo { name: "Icon", detail: "Vector icon" },
    ComponentInfo { name: "Carousel", detail: "Image carousel with Carousel.Slide" },
];

const TYPOGRAPHY_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Text", detail: "Paragraph text (bold, italic, muted, uppercase)" },
    ComponentInfo { name: "Heading", detail: "Heading (h1-h6)" },
    ComponentInfo { name: "Code", detail: "Code block / inline code" },
    ComponentInfo { name: "Blockquote", detail: "Block quotation" },
];

const DOCUMENT_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Document", detail: "PDF document root (title:)" },
    ComponentInfo { name: "Section", detail: "Document section" },
    ComponentInfo { name: "Paragraph", detail: "Document paragraph" },
    ComponentInfo { name: "PageBreak", detail: "PDF page break" },
    ComponentInfo { name: "Header", detail: "Page header" },
    ComponentInfo { name: "Footer", detail: "Page footer" },
];

const SLIDES_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Presentation", detail: "Slide deck root (title:)" },
    ComponentInfo { name: "Slide", detail: "Freeform slide (one output page)" },
    ComponentInfo { name: "TitleSlide", detail: "Title slide: TitleSlide(\"Title\", \"Subtitle\")" },
    ComponentInfo { name: "SectionSlide", detail: "Section divider slide: SectionSlide(\"Label\")" },
    ComponentInfo { name: "TwoColumn", detail: "Two-column slide; takes exactly 2 Container children" },
    ComponentInfo { name: "ImageSlide", detail: "Image slide: ImageSlide(src: \"...\", caption: \"...\")" },
];

const ROUTING_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Router", detail: "Client-side router outlet" },
    ComponentInfo { name: "Route", detail: "Route definition (path:, page:)" },
];

const ALL_COMPONENT_GROUPS: &[&[ComponentInfo]] = &[
    LAYOUT_COMPONENTS,
    NAV_COMPONENTS,
    DATA_DISPLAY_COMPONENTS,
    INPUT_COMPONENTS,
    FEEDBACK_COMPONENTS,
    ACTION_COMPONENTS,
    MEDIA_COMPONENTS,
    TYPOGRAPHY_COMPONENTS,
    DOCUMENT_COMPONENTS,
    SLIDES_COMPONENTS,
    ROUTING_COMPONENTS,
];

// Sub-component mappings matching the actual WebFluent standard library
const SUB_COMPONENTS: &[(&str, &[(&str, &str)])] = &[
    (
        "Navbar",
        &[
            ("Brand", "Navbar brand / logo area"),
            ("Links", "Navbar link group"),
            ("Actions", "Navbar action buttons"),
        ],
    ),
    (
        "Sidebar",
        &[
            ("Header", "Sidebar header section"),
            ("Item", "Sidebar navigation link item: Sidebar.Item(\"Label\", to: \"/route\")"),
            ("Divider", "Sidebar horizontal divider"),
        ],
    ),
    (
        "Breadcrumb",
        &[
            ("Item", "Breadcrumb link item: Breadcrumb.Item(\"Label\", to: \"/route\")"),
        ],
    ),
    (
        "Carousel",
        &[
            ("Slide", "Carousel slide wrapper: Carousel.Slide { ... }"),
        ],
    ),
    (
        "Card",
        &[
            ("Header", "Card header section"),
            ("Body", "Card body content"),
            ("Footer", "Card footer section"),
        ],
    ),
    (
        "Modal",
        &[
            ("Header", "Modal dialog header"),
            ("Body", "Modal dialog body"),
            ("Footer", "Modal dialog footer"),
        ],
    ),
    (
        "Dialog",
        &[
            ("Header", "Dialog header"),
            ("Body", "Dialog body"),
            ("Footer", "Dialog footer"),
        ],
    ),
    (
        "Dropdown",
        &[
            ("Item", "Dropdown menu item"),
        ],
    ),
    (
        "Menu",
        &[
            ("Item", "Menu list item"),
        ],
    ),
];

const KEYWORDS: &[(&str, &str)] = &[
    ("state", "Declare reactive state variable"),
    ("derived", "Declare computed/derived signal"),
    ("effect", "Side-effect block that re-runs on dependency changes"),
    ("action", "Define a named action function"),
    ("if", "Conditional rendering"),
    ("else", "Else branch of conditional"),
    ("for", "List rendering / iteration"),
    ("in", "Iterator source in for-loop"),
    ("show", "Conditionally show/hide an element via CSS display"),
    ("use", "Import a store into current scope"),
    ("fetch", "Async data fetching block"),
    ("navigate", "Client-side navigation"),
    ("log", "Log expression to browser console"),
    ("return", "Return from an action"),
    ("animate", "Apply animation to element"),
    ("style", "Inline style block"),
    ("transition", "CSS transition block"),
    ("Theme", "Design system theme declaration"),
    ("token", "Design token declaration"),
];

const EVENTS: &[(&str, &str)] = &[
    ("click", "Mouse click event"),
    ("dblclick", "Double click event"),
    ("input", "Input value changed (real-time)"),
    ("change", "Value committed / changed"),
    ("submit", "Form submission"),
    ("focus", "Element gained focus"),
    ("blur", "Element lost focus"),
    ("keydown", "Key pressed down"),
    ("keyup", "Key released"),
    ("mouseenter", "Mouse entered element"),
    ("mouseleave", "Mouse left element"),
    ("scroll", "Element scrolled"),
];

const NAMED_ARGS: &[(&str, &str)] = &[
    ("bind:", "Two-way data binding to state variable"),
    ("placeholder:", "Placeholder text for inputs"),
    ("path:", "URL path for pages / routes"),
    ("title:", "Title text"),
    ("src:", "Source URL (images, video)"),
    ("alt:", "Alternative text for images"),
    ("href:", "Link destination URL"),
    ("to:", "Navigation target path"),
    ("label:", "Accessible label text"),
    ("icon:", "Icon name"),
    ("columns:", "Number of grid columns"),
    ("gap:", "Grid / flex gap spacing"),
    ("visible:", "Visibility binding for modals / dialogs"),
    ("disabled:", "Disable the element"),
    ("required:", "Mark input as required"),
    ("controls:", "Show media controls"),
    ("autoplay:", "Auto-play media"),
    ("loop:", "Loop media playback"),
    ("caption:", "Caption text for ImageSlide"),
    ("interval:", "Interval in ms for Carousel"),
    ("guard:", "Route guard condition"),
    ("redirect:", "Redirect path when guard fails"),
    ("accept:", "Accepted file types for FileUpload"),
];

const PAGE_ATTRS: &[(&str, &str)] = &[
    ("path", "URL route, e.g. \"/about\" or \"/user/:id\""),
    ("title", "Browser tab title and search-result heading"),
    ("description", "Search-result snippet and link-preview text"),
    ("image", "Link-preview image, site-relative or absolute"),
    ("type", "og:type \u{2014} \"website\" or \"article\""),
    ("noindex", "Keep this page out of search results"),
    ("guard", "Expression that must hold for the route to render"),
    ("redirect", "Path to redirect to when the guard fails"),
];

const BASELINE_TOKENS: &[(&str, &str)] = &[
    ("color-primary", "Primary brand accent color"),
    ("color-secondary", "Secondary neutral color"),
    ("color-success", "Success notification color"),
    ("color-danger", "Destructive / error color"),
    ("color-warning", "Warning / alert color"),
    ("color-info", "Informational color"),
    ("color-background", "Main background surface color"),
    ("color-surface", "Elevated card/surface color"),
    ("color-text", "Primary foreground text color"),
    ("color-text-muted", "Secondary muted text color"),
    ("color-border", "Border perimeter color"),
    ("font-family", "Body font family"),
    ("font-family-mono", "Monospace font family"),
    ("font-size-xs", "Fluid extra-small font size"),
    ("font-size-sm", "Fluid small font size"),
    ("font-size-base", "Fluid base font size (1rem)"),
    ("font-size-lg", "Fluid large font size"),
    ("font-size-xl", "Fluid extra-large font size"),
    ("font-size-2xl", "Fluid 2xl font size"),
    ("font-size-3xl", "Fluid 3xl font size"),
    ("spacing-xs", "Extra-small spacing (0.25rem)"),
    ("spacing-sm", "Small spacing (0.5rem)"),
    ("spacing-md", "Medium spacing (1rem)"),
    ("spacing-lg", "Large spacing (1.5rem)"),
    ("spacing-xl", "Fluid extra-large spacing"),
    ("radius-sm", "Small border radius (0.25rem)"),
    ("radius-md", "Medium border radius (0.5rem)"),
    ("radius-lg", "Large border radius (1rem)"),
    ("radius-full", "Fully rounded pill radius (9999px)"),
    ("shadow-sm", "Subtle elevation shadow"),
    ("shadow-md", "Medium surface shadow"),
    ("shadow-lg", "High elevation shadow"),
];

fn modifier_detail(modifier: &str) -> &'static str {
    match modifier {
        "small" | "medium" | "large" => "Size",
        "primary" | "secondary" | "success" | "danger" | "warning" | "info" => "Color variant",
        "error" | "loading" => "State",
        "rounded" | "pill" | "square" => "Shape",
        "flat" | "elevated" | "outlined" => "Elevation",
        "full" | "fit" => "Width",
        "bold" | "italic" | "underline" | "uppercase" | "lowercase" => "Text style",
        "left" | "center" | "right" => "Text alignment",
        "heading" | "subtitle" | "muted" => "Typography",
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => "Heading level",
        "dismissible" | "block" | "bordered" => "Behavior",
        "controls" | "autoplay" => "Media",
        "text" | "email" | "password" | "number" | "search" | "tel" | "url" | "date" | "time"
        | "datetime" | "color" => "Input type",
        "submit" | "reset" => "Button type",
        "fast" | "slow" => "Animation speed",
        _ => "Animation",
    }
}

// ---------------------------------------------------------------------------
// Top-level snippets
// ---------------------------------------------------------------------------

fn top_level_snippets() -> Vec<CompletionItem> {
    vec![
        snippet_item(
            "Page",
            "Page route declaration",
            "Page ${1:Name} (path: \"${2:/}\", title: \"${3:Title}\") {\n\t$0\n}",
        ),
        snippet_item(
            "Component",
            "Reusable component declaration",
            "Component ${1:Name} (${2:props}) {\n\t$0\n}",
        ),
        snippet_item(
            "Store",
            "Shared state store declaration",
            "Store ${1:Name} {\n\tstate ${2:value} = ${3:0}\n\t$0\n}",
        ),
        snippet_item(
            "Theme",
            "Design system theme declaration",
            "Theme ${1:Brand} {\n\ttoken ${2:color-primary}: \"${3:#3B82F6}\"\n\t$0\n}",
        ),
        snippet_item("App", "Root app declaration", "App {\n\t$0\n}"),
    ]
}

fn snippet_item(label: &str, detail: &str, insert_text: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(detail.to_string()),
        insert_text: Some(insert_text.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Context detection
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum CompletionContext {
    /// Cursor is right after a `.` — provide sub-components or store members for `parent`.
    DotAccess(String),
    /// Cursor is after `use ` — provide store names.
    UseStore,
    /// Cursor is inside `Theme { ... }` — provide design token suggestions.
    InsideTheme,
    /// Cursor is after `on:` — provide event names.
    EventTrigger,
    /// Cursor is inside `( )` — provide named args, modifiers, in-scope identifiers.
    InsideParens,
    /// Cursor is inside `{ }` — provide components and keywords.
    InsideBraces,
    /// Cursor is at top level — provide Page, Component, Store, Theme, App.
    TopLevel,
}

fn detect_context(source: &str, position: Position) -> CompletionContext {
    let line_idx = position.line as usize;
    let col_idx = position.character as usize;

    let lines: Vec<&str> = source.lines().collect();
    let current_line = lines.get(line_idx).copied().unwrap_or("");
    let prefix = if col_idx <= current_line.len() {
        &current_line[..col_idx]
    } else {
        current_line
    };

    let trimmed_prefix = prefix.trim_end();

    // Check for `use ` immediately before cursor
    if trimmed_prefix.ends_with("use") || trimmed_prefix.ends_with("use ") {
        return CompletionContext::UseStore;
    }

    // Check for `on:` immediately before cursor
    if trimmed_prefix.ends_with("on:") {
        return CompletionContext::EventTrigger;
    }

    // Check for dot access: e.g. "Navbar." or "CartStore."
    if let Some(dot_pos) = prefix.rfind('.') {
        let before_dot = prefix[..dot_pos].trim_end();
        if let Some(word) = before_dot.split(|c: char| !c.is_alphanumeric() && c != '_').next_back() {
            if !word.is_empty() {
                return CompletionContext::DotAccess(word.to_string());
            }
        }
    }

    // Count open/close parens and braces up to cursor to determine nesting
    let text_up_to_cursor: String = lines
        .iter()
        .take(line_idx)
        .copied()
        .chain(std::iter::once(prefix))
        .collect::<Vec<&str>>()
        .join("\n");

    let mut paren_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut in_string = false;
    let mut last_top_decl = String::new();

    let mut chars = text_up_to_cursor.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string => paren_depth -= 1,
            '{' if !in_string => brace_depth += 1,
            '}' if !in_string => brace_depth -= 1,
            _ if !in_string && brace_depth == 0 => {
                // Record declaration keyword if at top-level
                if ch.is_alphabetic() {
                    let mut word = String::new();
                    word.push(ch);
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() {
                            word.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if word == "Theme" || word == "Page" || word == "Component" || word == "Store" || word == "App" {
                        last_top_decl = word;
                    }
                }
            }
            _ => {}
        }
    }

    if paren_depth > 0 {
        return CompletionContext::InsideParens;
    }

    if brace_depth > 0 {
        if last_top_decl == "Theme" {
            return CompletionContext::InsideTheme;
        }
        return CompletionContext::InsideBraces;
    }

    CompletionContext::TopLevel
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn provide_completions(
    source: &str,
    position: Position,
    program: Option<&Program>,
) -> Vec<CompletionItem> {
    let ctx = detect_context(source, position);
    let mut items = Vec::new();

    match ctx {
        CompletionContext::DotAccess(parent) => {
            // 1. Check built-in sub-components
            for &(p, children) in SUB_COMPONENTS {
                if p == parent {
                    for &(child, detail) in children {
                        items.push(CompletionItem {
                            label: format!("{}.{}", parent, child),
                            kind: Some(CompletionItemKind::CLASS),
                            detail: Some(detail.to_string()),
                            insert_text: Some(child.to_string()),
                            ..Default::default()
                        });
                    }
                    return items;
                }
            }

            // 2. Check Store members if parent is a Store in program
            if let Some(prog) = program {
                for decl in &prog.declarations {
                    if let Declaration::Store(s) = decl {
                        if s.name == parent {
                            for stmt in &s.body {
                                match &stmt.kind {
                                    StatementKind::State(st) => {
                                        items.push(CompletionItem {
                                            label: st.name.clone(),
                                            kind: Some(CompletionItemKind::VARIABLE),
                                            detail: Some(format!("Store state of {}", s.name)),
                                            ..Default::default()
                                        });
                                    }
                                    StatementKind::Derived(d) => {
                                        items.push(CompletionItem {
                                            label: d.name.clone(),
                                            kind: Some(CompletionItemKind::VARIABLE),
                                            detail: Some(format!("Store derived signal of {}", s.name)),
                                            ..Default::default()
                                        });
                                    }
                                    StatementKind::Action(a) => {
                                        items.push(CompletionItem {
                                            label: a.name.clone(),
                                            kind: Some(CompletionItemKind::FUNCTION),
                                            detail: Some(format!("Store action of {}", s.name)),
                                            ..Default::default()
                                        });
                                    }
                                    _ => {}
                                }
                            }
                            return items;
                        }
                    }
                }
            }
        }

        CompletionContext::UseStore => {
            if let Some(prog) = program {
                for decl in &prog.declarations {
                    if let Declaration::Store(s) = decl {
                        items.push(CompletionItem {
                            label: s.name.clone(),
                            kind: Some(CompletionItemKind::MODULE),
                            detail: Some("Shared reactive store".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        CompletionContext::InsideTheme => {
            items.push(snippet_item(
                "token",
                "Declare design token: token <name>: <value>",
                "token ${1:name}: \"${2:value}\"",
            ));
            for &(name, desc) in BASELINE_TOKENS {
                items.push(CompletionItem {
                    label: format!("token {name}"),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some(desc.to_string()),
                    insert_text: Some(format!("token {name}: \"${{1:value}}\"")),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                });
            }
        }

        CompletionContext::EventTrigger => {
            for &(event, detail) in EVENTS {
                items.push(CompletionItem {
                    label: event.to_string(),
                    kind: Some(CompletionItemKind::EVENT),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
        }

        CompletionContext::InsideParens => {
            // Named arguments
            for &(arg, detail) in NAMED_ARGS {
                items.push(CompletionItem {
                    label: arg.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
            // Canonical modifiers
            for modifier in MODIFIER_KEYWORDS {
                items.push(CompletionItem {
                    label: modifier.to_string(),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    detail: Some(modifier_detail(modifier).to_string()),
                    ..Default::default()
                });
            }
            // Page attributes
            for &(attr, detail) in PAGE_ATTRS {
                items.push(CompletionItem {
                    label: format!("{attr}:"),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
            // In-scope variables (state, derived, props) if program is available
            if let Some(prog) = program {
                for decl in &prog.declarations {
                    match decl {
                        Declaration::Page(p) => {
                            collect_stmt_identifiers(&p.body, &mut items);
                        }
                        Declaration::Component(c) => {
                            for p in &c.props {
                                items.push(CompletionItem {
                                    label: p.name.clone(),
                                    kind: Some(CompletionItemKind::VARIABLE),
                                    detail: Some(format!("Prop of {}", c.name)),
                                    ..Default::default()
                                });
                            }
                            collect_stmt_identifiers(&c.body, &mut items);
                        }
                        Declaration::Store(s) => {
                            collect_stmt_identifiers(&s.body, &mut items);
                        }
                        _ => {}
                    }
                }
            }
        }

        CompletionContext::InsideBraces => {
            // Built-in components
            for group in ALL_COMPONENT_GROUPS {
                for info in *group {
                    items.push(CompletionItem {
                        label: info.name.to_string(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some(info.detail.to_string()),
                        ..Default::default()
                    });
                }
            }
            // User-defined components in program
            if let Some(prog) = program {
                for decl in &prog.declarations {
                    if let Declaration::Component(c) = decl {
                        items.push(CompletionItem {
                            label: c.name.clone(),
                            kind: Some(CompletionItemKind::CLASS),
                            detail: Some("User-defined component".to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
            // Keywords
            for &(kw, detail) in KEYWORDS {
                items.push(CompletionItem {
                    label: kw.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
        }

        CompletionContext::TopLevel => {
            items = top_level_snippets();
        }
    }

    items
}

fn collect_stmt_identifiers(stmts: &[Statement], items: &mut Vec<CompletionItem>) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::State(s) => {
                items.push(CompletionItem {
                    label: s.name.clone(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some("Reactive state variable".to_string()),
                    ..Default::default()
                });
            }
            StatementKind::Derived(d) => {
                items.push(CompletionItem {
                    label: d.name.clone(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some("Computed derived signal".to_string()),
                    ..Default::default()
                });
            }
            StatementKind::Action(a) => {
                items.push(CompletionItem {
                    label: a.name.clone(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some("Action function".to_string()),
                    ..Default::default()
                });
            }
            _ => {}
        }
    }
}
