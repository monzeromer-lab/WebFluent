//! The names the language has: built-in functions, grammar words, and the browser's globals.

use super::*;

/// The types of the names every program can read.
/// The functions the language gives a program, which it calls by name.
pub(crate) const BUILT_IN_FUNCTIONS: &[&str] = &[
    "log",
    "navigate",
    "format",
    "ago",
    "t",
    "setLocale",
    "setTheme",
    "uuid",
    "keyIs",
    "keyLabel",
    "sanitize",
    "fetch",
    "optimistic",
    "beacon",
    "animate",
    "replayAnimation",
    "every",
    "after",
    "now",
    "ws",
    "sse",
    "broadcast",
    "rtc",
    "String",
    "Number",
    "Boolean",
    "Bool",
    "Money",
];

/// The words of the grammar a program's own name may not be: those a
/// statement in a body or an imperative block is written with.
pub(super) const GRAMMAR_WORDS: &[&str] = &[
    "state",
    "persist",
    "derived",
    "effect",
    "action",
    "use",
    "resource",
    "validate",
    "socket",
    "stream",
    "channel",
    "peer",
    "head",
    "slot",
    "part",
    "if",
    "else",
    "for",
    "in",
    "by",
    "show",
    "match",
    "children",
    "emit",
    "sequence",
    "on",
    "style",
    "transition",
    "let",
    "return",
    "await",
    "try",
    "catch",
];

/// What the language already means by `name`, when a program's own value of
/// that name would hide it: the build's settings, a value the browser keeps
/// current, one of the language's functions, or a word of its grammar. A
/// declaration of any of them is `E118`.
///
/// The names a handler brings with it (`event`, `e`, `value`, `key`) and the
/// browser's globals are not among them: those a program may name its own.
pub fn language_name(name: &str) -> Option<String> {
    let what = match name {
        "env" => "the build's settings, read as `env.PUBLIC_API`",
        "params" => "the route's parameters, read as `params.id`",
        "locale" => "the reader's language, which `setLocale` changes",
        "dir" => "the direction of the reader's language, `ltr` or `rtl`",
        "query" => "the address's query, read as `query.tab`",
        "hash" => "the address's `#fragment`",
        "theme" => "the reader's light or dark choice, which `setTheme` changes",
        "now" => "the current time, which the language keeps current",
        "viewport" => "the window's size, read as `viewport.md`",
        "network" => "the connection's state, read as `network.online`",
        "update" => "a new version of the site, read as `update.available`",
        "t" => "the translation function, `t(\"key\")`",
        _ if BUILT_IN_FUNCTIONS.contains(&name) && name.starts_with(char::is_lowercase) => {
            return Some(format!("one of its functions, `{name}(…)`"));
        }
        _ if GRAMMAR_WORDS.contains(&name) => "a word of its grammar",
        _ => return None,
    };
    Some(what.to_string())
}

/// A name of a program's own for a value `name` would have meant, when one
/// is usual.
pub(super) fn own_name_for(name: &str) -> Option<&'static str> {
    Some(match name {
        "env" => "`target` or `stage`",
        "params" => "`args`",
        "locale" => "`language`",
        "dir" => "`direction`",
        "query" => "`search` or `q`",
        "hash" => "`fragment`",
        "theme" => "`scheme`",
        "now" => "`at` or `moment`",
        "t" => "`item`, or a word for what it holds",
        _ => return None,
    })
}

pub(super) fn global_type(name: &str) -> Type {
    match name {
        "event" | "params" | "window" | "document" | "console" | "localStorage"
        | "sessionStorage" | "JSON" | "Math" | "Date" | "navigator" | "location" | "fetch"
        | "setTimeout" | "clearTimeout" | "setInterval" | "clearInterval" | "Object" | "Array"
        | "Promise" | "locale" | "dir" => Type::Any,
        // The project's `env`: a map of what the config set.
        "env" => Type::Map,
        // The browser as values.
        "viewport" => Type::Shape(vec![
            ("width".to_string(), Type::Number),
            ("height".to_string(), Type::Number),
            ("sm".to_string(), Type::Bool),
            ("md".to_string(), Type::Bool),
            ("lg".to_string(), Type::Bool),
            ("xl".to_string(), Type::Bool),
        ]),
        "query" => Type::Map,
        "hash" => Type::String,
        // What the reader chose: `light`, `dark` or `system`.
        "theme" => Type::String,
        // The moment the page is at, kept current.
        "now" => Type::Scalar(Scalar::DateTime),
        // The connection, as the page can read it.
        "network" => Type::Shape(vec![
            ("online".to_string(), Type::Bool),
            ("effectiveType".to_string(), Type::String),
            ("saveData".to_string(), Type::Bool),
            ("downlink".to_string(), Type::Number),
            // The writes kept for when the connection returns (`offline.sync`).
            ("queued".to_string(), Type::Number),
        ]),
        // A new version of the site, installed and waiting (`offline`).
        "update" => Type::Shape(vec![
            ("available".to_string(), Type::Bool),
            (
                "apply".to_string(),
                Type::Func(Vec::new(), Box::new(Type::Null)),
            ),
        ]),
        _ => Type::Any,
    }
}

/// The functions the language gives a program, which it calls by name.
pub fn built_in_functions() -> &'static [&'static str] {
    BUILT_IN_FUNCTIONS
}
