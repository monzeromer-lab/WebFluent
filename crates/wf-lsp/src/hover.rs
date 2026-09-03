use tower_lsp::lsp_types::*;
use webfluent::codegen::builtin::{builtin_to_html, implicit_role, landmark_label};
use webfluent::parser::ast::*;
use webfluent::parser::vocabulary::MODIFIER_KEYWORDS;

// ---------------------------------------------------------------------------
// Built-in component documentation
// ---------------------------------------------------------------------------

struct BuiltinDoc {
    name: &'static str,
    summary: &'static str,
    example: &'static str,
    subcomponents: &'static [&'static str],
}

const BUILTIN_DOCS: &[BuiltinDoc] = &[
    // Layout
    BuiltinDoc {
        name: "Container",
        summary: "Responsive centered container with max-width and horizontal padding.",
        example: "Container {\n  Heading(\"Welcome\", h1)\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Row",
        summary: "Horizontal flex layout container.",
        example: "Row(gap: md) {\n  Button(\"Left\")\n  Button(\"Right\")\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Column",
        summary: "Vertical flex layout container.",
        example: "Column(gap: sm) {\n  Text(\"Item 1\")\n  Text(\"Item 2\")\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Grid",
        summary: "CSS Grid layout. Use `columns:` or `cols:` to set column count and `gap:` for spacing.",
        example: "Grid(columns: 3, gap: md) {\n  Card { ... }\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Stack",
        summary: "Stacked/overlapping layout where children are layered on top of each other.",
        example: "Stack {\n  Image(src: \"/bg.jpg\", alt: \"\")\n  Text(\"Overlay text\")\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Spacer",
        summary: "Flexible space filler that expands to take up remaining room in a flex container.",
        example: "Row {\n  Text(\"Left\")\n  Spacer\n  Text(\"Right\")\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Divider",
        summary: "Horizontal visual divider rule.",
        example: "Divider",
        subcomponents: &[],
    },

    // Navigation
    BuiltinDoc {
        name: "Navbar",
        summary: "Top navigation bar. Houses branding, links, and action buttons.",
        example: "Navbar {\n  Navbar.Brand { Text(\"Logo\") }\n  Navbar.Links { Link(\"Home\", to: \"/\") }\n  Navbar.Actions { Button(\"Sign in\") }\n}",
        subcomponents: &["Navbar.Brand", "Navbar.Links", "Navbar.Actions"],
    },
    BuiltinDoc {
        name: "Sidebar",
        summary: "Side navigation panel. Contains headers, navigation items, and dividers.",
        example: "Sidebar {\n  Sidebar.Header { Text(\"Dashboard\") }\n  Sidebar.Item(\"Overview\", to: \"/overview\")\n  Sidebar.Divider\n  Sidebar.Item(\"Settings\", to: \"/settings\")\n}",
        subcomponents: &["Sidebar.Header", "Sidebar.Item", "Sidebar.Divider"],
    },
    BuiltinDoc {
        name: "Breadcrumb",
        summary: "Breadcrumb trail showing current page hierarchy.",
        example: "Breadcrumb {\n  Breadcrumb.Item(\"Home\", to: \"/\")\n  Breadcrumb.Item(\"Docs\", to: \"/docs\")\n  Breadcrumb.Item(\"Guide\")\n}",
        subcomponents: &["Breadcrumb.Item"],
    },
    BuiltinDoc {
        name: "Link",
        summary: "Navigation hyperlink. Use `to:` for internal routes or `href:` for external URLs.",
        example: "Link(\"Documentation\", to: \"/docs\")",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Menu",
        summary: "Dropdown menu component.",
        example: "Menu {\n  Menu.Item(\"Edit\")\n  Menu.Item(\"Delete\")\n}",
        subcomponents: &["Menu.Item"],
    },
    BuiltinDoc {
        name: "Tabs",
        summary: "Tabbed container switching between multiple panels via `TabPage` children.",
        example: "Tabs {\n  TabPage(\"Overview\") { Text(\"Tab 1 content\") }\n  TabPage(\"Settings\") { Text(\"Tab 2 content\") }\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "TabPage",
        summary: "Individual tab page panel within a `Tabs` component.",
        example: "TabPage(\"Details\") {\n  Text(\"Content here\")\n}",
        subcomponents: &[],
    },

    // Data Display
    BuiltinDoc {
        name: "Card",
        summary: "Structured card surface with optional Header, Body, and Footer sections.",
        example: "Card(elevated) {\n  Card.Header { Heading(\"Title\", h3) }\n  Card.Body { Text(\"Main body text\") }\n  Card.Footer { Button(\"Action\") }\n}",
        subcomponents: &["Card.Header", "Card.Body", "Card.Footer"],
    },
    BuiltinDoc {
        name: "Table",
        summary: "Structured data table. Uses `Thead`, `Tbody`, `Trow`, and `Tcell`.",
        example: "Table {\n  Thead { Trow { Tcell(\"Name\") Tcell(\"Role\") } }\n  Tbody { Trow { Tcell(\"Alice\") Tcell(\"Admin\") } }\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Thead",
        summary: "Table header section.",
        example: "Thead { Trow { Tcell(\"Col 1\") Tcell(\"Col 2\") } }",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Tbody",
        summary: "Table body section containing data rows.",
        example: "Tbody { for item in items { Trow { Tcell(item.name) } } }",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Trow",
        summary: "Table row.",
        example: "Trow { Tcell(\"Data\") }",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Tcell",
        summary: "Table cell.",
        example: "Tcell(\"Value\")",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "List",
        summary: "List container.",
        example: "List {\n  for item in items { Text(item) }\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Badge",
        summary: "Small status indicator, pill, or counter.",
        example: "Badge(\"New\", primary, pill)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Avatar",
        summary: "User avatar image or initials display.",
        example: "Avatar(src: \"/user.jpg\", alt: \"User Profile\")",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Tooltip",
        summary: "Hover tooltip displaying contextual hints.",
        example: "Tooltip(text: \"More info\") {\n  Button(\"Help\")\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Tag",
        summary: "Compact category or status chip.",
        example: "Tag(\"Rust\", outlined)",
        subcomponents: &[],
    },

    // Inputs
    BuiltinDoc {
        name: "Input",
        summary: "Interactive form input. Supports modifiers (`text`, `email`, `password`, `number`) and `bind:`.",
        example: "Input(\"Enter email...\", email, bind: userEmail)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Select",
        summary: "Dropdown selection control populated with `Option` children.",
        example: "Select(bind: chosenRole) {\n  Option(\"Admin\", value: \"admin\")\n  Option(\"User\", value: \"user\")\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Option",
        summary: "Individual option inside a `Select` element.",
        example: "Option(\"Label\", value: \"val\")",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Checkbox",
        summary: "Boolean toggle checkbox.",
        example: "Checkbox(\"I accept terms\", bind: accepted)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Radio",
        summary: "Single-choice radio button.",
        example: "Radio(\"Option A\", name: \"choice\", value: \"a\", bind: selectedChoice)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Switch",
        summary: "Modern toggle switch (on/off).",
        example: "Switch(\"Enable notifications\", bind: notificationsEnabled)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Slider",
        summary: "Range slider input.",
        example: "Slider(min: 0, max: 100, step: 5, bind: volume)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "DatePicker",
        summary: "Date selection calendar input.",
        example: "DatePicker(bind: selectedDate)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "FileUpload",
        summary: "File upload input with drag-and-drop support.",
        example: "FileUpload(accept: \".png,.jpg\", bind: uploadedFile)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Form",
        summary: "Form wrapper handling submission and inputs.",
        example: "Form(on:submit: handleSubmit) {\n  Input(\"Name\", bind: name)\n  Button(\"Save\", submit, primary)\n}",
        subcomponents: &[],
    },

    // Feedback
    BuiltinDoc {
        name: "Alert",
        summary: "Notice banner for feedback. Modifiers: `success`, `warning`, `danger`, `info`.",
        example: "Alert(\"Changes saved successfully!\", success, dismissible)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Toast",
        summary: "Floating notification toast message.",
        example: "Toast(\"Saved\", success)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Modal",
        summary: "Accessible dialog modal dialog overlay.",
        example: "Modal(title: \"Confirm Action\", visible: isModalOpen) {\n  Modal.Body { Text(\"Are you sure?\") }\n  Modal.Footer { Button(\"Cancel\") Button(\"Confirm\", danger) }\n}",
        subcomponents: &["Modal.Header", "Modal.Body", "Modal.Footer"],
    },
    BuiltinDoc {
        name: "Dialog",
        summary: "Confirmation modal dialog.",
        example: "Dialog(title: \"Delete?\", visible: showConfirm) {\n  Text(\"This cannot be undone.\")\n}",
        subcomponents: &["Dialog.Header", "Dialog.Body", "Dialog.Footer"],
    },
    BuiltinDoc {
        name: "Spinner",
        summary: "Animated loading spinner indicator.",
        example: "Spinner(large, primary)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Progress",
        summary: "Progress bar showing completion status.",
        example: "Progress(value: 75, max: 100, primary)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Skeleton",
        summary: "Content loading placeholder skeleton.",
        example: "Skeleton(rounded)",
        subcomponents: &[],
    },

    // Actions
    BuiltinDoc {
        name: "Button",
        summary: "Clickable action button. Modifiers: `primary`, `secondary`, `outlined`, `danger`, `small`, `large`.",
        example: "Button(\"Submit\", primary, large, onClick: handleClick)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "IconButton",
        summary: "Icon-only button requiring `icon:` and `label:` for accessibility.",
        example: "IconButton(icon: \"search\", label: \"Search items\")",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "ButtonGroup",
        summary: "Group of cohesive buttons displayed inline.",
        example: "ButtonGroup {\n  Button(\"Day\")\n  Button(\"Week\")\n  Button(\"Month\")\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Dropdown",
        summary: "Button triggering a dropdown popover menu.",
        example: "Dropdown(\"Options\") {\n  Dropdown.Item(\"Profile\")\n  Dropdown.Item(\"Logout\")\n}",
        subcomponents: &["Dropdown.Item"],
    },

    // Media
    BuiltinDoc {
        name: "Image",
        summary: "Responsive image element. Requires `src:` and `alt:` for accessibility.",
        example: "Image(src: \"/hero.png\", alt: \"Hero banner\", rounded)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Video",
        summary: "HTML5 video player. Supports `controls:`, `autoplay:`, `loop:`.",
        example: "Video(src: \"/intro.mp4\", controls: true)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Icon",
        summary: "Vector icon element. Specify icon name via positional argument or `icon:`.",
        example: "Icon(\"check\", small)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Carousel",
        summary: "Slide show / carousel container for cycling through `Carousel.Slide` items.",
        example: "Carousel(autoplay: true, interval: 3000) {\n  Carousel.Slide { Image(src: \"/1.jpg\", alt: \"Slide 1\") }\n  Carousel.Slide { Image(src: \"/2.jpg\", alt: \"Slide 2\") }\n}",
        subcomponents: &["Carousel.Slide"],
    },

    // Typography
    BuiltinDoc {
        name: "Text",
        summary: "Paragraph or inline text block. Modifiers: `bold`, `italic`, `muted`, `uppercase`, etc.",
        example: "Text(\"Hello world\", bold, muted)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Heading",
        summary: "Section heading element. Use `h1` through `h6` modifiers to set heading depth.",
        example: "Heading(\"Main Section\", h2)",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Code",
        summary: "Inline code snippet or preformatted code block.",
        example: "Code(\"let x = 42;\")",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Blockquote",
        summary: "Quotation block for quoted references.",
        example: "Blockquote(\"Simplicity is prerequisite for reliability.\")",
        subcomponents: &[],
    },

    // PDF / Document output
    BuiltinDoc {
        name: "Document",
        summary: "Root container for print and PDF document compilation.",
        example: "Document(title: \"Invoice\") {\n  Section { Paragraph(\"Content\") }\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Section",
        summary: "Structural section in a PDF document.",
        example: "Section(title: \"Summary\") {\n  Paragraph(\"...\")\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Paragraph",
        summary: "Text paragraph within a PDF document.",
        example: "Paragraph(\"Document text\")",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "PageBreak",
        summary: "Explicit page break separator in PDF document compilation.",
        example: "PageBreak",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Header",
        summary: "Repeating header in PDF document output.",
        example: "Header { Text(\"Company Report\") }",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Footer",
        summary: "Repeating footer in PDF document output.",
        example: "Footer { Text(\"Confidential\") }",
        subcomponents: &[],
    },

    // Slides & Presentations
    BuiltinDoc {
        name: "Presentation",
        summary: "Root presentation container for slide deck compilation.",
        example: "Presentation(title: \"Tech Talk\") {\n  TitleSlide(\"WebFluent 2.0\", \"Next-gen Web\")\n  Slide { ... }\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Slide",
        summary: "Freeform 16:9 presentation slide (one output page).",
        example: "Slide {\n  Heading(\"Architecture\", h2)\n  Text(\"Key principles...\")\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "TitleSlide",
        summary: "Title slide layout: `TitleSlide(\"Title\", \"Subtitle\")`.",
        example: "TitleSlide(\"WebFluent\", \"Declarative Web Language\")",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "SectionSlide",
        summary: "Section divider slide: `SectionSlide(\"Section Label\")`.",
        example: "SectionSlide(\"Deep Dive\")",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "TwoColumn",
        summary: "Two-column slide layout. Takes exactly two `Container` children.",
        example: "TwoColumn {\n  Container { Text(\"Left content\") }\n  Container { Text(\"Right content\") }\n}",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "ImageSlide",
        summary: "Slide with full-frame image and optional caption: `ImageSlide(src: \"...\", caption: \"...\")`.",
        example: "ImageSlide(src: \"/diagram.png\", caption: \"System Overview\")",
        subcomponents: &[],
    },

    // Routing
    BuiltinDoc {
        name: "Router",
        summary: "Client-side router outlet that renders the matched page based on current URL path.",
        example: "Router",
        subcomponents: &[],
    },
    BuiltinDoc {
        name: "Route",
        summary: "Route mapping associating a path with a page destination.",
        example: "Route(path: \"/about\", page: AboutPage)",
        subcomponents: &[],
    },
];

// ---------------------------------------------------------------------------
// Keywords documentation
// ---------------------------------------------------------------------------

const KEYWORD_DOCS: &[(&str, &str)] = &[
    (
        "Page",
        "**Page** \u{2014} Declares a route page with a URL path and metadata.\n\n```wf\nPage Home (path: \"/\", title: \"Home\") {\n  Container {\n    Heading(\"Welcome\", h1)\n  }\n}\n```",
    ),
    (
        "Component",
        "**Component** \u{2014} Declares a reusable UI component with typed props.\n\n```wf\nComponent UserCard (name: String, role: String = \"Member\") {\n  Card {\n    Heading(name, h3)\n    Badge(role, primary)\n  }\n}\n```",
    ),
    (
        "Store",
        "**Store** \u{2014} Declares a shared reactive state store with signals and actions.\n\n```wf\nStore CartStore {\n  state items = []\n  derived count = items.length\n  action add(item: Map) { items.push(item) }\n}\n```",
    ),
    (
        "Theme",
        "**Theme** \u{2014} Declares a design system theme with design tokens.\n\n```wf\nTheme Brand {\n  token color-primary: \"#0F766E\"\n  token radius-md: \"0.5rem\"\n}\n```",
    ),
    (
        "token",
        "**token** \u{2014} Declares a design token within a `Theme` block.\n\n```wf\ntoken color-primary: \"#3B82F6\"\n```",
    ),
    (
        "App",
        "**App** \u{2014} The root application shell defining global frame and routing.\n\n```wf\nApp {\n  Navbar { ... }\n  Router\n  Footer { ... }\n}\n```",
    ),
    (
        "state",
        "**state** \u{2014} Declares a reactive state signal. Mutations automatically trigger fine-grained DOM updates.\n\n```wf\nstate count = 0\nstate query = \"\"\n```",
    ),
    (
        "derived",
        "**derived** \u{2014} Declares a computed reactive signal derived from other state.\n\n```wf\nderived doubled = count * 2\n```",
    ),
    (
        "action",
        "**action** \u{2014} Declares a state mutation function with optional typed parameters.\n\n```wf\naction increment(step: Number = 1) {\n  count = count + step\n}\n```",
    ),
    (
        "effect",
        "**effect** \u{2014} Declares a side-effect block that runs on dependency change.\n\n```wf\neffect {\n  log(\"Count changed: \" + count)\n}\n```",
    ),
    (
        "use",
        "**use** \u{2014} Imports a shared `Store` into the current page or component scope.\n\n```wf\nuse CartStore\n```",
    ),
    (
        "fetch",
        "**fetch** \u{2014} Reactive data fetching block with loading, error, and success states.\n\n```wf\nfetch user from \"https://api.example.com/me\" {\n  loading { Spinner }\n  error (err) { Alert(err, danger) }\n  success { Text(user.name) }\n}\n```",
    ),
    (
        "if",
        "**if** \u{2014} Conditional rendering block. Supports `else if` and `else` branches.\n\n```wf\nif isLoggedIn {\n  Text(\"Welcome back!\")\n} else {\n  Button(\"Log In\", primary)\n}\n```",
    ),
    (
        "for",
        "**for** \u{2014} List iteration rendering element children for each item.\n\n```wf\nfor item in items {\n  Card { Text(item.title) }\n}\n```",
    ),
    (
        "show",
        "**show** \u{2014} Toggles element visibility via CSS display while keeping node mounted in the DOM.\n\n```wf\nshow isVisible {\n  Alert(\"Notice\")\n}\n```",
    ),
    (
        "navigate",
        "**navigate** \u{2014} Client-side programmatic router navigation.\n\n```wf\nnavigate \"/dashboard\"\n```",
    ),
    (
        "log",
        "**log** \u{2014} Outputs an expression to the browser developer console.\n\n```wf\nlog(user.profile)\n```",
    ),
    (
        "return",
        "**return** \u{2014} Early return from an action body.\n\n```wf\nif count <= 0 { return }\n```",
    ),
    (
        "style",
        "**style** \u{2014} Component-level style overrides and media queries.\n\n```wf\nstyle {\n  color: \"var(--color-primary)\"\n  padding: \"1rem\"\n}\n```",
    ),
    (
        "transition",
        "**transition** \u{2014} Declares animated CSS transitions on element property changes.\n\n```wf\ntransition {\n  transform 200ms ease\n}\n```",
    ),
];

// ---------------------------------------------------------------------------
// Modifier descriptions from the compiler's canonical vocabulary
// ---------------------------------------------------------------------------

fn describe_modifier(mod_name: &str) -> Option<String> {
    if !MODIFIER_KEYWORDS.contains(&mod_name) {
        return None;
    }

    let (category, desc) = match mod_name {
        // Colors
        "primary" => ("Color Variant", "Applies primary theme accent styling."),
        "secondary" => ("Color Variant", "Applies secondary neutral styling."),
        "success" => ("Color Variant", "Applies success styling (typically emerald / green)."),
        "danger" => ("Color Variant", "Applies destructive / error styling (typically crimson / red)."),
        "warning" => ("Color Variant", "Applies warning styling (typically amber / orange)."),
        "info" => ("Color Variant", "Applies informational styling (typically cyan / blue)."),

        // Sizes
        "small" => ("Size", "Compact sizing with reduced padding and typography."),
        "medium" => ("Size", "Default standard component sizing."),
        "large" => ("Size", "Expanded sizing with increased padding and typography."),

        // Widths
        "full" => ("Width", "Stretches element to 100% of parent width."),
        "fit" => ("Width", "Sizes element to fit its content width (`width: fit-content`)."),

        // Shapes & Elevations
        "rounded" => ("Shape", "Applies default border-radius curvature."),
        "pill" => ("Shape", "Applies 9999px fully rounded pill border-radius."),
        "square" => ("Shape", "Removes border-radius for sharp square edges (`radius: 0`)."),
        "outlined" => ("Elevation", "Bordered outline variant with transparent surface background."),
        "elevated" => ("Elevation", "Applies subtle box-shadow elevation above page surface."),
        "flat" => ("Elevation", "Flat surface with zero drop shadow."),

        // Typography
        "bold" => ("Typography", "Sets font-weight to bold (700)."),
        "italic" => ("Typography", "Applies italic font style."),
        "underline" => ("Typography", "Applies text underline decoration."),
        "uppercase" => ("Typography", "Transforms text to uppercase capitalization."),
        "lowercase" => ("Typography", "Transforms text to lowercase capitalization."),
        "heading" => ("Typography", "Applies heading font styling and scale."),
        "subtitle" => ("Typography", "Applies muted subtitle text scale."),
        "muted" => ("Typography", "Applies secondary muted text color (`--color-text-muted`)."),
        "left" => ("Alignment", "Aligns text to the left."),
        "center" => ("Alignment", "Centers text content horizontally."),
        "right" => ("Alignment", "Aligns text to the right."),
        "h1" => ("Heading Level", "Renders as HTML `<h1>` heading."),
        "h2" => ("Heading Level", "Renders as HTML `<h2>` heading."),
        "h3" => ("Heading Level", "Renders as HTML `<h3>` heading."),
        "h4" => ("Heading Level", "Renders as HTML `<h4>` heading."),
        "h5" => ("Heading Level", "Renders as HTML `<h5>` heading."),
        "h6" => ("Heading Level", "Renders as HTML `<h6>` heading."),

        // State & Behavior
        "disabled" => ("State", "Renders element in disabled non-interactive state with reduced opacity."),
        "loading" => ("State", "Shows loading spinner overlay and disables user interaction."),
        "error" => ("State", "Highlights input in error state with danger border color."),
        "dismissible" => ("Behavior", "Adds close icon button to dismiss alert / banner."),
        "block" => ("Display", "Renders as block-level element filling available width."),
        "bordered" => ("Border", "Adds standard container perimeter border."),

        // Media
        "controls" => ("Media", "Displays playback transport controls on audio/video elements."),
        "autoplay" => ("Media", "Automatically begins playback when media is loaded."),

        // Input types
        "text" => ("Input Type", "Standard single-line text input (`type=\"text\"`)."),
        "email" => ("Input Type", "Email address input with keyboard optimization (`type=\"email\"`)."),
        "password" => ("Input Type", "Masked password security input (`type=\"password\"`)."),
        "number" => ("Input Type", "Numeric input with stepper (`type=\"number\"`)."),
        "search" => ("Input Type", "Search field input (`type=\"search\"`)."),
        "tel" => ("Input Type", "Telephone number input (`type=\"tel\"`)."),
        "url" => ("Input Type", "URL address input (`type=\"url\"`)."),
        "date" => ("Input Type", "Date picker input (`type=\"date\"`)."),
        "time" => ("Input Type", "Time picker input (`type=\"time\"`)."),
        "datetime" => ("Input Type", "Date and time picker input (`type=\"datetime-local\"`)."),
        "color" => ("Input Type", "Color picker swatch input (`type=\"color\"`)."),
        "submit" => ("Button Type", "Form submit button (`type=\"submit\"`)."),
        "reset" => ("Button Type", "Form reset button (`type=\"reset\"`)."),

        // Animation modifiers
        "fast" => ("Animation Speed", "150ms accelerated transition duration."),
        "slow" => ("Animation Speed", "500ms relaxed transition duration."),
        "fadeIn" => ("Animation", "Fades opacity from 0% to 100%."),
        "fadeOut" => ("Animation", "Fades opacity from 100% to 0%."),
        "slideUp" => ("Animation", "Translates vertically upwards while fading in."),
        "slideDown" => ("Animation", "Translates vertically downwards while fading in."),
        "slideLeft" => ("Animation", "Translates horizontally leftwards while fading in."),
        "slideRight" => ("Animation", "Translates horizontally rightwards while fading in."),
        "scaleIn" => ("Animation", "Scales up from 95% to 100%."),
        "scaleOut" => ("Animation", "Scales down from 100% to 95%."),
        "bounce" => ("Animation", "Playful bounce entrance keyframe effect."),
        "shake" => ("Animation", "Horizontal shake keyframe animation for attention/error feedback."),
        "pulse" => ("Animation", "Subtle breathing opacity pulse effect."),
        "spin" => ("Animation", "Continuous 360 degree rotational spin animation."),

        _ => ("Modifier", "Valid WebFluent component modifier."),
    };

    Some(format!(
        "**Modifier** `{mod_name}` ({category})\n\n{desc}\n\n*Emits class*: `.wf-*--{mod_name}`"
    ))
}

// ---------------------------------------------------------------------------
// AST-based dynamic hover resolution
// ---------------------------------------------------------------------------

fn hover_ast_symbol(program: &Program, word: &str) -> Option<String> {
    for decl in &program.declarations {
        match decl {
            Declaration::Component(c) if c.name == word => {
                let props_sig = c
                    .props
                    .iter()
                    .map(|p| {
                        let ty = format!("{:?}", p.prop_type);
                        let opt = if p.optional { "?" } else { "" };
                        let def = if let Some(e) = &p.default {
                            format!(" = {:?}", e)
                        } else {
                            String::new()
                        };
                        format!("{}: {}{}{}", p.name, ty, opt, def)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                return Some(format!(
                    "**Component** `{}`\n\n```wf\nComponent {}({})\n```\n\nUser-defined reusable component.",
                    c.name, c.name, props_sig
                ));
            }
            Declaration::Page(p) if p.name == word => {
                let title_info = p
                    .title
                    .as_deref()
                    .map(|t| format!(" (title: \"{t}\")"))
                    .unwrap_or_default();
                return Some(format!(
                    "**Page** `{}`\n\nRoute: `{}`{}\n\nDeclared page route.",
                    p.name, p.path, title_info
                ));
            }
            Declaration::Store(s) if s.name == word => {
                let mut members = Vec::new();
                for stmt in &s.body {
                    match &stmt.kind {
                        StatementKind::State(st) => members.push(format!("state {}", st.name)),
                        StatementKind::Derived(d) => members.push(format!("derived {}", d.name)),
                        StatementKind::Action(a) => members.push(format!("action {}()", a.name)),
                        _ => {}
                    }
                }
                let members_doc = if members.is_empty() {
                    String::new()
                } else {
                    format!("\n\n*Members*:\n- {}", members.join("\n- "))
                };
                return Some(format!(
                    "**Store** `{}`\n\nShared reactive state store.{}",
                    s.name, members_doc
                ));
            }
            Declaration::Theme(t) if t.name == word => {
                return Some(format!(
                    "**Theme** `{}`\n\nDesign system palette defining {} custom design tokens.",
                    t.name,
                    t.tokens.len()
                ));
            }
            Declaration::Theme(t) => {
                for token in &t.tokens {
                    if token.name == word {
                        return Some(format!(
                            "**Design Token** `{}`\n\nDeclared in Theme `{}`.\n\n*Emits CSS Variable*: `var(--{})`",
                            token.name, t.name, token.name
                        ));
                    }
                }
            }
            _ => {}
        }

        // Search body statements for state, derived, actions, props
        match decl {
            Declaration::Component(c) => {
                for p in &c.props {
                    if p.name == word {
                        return Some(format!(
                            "**prop** `{}: {:?}`\n\nParameter of Component `{}`.",
                            p.name, p.prop_type, c.name
                        ));
                    }
                }
                if let Some(doc) = find_stmt_symbol(&c.body, word) {
                    return Some(doc);
                }
            }
            Declaration::Page(p) => {
                if let Some(doc) = find_stmt_symbol(&p.body, word) {
                    return Some(doc);
                }
            }
            Declaration::Store(s) => {
                if let Some(doc) = find_stmt_symbol(&s.body, word) {
                    return Some(format!("{doc}\n\nBelongs to Store `{}`.", s.name));
                }
            }
            _ => {}
        }
    }
    None
}

fn find_stmt_symbol(stmts: &[Statement], word: &str) -> Option<String> {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::State(s) if s.name == word => {
                return Some(format!(
                    "**state** `{}`\n\nReactive state signal. Reading it registers dependency; assigning updates DOM.",
                    s.name
                ));
            }
            StatementKind::Derived(d) if d.name == word => {
                return Some(format!(
                    "**derived** `{}`\n\nComputed reactive signal. Recomputes when referenced state variables change.",
                    d.name
                ));
            }
            StatementKind::Action(a) if a.name == word => {
                let params = a
                    .params
                    .iter()
                    .map(|p| format!("{}: {:?}", p.name, p.param_type))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Some(format!(
                    "**action** `{}({})`\n\nNamed action function for state mutations.",
                    a.name, params
                ));
            }
            StatementKind::For(f) if f.item == word => {
                return Some(format!("**loop variable** `{}`\n\nIterator item in `for` loop.", f.item));
            }
            StatementKind::Fetch(f) if f.variable == word => {
                return Some(format!("**fetch binding** `{}`\n\nResponse data binding from `fetch`.", f.variable));
            }
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn provide_hover(source: &str, position: Position, program: Option<&Program>) -> Option<Hover> {
    let word = word_at_position(source, position)?;

    // 1. Dynamic AST lookup if available
    if let Some(prog) = program {
        if let Some(ast_doc) = hover_ast_symbol(prog, &word) {
            return Some(make_hover(ast_doc));
        }
    }

    // 2. Built-in components
    if let Some(doc) = BUILTIN_DOCS.iter().find(|d| d.name == word) {
        let (html_tag, base_class) = builtin_to_html(doc.name);
        let mut details = format!("**Component** `<{}>` (`.{}`)\n\n{}\n\n", html_tag, base_class, doc.summary);

        if let Some(role) = implicit_role(doc.name, &[]) {
            details.push_str(&format!("*ARIA role*: `{role}`  \n"));
        }
        if let Some(landmark) = landmark_label(doc.name) {
            details.push_str(&format!("*Landmark*: `{landmark}`  \n"));
        }
        if !doc.subcomponents.is_empty() {
            details.push_str(&format!("*Subcomponents*: `{}`  \n", doc.subcomponents.join("`, `")));
        }

        details.push_str(&format!("\n```wf\n{}\n```", doc.example));
        return Some(make_hover(details));
    }

    // 3. Subcomponent references like "Navbar.Brand", "Sidebar.Item", etc.
    if word.contains('.') {
        let parts: Vec<&str> = word.split('.').collect();
        if parts.len() == 2 {
            let parent = parts[0];
            if let Some(doc) = BUILTIN_DOCS.iter().find(|d| d.name == parent) {
                if doc.subcomponents.contains(&word.as_str()) {
                    return Some(make_hover(format!(
                        "**Subcomponent** `{}`\n\nSubcomponent section of `{}`.",
                        word, parent
                    )));
                }
            }
        }
    }

    // 4. Keywords
    if let Some((_, doc)) = KEYWORD_DOCS.iter().find(|(k, _)| *k == word) {
        return Some(make_hover(doc.to_string()));
    }

    // 5. Valid modifiers
    if let Some(mod_doc) = describe_modifier(&word) {
        return Some(make_hover(mod_doc));
    }

    // 6. Events (strip "on:" prefix if present)
    let event_name = word.strip_prefix("on:").unwrap_or(&word);
    if let Some(event_doc) = describe_event(event_name) {
        return Some(make_hover(event_doc));
    }

    None
}

fn describe_event(event: &str) -> Option<String> {
    let desc = match event {
        "click" => "Fires when element is clicked.",
        "dblclick" => "Fires on double-click.",
        "input" => "Fires on every keystroke/value change in real-time.",
        "change" => "Fires when input value is committed (on enter or blur).",
        "submit" => "Fires on form submission.",
        "focus" => "Fires when element gains focus.",
        "blur" => "Fires when element loses focus.",
        "keydown" => "Fires when a keyboard key is pressed down.",
        "keyup" => "Fires when a keyboard key is released.",
        "mouseenter" => "Fires when cursor enters element bounds.",
        "mouseleave" => "Fires when cursor leaves element bounds.",
        "scroll" => "Fires when element or container is scrolled.",
        _ => return None,
    };
    Some(format!("**Event** `on:{event}`\n\n{desc}"))
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

/// Extract the word or dotted identifier under cursor position.
fn word_at_position(source: &str, position: Position) -> Option<String> {
    let line = source.lines().nth(position.line as usize)?;
    let col = position.character as usize;

    if col > line.len() {
        return None;
    }

    let bytes = line.as_bytes();

    let mut start = col;
    while start > 0 {
        let ch = bytes[start - 1] as char;
        if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == ':' {
            start -= 1;
        } else {
            break;
        }
    }

    let mut end = col;
    while end < bytes.len() {
        let ch = bytes[end] as char;
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            end += 1;
        } else {
            break;
        }
    }

    if start == end {
        return None;
    }

    let raw = &line[start..end];
    // Trim leading/trailing punctuation if not part of identifier
    let trimmed = raw.trim_matches(|c: char| c == '(' || c == ')' || c == '{' || c == '}');
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}
