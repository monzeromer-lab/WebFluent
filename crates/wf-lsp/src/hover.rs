use tower_lsp::lsp_types::*;

// ---------------------------------------------------------------------------
// Built-in component documentation
// ---------------------------------------------------------------------------

const COMPONENT_DOCS: &[(&str, &str)] = &[
    // Layout
    ("Container", "**Container** — Responsive centered container with max-width and horizontal padding."),
    ("Row", "**Row** — Horizontal flex layout. Children are laid out in a row."),
    ("Column", "**Column** — Vertical flex layout. Children are laid out in a column."),
    ("Grid", "**Grid** — CSS Grid layout. Use `cols:` to set column count and `gap:` for spacing."),
    ("Stack", "**Stack** — Stacked/overlapping layout where children are layered on top of each other."),
    ("Spacer", "**Spacer** — Flexible space that fills available room in a flex container."),
    ("Divider", "**Divider** — A horizontal divider line for separating content."),
    // Navigation
    ("Navbar", "**Navbar** — Top navigation bar. Sub-components: `Navbar.Brand`, `Navbar.Links`, `Navbar.Actions`."),
    ("Sidebar", "**Sidebar** — Side navigation panel. Sub-components: `Sidebar.Header`, `Sidebar.Content`, `Sidebar.Footer`."),
    ("Breadcrumb", "**Breadcrumb** — Breadcrumb trail showing the current page hierarchy."),
    ("Link", "**Link** — Navigation link. Use `to:` for internal routes or `href:` for external URLs."),
    ("Menu", "**Menu** — Dropdown menu component."),
    ("Tabs", "**Tabs** — Tab navigation with multiple panels. Sub-components: `Tabs.Tab`, `Tabs.Panel`."),
    ("TabPage", "**TabPage** — Individual page within a tab group."),
    // Data display
    ("Card", "**Card** — Content card with optional header, body, and footer. Sub-components: `Card.Header`, `Card.Body`, `Card.Footer`, `Card.Image`."),
    ("Table", "**Table** — Data table. Use `Thead`, `Tbody`, `Trow`, `Tcell` for structure."),
    ("Thead", "**Thead** — Table header section containing header row(s)."),
    ("Tbody", "**Tbody** — Table body section containing data rows."),
    ("Trow", "**Trow** — A row within a table header or body."),
    ("Tcell", "**Tcell** — A cell within a table row."),
    ("List", "**List** — Ordered or unordered list. Use `ordered` / `unordered` modifiers."),
    ("Badge", "**Badge** — Small status indicator or counter."),
    ("Avatar", "**Avatar** — User avatar, typically a circular image."),
    ("Tooltip", "**Tooltip** — Floating tooltip shown on hover."),
    ("Tag", "**Tag** — Label tag or chip for categories and statuses."),
    // Inputs
    ("Input", "**Input** — Text input field. Use modifiers like `text`, `email`, `password`, `number`. Supports `bind:` for two-way binding."),
    ("Select", "**Select** — Dropdown select with `Option` children. Supports `bind:`."),
    ("Option", "**Option** — An option within a `Select` dropdown."),
    ("Checkbox", "**Checkbox** — Boolean toggle checkbox. Use `bind:` and `label:`."),
    ("Radio", "**Radio** — Radio button for single-choice selection. Use `bind:` and `label:`."),
    ("Switch", "**Switch** — Toggle switch (on/off). Use `bind:` and `label:`."),
    ("Slider", "**Slider** — Range slider input. Use `min:`, `max:`, `step:`, `bind:`."),
    ("DatePicker", "**DatePicker** — Date selection input. Supports `bind:`."),
    ("FileUpload", "**FileUpload** — File upload input with drag-and-drop support."),
    ("Form", "**Form** — Form wrapper. Use `on:submit` for handling submissions."),
    // Feedback
    ("Alert", "**Alert** — Alert message banner. Use `success`, `warning`, `danger`, `info` modifiers."),
    ("Toast", "**Toast** — Toast notification popup."),
    ("Modal", "**Modal** — Full-screen overlay dialog. Use `visible:` binding and `title:`. Sub-components: `Modal.Header`, `Modal.Body`, `Modal.Footer`."),
    ("Dialog", "**Dialog** — Confirmation dialog. Use `visible:` and `title:`. Sub-components: `Dialog.Header`, `Dialog.Body`, `Dialog.Footer`."),
    ("Spinner", "**Spinner** — Loading spinner indicator."),
    ("Progress", "**Progress** — Progress bar. Use `value:`, `min:`, `max:`."),
    ("Skeleton", "**Skeleton** — Skeleton loading placeholder that mimics content shape."),
    // Actions
    ("Button", "**Button** — Clickable button. Modifiers: `primary`, `secondary`, `outline`, `ghost`, `danger`, `small`, `large`."),
    ("IconButton", "**IconButton** — Icon-only button. Requires `icon:` and `label:` for accessibility."),
    ("ButtonGroup", "**ButtonGroup** — Group of related buttons displayed together."),
    ("Dropdown", "**Dropdown** — Button with a dropdown menu."),
    // Media
    ("Image", "**Image** — Responsive image. Requires `src:` and `alt:` for accessibility."),
    ("Video", "**Video** — Video player. Use `src:`, `controls:`, `autoplay:`, `loop:`."),
    ("Icon", "**Icon** — SVG icon. Use `icon:` to specify the icon name."),
    ("Carousel", "**Carousel** — Image carousel / slider with navigation."),
    // Typography
    ("Text", "**Text** — Paragraph text. Supports `bold`, `italic`, `underline`, `strikethrough` modifiers."),
    ("Heading", "**Heading** — Heading element. Use `h1`–`h6` modifiers for level."),
    ("Code", "**Code** — Code block or inline code snippet."),
    ("Blockquote", "**Blockquote** — Block quotation for cited text."),
    // Document/PDF
    ("Document", "**Document** — Root element for PDF document output."),
    ("Section", "**Section** — A section within a PDF document."),
    ("Paragraph", "**Paragraph** — A paragraph within a PDF document."),
    ("PageBreak", "**PageBreak** — Forces a page break in PDF output."),
    ("Header", "**Header** — Repeated page header in PDF output."),
    ("Footer", "**Footer** — Repeated page footer in PDF output."),
    // Routing
    ("Router", "**Router** — Client-side router outlet that renders the matched page."),
    ("Route", "**Route** — Route definition within a Router."),
];

// ---------------------------------------------------------------------------
// Keyword documentation
// ---------------------------------------------------------------------------

const KEYWORD_DOCS: &[(&str, &str)] = &[
    ("Page", "**Page** — Declares a page with a URL path and optional title.\n\n```\nPage Home (path: \"/\", title: \"Home\") { ... }\n```"),
    ("Component", "**Component** — Declares a reusable component with typed props.\n\n```\nComponent Card (title: String, count: Number?) { ... }\n```"),
    ("Store", "**Store** — Declares a shared state store with state, derived, and actions.\n\n```\nStore Counter {\n  state count = 0\n  action increment() { count = count + 1 }\n}\n```"),
    ("App", "**App** — The root application declaration defining the top-level layout.\n\n```\nApp { Navbar { ... } Router Footer { ... } }\n```"),
    ("state", "**state** — Declares a reactive state variable. Changes trigger re-renders.\n\n```\nstate count = 0\nstate name = \"World\"\n```"),
    ("derived", "**derived** — Declares a computed value that updates when dependencies change.\n\n```\nderived doubled = count * 2\n```"),
    ("effect", "**effect** — Side-effect block that re-runs when its dependencies change.\n\n```\neffect { log(count) }\n```"),
    ("action", "**action** — Named function that can mutate state.\n\n```\naction increment() { count = count + 1 }\n```"),
    ("if", "**if** — Conditional rendering. Optionally chain `else` / `else if`.\n\n```\nif isLoggedIn { Text(\"Welcome!\") }\nelse { Button(\"Log in\") }\n```"),
    ("for", "**for** — Iterate over a list and render UI for each item.\n\n```\nfor item in items { Text(item.name) }\n```"),
    ("show", "**show** — Conditionally show/hide an element (stays in DOM, toggles visibility).\n\n```\nshow isVisible { Modal(title: \"Info\") { ... } }\n```"),
    ("use", "**use** — Import a Store into the current scope.\n\n```\nuse CounterStore\n```"),
    ("fetch", "**fetch** — Async data fetching with loading/error/success blocks.\n\n```\nfetch data from \"/api/items\" {\n  loading { Spinner }\n  error(e) { Alert(e, danger) }\n  success { List { ... } }\n}\n```"),
    ("navigate", "**navigate** — Programmatic client-side navigation.\n\n```\nnavigate \"/dashboard\"\n```"),
    ("log", "**log** — Log an expression to the browser console.\n\n```\nlog(count)\n```"),
    ("animate", "**animate** — Apply enter/exit animations.\n\n```\nanimate target \"fadeIn\"\n```"),
    ("style", "**style** — Inline style block for custom CSS properties.\n\n```\nstyle { background: \"#f0f0f0\"; padding: \"1rem\" }\n```"),
    ("transition", "**transition** — CSS transition block for animated property changes.\n\n```\ntransition { opacity 300ms ease-in-out }\n```"),
];

// ---------------------------------------------------------------------------
// Modifier documentation
// ---------------------------------------------------------------------------

const MODIFIER_DOCS: &[(&str, &str)] = &[
    ("primary", "**primary** — Primary accent style (buttons, badges, etc.)"),
    ("secondary", "**secondary** — Secondary / muted style variant"),
    ("outline", "**outline** — Bordered outline variant with transparent background"),
    ("ghost", "**ghost** — Ghost variant with no background or border"),
    ("danger", "**danger** — Destructive / error style (red)"),
    ("warning", "**warning** — Warning style (amber/yellow)"),
    ("success", "**success** — Success style (green)"),
    ("info", "**info** — Informational style (blue)"),
    ("small", "**small** — Reduced size variant"),
    ("large", "**large** — Increased size variant"),
    ("disabled", "**disabled** — Visually and functionally disabled"),
    ("full-width", "**full-width** — Stretches to fill the parent width"),
    ("rounded", "**rounded** — Applies rounded corners"),
    ("centered", "**centered** — Centers the content"),
    ("bold", "**bold** — Bold text weight"),
    ("italic", "**italic** — Italic text style"),
    ("underline", "**underline** — Underlined text"),
    ("strikethrough", "**strikethrough** — Strikethrough text"),
    ("h1", "**h1** — Heading level 1 (largest)"),
    ("h2", "**h2** — Heading level 2"),
    ("h3", "**h3** — Heading level 3"),
    ("h4", "**h4** — Heading level 4"),
    ("h5", "**h5** — Heading level 5"),
    ("h6", "**h6** — Heading level 6 (smallest)"),
    ("ordered", "**ordered** — Ordered (numbered) list"),
    ("unordered", "**unordered** — Unordered (bulleted) list"),
    ("text", "**text** — Plain text input type"),
    ("email", "**email** — Email input type"),
    ("password", "**password** — Password input type (masked)"),
    ("number", "**number** — Numeric input type"),
    ("horizontal", "**horizontal** — Horizontal orientation"),
    ("vertical", "**vertical** — Vertical orientation"),
];

// ---------------------------------------------------------------------------
// Event documentation
// ---------------------------------------------------------------------------

const EVENT_DOCS: &[(&str, &str)] = &[
    ("click", "**on:click** — Fires when the element is clicked."),
    ("dblclick", "**on:dblclick** — Fires on double click."),
    ("input", "**on:input** — Fires on every input value change (real-time)."),
    ("change", "**on:change** — Fires when the value is committed (on blur or enter)."),
    ("submit", "**on:submit** — Fires when a form is submitted."),
    ("focus", "**on:focus** — Fires when the element gains focus."),
    ("blur", "**on:blur** — Fires when the element loses focus."),
    ("keydown", "**on:keydown** — Fires when a key is pressed down."),
    ("keyup", "**on:keyup** — Fires when a key is released."),
    ("mouseenter", "**on:mouseenter** — Fires when the mouse enters the element."),
    ("mouseleave", "**on:mouseleave** — Fires when the mouse leaves the element."),
    ("scroll", "**on:scroll** — Fires when the element is scrolled."),
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn provide_hover(source: &str, position: Position) -> Option<Hover> {
    let word = word_at_position(source, position)?;

    // Try components first
    if let Some(doc) = lookup(&word, COMPONENT_DOCS) {
        return Some(make_hover(doc));
    }

    // Try keywords
    if let Some(doc) = lookup(&word, KEYWORD_DOCS) {
        return Some(make_hover(doc));
    }

    // Try modifiers
    if let Some(doc) = lookup(&word, MODIFIER_DOCS) {
        return Some(make_hover(doc));
    }

    // Try events (strip "on:" prefix if present)
    let event_name = word.strip_prefix("on:").unwrap_or(&word);
    if let Some(doc) = lookup(event_name, EVENT_DOCS) {
        return Some(make_hover(doc));
    }

    None
}

fn lookup(key: &str, table: &[(&str, &str)]) -> Option<String> {
    table
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.to_string())
}

fn make_hover(content: String) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: None,
    }
}

/// Extract the word under the cursor position.
fn word_at_position(source: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let line = lines.get(position.line as usize)?;
    let col = position.character as usize;

    if col > line.len() {
        return None;
    }

    let bytes = line.as_bytes();

    // Find word start
    let mut start = col;
    while start > 0 {
        let ch = bytes[start - 1] as char;
        if ch.is_alphanumeric() || ch == '_' || ch == ':' {
            start -= 1;
        } else if ch == '.' {
            // Include dot for sub-component references like "Card.Header"
            start -= 1;
        } else {
            break;
        }
    }

    // Find word end
    let mut end = col;
    while end < bytes.len() {
        let ch = bytes[end] as char;
        if ch.is_alphanumeric() || ch == '_' {
            end += 1;
        } else {
            break;
        }
    }

    if start == end {
        return None;
    }

    Some(line[start..end].to_string())
}
