use tower_lsp::lsp_types::*;

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
    ComponentInfo { name: "Grid", detail: "CSS grid layout" },
    ComponentInfo { name: "Stack", detail: "Stacked/overlapping layout" },
    ComponentInfo { name: "Spacer", detail: "Flexible space filler" },
    ComponentInfo { name: "Divider", detail: "Horizontal divider line" },
];

const NAV_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Navbar", detail: "Navigation bar" },
    ComponentInfo { name: "Sidebar", detail: "Side navigation panel" },
    ComponentInfo { name: "Breadcrumb", detail: "Breadcrumb navigation trail" },
    ComponentInfo { name: "Link", detail: "Navigation link" },
    ComponentInfo { name: "Menu", detail: "Dropdown menu" },
    ComponentInfo { name: "Tabs", detail: "Tab navigation" },
    ComponentInfo { name: "TabPage", detail: "Individual tab page" },
];

const DATA_DISPLAY_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Card", detail: "Content card container" },
    ComponentInfo { name: "Table", detail: "Data table" },
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
    ComponentInfo { name: "Input", detail: "Text input field" },
    ComponentInfo { name: "Select", detail: "Dropdown select" },
    ComponentInfo { name: "Option", detail: "Select option" },
    ComponentInfo { name: "Checkbox", detail: "Checkbox toggle" },
    ComponentInfo { name: "Radio", detail: "Radio button" },
    ComponentInfo { name: "Switch", detail: "Toggle switch" },
    ComponentInfo { name: "Slider", detail: "Range slider" },
    ComponentInfo { name: "DatePicker", detail: "Date picker input" },
    ComponentInfo { name: "FileUpload", detail: "File upload input" },
    ComponentInfo { name: "Form", detail: "Form wrapper" },
];

const FEEDBACK_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Alert", detail: "Alert message banner" },
    ComponentInfo { name: "Toast", detail: "Toast notification" },
    ComponentInfo { name: "Modal", detail: "Modal overlay dialog" },
    ComponentInfo { name: "Dialog", detail: "Confirmation dialog" },
    ComponentInfo { name: "Spinner", detail: "Loading spinner" },
    ComponentInfo { name: "Progress", detail: "Progress bar" },
    ComponentInfo { name: "Skeleton", detail: "Skeleton loading placeholder" },
];

const ACTION_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Button", detail: "Clickable button" },
    ComponentInfo { name: "IconButton", detail: "Icon-only button" },
    ComponentInfo { name: "ButtonGroup", detail: "Group of related buttons" },
    ComponentInfo { name: "Dropdown", detail: "Dropdown button" },
];

const MEDIA_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Image", detail: "Responsive image" },
    ComponentInfo { name: "Video", detail: "Video player" },
    ComponentInfo { name: "Icon", detail: "SVG icon" },
    ComponentInfo { name: "Carousel", detail: "Image carousel / slider" },
];

const TYPOGRAPHY_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Text", detail: "Paragraph text" },
    ComponentInfo { name: "Heading", detail: "Heading (h1-h6)" },
    ComponentInfo { name: "Code", detail: "Code block / inline code" },
    ComponentInfo { name: "Blockquote", detail: "Block quotation" },
];

const DOCUMENT_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Document", detail: "PDF document root" },
    ComponentInfo { name: "Section", detail: "Document section" },
    ComponentInfo { name: "Paragraph", detail: "Document paragraph" },
    ComponentInfo { name: "PageBreak", detail: "PDF page break" },
    ComponentInfo { name: "Header", detail: "Page header" },
    ComponentInfo { name: "Footer", detail: "Page footer" },
];

const ROUTING_COMPONENTS: &[ComponentInfo] = &[
    ComponentInfo { name: "Router", detail: "Client-side router outlet" },
    ComponentInfo { name: "Route", detail: "Route definition" },
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
    ROUTING_COMPONENTS,
];

// Sub-component mappings: parent -> list of (child_suffix, detail)
const SUB_COMPONENTS: &[(&str, &[(&str, &str)])] = &[
    ("Card", &[
        ("Header", "Card header section"),
        ("Body", "Card body content"),
        ("Footer", "Card footer section"),
        ("Image", "Card image"),
    ]),
    ("Navbar", &[
        ("Brand", "Navbar brand / logo area"),
        ("Links", "Navbar link group"),
        ("Actions", "Navbar action buttons"),
    ]),
    ("Modal", &[
        ("Header", "Modal header"),
        ("Body", "Modal body content"),
        ("Footer", "Modal footer / actions"),
    ]),
    ("Dialog", &[
        ("Header", "Dialog header"),
        ("Body", "Dialog body content"),
        ("Footer", "Dialog footer / actions"),
    ]),
    ("Tabs", &[
        ("Tab", "Individual tab"),
        ("Panel", "Tab panel content"),
    ]),
    ("Table", &[
        ("Header", "Table header"),
        ("Body", "Table body"),
        ("Row", "Table row"),
        ("Cell", "Table cell"),
    ]),
    ("Form", &[
        ("Group", "Form field group"),
        ("Actions", "Form action buttons"),
    ]),
    ("Sidebar", &[
        ("Header", "Sidebar header"),
        ("Content", "Sidebar content"),
        ("Footer", "Sidebar footer"),
    ]),
];

const KEYWORDS: &[(&str, &str)] = &[
    ("state", "Declare reactive state variable"),
    ("derived", "Declare computed/derived value"),
    ("effect", "Side-effect block that re-runs on dependency changes"),
    ("action", "Define a named action function"),
    ("if", "Conditional rendering"),
    ("else", "Else branch of conditional"),
    ("for", "List rendering / iteration"),
    ("in", "Iterator source in for-loop"),
    ("show", "Conditionally show/hide an element"),
    ("use", "Import a store into scope"),
    ("fetch", "Async data fetching block"),
    ("navigate", "Client-side navigation"),
    ("log", "Log expression to console"),
    ("animate", "Apply animation to element"),
    ("style", "Inline style block"),
    ("transition", "CSS transition block"),
];

const EVENTS: &[(&str, &str)] = &[
    ("click", "Mouse click event"),
    ("dblclick", "Double click event"),
    ("input", "Input value changed"),
    ("change", "Value committed / changed"),
    ("submit", "Form submission"),
    ("focus", "Element gained focus"),
    ("blur", "Element lost focus"),
    ("keydown", "Key pressed down"),
    ("keyup", "Key released"),
    ("mouseenter", "Mouse entered element"),
    ("mouseleave", "Mouse left element"),
    ("mouseover", "Mouse over element"),
    ("mouseout", "Mouse left element (including children)"),
    ("scroll", "Element scrolled"),
    ("resize", "Element resized"),
    ("load", "Resource loaded"),
    ("error", "Error occurred"),
    ("touchstart", "Touch began"),
    ("touchend", "Touch ended"),
    ("dragstart", "Drag started"),
    ("dragend", "Drag ended"),
    ("drop", "Item dropped"),
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
    ("type:", "Input type (text, email, password, number, etc.)"),
    ("value:", "Initial / current value"),
    ("min:", "Minimum value"),
    ("max:", "Maximum value"),
    ("step:", "Step increment"),
    ("cols:", "Number of grid columns"),
    ("gap:", "Grid / flex gap spacing"),
    ("visible:", "Visibility binding for modals / dialogs"),
    ("disabled:", "Disable the element"),
    ("required:", "Mark input as required"),
    ("controls:", "Show media controls"),
    ("autoplay:", "Auto-play media"),
    ("loop:", "Loop media playback"),
    ("method:", "HTTP method for fetch"),
    ("headers:", "HTTP headers for fetch"),
    ("guard:", "Route guard condition"),
    ("redirect:", "Redirect path when guard fails"),
];

const MODIFIERS: &[(&str, &str)] = &[
    ("primary", "Primary style variant"),
    ("secondary", "Secondary style variant"),
    ("outline", "Outline / bordered variant"),
    ("ghost", "Ghost / transparent variant"),
    ("danger", "Danger / destructive variant"),
    ("warning", "Warning style variant"),
    ("success", "Success style variant"),
    ("info", "Informational style variant"),
    ("small", "Small size"),
    ("large", "Large size"),
    ("disabled", "Disabled state"),
    ("full-width", "Full-width element"),
    ("rounded", "Rounded corners"),
    ("centered", "Center aligned"),
    ("h1", "Heading level 1"),
    ("h2", "Heading level 2"),
    ("h3", "Heading level 3"),
    ("h4", "Heading level 4"),
    ("h5", "Heading level 5"),
    ("h6", "Heading level 6"),
    ("ordered", "Ordered list"),
    ("unordered", "Unordered list"),
    ("bold", "Bold text"),
    ("italic", "Italic text"),
    ("underline", "Underlined text"),
    ("strikethrough", "Strikethrough text"),
    ("text", "Text input type"),
    ("email", "Email input type"),
    ("password", "Password input type"),
    ("number", "Number input type"),
    ("horizontal", "Horizontal orientation"),
    ("vertical", "Vertical orientation"),
];

// ---------------------------------------------------------------------------
// Top-level snippets
// ---------------------------------------------------------------------------

fn top_level_snippets() -> Vec<CompletionItem> {
    vec![
        snippet_item(
            "Page",
            "Page declaration",
            "Page ${1:Name} (path: \"${2:/}\", title: \"${3:Title}\") {\n\t$0\n}",
        ),
        snippet_item(
            "Component",
            "Component declaration",
            "Component ${1:Name} (${2:props}) {\n\t$0\n}",
        ),
        snippet_item(
            "Store",
            "Store declaration",
            "Store ${1:Name} {\n\tstate ${2:value} = ${3:0}\n\t$0\n}",
        ),
        snippet_item(
            "App",
            "App declaration",
            "App {\n\t$0\n}",
        ),
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
    /// Cursor is right after a `.` — provide sub-component completions for `parent`.
    DotAccess(String),
    /// Cursor is after `on:` — provide event names.
    EventTrigger,
    /// Cursor is inside `( )` — provide named args.
    InsideParens,
    /// Cursor is inside `{ }` — provide components and keywords.
    InsideBraces,
    /// Cursor is at top level — provide Page, Component, Store, App.
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

    // Check for `on:` immediately before cursor
    if prefix.trim_end().ends_with("on:") {
        return CompletionContext::EventTrigger;
    }

    // Check for dot access: e.g. "Card."
    if let Some(dot_pos) = prefix.rfind('.') {
        let before_dot = prefix[..dot_pos].trim_end();
        if let Some(word) = before_dot.split_whitespace().last() {
            if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                return CompletionContext::DotAccess(word.to_string());
            }
        }
    }

    // Count open/close parens and braces up to cursor to determine nesting.
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

    for ch in text_up_to_cursor.chars() {
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string => paren_depth -= 1,
            '{' if !in_string => brace_depth += 1,
            '}' if !in_string => brace_depth -= 1,
            _ => {}
        }
    }

    if paren_depth > 0 {
        return CompletionContext::InsideParens;
    }

    if brace_depth > 0 {
        return CompletionContext::InsideBraces;
    }

    CompletionContext::TopLevel
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn provide_completions(source: &str, position: Position) -> Vec<CompletionItem> {
    let ctx = detect_context(source, position);
    let mut items = Vec::new();

    match ctx {
        CompletionContext::DotAccess(parent) => {
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
                    break;
                }
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
            for &(arg, detail) in NAMED_ARGS {
                items.push(CompletionItem {
                    label: arg.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
            for &(modifier, detail) in MODIFIERS {
                items.push(CompletionItem {
                    label: modifier.to_string(),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    detail: Some(detail.to_string()),
                    ..Default::default()
                });
            }
        }

        CompletionContext::InsideBraces => {
            // Components
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
