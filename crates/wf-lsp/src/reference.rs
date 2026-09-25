//! The language reference the server shows beside what the registry knows:
//! the keywords with an example each, the page attributes, the nested
//! selectors a style block takes, the baseline design tokens, the icons and
//! the CSS properties worth offering. Every built-in component, its props,
//! flags, cases, parts and events come from `webfluent::registry`, the one
//! table the compiler reads too.

/// A named argument with its documentation.
pub struct ArgDoc {
    pub name: &'static str,
    pub doc: &'static str,
}

const fn arg(name: &'static str, doc: &'static str) -> ArgDoc {
    ArgDoc { name, doc }
}

/// The attributes a `page` header takes; anything else is a route parameter.
pub const PAGE_ATTRIBUTES: &[ArgDoc] = &[
    arg(
        "path",
        "URL route. `/user/:id` captures `id`, which the header declares: `id: String`",
    ),
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
        "`noindex: true` keeps the page out of search results and the sitemap",
    ),
    arg(
        "layout",
        "The component that frames the page: `layout: Shell(crumb: \"Home\")`; the page renders in its default slot",
    ),
    arg("guard", "Expression that must hold for the route to render"),
    arg("redirect", "Where to send the visitor when the guard fails"),
];

/// The options `fetch(url, …)` takes in a `resource`.
pub const RESOURCE_OPTIONS: &[ArgDoc] = &[
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
    Style,
}

pub const KEYWORDS: &[KeywordDoc] = &[
    KeywordDoc {
        name: "page",
        summary: "A routed page. Every page declares its own `path`; the `Router` in `app` shows the one that matches.",
        example: "page Home(path: \"/\", title: \"Home\") {\n    Container { Heading(\"Welcome\").h1 }\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "component",
        summary: "A reusable component with typed props: `String`, `Number`, `Bool`, `[T]`, `Map`, `T?`, or an enum or type you declare; `= value` gives a default and `_ name` marks the one positional prop. It may declare `event`s and `slot`s.",
        example: "component UserCard(_ name: String, active: Bool = true) {\n    Card { Text(name).bold }\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "store",
        summary: "Shared reactive state: `state`, `derived` and `action`s, reached as `Name.member` after `use Name`. It is built the first time something reads it. `store Cart(scope: .route)` drops what it holds when the route changes; `.session` is the tab's; `eager: true` builds it at boot.",
        example: "store CartStore(scope: .app) {\n    state items = []\n    derived count = items.length\n    action clear() { items = [] }\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "persist",
        summary: "State kept in the browser's storage across visits. Its block says where (`in: .local` or `.session`), the version of its shape, whether another tab's write is adopted (`sync:`), and how a value an older build wrote is brought forward.",
        example: "persist items: [Item] = [] {\n    in: .local\n    version: 2\n    migrate 1 -> 2 { old.map(i => Item(id: i.id, qty: i.count)) }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "migrate",
        summary: "One step forward for a persisted value: what a value of the older version becomes, reading it as `old`. Each step moves one version on, so every value can be brought forward.",
        example: "migrate 1 -> 2 { old.map(i => Item(id: i.id, qty: i.count)) }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "theme",
        summary: "Design tokens: `name: value` lines, raw CSS values. Each is `$name` in any style block and `var(--name)` in a stylesheet.",
        example: "theme Brand {\n    color-primary: #6366F1\n    radius-md: 10px\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "app",
        summary: "The root of the site: what wraps every page, and a bare `Router` where the current page renders.",
        example: "app {\n    Navbar(brand: \"Ledger\")\n    Router\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "type",
        summary: "A record type: named, typed fields with optional defaults. Construct one with named arguments.",
        example: "type Todo { id: String, title: String, done: Bool = false }",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "enum",
        summary: "A closed set of cases. A value is written `.case`; a prop typed with the enum takes `.case` by name, or the case as a flag.",
        example: "enum Tone { calm, loud }",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "state",
        summary: "A reactive variable. Everything that reads it updates when it changes; an optional type: `state draft: String = \"\"`.",
        example: "state count = 0",
        place: Place::Body,
    },
    KeywordDoc {
        name: "derived",
        summary: "A value computed from state, recomputed when what it reads changes.",
        example: "derived total = items.map(i => i.price).sum()",
        place: Place::Body,
    },
    KeywordDoc {
        name: "effect",
        summary: "Runs when the page mounts and again whenever the state it reads changes.",
        example: "effect { log(count) }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "action",
        summary: "A function that changes state, called from handlers and other actions. `await` inside makes it async.",
        example: "action add(title: String) {\n    items = items.concat([title])\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "use",
        summary: "Brings a store into scope: `use CartStore`, then `CartStore.items`.",
        example: "use CartStore",
        place: Place::Body,
    },
    KeywordDoc {
        name: "resource",
        summary: "An async value: `resource rows = fetch(url, method: \"POST\")`. Render its states with `match`; the request repeats when a URL that reads state changes.",
        example: "resource users = fetch(\"/api/users\")\nmatch users {\n    loading { Spinner }\n    error(e) { Alert(e.message).danger }\n    ready(list) { for u in list by u.id { Text(u.name) } }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "match",
        summary: "Renders one arm by the value: a resource's `loading`, `error(e)`, `ready(v)`, or an enum's `.case`, with `else` as the rest. As an expression, `match t { .a { 1 } else { 2 } }`.",
        example: "match users {\n    loading { Spinner }\n    error(e) { Text(e.message) }\n    ready(list) { Text(\"{list.length} users\") }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "if",
        summary: "Renders the block while the condition holds; `else if` and `else` follow. `if let name = value { }` binds a value that is not `null`.",
        example: "if open {\n    Text(\"Open\")\n} else {\n    Text(\"Closed\")\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "else",
        summary: "The block rendered when the `if` does not hold.",
        example: "if open { Text(\"Open\") } else { Text(\"Closed\") }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "for",
        summary: "Renders the block once per item. `by key` gives each item an identity, so the list moves and keeps items rather than rebuilding them; `for item, i in items` also binds the index.",
        example: "for todo in todos by todo.id {\n    TodoRow(todo.title)\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "by",
        summary: "The key of a `for` item: `for t in todos by t.id`. An item keeps its nodes while it is the same value under the same key.",
        example: "for t in todos by t.id { Text(t.title) }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "show",
        summary: "Keeps the block in the page and shows or hides it, where `if` adds and removes it.",
        example: "show expanded {\n    Card { Text(\"Details\") }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "sequence",
        summary: "Starts a group of elements on a clock. `after:` is measured from the step before it, and is written onto the step's elements as a `delay:`.",
        example: "sequence {\n    step { Heading(\"Welcome\").h1.fadeIn }\n    step(after: \"120ms\") { Text(\"What we do.\").slideUp }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "let",
        summary: "A local of an action or handler.",
        example: "action save() {\n    let payload = { title: draft }\n    Todos.add(payload)\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "return",
        summary: "Leaves an action, with a value or without.",
        example: "action total() { return items.length }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "emit",
        summary: "Fires one of the component's declared events with its arguments; the caller handles it with `on name(args) { }`.",
        example: "event toggle(id: String)\nButton(\"x\") { on click { emit toggle(todo.id) } }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "event",
        summary: "Declares an event the component fires with `emit`, and the arguments it carries. Declared before the elements.",
        example: "event toggle(id: String)",
        place: Place::Body,
    },
    KeywordDoc {
        name: "slot",
        summary: "Declares a named slot the caller fills with `name { … }`; a bare `slot` names the default one, placed with `children`. Declared before the elements.",
        example: "slot trailing\nRow { children  trailing }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "children",
        summary: "Where the caller's block renders inside a component; a named slot renders where its name stands.",
        example: "component Panel(_ title: String) {\n    Card { Heading(title).h3  children }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "on",
        summary: "An event handler on an element: `on click { }`; `on click(event) { }` names the DOM event. On a component, a handler for an event it declares receives its arguments: `on toggle(id) { }`.",
        example: "Button(\"Save\") {\n    on click { save() }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "navigate",
        summary: "Goes to a route.",
        example: "navigate(\"/login\")",
        place: Place::Body,
    },
    KeywordDoc {
        name: "log",
        summary: "Writes to the browser console.",
        example: "log(count)",
        place: Place::Body,
    },
    KeywordDoc {
        name: "await",
        summary: "Waits for a promise inside an action or handler, which becomes async: `let r = await fetch(url)`.",
        example: "action load() {\n    let r = await fetch(\"/api\")\n    items = r.items\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "null",
        summary: "No value. `a ?? b` takes `b` when `a` is null; `if let x = a { }` renders when it is not.",
        example: "state selected = null\nText(selected ?? \"none\")",
        place: Place::Body,
    },
    KeywordDoc {
        name: "style",
        summary: "CSS for this element: `property: value` lines in raw CSS, `$token` for a design token, `{expr}` to read state. A literal value compiles to a stylesheet rule; one that reads state is set on the element. Nested rules take CSS nesting: `&:hover { }`, `@media (…) { }`.",
        example: "style {\n    padding: 6px 0\n    background: $surface\n    width: {pct}%\n    &:hover { background: $surface-hover }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "transition",
        summary: "Which properties animate between states, and how: `property: duration easing`.",
        example: "transition {\n    background: 200ms ease\n    transform: 150ms spring\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "@media",
        summary: "A rule that applies within a media query; its values must be literals or tokens.",
        example: "@media (max-width: 768px) {\n    padding: $sm\n}",
        place: Place::Style,
    },
];

pub fn keyword(name: &str) -> Option<&'static KeywordDoc> {
    KEYWORDS.iter().find(|k| k.name == name)
}

/// The nested selectors a style block takes, and what each one matches.
pub const SELECTORS: &[(&str, &str)] = &[
    ("&:hover", "While the pointer is over the element"),
    (
        "&:focus-visible",
        "While the element has keyboard focus: the focus ring, not a ring on every click",
    ),
    ("&:active", "While the element is pressed"),
    ("&:disabled", "While the control is disabled"),
    (
        "&:focus-within",
        "While the element or something inside it has focus",
    ),
    ("&::placeholder", "The placeholder text of an input"),
    (
        "&[aria-current=\"page\"]",
        "The link to the current route, which the router marks",
    ),
    ("&[aria-pressed=\"true\"]", "A toggle button that is on"),
    ("&[aria-selected=\"true\"]", "The selected tab or option"),
    ("&[aria-checked=\"true\"]", "A checked control"),
    ("&[aria-expanded=\"true\"]", "An opened disclosure"),
    ("&[aria-invalid=\"true\"]", "A field with an error"),
];

/// The DOM events any element takes in an `on` handler.
pub fn events() -> &'static [(&'static str, &'static str)] {
    webfluent::registry::UNIVERSAL_EVENTS
}

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

/// The built-in icon names `Icon(…)`, `icon:` and `Sidebar.Item(icon:)` take:
/// the registry's, which is pinned to the runtime's table.
pub const ICONS: &[&str] = webfluent::registry::ICONS;

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
