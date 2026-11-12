//! What a request and a connection take, and what each hears: the options
//! of `fetch(…)`, of an endpoint's call and of the functions that open a
//! connection, the cases those options take, the events a connection's
//! block handles, and what each event a page handles outside any element
//! is. The language server offers them from here; the tests below hold
//! each name to the parser that reads it and the runtime that acts on it.

use crate::syntax::parser::v2::PAGE_EVENTS;

/// A named argument with its documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgDoc {
    /// The argument's name, as it is written before the `:`.
    pub name: &'static str,
    /// What it does, in Markdown.
    pub doc: &'static str,
}

/// An argument with its documentation, for a table.
pub const fn arg(name: &'static str, doc: &'static str) -> ArgDoc {
    ArgDoc { name, doc }
}

/// The options `fetch(url, …)` takes, in a `resource` or after `await`.
pub const RESOURCE_OPTIONS: &[ArgDoc] = &[
    arg("method", "HTTP method, e.g. `\"POST\"`"),
    arg("headers", "Map of request headers; quote hyphenated keys"),
    arg("body", "Request body, sent as JSON"),
    arg(
        "on",
        "When a resource asks again: `.focus`, `.reconnect` or `.interval(30.seconds)`. Asking again is a refresh — the data stays on the page while it runs",
    ),
    arg(
        "cache",
        "How long an answer stands: `.swr(60.seconds)` (served, then revalidated), `.none`, `.forever`",
    ),
    arg(
        "paginate",
        "`paginate: .page` — the resource gathers the pages as `.items`, with `.loadMore()` and `.hasMore`",
    ),
];

/// What a call of a service's endpoint (`Backend.users(…)`) takes beside the
/// endpoint's own parameters.
pub const ENDPOINT_CALL_OPTIONS: &[ArgDoc] = &[
    arg(
        "cache",
        "How long an answer stands: `.swr(60.seconds)`, `.none`, `.forever` — keyed by method, address and body",
    ),
    arg(
        "on",
        "When a resource over it asks again: `.focus`, `.reconnect` or `.interval(30.seconds)`",
    ),
    arg(
        "paginate",
        "`paginate: .page` — `.items` gathers the pages; `.loadMore()` and `.hasMore`",
    ),
    arg(
        "as",
        "How the response is read: JSON by default, or `.text`, `.blob`, `.arrayBuffer`",
    ),
    arg(
        "errorAs",
        "How an error's body is read: `.text`, `.blob`, `.arrayBuffer`, JSON by default",
    ),
    arg(
        "fileField",
        "The form field a `File` parameter is sent as, when it is not the parameter's name",
    ),
];

/// A case an option takes: its name, the text it inserts, what it does.
pub type OptionCase = (&'static str, &'static str, &'static str);

/// The cases an option of a request takes after `name: .`, each with the
/// text it inserts and what it does.
pub const OPTION_CASES: &[(&str, &[OptionCase])] = &[
    (
        "on",
        &[
            ("focus", "focus", "Ask again when the window regains focus"),
            (
                "reconnect",
                "reconnect",
                "Ask again when the network comes back",
            ),
            (
                "interval",
                "interval(${1:30}.seconds)",
                "Ask again every so often",
            ),
        ],
    ),
    (
        "cache",
        &[
            (
                "swr",
                "swr(${1:60}.seconds)",
                "Serve what is held, and revalidate it once it is older than this",
            ),
            ("none", "none", "Hold nothing; every call asks the network"),
            ("forever", "forever", "Hold the first answer for good"),
        ],
    ),
    (
        "paginate",
        &[(
            "page",
            "page",
            "By the `page` argument: `.loadMore()` asks for the next one",
        )],
    ),
    (
        "as",
        &[
            ("text", "text", "The body as text"),
            ("blob", "blob", "The body as a `Blob`"),
            ("arrayBuffer", "arrayBuffer", "The body as an `ArrayBuffer`"),
        ],
    ),
    (
        "errorAs",
        &[
            ("text", "text", "An error's body as text"),
            ("blob", "blob", "An error's body as a `Blob`"),
            (
                "arrayBuffer",
                "arrayBuffer",
                "An error's body as an `ArrayBuffer`",
            ),
        ],
    ),
    (
        "type",
        &[(
            "module",
            "module",
            "A module worker: the script may `import`",
        )],
    ),
];

/// The options each function that opens a connection takes: `ws("wss://…",
/// protocols: […])`.
pub const CONNECTION_OPTIONS: &[(&str, &[ArgDoc])] = &[
    (
        "ws",
        &[
            arg("protocols", "The subprotocols to offer: `[\"v2\"]`"),
            arg(
                "heartbeat",
                "How often to ping, to keep the socket alive: `20.seconds`",
            ),
            arg(
                "keep",
                "How many messages `.messages` holds (the newest; 500 unset)",
            ),
            arg(
                "reconnect",
                "`false` to stay closed when the socket drops; it reconnects with backoff otherwise",
            ),
        ],
    ),
    (
        "sse",
        &[
            arg(
                "events",
                "The named events to hear beside `message`: `[\"price\", \"status\"]`",
            ),
            arg(
                "keep",
                "How many messages `.messages` holds (the newest; 500 unset)",
            ),
            arg("credentials", "`true` to send the site's cookies"),
        ],
    ),
    (
        "broadcast",
        &[arg(
            "keep",
            "How many messages `.messages` holds (the newest; 500 unset)",
        )],
    ),
    (
        "rtc",
        &[
            arg(
                "signal",
                "Where what the other side must hear goes: `m => lobby.post(m)`; what it sent back is handed to `link.signal(m)`",
            ),
            arg(
                "initiator",
                "Whether this side makes the offer; exactly one side does",
            ),
            arg(
                "ice",
                "The servers that help two readers on different networks find each other: `[{ urls: \"stun:…\" }]`",
            ),
            arg(
                "keep",
                "How many messages `.messages` holds (the newest; 500 unset)",
            ),
        ],
    ),
    (
        "spawn",
        &[
            arg(
                "restart",
                "How many times a worker that crashed is started again before it is `error`",
            ),
            arg(
                "timeout",
                "How long a `.request` may wait for its answer, in ms (10 s unset)",
            ),
            arg("type", "`.module` for a module worker"),
        ],
    ),
];

/// The events a connection's block may handle, by the word it is declared
/// with — what the runtime calls (`onMessage`, `onRestart`, …).
pub const CONNECTION_EVENTS: &[(&str, &[(&str, &str)])] = &[
    (
        "socket",
        &[
            ("message", "A message arrived, read as the `receive` type"),
            ("open", "The socket opened, or opened again"),
        ],
    ),
    ("stream", &[("message", "An event arrived")]),
    (
        "channel",
        &[
            ("message", "Another tab posted a value"),
            ("open", "The channel is joined"),
        ],
    ),
    (
        "peer",
        &[
            ("message", "The other side sent a value"),
            ("open", "The data channel opened"),
        ],
    ),
    (
        "worker",
        &[
            (
                "message",
                "The worker posted something with no `id`: not an answer to a request",
            ),
            (
                "restart",
                "The worker crashed and is started again; what was out failed",
            ),
            (
                "error",
                "The worker crashed past its `restart:` count, and stopped",
            ),
        ],
    ),
];

/// What each event a page, a component or the app may handle outside any
/// element is: the document's keys, and what only the window says.
pub fn page_event_doc(name: &str) -> &'static str {
    match name {
        "key" => "One key combination, on the document: `on key(\"mod+k\") { … }`",
        "keydown" => "A key went down anywhere on the page",
        "keyup" => "A key came up anywhere on the page",
        "resize" => "The window changed size",
        "scroll" => "The window scrolled",
        "hashchange" => "The address's `#fragment` changed",
        "popstate" => "The reader went back or forward",
        "online" => "The connection came back",
        "offline" => "The connection went away",
        "message" => "Another window or frame posted a message to this one",
        "storage" => "Another tab changed the site's storage",
        "visibilitychange" => "The page was hidden or shown",
        "blur" => "The window lost focus",
        "focus" => "The window gained focus",
        _ => "An event of the window or the document",
    }
}

/// What each of [`PAGE_EVENTS`] is.
pub fn page_event_docs() -> impl Iterator<Item = (&'static str, &'static str)> {
    PAGE_EVENTS.iter().map(|name| (*name, page_event_doc(name)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parser::v2::ENDPOINT_SETTINGS;

    /// The runtime's modules, which read every option and call every
    /// handler named here.
    fn runtime() -> String {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/web/runtime/modules");
        let mut all = String::new();
        for entry in std::fs::read_dir(dir).expect("the runtime's modules") {
            let path = entry.expect("a module").path();
            if path.extension().is_some_and(|e| e == "js") {
                all.push_str(&std::fs::read_to_string(&path).expect("a module's text"));
            }
        }
        all
    }

    #[test]
    fn every_setting_the_parser_takes_on_an_endpoint_call_is_documented() {
        for name in ENDPOINT_SETTINGS {
            assert!(
                ENDPOINT_CALL_OPTIONS.iter().any(|o| o.name == *name),
                "`{name}` is parsed on an endpoint's call but not documented"
            );
        }
    }

    #[test]
    fn every_connection_option_is_one_the_runtime_reads() {
        let runtime = runtime();
        for (opener, options) in CONNECTION_OPTIONS {
            for option in *options {
                let name = option.name;
                assert!(
                    runtime.contains(&format!("opts.{name}")),
                    "`{opener}(…, {name}:)` is offered, but no runtime module reads `opts.{name}`"
                );
            }
        }
    }

    #[test]
    fn every_connection_event_is_one_the_runtime_calls() {
        let runtime = runtime();
        for (kind, events) in CONNECTION_EVENTS {
            for (event, _) in *events {
                let mut handler = String::from("on");
                handler.push_str(&event[..1].to_uppercase());
                handler.push_str(&event[1..]);
                assert!(
                    runtime.contains(&handler),
                    "`{kind}` offers `on {event}`, but no runtime module calls `{handler}`"
                );
            }
        }
    }

    #[test]
    fn every_page_event_says_what_it_is() {
        for (name, doc) in page_event_docs() {
            assert_ne!(
                doc, "An event of the window or the document",
                "`on {name}` at a page's top has no description of its own"
            );
        }
    }
}
