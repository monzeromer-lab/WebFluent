//! Libraries from other origins, imported by a generated module.

/// The loader for the libraries `meta.scripts` names: each ES module
/// imported and put on the global its `as` names, and every class a
/// library hands over made callable.
///
/// It is a **separate file**, not part of the bundle, for one reason: a
/// module has its own scope, and the page chunks a split build writes are
/// classic scripts that read the bundle's names from the global one. So
/// the imports live here, each bound to a global, and the bundle stays
/// what it was. It is linked after the plain library scripts and before
/// the project's own, and a module script and a deferred classic script
/// run in the order they appear in the document, so it has run before
/// anything that reads it.
pub fn externals_module(config: &crate::project::config::ProjectConfig) -> Option<String> {
    use crate::project::config::project::ScriptEntry;
    let modules: Vec<(&str, &str)> = config
        .meta
        .scripts
        .iter()
        .filter_map(|s| match s {
            ScriptEntry::Spec(spec) if spec.module => {
                Some((spec.src.as_str(), spec.as_name.as_deref()?))
            }
            _ => None,
        })
        .collect();
    let globals: Vec<&str> = config
        .meta
        .scripts
        .iter()
        .filter_map(|s| match s {
            ScriptEntry::Spec(spec) if !spec.module => Some(spec.globals.iter()),
            _ => None,
        })
        .flatten()
        .map(String::as_str)
        .collect();
    if modules.is_empty() && globals.is_empty() {
        return None;
    }
    let mut out =
        String::from("// The libraries `meta.scripts` names, as this project names them.\n");
    for (src, name) in &modules {
        out.push_str(&format!(
            "import * as {name} from {};\n",
            serde_json::to_string(src).unwrap_or_default()
        ));
    }
    // A library's export may be a class — Chart.js's `Chart`, most modern
    // libraries' main export — and a WebFluent call has no `new`. Calling a
    // class without it throws, so a class is put behind a proxy that
    // constructs it when it is called, its static members as they were; a
    // module's exports are copied onto a plain object first, since a module
    // namespace cannot be proxied.
    out.push_str(
        "const __wfClass = (v) => typeof v === \"function\" && /^class[\\s{]/.test(Function.prototype.toString.call(v)) \
         ? new Proxy(v, { apply: (c, _, args) => Reflect.construct(c, args) }) : v;\n\
         const __wfCallable = (ns) => { const o = {}; for (const k of Object.keys(ns)) o[k] = __wfClass(ns[k]); return o; };\n",
    );
    for (_, name) in &modules {
        out.push_str(&format!("globalThis.{name} = __wfCallable({name});\n"));
    }
    for name in &globals {
        out.push_str(&format!(
            "globalThis.{name} = __wfClass(globalThis.{name});\n"
        ));
    }
    Some(out)
}

/// Every module `meta.scripts` imports, with the hash `meta.integrity`
/// gives it.
pub fn external_modules(
    config: &crate::project::config::ProjectConfig,
) -> Vec<(String, Option<String>)> {
    config
        .meta
        .scripts
        .iter()
        .filter(|s| s.is_module())
        .map(|s| {
            (
                s.src().to_string(),
                config.meta.integrity.get(s.src()).cloned(),
            )
        })
        .collect()
}

/// The browser's classes a program may call by name: `Blob([text])` is
/// `new Blob([text])`, since most of them throw when called without `new`
/// (and `Date(x)` ignores `x`).
pub(super) const BROWSER_CLASSES: &[&str] = &[
    "AbortController",
    "ArrayBuffer",
    "Blob",
    "BroadcastChannel",
    "CustomEvent",
    "DOMParser",
    "DataView",
    "Date",
    "Event",
    "File",
    "FileReader",
    "FormData",
    "Headers",
    "IntersectionObserver",
    "Map",
    "MutationObserver",
    "Promise",
    "Request",
    "ResizeObserver",
    "Response",
    "Set",
    "TextDecoder",
    "TextEncoder",
    "URL",
    "URLSearchParams",
    "Uint8Array",
    "WeakMap",
    "WeakSet",
    "Worker",
];
