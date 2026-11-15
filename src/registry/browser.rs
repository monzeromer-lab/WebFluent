//! What the browser and the runtime bring: the names a program may use
//! unprefixed, the live values, what the language's own types can do, the
//! gestures and drag props, and the attributes a component's root takes.

/// The events a runtime recogniser turns the pointer into (`gesture.js`),
/// and `visible`, which it hears from an `IntersectionObserver`.
pub const GESTURE_EVENTS: &[&str] = &["longpress", "swipe", "receive", "dragmove", "visible"];

/// The props of a drag and of what takes a drop: the runtime's, not
/// attributes the element carries.
pub const DRAG_PROPS: &[&str] = &["drag", "dragGroup", "dropValue", "dropAxis", "dropInside"];

/// The names the language reads from the browser, each a live value:
/// `viewport.md`, `query.tab`, `now`, `network.online`, `update.available`.
/// The browser's own names a program may use unprefixed: each compiles to
/// itself. The type checker reads the same list, so a name here is never
/// reported as declared by nothing.
pub const BROWSER_GLOBALS: &[&str] = &[
    "window",
    "document",
    "console",
    "localStorage",
    "sessionStorage",
    "JSON",
    "Math",
    "Date",
    "setTimeout",
    "setInterval",
    "clearTimeout",
    "clearInterval",
    "parseInt",
    "parseFloat",
    "Array",
    "Object",
    "String",
    "Number",
    "Boolean",
    "Promise",
    "Error",
    "Map",
    "Set",
    "RegExp",
    "Infinity",
    "NaN",
    "undefined",
    "encodeURIComponent",
    "decodeURIComponent",
    "encodeURI",
    "decodeURI",
    "atob",
    "btoa",
    "fetch",
    "alert",
    "confirm",
    "prompt",
    "requestAnimationFrame",
    "cancelAnimationFrame",
    "navigator",
    "location",
    "history",
    "screen",
    "performance",
    "crypto",
    "globalThis",
    "Intl",
    "Symbol",
    "Reflect",
    "URL",
    "URLSearchParams",
    "FormData",
    "Blob",
    "File",
    "FileReader",
    "Event",
    "CustomEvent",
    "AbortController",
    "TextEncoder",
    "TextDecoder",
    "Notification",
    "matchMedia",
    "getComputedStyle",
    "structuredClone",
    "queueMicrotask",
];

pub const BROWSER_VALUES: &[&str] = &[
    "viewport", "query", "hash", "theme", "now", "network", "update",
];

/// What the language's own types can do — a date's arithmetic, money's, a
/// URL's parts, a colour's mixing — each as the runtime function that does
/// it. Each is a plain JSON value at run time, so the work is always the
/// runtime's, never a method on the value. A list rather than a `match`, so
/// `tests/build_and_runtime_agree.rs` can hold every one of them to the
/// runtime and to the static paint.
pub const SCALAR_METHODS: &[(&str, &str)] = &[
    ("year", "year"),
    ("month", "month"),
    ("day", "day"),
    ("weekday", "weekday"),
    ("hour", "hour"),
    ("minute", "minute"),
    ("second", "second"),
    ("native", "native"),
    ("plus", "plus"),
    ("minus", "minus"),
    ("isBefore", "isBefore"),
    ("isAfter", "isAfter"),
    ("isSame", "isSame"),
    ("until", "until"),
    ("startOfDay", "startOfDay"),
    ("startOfWeek", "startOfWeek"),
    ("startOfMonth", "startOfMonth"),
    ("endOfDay", "endOfDay"),
    ("inZone", "inZone"),
    ("days", "days"),
    ("hours", "hours"),
    ("minutes", "minutes"),
    ("seconds", "seconds"),
    ("ms", "ms"),
    ("times", "times"),
    ("convert", "convert"),
    ("host", "host"),
    ("path", "path"),
    ("domain", "domain"),
    ("mix", "mix"),
    ("lighten", "lighten"),
    ("darken", "darken"),
    ("alpha", "alpha"),
    ("contrast", "contrast"),
    ("preview", "preview"),
    // Named apart from the browser's own `query`, `date` and `time`.
    ("date", "dateOf"),
    ("time", "timeOf"),
    ("query", "urlQuery"),
    ("with", "urlWith"),
];

/// An attribute a component call hands its root element when the component
/// takes no prop of that name: the ARIA and data families (not the motion
/// markers, which `WF.mark` sets) and the global attributes.
pub(crate) fn is_root_attribute(name: &str) -> bool {
    (name.starts_with("aria-") || (name.starts_with("data-") && !name.starts_with("data-wf-")))
        || matches!(
            name,
            "id" | "role" | "tabindex" | "title" | "hidden" | "lang" | "dir"
        )
}

/// The browser, typed: what each global a program reads unprefixed is, and
/// the members of every object the browser hands it — an element, an event,
/// storage, the clipboard. The checker, the editor and `wf types` read it;
/// a global or a member it does not list is `T30`, and reached from a
/// project script with JSDoc instead.
///
/// One declaration a line: `let name: Type` is a value, `fn name(…) -> R`
/// a function, `object Name { … }` (or `object Name: Base { … }`, which
/// has every member of `Base` too) an object's fields and methods, `event
/// a, b: Type` what a DOM event's handler is given. A parameter written
/// `name?: T` may be left out, `...name: T` takes any number, and
/// `...name` alone any value but a secret. An `async` method hands back a
/// promise of its result. Types are written as a program writes them.
pub(crate) const BROWSER_TYPES: &str = r"
let window: Window
let document: Document
let console: Console
let localStorage: Storage
let sessionStorage: Storage
let JSON: JSON
let Math: Math
let Date: DateConstructor
let Object: ObjectConstructor
let Array: ArrayConstructor
let Number: NumberConstructor
let navigator: Navigator
let location: Location
let history: History
let performance: Performance
let screen: Screen
let Infinity: Number
let NaN: Number
let undefined: ()

fn setTimeout(run: () -> (), ms?: Number) -> Number
fn setInterval(run: () -> (), ms?: Number) -> Number
fn clearTimeout(id: Number?) -> ()
fn clearInterval(id: Number?) -> ()
fn requestAnimationFrame(run: (Number) -> ()) -> Number
fn cancelAnimationFrame(id: Number?) -> ()
fn queueMicrotask(run: () -> ()) -> ()
fn parseInt(text: String, radix?: Number) -> Number
fn parseFloat(text: String) -> Number
fn isNaN(value: Number) -> Bool
fn isFinite(value: Number) -> Bool
fn encodeURIComponent(text: String) -> String
fn decodeURIComponent(text: String) -> String
fn encodeURI(text: String) -> String
fn decodeURI(text: String) -> String
fn atob(text: String) -> String
fn btoa(text: String) -> String
fn alert(message: String) -> ()
fn confirm(message: String) -> Bool
fn prompt(message: String, value?: String) -> String?
fn matchMedia(query: String) -> MediaQueryList
fn getComputedStyle(element: Element) -> Style
fn structuredClone(value: Json) -> Json

object Window {
    innerWidth: Number
    innerHeight: Number
    outerWidth: Number
    outerHeight: Number
    scrollX: Number
    scrollY: Number
    devicePixelRatio: Number
    isSecureContext: Bool
    origin: String
    document: Document
    location: Location
    history: History
    navigator: Navigator
    localStorage: Storage
    sessionStorage: Storage
    parent: Window
    top: Window?
    open(url: String, target?: String, features?: String) -> ()
    close() -> ()
    print() -> ()
    focus() -> ()
    scrollTo(x: Number, y: Number) -> ()
    scrollBy(x: Number, y: Number) -> ()
    postMessage(message: Json, targetOrigin: String) -> ()
    addEventListener(type: String, listener: (Event) -> (), options?: Json) -> ()
    removeEventListener(type: String, listener: (Event) -> (), options?: Json) -> ()
    dispatchEvent(event: Event) -> Bool
    matchMedia(query: String) -> MediaQueryList
    getSelection() -> Selection?
}

object Document {
    title: String
    lang: String
    dir: String
    readyState: String
    visibilityState: String
    hidden: Bool
    referrer: String
    cookie: String
    documentElement: Element
    body: Element
    head: Element
    activeElement: Element?
    fullscreenElement: Element?
    querySelector(selector: String) -> Element?
    querySelectorAll(selector: String) -> [Element]
    getElementById(id: String) -> Element?
    createElement(tag: String) -> Element
    addEventListener(type: String, listener: (Event) -> (), options?: Json) -> ()
    removeEventListener(type: String, listener: (Event) -> (), options?: Json) -> ()
    dispatchEvent(event: Event) -> Bool
    hasFocus() -> Bool
    getSelection() -> Selection?
    async exitFullscreen() -> ()
}

object Selection {
    rangeCount: Number
    isCollapsed: Bool
    toString() -> String
    removeAllRanges() -> ()
}

object Element {
    id: String
    tagName: String
    className: String
    value: String
    checked: Bool
    disabled: Bool
    hidden: Bool
    open: Bool
    files: [File]?
    textContent: String?
    innerText: String
    title: String
    lang: String
    dir: String
    tabIndex: Number
    dataset: [String: String]
    style: Style
    classList: ClassList
    parentElement: Element?
    children: [Element]
    firstElementChild: Element?
    lastElementChild: Element?
    nextElementSibling: Element?
    previousElementSibling: Element?
    scrollTop: Number
    scrollLeft: Number
    scrollWidth: Number
    scrollHeight: Number
    clientWidth: Number
    clientHeight: Number
    offsetWidth: Number
    offsetHeight: Number
    offsetTop: Number
    offsetLeft: Number
    selectionStart: Number?
    selectionEnd: Number?
    currentTime: Number
    duration: Number
    paused: Bool
    muted: Bool
    volume: Number
    focus(options?: Json) -> ()
    blur() -> ()
    click() -> ()
    select() -> ()
    setSelectionRange(start: Number, end: Number) -> ()
    scrollIntoView(options?: Json) -> ()
    scrollTo(x: Number, y: Number) -> ()
    scrollBy(x: Number, y: Number) -> ()
    getBoundingClientRect() -> Rect
    contains(other: Element?) -> Bool
    closest(selector: String) -> Element?
    matches(selector: String) -> Bool
    querySelector(selector: String) -> Element?
    querySelectorAll(selector: String) -> [Element]
    getAttribute(name: String) -> String?
    setAttribute(name: String, value: String) -> ()
    removeAttribute(name: String) -> ()
    hasAttribute(name: String) -> Bool
    toggleAttribute(name: String, force?: Bool) -> Bool
    addEventListener(type: String, listener: (Event) -> (), options?: Json) -> ()
    removeEventListener(type: String, listener: (Event) -> (), options?: Json) -> ()
    dispatchEvent(event: Event) -> Bool
    remove() -> ()
    showModal() -> ()
    show() -> ()
    close() -> ()
    showPopover() -> ()
    hidePopover() -> ()
    togglePopover() -> Bool
    reset() -> ()
    submit() -> ()
    requestSubmit() -> ()
    checkValidity() -> Bool
    reportValidity() -> Bool
    async play() -> ()
    pause() -> ()
    async requestFullscreen() -> ()
}

object ClassList {
    length: Number
    add(...names: String) -> ()
    remove(...names: String) -> ()
    toggle(name: String, force?: Bool) -> Bool
    contains(name: String) -> Bool
}

object Style {
    getPropertyValue(name: String) -> String
    setProperty(name: String, value: String) -> ()
    removeProperty(name: String) -> String
}

object Rect {
    x: Number
    y: Number
    width: Number
    height: Number
    top: Number
    left: Number
    right: Number
    bottom: Number
}

object Event {
    type: String
    target: Element
    currentTarget: Element
    defaultPrevented: Bool
    isTrusted: Bool
    timeStamp: Number
    preventDefault() -> ()
    stopPropagation() -> ()
    stopImmediatePropagation() -> ()
}

object MouseEvent: Event {
    clientX: Number
    clientY: Number
    pageX: Number
    pageY: Number
    offsetX: Number
    offsetY: Number
    screenX: Number
    screenY: Number
    movementX: Number
    movementY: Number
    button: Number
    buttons: Number
    detail: Number
    shiftKey: Bool
    ctrlKey: Bool
    altKey: Bool
    metaKey: Bool
    relatedTarget: Element?
}

object PointerEvent: MouseEvent {
    pointerId: Number
    pointerType: String
    pressure: Number
    isPrimary: Bool
}

object WheelEvent: MouseEvent {
    deltaX: Number
    deltaY: Number
    deltaZ: Number
    deltaMode: Number
}

object DragEvent: MouseEvent {
    dataTransfer: DataTransfer?
}

object KeyboardEvent: Event {
    key: String
    code: String
    location: Number
    repeat: Bool
    isComposing: Bool
    shiftKey: Bool
    ctrlKey: Bool
    altKey: Bool
    metaKey: Bool
    getModifierState(key: String) -> Bool
}

object InputEvent: Event {
    data: String?
    inputType: String
    isComposing: Bool
}

object SubmitEvent: Event {
    submitter: Element?
}

object FocusEvent: Event {
    relatedTarget: Element?
}

object ClipboardEvent: Event {
    clipboardData: DataTransfer?
}

object TouchEvent: Event {
    touches: [Touch]
    changedTouches: [Touch]
    shiftKey: Bool
    ctrlKey: Bool
    altKey: Bool
    metaKey: Bool
}

object Touch {
    identifier: Number
    clientX: Number
    clientY: Number
    pageX: Number
    pageY: Number
    screenX: Number
    screenY: Number
    target: Element
}

object AnimationEvent: Event {
    animationName: String
    elapsedTime: Number
}

object TransitionEvent: Event {
    propertyName: String
    elapsedTime: Number
}

object MessageEvent: Event {
    data: Json
    origin: String
}

object DataTransfer {
    dropEffect: String
    effectAllowed: String
    files: [File]
    types: [String]
    getData(format: String) -> String
    setData(format: String, data: String) -> ()
    clearData(format?: String) -> ()
}

object LongPress {
    x: Number
    y: Number
    pointer: String
}

object Swipe {
    direction: String
    dx: Number
    dy: Number
}

object Visible {
    ratio: Number
}

object DragMove {
    value: Json
    over: Json
    position: String?
}

object Drop {
    value: Json
    over: Json
    position: String
}

event click, dblclick, contextmenu, auxclick, mousedown, mouseup, mousemove, mouseover, mouseout, mouseenter, mouseleave: MouseEvent
event pointerdown, pointerup, pointermove, pointerover, pointerout, pointerenter, pointerleave, pointercancel, gotpointercapture, lostpointercapture: PointerEvent
event wheel: WheelEvent
event drag, dragstart, dragend, dragenter, dragleave, dragover, drop: DragEvent
event keydown, keyup, keypress: KeyboardEvent
event input, beforeinput: InputEvent
event submit: SubmitEvent
event focus, blur, focusin, focusout: FocusEvent
event copy, cut, paste: ClipboardEvent
event touchstart, touchmove, touchend, touchcancel: TouchEvent
event animationstart, animationend, animationiteration, animationcancel: AnimationEvent
event transitionstart, transitionend, transitionrun, transitioncancel: TransitionEvent
event message: MessageEvent
event longpress: LongPress
event swipe: Swipe
event visible: Visible
event dragmove: DragMove
event receive: Drop

object Console {
    log(...values) -> ()
    info(...values) -> ()
    warn(...values) -> ()
    error(...values) -> ()
    debug(...values) -> ()
    table(...values) -> ()
    group(...values) -> ()
    groupEnd() -> ()
    time(label?: String) -> ()
    timeEnd(label?: String) -> ()
}

object Storage {
    length: Number
    getItem(key: String) -> String?
    setItem(key: String, value: String) -> ()
    removeItem(key: String) -> ()
    clear() -> ()
    key(index: Number) -> String?
}

object JSON {
    parse(text: String) -> Json
    stringify(value: Json, replacer?: (), space?: Number) -> String
}

object Math {
    PI: Number
    E: Number
    LN2: Number
    LN10: Number
    SQRT2: Number
    abs(x: Number) -> Number
    sign(x: Number) -> Number
    floor(x: Number) -> Number
    ceil(x: Number) -> Number
    round(x: Number) -> Number
    trunc(x: Number) -> Number
    sqrt(x: Number) -> Number
    cbrt(x: Number) -> Number
    exp(x: Number) -> Number
    log(x: Number) -> Number
    log2(x: Number) -> Number
    log10(x: Number) -> Number
    sin(x: Number) -> Number
    cos(x: Number) -> Number
    tan(x: Number) -> Number
    asin(x: Number) -> Number
    acos(x: Number) -> Number
    atan(x: Number) -> Number
    atan2(y: Number, x: Number) -> Number
    pow(x: Number, y: Number) -> Number
    min(...values: Number) -> Number
    max(...values: Number) -> Number
    hypot(...values: Number) -> Number
    random() -> Number
}

object DateConstructor {
    now() -> Number
    parse(text: String) -> Number
}

object ObjectConstructor {
    keys(value: Json) -> [String]
}

object ArrayConstructor {
    isArray(value: Json) -> Bool
}

object NumberConstructor {
    MAX_SAFE_INTEGER: Number
    MIN_SAFE_INTEGER: Number
    EPSILON: Number
    isFinite(value: Number) -> Bool
    isInteger(value: Number) -> Bool
    isNaN(value: Number) -> Bool
    parseFloat(text: String) -> Number
    parseInt(text: String, radix?: Number) -> Number
}

object Navigator {
    language: String
    languages: [String]
    onLine: Bool
    userAgent: String
    platform: String
    maxTouchPoints: Number
    clipboard: Clipboard
    sendBeacon(url: String, data?: String) -> Bool
    vibrate(ms: Number) -> Bool
    async share(data: Json) -> ()
}

object Clipboard {
    async writeText(text: String) -> ()
    async readText() -> String
}

object Location {
    href: String
    origin: String
    protocol: String
    host: String
    hostname: String
    port: String
    pathname: String
    search: String
    hash: String
    reload() -> ()
    assign(url: String) -> ()
    replace(url: String) -> ()
}

object History {
    length: Number
    state: Json
    back() -> ()
    forward() -> ()
    go(delta: Number) -> ()
    pushState(state: Json, title: String, url?: String) -> ()
    replaceState(state: Json, title: String, url?: String) -> ()
}

object Performance {
    now() -> Number
}

object Screen {
    width: Number
    height: Number
    availWidth: Number
    availHeight: Number
}

object MediaQueryList {
    matches: Bool
    media: String
}

object Error {
    name: String
    message: String
    stack: String?
}
";

/// A parameter of a function or a method of [`BROWSER_TYPES`].
#[derive(Debug, Clone)]
pub(crate) struct BrowserParam {
    pub(crate) name: String,
    /// What it takes; `None` for `...name` alone, which takes any value
    /// but a secret.
    pub(crate) ty: Option<crate::syntax::parser::ast::TypeRef>,
    /// `name?: T`: it may be left out.
    pub(crate) optional: bool,
    /// `...name`: it takes any number of arguments, the last.
    pub(crate) rest: bool,
}

/// A function, or a method, the browser has.
#[derive(Debug, Clone)]
pub(crate) struct BrowserFn {
    pub(crate) params: Vec<BrowserParam>,
    pub(crate) ret: crate::syntax::parser::ast::TypeRef,
    /// Whether it hands back a promise of `ret`.
    pub(crate) is_async: bool,
}

/// A member of a browser object: a field, or a method.
#[derive(Debug, Clone)]
pub(crate) enum BrowserMember {
    Field(crate::syntax::parser::ast::TypeRef),
    Method(BrowserFn),
}

/// An object the browser hands a program.
#[derive(Debug, Clone)]
pub(crate) struct BrowserObject {
    pub(crate) name: String,
    /// The object whose members it has too: `KeyboardEvent`'s `Event`.
    pub(crate) base: Option<String>,
    pub(crate) members: Vec<(String, BrowserMember)>,
}

/// [`BROWSER_TYPES`], read.
#[derive(Debug, Default)]
pub(crate) struct Browser {
    pub(crate) values: Vec<(String, crate::syntax::parser::ast::TypeRef)>,
    pub(crate) functions: Vec<(String, BrowserFn)>,
    pub(crate) objects: Vec<BrowserObject>,
    /// What each DOM event's handler is given, by the event's name.
    pub(crate) events: Vec<(String, String)>,
}

impl Browser {
    /// The type of a global value, `window`.
    pub(crate) fn value(&self, name: &str) -> Option<&crate::syntax::parser::ast::TypeRef> {
        self.values.iter().find(|(n, _)| n == name).map(|(_, t)| t)
    }

    /// A global function, `setTimeout`.
    pub(crate) fn function(&self, name: &str) -> Option<&BrowserFn> {
        self.functions.iter().find(|(n, _)| n == name).map(|(_, f)| f)
    }

    pub(crate) fn object(&self, name: &str) -> Option<&BrowserObject> {
        self.objects.iter().find(|o| o.name == name)
    }

    /// A member of an object, or of an object it extends.
    pub(crate) fn member(&self, object: &str, member: &str) -> Option<&BrowserMember> {
        let mut at = self.object(object);
        while let Some(o) = at {
            if let Some((_, m)) = o.members.iter().find(|(n, _)| n == member) {
                return Some(m);
            }
            at = o.base.as_deref().and_then(|b| self.object(b));
        }
        None
    }

    /// Every member's name an object has, its own first.
    pub(crate) fn member_names(&self, object: &str) -> Vec<&str> {
        let mut names = Vec::new();
        let mut at = self.object(object);
        while let Some(o) = at {
            names.extend(o.members.iter().map(|(n, _)| n.as_str()));
            at = o.base.as_deref().and_then(|b| self.object(b));
        }
        names
    }

    /// Whether `object` is `ancestor`, or extends it.
    pub(crate) fn extends(&self, object: &str, ancestor: &str) -> bool {
        let mut at = Some(object);
        while let Some(name) = at {
            if name == ancestor {
                return true;
            }
            at = self.object(name).and_then(|o| o.base.as_deref());
        }
        false
    }

    /// What a DOM event's handler is given: `KeyboardEvent` for `keydown`,
    /// `Event` for an event the table does not name.
    pub(crate) fn event(&self, event: &str) -> &str {
        self.events
            .iter()
            .find(|(n, _)| n == event)
            .map_or("Event", |(_, t)| t.as_str())
    }
}

/// [`BROWSER_TYPES`], read once.
pub(crate) fn browser() -> &'static Browser {
    static BROWSER: std::sync::LazyLock<Browser> = std::sync::LazyLock::new(|| {
        read_browser(BROWSER_TYPES).unwrap_or_else(|e| panic!("BROWSER_TYPES: {e}"))
    });
    &BROWSER
}

fn read_type(text: &str) -> Result<crate::syntax::parser::ast::TypeRef, String> {
    crate::syntax::parser::v2::parse_type_text(text.trim())
        .ok_or_else(|| format!("`{text}` is not a type"))
}

/// `name(params) -> R`, with `async` already taken off.
fn read_fn(text: &str, is_async: bool) -> Result<(String, BrowserFn), String> {
    let open = text.find('(').ok_or_else(|| format!("`{text}` has no `(`"))?;
    let close = text.rfind(") ->").ok_or_else(|| format!("`{text}` has no `) ->`"))?;
    let name = text[..open].trim().to_string();
    let mut params = Vec::new();
    let inside = &text[open + 1..close];
    // Split at top-level commas: a parameter's type may hold its own.
    let mut depth = 0i32;
    let mut start = 0;
    let mut pieces = Vec::new();
    for (i, c) in inside.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                pieces.push(&inside[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    pieces.push(&inside[start..]);
    for piece in pieces.into_iter().map(str::trim).filter(|p| !p.is_empty()) {
        let (rest, piece) = match piece.strip_prefix("...") {
            Some(p) => (true, p),
            None => (false, piece),
        };
        let (head, ty) = match piece.split_once(':') {
            Some((h, t)) => (h.trim(), Some(read_type(t)?)),
            None => (piece, None),
        };
        let (pname, optional) = match head.strip_suffix('?') {
            Some(n) => (n, true),
            None => (head, false),
        };
        params.push(BrowserParam {
            name: pname.to_string(),
            ty,
            optional,
            rest,
        });
    }
    let ret = read_type(&text[close + 4..])?;
    Ok((
        name,
        BrowserFn {
            params,
            ret,
            is_async,
        },
    ))
}

fn read_browser(text: &str) -> Result<Browser, String> {
    let mut out = Browser::default();
    let mut object: Option<BrowserObject> = None;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if let Some(o) = object.as_mut() {
            if line == "}" {
                out.objects.extend(object.take());
                continue;
            }
            let (is_async, line) = match line.strip_prefix("async ") {
                Some(l) => (true, l),
                None => (false, line),
            };
            let paren = line.find('(');
            let colon = line.find(':');
            if paren.is_some_and(|p| colon.is_none_or(|c| p < c)) {
                let (name, f) = read_fn(line, is_async)?;
                o.members.push((name, BrowserMember::Method(f)));
            } else {
                let (name, ty) = line
                    .split_once(':')
                    .ok_or_else(|| format!("`{line}` is neither a field nor a method"))?;
                o.members
                    .push((name.trim().to_string(), BrowserMember::Field(read_type(ty)?)));
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("let ") {
            let (name, ty) = rest
                .split_once(':')
                .ok_or_else(|| format!("`{line}` has no type"))?;
            out.values.push((name.trim().to_string(), read_type(ty)?));
        } else if let Some(rest) = line.strip_prefix("fn ") {
            out.functions.push(read_fn(rest, false)?);
        } else if let Some(rest) = line.strip_prefix("object ") {
            let head = rest
                .strip_suffix('{')
                .ok_or_else(|| format!("`{line}` does not open a block"))?
                .trim();
            let (name, base) = match head.split_once(':') {
                Some((n, b)) => (n.trim(), Some(b.trim().to_string())),
                None => (head, None),
            };
            object = Some(BrowserObject {
                name: name.to_string(),
                base,
                members: Vec::new(),
            });
        } else if let Some(rest) = line.strip_prefix("event ") {
            let (names, ty) = rest
                .rsplit_once(':')
                .ok_or_else(|| format!("`{line}` has no type"))?;
            for name in names.split(',').map(str::trim) {
                out.events.push((name.to_string(), ty.trim().to_string()));
            }
        } else {
            return Err(format!("`{line}` is not a declaration"));
        }
    }
    if object.is_some() {
        return Err("an object is not closed".to_string());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_browser_table_reads_and_names_only_what_it_declares() {
        // Every type it names is the language's, or an object it declares.
        fn named(t: &crate::syntax::parser::ast::TypeRef, out: &mut Vec<String>) {
            use crate::syntax::parser::ast::TypeRef;
            match t {
                TypeRef::Named(n) => out.push(n.clone()),
                TypeRef::List(i) | TypeRef::Optional(i) | TypeRef::Refined(i, _) => named(i, out),
                TypeRef::Dict(k, v) => {
                    named(k, out);
                    named(v, out);
                }
                TypeRef::Func(ps, r) => {
                    ps.iter().for_each(|p| named(p, out));
                    named(r, out);
                }
                _ => {}
            }
        }
        let b = browser();
        assert!(b.value("window").is_some());
        assert!(b.function("setTimeout").is_some());
        assert!(matches!(b.member("KeyboardEvent", "key"), Some(BrowserMember::Field(_))));
        assert!(
            matches!(b.member("KeyboardEvent", "preventDefault"), Some(BrowserMember::Method(_))),
            "an event has every member of `Event`"
        );
        assert_eq!(b.event("keydown"), "KeyboardEvent");
        assert_eq!(b.event("anything-else"), "Event");
        let mut names = Vec::new();
        let fns = b
            .functions
            .iter()
            .map(|(_, f)| f)
            .chain(b.objects.iter().flat_map(|o| {
                o.members.iter().filter_map(|(_, m)| match m {
                    BrowserMember::Method(f) => Some(f),
                    BrowserMember::Field(_) => None,
                })
            }));
        for f in fns {
            f.params.iter().filter_map(|p| p.ty.as_ref()).for_each(|t| named(t, &mut names));
            named(&f.ret, &mut names);
        }
        for o in &b.objects {
            for (_, m) in &o.members {
                if let BrowserMember::Field(t) = m {
                    named(t, &mut names);
                }
            }
        }
        b.values.iter().for_each(|(_, t)| named(t, &mut names));
        names.extend(b.events.iter().map(|(_, t)| t.clone()));
        for name in names {
            assert!(
                b.object(&name).is_some() || matches!(name.as_str(), "Json" | "File" | "Date"),
                "`{name}` is not an object the table declares"
            );
        }
    }
}
