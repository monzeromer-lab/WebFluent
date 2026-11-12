//! What a program's names become in JavaScript: signals, bare names, top-level names, and the names a body declares.

use super::{Statement, StatementKind};

/// A component's JavaScript name: a part's `Owner.Part` is `Owner_Part`.
pub(super) fn js_component(name: &str) -> String {
    name.replace('.', "_")
}

/// `text` as a JavaScript string literal, quoted. Every string the
/// generator writes between quotes goes through here, so a title, a path
/// or a locale holding a quote, a backslash or a line break is the string
/// it says and never the end of one. The lexer's escaped-brace
/// placeholders are the braces they stand for.
pub(crate) fn js_str(text: &str) -> String {
    let text = text.replace('\u{FFFE}', "{").replace('\u{FFFF}', "}");
    serde_json::to_string(&text).unwrap_or_else(|_| "\"\"".to_string())
}

/// The literal text of a template literal (`` `…${x}…` ``) as JavaScript
/// reads it back: `\`, `` ` `` and `${` escaped, so `"C:\\dir\\{name}"`
/// keeps its backslashes and `"$\{x\}"` stays text rather than becoming a
/// splice. (A text part is followed by a splice or by the end, never by
/// another text part, so a `$` that ends one cannot meet a `{`.)
pub(crate) fn template_text(text: &str) -> String {
    text.replace('\u{FFFE}', "{")
        .replace('\u{FFFF}', "}")
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${")
}

/// The names the generated code declares for itself beside a program's
/// signals: a component's props and slots, the fragment a body is drawn
/// into, the app's root and router, a form's handle, a handler's event, a
/// caught error, a scoped slot's values, a loop's index. (`_n`, the node a
/// component's call marks, holds no program code, so `n` is free.)
pub(super) const RESERVED_SIGNALS: &[&str] = &[
    "p", "slots", "frag", "root", "app", "routes", "routerEl", "error", "wfForm", "ev", "cf", "s",
    "idx",
];

/// What a program's state, derived value, resource or connection `name` is
/// called in the generated code: `_name`, unless that is one of the
/// generated code's own names — then `_name$`, which nothing else spells. A
/// `derived p` was `const _p` beside the props' `_p`, so the page's script
/// did not parse and nothing drew.
pub(super) fn sig(name: &str) -> String {
    // `_e3` and what hangs off it (`_e3_drawn`) are the elements the
    // generated code makes; `_x_check` and `_x_pending` are what it keeps
    // for a validated state and an async action.
    let element = name.strip_prefix('e').is_some_and(|rest| {
        let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        digits > 0 && (digits == rest.len() || rest[digits..].starts_with('_'))
    });
    if element
        || RESERVED_SIGNALS.contains(&name)
        || name.ends_with("_check")
        || name.ends_with("_pending")
    {
        format!("_{name}$")
    } else {
        format!("_{name}")
    }
}

/// Names a program's binding cannot be written as in the generated code:
/// JavaScript's own words (`for class in classes` was `for (const class
/// of …)`), the browser's names the generated code reads inside a
/// program's code (`document.createDocumentFragment()` under `for document
/// in docs`), the globals no script may declare again, and the parameters
/// of the functions the generated code wraps around a program's code — a
/// store's `store`, a layout's `page`, the router's `routes` and
/// `container`. (An event or a bound value it hands such a function is
/// `$$e` or `$$v`, which no program spells.)
pub(super) const RESERVED_BARE: &[&str] = &[
    // JavaScript's.
    "arguments",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "undefined",
    "NaN",
    "Infinity",
    // The browser's, as the generated code reads them.
    "Array",
    "Number",
    "Object",
    "String",
    "console",
    "document",
    "encodeURIComponent",
    "globalThis",
    "location",
    "top",
    "window",
    // The generated code's own.
    "WF",
    "store",
    "page",
    "routes",
    "container",
    "__x",
];

/// What a program's binding — a loop's item, a lambda's parameter, a
/// `let`, a caught error, an action, a `const` — is called in the generated
/// code: its own name, unless that is one of [`RESERVED_BARE`] or begins
/// like a page's or a component's function — then `name$`, which nothing
/// else spells (a program's names have no `$`).
pub(super) fn bare(name: &str) -> String {
    if RESERVED_BARE.contains(&name)
        || name.starts_with("Page_")
        || name.starts_with("Component_")
        || name.starts_with("__wf")
    {
        format!("{name}$")
    } else {
        name.to_string()
    }
}

/// Every name a browser's global object has (`scripts/browser-globals.sh`).
pub(super) static BROWSER_GLOBAL_NAMES: std::sync::LazyLock<
    std::collections::HashSet<&'static str>,
> = std::sync::LazyLock::new(|| {
    include_str!("../browser_globals.txt")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
});

/// What a program's top-level name — a `const`, a `data` or `image`
/// constant, a store, a service — is called in the generated code: as
/// [`bare`] has it, and `name$` when the browser's global object has the
/// name. Every script's top-level `const` shares one scope with the
/// runtime's functions, so `const Map = 3` was the runtime's `Map` — "Map
/// is not a constructor", and no page drew — and a store called `Image` was
/// a library's.
pub(super) fn top_level(name: &str) -> String {
    if BROWSER_GLOBAL_NAMES.contains(name) {
        format!("{name}$")
    } else {
        bare(name)
    }
}

/// A lambda's parameters, `a, b`, each as [`bare`] has it.
pub(super) fn bare_list(params: &str) -> String {
    params
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(bare)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Every name `body` declares at any depth: state, derived values,
/// resources, actions and the locals of their bodies.
pub(super) fn declared_names(body: &[Statement]) -> Vec<String> {
    fn walk(stmts: &[Statement], out: &mut Vec<String>) {
        for stmt in stmts {
            let name = match &stmt.kind {
                StatementKind::State(s) => Some(&s.name),
                StatementKind::Derived(d) => Some(&d.name),
                StatementKind::Resource(r) => Some(&r.name),
                StatementKind::Action(a) => Some(&a.name),
                _ => None,
            };
            if let Some(name) = name
                && !out.contains(name)
            {
                out.push(name.clone());
            }
            if let StatementKind::Action(a) = &stmt.kind {
                for p in &a.params {
                    if !out.contains(&p.name) {
                        out.push(p.name.clone());
                    }
                }
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
            if let StatementKind::UIElement(el) = &stmt.kind {
                walk(&el.children, out);
                for h in &el.events {
                    walk(&h.body, out);
                }
                for f in &el.slot_fills {
                    walk(&f.body, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(body, &mut out);
    out
}

/// The signals declared directly in `body`: states, derived values,
/// resources and connections.
pub(super) fn body_signals(body: &[Statement]) -> Vec<String> {
    body.iter()
        .filter_map(|s| match &s.kind {
            StatementKind::State(d) => Some(d.name.clone()),
            StatementKind::Derived(d) => Some(d.name.clone()),
            StatementKind::Resource(r) => Some(r.name.clone()),
            StatementKind::Connection(c) => Some(c.name.clone()),
            _ => None,
        })
        .collect()
}

/// The names of the actions declared directly in `body`.
pub(super) fn action_names(body: &[Statement]) -> Vec<String> {
    body.iter()
        .filter_map(|s| match &s.kind {
            StatementKind::Action(a) => Some(a.name.clone()),
            _ => None,
        })
        .collect()
}
