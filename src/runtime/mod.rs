//! The JavaScript runtime, in feature modules.
//!
//! The runtime is not one file any more. It is a set of modules — `each`,
//! `router`, `format`, `icons`, `carousel` … — each a fragment of the same
//! closure, and a build carries only the ones its program reaches. What a
//! program reaches is read from the program itself: every `WF.<name>` the
//! code generator emitted, plus a few attributes that name a feature without
//! naming a function (`"data-icon"` reaches the icon table), closed over the
//! manifest's dependencies. Nothing is hand-marked, so the set cannot drift
//! from the ~100 places that emit a call.
//!
//! The modules are concatenated **inside one closure**, in the order they are
//! declared here — which is the order they were written in — so the private
//! state they share (`currentEffect`, `_basePath`, `pages`, `_ICONS`) keeps
//! working exactly as it did when this was a single file. Every module is
//! declarations only: nothing runs at load but registering a hook.
//!
//! `build.runtime: "full"` in `webfluent.app.json` turns the pruning off and
//! ships everything, for a site whose own `<script>` reaches for `WF` in ways
//! a build cannot see.

use std::collections::BTreeSet;

/// One feature of the runtime: a fragment of the closure, what it puts on
/// `WF`, what it needs, and what in a program reaches it.
pub struct Module {
    /// The module's name, as `deps` spell it.
    pub name: &'static str,
    /// Its source: top-level declarations at two spaces of indentation.
    pub source: &'static str,
    /// What it exports, as `(the name on `WF`, the name in the closure)`.
    pub exports: &'static [(&'static str, &'static str)],
    /// The modules it calls into. `core` is implicit and always present.
    pub deps: &'static [&'static str],
    /// Text in a program that reaches this module without calling one of its
    /// exports — an attribute a hook answers to.
    pub triggers: &'static [&'static str],
}

macro_rules! module {
    ($name:literal, exports: [$($ex:expr),* $(,)?], deps: [$($d:literal),* $(,)?], triggers: [$($t:literal),* $(,)?]) => {
        Module {
            name: $name,
            source: include_str!(concat!("modules/", $name, ".js")),
            exports: &[$($ex),*],
            deps: &[$($d),*],
            triggers: &[$($t),*],
        }
    };
}

/// A name exported under itself.
const fn same(n: &'static str) -> (&'static str, &'static str) {
    (n, n)
}

/// Every module, in the order they are emitted.
pub const MODULES: &[Module] = &[
    module!("core", exports: [
        same("signal"), same("effect"), same("computed"), same("scoped"), same("onCleanup"),
        same("every"), same("after"), same("listen"), same("onKey"), same("keyIs"),
        same("ref"), same("safeUrl"), same("jsonAttr"), same("el"), same("text"), same("props"),
        same("onRoot"), same("mark"), same("classes"), same("caseOf"), same("payload"),
        same("emit"), same("navigate"), same("setSsgMode"), same("setBasePath"),
        same("theme"), same("setTheme"), same("mount"), same("mainOf"),
    ], deps: [], triggers: []),
    module!("motion", exports: [
        same("animate"), same("replay"), same("animateIn"), same("animateOut"),
        same("expand"), same("countTo"), same("counted"), same("onEnterView"), same("shared"),
    ], deps: [], triggers: []),
    module!("helpers", exports: [
        same("sortBy"), same("groupBy"), same("unique"), same("take"), same("range"),
        same("first"), same("last"), same("capitalize"), same("truncate"),
        same("dedent"), same("lines"), same("words"),
    ], deps: [], triggers: []),
    module!("format", exports: [same("format"), same("ago")], deps: [], triggers: []),
    module!("scalars", exports: [
        same("year"), same("month"), same("day"), same("weekday"), same("hour"),
        same("minute"), same("second"), same("native"), same("plus"), same("minus"),
        same("isBefore"), same("isAfter"), same("isSame"), same("until"),
        same("startOfDay"), same("startOfWeek"), same("startOfMonth"), same("endOfDay"),
        same("inZone"), same("days"), same("hours"), same("minutes"), same("seconds"),
        same("ms"), same("money"), same("times"), same("convert"), same("host"),
        same("path"), same("domain"), same("mix"), same("lighten"),
        same("darken"), same("alpha"), same("contrast"), same("preview"), same("uuid"),
        ("dateOf", "dateOf"), ("timeOf", "timeOf"),
        ("urlWith", "withParts"), ("urlQuery", "urlQuery"),
    ], deps: [], triggers: []),
    module!("form", exports: [same("form"), same("validate")], deps: ["announce"], triggers: []),
    module!("text", exports: [same("markdown"), same("highlight")],
        deps: [], triggers: ["markdown: ", "highlight: { code:"]),
    module!("when", exports: [same("when")], deps: ["motion"], triggers: []),
    module!("match", exports: [same("match")], deps: ["motion"], triggers: []),
    module!("slot", exports: [same("slot")], deps: ["motion"], triggers: []),
    module!("each", exports: [same("each")], deps: ["motion"], triggers: []),
    module!("show", exports: [same("show")], deps: ["motion"], triggers: []),
    module!("browser", exports: [
        same("viewport"), same("query"), same("hash"), same("now"), same("network"),
    ],
        deps: [], triggers: []),
    module!("router", exports: [same("router"), same("params"), same("activeLink")],
        deps: ["motion", "match", "pages", "announce"], triggers: []),
    module!("pages", exports: [same("page"), same("loadPage"), same("loadSheet")],
        deps: [], triggers: []),
    module!("host", exports: [same("attach")], deps: [], triggers: []),
    module!("sanitize", exports: [same("sanitize")], deps: [], triggers: ["markup:"]),
    module!("keep", exports: [same("persist")], deps: [], triggers: []),
    module!("store", exports: [
        same("store"), same("dropRouteStores"), same("watchStores"), same("storeSnapshot"),
        same("restoreStore"),
    ], deps: ["keep"], triggers: []),
    module!("http", exports: [
        same("send"), same("beacon"), same("invalidate"),
    ], deps: [], triggers: []),
    module!("api", exports: [
        same("api"), same("ws"), same("sse"), same("broadcast"),
    ], deps: ["http"], triggers: []),
    module!("net", exports: [
        same("resource"), same("request"), same("optimistic"), same("attempt"),
        ("fetch", "wfFetch"),
    ],
        deps: ["http"], triggers: []),
    module!("toast", exports: [same("toast")], deps: [], triggers: []),
    module!("overlay", exports: [
        same("dialog"), same("popup"), same("tabs"), same("menu"),
    ], deps: [], triggers: []),
    module!("announce", exports: [same("announce")], deps: [], triggers: []),
    module!("field", exports: [same("field")], deps: [], triggers: []),
    module!("tooltip", exports: [same("tooltip")], deps: [], triggers: []),
    module!("carousel", exports: [same("carousel")], deps: ["field"], triggers: []),
    module!("drawer", exports: [same("drawer")], deps: [], triggers: []),
    module!("head", exports: [same("head")], deps: [], triggers: []),
    module!("hydrate", exports: [same("hydrate")], deps: [], triggers: []),
    module!("i18n", exports: [same("locales")], deps: ["format"], triggers: []),
    module!("picture", exports: [same("picture")], deps: [], triggers: []),
    module!("icons", exports: [], deps: [], triggers: ["\"data-icon\""]),
    module!("debug", exports: [same("__debug"), same("__reg")], deps: [], triggers: []),
];

const HEADER: &str = "\"use strict\";\n\
// WebFluent Runtime v1.0\n\
// Fine-grained reactivity + DOM helpers + Router + Store + Fetch\n\
\n\
const WF = (() => {\n";

/// The names of the modules a program reaches: what it calls, what its
/// attributes name, and everything those need.
pub fn modules_for(program: &str) -> BTreeSet<&'static str> {
    let called = calls_in(program);
    let mut want: BTreeSet<&'static str> = BTreeSet::new();
    want.insert("core");
    for m in MODULES {
        let reached = m.exports.iter().any(|(name, _)| called.contains(name))
            || m.triggers.iter().any(|t| program.contains(t));
        if reached {
            want.insert(m.name);
        }
    }
    // Close over the dependencies until nothing new arrives.
    loop {
        let mut added = false;
        for m in MODULES {
            if !want.contains(m.name) {
                continue;
            }
            for d in m.deps {
                added |= want.insert(d);
            }
        }
        if !added {
            break;
        }
    }
    want
}

/// Every `WF.<name>` a program names.
fn calls_in(program: &str) -> BTreeSet<&str> {
    let mut out = BTreeSet::new();
    let bytes = program.as_bytes();
    let mut from = 0;
    while let Some(at) = program[from..].find("WF.") {
        let start = from + at + 3;
        // Not `WF` if it is part of a longer word.
        let before_ok = from + at == 0 || {
            let c = bytes[from + at - 1] as char;
            !(c.is_alphanumeric() || c == '_' || c == '$' || c == '.')
        };
        let end = program[start..]
            .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
            .map_or(program.len(), |n| start + n);
        if before_ok && end > start {
            out.insert(&program[start..end]);
        }
        from = start;
    }
    out
}

/// One module a build kept: what it weighs, and what brought it in.
pub struct Kept {
    pub name: &'static str,
    pub bytes: usize,
    /// The call, the attribute, or the module that reached it.
    pub reason: String,
}

/// The modules a build keeps, in the order they are emitted.
pub fn kept(program: &str, full: bool) -> Vec<&'static str> {
    report(program, full).into_iter().map(|k| k.name).collect()
}

/// What each kept module weighs and why it is there.
pub fn report(program: &str, full: bool) -> Vec<Kept> {
    let want: BTreeSet<&'static str> = if full {
        MODULES.iter().map(|m| m.name).collect()
    } else {
        modules_for(program)
    };
    let called = calls_in(program);
    MODULES
        .iter()
        .filter(|m| want.contains(m.name))
        .map(|m| {
            let reason = if full {
                "build.runtime: full".to_string()
            } else if m.name == "core" {
                "always".to_string()
            } else if let Some((name, _)) = m.exports.iter().find(|(n, _)| called.contains(n)) {
                format!("WF.{name}")
            } else if let Some(t) = m.triggers.iter().find(|t| program.contains(**t)) {
                (*t).to_string()
            } else {
                let by: Vec<&str> = MODULES
                    .iter()
                    .filter(|o| want.contains(o.name) && o.deps.contains(&m.name))
                    .map(|o| o.name)
                    .collect();
                format!("needed by {}", by.join(", "))
            };
            let bytes = if m.name == "icons" && !full {
                prune_icons(m.source, program).len()
            } else {
                m.source.len()
            };
            Kept {
                name: m.name,
                bytes,
                reason,
            }
        })
        .collect()
}

/// The modules a build left out, in the order they would have been emitted.
pub fn dropped(kept: &[&str]) -> Vec<&'static str> {
    MODULES
        .iter()
        .map(|m| m.name)
        .filter(|n| !kept.contains(n))
        .collect()
}

/// The runtime a program needs, as JavaScript. `full` ships every module.
pub fn assemble(program: &str, full: bool) -> String {
    let want = if full {
        MODULES.iter().map(|m| m.name).collect()
    } else {
        modules_for(program)
    };
    let mut out = String::with_capacity(64 * 1024);
    out.push_str(HEADER);
    for m in MODULES.iter().filter(|m| want.contains(m.name)) {
        let source = if m.name == "icons" && !full {
            std::borrow::Cow::Owned(prune_icons(m.source, program))
        } else {
            std::borrow::Cow::Borrowed(m.source)
        };
        out.push_str(&source);
        if !source.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("  return {\n");
    for m in MODULES.iter().filter(|m| want.contains(m.name)) {
        if m.exports.is_empty() {
            continue;
        }
        let line: Vec<String> = m
            .exports
            .iter()
            .map(|(name, local)| {
                if name == local {
                    (*name).to_string()
                } else {
                    format!("{name}: {local}")
                }
            })
            .collect();
        out.push_str(&format!("    {},\n", line.join(", ")));
    }
    // The base path is read by every link a static build writes.
    out.push_str("    get _basePath() { return _basePath; },\n");
    out.push_str("  };\n})();\n");
    out
}

/// The icon table, with the glyphs the program never names left out.
///
/// An icon is drawn by name, and a name is a string: `Icon("home")` puts
/// `"home"` in the bundle, and so does a name that comes from a `data` file
/// or a list the program holds. A name a build cannot see — one assembled at
/// run time, or fetched — is drawn as text instead, which is when a project
/// sets `build.runtime: "full"`.
fn prune_icons(source: &str, program: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_table = false;
    for line in source.lines() {
        if line.contains("const _ICONS = {") {
            in_table = true;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_table {
            if line.trim_start().starts_with("};") {
                in_table = false;
            } else if let Some(name) = icon_name(line) {
                let named = program.contains(&format!("\"{name}\""))
                    || program.contains(&format!("'{name}'"));
                if !named {
                    continue;
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// The key of one line of the icon table: `close: '…'` or `"arrow-left": '…'`.
fn icon_name(line: &str) -> Option<&str> {
    let key = line.trim_start().split(':').next()?.trim();
    let key = key.trim_matches('"');
    (!key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase() || c == '-')).then_some(key)
}

/// Every module, for a caller that wants the whole runtime.
pub fn full() -> String {
    assemble("", true)
}

/// The manifest as JSON, so the JavaScript tests assemble the runtime the
/// same way this does. `modules/manifest.json` is written from it, and a
/// test here fails when the file is stale.
pub fn manifest_json() -> String {
    let modules: Vec<serde_json::Value> = MODULES
        .iter()
        .map(|m| {
            serde_json::json!({
                "name": m.name,
                "exports": m.exports.iter().map(|(n, l)| vec![*n, *l]).collect::<Vec<_>>(),
                "deps": m.deps,
                "triggers": m.triggers,
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({ "header": HEADER, "modules": modules }))
        .expect("the manifest is JSON")
        + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every module declares what it exports, and declares it once.
    #[test]
    fn what_a_module_exports_it_defines() {
        let mut seen: Vec<&str> = Vec::new();
        for m in MODULES {
            for (name, local) in m.exports {
                assert!(
                    m.source.contains(&format!("function {local}("))
                        || m.source.contains(&format!("function* {local}("))
                        || m.source.contains(&format!("const {local} "))
                        || m.source.contains(&format!("let {local} ")),
                    "{}: `{local}` is exported but not defined there",
                    m.name
                );
                assert!(!seen.contains(name), "`{name}` is exported twice");
                seen.push(name);
            }
        }
    }

    /// A module names only modules that exist, and nothing names itself.
    #[test]
    fn the_dependencies_resolve() {
        for m in MODULES {
            for d in m.deps {
                assert_ne!(*d, m.name, "{} depends on itself", m.name);
                assert!(
                    MODULES.iter().any(|o| o.name == *d),
                    "{} depends on `{d}`, which is not a module",
                    m.name
                );
            }
        }
    }

    /// A program that calls into a module gets it, and its dependencies.
    #[test]
    fn what_a_program_reaches_comes_with_what_it_needs() {
        let want = modules_for("WF.each(p, l, i);");
        assert!(want.contains("each"));
        assert!(
            want.contains("motion"),
            "`each` plays enter and exit animations"
        );
        assert!(want.contains("core"));
        assert!(!want.contains("router"), "one list is not a router");
        assert!(!want.contains("carousel"));
        // A router brings the pieces a route change uses.
        let want = modules_for("WF.router(routes, c);");
        for m in ["router", "pages", "announce", "match", "motion", "core"] {
            assert!(want.contains(m), "a router needs `{m}`");
        }
        // An attribute reaches a module that exports nothing.
        assert!(modules_for("WF.el(\"i\", { \"data-icon\": \"home\" })").contains("icons"));
        assert!(!modules_for("WF.el(\"i\", {})").contains("icons"));
        // A name that merely ends in WF is not a call.
        assert!(!modules_for("myWF.each()").contains("each"));
    }

    /// The smallest page carries the smallest runtime, and `full` carries it all.
    #[test]
    fn the_runtime_is_only_as_large_as_the_program_needs() {
        let small = assemble("WF.mount(() => WF.el(\"p\", {}), c);", false);
        let all = full();
        assert!(
            small.len() * 3 < all.len(),
            "a bare page must not ship the lot"
        );
        assert!(small.contains("function mount("));
        assert!(!small.contains("function carousel("));
        assert!(all.contains("function carousel("));
        // Both are one closure, and both export what they hold.
        for js in [&small, &all] {
            assert!(js.starts_with("\"use strict\";"));
            assert!(js.ends_with("  };\n})();\n"));
            assert!(js.contains("    get _basePath() { return _basePath; },"));
        }
    }

    /// A build carries the glyphs it names, not the whole sheet.
    #[test]
    fn the_icon_table_holds_what_the_program_names() {
        let js = assemble("WF.el(\"i\", { \"data-icon\": \"home\" })", false);
        assert!(js.contains("const _ICONS = {"));
        assert!(js.contains("\n    home:"), "the icon it names");
        assert!(!js.contains("\n    \"arrow-left\":"), "one it does not");
        assert!(js.contains("function _renderIcon("));
        // `full` keeps the sheet whole.
        assert!(full().contains("\n    \"arrow-left\":"));
    }

    /// The JavaScript tests read `modules/manifest.json`; it is written here.
    #[test]
    fn the_manifest_json_is_current() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/runtime/modules/manifest.json"
        );
        let want = manifest_json();
        let have = std::fs::read_to_string(path).unwrap_or_default();
        if have != want {
            std::fs::write(path, &want).expect("write the manifest");
            panic!("src/runtime/modules/manifest.json was stale; it has been rewritten");
        }
    }
}
