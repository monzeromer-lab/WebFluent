//! The table of built-in components.
//!
//! Order follows the language reference (`AGENTS.md`): layout, navigation,
//! data display, form, feedback, actions, media, typography, document,
//! slides, routing. Parts (`Card.Header`) follow their owner. Every enum
//! case carries the modifier word it was in the original grammar, which is
//! the word the code generators read; `""` marks a default that has no
//! class of its own.

use super::{
    AttrFamily, CaseSig, Children, ComponentSig, FileKind, Ir, Legacy, NOT_ON_PAPER, Paper,
    PropSig, PropType, Sink,
};

// ─── Construction helpers ────────────────────────────────────────────────

/// Which pages a running element is drawn on.
const ON_PAGES: &[CaseSig] = &[
    case("all", "", "Every page"),
    case("first", "", "The first page only"),
    case("rest", "", "Every page but the first"),
    case("odd", "", "Odd pages"),
    case("even", "", "Even pages"),
];

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
        required: false,
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
        required: false,
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
        required: false,
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
        required: false,
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
        required: false,
    }
}

/// A prop the component cannot be placed without (`C01` when it is left
/// out).
const fn required(prop: PropSig) -> PropSig {
    PropSig {
        required: true,
        ..prop
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "a const constructor for the built-ins table, one argument per column"
)]
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
        tag: "div",
        class: "",
        css: &[],
        paper: Paper::Undecided,
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "a const constructor for the built-ins table, one argument per column"
)]
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
        tag: "div",
        class: "",
        css: &[],
        paper: Paper::Undecided,
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
    case(
        "expand",
        "expand",
        "Open to the height of what is inside, which CSS alone cannot do",
    ),
    case("collapse", "collapse", "Close to nothing"),
];
/// The easings a design names. Each is a token, so a theme retunes them
/// once — and any CSS timing function may be written as a string instead.
const EASINGS: &[CaseSig] = &[
    case("standard", "", "The design's own pacing"),
    case(
        "spring",
        "",
        "Overshoots a little and settles, like a real spring",
    ),
    case("ease", "", "Slow at both ends"),
    case("easeIn", "", "Slow to start"),
    case("easeOut", "", "Slow to stop"),
    case("easeInOut", "", "Slow at both ends, more so than `ease`"),
    case("linear", "", "The same speed throughout"),
    case("bouncy", "", "Overshoots hard"),
    case("smooth", "", "Even, with no overshoot"),
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
/// A layout that asks to reflow on a narrow screen. Without it a layout
/// stays as it was written, at every width.
const STACKS: PropSig = flag(
    "stacks",
    "stacks",
    "Reflow on a narrow screen: a Row becomes a column, a Grid one column",
);
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
/// A modal's or a dialog's close button, in its heading row.
const CLOSABLE: PropSig = flag(
    "closable",
    "closable",
    "A close button in the heading row, which closes it as Escape does",
);
/// Where a layer goes against its anchor: a side, and where along it.
const PLACEMENT: PropSig = special(
    "placement",
    PropType::Enum(&[
        case("bottom", "", "Below, centred"),
        case("bottomStart", "", "Below, from its start"),
        case("bottomEnd", "", "Below, to its end"),
        case("top", "", "Above, centred"),
        case("topStart", "", "Above, from its start"),
        case("topEnd", "", "Above, to its end"),
        case("left", "", "Before it, centred"),
        case("leftStart", "", "Before it, from its top"),
        case("leftEnd", "", "Before it, to its bottom"),
        case("right", "", "After it, centred"),
        case("rightStart", "", "After it, from its top"),
        case("rightEnd", "", "After it, to its bottom"),
    ]),
    "Where it goes against its anchor; it flips to the other side when there is no room. Unset, `.bottomStart`",
);
/// What a layer is placed against.
const ANCHOR: PropSig = special(
    "anchor",
    PropType::Any,
    "What it is placed against: an element's `ref:`, or a point `{ x: e.clientX, y: e.clientY }`",
);
/// A point to place a layer at, such as where a context menu was asked for.
const AT: PropSig = special(
    "at",
    PropType::Any,
    "A point to place it at, `{ x: e.clientX, y: e.clientY }`: where the pointer asked for a context menu",
);
const OFFSET: PropSig = special("offset", PropType::Num, "How far from the anchor, in px");
/// What closes a layer besides the page's own button.
const DISMISS: PropSig = special(
    "dismiss",
    PropType::Enum(&[
        case("any", "", "Escape, or a press outside it (the default)"),
        case("escape", "", "Escape only: a press outside is the page's"),
        case(
            "none",
            "",
            "Neither: only the page's own button closes it, for what must be answered",
        ),
    ]),
    "What closes it besides the page's own button",
);
/// What takes the focus when a layer opens.
const INITIAL_FOCUS: PropSig = special(
    "initialFocus",
    PropType::Any,
    "What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control",
);
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
        required: false,
    },
    variant("speed", SPEEDS, "How fast its animation runs"),
    special("duration", PropType::Str, "Animation length, e.g. `300ms`"),
    special("delay", PropType::Str, "Wait before the animation starts"),
    PropSig {
        name: "easing",
        ty: PropType::Enum(EASINGS),
        summary: "How the animation is paced — a named easing, or any CSS timing function",
        sink: Sink::Special,
        legacy: Legacy::Attr,
        shorthand: false,
        required: false,
    },
    special(
        "stagger",
        PropType::Str,
        "Delay added per item of a list, e.g. `50ms`",
    ),
    PropSig {
        name: "on",
        ty: PropType::Enum(&[
            case("mount", "mount", "When it appears (the default)"),
            case("enterView", "enterView", "When it first scrolls into view"),
        ]),
        summary: "When its animation plays",
        sink: Sink::Special,
        legacy: Legacy::Attr,
        shorthand: false,
        required: false,
    },
    special(
        "count",
        PropType::Str,
        "How long the number it shows takes to count to a new value, e.g. `600ms`",
    ),
    special(
        "shared",
        PropType::Str,
        "A name this element keeps across a route change, so the browser carries it from one page to the next",
    ),
    special(
        "class",
        PropType::Str,
        "Classes added beside the engine's, from the project's own stylesheets",
    ),
    special(
        "ref",
        PropType::State,
        "A handle on the element, usable as the element: `ref: nameInput`, then `nameInput.focus()`",
    ),
    special(
        "drag",
        PropType::Any,
        "What it carries when it is picked up — by a mouse moved a few pixels, or a finger held still — and dropped on something with `on receive(d)`",
    ),
    special(
        "dragGroup",
        PropType::Str,
        "Which drops it belongs to: a drag goes only to targets of its own group, so tabs and tree rows stay apart",
    ),
    special(
        "dropValue",
        PropType::Any,
        "What this target is, as a drag sees it: `d.over` in the drag's `on dragmove(d)` and in `on receive(d)`",
    ),
    PropSig {
        name: "dropAxis",
        ty: PropType::Enum(&[
            case(
                "y",
                "",
                "A list down the page: before is above, after below (the default)",
            ),
            case(
                "x",
                "",
                "A list across it, a tab strip: before is to the start, after to the end",
            ),
        ]),
        summary: "Which way its list runs, for where a drop lands",
        sink: Sink::Special,
        legacy: Legacy::Attr,
        shorthand: false,
        required: false,
    },
    special(
        "dropInside",
        PropType::Bool,
        "A drop on its middle lands inside it — a folder, a container — as well as before or after",
    ),
    PropSig {
        name: "roving",
        ty: PropType::Enum(&[
            case(
                "x",
                "",
                "A row: ArrowLeft and ArrowRight move, the other way round in a right-to-left box",
            ),
            case("y", "", "A column: ArrowUp and ArrowDown move"),
            case("both", "", "Both: a wrapping row of choices"),
        ]),
        summary: "One Tab stop for the group, and the arrow keys, Home, End and the first letters of a name to move inside it: a toolbar, a row of choices, a list",
        sink: Sink::Special,
        legacy: Legacy::Attr,
        shorthand: false,
        required: false,
    },
    special(
        "trapFocus",
        PropType::Bool,
        "While true, focus moves into it and Tab and Shift+Tab stay inside; when it turns false or the element leaves, focus goes back where it was",
    ),
    special(
        "mount",
        PropType::Any,
        "Run with the element each time it is made — `mount: (n) => initTilt(n)` — and what it returns is the handle `update:` and `cleanup:` get",
    ),
    special(
        "update",
        PropType::Any,
        "Run with the handle again whenever the state it reads changes: `update: (h) => h.setMax(max)`",
    ),
    special(
        "cleanup",
        PropType::Any,
        "Run with the handle when the element leaves — a route change, an `if` branch, a list item: `cleanup: (h) => h.destroy()`",
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
    (
        "longpress",
        "Held still for half a second; `p.x`, `p.y` say where. The click it ends with is not a click",
    ),
    (
        "swipe",
        "A finger or a pen swept across it; `s.direction` is \"left\", \"right\", \"up\" or \"down\"",
    ),
    (
        "receive",
        "Something with `drag:` was dropped on it; `d.value` is what, `d.position` \"before\", \"after\" or \"inside\"",
    ),
    (
        "dragmove",
        "On what is dragged, while it moves: `d.over` is the target's `dropValue` or null, `d.position` where it would land",
    ),
    (
        "visible",
        "It came into view: scrolled to, or drawn where the reader is looking. `v.ratio` is how much of it shows. A \"load more\" row's cue",
    ),
    ("key", "One key, named: `on key(\"ctrl+k\") { }`"),
    ("keyup", "A key came up"),
    ("keypress", "A character key was pressed"),
    ("mouseenter", "The pointer entered the element"),
    ("mouseleave", "The pointer left the element"),
    ("dblclick", "The element was double-clicked"),
    (
        "auxclick",
        "A button other than the main one was clicked; `e.button` is which — 1 is the middle, which closes a tab",
    ),
    (
        "focusin",
        "Focus moved onto the element or something inside it",
    ),
    ("focusout", "Focus left the element or something inside it"),
    (
        "contextmenu",
        "The context menu was asked for; `e.preventDefault()` keeps the browser's",
    ),
    ("mousedown", "A mouse button went down over the element"),
    ("mouseup", "A mouse button came up over the element"),
    ("mousemove", "The mouse moved over the element"),
    (
        "mouseover",
        "The pointer moved onto the element or one of its children",
    ),
    (
        "mouseout",
        "The pointer moved off the element or one of its children",
    ),
    ("pointerdown", "A pointer — mouse, pen or touch — went down"),
    ("pointerup", "A pointer came up"),
    ("pointermove", "A pointer moved"),
    ("pointerenter", "A pointer entered the element"),
    ("pointerleave", "A pointer left the element"),
    (
        "pointercancel",
        "The browser took the pointer back, for a scroll or a gesture",
    ),
    ("touchstart", "A finger touched the element"),
    ("touchend", "A finger left the element"),
    ("touchmove", "A finger moved across the element"),
    ("wheel", "The wheel or trackpad scrolled over the element"),
    ("scroll", "The element's content scrolled"),
    (
        "focusin",
        "The element or something inside it received focus",
    ),
    ("focusout", "The element or something inside it lost focus"),
    ("reset", "The form was reset"),
    ("invalid", "A control failed the browser's own validation"),
    ("select", "Text in a field was selected"),
    ("copy", "Something was copied from the element"),
    ("cut", "Something was cut from the element"),
    (
        "paste",
        "Something was pasted into the element; `e.clipboardData` holds it",
    ),
    ("dragstart", "The element started being dragged"),
    ("drag", "The element is being dragged"),
    ("dragend", "A drag of the element ended"),
    ("dragenter", "Something dragged entered the element"),
    (
        "dragover",
        "Something dragged is over the element; `e.preventDefault()` allows a drop",
    ),
    ("dragleave", "Something dragged left the element"),
    (
        "drop",
        "Something was dropped on the element; `e.dataTransfer` holds it",
    ),
    ("load", "An image, video or frame finished loading"),
    ("error", "An image, video or script failed to load"),
    ("play", "A video or audio started playing"),
    ("pause", "A video or audio paused"),
    ("ended", "A video or audio reached its end"),
    ("timeupdate", "A playing video or audio moved on"),
    ("animationend", "A CSS animation on the element ended"),
    ("transitionend", "A CSS transition on the element ended"),
    ("toggle", "A details element opened or closed"),
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
    )
    .html("div", "wf-container")
    .on_paper(),
    comp(
        "Row",
        "Layout",
        "Horizontal flex layout. It stays a row at every width unless `.stacks` or a responsive `direction` says otherwise.",
        None,
        &[
            GAP,
            ALIGN,
            JUSTIFY,
            special(
                "direction",
                PropType::Enum(&[
                    case("row", "", "Side by side"),
                    case("column", "", "Stacked"),
                ]),
                "Which way its children run; takes a value per breakpoint",
            ),
            STACKS,
        ],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-row")
    .on_paper(),
    comp(
        "Column",
        "Layout",
        "A child of the 12-column grid, spanning `span` columns.",
        None,
        &[
            special(
                "span",
                PropType::Num,
                "How many of the 12 grid columns to span",
            ),
            STACKS,
        ],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-col")
    .css(&["Column"])
    .on_paper(),
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
            STACKS,
        ],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-grid")
    .css(&["Grid"])
    .on_paper(),
    comp(
        "Stack",
        "Layout",
        "Vertical flex layout.",
        None,
        &[GAP, ALIGN, JUSTIFY],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-stack")
    .on_paper(),
    comp(
        "Spacer",
        "Layout",
        "Vertical space; `md` when no size is given.",
        None,
        &[variant("size", SPACES, "How much space")],
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-spacer")
    .css(&["Spacer"])
    .on_paper(),
    comp(
        "VirtualList",
        "Layout",
        "A scrolling box that draws only the rows in view of the one keyed `for` inside it — for a tree, search results or a diff of thousands of rows.",
        None,
        &[
            special(
                "rowHeight",
                PropType::Num,
                "Every row's height, in px: the fast path. Unset, each row is measured",
            ),
            special(
                "estimate",
                PropType::Num,
                "The height a row not yet drawn stands for, in px. Unset, 32",
            ),
            special(
                "reveal",
                PropType::Any,
                "A row's key: when it changes, that row is scrolled into view — the selection, the current match",
            ),
            special(
                "label",
                PropType::Str,
                "What the list is, for a screen reader",
            ),
        ],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-virtual-list")
    .css(&["Virtual List"])
    .not_on_paper(NOT_ON_PAPER),
    comp(
        "Splitter",
        "Layout",
        "A handle that sets the size of a region beside it: dragged, between `min:` and `max:`; double-clicked or Enter, back to `reset:`; the arrow keys move it.",
        None,
        &[
            BIND,
            special("min", PropType::Num, "The smallest size, in px"),
            special("max", PropType::Num, "The largest size, in px"),
            special(
                "reset",
                PropType::Num,
                "The size a double-click or Enter goes back to, taken as written: `0` can mean the region's default share",
            ),
            special(
                "label",
                PropType::Str,
                "What it resizes, for a screen reader",
            ),
            flag(
                "horizontal",
                "horizontal",
                "Between two regions one above the other: it moves up and down",
            ),
            flag(
                "reverse",
                "reverse",
                "On the start side of what it sizes — a pane on the right, a panel below — so a drag toward the start grows it",
            ),
        ],
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-splitter")
    .css(&["Splitter"])
    .not_on_paper(NOT_ON_PAPER),
    comp(
        "Divider",
        "Layout",
        "Horizontal rule.",
        None,
        &[],
        &[],
        G,
        Children::None,
    )
    .html("hr", "wf-divider")
    .css(&["Divider"])
    .on_paper(),
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
    )
    .html("nav", "wf-navbar")
    .css(&["Navbar", "Off-canvas controls", "Link"])
    .not_on_paper("navigation components are web-only"),
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
    )
    .html("aside", "wf-sidebar")
    .css(&["Sidebar", "Off-canvas controls", "Link", "Icon"])
    .not_on_paper("navigation components are web-only"),
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
    )
    .html("a", "wf-link")
    .css(&["Link"])
    .on_paper(),
    ComponentSig {
        events: &["change"],
        ..comp(
            "Tabs",
            "Navigation",
            "Tab strip switching between its pages: the arrow keys move between the tabs (the other way round in a right-to-left page), Home and End go to the ends.",
            None,
            &[
                special(
                    "bind",
                    PropType::State,
                    "The open tab, read and written: a page's `value:`, or its place from 0 when the pages give none",
                ),
                special(
                    "value",
                    PropType::Any,
                    "The open tab, read only: `on change(v)` hears the one the reader picks",
                ),
                PropSig {
                    name: "orientation",
                    ty: PropType::Enum(&[
                        case("horizontal", "", "Tabs in a row; ArrowLeft and ArrowRight move (the default)"),
                        case("vertical", "", "Tabs in a column; ArrowUp and ArrowDown move"),
                    ]),
                    summary: "Which way the tabs run",
                    sink: Sink::Special,
                    legacy: Legacy::Attr,
                    shorthand: true,
                    required: false,
                },
                PropSig {
                    name: "activation",
                    ty: PropType::Enum(&[
                        case("automatic", "", "A tab opens as the focus reaches it (the default)"),
                        case("manual", "", "The focus moves alone; Enter or Space opens the tab, for panels slow to draw"),
                    ]),
                    summary: "Whether moving the focus opens a tab",
                    sink: Sink::Special,
                    legacy: Legacy::Attr,
                    shorthand: true,
                    required: false,
                },
                special("label", PropType::Str, "The tab strip's accessible name"),
            ],
            &["Page"],
            G,
            Children::Elements,
        )
    }
    .html("div", "wf-tabs")
    .css(&["Tabs"])
    .not_on_paper("navigation components are web-only"),
    part(
        "Tabs",
        "Page",
        Ir::BuiltIn("TabPage"),
        "One tab and its panel.",
        Some(special("label", PropType::Str, "The tab's label")),
        &[
            special(
                "value",
                PropType::Any,
                "What the Tabs' `bind:` holds while this tab is open",
            ),
            special(
                "disabled",
                PropType::Bool,
                "Shown, but not opened: the arrow keys pass over it",
            ),
        ],
        G,
        Children::Elements,
    )
    .html("div", "wf-tab-page")
    .css(&["Tabs"])
    .not_on_paper("navigation components are web-only"),
    comp(
        "TabPage",
        "Navigation",
        "One tab inside `Tabs` (the original spelling of `Tabs.Page`).",
        Some(special("label", PropType::Str, "The tab's label")),
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-tab-page")
    .css(&["Tabs"])
    .not_on_paper("navigation components are web-only"),
    comp(
        "Breadcrumb",
        "Navigation",
        "Breadcrumb trail. An item without `to` is the current page.",
        None,
        &[],
        &["Item"],
        G,
        Children::Elements,
    )
    .html("nav", "wf-breadcrumb")
    .css(&["Breadcrumb", "Link"])
    .on_paper(),
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
    ComponentSig {
        events: &["close"],
        ..comp(
            "Menu",
            "Navigation",
            "A menu above everything: opened by its `trigger:` button, or by `visible:` at an `anchor:` or a point `at:` — a context menu. Escape, a press outside or a choice close it, write `false` back to `visible:` and fire `on close`.",
            None,
            &[
                special(
                    "trigger",
                    PropType::Str,
                    "Label of the button that opens the menu",
                ),
                special("label", PropType::Str, "Same as `trigger`"),
                VISIBLE,
                ANCHOR,
                AT,
                PLACEMENT,
                OFFSET,
            ],
            &["Item", "Divider", "Sub"],
            G,
            Children::Elements,
        )
    }
    .html("div", "wf-menu")
    .css(&["Menu", "Button", "Layer"])
    .not_on_paper("navigation components are web-only"),
    part(
        "Menu",
        "Item",
        Ir::Sub("Menu", "Item"),
        "One entry of the menu; `on click` is what choosing it does.",
        None,
        &[
            special(
                "shortcut",
                PropType::Str,
                "The keys that do the same, `mod+b`, shown at its end as this system writes them (`⌘B`, `Ctrl+B`)",
            ),
            special(
                "checked",
                PropType::Bool,
                "A check mark: the item turns something on and off",
            ),
            special(
                "disabled",
                PropType::Bool,
                "Shown, skipped by the arrow keys, and cannot be chosen",
            ),
            flag(
                "danger",
                "danger",
                "What it does cannot be taken back — Delete, Discard: drawn in the danger colour",
            ),
        ],
        G,
        Children::Elements,
    ),
    part(
        "Menu",
        "Sub",
        Ir::Sub("Menu", "Sub"),
        "An entry that opens a menu of its own beside it.",
        None,
        &[special("label", PropType::Str, "What the entry says")],
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
    )
    .html("div", "wf-card")
    .css(&["Card"])
    .on_paper(),
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
    )
    .html("table", "wf-table")
    .css(&["Table"])
    .on_paper(),
    part(
        "Table",
        "Head",
        Ir::BuiltIn("Thead"),
        "Header section; its cells become `<th scope=\"col\">`.",
        None,
        &[],
        G,
        Children::Elements,
    )
    .html("thead", "")
    .css(&["Table"])
    .on_paper(),
    part(
        "Table",
        "Body",
        Ir::BuiltIn("Tbody"),
        "The body of a table: its rows of data.",
        None,
        &[],
        G,
        Children::Elements,
    )
    .html("tbody", "")
    .css(&["Table"])
    .on_paper(),
    part(
        "Table",
        "Row",
        Ir::BuiltIn("Trow"),
        "A row of cells.",
        None,
        &[],
        G,
        Children::Elements,
    )
    .html("tr", "")
    .css(&["Table"])
    .on_paper(),
    part(
        "Table",
        "Cell",
        Ir::BuiltIn("Tcell"),
        "A cell. `header` makes it a `<th>` anywhere, as `Table.Head` does for its cells.",
        Some(special("content", PropType::Any, "The cell's content")),
        &[flag("header", "header", "Render as a `<th>`")],
        G_TABLE,
        Children::Elements,
    )
    .html("td", "")
    .css(&["Table"])
    .on_paper(),
    comp(
        "Thead",
        "Data display",
        "Table header section (the original spelling of `Table.Head`).",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("thead", "")
    .css(&["Table"])
    .on_paper(),
    comp(
        "Tbody",
        "Data display",
        "Table body section (the original spelling of `Table.Body`).",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("tbody", "")
    .css(&["Table"])
    .on_paper(),
    comp(
        "Trow",
        "Data display",
        "Table row (the original spelling of `Table.Row`).",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("tr", "")
    .css(&["Table"])
    .on_paper(),
    comp(
        "Tcell",
        "Data display",
        "Table cell (the original spelling of `Table.Cell`).",
        Some(special("content", PropType::Any, "The cell's content")),
        &[flag("header", "header", "Render as a `<th>`")],
        &[],
        G_TABLE,
        Children::Elements,
    )
    .html("td", "")
    .css(&["Table"])
    .on_paper(),
    comp(
        "List",
        "Data display",
        "List of its children; `ordered` for a numbered list.",
        None,
        &[flag("ordered", "ordered", "Numbered (`<ol>`)")],
        &["Item"],
        G,
        Children::Elements,
    )
    .html("ul", "wf-list")
    .css(&["List"])
    .on_paper(),
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
    )
    .html("span", "wf-badge")
    .css(&["Badge"])
    .on_paper(),
    comp(
        "Tag",
        "Data display",
        "Compact chip.",
        Some(special("label", PropType::Str, "The tag text")),
        &[],
        &[],
        G,
        Children::None,
    )
    .html("span", "wf-tag")
    .css(&["Tag"])
    .on_paper(),
    comp(
        "Avatar",
        "Data display",
        "A picture of a person, or their initials.",
        None,
        &[
            special("src", PropType::File(FileKind::Image), "Image URL"),
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
    )
    .html("div", "wf-avatar")
    .css(&["Avatar", "Image"])
    .on_paper(),
    comp(
        "Tooltip",
        "Data display",
        "Shows `text` above everything when its child is hovered (after `delay:`) or focused from the keyboard.",
        Some(special("text", PropType::Str, "The tooltip text")),
        &[
            special(
                "shortcut",
                PropType::Str,
                "The keys that do the same, `mod+s`, shown as a chip as this system writes them",
            ),
            PLACEMENT,
            special(
                "delay",
                PropType::Num,
                "How long a hover waits before it shows, in ms. Unset, 500",
            ),
        ],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-tooltip")
    .css(&["Tooltip", "Layer"])
    .not_on_paper(NOT_ON_PAPER),
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
    )
    .html("input", "wf-input")
    .css(&["Input"])
    .not_on_paper("interactive elements are not supported in PDF"),
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
                required: false,
            },
        ],
        &["Option"],
        G_INPUT,
        Children::Elements,
    )
    .html("select", "wf-select")
    .css(&["Input", "Select"])
    .not_on_paper("interactive elements are not supported in PDF"),
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
    )
    .html("option", "")
    .not_on_paper("interactive elements are not supported in PDF"),
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
    )
    .html("option", "")
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "Checkbox",
        "Form",
        "Checkbox bound to a boolean state.",
        None,
        &[BIND, CHECKED, LABEL, DISABLED],
        &[],
        G_INPUT,
        Children::None,
    )
    .html("label", "wf-checkbox")
    .css(&["Checkbox"])
    .not_on_paper("interactive elements are not supported in PDF"),
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
    )
    .html("label", "wf-radio")
    .css(&["Radio"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "Switch",
        "Form",
        "On/off toggle bound to a boolean state.",
        None,
        &[BIND, CHECKED, LABEL, DISABLED],
        &[],
        G_INPUT,
        Children::None,
    )
    .html("label", "wf-switch")
    .css(&["Switch"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "Slider",
        "Form",
        "Range input. `aria-*` and its other attributes reach the input; with `aria-valuetext` the raw number is not shown beside the track.",
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
    )
    .html("div", "wf-slider")
    .css(&["Input", "Slider"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "DatePicker",
        "Form",
        "A date field: the reader picks a day, and the bound state holds a `Date`.",
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
    )
    .html("div", "wf-datepicker")
    .css(&["Input", "Date Picker"])
    .not_on_paper("interactive elements are not supported in PDF"),
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
                required: false,
            },
            DISABLED,
        ],
        &[],
        G_INPUT,
        Children::Elements,
    )
    .html("div", "wf-file-upload")
    .css(&["Input", "File Upload"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "Form",
        "Form",
        "Groups controls; `on submit` runs when it is submitted, and the page never navigates away.",
        None,
        &[
            special(
                "bind",
                PropType::State,
                "A handle on the form: `form.valid`, `form.errors`, `form.values` by field name, `form.reset()`, `form.apply(serverErrors)`",
            ),
            attr(
                "show",
                PropType::Enum(&[
                    case(
                        "onBlur",
                        "onBlur",
                        "After the reader leaves the field (the default)",
                    ),
                    case("onSubmit", "onSubmit", "Only after a submit"),
                    case("live", "live", "As they type"),
                ]),
                "When a `validate` block's message is shown",
            ),
        ],
        &[],
        G_FORM,
        Children::Elements,
    )
    .html("form", "wf-form")
    .css(&["Button", "Input", "Form"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "Textarea",
        "Form",
        "Several lines of text. With `label`, `hint` or `error` it is a labelled field; with `maxLength` it counts what is left.",
        None,
        &[
            BIND,
            LABEL,
            HINT,
            ERROR,
            attr(
                "placeholder",
                PropType::Str,
                "Hint shown while the field is empty",
            ),
            attr("rows", PropType::Num, "How many lines it shows"),
            attr("maxLength", PropType::Num, "The most characters it takes"),
            attr_flag("required", "Whether the form refuses to submit without it"),
            DISABLED,
        ],
        &[],
        G_INPUT,
        Children::None,
    )
    .html("textarea", "wf-input wf-textarea")
    .not_on_paper("interactive elements are not supported in PDF"),
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
    )
    .html("div", "wf-alert")
    .css(&["Alert"])
    .on_paper(),
    comp(
        "Toast",
        "Feedback",
        "Temporary notification.",
        Some(special("message", PropType::Str, "The message")),
        &[variant("tone", STATUS_TONES, "Kind of notice")],
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-toast")
    .css(&["Toast", "Animation Keyframes", "Animation Utility Classes", "Hover Replay"])
    .not_on_paper(NOT_ON_PAPER),
    ComponentSig {
        events: &["close"],
        ..comp(
            "Modal",
            "Feedback",
            "Modal `<dialog>`, opened by a state flag; the browser supplies focus trapping and Escape. `on close` runs when it closes, however it was closed, and the focus goes back to what had it.",
            None,
            &[VISIBLE, TITLE, CLOSABLE, DISMISS, INITIAL_FOCUS],
            &["Title", "Footer"],
            G,
            Children::Elements,
        )
    }
    .html("dialog", "wf-modal")
    .css(&["Button", "Modal"])
    .not_on_paper(NOT_ON_PAPER),
    part(
        "Modal",
        "Title",
        Ir::Sub("Modal", "Title"),
        "What the modal's heading draws, in place of its `title:`'s words, which stay its name: `Modal.Title { Text { \"Rename \"  Code(name) } }`.",
        None,
        &[],
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
    ComponentSig {
        events: &["close"],
        ..comp(
            "Popover",
            "Feedback",
            "A layer at a control: drawn above everything, placed against its `anchor:`, closed by `Escape` or a press outside, which write `false` back to `visible:` and fire `on close`.",
            None,
            &[
                VISIBLE,
                ANCHOR,
                AT,
                PLACEMENT,
                OFFSET,
                special(
                    "label",
                    PropType::Str,
                    "What it is, for a screen reader: its accessible name",
                ),
                flag("arrow", "arrow", "A point toward the anchor"),
                DISMISS,
                INITIAL_FOCUS,
            ],
            &[],
            G,
            Children::Elements,
        )
    }
    .html("div", "wf-popover")
    .css(&["Popover", "Layer"])
    .not_on_paper(NOT_ON_PAPER),
    ComponentSig {
        events: &["close"],
        ..comp(
            "Dialog",
            "Feedback",
            "Confirmation `<dialog>`, opened by a state flag. `on close` runs when it closes, however it was closed, and the focus goes back to what had it.",
            None,
            &[VISIBLE, TITLE, CLOSABLE, DISMISS, INITIAL_FOCUS],
            &["Title"],
            G,
            Children::Elements,
        )
    }
    .html("dialog", "wf-dialog")
    .css(&["Button", "Dialog"])
    .not_on_paper(NOT_ON_PAPER),
    part(
        "Dialog",
        "Title",
        Ir::Sub("Dialog", "Title"),
        "What the dialog's heading draws, in place of its `title:`'s words, which stay its name.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    ComponentSig {
        events: &["close"],
        ..comp(
            "AlertDialog",
            "Feedback",
            "A `<dialog>` that asks before something that cannot be undone: `role=\"alertdialog\"`, its body read out with its title, and closed by Escape or its own buttons, not a press outside. For a question asked from a handler, `await ask(…)` draws one.",
            None,
            &[VISIBLE, TITLE, DISMISS, INITIAL_FOCUS],
            &["Title", "Footer"],
            G,
            Children::Elements,
        )
    }
    .html("dialog", "wf-dialog wf-alert-dialog")
    .css(&["Button", "Dialog"])
    .not_on_paper(NOT_ON_PAPER),
    part(
        "AlertDialog",
        "Title",
        Ir::Sub("AlertDialog", "Title"),
        "What the alert dialog's heading draws, in place of its `title:`'s words, which stay its name.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "AlertDialog",
        "Footer",
        Ir::Sub("AlertDialog", "Footer"),
        "Its answers: the button that loses nothing first.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    ComponentSig {
        events: &["close"],
        ..comp(
            "Sheet",
            "Feedback",
            "A modal `<dialog>` that comes in from an edge: the bottom (a phone's filters, a share list) or the start or end of the line. Dragged by its handle or heading toward its edge it closes; a bottom sheet with `snap:` points settles at the nearest.",
            None,
            &[
                VISIBLE,
                TITLE,
                CLOSABLE,
                special(
                    "side",
                    PropType::Enum(&[
                        case("bottom", "", "From the bottom of the window (the default)"),
                        case("start", "", "From where a line starts: the left, the right in a right-to-left page"),
                        case("end", "", "From where a line ends"),
                    ]),
                    "The edge it comes in from",
                ),
                special(
                    "snap",
                    PropType::Any,
                    "A bottom sheet's heights, as fractions of the window's, `[0.5, 1]`: it opens at the first",
                ),
                special("label", PropType::Str, "Its accessible name, when it has no `title:`"),
                DISMISS,
                INITIAL_FOCUS,
            ],
            &["Title", "Footer"],
            G,
            Children::Elements,
        )
    }
    .html("dialog", "wf-sheet")
    .css(&["Button", "Sheet"])
    .not_on_paper(NOT_ON_PAPER),
    part(
        "Sheet",
        "Title",
        Ir::Sub("Sheet", "Title"),
        "What the sheet's heading draws, in place of its `title:`'s words, which stay its name.",
        None,
        &[],
        G,
        Children::Elements,
    ),
    part(
        "Sheet",
        "Footer",
        Ir::Sub("Sheet", "Footer"),
        "The sheet's action row, kept in view below what scrolls.",
        None,
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
    )
    .html("div", "wf-spinner")
    .css(&["Spinner", "Animation Keyframes", "Animation Utility Classes", "Hover Replay"])
    .not_on_paper(NOT_ON_PAPER),
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
    )
    .html("progress", "wf-progress")
    .css(&["Progress", "Animation Keyframes", "Animation Utility Classes", "Hover Replay"])
    .on_paper(),
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
    )
    .html("div", "wf-skeleton")
    .css(&["Skeleton", "Animation Keyframes", "Animation Utility Classes", "Hover Replay"])
    .not_on_paper(NOT_ON_PAPER),
    // ─── Content ──────────────────────────────────────────────────────────
    comp(
        "Callout",
        "Content",
        "A note set apart from the text around it — `.note`, `.tip`, `.important`, `.warning`, `.caution` — with an icon and a title, as GitHub's alerts are. Its text is the positional, or its block.",
        Some(special("text", PropType::Str, "What it says")),
        &[
            variant(
                "kind",
                &[
                    case("note", "", "Context worth having (the default)"),
                    case("tip", "", "A way to do it better or sooner"),
                    case("important", "", "Something to know before going on"),
                    case("warning", "", "A way to lose work or ship a bug"),
                    case("caution", "", "A way to lose data or expose a secret"),
                ],
                "What kind of note it is",
            ),
            special("title", PropType::Str, "Its title; unset, the kind's word (`Tip`)"),
        ],
        &["Title"],
        G,
        Children::Elements,
    )
    .html("div", "wf-callout")
    .css(&["Callout", "Icon"])
    .on_paper(),
    part(
        "Callout",
        "Title",
        Ir::Sub("Callout", "Title"),
        "Its title row: the icon and the words. Drawn for it; written by hand to draw something else there.",
        None,
        &[],
        G,
        Children::Elements,
    )
    .html("p", "wf-callout__title"),
    comp(
        "Kbd",
        "Content",
        "A key or a combination, written as this system writes it: `Kbd(\"mod+k\")` is `⌘K` on a Mac and `Ctrl+K` elsewhere.",
        Some(required(special("keys", PropType::Str, "The combination, as `on key` names it: `mod+k`, `shift+Enter`"))),
        &[],
        &[],
        G,
        Children::None,
    )
    .html("kbd", "wf-kbd")
    .css(&["Kbd"])
    .on_paper(),
    comp(
        "EmptyState",
        "Content",
        "What a list or a page shows when it has nothing yet: an icon, a title, a sentence and, in its block, what to do about it.",
        None,
        &[
            special("icon", PropType::Str, "An icon's name, drawn large above the title"),
            required(special("title", PropType::Str, "What is missing: `No projects yet`")),
            special("text", PropType::Str, "A sentence on what to do about it"),
        ],
        &["Title", "Text", "Actions"],
        G,
        Children::Elements,
    )
    .html("div", "wf-empty-state")
    .css(&["Empty State", "Icon"])
    .on_paper(),
    part("EmptyState", "Title", Ir::Sub("EmptyState", "Title"), "Its title.", Some(special("text", PropType::Str, "The title")), &[], G, Children::None)
        .html("p", "wf-empty-state__title"),
    part("EmptyState", "Text", Ir::Sub("EmptyState", "Text"), "Its sentence.", Some(special("text", PropType::Str, "The sentence")), &[], G, Children::None)
        .html("p", "wf-empty-state__text"),
    part("EmptyState", "Actions", Ir::Sub("EmptyState", "Actions"), "What to do: the block it was given.", None, &[], G, Children::Elements)
        .html("div", "wf-empty-state__actions"),
    comp(
        "Stat",
        "Content",
        "A figure with its label — revenue, sign-ups — and how it moved: `Stat(\"Revenue\", value: total, format: .currency, change: 0.124)`. `count:` counts to a new value.",
        Some(required(special("label", PropType::Str, "What the figure is"))),
        &[
            required(special("value", PropType::Any, "The figure")),
            special(
                "format",
                PropType::Enum(&[
                    case("number", "", "As a number (the default)"),
                    case("integer", "", "A whole number"),
                    case("currency", "", "Money, in the page's currency"),
                    case("percent", "", "A fraction as a percentage"),
                    case("compact", "", "Short: 1.2K"),
                ]),
                "How the figure is written, as `format` writes it",
            ),
            special("change", PropType::Num, "How it moved, as a fraction: `0.124` is up 12.4%"),
        ],
        &["Label", "Value", "Change"],
        G,
        Children::Elements,
    )
    .html("div", "wf-stat")
    .css(&["Stat"])
    .on_paper(),
    part("Stat", "Label", Ir::Sub("Stat", "Label"), "Its label.", Some(special("text", PropType::Str, "The label")), &[], G, Children::None)
        .html("p", "wf-stat__label"),
    part("Stat", "Value", Ir::Sub("Stat", "Value"), "Its figure.", Some(special("text", PropType::Any, "The figure")), &[], G, Children::None)
        .html("p", "wf-stat__value"),
    part("Stat", "Change", Ir::Sub("Stat", "Change"), "How it moved, with its direction.", Some(special("text", PropType::Str, "The change")), &[], G, Children::None)
        .html("p", "wf-stat__change"),
    comp(
        "CodeBlock",
        "Content",
        "A block of code with a bar above it: a file name, a copy button (`.copy`), and line numbers (`.lines`). `language:` colours it, as on `Code`.",
        Some(required(special("code", PropType::Str, "The code"))),
        &[
            special("language", PropType::Str, "Colours the code as this language: `wf`, `json`, `bash` or `css`"),
            special("title", PropType::Str, "The file name or the language, in the bar"),
            special("copy", PropType::Bool, "A button in the bar that copies the code"),
            special("lines", PropType::Bool, "Line numbers beside it"),
        ],
        &["Bar", "Lines"],
        G,
        Children::None,
    )
    .html("div", "wf-code-block")
    .css(&["Code Block", "Code", "Highlight", "Copy Button", "Button"])
    .on_paper(),
    part("CodeBlock", "Bar", Ir::Sub("CodeBlock", "Bar"), "The bar above the code.", None, &[], G, Children::Elements)
        .html("div", "wf-code-block__bar"),
    part("CodeBlock", "Lines", Ir::Sub("CodeBlock", "Lines"), "The line numbers.", Some(special("text", PropType::Str, "The numbers, a line each")), &[], G, Children::None)
        .html("pre", "wf-code-block__lines"),
    ComponentSig {
        events: &["remove"],
        ..comp(
            "Chip",
            "Content",
            "A compact label — a filter, a tag, a person — that can carry an icon, be pressed (`selected:`, with `on click`), and be removed (`.removable`, with `on remove`). `Tag` is the one that does none of it.",
            Some(required(special("label", PropType::Str, "Its words"))),
            &[
                special("icon", PropType::Str, "An icon before the words"),
                special("selected", PropType::Bool, "Pressed: a filter that is on. Makes it a toggle button"),
                special("removable", PropType::Bool, "A button that removes it, named `Remove {label}`, firing `on remove`"),
            ],
            &["Action", "Label", "Remove"],
            G,
            Children::None,
        )
    }
    .html("span", "wf-chip")
    .css(&["Chip", "Icon", "Icon Button"])
    .on_paper(),
    part("Chip", "Action", Ir::Sub("Chip", "Action"), "What pressing a chip presses: its icon and words, a toggle button when it can be `selected:`.", Some(special("text", PropType::Str, "Its words")), &[], G, Children::Elements)
        .html("button", "wf-chip__action"),
    part("Chip", "Label", Ir::Sub("Chip", "Label"), "Its icon and words, when pressing it does nothing.", Some(special("text", PropType::Str, "Its words")), &[], G, Children::Elements)
        .html("span", "wf-chip__label"),
    part("Chip", "Remove", Ir::Sub("Chip", "Remove"), "Its remove button.", None, &[], G, Children::Elements)
        .html("button", "wf-chip__remove"),
    comp(
        "VisuallyHidden",
        "Content",
        "Words for a screen reader only: `Link(…) { Text(\"Docs\")  VisuallyHidden(\"(opens in a new tab)\") }`.",
        Some(special("text", PropType::Str, "The words")),
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("span", "wf-visually-hidden")
    .on_paper(),
    comp(
        "CopyButton",
        "Actions",
        "A button that copies `value:` to the clipboard, says `Copied` for a moment, and tells a screen reader so.",
        Some(special("label", PropType::Str, "Its words; unset, `Copy`")),
        &[
            required(special("value", PropType::Any, "What it copies")),
            special("copied", PropType::Str, "What it says once it has copied; unset, `Copied`"),
        ],
        &[],
        G,
        Children::None,
    )
    .html("button", "wf-btn wf-copy-button")
    .css(&["Button", "Copy Button"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "ThemeToggle",
        "Actions",
        "A button that switches the page between light and dark, over `setTheme`: a moon in a light page, a sun in a dark one, named for the theme it switches to. The reader's choice is kept.",
        None,
        &[],
        &[],
        G,
        Children::None,
    )
    .html("button", "wf-icon-btn wf-theme-toggle")
    .css(&["Icon Button", "Theme Toggle", "Icon"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "LocaleSwitch",
        "Actions",
        "One button for each of `i18n.locales`, each written in its own language (`English`, `العربية`), the current one pressed; a press is `setLocale`.",
        None,
        &[special("label", PropType::Str, "The group's name for a screen reader; unset, `Language`")],
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-locale-switch")
    .css(&["Button", "Locale Switch"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "Accordion",
        "Content",
        "Sections a reader opens one by one, each a native `<details>`: `Accordion.Item(\"Shipping\") { … }`. `.single` keeps one open at a time.",
        None,
        &[special("single", PropType::Bool, "One open at a time: opening one closes the others")],
        &["Item", "Summary", "Panel"],
        G,
        Children::Elements,
    )
    .html("div", "wf-accordion")
    .css(&["Accordion"])
    .on_paper(),
    part(
        "Accordion",
        "Item",
        Ir::Sub("Accordion", "Item"),
        "One section: its title is what a reader presses, its block what opens.",
        Some(required(special("title", PropType::Str, "What a reader presses"))),
        &[
            attr_flag("open", "Open when it is drawn"),
            attr("name", PropType::Str, "Items of one name open one at a time: what `.single` writes"),
        ],
        G,
        Children::Elements,
    )
    .html("details", "wf-accordion__item"),
    part("Accordion", "Summary", Ir::Sub("Accordion", "Summary"), "What a reader presses to open an item: its title. Drawn for it.", Some(special("text", PropType::Str, "The title")), &[], G, Children::Elements)
        .html("summary", "wf-accordion__summary"),
    part("Accordion", "Panel", Ir::Sub("Accordion", "Panel"), "What opens: the item's block. Drawn for it.", None, &[], G, Children::Elements)
        .html("div", "wf-accordion__panel"),
    comp(
        "Field",
        "Form",
        "A label, a hint and an error around a control of your own: the first control in its block is named by the label and described by the hint and the error, and marked invalid while there is one.",
        None,
        &[LABEL, HINT, ERROR],
        &["Label", "Hint", "Error"],
        G,
        Children::Elements,
    )
    .html("div", "wf-field wf-field--custom")
    .css(&["Input", "Field"])
    .on_paper(),
    part("Field", "Label", Ir::Sub("Field", "Label"), "Its label. Drawn for it.", Some(special("text", PropType::Str, "The label")), &[], G, Children::Elements)
        .html("label", "wf-label"),
    part("Field", "Hint", Ir::Sub("Field", "Hint"), "Its hint. Drawn for it.", Some(special("text", PropType::Str, "The hint")), &[], G, Children::Elements)
        .html("p", "wf-field__hint"),
    part("Field", "Error", Ir::Sub("Field", "Error"), "Its error, announced as it appears. Drawn for it.", Some(special("text", PropType::Str, "The error")), &[], G, Children::Elements)
        .html("p", "wf-field__error"),
    comp(
        "Fieldset",
        "Form",
        "Controls that belong together — an address, a choice of plan — under a legend that names them.",
        None,
        &[required(special("legend", PropType::Str, "What the group is"))],
        &["Legend"],
        G,
        Children::Elements,
    )
    .html("fieldset", "wf-fieldset")
    .css(&["Field"])
    .on_paper(),
    part("Fieldset", "Legend", Ir::Sub("Fieldset", "Legend"), "Its legend. Drawn for it.", Some(special("text", PropType::Str, "The legend")), &[], G, Children::Elements)
        .html("legend", "wf-fieldset__legend"),
    comp(
        "InputGroup",
        "Form",
        "An input drawn as one box with what goes beside it: an icon, `InputGroup.Addon(\"https://\")`, a button, or `InputGroup.Clear`, which empties the input and gives it the focus back.",
        None,
        &[],
        &["Addon", "Clear"],
        G,
        Children::Elements,
    )
    .html("div", "wf-input-group")
    .css(&["Input", "Input Group", "Icon"])
    .on_paper(),
    part("InputGroup", "Addon", Ir::Sub("InputGroup", "Addon"), "Text beside the input: `https://`, `.com`, `kg`.", Some(special("text", PropType::Str, "The text")), &[], G, Children::Elements)
        .html("span", "wf-input-group__addon"),
    part("InputGroup", "Clear", Ir::Sub("InputGroup", "Clear"), "A button that empties the input and puts the focus back in it; hidden while it is empty.", None, &[], G, Children::None)
        .html("button", "wf-input-group__clear"),
    comp(
        "OnThisPage",
        "Navigation",
        "The page's headings as links, the one being read marked as the reader scrolls: a table of contents beside a long page.",
        None,
        &[
            special("levels", PropType::Num, "How many levels below the title it lists: `2` is `h2` and `h3` (the default 1, `h2` alone)"),
            special("label", PropType::Str, "Its title and name; unset, `On this page`"),
        ],
        &["Title"],
        G,
        Children::None,
    )
    .html("nav", "wf-on-this-page")
    .css(&["On This Page"])
    .not_on_paper("a PDF lists its headings with TableOfContents"),
    part("OnThisPage", "Title", Ir::Sub("OnThisPage", "Title"), "Its title. Drawn for it.", Some(special("text", PropType::Str, "The title")), &[], G, Children::None)
        .html("p", "wf-on-this-page__title"),
    // ─── Choosing ─────────────────────────────────────────────────────────
    comp(
        "Toggle",
        "Actions",
        "A button that stays pressed while its state holds — bold in an editor, a filter that is on: `aria-pressed`, a press flipping `bind:`.",
        Some(required(special("label", PropType::Str, "Its words"))),
        &[
            required(special("bind", PropType::State, "The `Bool` state it shows and flips")),
            special("icon", PropType::Str, "An icon before the words"),
        ],
        &[],
        G,
        Children::None,
    )
    .html("button", "wf-btn wf-toggle")
    .css(&["Button", "Toggle", "Icon"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "ToggleGroup",
        "Actions",
        "Buttons of which one is chosen — a view switcher, a segmented control — or, `.multiple`, several: one stop for Tab, the arrow keys inside it, each `aria-pressed`. `bind:` holds the chosen `value:` (a list with `.multiple`).",
        None,
        &[
            required(special("bind", PropType::State, "The chosen value, or a list of them with `.multiple`")),
            required(special("label", PropType::Str, "What the group chooses, for a screen reader")),
            special("multiple", PropType::Bool, "Several at once: `bind:` is a list"),
        ],
        &["Item"],
        G,
        Children::Elements,
    )
    .html("div", "wf-toggle-group")
    .css(&["Button", "Toggle", "Icon"])
    .not_on_paper("interactive elements are not supported in PDF"),
    part(
        "ToggleGroup",
        "Item",
        Ir::Sub("ToggleGroup", "Item"),
        "One choice: its words, and the value `bind:` holds when it is chosen.",
        Some(required(special("label", PropType::Str, "Its words"))),
        &[
            required(special("value", PropType::Any, "What `bind:` holds when it is chosen")),
            special("icon", PropType::Str, "An icon before the words"),
            DISABLED,
        ],
        G,
        Children::None,
    )
    .html("button", "wf-btn wf-toggle-group__item"),
    comp(
        "RadioGroup",
        "Form",
        "One choice of several, as native radio buttons under a legend: the arrow keys move between them, and `.cards` draws each as a card with its hint.",
        None,
        &[
            required(special("bind", PropType::State, "The chosen value")),
            required(special("label", PropType::Str, "What is being chosen: the group's legend")),
            special("cards", PropType::Bool, "Each option a card, its hint beneath its words"),
        ],
        &["Option", "Legend", "Hint"],
        G,
        Children::Elements,
    )
    .html("fieldset", "wf-radio-group")
    .css(&["Radio", "Radio Group", "Field"])
    .on_paper(),
    part(
        "RadioGroup",
        "Option",
        Ir::Sub("RadioGroup", "Option"),
        "One choice: its words, its value, and a hint said with it.",
        Some(required(special("label", PropType::Str, "Its words"))),
        &[
            required(special("value", PropType::Any, "What `bind:` holds when it is chosen")),
            special("hint", PropType::Str, "A line beneath it, read with it"),
        ],
        G,
        Children::Elements,
    )
    .html("div", "wf-radio-group__option"),
    part("RadioGroup", "Legend", Ir::Sub("RadioGroup", "Legend"), "Its legend. Drawn for it.", Some(special("text", PropType::Str, "The legend")), &[], G, Children::Elements)
        .html("legend", "wf-radio-group__legend"),
    part("RadioGroup", "Hint", Ir::Sub("RadioGroup", "Hint"), "An option's hint. Drawn for it.", Some(special("text", PropType::Str, "The hint")), &[], G, Children::Elements)
        .html("p", "wf-radio-group__hint"),
    comp(
        "Pagination",
        "Navigation",
        "Pages of a list: Previous, the first and last pages, those around the current one with a gap for the rest, and Next. The current page is `aria-current=\"page\"`; `bind:` holds it, from 1.",
        None,
        &[
            required(special("bind", PropType::State, "The current page, from 1")),
            required(special("pages", PropType::Num, "How many pages there are")),
            special("siblings", PropType::Num, "How many pages either side of the current one it shows (unset, 1)"),
            special("label", PropType::Str, "Its name for a screen reader; unset, `Pagination`"),
        ],
        &["Prev", "Next", "Page", "Gap"],
        G,
        Children::None,
    )
    .html("nav", "wf-pagination")
    .css(&["Button", "Pagination", "Icon"])
    .not_on_paper("interactive elements are not supported in PDF"),
    part("Pagination", "Prev", Ir::Sub("Pagination", "Prev"), "The previous page. Drawn for it.", None, &[DISABLED], G, Children::Elements)
        .html("button", "wf-btn wf-pagination__prev"),
    part("Pagination", "Next", Ir::Sub("Pagination", "Next"), "The next page. Drawn for it.", None, &[DISABLED], G, Children::Elements)
        .html("button", "wf-btn wf-pagination__next"),
    part("Pagination", "Page", Ir::Sub("Pagination", "Page"), "A page. Drawn for it.", Some(special("text", PropType::Str, "Its number")), &[], G, Children::Elements)
        .html("button", "wf-btn wf-pagination__page"),
    part("Pagination", "Gap", Ir::Sub("Pagination", "Gap"), "The pages left out. Drawn for it.", Some(special("text", PropType::Str, "An ellipsis")), &[], G, Children::Elements)
        .html("span", "wf-pagination__gap"),
    comp(
        "Stepper",
        "Navigation",
        "Steps taken in order — an account, then billing, then a summary — with each step's state (done, current, an error) and the current step's block shown. `bind:` holds the current step, from 0; `.linear` keeps a step out of reach until the ones before it are passed.",
        None,
        &[
            required(special("bind", PropType::State, "The current step, from 0")),
            special("linear", PropType::Bool, "A step ahead of the current one cannot be pressed"),
        ],
        &["Step", "List", "Item"],
        G,
        Children::Elements,
    )
    .html("div", "wf-stepper")
    .css(&["Button", "Stepper"])
    .on_paper(),
    part(
        "Stepper",
        "Step",
        Ir::Sub("Stepper", "Step"),
        "One step: its title in the list, its block shown while it is the current one.",
        Some(required(special("title", PropType::Str, "Its title"))),
        &[
            special("done", PropType::Bool, "Passed: drawn with a check"),
            special("error", PropType::Bool, "Something in it needs fixing"),
        ],
        G,
        Children::Elements,
    )
    .html("div", "wf-stepper__panel"),
    part("Stepper", "List", Ir::Sub("Stepper", "List"), "The steps, in order. Drawn for it.", None, &[], G, Children::Elements)
        .html("ol", "wf-stepper__list"),
    part("Stepper", "Item", Ir::Sub("Stepper", "Item"), "A step in the list. Drawn for it.", None, &[], G, Children::Elements)
        .html("li", "wf-stepper__item"),
    comp(
        "NumberInput",
        "Form",
        "A number with − and + beside it: the arrow keys step it, PageUp and PageDown by ten steps, and it stays between `min:` and `max:`.",
        None,
        &[
            required(special("bind", PropType::State, "The `Number` it holds")),
            special("min", PropType::Num, "The lowest it goes"),
            special("max", PropType::Num, "The highest it goes"),
            special("step", PropType::Num, "How far a press moves it (unset, 1)"),
            LABEL,
            HINT,
        ],
        &["Decrease", "Increase"],
        G,
        Children::None,
    )
    .html("div", "wf-number-input")
    .css(&["Input", "Field", "Input Group", "Number Input", "Icon"])
    .not_on_paper("interactive elements are not supported in PDF"),
    part("NumberInput", "Decrease", Ir::Sub("NumberInput", "Decrease"), "Its − button. Drawn for it.", None, &[DISABLED], G, Children::Elements)
        .html("button", "wf-number-input__step"),
    part("NumberInput", "Increase", Ir::Sub("NumberInput", "Increase"), "Its + button. Drawn for it.", None, &[DISABLED], G, Children::Elements)
        .html("button", "wf-number-input__step"),
    comp(
        "Calendar",
        "Form",
        "A month as a grid of days, chosen with a press or the keyboard — the arrows by day and week, PageUp and PageDown by month, Home and End to the week's ends — one day, a range, or several. `bind:` holds a `Date`, a `{ start, end }` with `.range`, a list with `.multiple`.",
        None,
        &[
            required(special("bind", PropType::State, "The chosen day, range or days")),
            special(
                "mode",
                PropType::Enum(&[
                    case("single", "", "One day (the default)"),
                    case("range", "", "A first and a last day: `{ start, end }`"),
                    case("multiple", "", "Any number of days: a list"),
                ]),
                "How many days it holds",
            ),
            special("min", PropType::Any, "The first day that can be chosen"),
            special("max", PropType::Any, "The last day that can be chosen"),
            special("disabled", PropType::Any, "Days that cannot be chosen: a list of them, or `d => d.weekday() >= 6`"),
            special("label", PropType::Str, "Its name for a screen reader; unset, `Calendar`"),
        ],
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-calendar")
    .css(&["Calendar"])
    .not_on_paper("interactive elements are not supported in PDF"),
    // ─── Actions ──────────────────────────────────────────────────────────
    comp(
        "Button",
        "Actions",
        "A button: its label or block is what it shows, `on click` what it does.",
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
            special("to", PropType::Path, "The route to go to when it is pressed"),
            DISABLED,
            special(
                "loading",
                PropType::Bool,
                "Busy while it holds — `loading: Checkout.pay.pending`: a spinner in place of its label, at the same width, and a press does nothing",
            ),
        ],
        &[],
        G,
        Children::Elements,
    )
    .html("button", "wf-btn")
    .css(&["Button"])
    .not_on_paper("interactive elements are not supported in PDF"),
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
    )
    .html("button", "wf-icon-btn")
    .css(&["Button", "Icon Button", "Icon"])
    .not_on_paper("interactive elements are not supported in PDF"),
    comp(
        "ButtonGroup",
        "Actions",
        "Buttons joined into one row.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-btn-group")
    .css(&["Button", "Button Group"])
    .not_on_paper("interactive elements are not supported in PDF"),
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
    )
    .html("div", "wf-dropdown")
    .css(&["Button", "Dropdown", "Layer"])
    .not_on_paper("interactive elements are not supported in PDF"),
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
        "Image; `alt` is required for accessibility (rule A01). Given an `image` the program declares, it becomes a `<picture>` with every width the build wrote.",
        Some(special(
            "source",
            PropType::Any,
            "An `image` the program declares, or a URL",
        )),
        &[
            attr("src", PropType::File(FileKind::Image), "Image URL"),
            special(
                "sizes",
                PropType::Str,
                "How wide it will be shown, so the browser picks a width: `(max-width: 768px) 100vw, 1200px`",
            ),
            special(
                "placeholder",
                PropType::Enum(&[
                    case(
                        "blur",
                        "blur",
                        "A tiny blurred copy, until the real one lands",
                    ),
                    case("color", "color", "The image's average colour"),
                    case("none", "none", "Nothing"),
                ]),
                "What fills the box while the image loads",
            ),
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
    )
    .html("img", "wf-image")
    .css(&["Image"])
    .on_paper(),
    comp(
        "Video",
        "Media",
        "Video player. Without `captions` it draws an `A09` warning: a video nobody can hear is a video nobody can follow.",
        None,
        &[
            attr("src", PropType::File(FileKind::Video), "Video URL"),
            attr_flag("controls", "Show the browser's playback controls"),
            attr_flag("autoplay", "Start playing when shown"),
            attr(
                "poster",
                PropType::File(FileKind::Image),
                "The frame shown before it plays",
            ),
            attr_flag("muted", "Start with the sound off"),
            attr_flag("loop", "Start again when it ends"),
            attr_flag("playsinline", "Play where it sits, rather than full screen"),
            special(
                "captions",
                PropType::File(FileKind::Captions),
                "A WebVTT file of captions, which becomes a `<track>`",
            ),
            attr("width", PropType::Num, "Intrinsic width"),
            attr("height", PropType::Num, "Intrinsic height"),
        ],
        &[],
        G_MEDIA,
        Children::None,
    )
    .html("video", "wf-video")
    .not_on_paper("a PDF cannot play or animate it"),
    comp(
        "Audio",
        "Media",
        "Audio player. `transcript` links what was said, which is what a reader who cannot hear it needs.",
        None,
        &[
            attr("src", PropType::File(FileKind::Audio), "Audio URL"),
            attr_flag("controls", "Show the browser's playback controls"),
            attr_flag("autoplay", "Start playing when shown"),
            attr_flag("loop", "Start again when it ends"),
            special(
                "transcript",
                PropType::File(FileKind::Document),
                "A link to what was said, shown beneath the player",
            ),
        ],
        &[],
        G,
        Children::None,
    )
    .html("audio", "wf-audio")
    .not_on_paper("a PDF cannot play or animate it"),
    ComponentSig {
        events: &["message", "load"],
        ..comp(
            "Frame",
            "Media",
            "Another page, shown inside this one: an `iframe` — the page at `src:`, or the HTML `srcdoc:` gives it, sandboxed as `sandbox:` says. `on message(m)` hears what that page posts to this one (`m.data`), and nothing any other page posts. Without `title:` it draws an `A17` warning: a frame is announced by its title.",
            None,
            &[
                attr("src", PropType::Str, "The address of the page it shows"),
                attr(
                    "srcdoc",
                    PropType::Str,
                    "The HTML it draws in place of an address, sandboxed as `sandbox:` says; a value that reads state draws the new HTML as it changes",
                ),
                attr(
                    "title",
                    PropType::Str,
                    "What it shows, in a few words: its accessible name",
                ),
                attr(
                    "sandbox",
                    PropType::Str,
                    "What the framed page may do: `\"allow-scripts allow-forms\"`; empty, it may do nothing",
                ),
                attr(
                    "allow",
                    PropType::Str,
                    "The features it may use: `\"fullscreen; clipboard-write\"`",
                ),
                attr(
                    "loading",
                    PropType::Enum(&[
                        case("lazy", "", "Load when near the viewport"),
                        case("eager", "", "Load at once"),
                    ]),
                    "Unset, at once",
                ),
            ],
            &[],
            G,
            Children::None,
        )
    }
    .html("iframe", "wf-frame")
    .css(&["Frame"])
    .not_on_paper("a PDF cannot show another page inside it"),
    comp(
        "Icon",
        "Media",
        "One of the built-in SVG icons, rendered inline.",
        Some(special("name", PropType::Str, "The icon name")),
        &[SIZE, variant("tone", TONES, "Colour"), MUTED],
        &[],
        G,
        Children::None,
    )
    .html("i", "wf-icon")
    .css(&["Icon"])
    .on_paper(),
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
    )
    .html("div", "wf-carousel")
    .css(&["Image", "Carousel"])
    .not_on_paper("a PDF cannot play or animate it"),
    part(
        "Carousel",
        "Slide",
        Ir::Sub("Carousel", "Slide"),
        "One slide of a carousel.",
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
    )
    .html("p", "wf-text")
    .on_paper(),
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
    )
    .html("h2", "wf-heading")
    .on_paper(),
    comp(
        "Code",
        "Typography",
        "Inline code, or a code block with `block`; `language:` colours it.",
        Some(special("content", PropType::Any, "The code")),
        &[
            flag("block", "block", "A multi-line block"),
            special(
                "language",
                PropType::Str,
                "Colours the code as this language: `wf`, `json`, `bash` or `css`",
            ),
        ],
        &[],
        G,
        Children::None,
    )
    .html("code", "wf-code")
    .css(&["Code", "Highlight"])
    .on_paper(),
    comp(
        "Markdown",
        "Typography",
        "Markdown text rendered as HTML: headings, paragraphs, lists, quotes, code, links, emphasis. The text is escaped first, so HTML in it is shown, not run, and a link or picture whose URL a browser would run is drawn as its words. `.inert` is for text somebody else wrote.",
        Some(special("text", PropType::Str, "The Markdown text")),
        &[special(
            "inert",
            PropType::Bool,
            "Text somebody else wrote, such as a language server's documentation: its entities are decoded, a link is drawn as its words and a picture is left out",
        )],
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-markdown")
    .css(&["Highlight", "Markdown"])
    .on_paper(),
    comp(
        "Host",
        "Layout",
        "A bare node handed to somebody else's code, with a lifetime. `mount:` runs with the node each time it is made, `update:` again whenever the state it reads changes, and `cleanup:` when the page, branch or list item it belongs to leaves. Every other element takes the same three.",
        None,
        &[special(
            "tag",
            PropType::Str,
            "The element to make: `div` (the default), `span`, `canvas`, `svg`, `section`, `figure`, `pre`, `p`, `ul` or `table`",
        )],
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-host")
    .not_on_paper("a script draws it in a browser, and a PDF runs no script"),
    comp(
        "Portal",
        "Layout",
        "Its block, drawn into a node somebody else's code owns: an element a script hands over, or a `ref:` handle. When `into:` names another node the block moves there with its nodes and state; `null` keeps it aside until a node comes back; it goes when the page, branch or list item it is written in leaves. What is inside belongs to where it is written: its handlers, `emit` and lifetimes.",
        None,
        &[required(special(
            "into",
            PropType::Any,
            "Where it draws: an element, a `ref:` handle, or `null` to keep it aside; read again whenever what it reads changes",
        ))],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-portal")
    .css(&["Portal"])
    .not_on_paper("a script draws it in a browser, and a PDF runs no script"),
    comp(
        "Element",
        "Layout",
        "A custom element a script defines, placed by its tag: `Element(\"stripe-pricing-table\", publishableKey: key) { on ready { … } }`. Every named argument is an attribute, every `on` handler an event it fires, and the block its children.",
        Some(special(
            "tag",
            PropType::Str,
            "The custom element's tag: lower case, with a hyphen, as every custom element's is",
        )),
        &[],
        &[],
        &[AttrFamily::Any],
        Children::Elements,
    )
    .not_on_paper("a script draws it in a browser, and a PDF runs no script"),
    comp(
        "Unsafe",
        "Typography",
        "The one door for markup a page did not write. It has a single part, `Unsafe.Html`.",
        None,
        &[],
        &["Html"],
        G,
        Children::None,
    )
    .css(&["Highlight"])
    .on_paper(),
    part(
        "Unsafe",
        "Html",
        Ir::BuiltIn("UnsafeHtml"),
        "Markup, put in as markup. It is **not** sanitised — the name says so — and `sanitize(…)` is one call away for anything the project did not write itself.",
        Some(special("markup", PropType::Str, "The HTML to insert")),
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-html")
    .on_paper(),
    comp(
        "Blockquote",
        "Typography",
        "Quotation: the text, or its block as the quoted content.",
        Some(special("content", PropType::Any, "The quoted text")),
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("blockquote", "wf-blockquote")
    .css(&["Blockquote"])
    .on_paper(),
    // ─── Charts and codes ─────────────────────────────────────────────────
    comp(
        "Chart",
        "Data display",
        "A chart drawn as vector graphics when the page is built: bars, lines, areas, a pie or a donut over a list of records.",
        None,
        &[
            special(
                "kind",
                PropType::Enum(&[
                    case(
                        "bar",
                        "",
                        "Bars, one per row (grouped or stacked for several series)",
                    ),
                    case("line", "", "A line through the rows"),
                    case("area", "", "A line with the area under it filled"),
                    case("pie", "", "Slices of a whole"),
                    case("donut", "", "A pie with a hole"),
                ]),
                "What it draws",
            ),
            special(
                "data",
                PropType::Any,
                "The rows: a list of records, or of numbers",
            ),
            special("x", PropType::Str, "The key of each row's label"),
            special(
                "y",
                PropType::Any,
                "The key of each row's value, or a list of keys for several series",
            ),
            special("colors", PropType::Any, "The series' colours, in order"),
            special(
                "width",
                PropType::Num,
                "Its width in user units (it scales to its box)",
            ),
            special("height", PropType::Num, "Its height in user units"),
            special("ink", PropType::Str, "The colour of its labels"),
            special("grid", PropType::Str, "The colour of its grid lines"),
            special("legend", PropType::Bool, "Name the series"),
            special("stacked", PropType::Bool, "Stack a bar chart's series"),
            special(
                "labels",
                PropType::Bool,
                "Write each bar's or slice's value on it",
            ),
            special(
                "unit",
                PropType::Str,
                "A unit after each value on the axis: `ms`, `%`",
            ),
        ],
        &[],
        G,
        Children::None,
    )
    .html("figure", "wf-chart")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "QrCode",
        "Data display",
        "A QR code for a URL or any text, drawn as vector graphics when the page is built.",
        Some(special("value", PropType::Str, "What it encodes")),
        &[
            special("color", PropType::Str, "The colour of the dark modules"),
            special("background", PropType::Str, "The colour behind them"),
        ],
        &[],
        G,
        Children::None,
    )
    .html("figure", "wf-qr")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "TableOfContents",
        "PDF",
        "The document's headings with the page each lands on, each a link to it.",
        None,
        &[
            special(
                "levels",
                PropType::Num,
                "How deep it lists: 1 is the `h1`s only (default 3)",
            ),
            special("title", PropType::Str, "A heading above the list"),
        ],
        &[],
        G,
        Children::None,
    )
    .html("nav", "wf-toc")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    // ─── Document (PDF) ───────────────────────────────────────────────────
    comp(
        "Document",
        "PDF",
        "Root of a PDF document: the paper, the margins and what the file says about itself.",
        None,
        &[
            special(
                "page_size",
                PropType::Enum(&[
                    case("A4", "", ""),
                    case("A3", "", ""),
                    case("A5", "", ""),
                    case("Letter", "", ""),
                    case("Legal", "", ""),
                ]),
                "Paper size (also `size:`)",
            ),
            special(
                "size",
                PropType::Str,
                "Paper size: `A4`, `Letter`, `Tabloid`, or two lengths, `210mm 297mm`",
            ),
            special("landscape", PropType::Bool, "Turn the paper on its side"),
            special(
                "margin",
                PropType::Any,
                "The page margins: one length, or top right bottom left (`72`, `20mm 25mm`)",
            ),
            special(
                "title",
                PropType::Str,
                "The file's title, shown by a reader in place of its name",
            ),
            special(
                "author",
                PropType::Str,
                "The file's author; several separated by commas",
            ),
            special("subject", PropType::Str, "What the document is about"),
            special("keywords", PropType::Str, "Comma-separated keywords"),
            special(
                "lang",
                PropType::Str,
                "The document's language, for screen readers and hyphenation",
            ),
        ],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-document")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "Section",
        "PDF",
        "Groups content with spacing; on the web, a `<section>`.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("section", "wf-section")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "Paragraph",
        "PDF",
        "Block of text with paragraph spacing.",
        Some(special("content", PropType::Any, "The text")),
        TEXT_PROPS,
        &[],
        G,
        Children::None,
    )
    .html("p", "wf-text")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "Header",
        "PDF",
        "Repeated at the top of every PDF page (or those `on:` names), where `page` and `pages` are the page's number and the count; a `<header>` on the web.",
        None,
        &[special(
            "on",
            PropType::Enum(ON_PAGES),
            "Which pages it is drawn on",
        )],
        &[],
        G,
        Children::Elements,
    )
    .html("header", "wf-header")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "Footer",
        "PDF",
        "Repeated at the bottom of every PDF page (or those `on:` names), where `page` and `pages` are the page's number and the count; a `<footer>` on the web.",
        None,
        &[special(
            "on",
            PropType::Enum(ON_PAGES),
            "Which pages it is drawn on",
        )],
        &[],
        G,
        Children::Elements,
    )
    .html("footer", "wf-footer")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "Background",
        "PDF",
        "Drawn behind each PDF page's content, the size of the page: a cover, a band, a full-bleed picture.",
        None,
        &[special(
            "on",
            PropType::Enum(ON_PAGES),
            "Which pages it is drawn on",
        )],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-background")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "Watermark",
        "PDF",
        "Large, faint, rotated text behind each PDF page's content.",
        Some(special("text", PropType::Str, "The text")),
        &[special(
            "on",
            PropType::Enum(ON_PAGES),
            "Which pages it is drawn on",
        )],
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-watermark")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "PageBreak",
        "PDF",
        "Forces a new PDF page.",
        None,
        &[],
        &[],
        G,
        Children::None,
    )
    .html("div", "wf-page-break")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
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
    )
    .html("div", "wf-presentation")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "Slide",
        "Slides",
        "One slide, one PDF page; content is top-aligned and clipped at the bottom margin.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("section", "wf-slide")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "TitleSlide",
        "Slides",
        "Cover slide: title and subtitle, vertically centred.",
        Some(special("title", PropType::Str, "The title")),
        &[special("subtitle", PropType::Str, "The subtitle")],
        &[],
        G,
        Children::None,
    )
    .html("section", "wf-title-slide")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "SectionSlide",
        "Slides",
        "Full-bleed coloured band with a centred label.",
        Some(special("label", PropType::Str, "The label")),
        &[variant("tone", BAND_TONES, "Colour of the band")],
        &[],
        G,
        Children::None,
    )
    .html("section", "wf-section-slide")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "TwoColumn",
        "Slides",
        "Two equal columns; takes exactly two `Container` children.",
        None,
        &[],
        &[],
        G,
        Children::Elements,
    )
    .html("section", "wf-two-column")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    comp(
        "ImageSlide",
        "Slides",
        "Image slide with an optional caption.",
        None,
        &[
            special(
                "src",
                PropType::File(FileKind::Image),
                "Image path (required)",
            ),
            special("caption", PropType::Str, "Caption under the image"),
        ],
        &[],
        G,
        Children::None,
    )
    .html("section", "wf-image-slide")
    .css(&["Paged documents and slide decks"])
    .on_paper(),
    // ─── Routing ──────────────────────────────────────────────────────────
    comp(
        "Router",
        "Routing",
        "Where the current page renders. May sit at any depth inside `app`; pages declare their own paths.",
        None,
        &[
            special(
                "transition",
                PropType::Enum(&[
                    case("none", "", "The next page replaces the last at once"),
                    case("fade", "", "The last page fades out, the next fades in"),
                    case(
                        "slide",
                        "",
                        "The last page slides out to the left, the next in from the right",
                    ),
                ]),
                "How a route change moves between pages",
            ),
            special(
                "duration",
                PropType::Str,
                "How long each half of the transition plays, e.g. `200ms`",
            ),
        ],
        &[],
        G,
        Children::Elements,
    )
    .html("div", "wf-router")
    .not_on_paper("navigation components are web-only"),
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
    )
    .not_on_paper("navigation components are web-only"),
];
