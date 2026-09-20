//! The table of built-in components.
//!
//! Order follows the language reference (`AGENTS.md`): layout, navigation,
//! data display, form, feedback, actions, media, typography, document,
//! slides, routing. Parts (`Card.Header`) follow their owner. Every enum
//! case carries the modifier word it was in the original grammar, which is
//! the word the code generators read; `""` marks a default that has no
//! class of its own.

use super::{AttrFamily, CaseSig, Children, ComponentSig, Ir, Legacy, PropSig, PropType, Sink};

// ─── Construction helpers ────────────────────────────────────────────────

const fn case(name: &'static str, legacy: &'static str, summary: &'static str) -> CaseSig {
    CaseSig {
        name,
        legacy,
        summary,
    }
}

/// A prop written to the root element as an attribute.
const fn attr(name: &'static str, ty: PropType, summary: &'static str) -> PropSig {
    PropSig {
        name,
        ty,
        summary,
        sink: Sink::Attr,
        legacy: Legacy::Attr,
        shorthand: true,
    }
}

/// A prop the component's own emitter reads.
const fn special(name: &'static str, ty: PropType, summary: &'static str) -> PropSig {
    PropSig {
        name,
        ty,
        summary,
        sink: Sink::Special,
        legacy: Legacy::Attr,
        shorthand: true,
    }
}

/// An enum prop whose case becomes a class.
const fn variant(name: &'static str, cases: &'static [CaseSig], summary: &'static str) -> PropSig {
    PropSig {
        name,
        ty: PropType::Enum(cases),
        summary,
        sink: Sink::Class,
        legacy: Legacy::Attr,
        shorthand: true,
    }
}

/// A boolean prop that was a bare modifier word and becomes a class or a tag.
const fn flag(name: &'static str, legacy: &'static str, summary: &'static str) -> PropSig {
    PropSig {
        name,
        ty: PropType::Bool,
        summary,
        sink: Sink::Class,
        legacy: Legacy::Modifier(legacy),
        shorthand: true,
    }
}

/// A boolean prop that becomes an attribute.
const fn attr_flag(name: &'static str, summary: &'static str) -> PropSig {
    PropSig {
        name,
        ty: PropType::Bool,
        summary,
        sink: Sink::Attr,
        legacy: Legacy::Attr,
        shorthand: true,
    }
}

#[allow(clippy::too_many_arguments)]
const fn comp(
    name: &'static str,
    group: &'static str,
    summary: &'static str,
    positional: Option<PropSig>,
    props: &'static [PropSig],
    parts: &'static [&'static str],
    attrs: &'static [AttrFamily],
    children: Children,
) -> ComponentSig {
    ComponentSig {
        name,
        owner: None,
        group,
        summary,
        ir: Ir::BuiltIn(name),
        positional,
        props,
        events: &[],
        parts,
        attrs,
        children,
    }
}

#[allow(clippy::too_many_arguments)]
const fn part(
    owner: &'static str,
    name: &'static str,
    ir: Ir,
    summary: &'static str,
    positional: Option<PropSig>,
    props: &'static [PropSig],
    attrs: &'static [AttrFamily],
    children: Children,
) -> ComponentSig {
    ComponentSig {
        name,
        owner: Some(owner),
        group: "Part",
        summary,
        ir,
        positional,
        props,
        events: &[],
        parts: &[],
        attrs,
        children,
    }
}

// ─── Shared cases ────────────────────────────────────────────────────────

/// The theme's colour variants.
const TONES: &[CaseSig] = &[
    case("primary", "primary", "The brand colour"),
    case("secondary", "secondary", "The secondary colour"),
    case("success", "success", "Green: done, healthy"),
    case("danger", "danger", "Red: destructive, failed"),
    case("warning", "warning", "Amber: attention"),
    case("info", "info", "Blue: neutral notice"),
];
/// The status variants a notice takes.
const STATUS_TONES: &[CaseSig] = &[
    case("success", "success", "Green: done, healthy"),
    case("danger", "danger", "Red: destructive, failed"),
    case("warning", "warning", "Amber: attention"),
    case("info", "info", "Blue: neutral notice"),
];
const BADGE_TONES: &[CaseSig] = &[
    case("primary", "primary", "The brand colour"),
    case("secondary", "secondary", "Grey, for the quiet"),
    case("success", "success", "Green"),
    case("danger", "danger", "Red"),
    case("warning", "warning", "Amber"),
    case("info", "info", "Blue"),
];
/// The band colours a `SectionSlide` paints.
const BAND_TONES: &[CaseSig] = &[
    case("primary", "primary", "The brand colour"),
    case("success", "success", "Green"),
    case("danger", "danger", "Red"),
    case("warning", "warning", "Amber"),
    case("info", "info", "Blue"),
];
const PRIMARY_ONLY: &[CaseSig] = &[case("primary", "primary", "The brand colour")];
const PRIMARY_DANGER: &[CaseSig] = &[
    case("primary", "primary", "The brand colour"),
    case("danger", "danger", "Red: destructive"),
];
/// Small / medium / large, as `small` and `large` were spelled.
const SIZES: &[CaseSig] = &[
    case("sm", "small", "Smaller than the default"),
    case("md", "", "The default size"),
    case("lg", "large", "Larger than the default"),
];
/// The spacing scale.
const SPACES: &[CaseSig] = &[
    case("xs", "xs", "Extra small"),
    case("sm", "sm", "Small"),
    case("md", "md", "Medium, the default"),
    case("lg", "lg", "Large"),
    case("xl", "xl", "Extra large"),
];
// Layout values were never modifier words — `align: center` was a named
// argument — so they carry no legacy spelling.
const ALIGNS: &[CaseSig] = &[
    case("start", "", "Cross-axis start"),
    case("center", "", "Cross-axis centre"),
    case("end", "", "Cross-axis end"),
    case("stretch", "", "Fill the cross axis"),
    case("baseline", "", "Align text baselines"),
];
const JUSTIFIES: &[CaseSig] = &[
    case("start", "", "Pack at the start"),
    case("center", "", "Pack in the centre"),
    case("end", "", "Pack at the end"),
    case("between", "", "Space between children"),
    case("around", "", "Space around children"),
    case("evenly", "", "Equal space everywhere"),
];
const TEXT_ALIGNS: &[CaseSig] = &[
    case("left", "left", "Left-aligned text"),
    case("center", "center", "Centred text"),
    case("right", "right", "Right-aligned text"),
];
const ANIMATIONS: &[CaseSig] = &[
    case("fadeIn", "fadeIn", "Fade in"),
    case("fadeOut", "fadeOut", "Fade out"),
    case("slideUp", "slideUp", "Slide up into place"),
    case("slideDown", "slideDown", "Slide down into place"),
    case("slideLeft", "slideLeft", "Slide in from the right"),
    case("slideRight", "slideRight", "Slide in from the left"),
    case("scaleIn", "scaleIn", "Grow into place"),
    case("scaleOut", "scaleOut", "Shrink away"),
    case("bounce", "bounce", "Bounce"),
    case("shake", "shake", "Shake"),
    case("pulse", "pulse", "Pulse"),
    case("spin", "spin", "Spin"),
];
const SPEEDS: &[CaseSig] = &[
    case("fast", "fast", "150ms"),
    case("normal", "", "The default"),
    case("slow", "slow", "500ms"),
];

// ─── Shared props ────────────────────────────────────────────────────────

/// The spacing scale as a named argument's value (`gap: .md`), which was
/// never a modifier word and so carries no legacy spelling.
const GAP_SPACES: &[CaseSig] = &[
    case("xs", "", "Extra small"),
    case("sm", "", "Small"),
    case("md", "", "Medium, the default"),
    case("lg", "", "Large"),
    case("xl", "", "Extra large"),
];

const BIND: PropSig = special(
    "bind",
    PropType::State,
    "State variable the control reads and writes (two-way)",
);
const LABEL: PropSig = special(
    "label",
    PropType::Str,
    "Visible label; also the control's accessible name",
);
const HINT: PropSig = special(
    "hint",
    PropType::Str,
    "Help text under the control, linked to it by `aria-describedby`",
);
const ERROR: PropSig = special(
    "error",
    PropType::Str,
    "Error text (a string; empty when there is none): announced as it appears and sets `aria-invalid`",
);
const DISABLED: PropSig = attr_flag("disabled", "Whether the control is inert");
const VALUE: PropSig = attr(
    "value",
    PropType::Any,
    "The control's value, when its state is not bound",
);
const CHECKED: PropSig = attr(
    "checked",
    PropType::Bool,
    "Whether it is checked, when its state is not bound",
);
const GAP: PropSig = special("gap", PropType::Enum(GAP_SPACES), "Space between children");
const ALIGN: PropSig = special("align", PropType::Enum(ALIGNS), "Cross-axis alignment");
const JUSTIFY: PropSig = special(
    "justify",
    PropType::Enum(JUSTIFIES),
    "Main-axis distribution",
);
const TO: PropSig = special(
    "to",
    PropType::Path,
    "Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current=\"page\"`",
);
const ACTIVE: PropSig = special(
    "active",
    PropType::Enum(&[case("prefix", "", "Also active on routes beneath `to`")]),
    "How the link decides it is the current one",
);
const VISIBLE: PropSig = special(
    "visible",
    PropType::State,
    "State flag that opens and closes it",
);
const TITLE: PropSig = special("title", PropType::Str, "Heading shown at the top");
const SIZE: PropSig = variant("size", SIZES, "Its size");
const MUTED: PropSig = flag("muted", "muted", "Dimmed, for secondary text");

/// Props every element takes.
pub const UNIVERSAL_PROPS: &[PropSig] = &[
    variant(
        "animate",
        ANIMATIONS,
        "Animation played when the element appears",
    ),
    PropSig {
        name: "exit",
        ty: PropType::Enum(ANIMATIONS),
        summary: "Animation played when the element leaves",
        sink: Sink::Special,
        legacy: Legacy::Attr,
        shorthand: false,
    },
    variant("speed", SPEEDS, "How fast its animation runs"),
    special("duration", PropType::Str, "Animation length, e.g. `300ms`"),
    special("delay", PropType::Str, "Wait before the animation starts"),
    special(
        "easing",
        PropType::Str,
        "Timing function of the animation, e.g. `ease-out`",
    ),
    special(
        "stagger",
        PropType::Str,
        "Delay added per item of a list, e.g. `50ms`",
    ),
    special(
        "class",
        PropType::Str,
        "Classes added beside the engine's, from the project's own stylesheets",
    ),
];

/// The DOM events any element accepts in an `on` handler.
pub const UNIVERSAL_EVENTS: &[(&str, &str)] = &[
    ("click", "The element was clicked or activated"),
    ("input", "The value changed, on every keystroke"),
    ("change", "The value was committed"),
    ("submit", "The form was submitted"),
    ("focus", "The element received focus"),
    ("blur", "The element lost focus"),
    ("keydown", "A key went down; `e.key` names it"),
    ("keyup", "A key came up"),
    ("keypress", "A character key was pressed"),
    ("mouseenter", "The pointer entered the element"),
    ("mouseleave", "The pointer left the element"),
];

const G: &[AttrFamily] = &[AttrFamily::Global, AttrFamily::Aria, AttrFamily::Data];
const G_INPUT: &[AttrFamily] = &[
    AttrFamily::Global,
    AttrFamily::Aria,
    AttrFamily::Data,
    AttrFamily::Input,
];
const G_ANCHOR: &[AttrFamily] = &[
    AttrFamily::Global,
    AttrFamily::Aria,
    AttrFamily::Data,
    AttrFamily::Anchor,
];
const G_MEDIA: &[AttrFamily] = &[
    AttrFamily::Global,
    AttrFamily::Aria,
    AttrFamily::Data,
    AttrFamily::Media,
];
const G_FORM: &[AttrFamily] = &[
    AttrFamily::Global,
    AttrFamily::Aria,
    AttrFamily::Data,
    AttrFamily::Form,
];
const G_TABLE: &[AttrFamily] = &[
    AttrFamily::Global,
    AttrFamily::Aria,
    AttrFamily::Data,
    AttrFamily::Table,
];

const TEXT_PROPS: &[PropSig] = &[
    variant("tone", TONES, "Colour"),
    SIZE,
    variant("align", TEXT_ALIGNS, "Text alignment"),
    variant(
        "transform",
        &[
            case("uppercase", "uppercase", "ALL CAPS"),
            case("lowercase", "lowercase", "all lower case"),
        ],
        "Letter case",
    ),
    variant(
        "kind",
        &[
            case("heading", "heading", "Styled as a heading"),
            case("subtitle", "subtitle", "Styled as a subtitle"),
        ],
        "Typographic role",
    ),
    flag("bold", "bold", "Bold"),
    flag("italic", "italic", "Italic"),
    flag("underline", "underline", "Underlined"),
    MUTED,
];

// ─── The table ───────────────────────────────────────────────────────────

pub const COMPONENTS: &[ComponentSig] = &[
    // ─── Layout ───────────────────────────────────────────────────────────
    comp(
        "Container",
        "Layout",
        "Centred wrapper with a maximum width and horizontal padding.",
        None,
        &[flag("fluid", "fluid", "Full width, no maximum")],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Row",
        "Layout",
        "Horizontal flex layout.",
        None,
        &[GAP, ALIGN, JUSTIFY],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Column",
        "Layout",
        "A child of the 12-column grid, spanning `span` columns.",
        None,
        &[special(
            "span",
            PropType::Num,
            "How many of the 12 grid columns to span",
        )],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Grid",
        "Layout",
        "CSS grid with a fixed number of columns.",
        None,
        &[
            special("columns", PropType::Num, "Number of equal columns"),
            GAP,
            ALIGN,
            JUSTIFY,
        ],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Stack",
        "Layout",
        "Vertical flex layout.",
        None,
        &[GAP, ALIGN, JUSTIFY],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Spacer",
        "Layout",
        "Vertical space; `md` when no size is given.",
        None,
        &[variant("size", SPACES, "How much space")],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Divider",
        "Layout",
        "Horizontal rule.",
        None,
        &[],
        &[],
        G,
        Children::None,
    ),
    // ─── Navigation ───────────────────────────────────────────────────────
    comp(
        "Navbar",
        "Navigation",
        "Top navigation bar with brand, links and action areas.",
        None,
        &[],
        &["Brand", "Links", "Actions"],
        G,
        Children::Elements,
    ),
    part(
        "Navbar",
        "Brand",
        Ir::Sub("Navbar", "Brand"),
        "The brand area, usually a link home.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Navbar",
        "Links",
        Ir::Sub("Navbar", "Links"),
        "The navigation links; collapses behind a toggle on small screens.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Navbar",
        "Actions",
        Ir::Sub("Navbar", "Actions"),
        "Buttons at the end of the bar.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Sidebar",
        "Navigation",
        "Side navigation. A `Sidebar.Item` whose `to` matches the current route is marked `.active` and `aria-current=\"page\"`.",
        None,
        &[],
        &["Header", "Item", "Divider"],
        G,
        Children::Elements,
    ),
    part(
        "Sidebar",
        "Header",
        Ir::Sub("Sidebar", "Header"),
        "The sidebar's heading area.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Sidebar",
        "Item",
        Ir::Sub("Sidebar", "Item"),
        "One navigation entry.",
        None,
        &[
            TO,
            special("icon", PropType::Str, "Icon shown before the label"),
            ACTIVE,
        ],
        G_ANCHOR,
        Children::Elements,
    ),
    part(
        "Sidebar",
        "Divider",
        Ir::Sub("Sidebar", "Divider"),
        "A rule between groups of items.",
        None,
        &[],
        G,
        Children::None,
    ),
    comp(
        "Link",
        "Navigation",
        "Client-side navigation link; its label or its block is the link text.",
        Some(special("label", PropType::Str, "The link text")),
        &[TO, ACTIVE],
        &[],
        G_ANCHOR,
        Children::Elements,
    ),
    comp(
        "Tabs",
        "Navigation",
        "Tab strip switching between its pages.",
        None,
        &[],
        &["Page"],
        G,
        Children::Elements,
    ),
    part(
        "Tabs",
        "Page",
        Ir::BuiltIn("TabPage"),
        "One tab and its panel.",
        Some(special("label", PropType::Str, "The tab's label")),
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "TabPage",
        "Navigation",
        "One tab inside `Tabs` (the original spelling of `Tabs.Page`).",
        Some(special("label", PropType::Str, "The tab's label")),
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Breadcrumb",
        "Navigation",
        "Breadcrumb trail. An item without `to` is the current page.",
        None,
        &[],
        &["Item"],
        G,
        Children::Elements,
    ),
    part(
        "Breadcrumb",
        "Item",
        Ir::Sub("Breadcrumb", "Item"),
        "One step of the trail.",
        None,
        &[TO],
        G_ANCHOR,
        Children::Elements,
    ),
    comp(
        "Menu",
        "Navigation",
        "Menu opened by a trigger button.",
        None,
        &[
            special(
                "trigger",
                PropType::Str,
                "Label of the button that opens the menu",
            ),
            special("label", PropType::Str, "Same as `trigger`"),
        ],
        &["Item", "Divider"],
        G,
        Children::Elements,
    ),
    part(
        "Menu",
        "Item",
        Ir::Sub("Menu", "Item"),
        "One entry of the menu.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Menu",
        "Divider",
        Ir::Sub("Menu", "Divider"),
        "A rule between groups of entries.",
        None,
        &[],
        G,
        Children::None,
    ),
    // ─── Data display ─────────────────────────────────────────────────────
    comp(
        "Card",
        "Data display",
        "Card surface with optional header, body and footer sections.",
        None,
        &[variant(
            "surface",
            &[
                case("flat", "flat", "No shadow"),
                case("elevated", "elevated", "Raised with a shadow"),
                case("outlined", "outlined", "A border, no shadow"),
            ],
            "Surface treatment",
        )],
        &["Header", "Body", "Footer"],
        G,
        Children::Elements,
    ),
    part(
        "Card",
        "Header",
        Ir::Sub("Card", "Header"),
        "The card's heading area.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Card",
        "Body",
        Ir::Sub("Card", "Body"),
        "The card's content area.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Card",
        "Footer",
        Ir::Sub("Card", "Footer"),
        "The card's action area.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Table",
        "Data display",
        "Data table. Cells inside `Table.Head` render as `<th scope=\"col\">`; `caption` is the table's accessible name, visually hidden.",
        None,
        &[special(
            "caption",
            PropType::Str,
            "Accessible name of the table (rendered visually hidden)",
        )],
        &["Head", "Body", "Row", "Cell"],
        G,
        Children::Elements,
    ),
    part(
        "Table",
        "Head",
        Ir::BuiltIn("Thead"),
        "Header section; its cells become `<th scope=\"col\">`.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Table",
        "Body",
        Ir::BuiltIn("Tbody"),
        "Body section.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Table",
        "Row",
        Ir::BuiltIn("Trow"),
        "A row.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Table",
        "Cell",
        Ir::BuiltIn("Tcell"),
        "A cell. `header` makes it a `<th>` anywhere, as `Table.Head` does for its cells.",
        Some(special("content", PropType::Any, "The cell's content")),
        &[flag("header", "header", "Render as a `<th>`")],
        G_TABLE,
        Children::Elements,
    ),
    comp(
        "Thead",
        "Data display",
        "Table header section (the original spelling of `Table.Head`).",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Tbody",
        "Data display",
        "Table body section (the original spelling of `Table.Body`).",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Trow",
        "Data display",
        "Table row (the original spelling of `Table.Row`).",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Tcell",
        "Data display",
        "Table cell (the original spelling of `Table.Cell`).",
        Some(special("content", PropType::Any, "The cell's content")),
        &[flag("header", "header", "Render as a `<th>`")],
        &[],
        G_TABLE,
        Children::Elements,
    ),
    comp(
        "List",
        "Data display",
        "List of its children; `ordered` for a numbered list.",
        None,
        &[flag("ordered", "ordered", "Numbered (`<ol>`)")],
        &["Item"],
        G,
        Children::Elements,
    ),
    part(
        "List",
        "Item",
        Ir::Sub("List", "Item"),
        "One entry (`<li>`).",
        None,
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Badge",
        "Data display",
        "Small status label.",
        Some(special("label", PropType::Str, "The badge text")),
        &[
            variant("tone", BADGE_TONES, "Colour"),
            flag("pill", "pill", "Fully rounded ends"),
        ],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Tag",
        "Data display",
        "Compact chip.",
        Some(special("label", PropType::Str, "The tag text")),
        &[],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Avatar",
        "Data display",
        "A picture of a person, or their initials.",
        None,
        &[
            special("src", PropType::Str, "Image URL"),
            special("alt", PropType::Str, "Text alternative for the image"),
            special(
                "initials",
                PropType::Str,
                "Letters shown when there is no image",
            ),
            SIZE,
            variant("tone", PRIMARY_ONLY, "Colour of the initials"),
        ],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Tooltip",
        "Data display",
        "Shows `text` when its child is hovered or focused.",
        Some(special("text", PropType::Str, "The tooltip text")),
        &[],
        &[],
        G,
        Children::Elements,
    ),
    // ─── Form & input ─────────────────────────────────────────────────────
    comp(
        "Input",
        "Form",
        "Text field. With `label`, `hint` or `error` it is a labelled field.",
        None,
        &[
            BIND,
            LABEL,
            HINT,
            ERROR,
            attr(
                "type",
                PropType::Enum(&[
                    case("text", "text", "Plain text"),
                    case("email", "email", "An email address"),
                    case("password", "password", "A password"),
                    case("number", "number", "A number"),
                    case("search", "search", "A search query"),
                    case("tel", "tel", "A phone number"),
                    case("url", "url", "A URL"),
                    case("date", "date", "A date"),
                    case("time", "time", "A time"),
                    case("datetime", "datetime", "A date and time"),
                    case("color", "color", "A colour"),
                ]),
                "What the field holds",
            ),
            attr(
                "placeholder",
                PropType::Str,
                "Hint shown while the field is empty",
            ),
            VALUE,
            attr("min", PropType::Any, "Lowest value"),
            attr("max", PropType::Any, "Highest value"),
            attr("step", PropType::Num, "Granularity"),
            attr_flag("required", "Whether the form refuses to submit without it"),
            DISABLED,
            SIZE,
            variant("width", &[case("full", "full", "Full width")], "Width"),
            flag("rounded", "rounded", "Rounded corners"),
        ],
        &[],
        G_INPUT,
        Children::None,
    ),
    comp(
        "Select",
        "Form",
        "Drop-down of `Select.Option` children.",
        None,
        &[
            BIND,
            VALUE,
            LABEL,
            HINT,
            ERROR,
            DISABLED,
            PropSig {
                name: "multiple",
                ty: PropType::Bool,
                summary: "Allow several choices",
                sink: Sink::Attr,
                legacy: Legacy::Modifier("multiple"),
                shorthand: true,
            },
        ],
        &["Option"],
        G_INPUT,
        Children::Elements,
    ),
    part(
        "Select",
        "Option",
        Ir::BuiltIn("Option"),
        "One choice: the visible label, and the value sent with the form.",
        Some(special("label", PropType::Str, "The visible label")),
        &[special(
            "value",
            PropType::Any,
            "The value `bind` takes when this one is chosen; the label when absent",
        )],
        G,
        Children::None,
    ),
    comp(
        "Option",
        "Form",
        "One choice in a `Select` (the original spelling of `Select.Option`).",
        Some(special("label", PropType::Str, "The visible label")),
        &[special(
            "value",
            PropType::Any,
            "The value sent with the form",
        )],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Checkbox",
        "Form",
        "Checkbox bound to a boolean state.",
        None,
        &[BIND, CHECKED, LABEL, DISABLED],
        &[],
        G_INPUT,
        Children::None,
    ),
    comp(
        "Radio",
        "Form",
        "One choice of a group; every radio bound to the same state is one group.",
        None,
        &[
            BIND,
            special(
                "value",
                PropType::Any,
                "The value `bind` takes when this one is chosen",
            ),
            CHECKED,
            LABEL,
            DISABLED,
        ],
        &[],
        G_INPUT,
        Children::None,
    ),
    comp(
        "Switch",
        "Form",
        "On/off toggle bound to a boolean state.",
        None,
        &[BIND, CHECKED, LABEL, DISABLED],
        &[],
        G_INPUT,
        Children::None,
    ),
    comp(
        "Slider",
        "Form",
        "Range input. `aria-*` arguments reach the input; with `aria-valuetext` the raw number is not shown beside the track.",
        None,
        &[
            BIND,
            special("min", PropType::Num, "Lowest value"),
            special("max", PropType::Num, "Highest value"),
            special("step", PropType::Num, "Granularity"),
            LABEL,
            DISABLED,
        ],
        &[],
        G_INPUT,
        Children::None,
    ),
    comp(
        "DatePicker",
        "Form",
        "Date input.",
        None,
        &[
            BIND,
            LABEL,
            special("min", PropType::Str, "Earliest date, `YYYY-MM-DD`"),
            special("max", PropType::Str, "Latest date, `YYYY-MM-DD`"),
            DISABLED,
        ],
        &[],
        G_INPUT,
        Children::None,
    ),
    comp(
        "FileUpload",
        "Form",
        "File input; handle the chosen files in `on change`.",
        None,
        &[
            special(
                "accept",
                PropType::Str,
                "Accepted types, e.g. `image/*` or `.pdf,.doc`",
            ),
            LABEL,
            PropSig {
                name: "multiple",
                ty: PropType::Bool,
                summary: "Accept several files",
                sink: Sink::Attr,
                legacy: Legacy::Modifier("multiple"),
                shorthand: true,
            },
            DISABLED,
        ],
        &[],
        G_INPUT,
        Children::Elements,
    ),
    comp(
        "Form",
        "Form",
        "Groups controls; `on submit` runs when it is submitted, and the page never navigates away.",
        None,
        &[],
        &[],
        G_FORM,
        Children::Elements,
    ),
    // ─── Feedback ─────────────────────────────────────────────────────────
    comp(
        "Alert",
        "Feedback",
        "Inline notice; a `danger` or `warning` one is announced as an alert.",
        Some(special("message", PropType::Str, "The message")),
        &[variant("tone", STATUS_TONES, "Kind of notice")],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Toast",
        "Feedback",
        "Temporary notification.",
        Some(special("message", PropType::Str, "The message")),
        &[variant("tone", STATUS_TONES, "Kind of notice")],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Modal",
        "Feedback",
        "Modal `<dialog>`, opened by a state flag; the browser supplies focus trapping and Escape.",
        None,
        &[VISIBLE, TITLE],
        &["Footer"],
        G,
        Children::Elements,
    ),
    part(
        "Modal",
        "Footer",
        Ir::Sub("Modal", "Footer"),
        "The modal's action row.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Dialog",
        "Feedback",
        "Confirmation `<dialog>`, opened by a state flag.",
        None,
        &[VISIBLE, TITLE],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Spinner",
        "Feedback",
        "Loading indicator, announced as a status.",
        None,
        &[SIZE],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Progress",
        "Feedback",
        "Progress bar; a `value` that reads state moves with it.",
        None,
        &[
            special("value", PropType::Num, "Current value"),
            attr("max", PropType::Num, "Value at 100%"),
        ],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Skeleton",
        "Feedback",
        "Placeholder shape shown while content loads.",
        None,
        &[
            special("height", PropType::Str, "CSS height"),
            special("width", PropType::Str, "CSS width"),
            special("size", PropType::Str, "Diameter of a `circle`"),
            flag("circle", "circle", "A round placeholder"),
        ],
        &[],
        G,
        Children::None,
    ),
    // ─── Actions ──────────────────────────────────────────────────────────
    comp(
        "Button",
        "Actions",
        "Button. Its block holds what it shows; `on click` what it does.",
        Some(special("label", PropType::Str, "The label")),
        &[
            variant("tone", TONES, "Colour"),
            SIZE,
            variant(
                "shape",
                &[
                    case("rounded", "rounded", "Rounded corners"),
                    case("pill", "pill", "Fully rounded ends"),
                ],
                "Corner shape",
            ),
            variant("width", &[case("full", "full", "Full width")], "Width"),
            flag("outlined", "outlined", "A border and no fill"),
            attr(
                "type",
                PropType::Enum(&[
                    case("button", "", "A plain button, the default"),
                    case("submit", "submit", "Submits the enclosing form"),
                    case("reset", "reset", "Resets the enclosing form"),
                ]),
                "Its role inside a form",
            ),
            TO,
            DISABLED,
        ],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "IconButton",
        "Actions",
        "Icon-only button. `label` is its accessible name, never visible text.",
        None,
        &[
            special("icon", PropType::Str, "One of the built-in icon names"),
            special(
                "label",
                PropType::Str,
                "Accessible name (`aria-label` and `title`)",
            ),
            SIZE,
            variant("tone", PRIMARY_DANGER, "Colour"),
            attr(
                "type",
                PropType::Enum(&[
                    case("button", "", "A plain button, the default"),
                    case("submit", "submit", "Submits the enclosing form"),
                    case("reset", "reset", "Resets the enclosing form"),
                ]),
                "Its role inside a form",
            ),
            DISABLED,
        ],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "ButtonGroup",
        "Actions",
        "Buttons joined into one row.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Dropdown",
        "Actions",
        "Button that opens a menu of `Dropdown.Item`s.",
        Some(special(
            "label",
            PropType::Str,
            "Label of the button that opens it",
        )),
        &[],
        &["Item", "Divider"],
        G,
        Children::Elements,
    ),
    part(
        "Dropdown",
        "Item",
        Ir::Sub("Dropdown", "Item"),
        "One entry of the menu.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Dropdown",
        "Divider",
        Ir::Sub("Dropdown", "Divider"),
        "A rule between groups of entries.",
        None,
        &[],
        G,
        Children::None,
    ),
    // ─── Media ────────────────────────────────────────────────────────────
    comp(
        "Image",
        "Media",
        "Image; `alt` is required for accessibility (rule A01).",
        None,
        &[
            attr("src", PropType::Str, "Image URL"),
            attr(
                "alt",
                PropType::Str,
                "Text alternative; empty for a purely decorative image",
            ),
            attr("width", PropType::Num, "Intrinsic width"),
            attr("height", PropType::Num, "Intrinsic height"),
            attr(
                "loading",
                PropType::Enum(&[
                    case("lazy", "", "Load when near the viewport"),
                    case("eager", "", "Load at once"),
                ]),
                "Unset, the first image on a page is eager at high priority and the rest lazy",
            ),
            variant(
                "shape",
                &[
                    case("rounded", "rounded", "Rounded corners"),
                    case("circle", "circle", "Clipped to a circle"),
                ],
                "Shape",
            ),
        ],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Video",
        "Media",
        "Video player.",
        None,
        &[
            attr("src", PropType::Str, "Video URL"),
            attr_flag("controls", "Show the browser's playback controls"),
            attr_flag("autoplay", "Start playing when shown"),
            attr("width", PropType::Num, "Intrinsic width"),
            attr("height", PropType::Num, "Intrinsic height"),
        ],
        &[],
        G_MEDIA,
        Children::None,
    ),
    comp(
        "Icon",
        "Media",
        "One of the built-in SVG icons, rendered inline.",
        Some(special("name", PropType::Str, "The icon name")),
        &[SIZE, variant("tone", TONES, "Colour"), MUTED],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Carousel",
        "Media",
        "Slide track with named controls and dots; optional autoplay.",
        None,
        &[
            special("label", PropType::Str, "Accessible name of the region"),
            special("autoplay", PropType::Bool, "Advance automatically"),
            special("interval", PropType::Num, "Milliseconds between slides"),
        ],
        &["Slide"],
        G,
        Children::Elements,
    ),
    part(
        "Carousel",
        "Slide",
        Ir::Sub("Carousel", "Slide"),
        "One slide.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    // ─── Typography ───────────────────────────────────────────────────────
    comp(
        "Text",
        "Typography",
        "A paragraph of text. `{…}` in the string reads state and stays live.",
        Some(special("content", PropType::Any, "The text")),
        TEXT_PROPS,
        &[],
        G,
        Children::None,
    ),
    comp(
        "Heading",
        "Typography",
        "Heading; `level` picks the tag (`h2` when none is given).",
        Some(special("content", PropType::Any, "The heading text")),
        &[
            special(
                "level",
                PropType::Enum(&[
                    case("h1", "h1", "The page's one top heading"),
                    case("h2", "h2", "A section heading, the default"),
                    case("h3", "h3", "A sub-section"),
                    case("h4", "h4", ""),
                    case("h5", "h5", ""),
                    case("h6", "h6", ""),
                ]),
                "Heading level: the tag, and its place in the outline",
            ),
            variant("align", TEXT_ALIGNS, "Text alignment"),
            MUTED,
        ],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Code",
        "Typography",
        "Inline code, or a code block with `block`.",
        Some(special("content", PropType::Any, "The code")),
        &[flag("block", "block", "A multi-line block")],
        &[],
        G,
        Children::None,
    ),
    comp(
        "Blockquote",
        "Typography",
        "Quotation: the text, or its block as the quoted content.",
        Some(special("content", PropType::Any, "The quoted text")),
        &[],
        &[],
        G,
        Children::Elements,
    ),
    // ─── Document (PDF) ───────────────────────────────────────────────────
    comp(
        "Document",
        "PDF",
        "Root of a PDF document.",
        None,
        &[special(
            "page_size",
            PropType::Enum(&[
                case("A4", "", ""),
                case("A3", "", ""),
                case("A5", "", ""),
                case("Letter", "", ""),
                case("Legal", "", ""),
            ]),
            "Paper size",
        )],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Section",
        "PDF",
        "Groups content with spacing; on the web, a `<section>`.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Paragraph",
        "PDF",
        "Block of text with paragraph spacing.",
        Some(special("content", PropType::Any, "The text")),
        TEXT_PROPS,
        &[],
        G,
        Children::None,
    ),
    comp(
        "Header",
        "PDF",
        "Repeated at the top of every PDF page; a `<header>` on the web.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Footer",
        "PDF",
        "Repeated at the bottom of every PDF page; a `<footer>` on the web.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "PageBreak",
        "PDF",
        "Forces a new PDF page.",
        None,
        &[],
        &[],
        G,
        Children::None,
    ),
    // ─── Slides ───────────────────────────────────────────────────────────
    comp(
        "Presentation",
        "Slides",
        "Root of a slide deck; its children must be slides.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Slide",
        "Slides",
        "One slide, one PDF page; content is top-aligned and clipped at the bottom margin.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "TitleSlide",
        "Slides",
        "Cover slide: title and subtitle, vertically centred.",
        Some(special("title", PropType::Str, "The title")),
        &[special("subtitle", PropType::Str, "The subtitle")],
        &[],
        G,
        Children::None,
    ),
    comp(
        "SectionSlide",
        "Slides",
        "Full-bleed coloured band with a centred label.",
        Some(special("label", PropType::Str, "The label")),
        &[variant("tone", BAND_TONES, "Colour of the band")],
        &[],
        G,
        Children::None,
    ),
    comp(
        "TwoColumn",
        "Slides",
        "Two equal columns; takes exactly two `Container` children.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "ImageSlide",
        "Slides",
        "Image slide with an optional caption.",
        None,
        &[
            special("src", PropType::Str, "Image path (required)"),
            special("caption", PropType::Str, "Caption under the image"),
        ],
        &[],
        G,
        Children::None,
    ),
    // ─── Routing ──────────────────────────────────────────────────────────
    comp(
        "Router",
        "Routing",
        "Where the current page renders. May sit at any depth inside `app`; pages declare their own paths.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    ),
    comp(
        "Route",
        "Routing",
        "Maps a path to a page (the original grammar; pages now declare their paths).",
        None,
        &[
            special("path", PropType::Path, "URL pattern"),
            special("page", PropType::Decl, "The page to render"),
        ],
        &[],
        G,
        Children::None,
    ),
];
