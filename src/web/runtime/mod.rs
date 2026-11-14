//! JavaScript runtime — embedded runtime for reactivity, routing, and DOM helpers.
//!
//! The runtime provides signal-based reactivity, conditional/list rendering,
//! client-side routing, store management, i18n, animations, and toast notifications.
//! It is a set of feature modules, and a build ships only the ones its program
//! reaches ([`assemble`]); [`full`] is every module at once.
//!
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

mod locals;

/// One feature of the runtime: a fragment of the closure, what it puts on
/// `WF`, what it needs, and what in a program reaches it.
#[derive(Debug)]
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
        same("every"), same("after"), same("listen"), same("allowSchemes"),
        same("ref"), same("safeUrl"), same("safePicture"), same("jsonAttr"), same("el"), same("bound"), same("text"), same("props"),
        same("onRoot"), same("mark"), same("rootAttrs"), same("classes"), same("caseOf"), same("payload"),
        same("emit"), same("navigate"), same("setSsgMode"), same("setBasePath"),
        same("mount"), same("mainOf"), same("live"),
    ], deps: [], triggers: []),
    // What a branch, a list and a route need to put nodes in and take them
    // out; they ship it, and motion only when something animates.
    // The reader's light or dark choice: `theme`, `setTheme`, applied before
    // a page is drawn.
    module!("theme", exports: [same("theme"), same("setTheme")], deps: [], triggers: []),
    // Key combinations: `on key("mod+k")`, `keyIs`, `keyLabel`. Listening
    // starts when the runtime loads, so a chord hears its first step.
    module!("keys", exports: [same("onKey"), same("keyIs"), same("keyLabel"), same("keyAria")],
        deps: [], triggers: []),
    // A group the arrow keys move through: tabs, menus, a carousel's
    // dots, and any element given `roving:`.
    module!("focus", exports: [same("roving")], deps: [], triggers: []),
    module!("nodes", exports: [], deps: [], triggers: []),
    // Reached by a call (`WF.counted`, `WF.animate` …) or by what the code
    // generator writes when something animates: an animation's config
    // (`enter:`, `exit:`), a mark on an element (`data-wf-exit`,
    // `data-wf-animate`) or a router's `transition:`.
    module!("motion", exports: [
        same("animate"), same("replay"), same("animateIn"), same("animateOut"),
        same("expand"), same("countTo"), same("counted"), same("onEnterView"), same("shared"),
    ], deps: ["nodes"], triggers: ["enter:", "exit:", "data-wf-exit", "data-wf-animate", "transition:"]),
    module!("helpers", exports: [
        same("sortBy"), same("groupBy"), same("unique"), same("take"), same("removeAt"), same("setIn"), same("mutated"),
        same("range"),
        same("first"), same("last"), same("capitalize"), same("truncate"),
        same("dedent"), same("lines"), same("words"),
    ], deps: [], triggers: []),
    module!("format", exports: [same("format"), same("ago")], deps: [], triggers: []),
    module!("offline", exports: [same("offline"), same("update"), same("clearQueued")], deps: [], triggers: []),
    module!("peer", exports: [same("rtc")], deps: [], triggers: []),
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
    module!("form", exports: [same("form"), same("validate"), same("askRules")], deps: ["announce"], triggers: []),
    module!("text", exports: [same("markdown"), same("highlight")],
        deps: [], triggers: ["markdown: ", "highlight: { code:"]),
    module!("when", exports: [same("when"), same("held")], deps: ["nodes"], triggers: []),
    module!("match", exports: [same("match")], deps: ["nodes"], triggers: []),
    module!("slot", exports: [same("slot")], deps: ["nodes"], triggers: []),
    module!("each", exports: [same("each")], deps: ["nodes"], triggers: []),
    module!("show", exports: [same("show")], deps: ["nodes"], triggers: []),
    module!("browser", exports: [
        same("viewport"), same("query"), same("hash"), same("now"), same("network"),
    ],
        deps: [], triggers: []),
    module!("router", exports: [same("router"), same("params"), same("activeLink"), same("playRoute")],
        deps: ["nodes", "match", "pages", "announce"], triggers: []),
    module!("pages", exports: [same("page"), same("loadPage"), same("loadSheet")],
        deps: [], triggers: []),
    module!("host", exports: [same("attach"), same("portal")], deps: [], triggers: []),
    module!("sanitize", exports: [same("sanitize")], deps: [], triggers: ["markup:", "html:"]),
    module!("keep", exports: [same("persist")], deps: [], triggers: []),
    module!("store", exports: [
        same("store"), same("dropRouteStores"), same("watchStores"), same("storeSnapshot"),
        same("restoreStore"),
    ], deps: ["keep", "busy"], triggers: []),
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
    module!("toast", exports: [same("toast"), same("toastPromise")], deps: ["announce"], triggers: []),
    module!("overlay", exports: [
        same("dialog"), same("popup"), same("tabs"), same("ask"),
    ], deps: ["focus"], triggers: []),
    module!("sheet", exports: [same("sheet")], deps: ["overlay"], triggers: []),
    module!("worker", exports: [same("worker")], deps: [], triggers: []),
    module!("windowed", exports: [same("windowed")], deps: ["nodes"], triggers: []),
    module!("gesture", exports: [
        same("gesture"), same("draggable"), same("dropTarget"), same("splitter"),
    ], deps: [], triggers: []),
    module!("layer", exports: [
        same("layer"), same("placeAt"), same("menu"), same("submenu"), same("tooltip"),
        same("trapFocus"), same("uid"),
    ], deps: ["keys", "focus"], triggers: []),
    module!("announce", exports: [same("announce")], deps: [], triggers: []),
    module!("busy", exports: [same("sameCall"), same("busy")], deps: [], triggers: []),
    module!("pace", exports: [same("debounce"), same("throttle")], deps: [], triggers: []),
    module!("content", exports: [
        same("copyButton"), same("themeToggle"), same("localeSwitch"), same("fieldFor"),
        same("inputClear"), same("onThisPage"),
    ], deps: ["theme", "announce"], triggers: []),
    module!("field", exports: [same("field"), same("choiceError")], deps: [], triggers: []),
    module!("carousel", exports: [same("carousel")], deps: ["field", "focus"], triggers: []),
    module!("drawer", exports: [same("drawer")], deps: ["layer"], triggers: []),
    module!("head", exports: [same("head")], deps: [], triggers: []),
    module!("hydrate", exports: [same("hydrate"), same("hydrating")], deps: [], triggers: []),
    module!("i18n", exports: [same("locales")], deps: ["format"], triggers: []),
    module!("picture", exports: [same("picture")], deps: [], triggers: []),
    module!("later", exports: [same("loadAfter")], deps: [], triggers: []),
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
#[derive(Debug)]
pub struct Kept {
    pub name: &'static str,
    /// What of the module the build keeps, before minifying: the functions
    /// the program reaches, not the whole module.
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
    // What of each module the build keeps: its declarations the program
    // reaches, and its statements.
    let mut kept_bytes: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    if !full {
        let mut body = String::new();
        let mut owner: Vec<(usize, &'static str)> = Vec::new();
        for m in MODULES.iter().filter(|m| want.contains(m.name)) {
            owner.push((body.len(), m.name));
            if m.name == "icons" {
                body.push_str(&prune_icons(m.source, program));
            } else {
                body.push_str(m.source);
            }
            if !body.ends_with('\n') {
                body.push('\n');
            }
        }
        let mut roots: BTreeSet<&str> = MODULES
            .iter()
            .filter(|m| want.contains(m.name))
            .flat_map(|m| m.exports.iter())
            .filter(|(name, _)| called.contains(name))
            .map(|(_, local)| *local)
            .collect();
        roots.insert("_basePath");
        let items = items(&body);
        for (item, keep) in items.iter().zip(reached(&items, &roots)) {
            if !keep {
                continue;
            }
            // By its first line of code: a comment run before it may still
            // be the end of the module before.
            let lead = item
                .text
                .split_inclusive('\n')
                .take_while(|l| l.trim().is_empty() || l.trim_start().starts_with("//"))
                .map(str::len)
                .sum::<usize>();
            let at = item.text.as_ptr() as usize - body.as_ptr() as usize + lead;
            let name = owner
                .iter()
                .rev()
                .find(|(start, _)| *start <= at)
                .map_or("core", |(_, name)| name);
            *kept_bytes.entry(name).or_default() += item.text.len();
        }
    }
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
            let bytes = if full {
                m.source.len()
            } else {
                kept_bytes.get(m.name).copied().unwrap_or(0)
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
    assemble_with(program, full, false)
}

/// [`assemble`], with the runtime's own names shortened when `mangle`
/// (a minified build that is not the dev server's): `_watchListeners` is
/// `Bq`. What a program reads, `WF.<name>`, is as it was.
pub fn assemble_with(program: &str, full: bool, mangle: bool) -> String {
    let want = if full {
        MODULES.iter().map(|m| m.name).collect()
    } else {
        modules_for(program)
    };
    let mut body = String::with_capacity(64 * 1024);
    for m in MODULES.iter().filter(|m| want.contains(m.name)) {
        let source = if m.name == "icons" && !full {
            std::borrow::Cow::Owned(prune_icons(m.source, program))
        } else {
            std::borrow::Cow::Borrowed(m.source)
        };
        body.push_str(&source);
        if !source.ends_with('\n') {
            body.push('\n');
        }
    }
    // What goes on `WF`: every export of the modules shipped, or, pruned,
    // the ones the program calls.
    let called = calls_in(program);
    let exports: Vec<(&str, &str)> = MODULES
        .iter()
        .filter(|m| want.contains(m.name))
        .flat_map(|m| m.exports.iter().copied())
        .filter(|(name, _)| full || called.contains(name))
        .collect();
    if !full {
        let mut roots: BTreeSet<&str> = exports.iter().map(|(_, local)| *local).collect();
        roots.insert("_basePath");
        body = keep_reached(&body, &roots);
    }
    let renamed = if mangle {
        let (short, renamed) = mangle_names(&body, program);
        // Then each item's own names. Every top-level name counts as one,
        // as the body now spells it, shipped or not: a `typeof` may name
        // one that was left out.
        let top: std::collections::HashSet<&str> = MODULES
            .iter()
            .flat_map(|m| items(m.source))
            .filter_map(|item| item.declares)
            .map(|name| renamed.get(name).map_or(name, String::as_str))
            .collect();
        body = locals::rename_locals(&short, &top);
        renamed
    } else {
        std::collections::HashMap::new()
    };
    let local = |name: &str| renamed.get(name).map_or(name, String::as_str).to_string();
    let mut out = String::with_capacity(body.len() + 4096);
    out.push_str(HEADER);
    out.push_str(&body);
    out.push_str("  return {\n");
    for chunk in exports.chunks(6) {
        let line: Vec<String> = chunk
            .iter()
            .map(|(name, inside)| {
                let inside = local(inside);
                if *name == inside {
                    (*name).to_string()
                } else {
                    format!("{name}: {inside}")
                }
            })
            .collect();
        out.push_str(&format!("    {},\n", line.join(", ")));
    }
    // The base path is read by every link a static build writes.
    out.push_str(&format!(
        "    get _basePath() {{ return {}; }},\n",
        local("_basePath")
    ));
    out.push_str("  };\n})();\n");
    out
}

/// Whether the `(` after `at` closes on a `{` — a method's body, as in
/// `{ name(a) { … } }` — rather than on what follows a call.
fn method_follows(bytes: &[u8], at: usize) -> bool {
    let Some(open) = bytes[at..].iter().position(|&c| c == b'(').map(|n| at + n) else {
        return false;
    };
    let mut depth = 0usize;
    for (i, &c) in bytes.iter().enumerate().skip(open) {
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return bytes[i + 1..]
                        .iter()
                        .find(|c| !c.is_ascii_whitespace())
                        .is_some_and(|&c| c == b'{');
                }
            }
            _ => {}
        }
    }
    false
}

/// What a shortened name may start with, and go on with.
const FIRST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$";
const REST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$0123456789";

/// The words JavaScript reserves, which no shortened name may be.
const RESERVED: &[&str] = &[
    "do",
    "if",
    "in",
    "for",
    "let",
    "new",
    "try",
    "var",
    "case",
    "else",
    "enum",
    "eval",
    "null",
    "this",
    "true",
    "void",
    "with",
    "await",
    "break",
    "catch",
    "class",
    "const",
    "false",
    "super",
    "throw",
    "while",
    "yield",
    "delete",
    "export",
    "import",
    "public",
    "return",
    "static",
    "switch",
    "typeof",
    "default",
    "extends",
    "finally",
    "package",
    "private",
    "continue",
    "debugger",
    "function",
    "arguments",
    "interface",
    "protected",
    "implements",
    "instanceof",
    "of",
    "as",
    "get",
    "set",
    "NaN",
    "Infinity",
    "undefined",
];

/// `body` with its top-level names shortened, and what each became.
///
/// A name is renamed wherever it is code (never in a string, a comment, a
/// pattern, or after a `.`), so a local that shares it is renamed with it
/// and still means what it did. A name that is ever written where it could
/// be an object's key — `{ name: …}`, `{ name }`, `{ name() {} }` — is left
/// as it is, since renaming it there would rename the key. A new name
/// clashes with nothing the runtime or the program names.
fn mangle_names(body: &str, program: &str) -> (String, std::collections::HashMap<String, String>) {
    use std::collections::{HashMap, HashSet};
    let (spans, code) = scan(body);
    let declared: HashSet<&str> = items(body).iter().filter_map(|i| i.declares).collect();
    // Neighbours are read where the code is: a comment between `{` and a
    // method's name is not what comes before it.
    let bytes = code.as_slice();
    let significant_before = |at: usize| {
        bytes[..at]
            .iter()
            .copied()
            .rev()
            .find(|c| !c.is_ascii_whitespace())
            .unwrap_or(b'\n')
    };
    let significant_after = |at: usize| {
        bytes[at..]
            .iter()
            .copied()
            .find(|c| !c.is_ascii_whitespace())
            .unwrap_or(b'\n')
    };
    let word_before = |at: usize| {
        let trimmed = std::str::from_utf8(&bytes[..at]).unwrap_or("").trim_end();
        let start = trimmed
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
            .map_or(0, |n| n + 1);
        &trimmed[start..]
    };
    let mut unsafe_names: HashSet<&str> = HashSet::new();
    let mut uses: HashMap<&str, usize> = HashMap::new();
    let mut taken: HashSet<&str> = HashSet::new();
    for &(start, end) in &spans {
        let word = &body[start..end];
        taken.insert(word);
        if !declared.contains(word) {
            continue;
        }
        *uses.entry(word).or_default() += 1;
        let before = significant_before(start);
        let after = significant_after(end);
        // `{ name: …`, `{ name, …`, `…, name }`, and `{ name(…) {` — a key,
        // a shorthand or a method; a call, `{ name(…);`, is none of them.
        let key_place = matches!(before, b'{' | b',')
            && match after {
                b':' | b',' | b'}' => true,
                b'(' => method_follows(bytes, end),
                _ => false,
            };
        let accessor = matches!(word_before(start), "get" | "set" | "static" | "async");
        if key_place || accessor {
            unsafe_names.insert(word);
        }
    }
    // What the program names, so a short name never hides one of its own.
    for (start, end) in identifiers(program) {
        taken.insert(&program[start..end]);
    }
    let mut order: Vec<(&str, usize)> = uses
        .into_iter()
        .filter(|(name, _)| !unsafe_names.contains(name) && name.len() > 2)
        .collect();
    order.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let mut counter = 0usize;
    let mut next_name = || loop {
        let mut n = counter;
        counter += 1;
        let mut name = String::new();
        name.push(char::from(FIRST[n % FIRST.len()]));
        n /= FIRST.len();
        while n > 0 {
            n -= 1;
            name.push(char::from(REST[n % REST.len()]));
            n /= REST.len();
        }
        if !taken.contains(name.as_str()) && !RESERVED.contains(&name.as_str()) {
            return name;
        }
    };
    let renamed: HashMap<String, String> = order
        .into_iter()
        .map(|(name, _)| (name.to_string(), next_name()))
        .collect();
    let mut out = String::with_capacity(body.len());
    let mut at = 0;
    for (start, end) in spans {
        if let Some(short) = renamed.get(&body[start..end]) {
            out.push_str(&body[at..start]);
            out.push_str(short);
            at = end;
        }
    }
    out.push_str(&body[at..]);
    (out, renamed)
}

/// One top-level item of the runtime's closure: a declaration, with the
/// comment above it, or a statement that runs when the runtime loads.
struct Item<'a> {
    text: &'a str,
    /// The name a declaration declares; `None` for a statement, which is
    /// always kept.
    declares: Option<&'a str>,
}

/// The runtime's closure split into its top-level items. A module is
/// written at two spaces of indentation: an item starts on a line indented
/// exactly that far that does not close a bracket, and a comment line
/// belongs to the item after it.
fn items(body: &str) -> Vec<Item<'_>> {
    let mut starts: Vec<usize> = Vec::new();
    let mut comment_run: Option<usize> = None;
    let mut at = 0;
    for line in body.split_inclusive('\n') {
        let top = line.starts_with("  ") && !line[2..].starts_with([' ', '\t', '\n', '\r']);
        let comment = top && line[2..].starts_with("//");
        if comment {
            comment_run.get_or_insert(at);
        } else if top && !line[2..].starts_with(['}', ')', ']']) {
            starts.push(comment_run.take().unwrap_or(at));
        } else if line.trim().is_empty() {
            // A blank line keeps a comment run open.
        } else {
            comment_run = None;
        }
        at += line.len();
    }
    if starts.first() != Some(&0) {
        starts.insert(0, 0);
    }
    let mut out = Vec::with_capacity(starts.len());
    for (k, &start) in starts.iter().enumerate() {
        let end = starts.get(k + 1).copied().unwrap_or(body.len());
        let text = &body[start..end];
        let code = text
            .lines()
            .find(|l| l.starts_with("  ") && !l[2..].starts_with("//") && !l.trim().is_empty())
            .unwrap_or("");
        out.push(Item {
            text,
            declares: declared_name(code),
        });
    }
    out
}

/// The name a top-level line declares: `function name`, `async function
/// name`, `const name`, `let name`, `class name`.
fn declared_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("  ")?;
    let rest = rest.strip_prefix("async ").unwrap_or(rest);
    let rest = ["function* ", "function ", "const ", "let ", "class "]
        .iter()
        .find_map(|k| rest.strip_prefix(k))?;
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
        .unwrap_or(rest.len());
    (end > 0).then(|| &rest[..end])
}

/// Every identifier `text` names as code: not in a comment, not in a
/// string (a closure's names cannot be reached through one), and not a
/// property name (`x.name`, though `...name` is). A template literal's
/// `${…}` is code, and a regular expression is skipped as a whole.
fn names_in(text: &str) -> BTreeSet<&str> {
    identifiers(text)
        .into_iter()
        .map(|(a, b)| &text[a..b])
        .collect()
}

/// Where each name [`names_in`] reads sits in `text`, as byte ranges, in
/// order.
fn identifiers(text: &str) -> Vec<(usize, usize)> {
    scan(text).0
}

/// [`identifiers`], and `text` with its comments, and the insides of its
/// strings, patterns and template text, made spaces: where the code is.
fn scan(text: &str) -> (Vec<(usize, usize)>, Vec<u8>) {
    let bytes = text.as_bytes();
    let mut code = bytes.to_vec();
    let mut blank = |from: usize, to: usize| {
        for b in &mut code[from.min(bytes.len())..to.min(bytes.len())] {
            *b = b' ';
        }
    };
    let mut out = Vec::new();
    // The template literals open around the scan, each with the depth of
    // `{` inside its current `${`.
    let mut templates: Vec<usize> = Vec::new();
    let mut in_template_text = false;
    let mut last_code = b'\n';
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if in_template_text {
            match c {
                b'\\' => i += 2,
                b'`' => {
                    in_template_text = false;
                    templates.pop();
                    last_code = b'`';
                    i += 1;
                }
                b'$' if bytes.get(i + 1) == Some(&b'{') => {
                    in_template_text = false;
                    if let Some(depth) = templates.last_mut() {
                        *depth = 0;
                    }
                    last_code = b'{';
                    i += 2;
                }
                _ => {
                    blank(i, i + 1);
                    i += 1;
                }
            }
            continue;
        }
        match c {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                let to = text[i..].find('\n').map_or(bytes.len(), |n| i + n);
                blank(i, to);
                i = to;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let to = text[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |n| i + 2 + n + 2);
                blank(i, to);
                i = to;
            }
            // A regular expression where an expression may start.
            b'/' if matches!(
                last_code,
                b'(' | b','
                    | b'='
                    // `x => /…/`
                    | b'>'
                    | b':'
                    | b'['
                    | b'!'
                    | b'&'
                    | b'|'
                    | b'?'
                    | b'{'
                    | b'}'
                    | b';'
                    | b'\n'
            ) =>
            {
                let from = i + 1;
                i += 1;
                let mut in_class = false;
                while i < bytes.len() && bytes[i] != b'\n' {
                    match bytes[i] {
                        b'\\' => i += 1,
                        b'[' => in_class = true,
                        b']' => in_class = false,
                        b'/' if !in_class => break,
                        _ => {}
                    }
                    i += 1;
                }
                blank(from, i);
                i += 1;
                // The flags are the pattern's, not a name: `/x/gi`.
                while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                    i += 1;
                }
                last_code = b'/';
            }
            b'"' | b'\'' => {
                i += 1;
                let from = i;
                while i < bytes.len() && bytes[i] != c && bytes[i] != b'\n' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                blank(from, i);
                i += 1;
                last_code = b'"';
            }
            b'`' => {
                templates.push(0);
                in_template_text = true;
                i += 1;
            }
            b'{' => {
                if let Some(depth) = templates.last_mut() {
                    *depth += 1;
                }
                last_code = c;
                i += 1;
            }
            b'}' => {
                match templates.last_mut() {
                    // The `}` that closes a template's `${`.
                    Some(0) => in_template_text = true,
                    Some(depth) => *depth -= 1,
                    None => {}
                }
                last_code = c;
                i += 1;
            }
            // A number, whole: the `e3` of `1e3` and the `x1f` of `0x1f` are
            // not names.
            c if c.is_ascii_digit() => {
                let hex = bytes.get(i + 1).is_some_and(|b| matches!(b, b'x' | b'X'));
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.')
                {
                    // `1e-3`: the sign belongs to the exponent.
                    if !hex
                        && matches!(bytes[i], b'e' | b'E')
                        && matches!(bytes.get(i + 1), Some(b'-' | b'+'))
                    {
                        i += 1;
                    }
                    i += 1;
                }
                last_code = b'0';
            }
            c if c.is_ascii_alphabetic() || c == b'_' || c == b'$' => {
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
                {
                    i += 1;
                }
                let before = &text[..start];
                if !before.ends_with('.') || before.ends_with("...") {
                    out.push((start, i));
                }
                // After a word an expression may start (`return /x/`) or
                // not (`a / b`); `return`, `typeof` and the like are the
                // ones that start one.
                last_code = if matches!(
                    &text[start..i],
                    "return"
                        | "typeof"
                        | "case"
                        | "in"
                        | "of"
                        | "delete"
                        | "void"
                        | "throw"
                        | "new"
                        | "yield"
                        | "await"
                ) {
                    b'('
                } else {
                    b'a'
                };
            }
            c if c.is_ascii_whitespace() => {
                if c == b'\n' && last_code != b'a' && last_code != b')' && last_code != b']' {
                    last_code = b'\n';
                }
                i += 1;
            }
            _ => {
                last_code = c;
                i += 1;
            }
        }
    }
    (out, code)
}

/// `body` with every declaration nothing reaches left out. What runs at
/// load is kept, and so is every declaration `roots` names, and every one
/// those name, and so on.
fn keep_reached(body: &str, roots: &BTreeSet<&str>) -> String {
    let items = items(body);
    items
        .iter()
        .zip(reached(&items, roots))
        .filter(|(_, k)| *k)
        .map(|(item, _)| item.text)
        .collect()
}

/// Which of `items` a build keeps: every statement, the `roots`, and what
/// those name, transitively.
fn reached(items: &[Item<'_>], roots: &BTreeSet<&str>) -> Vec<bool> {
    let mut by_name: std::collections::HashMap<&str, Vec<usize>> = std::collections::HashMap::new();
    for (ix, item) in items.iter().enumerate() {
        if let Some(name) = item.declares {
            by_name.entry(name).or_default().push(ix);
        }
    }
    let mut keep = vec![false; items.len()];
    let mut queue: Vec<usize> = Vec::new();
    let reach = |name: &str, keep: &mut Vec<bool>, queue: &mut Vec<usize>| {
        for &ix in by_name.get(name).map_or(&[][..], Vec::as_slice) {
            if !keep[ix] {
                keep[ix] = true;
                queue.push(ix);
            }
        }
    };
    for (ix, item) in items.iter().enumerate() {
        if item.declares.is_none() {
            keep[ix] = true;
            queue.push(ix);
        }
    }
    for root in roots {
        reach(root, &mut keep, &mut queue);
    }
    while let Some(ix) = queue.pop() {
        for name in names_in(items[ix].text) {
            reach(name, &mut keep, &mut queue);
        }
    }
    keep
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

/// [`full`], with its names shortened as a minified build shortens them.
#[doc(hidden)]
pub fn full_mangled() -> String {
    assemble_with("", true, true)
}

/// The manifest as JSON, so the JavaScript tests assemble the runtime the
/// same way this does. `modules/manifest.json` is written from it, and a
/// test here fails when the file is stale.
///
/// # Panics
///
/// Never: the manifest is strings and lists of them, which always
/// serialise.
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
    format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({ "header": HEADER, "modules": modules }))
            .expect("the manifest is JSON")
    )
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
            want.contains("nodes"),
            "a list puts nodes in and takes them out"
        );
        assert!(
            !want.contains("motion"),
            "a list that animates nothing carries no animations"
        );
        assert!(want.contains("core"));
        assert!(!want.contains("router"), "one list is not a router");
        assert!(!want.contains("carousel"));
        // A list, a branch or a route that animates brings motion with it.
        for animated in [
            "WF.each(p, l, i, { enter: \"fadeIn\" });",
            "WF.when(p, c, t, e, { exit: \"fadeOut\" });",
            "WF.mark(n, { \"data-wf-exit\": \"fadeOut\" });",
            "WF.mark(n, { \"data-wf-animate\": \"fadeIn\" });",
            "WF.router(routes, c, { transition: \"fade\", play: WF.playRoute });",
            "WF.counted(n, v, \"600ms\");",
        ] {
            assert!(modules_for(animated).contains("motion"), "{animated}");
        }
        // A router brings the pieces a route change uses.
        let want = modules_for("WF.router(routes, c);");
        for m in ["router", "pages", "announce", "match", "nodes", "core"] {
            assert!(want.contains(m), "a router needs `{m}`");
        }
        assert!(
            !want.contains("motion"),
            "a route change without a transition plays nothing"
        );
        // An attribute reaches a module that exports nothing.
        assert!(modules_for("WF.el(\"i\", { \"data-icon\": \"home\" })").contains("icons"));
        assert!(!modules_for("WF.el(\"i\", {})").contains("icons"));
        // A name that merely ends in WF is not a call.
        assert!(!modules_for("myWF.each()").contains("each"));
    }

    /// The runtime is cut down to the functions the program reaches: what
    /// it calls, what those call, and everything that runs at load.
    #[test]
    fn only_the_functions_a_program_reaches_are_shipped() {
        let body = "  // The first.\n  function first() { return second(); }\n\n  function second() { return 1; }\n  // Nothing calls this; `first` in a comment does not keep it.\n  function unused() { return 2; }\n  const table = { a: 1 };\n  register(() => table);\n";
        let roots: BTreeSet<&str> = ["first"].into_iter().collect();
        let kept = keep_reached(body, &roots);
        assert!(kept.contains("function first"), "{kept}");
        assert!(
            kept.contains("function second"),
            "what a kept function calls is kept"
        );
        assert!(
            kept.contains("register(() => table);"),
            "what runs at load is kept"
        );
        assert!(kept.contains("const table"), "and what it names");
        assert!(!kept.contains("function unused"), "{kept}");
        assert!(
            kept.contains("// The first."),
            "a comment goes with its item"
        );
    }

    /// Every export, called on its own, makes a runtime whose names are all
    /// declared: nothing a kept function reaches was cut.
    #[test]
    fn every_export_alone_reaches_everything_it_needs() {
        for m in MODULES {
            for (name, _) in m.exports {
                let program = format!("WF.{name}();");
                // What the shipped modules declare, before any is cut.
                let shipped: String = MODULES
                    .iter()
                    .filter(|m| modules_for(&program).contains(m.name))
                    .map(|m| m.source)
                    .collect();
                let declared: BTreeSet<&str> =
                    items(&shipped).iter().filter_map(|i| i.declares).collect();
                let js = assemble(&program, false);
                let kept: BTreeSet<&str> = items(&js).iter().filter_map(|i| i.declares).collect();
                for item in items(&js) {
                    for word in names_in(item.text) {
                        assert!(
                            !declared.contains(word) || kept.contains(word),
                            "`WF.{name}` reaches `{word}`, which was cut"
                        );
                    }
                }
            }
        }
    }

    /// The scan skips strings, comments and regular expressions, and
    /// nothing else: every word it does not report appears in one of them
    /// (or after a `.`), so a mistaken skip cannot cut a function a page
    /// needs.
    #[test]
    fn mangling_shortens_names_but_never_a_key() {
        let body = "  function counter(start) { return { counter: start, label() { return 1; } }; }\n\
                    \x20 function helper(value) { if (value) { helper(value - 1); } return counter(value); }\n\
                    \x20 function label() { return \"helper\"; }\n\
                    \x20 const options = { label };\n\
                    \x20 function send(v) { return { state: 1,\n    /// Send it.\n    send(value) { return v; } }; }\n\
                    \x20 register(() => helper(options));\n";
        let (short, renamed) = mangle_names(body, "WF.helper()");
        // `helper` is code everywhere it is written: renamed, but not in the string.
        let new = &renamed["helper"];
        assert!(short.contains(&format!("function {new}(value)")), "{short}");
        assert!(
            short.contains(&format!("{{ {new}(value - 1); }}")),
            "{short}"
        );
        assert!(short.contains("return \"helper\""), "{short}");
        // `counter` is a key, `label` a method and a shorthand: left alone.
        assert!(!renamed.contains_key("counter"), "{renamed:?}");
        assert!(!renamed.contains_key("label"), "{renamed:?}");
        // A method with a comment above it is a method still.
        assert!(!renamed.contains_key("send"), "{renamed:?}");
        // A new name is nothing the body or the program already names.
        for name in renamed.values() {
            assert!(!body.contains(&format!(" {name}(")), "{name}");
            assert!(!RESERVED.contains(&name.as_str()));
        }
    }

    #[test]
    fn the_scan_skips_only_strings_comments_and_patterns() {
        for m in MODULES {
            for item in items(m.source) {
                let found = names_in(item.text);
                let mut from = 0;
                let text = item.text;
                while let Some(at) =
                    text[from..].find(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '$')
                {
                    let start = from + at;
                    let end = text[start..]
                        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
                        .map_or(text.len(), |n| start + n);
                    let word = &text[start..end];
                    from = end;
                    if found.contains(word) || text[..start].ends_with('.') {
                        continue;
                    }
                    let line_start = text[..start].rfind('\n').map_or(0, |n| n + 1);
                    let line = &text
                        [line_start..text[start..].find('\n').map_or(text.len(), |n| start + n)];
                    let col = start - line_start;
                    let before = &line[..col];
                    // Whether the word sits inside a quote on its line, escapes
                    // and a pattern's `[…]` honoured.
                    let mut open: Option<char> = None;
                    let mut escaped = false;
                    let mut in_class = false;
                    for ch in before.chars() {
                        if escaped {
                            escaped = false;
                        } else if ch == '\\' {
                            escaped = true;
                        } else if open == Some('/') && ch == '[' {
                            in_class = true;
                        } else if open == Some('/') && ch == ']' {
                            in_class = false;
                        } else if open.is_none() && matches!(ch, '"' | '\'' | '`' | '/') {
                            open = Some(ch);
                        } else if open == Some(ch) && !in_class {
                            open = None;
                        }
                    }
                    // A pattern's flags, `/…/gi`, and the letters of a
                    // number, `1e3`, are not names either.
                    let flags =
                        before.ends_with('/') && word.bytes().all(|c| b"dgimsuyv".contains(&c));
                    let in_number = before
                        .bytes()
                        .last()
                        .is_some_and(|c| c.is_ascii_digit() || c == b'.');
                    let quoted = open.is_some()
                        || flags
                        || in_number
                        || line.trim_start().starts_with("//")
                        || line.trim_start().starts_with('*')
                        || line.trim_start().starts_with("/*")
                        || before.contains("//");
                    assert!(
                        quoted,
                        "`{word}` in {} was skipped outside a string, comment or pattern:\n{line}",
                        m.name
                    );
                }
            }
        }
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
            "/src/web/runtime/modules/manifest.json"
        );
        let want = manifest_json();
        let have = std::fs::read_to_string(path).unwrap_or_default();
        if have != want {
            std::fs::write(path, &want).expect("write the manifest");
            panic!("src/web/runtime/modules/manifest.json was stale; it has been rewritten");
        }
    }
}
