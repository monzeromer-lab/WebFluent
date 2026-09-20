//! The language reference the server shows: every built-in component with the
//! arguments and modifiers it takes, the keywords, the modifiers, the events,
//! the design tokens and the icons.
//!
//! The usage lines follow `AGENTS.md` — the language reference the compiler is
//! held to — and the argument handling in `codegen::js`. When the language
//! grows, both of those change first and this file follows; the vocabulary
//! itself (`MODIFIER_KEYWORDS`, `Parser::PSEUDO_STATES`) is read from the
//! compiler rather than copied.

use webfluent::parser::Parser;
use webfluent::parser::vocabulary::MODIFIER_KEYWORDS;

/// A named argument a component reads.
pub struct ArgDoc {
    pub name: &'static str,
    pub doc: &'static str,
}

const fn arg(name: &'static str, doc: &'static str) -> ArgDoc {
    ArgDoc { name, doc }
}

pub struct ComponentDoc {
    pub name: &'static str,
    pub group: &'static str,
    pub summary: &'static str,
    /// The shape of a call, as `AGENTS.md` writes it.
    pub usage: &'static str,
    /// What the positional arguments mean; empty when there are none.
    pub positional: &'static str,
    /// The named arguments this component reads. Any other `name: value`
    /// becomes an HTML attribute on its root element.
    pub args: &'static [ArgDoc],
    /// The modifier words that select a variant, size or shape of it.
    pub modifiers: &'static [&'static str],
    /// Its sub-components, written `Name.Child`.
    pub children: &'static [&'static str],
}

// Named arguments several components share.
const BIND: ArgDoc = arg(
    "bind",
    "State variable the control reads and writes (two-way)",
);
const LABEL: ArgDoc = arg("label", "Visible label; also the control's accessible name");
const HINT: ArgDoc = arg(
    "hint",
    "Help text under the control, linked to it by `aria-describedby`",
);
const ERROR: ArgDoc = arg(
    "error",
    "Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid`",
);
const GAP: ArgDoc = arg(
    "gap",
    "Space between children: `xs`, `sm`, `md`, `lg` or `xl`",
);
const ALIGN: ArgDoc = arg(
    "align",
    "Cross-axis alignment: `start`, `center`, `end`, `stretch` or `baseline`",
);
const JUSTIFY: ArgDoc = arg(
    "justify",
    "Main-axis distribution: `start`, `center`, `end`, `between`, `around` or `evenly`",
);
const TO: ArgDoc = arg(
    "to",
    "Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current=\"page\"`",
);
const ACTIVE: ArgDoc = arg(
    "active",
    "`\"prefix\"` also marks the link active on routes beneath `to`",
);
const VISIBLE: ArgDoc = arg("visible", "State flag that opens and closes it");
const TITLE: ArgDoc = arg("title", "Heading shown at the top");

const COLOR_VARIANTS: &[&str] = &[
    "primary",
    "secondary",
    "success",
    "danger",
    "warning",
    "info",
];
const SIZES: &[&str] = &["small", "large"];
const SPACING: &[&str] = &["xs", "sm", "md", "lg", "xl"];

pub const COMPONENTS: &[ComponentDoc] = &[
    // ─── Layout ───────────────────────────────────────────────────────────
    ComponentDoc {
        name: "Container",
        group: "Layout",
        summary: "Centred wrapper with a maximum width and horizontal padding.",
        usage: "Container { … }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Row",
        group: "Layout",
        summary: "Horizontal flex layout.",
        usage: "Row(gap: md, align: center, justify: between) { … }",
        positional: "",
        args: &[GAP, ALIGN, JUSTIFY],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Column",
        group: "Layout",
        summary: "A child of the 12-column grid, spanning `span` columns.",
        usage: "Column(span: 6) { … }",
        positional: "",
        args: &[arg("span", "How many of the 12 grid columns to span")],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Grid",
        group: "Layout",
        summary: "CSS grid with a fixed number of columns.",
        usage: "Grid(columns: 3, gap: md) { … }",
        positional: "",
        args: &[
            arg("columns", "Number of equal columns"),
            GAP,
            ALIGN,
            JUSTIFY,
        ],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Stack",
        group: "Layout",
        summary: "Vertical flex layout.",
        usage: "Stack(gap: md) { … }",
        positional: "",
        args: &[GAP, ALIGN, JUSTIFY],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Spacer",
        group: "Layout",
        summary: "Vertical space. `md` when no size is given.",
        usage: "Spacer()  ·  Spacer(sm)  ·  Spacer(xl)",
        positional: "",
        args: &[],
        modifiers: SPACING,
        children: &[],
    },
    ComponentDoc {
        name: "Divider",
        group: "Layout",
        summary: "Horizontal rule.",
        usage: "Divider()",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    // ─── Navigation ───────────────────────────────────────────────────────
    ComponentDoc {
        name: "Navbar",
        group: "Navigation",
        summary: "Top navigation bar with brand, links and action areas.",
        usage: "Navbar { Navbar.Brand { … } Navbar.Links { … } Navbar.Actions { … } }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &["Navbar.Brand", "Navbar.Links", "Navbar.Actions"],
    },
    ComponentDoc {
        name: "Sidebar",
        group: "Navigation",
        summary: "Side navigation. A `Sidebar.Item` whose `to` matches the current route is marked `.active` and `aria-current=\"page\"`.",
        usage: "Sidebar { Sidebar.Header { … } Sidebar.Item(to: \"/\", icon: \"home\") { Text(\"Home\") } Sidebar.Divider() }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &["Sidebar.Header", "Sidebar.Item", "Sidebar.Divider"],
    },
    ComponentDoc {
        name: "Link",
        group: "Navigation",
        summary: "Client-side navigation link; its block is the link text.",
        usage: "Link(to: \"/about\") { Text(\"About\") }",
        positional: "",
        args: &[TO, ACTIVE],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Tabs",
        group: "Navigation",
        summary: "Tab strip switching between `TabPage` children.",
        usage: "Tabs { TabPage(\"Tab 1\") { … } TabPage(\"Tab 2\") { … } }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "TabPage",
        group: "Navigation",
        summary: "One tab inside `Tabs`.",
        usage: "TabPage(\"Label\") { … }",
        positional: "the tab's label",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Breadcrumb",
        group: "Navigation",
        summary: "Breadcrumb trail. An item without `to` is the current page.",
        usage: "Breadcrumb { Breadcrumb.Item(to: \"/\") { Text(\"Home\") } Breadcrumb.Item { Text(\"Current\") } }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &["Breadcrumb.Item"],
    },
    ComponentDoc {
        name: "Menu",
        group: "Navigation",
        summary: "Menu opened by a trigger button.",
        usage: "Menu(trigger: \"Options\") { Menu.Item { … } }",
        positional: "",
        args: &[arg("trigger", "Label of the button that opens the menu")],
        modifiers: &[],
        children: &["Menu.Item"],
    },
    // ─── Data display ─────────────────────────────────────────────────────
    ComponentDoc {
        name: "Card",
        group: "Data display",
        summary: "Card surface with optional header, body and footer sections.",
        usage: "Card(elevated) { Card.Header { … } Card.Body { … } Card.Footer { … } }",
        positional: "",
        args: &[],
        modifiers: &["elevated", "outlined", "flat"],
        children: &["Card.Header", "Card.Body", "Card.Footer"],
    },
    ComponentDoc {
        name: "Table",
        group: "Data display",
        summary: "Data table. Cells inside `Thead` render as `<th scope=\"col\">`; `caption` is the table's accessible name, visually hidden.",
        usage: "Table(caption: \"Deployments\") { Thead { Trow { Tcell(\"Col\") } } Tbody { Trow { Tcell(\"Val\") } } }",
        positional: "",
        args: &[arg(
            "caption",
            "Accessible name of the table (rendered visually hidden)",
        )],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Thead",
        group: "Data display",
        summary: "Table header section; its cells become `<th scope=\"col\">`.",
        usage: "Thead { Trow { Tcell(\"Column\") } }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Tbody",
        group: "Data display",
        summary: "Table body section.",
        usage: "Tbody { for row in rows { Trow { Tcell(row.name) } } }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Trow",
        group: "Data display",
        summary: "Table row.",
        usage: "Trow { Tcell(\"Value\") }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Tcell",
        group: "Data display",
        summary: "Table cell. `header` makes it a `<th>` anywhere, as `Thead` does for its cells.",
        usage: "Tcell(\"Value\")  ·  Tcell(\"Build\", header)",
        positional: "the cell's content",
        args: &[],
        modifiers: &["header"],
        children: &[],
    },
    ComponentDoc {
        name: "List",
        group: "Data display",
        summary: "List of its children; `ordered` for a numbered list.",
        usage: "List { Text(\"Item 1\") Text(\"Item 2\") }  ·  List(ordered) { … }",
        positional: "",
        args: &[],
        modifiers: &["ordered"],
        children: &[],
    },
    ComponentDoc {
        name: "Badge",
        group: "Data display",
        summary: "Small status label.",
        usage: "Badge(\"Label\", primary)",
        positional: "the badge text",
        args: &[],
        modifiers: &["primary", "success", "danger", "warning", "info", "pill"],
        children: &[],
    },
    ComponentDoc {
        name: "Tag",
        group: "Data display",
        summary: "Compact chip.",
        usage: "Tag(\"JavaScript\")",
        positional: "the tag text",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Avatar",
        group: "Data display",
        summary: "A picture of a person, or their initials.",
        usage: "Avatar(src: \"/photo.jpg\", alt: \"User\")  ·  Avatar(initials: \"MO\", primary)",
        positional: "",
        args: &[
            arg("src", "Image URL"),
            arg("alt", "Text alternative for the image"),
            arg("initials", "Letters shown when there is no image"),
        ],
        modifiers: &["small", "large", "primary"],
        children: &[],
    },
    ComponentDoc {
        name: "Tooltip",
        group: "Data display",
        summary: "Shows `text` when its child is hovered.",
        usage: "Tooltip(text: \"Click to save\") { Button(\"Save\", primary) }",
        positional: "",
        args: &[arg("text", "The tooltip text")],
        modifiers: &[],
        children: &[],
    },
    // ─── Form & input ─────────────────────────────────────────────────────
    ComponentDoc {
        name: "Input",
        group: "Form",
        summary: "Text field. The first positional word is the input type.",
        usage: "Input(text, bind: name, label: \"Name\", hint: \"As on your passport\", error: nameError)",
        positional: "the type: `text`, `email`, `password`, `number`, `search`, `tel`, `url`, `date`, `time` or `color`",
        args: &[
            BIND,
            LABEL,
            HINT,
            ERROR,
            arg("placeholder", "Hint shown while the field is empty"),
            arg("required", "Whether the form refuses to submit without it"),
            arg("disabled", "Whether the field is inert"),
        ],
        modifiers: &[
            "text", "email", "password", "number", "search", "tel", "url", "date", "time", "color",
        ],
        children: &[],
    },
    ComponentDoc {
        name: "Select",
        group: "Form",
        summary: "Drop-down of `Option` children.",
        usage: "Select(bind: role, label: \"Choose\") { Option(\"admin\", \"Admin\") }",
        positional: "",
        args: &[
            BIND,
            LABEL,
            HINT,
            ERROR,
            arg("disabled", "Whether the control is inert"),
        ],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Option",
        group: "Form",
        summary: "One choice in a `Select`: the value, then the visible label (one positional is both).",
        usage: "Option(\"val1\", \"Label 1\")",
        positional: "the value, then the label",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Checkbox",
        group: "Form",
        summary: "Checkbox bound to a boolean state.",
        usage: "Checkbox(bind: agreed, label: \"Agree\")",
        positional: "",
        args: &[BIND, LABEL, arg("disabled", "Whether the control is inert")],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Radio",
        group: "Form",
        summary: "One choice of a group; every radio bound to the same state is one group.",
        usage: "Radio(bind: choice, value: \"opt1\", label: \"Option 1\")",
        positional: "",
        args: &[
            BIND,
            arg("value", "The value `bind` takes when this one is chosen"),
            LABEL,
        ],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Switch",
        group: "Form",
        summary: "On/off toggle bound to a boolean state.",
        usage: "Switch(bind: enabled, label: \"Enable\")",
        positional: "",
        args: &[BIND, LABEL, arg("disabled", "Whether the control is inert")],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Slider",
        group: "Form",
        summary: "Range input. `aria-*` arguments reach the input; with `aria-valuetext:` the raw number is not shown beside the track.",
        usage: "Slider(bind: volume, min: 0, max: 100, step: 1, label: \"Volume\")",
        positional: "",
        args: &[
            BIND,
            arg("min", "Lowest value"),
            arg("max", "Highest value"),
            arg("step", "Granularity"),
            LABEL,
            arg(
                "aria-valuetext",
                "Spoken value; also hides the raw number beside the track",
            ),
        ],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "DatePicker",
        group: "Form",
        summary: "Date input.",
        usage: "DatePicker(bind: start, label: \"Start Date\", min: \"2026-01-01\")",
        positional: "",
        args: &[
            BIND,
            LABEL,
            arg("min", "Earliest date, `YYYY-MM-DD`"),
            arg("max", "Latest date, `YYYY-MM-DD`"),
        ],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "FileUpload",
        group: "Form",
        summary: "File input; handle the chosen files in `on:change`.",
        usage: "FileUpload(accept: \"image/*\", label: \"Upload Photo\") { on:change { handleFile(event) } }",
        positional: "",
        args: &[
            arg(
                "accept",
                "Accepted types, e.g. `\"image/*\"` or `\".pdf,.doc\"`",
            ),
            LABEL,
        ],
        modifiers: &["multiple"],
        children: &[],
    },
    ComponentDoc {
        name: "Form",
        group: "Form",
        summary: "Groups controls; `on:submit` runs when it is submitted.",
        usage: "Form { … on:submit { save() } }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    // ─── Feedback ─────────────────────────────────────────────────────────
    ComponentDoc {
        name: "Alert",
        group: "Feedback",
        summary: "Inline notice.",
        usage: "Alert(\"Message\", success)",
        positional: "the message",
        args: &[],
        modifiers: &["success", "danger", "warning", "info", "dismissible"],
        children: &[],
    },
    ComponentDoc {
        name: "Toast",
        group: "Feedback",
        summary: "Temporary notification.",
        usage: "Toast(\"Saved!\", success)",
        positional: "the message",
        args: &[],
        modifiers: &["success", "danger", "warning", "info"],
        children: &[],
    },
    ComponentDoc {
        name: "Modal",
        group: "Feedback",
        summary: "Modal `<dialog>`, opened by a state flag; the browser supplies focus trapping and Escape.",
        usage: "Modal(visible: showModal, title: \"Title\") { … Modal.Footer { … } }",
        positional: "",
        args: &[VISIBLE, TITLE],
        modifiers: &[],
        children: &["Modal.Footer"],
    },
    ComponentDoc {
        name: "Dialog",
        group: "Feedback",
        summary: "Confirmation `<dialog>`, opened by a state flag.",
        usage: "Dialog(visible: show, title: \"Confirm\") { … }",
        positional: "",
        args: &[VISIBLE, TITLE],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Spinner",
        group: "Feedback",
        summary: "Loading indicator.",
        usage: "Spinner()  ·  Spinner(large)",
        positional: "",
        args: &[],
        modifiers: &["small", "large"],
        children: &[],
    },
    ComponentDoc {
        name: "Progress",
        group: "Feedback",
        summary: "Progress bar; a `value` that reads state moves with it.",
        usage: "Progress(value: 75, max: 100)",
        positional: "",
        args: &[arg("value", "Current value"), arg("max", "Value at 100%")],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Skeleton",
        group: "Feedback",
        summary: "Placeholder shape shown while content loads.",
        usage: "Skeleton(height: \"20px\", width: \"200px\")  ·  Skeleton(circle, size: \"48px\")",
        positional: "",
        args: &[
            arg("height", "CSS height"),
            arg("width", "CSS width"),
            arg("size", "Diameter of a `circle`"),
        ],
        modifiers: &["circle"],
        children: &[],
    },
    // ─── Actions ──────────────────────────────────────────────────────────
    ComponentDoc {
        name: "Button",
        group: "Actions",
        summary: "Button. Action statements in its block are its click handler; elements in the block are its content.",
        usage: "Button(\"Label\", primary, large) { doSave() }",
        positional: "the label",
        args: &[
            arg("type", "`\"submit\"` or `\"reset\"` inside a `Form`"),
            arg("disabled", "Whether the button is inert"),
        ],
        modifiers: &[
            "primary",
            "secondary",
            "success",
            "danger",
            "warning",
            "info",
            "small",
            "large",
            "full",
            "rounded",
            "pill",
            "outlined",
            "submit",
            "reset",
        ],
        children: &[],
    },
    ComponentDoc {
        name: "IconButton",
        group: "Actions",
        summary: "Icon-only button. `label` is its accessible name, never visible text.",
        usage: "IconButton(icon: \"close\", label: \"Close\")  ·  IconButton(icon: \"edit\", label: \"Edit\", primary) { editItem() }",
        positional: "",
        args: &[
            arg("icon", "One of the built-in icon names"),
            arg("label", "Accessible name (`aria-label` and `title`)"),
            arg("disabled", "Whether the button is inert"),
        ],
        modifiers: &["small", "large", "primary", "danger"],
        children: &[],
    },
    ComponentDoc {
        name: "ButtonGroup",
        group: "Actions",
        summary: "Buttons joined into one row.",
        usage: "ButtonGroup { Button(\"A\") Button(\"B\") }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Dropdown",
        group: "Actions",
        summary: "Button that opens a list of `Dropdown.Item`s.",
        usage: "Dropdown(label: \"Actions\") { Dropdown.Item { … } }",
        positional: "",
        args: &[arg("label", "Label of the button that opens it")],
        modifiers: &[],
        children: &["Dropdown.Item"],
    },
    // ─── Media ────────────────────────────────────────────────────────────
    ComponentDoc {
        name: "Image",
        group: "Media",
        summary: "Image; `alt` is required for accessibility (rule A01).",
        usage: "Image(src: \"/photo.jpg\", alt: \"Description\")",
        positional: "",
        args: &[
            arg("src", "Image URL"),
            arg(
                "alt",
                "Text alternative; empty for a purely decorative image",
            ),
            arg("width", "Intrinsic width"),
            arg("height", "Intrinsic height"),
            arg(
                "loading",
                "`\"lazy\"` or `\"eager\"`; unset, the first image on a page is eager at high priority and the rest lazy",
            ),
        ],
        modifiers: &["rounded", "circle"],
        children: &[],
    },
    ComponentDoc {
        name: "Video",
        group: "Media",
        summary: "Video player.",
        usage: "Video(src: \"/video.mp4\", controls: true)",
        positional: "",
        args: &[
            arg("src", "Video URL"),
            arg("controls", "Show the browser's playback controls"),
            arg("autoplay", "Start playing when shown"),
        ],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Icon",
        group: "Media",
        summary: "One of the 30 built-in SVG icons, rendered inline.",
        usage: "Icon(\"home\")  ·  Icon(\"search\", large, primary)",
        positional: "the icon name",
        args: &[],
        modifiers: &[
            "small",
            "large",
            "primary",
            "secondary",
            "success",
            "danger",
            "warning",
            "info",
            "muted",
        ],
        children: &[],
    },
    ComponentDoc {
        name: "Carousel",
        group: "Media",
        summary: "Slide track with dots; optional autoplay.",
        usage: "Carousel(autoplay: true, interval: 5000) { Carousel.Slide { Image(src: \"…\", alt: \"…\") } }",
        positional: "",
        args: &[
            arg("autoplay", "Advance automatically"),
            arg("interval", "Milliseconds between slides"),
        ],
        modifiers: &[],
        children: &["Carousel.Slide"],
    },
    // ─── Typography ───────────────────────────────────────────────────────
    ComponentDoc {
        name: "Text",
        group: "Typography",
        summary: "A paragraph of text. `{…}` in the string reads state and stays live.",
        usage: "Text(\"Hello, {name}\", bold, muted, center)",
        positional: "the text",
        args: &[],
        modifiers: &[
            "bold",
            "italic",
            "underline",
            "uppercase",
            "lowercase",
            "left",
            "center",
            "right",
            "muted",
            "small",
            "large",
            "heading",
            "subtitle",
            "primary",
            "secondary",
            "success",
            "danger",
            "warning",
            "info",
        ],
        children: &[],
    },
    ComponentDoc {
        name: "Heading",
        group: "Typography",
        summary: "Heading; the level modifier picks the tag (`h2` when none is given).",
        usage: "Heading(\"Title\", h1)",
        positional: "the heading text",
        args: &[],
        modifiers: &["h1", "h2", "h3", "h4", "h5", "h6", "center", "muted"],
        children: &[],
    },
    ComponentDoc {
        name: "Code",
        group: "Typography",
        summary: "Inline code, or a code block with `block`. Escape literal braces as `\\{` and `\\}`.",
        usage: "Code(\"const x = 1\", block)",
        positional: "the code",
        args: &[],
        modifiers: &["block"],
        children: &[],
    },
    ComponentDoc {
        name: "Blockquote",
        group: "Typography",
        summary: "Quotation; its block is the quoted content.",
        usage: "Blockquote { Text(\"Quote text\") }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    // ─── Document (PDF) ───────────────────────────────────────────────────
    ComponentDoc {
        name: "Document",
        group: "PDF",
        summary: "Root of a PDF document.",
        usage: "Document(page_size: \"A4\") { Header { … } Footer { … } Section { … } }",
        positional: "",
        args: &[arg("page_size", "`A4`, `A3`, `A5`, `Letter` or `Legal`")],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Section",
        group: "PDF",
        summary: "Groups content with spacing; on the web, a `<section>`.",
        usage: "Section { … }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Paragraph",
        group: "PDF",
        summary: "Block of text with paragraph spacing.",
        usage: "Paragraph(\"Text\")",
        positional: "the text",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Header",
        group: "PDF",
        summary: "Repeated at the top of every PDF page; a `<header>` on the web.",
        usage: "Header { Text(\"Company Inc.\", muted, small, right) }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Footer",
        group: "PDF",
        summary: "Repeated at the bottom of every PDF page; a `<footer>` on the web (`App { … Footer }`).",
        usage: "Footer { Text(\"Confidential\", muted, small, center) }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "PageBreak",
        group: "PDF",
        summary: "Forces a new PDF page.",
        usage: "PageBreak()",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    // ─── Slides ───────────────────────────────────────────────────────────
    ComponentDoc {
        name: "Presentation",
        group: "Slides",
        summary: "Root of a slide deck; its children must be slides.",
        usage: "Presentation { TitleSlide(\"Title\", \"Subtitle\") Slide { … } }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Slide",
        group: "Slides",
        summary: "One slide, one PDF page; content is top-aligned and clipped at the bottom margin.",
        usage: "Slide { Heading(\"Highlights\", h1) List { … } }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "TitleSlide",
        group: "Slides",
        summary: "Cover slide: title and subtitle, vertically centred.",
        usage: "TitleSlide(\"Title\", \"Subtitle\")",
        positional: "the title, then the subtitle",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "SectionSlide",
        group: "Slides",
        summary: "Full-bleed coloured band with a centred label.",
        usage: "SectionSlide(\"Q2 Plan\", primary)",
        positional: "the label",
        args: &[],
        modifiers: &["primary", "success", "danger", "warning", "info"],
        children: &[],
    },
    ComponentDoc {
        name: "TwoColumn",
        group: "Slides",
        summary: "Two equal columns; takes exactly two `Container` children.",
        usage: "TwoColumn { Container { … } Container { … } }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "ImageSlide",
        group: "Slides",
        summary: "Image slide with an optional caption.",
        usage: "ImageSlide(src: \"chart.png\", caption: \"Revenue by region\")",
        positional: "",
        args: &[
            arg("src", "Image path (required)"),
            arg("caption", "Caption under the image"),
        ],
        modifiers: &[],
        children: &[],
    },
    // ─── Routing ──────────────────────────────────────────────────────────
    ComponentDoc {
        name: "Router",
        group: "Routing",
        summary: "Where the current page renders. May sit at any depth inside `App`.",
        usage: "Router { Route(path: \"/\", page: Home) Route(path: \"*\", page: NotFound) }",
        positional: "",
        args: &[],
        modifiers: &[],
        children: &[],
    },
    ComponentDoc {
        name: "Route",
        group: "Routing",
        summary: "Maps a path to a page. `/user/:id` captures `params.id`; `*` catches everything else.",
        usage: "Route(path: \"/about\", page: About)",
        positional: "",
        args: &[
            arg("path", "URL pattern"),
            arg("page", "The `Page` to render"),
        ],
        modifiers: &[],
        children: &[],
    },
];

pub fn component(name: &str) -> Option<&'static ComponentDoc> {
    COMPONENTS.iter().find(|c| c.name == name)
}

/// The parent's doc for a `Parent.Child` name, when `Child` is one it lists.
pub fn sub_component(name: &str) -> Option<(&'static ComponentDoc, &'static str)> {
    let (parent, _) = name.split_once('.')?;
    let doc = component(parent)?;
    doc.children
        .iter()
        .find(|child| **child == name)
        .map(|child| (doc, *child))
}

/// The attributes a `Page` header takes.
pub const PAGE_ATTRIBUTES: &[ArgDoc] = &[
    arg("path", "URL route. `/user/:id` captures `params.id`"),
    arg(
        "title",
        "Browser tab title, and the heading of a search result",
    ),
    arg(
        "description",
        "The snippet a search result and a link preview show",
    ),
    arg("image", "Link-preview image, site-relative or absolute"),
    arg("type", "`\"website\"` (default) or `\"article\"`"),
    arg(
        "noindex",
        "Bare flag: keep the page out of search results and the sitemap",
    ),
    arg("guard", "Expression that must hold for the route to render"),
    arg("redirect", "Where to send the visitor when the guard fails"),
];

/// The arguments an `animate(…)` clause takes after the enter animation.
pub const ANIMATE_ARGUMENTS: &[ArgDoc] = &[
    arg("duration", "How long the animation runs, e.g. `\"300ms\"`"),
    arg("delay", "Wait before it starts"),
    arg("stagger", "Delay added per list item, e.g. `\"50ms\"`"),
    arg("easing", "Timing function"),
];

/// The options a `fetch … from url (…)` group takes.
pub const FETCH_OPTIONS: &[ArgDoc] = &[
    arg("method", "HTTP method, e.g. `\"POST\"`"),
    arg("headers", "Map of request headers; quote hyphenated keys"),
    arg("body", "Request body, sent as JSON"),
];

pub struct KeywordDoc {
    pub name: &'static str,
    pub summary: &'static str,
    pub example: &'static str,
    /// Where it may appear.
    pub place: Place,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Place {
    TopLevel,
    Body,
    Theme,
    Fetch,
    Style,
}

pub const KEYWORDS: &[KeywordDoc] = &[
    KeywordDoc {
        name: "Page",
        summary: "A routed page. Every page needs a `path`.",
        example: "Page Home (path: \"/\", title: \"Home\") {\n    Container { Heading(\"Welcome\", h1) }\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "Component",
        summary: "A reusable component with typed props: `String`, `Number`, `Bool`, `List`, `Map`; `name?:` is optional, `= value` a default. Called with named arguments; `children` places the caller's block.",
        example: "Component UserCard (name: String, active: Bool = true) {\n    Card { Text(name, bold) }\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "Store",
        summary: "Shared reactive state: `state`, `derived` and `action`s, reached as `Name.member` after `use Name`.",
        example: "Store CartStore {\n    state items = []\n    derived count = items.length\n    action clear() { items = [] }\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "Theme",
        summary: "Design tokens layered over the baseline; every token becomes `--name` on `:root`. One theme is used automatically; pick among several with `\"theme\": { \"name\": … }`.",
        example: "Theme Brand {\n    token color-primary: \"#3B82F6\"\n    token radius-md: \"0.5rem\"\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "App",
        summary: "The root: navigation, the `Router`, footer. The `Router` may sit at any depth.",
        example: "App {\n    Navbar { … }\n    Router { Route(path: \"/\", page: Home) }\n    Footer\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "token",
        summary: "A design token inside a `Theme`; readable from any style as `var(--name)`.",
        example: "token color-primary: \"#3B82F6\"",
        place: Place::Theme,
    },
    KeywordDoc {
        name: "state",
        summary: "A reactive variable. Every element that reads it updates when it changes.",
        example: "state count = 0",
        place: Place::Body,
    },
    KeywordDoc {
        name: "derived",
        summary: "A computed value, recomputed when the state it reads changes.",
        example: "derived double = count * 2",
        place: Place::Body,
    },
    KeywordDoc {
        name: "effect",
        summary: "Runs once, and again whenever the state it reads changes.",
        example: "effect { log(count) }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "action",
        summary: "A function on a store, page or component; may `return` a value.",
        example: "action add(product: Map) {\n    items = items.concat([product])\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "use",
        summary: "Brings a store into scope: `use CartStore`, then `CartStore.count`.",
        example: "use CartStore",
        place: Place::Body,
    },
    KeywordDoc {
        name: "if",
        summary: "Renders a block when the condition holds; `else if` and `else` follow. `, animate(enter, exit)` animates the change.",
        example: "if isLoggedIn {\n    Text(\"Welcome!\")\n} else {\n    Button(\"Log In\") { navigate(\"/login\") }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "else",
        summary: "The other branch of an `if`; `else if` chains another condition.",
        example: "} else if count == 1 {\n    Text(\"one\")\n} else {\n    Text(\"none\")\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "for",
        summary: "Renders the block once per item; `for item, index in items` also binds the index. `, animate(…, stagger: \"50ms\")` animates entries.",
        example: "for item, index in items {\n    Text(\"{index}. {item.name}\")\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "in",
        summary: "Separates the loop variables from the list in a `for`.",
        example: "for item in items { … }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "show",
        summary: "Keeps the block in the DOM and toggles its visibility.",
        example: "show isVisible {\n    Modal { … }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "fetch",
        summary: "Loads data and renders `loading`, `error (err)` and `success` blocks. Options: `(method: \"POST\", headers: {…}, body: {…})`.",
        example: "fetch users from \"/api/users\" {\n    loading { Spinner() }\n    error (err) { Alert(\"Failed: {err.message}\", danger) }\n    success { for user in users { Text(user.name) } }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "from",
        summary: "Names the URL a `fetch` reads.",
        example: "fetch users from \"/api/users\" { … }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "loading",
        summary: "What a `fetch` renders while the request is in flight.",
        example: "loading { Spinner() }",
        place: Place::Fetch,
    },
    KeywordDoc {
        name: "error",
        summary: "What a `fetch` renders when the request fails; `(err)` names the error.",
        example: "error (err) { Alert(\"Failed: {err.message}\", danger) }",
        place: Place::Fetch,
    },
    KeywordDoc {
        name: "success",
        summary: "What a `fetch` renders once the data arrived, bound to the fetch's name.",
        example: "success { for user in users { Text(user.name) } }",
        place: Place::Fetch,
    },
    KeywordDoc {
        name: "navigate",
        summary: "Goes to a route.",
        example: "navigate(\"/login\")",
        place: Place::Body,
    },
    KeywordDoc {
        name: "log",
        summary: "Writes a value to the browser console.",
        example: "log(count)",
        place: Place::Body,
    },
    KeywordDoc {
        name: "return",
        summary: "Returns a value from an action.",
        example: "return h",
        place: Place::Body,
    },
    KeywordDoc {
        name: "animate",
        summary: "Replays an animation on an element, or — after `if`, `for`, `show` — animates enter and exit.",
        example: "if showPanel, animate(scaleIn, fadeOut) { … }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "children",
        summary: "Where a component places the block its caller passed. The block runs in the caller's scope.",
        example: "Component Panel (title: String) {\n    Card { Heading(title, h3) children }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "style",
        summary: "Inline styles with CSS property names. Nested `hover { }`, `focus { }` and `@media (…) { }` compile to stylesheet rules; `--name:` sets a custom property.",
        example: "style {\n    background: \"var(--brand)\"\n    hover { background: \"var(--brand-hover)\" }\n    @media (max-width: 768px) { display: \"none\" }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "transition",
        summary: "Per-property transitions: property, duration (`200ms`, `fast`, `slow`), optional easing.",
        example: "transition {\n    background 200ms ease\n    transform 150ms spring\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "@media",
        summary: "A responsive rule inside `style { }`, compiled to the stylesheet.",
        example: "@media (max-width: 768px) {\n    display: \"none\"\n}",
        place: Place::Style,
    },
];

pub fn keyword(name: &str) -> Option<&'static KeywordDoc> {
    KEYWORDS.iter().find(|k| k.name == name)
}

/// What a modifier word does, grouped as `AGENTS.md` groups them.
pub fn modifier_doc(word: &str) -> Option<(&'static str, &'static str)> {
    let (group, doc) = match word {
        "small" | "medium" | "large" => ("Size", "Sizes the element."),
        "primary" | "secondary" | "success" | "danger" | "warning" | "info" => {
            ("Color", "Colours the element with the theme's variant.")
        }
        "error" | "loading" => ("State", "Marks the element's state."),
        "rounded" | "pill" | "square" => ("Shape", "Shapes the element's corners."),
        "flat" | "elevated" | "outlined" => ("Elevation", "Sets the element's surface treatment."),
        "full" | "fit" | "fluid" => ("Width", "Sets how wide the element grows."),
        "circle" => (
            "Shape",
            "Makes the element round (`Skeleton`, `Image`, `Avatar`).",
        ),
        "xs" | "sm" | "md" | "lg" | "xl" => ("Spacing", "A spacing step (`Spacer`, `gap:`)."),
        "ordered" => ("Structure", "Numbers the list."),
        "multiple" => ("Structure", "Accepts several files."),
        "header" => ("Structure", "Renders the cell as a `<th>`."),
        "bold" | "italic" | "underline" | "uppercase" | "lowercase" => ("Text", "Styles the text."),
        "left" | "center" | "right" => ("Alignment", "Aligns the text."),
        "heading" | "subtitle" | "muted" => ("Typography", "Sets the text's role."),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => (
            "Heading level",
            "Picks the heading tag, and its place in the outline.",
        ),
        "dismissible" => ("Behaviour", "Adds a close button."),
        "block" => ("Behaviour", "Renders `Code` as a multi-line block."),
        "bordered" => ("Behaviour", "Draws a border."),
        "controls" | "autoplay" => ("Media", "Media playback option."),
        "text" | "email" | "password" | "number" | "search" | "tel" | "url" | "date" | "time"
        | "datetime" | "color" => ("Input type", "Sets the input's type."),
        "submit" | "reset" => ("Button type", "Sets the button's form role."),
        "fadeIn" | "fadeOut" | "slideUp" | "slideDown" | "slideLeft" | "slideRight" | "scaleIn"
        | "scaleOut" | "bounce" | "shake" | "pulse" | "spin" => (
            "Animation",
            "Plays the animation when the element appears, or on enter/exit in `animate(…)`.",
        ),
        "fast" | "slow" => ("Speed", "Animation speed: 150ms or 500ms."),
        _ => return None,
    };
    Some((group, doc))
}

pub fn is_modifier(word: &str) -> bool {
    MODIFIER_KEYWORDS.contains(&word)
}

pub fn pseudo_states() -> &'static [&'static str] {
    Parser::PSEUDO_STATES
}

pub const EVENTS: &[(&str, &str)] = &[
    ("click", "The element was clicked or activated"),
    ("input", "The value changed, on every keystroke"),
    ("change", "The value was committed"),
    ("submit", "The form was submitted"),
    ("focus", "The element received focus"),
    ("blur", "The element lost focus"),
    ("keydown", "A key went down; `event.key` names it"),
    ("keyup", "A key came up"),
    ("keypress", "A character key was pressed"),
    ("mouseenter", "The pointer entered the element"),
    ("mouseleave", "The pointer left the element"),
];

/// The baseline design tokens every theme starts from.
pub const TOKENS: &[(&str, &str)] = &[
    ("color-primary", "Primary brand colour"),
    ("color-secondary", "Secondary colour"),
    ("color-success", "Success colour"),
    ("color-danger", "Danger / error colour"),
    ("color-warning", "Warning colour"),
    ("color-info", "Informational colour"),
    ("color-background", "Page background"),
    ("color-surface", "Card and panel surface"),
    ("color-text", "Body text"),
    ("color-text-muted", "Secondary text"),
    ("color-border", "Borders and dividers"),
    ("font-family", "Body font"),
    ("font-family-mono", "Monospace font"),
    ("font-size-xs", "Font size"),
    ("font-size-sm", "Font size"),
    ("font-size-base", "Font size (1rem)"),
    ("font-size-lg", "Font size"),
    ("font-size-xl", "Font size"),
    ("font-size-2xl", "Font size"),
    ("font-size-3xl", "Font size"),
    ("font-weight-normal", "Font weight"),
    ("font-weight-medium", "Font weight"),
    ("font-weight-bold", "Font weight"),
    ("line-height-tight", "Line height"),
    ("line-height-normal", "Line height"),
    ("line-height-loose", "Line height"),
    ("spacing-xs", "Spacing step"),
    ("spacing-sm", "Spacing step"),
    ("spacing-md", "Spacing step (1rem)"),
    ("spacing-lg", "Spacing step"),
    ("spacing-xl", "Spacing step"),
    ("spacing-2xl", "Spacing step"),
    ("spacing-3xl", "Spacing step"),
    ("radius-none", "Border radius"),
    ("radius-sm", "Border radius"),
    ("radius-md", "Border radius"),
    ("radius-lg", "Border radius"),
    ("radius-xl", "Border radius"),
    ("radius-full", "Pill radius"),
    ("shadow-none", "No shadow"),
    ("shadow-sm", "Shadow"),
    ("shadow-md", "Shadow"),
    ("shadow-lg", "Shadow"),
    ("shadow-xl", "Shadow"),
    ("transition-fast", "Transition duration"),
    ("transition-normal", "Transition duration"),
    ("transition-slow", "Transition duration"),
];

/// The built-in icon names `Icon(…)`, `icon:` and `Sidebar.Item(icon:)` take.
pub const ICONS: &[&str] = &[
    "home",
    "menu",
    "search",
    "close",
    "user",
    "settings",
    "check",
    "plus",
    "minus",
    "edit",
    "trash",
    "star",
    "heart",
    "mail",
    "bell",
    "download",
    "upload",
    "eye",
    "link",
    "calendar",
    "filter",
    "info",
    "warning",
    "arrow-left",
    "arrow-right",
    "chevron-down",
    "chevron-right",
    "chevron-left",
    "logout",
    "copy",
];

/// CSS properties offered inside `style { }`. Any CSS property is accepted;
/// these are the ones worth offering.
pub const CSS_PROPERTIES: &[&str] = &[
    "align-items",
    "align-self",
    "animation",
    "background",
    "background-color",
    "background-image",
    "border",
    "border-bottom",
    "border-color",
    "border-left",
    "border-radius",
    "border-right",
    "border-top",
    "border-width",
    "bottom",
    "box-shadow",
    "color",
    "cursor",
    "display",
    "filter",
    "flex",
    "flex-direction",
    "flex-wrap",
    "font-family",
    "font-size",
    "font-weight",
    "gap",
    "grid-area",
    "grid-column",
    "grid-row",
    "grid-template-columns",
    "grid-template-rows",
    "height",
    "justify-content",
    "justify-self",
    "left",
    "letter-spacing",
    "line-height",
    "margin",
    "margin-bottom",
    "margin-left",
    "margin-right",
    "margin-top",
    "max-height",
    "max-width",
    "min-height",
    "min-width",
    "object-fit",
    "opacity",
    "outline",
    "outline-offset",
    "overflow",
    "overflow-x",
    "overflow-y",
    "padding",
    "padding-bottom",
    "padding-left",
    "padding-right",
    "padding-top",
    "pointer-events",
    "position",
    "right",
    "text-align",
    "text-decoration",
    "text-transform",
    "top",
    "transform",
    "transition",
    "visibility",
    "white-space",
    "width",
    "z-index",
];

/// The easing names a `transition` block accepts.
pub const EASINGS: &[&str] = &[
    "ease",
    "linear",
    "easeIn",
    "easeOut",
    "easeInOut",
    "spring",
    "bouncy",
    "smooth",
];

/// The colour variants, for `SectionSlide`, `Badge` and the like.
pub fn color_variants() -> &'static [&'static str] {
    COLOR_VARIANTS
}

pub fn sizes() -> &'static [&'static str] {
    SIZES
}

#[cfg(test)]
mod registry_tests {
    //! Until this file's component docs are replaced by the compiler's
    //! registry, the two must describe the same language.
    use super::*;
    use webfluent::registry;

    #[test]
    fn every_documented_component_and_argument_is_in_the_registry() {
        for doc in COMPONENTS {
            let sig = registry::component(doc.name)
                .unwrap_or_else(|| panic!("{} is documented but not in the registry", doc.name));
            for arg in doc.args {
                assert!(
                    sig.accepts_named(arg.name),
                    "{}: the documented argument `{}` is not a prop or attribute in the registry",
                    doc.name,
                    arg.name
                );
            }
            for modifier in doc.modifiers {
                assert!(
                    sig.spelling_of_legacy(modifier).is_some()
                        || registry::RETIRED_MODIFIERS.contains(modifier),
                    "{}: the documented modifier `{}` has no spelling in the registry",
                    doc.name,
                    modifier
                );
            }
            for child in doc.children {
                let (_, part) = child.split_once('.').unwrap();
                assert!(
                    registry::part(doc.name, part).is_some(),
                    "{child} is documented but not a part in the registry"
                );
            }
        }
    }
}
