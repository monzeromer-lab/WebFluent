//! What the browser and the runtime bring: the names a program may use
//! unprefixed, the live values, what the language's own types can do, the
//! gestures and drag props, and the attributes a component's root takes.

/// The events a runtime recogniser turns the pointer into (`gesture.js`).
pub const GESTURE_EVENTS: &[&str] = &["longpress", "swipe", "receive", "dragmove"];

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
