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
    /// A page's or a component's body.
    Body,
    /// An action's, a handler's or an effect's body.
    Imperative,
    /// A `test "…" { }` body, beside what a page's holds.
    Test,
    /// Inside an expression, never at a statement's start.
    Expression,
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
        place: Place::Expression,
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
        place: Place::Imperative,
    },
    KeywordDoc {
        name: "return",
        summary: "Leaves an action, with a value or without.",
        example: "action total() { return items.length }",
        place: Place::Imperative,
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
        place: Place::Imperative,
    },
    KeywordDoc {
        name: "null",
        summary: "No value. `a ?? b` takes `b` when `a` is null; `if let x = a { }` renders when it is not.",
        example: "state selected = null\nText(selected ?? \"none\")",
        place: Place::Expression,
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
    KeywordDoc {
        name: "animation",
        summary: "Keyframes, declared once: play them with `animate: .Name` or `exit: .Name` on any element, or write `animation: Name 1s infinite` in a style.",
        example: "animation Wobble {\n    from { transform: rotate(-2deg) }\n    to { transform: rotate(2deg) }\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "const",
        summary: "A value every page, component and store reads by name, fixed when the program is written. It may declare its type: `const PAGE_SIZE: Number = 20`.",
        example: "const API = \"/api/v1\"",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "data",
        summary: "A constant whose value is a JSON file's, read at build time from the project or its `src/` and inlined into the bundle. A `:param` page names the values it renders for with `paths:`.",
        example: "data posts: [Post] = \"content/posts.json\"",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "image",
        summary: "A picture the build reads and writes again at every width a page asks for, with its real size and colour. `Image(hero, alt: …)` draws it as a `<picture>`; `hero.src`, `.width`, `.height`, `.color`, `.srcset` read it.",
        example: "image hero = \"media/hero.jpg\"",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "api",
        summary: "A service, described once: its address, how it is reached (`timeout`, `retry`, `credentials`), hooks on each request and response, and its endpoints. Every call is typed, cached, deduplicated and cancellable. `api B from \"openapi.json\"` reads it from a specification.",
        example: "api Backend(base: \"/api/v1\") {\n    timeout: 10.seconds\n    get users(page: Number = 1) -> [User]\n    get user(id: String) at \"users/:id\" -> User\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "test",
        summary: "A test `wf test` runs: what it draws, and what the page must show. A step that acts — `click`, `type`, `press` — runs it in a headless Chrome; the rest is compared with a snapshot.",
        example: "test \"adds a row\" {\n    use Rows\n    Button(\"Add\") { on click { Rows.add() } }\n    click \"Add\"\n    expect \"1 row\"\n}",
        place: Place::TopLevel,
    },
    KeywordDoc {
        name: "validate",
        summary: "What the state it names must be. The control bound to that state shows what the rules say, with the accessible plumbing. Rules: `required`, `email`, `url`, `minLength(n)`, `maxLength(n)`, `min(v)`, `max(v)`, `pattern(/…/)`, `matches(other)`, `oneOf([…])`, `custom \"…\" { }`, `async \"…\" { }`.",
        example: "validate email {\n    required\n    email \"That is not an address\"\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "socket",
        summary: "A WebSocket the page holds open: typed both ways, reconnecting with backoff, kept alive by a heartbeat, and closed when the page leaves. `match` reads its state; `.send(v)`, `.messages`, `.last(kind)`, `.error`, `.closure`, `.close()`.",
        example: "socket chat = ws(\"wss://example.com/chat\") {\n    on message(m) { Feed.add(m) }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "stream",
        summary: "A stream of server-sent events, resumed where it was after a drop and closed when the page leaves. `.last(\"price\")` is the latest of one kind; `.messages` all of them.",
        example: "stream ticks = sse(\"/events\", events: [\"price\"])",
        place: Place::Body,
    },
    KeywordDoc {
        name: "channel",
        summary: "A message every tab of the site hears (`BroadcastChannel`). `.post(v)` sends one; `on message(m)` hears the others'.",
        example: "channel cart = broadcast(\"cart\") {\n    on message(m) { Cart.merge(m) }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "peer",
        summary: "A WebRTC data channel straight to another reader's page. What the two sides must tell each other goes out through `signal:`; what the other side sent is handed to `link.signal(m)`. `.send(v)` once it is `open`.",
        example: "peer link = rtc(signal: m => lobby.post(m), initiator: query.host == \"1\") {\n    on message(m) { heard = m.text }\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "every",
        summary: "A timer: the block runs every so many milliseconds, and stops when its page, branch or list item leaves.",
        example: "every(1000) { tick = tick + 1 }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "after",
        summary: "The block runs once, so many milliseconds from now — unless its page, branch or list item leaves first.",
        example: "after(3000) { toast = null }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "head",
        summary: "Tags this page adds to the document's head — painted at build time where the value is known, kept current on the live page, gone when the page is.",
        example: "head {\n    meta(property: \"og:image\", content: post.image)\n    link(rel: \"canonical\", href: \"https://example.com/p/{slug}\")\n}",
        place: Place::Body,
    },
    KeywordDoc {
        name: "part",
        summary: "A component of a component's own, called under its name: `Panel.Header(…)`.",
        example: "part Header(_ text: String) { Heading(text).h3 }",
        place: Place::Body,
    },
    KeywordDoc {
        name: "try",
        summary: "Run the block; if anything in it throws, run the `catch` block with what was thrown.",
        example: "try {\n    let r = await fetch(\"/api/sync\")\n} catch e {\n    error = e.message\n}",
        place: Place::Imperative,
    },
    KeywordDoc {
        name: "expect",
        summary: "What the page under test must show — or, with `not`, must not.",
        example: "expect \"3 open\"\nexpect not \"Error\"",
        place: Place::Test,
    },
    KeywordDoc {
        name: "click",
        summary: "Click whatever carries that name: a button's text, a control's label or `aria-label`. Runs the test in a headless Chrome.",
        example: "click \"Save\"",
        place: Place::Test,
    },
    KeywordDoc {
        name: "type",
        summary: "Type text into the control a label, placeholder or `aria-label` names (a `Select` takes an option's text).",
        example: "type \"Ada\" into \"Name\"",
        place: Place::Test,
    },
    KeywordDoc {
        name: "press",
        summary: "Press a key on the focused element, or on the one named after `in`. `Escape` closes an open `Modal` or `Dialog`.",
        example: "press \"Enter\"\npress \"Escape\" in \"Search\"",
        place: Place::Test,
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

/// What each function the language gives a program does, and each value a
/// page reads from the browser — for completion and hover. A test holds it
/// to the compiler's own lists.
pub const BUILTINS: &[(&str, &str, &str)] = &[
    (
        "log",
        "log(value)",
        "Writes a value to the browser's console.",
    ),
    (
        "navigate",
        "navigate(\"/path\")",
        "Goes to another route of the site, without a page load.",
    ),
    (
        "format",
        "format(value, .style, option)",
        "A number, money, a date or a time as the page's locale shows it: `.number`, `.integer`, `.decimal`, `.currency`, `.percent`, `.compact`, `.date`, `.time`, `.datetime`, `.relative`.",
    ),
    (
        "ago",
        "ago(when)",
        "How long ago or from now, in words: `3 minutes ago`, `yesterday`, `in 2 weeks`.",
    ),
    (
        "t",
        "t(\"key\", { name: value })",
        "A message from the project's translations, in the current locale; `count` picks a plural form.",
    ),
    (
        "setLocale",
        "setLocale(\"ar\")",
        "Switches the locale; an RTL one (`ar`, `he`, `fa`, `ur`) turns the page right to left.",
    ),
    (
        "setTheme",
        "setTheme(\"dark\")",
        "Chooses `\"dark\"`, `\"light\"` or `\"system\"`, kept across visits; `theme` reads the choice.",
    ),
    ("uuid", "uuid()", "A new random identifier, a `Uuid`."),
    (
        "sanitize",
        "sanitize(html)",
        "The allow-listed part of some markup — no script, style, frame, form or `on*` — for `Unsafe.Html`.",
    ),
    (
        "fetch",
        "fetch(url, method: \"POST\", body: …)",
        "A request through the language's engine — timeouts, retries, the cache — in a `resource` or after `await`.",
    ),
    (
        "optimistic",
        "optimistic(holder, change)",
        "Shows a change at once; if the action throws later, it is taken back.",
    ),
    (
        "beacon",
        "beacon(\"/analytics\", data)",
        "Sends data that survives the page being closed.",
    ),
    (
        "animate",
        "animate(target, \"pulse\", \"400ms\")",
        "Plays an animation on an element handle, and gives a handle back to play or cancel it.",
    ),
    (
        "replayAnimation",
        "replayAnimation(element, \"fadeIn\")",
        "Plays an element's animation again.",
    ),
    (
        "every",
        "every(ms) { … }",
        "A timer, stopped when its page, branch or item leaves.",
    ),
    (
        "after",
        "after(ms) { … }",
        "Runs once after a delay, unless its page, branch or item leaves first.",
    ),
    (
        "now",
        "now",
        "The current moment, a `DateTime` that keeps itself current — every minute, or as often as `now(every: 1.seconds)` asks.",
    ),
    (
        "ws",
        "ws(\"wss://…\", protocols: […], heartbeat: 20.seconds)",
        "Opens a WebSocket, for a `socket`.",
    ),
    (
        "sse",
        "sse(\"/events\", events: [\"price\"])",
        "Opens a stream of server-sent events, for a `stream`.",
    ),
    (
        "broadcast",
        "broadcast(\"name\")",
        "Joins a channel every tab of the site hears, for a `channel`.",
    ),
    (
        "rtc",
        "rtc(signal: m => …, initiator: true, ice: […])",
        "Opens a WebRTC data channel to another page, for a `peer`.",
    ),
    ("String", "String(value)", "A value as text."),
    ("Number", "Number(value)", "A value as a number."),
    ("Boolean", "Boolean(value)", "A value as `true` or `false`."),
    ("Bool", "Bool(value)", "A value as `true` or `false`."),
    (
        "Money",
        "Money(amount: 1299, currency: \"EUR\")",
        "An amount of money, in minor units.",
    ),
    (
        "viewport",
        "viewport.md",
        "The window's size, kept current: `.width`, `.height`, and `.sm` … `.xl`, whether it is at least that breakpoint.",
    ),
    (
        "query",
        "query.tab",
        "The address's query string, by name: `?tab=open` is `query.tab`.",
    ),
    ("hash", "hash", "The address's `#fragment`, kept current."),
    (
        "theme",
        "theme",
        "The reader's colour scheme choice: `\"light\"`, `\"dark\"` or `\"system\"`.",
    ),
    (
        "network",
        "network.online",
        "The connection: `.online`, `.effectiveType`, `.saveData`, `.downlink`, and `.queued` — the writes waiting for it.",
    ),
    (
        "update",
        "update.available",
        "A new version of an offline site: `.available`, and `.apply()` to take it and reload once.",
    ),
];

/// The entry for a built-in function or browser value.
pub fn builtin(name: &str) -> Option<&'static (&'static str, &'static str, &'static str)> {
    BUILTINS.iter().find(|(n, _, _)| *n == name)
}

/// What an `api` block's settings say, each with what it does.
pub const API_SETTINGS: &[(&str, &str, &str)] = &[
    (
        "timeout",
        "timeout: ${1:10}.seconds",
        "How long a request may take before it ends as `.timeout`.",
    ),
    (
        "retry",
        "retry: .backoff(times: ${1:3}, on: [.network, .timeout, .status5xx])",
        "When to try again: exponential backoff with jitter, per error kind, honouring `Retry-After`.",
    ),
    (
        "credentials",
        "credentials: .${1:sameOrigin}",
        "Whether the session cookie is sent: `.omit`, `.sameOrigin`, `.include` — where a session belongs.",
    ),
    (
        "cache",
        "cache: .swr(${1:60}.seconds)",
        "How long an answer stands: `.swr(…)`, `.none`, `.forever`.",
    ),
    (
        "mode",
        "mode: .${1:cors}",
        "The request's mode, as `fetch` names it.",
    ),
    (
        "redirect",
        "redirect: .${1:follow}",
        "What a redirect does, as `fetch` names it.",
    ),
    (
        "referrer",
        "referrer: .${1:strictOriginWhenCrossOrigin}",
        "The referrer policy, as `fetch` names it.",
    ),
    (
        "headers",
        "headers {\n\t\"${1:X-Api-Key}\": ${2:value}\n}",
        "Headers every request carries, each read at the moment of the request.",
    ),
];

/// What a `persist` value's block may say, each with what it does.
pub const PERSIST_KEYS: &[(&str, &str, &str)] = &[
    (
        "in",
        "in: .${1:session}",
        "Where it is kept: `.local` (the default, across visits) or `.session` (this tab).",
    ),
    (
        "version",
        "version: ${1:2}",
        "The version of the shape this build writes; an older value is brought forward by `migrate`.",
    ),
    (
        "sync",
        "sync: ${1:false}",
        "Whether a write in another tab is taken here (on by default for `.local`).",
    ),
    (
        "key",
        "key: ${1:id}",
        "What tells this instance's value from another's: one stored value per instance of a component placed more than once.",
    ),
    (
        "migrate",
        "migrate ${1:1} -> ${2:2} { ${0:old} }",
        "One step forward: what a value of the older version becomes, read as `old`.",
    ),
];
