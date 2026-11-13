//! Short names for the runtime's local names.
//!
//! The runtime is strict-mode code with no `with` and no `eval`. So inside
//! one of its top-level items, a name that is neither one of the runtime's
//! top-level names nor a global of the browser is bound in that item: a
//! parameter, a `const`, a `let`, a function of its own. Such a name can be
//! renamed through the item as a whole. Each local name gets one new name,
//! the most used first, and no two get the same one, so a name declared in
//! two sibling blocks keeps meaning what it meant in each. A new name is
//! never one the item still uses, so it captures nothing.
//!
//! A name written where it is an object's key — `{ name: … }`, a method,
//! a getter — is a key, and stays. A shorthand, `{ name }`, is written out,
//! `{ name: a }`, so the key is kept and the value renamed.

use std::collections::{HashMap, HashSet};

use super::{FIRST, REST, items, method_follows, scan};

/// The browser's and JavaScript's own names that the runtime uses. A local
/// of one of these names is left as it is, so a global of that name
/// written elsewhere in the same item is never taken for the local. A test
/// holds this to Chrome's whole list of globals.
pub(super) const GLOBALS: &[&str] = &[
    "AbortController",
    "Array",
    "ArrayBuffer",
    "Blob",
    "Boolean",
    "BroadcastChannel",
    "CustomEvent",
    "Date",
    "DocumentFragment",
    "Element",
    "Error",
    "Event",
    "EventSource",
    "Float64Array",
    "FormData",
    "Infinity",
    "IntersectionObserver",
    "Intl",
    "JSON",
    "Map",
    "Math",
    "MutationObserver",
    "Node",
    "Number",
    "Object",
    "Promise",
    "Proxy",
    "RTCPeerConnection",
    "ReadableStream",
    "Reflect",
    "RegExp",
    "ResizeObserver",
    "Set",
    "String",
    "SyncManager",
    "TextDecoder",
    "TextEncoder",
    "URL",
    "URLSearchParams",
    "Uint8Array",
    "WeakMap",
    "WeakSet",
    "WebSocket",
    "Worker",
    "XMLHttpRequest",
    "addEventListener",
    "alert",
    "cancelAnimationFrame",
    "clearInterval",
    "clearTimeout",
    "close",
    "closed",
    "console",
    "crypto",
    "decodeURIComponent",
    "document",
    "encodeURIComponent",
    "escape",
    "event",
    "external",
    "fetch",
    "focus",
    "frames",
    "getComputedStyle",
    "globalThis",
    "indexedDB",
    "localStorage",
    "location",
    "moveTo",
    "name",
    "navigator",
    "open",
    "parent",
    "parseFloat",
    "parseInt",
    "prompt",
    "queueMicrotask",
    "requestAnimationFrame",
    "screen",
    "scroll",
    "self",
    "setInterval",
    "setTimeout",
    "status",
    "stop",
    "top",
    "undefined",
    "viewport",
    "when",
    "window",
    // What a server-side or test runner provides in place of a window.
    "process",
    "global",
    "require",
    "module",
    "Buffer",
    "sessionStorage",
];

/// Words that are never a local's name, or are one only in a way renaming
/// would break: the keywords, and the contextual words that are keywords
/// where they stand (`get`, `of`, `async`).
const WORDS: &[&str] = &[
    "arguments",
    "as",
    "async",
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
    "from",
    "function",
    "get",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "of",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "set",
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
    "NaN",
    "WF",
];

/// `body` with every item's local names shortened. `top` is every top-level
/// name of the runtime, as the body now spells it, shipped or not: a
/// `typeof` may name one that was left out.
pub(super) fn rename_locals(body: &str, top: &HashSet<&str>) -> String {
    let mut out = String::with_capacity(body.len());
    for item in items(body) {
        out.push_str(&rename_in(item.text, top));
    }
    out
}

/// How an occurrence of a local is rewritten.
#[derive(Clone, Copy, PartialEq)]
enum Use {
    /// The name, renamed.
    Plain,
    /// `{ name }`: written `name: new`.
    Shorthand,
    /// A key, a method or a getter's name: left as it is.
    Key,
}

/// An item's names, each with how it is used; how often each local is
/// used; and the names that stay.
struct Names<'a> {
    spans: Vec<(usize, usize)>,
    uses: Vec<Use>,
    count: HashMap<&'a str, usize>,
    kept: HashSet<&'a str>,
}

fn names_of<'a>(text: &'a str, top: &HashSet<&str>) -> Names<'a> {
    let (spans, code) = scan(text);
    let uses = classify(&code, &spans);
    let word = |&(a, b): &(usize, usize)| &text[a..b];
    let labels: HashSet<&str> = spans
        .windows(2)
        .filter(|w| matches!(word(&w[0]), "break" | "continue"))
        .map(|w| word(&w[1]))
        .collect();
    let local = |name: &str| {
        !top.contains(name)
            && !GLOBALS.contains(&name)
            && !WORDS.contains(&name)
            && !labels.contains(name)
    };
    let mut count: HashMap<&str, usize> = HashMap::new();
    let mut kept: HashSet<&str> = HashSet::new();
    for (span, how) in spans.iter().zip(&uses) {
        let name = word(span);
        if *how == Use::Key {
            continue;
        }
        if local(name) {
            *count.entry(name).or_default() += 1;
        } else {
            kept.insert(name);
        }
    }
    Names {
        spans,
        uses,
        count,
        kept,
    }
}

/// One item, its locals renamed.
fn rename_in(text: &str, top: &HashSet<&str>) -> String {
    let Names {
        spans,
        uses,
        count,
        kept,
    } = names_of(text, top);
    if count.is_empty() {
        return text.to_string();
    }
    let mut order: Vec<(&str, usize)> = count.into_iter().collect();
    order.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let mut names = short_names(&kept);
    let renamed: HashMap<&str, String> = order
        .into_iter()
        .map(|(name, _)| (name, names.next().unwrap_or_default()))
        .collect();

    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    for (&(start, end), how) in spans.iter().zip(&uses) {
        let name = &text[start..end];
        let Some(new) = renamed.get(name) else {
            continue;
        };
        match how {
            Use::Key => continue,
            Use::Plain => {
                out.push_str(&text[at..start]);
                out.push_str(new);
            }
            Use::Shorthand => {
                out.push_str(&text[at..end]);
                out.push_str(": ");
                out.push_str(new);
            }
        }
        at = end;
    }
    out.push_str(&text[at..]);
    out
}

/// The short names, in order, none of them one of `kept` or a word.
fn short_names<'a>(kept: &'a HashSet<&'a str>) -> impl Iterator<Item = String> + 'a {
    (0usize..).filter_map(move |mut n| {
        let mut name = String::new();
        name.push(char::from(FIRST[n % FIRST.len()]));
        n /= FIRST.len();
        while n > 0 {
            n -= 1;
            name.push(char::from(REST[n % REST.len()]));
            n /= REST.len();
        }
        (!kept.contains(name.as_str())
            && !WORDS.contains(&name.as_str())
            && !super::RESERVED.contains(&name.as_str()))
        .then_some(name)
    })
}

/// What each name in `spans` is where it stands: a value, a shorthand
/// property, or a key. `code` is the item with its strings and comments
/// blanked, as [`scan`] gives it.
fn classify(code: &[u8], spans: &[(usize, usize)]) -> Vec<Use> {
    // The brackets open at each point, and whether a `{` is an object (a
    // literal or a pattern) rather than a block.
    let mut stack: Vec<(u8, bool)> = Vec::new();
    let mut out = Vec::with_capacity(spans.len());
    let mut next_span = 0;
    let mut i = 0;
    while i < code.len() {
        if next_span < spans.len() && spans[next_span].0 == i {
            let (start, end) = spans[next_span];
            out.push(use_of(code, start, end, stack.last().copied()));
            next_span += 1;
            i = end;
            continue;
        }
        match code[i] {
            b'{' => {
                let object = brace_is_object(code, i, stack.last().copied());
                stack.push((b'{', object));
            }
            b'(' | b'[' => stack.push((code[i], true)),
            b'}' | b')' | b']' => {
                stack.pop();
            }
            _ => {}
        }
        i += 1;
    }
    // Names after the last bracket, if any were not reached.
    while out.len() < spans.len() {
        out.push(Use::Plain);
    }
    out
}

/// The last character of code before `at`, and the word it ends, if any.
fn before(code: &[u8], at: usize) -> (u8, &str) {
    let mut j = at;
    while j > 0 && code[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    if j == 0 {
        return (b'\n', "");
    }
    let last = code[j - 1];
    let mut k = j;
    while k > 0
        && (code[k - 1].is_ascii_alphanumeric() || code[k - 1] == b'_' || code[k - 1] == b'$')
    {
        k -= 1;
    }
    (last, std::str::from_utf8(&code[k..j]).unwrap_or(""))
}

/// The first character of code at or after `at`, and the one after it.
fn after(code: &[u8], at: usize) -> (u8, u8) {
    let mut j = at;
    while j < code.len() && code[j].is_ascii_whitespace() {
        j += 1;
    }
    (
        code.get(j).copied().unwrap_or(b'\n'),
        code.get(j + 1).copied().unwrap_or(b'\n'),
    )
}

/// Whether the `{` at `at` opens an object literal or a pattern, rather
/// than a block (a function's body, a statement's, a template's `${`).
fn brace_is_object(code: &[u8], at: usize, around: Option<(u8, bool)>) -> bool {
    let (last, word) = before(code, at);
    if last.is_ascii_alphanumeric() || last == b'_' || last == b'$' {
        // `const {`, `return {`; `else {`, `try {` are blocks, and so is a
        // template's `${`, whose `$` ends no word of code.
        return matches!(
            word,
            "const"
                | "let"
                | "var"
                | "return"
                | "typeof"
                | "in"
                | "of"
                | "await"
                | "yield"
                | "throw"
                | "case"
                | "void"
                | "delete"
        );
    }
    match last {
        // `x => {` is a body.
        b'>' => !arrow_before(code, at),
        b'(' | b',' | b'=' | b'[' | b'?' | b'!' | b'&' | b'|' | b'+' | b'-' | b'*' | b'%'
        | b'<' | b'~' | b'^' => true,
        // `key: {` in an object or a call, and `c ? a : {` anywhere, are
        // objects; `case x: {` in a block is a block.
        b':' => around.is_none_or(|(c, object)| c != b'{' || object) || ternary_before(code, at),
        _ => false,
    }
}

/// Whether the `:` before `at` closes a `?` of the same statement: walk
/// back over balanced brackets to where the statement starts.
fn ternary_before(code: &[u8], at: usize) -> bool {
    let mut j = code[..at].iter().rposition(|&c| c == b':').unwrap_or(0);
    let mut depth = 0usize;
    while j > 0 {
        j -= 1;
        match code[j] {
            b')' | b']' | b'}' => depth += 1,
            b'(' | b'[' | b'{' => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
            b';' if depth == 0 => return false,
            // `?` alone: not `?.` nor `??`.
            b'?' if depth == 0
                && code.get(j + 1).is_some_and(|&c| c != b'.' && c != b'?')
                && (j == 0 || code[j - 1] != b'?') =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

/// Whether the code before `at` ends in `=>`.
fn arrow_before(code: &[u8], at: usize) -> bool {
    let mut j = at;
    while j > 0 && code[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    j >= 2 && &code[j - 2..j] == b"=>"
}

/// What the name at `start..end` is where it stands.
fn use_of(code: &[u8], start: usize, end: usize, around: Option<(u8, bool)>) -> Use {
    let in_object = matches!(around, Some((b'{', true)));
    let (last, word) = before(code, start);
    let (next, then) = after(code, end);
    // `get name() {`, `async name() {`: a method's name.
    if matches!(word, "get" | "set" | "static" | "async")
        && next == b'('
        && method_follows(code, end)
    {
        return Use::Key;
    }
    if !matches!(last, b'{' | b',') {
        return Use::Plain;
    }
    // `{ name: …` and `, name: …` are a key wherever they stand: the runtime
    // has no labels, and a ternary's `?` comes before its `:`. So is a
    // method, `{ name(…) { … } }`.
    match next {
        b':' => return Use::Key,
        b'(' if method_follows(code, end) => return Use::Key,
        _ => {}
    }
    if !in_object {
        return Use::Plain;
    }
    match next {
        b',' | b'}' => Use::Shorthand,
        // `{ name = 1 }` in a pattern: a shorthand with a default.
        b'=' if then != b'=' && then != b'>' => Use::Shorthand,
        _ => Use::Plain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn whole_runtime() -> Vec<&'static str> {
        super::super::MODULES.iter().map(|m| m.source).collect()
    }

    fn top_names() -> HashSet<&'static str> {
        super::super::MODULES
            .iter()
            .flat_map(|m| items(m.source))
            .filter_map(|item| item.declares)
            .collect()
    }

    /// A global the runtime names that is not on the list would be taken
    /// for a local, and renamed into a name that means nothing.
    #[test]
    fn every_global_the_runtime_names_is_on_the_list() {
        let chrome: HashSet<&str> = include_str!("browser_globals.txt").lines().collect();
        let mut missing: Vec<&str> = Vec::new();
        for source in whole_runtime() {
            for (a, b) in super::super::identifiers(source) {
                let name = &source[a..b];
                if chrome.contains(name) && !GLOBALS.contains(&name) && !missing.contains(&name) {
                    missing.push(name);
                }
            }
        }
        missing.sort_unstable();
        assert!(missing.is_empty(), "add these to `GLOBALS`: {missing:?}");
    }

    /// The names a piece of code binds, read loosely: what follows `const`,
    /// `let`, `var`, `function`, `class` and `catch (`, and what is in the
    /// parentheses of a function, a method or an arrow.
    fn bound(text: &str) -> HashSet<&str> {
        let (spans, code) = scan(text);
        let word = |k: usize| &text[spans[k].0..spans[k].1];
        let mut out = HashSet::new();
        // Each `(…)` that holds parameters: after `function`, `catch`, a
        // method's name, or followed by `=>`.
        let mut params: Vec<(usize, usize)> = Vec::new();
        let mut stack: Vec<usize> = Vec::new();
        for (i, &c) in code.iter().enumerate() {
            match c {
                b'(' => stack.push(i),
                b')' => {
                    if let Some(open) = stack.pop() {
                        let (next, then) = after(&code, i + 1);
                        let (last, w) = before(&code, open);
                        let arrow = next == b'=' && then == b'>';
                        let named = last.is_ascii_alphanumeric() || last == b'_' || last == b'$';
                        let body = next == b'{';
                        if arrow
                            || w == "catch"
                            || w == "function"
                            || (named && body && !matches!(w, "if" | "for" | "while" | "switch"))
                        {
                            params.push((open, i));
                        }
                    }
                }
                _ => {}
            }
        }
        for (k, &(a, b)) in spans.iter().enumerate() {
            let name = &text[a..b];
            let prev = if k > 0 { word(k - 1) } else { "" };
            let (next, then) = after(&code, b);
            if matches!(prev, "function" | "class") {
                out.insert(name);
            }
            if next == b'=' && then == b'>' {
                out.insert(name);
            }
            if params.iter().any(|&(o, c)| a > o && b <= c) {
                out.insert(name);
            }
        }
        // After `const`, `let` and `var`: the names up to the `=`, `of` or
        // `in` that ends the binding, at the same depth.
        for k in 0..spans.len() {
            if !matches!(word(k), "const" | "let" | "var") {
                continue;
            }
            let mut depth = 0i32;
            let mut j = spans[k].1;
            let mut names = spans[k + 1..].iter().peekable();
            while j < code.len() {
                match code[j] {
                    b'{' | b'[' | b'(' => depth += 1,
                    b'}' | b']' | b')' => {
                        depth -= 1;
                        if depth < 0 {
                            break;
                        }
                    }
                    // The initialiser: skip it, to a `,` that starts another
                    // binding, or to the end of the statement.
                    b'=' if depth == 0 => {
                        let mut d = 0i32;
                        let mut q = j + 1;
                        while q < code.len() {
                            match code[q] {
                                b'{' | b'[' | b'(' => d += 1,
                                b'}' | b']' | b')' => {
                                    if d == 0 {
                                        break;
                                    }
                                    d -= 1;
                                }
                                b',' | b';' if d == 0 => break,
                                _ => {}
                            }
                            q += 1;
                        }
                        if code.get(q) != Some(&b',') {
                            break;
                        }
                        while names.peek().is_some_and(|&&(a, _)| a <= q) {
                            names.next();
                        }
                        j = q;
                    }
                    b';' | b'\n' if depth == 0 => break,
                    _ => {}
                }
                while let Some(&&(a, b)) = names.peek() {
                    if a > j {
                        break;
                    }
                    if a == j {
                        let name = &text[a..b];
                        if depth == 0 && matches!(name, "of" | "in") {
                            j = code.len();
                            break;
                        }
                        out.insert(name);
                    }
                    names.next();
                }
                j += 1;
            }
        }
        out
    }

    /// A name taken for a local is bound in its item: a free name that is
    /// neither the runtime's nor on the list of globals would be renamed
    /// into one that means nothing.
    #[test]
    fn every_name_taken_for_a_local_is_bound_in_its_item() {
        let top = top_names();
        let mut unbound: Vec<String> = Vec::new();
        for source in whole_runtime() {
            for item in items(source) {
                let names = names_of(item.text, &top);
                let bound = bound(item.text);
                for name in names.count.keys() {
                    if !bound.contains(name) {
                        let head = item.text.trim_start().lines().next().unwrap_or("");
                        unbound.push(format!("`{name}` in {head}"));
                    }
                }
            }
        }
        assert!(
            unbound.is_empty(),
            "{} unbound:\n{}",
            unbound.len(),
            unbound.join("\n")
        );
    }

    /// A shorthand, `{ a, b }`, is written out only in a brace taken for an
    /// object. One in a brace taken for a block would keep its old name as
    /// the key's value, so no brace the runtime writes may look like that.
    #[test]
    fn no_brace_taken_for_a_block_holds_a_shorthand() {
        let mut found: Vec<String> = Vec::new();
        for source in whole_runtime() {
            for item in items(source) {
                let (spans, code) = scan(item.text);
                let mut stack: Vec<bool> = Vec::new();
                let mut next_span = 0;
                let mut i = 0;
                while i < code.len() {
                    if next_span < spans.len() && spans[next_span].0 == i {
                        let (a, b) = spans[next_span];
                        let (last, _) = before(&code, a);
                        let (next, _) = after(&code, b);
                        if stack.last() == Some(&false)
                            && matches!(last, b'{' | b',')
                            && matches!(next, b',' | b'}')
                        {
                            let line = item.text[..a].lines().count();
                            let head = item.text.trim_start().lines().next().unwrap_or("");
                            found.push(format!("`{}` on line {line} of {head}", &item.text[a..b]));
                        }
                        next_span += 1;
                        i = b;
                        continue;
                    }
                    match code[i] {
                        b'{' => {
                            let around = stack.last().map(|&o| (b'{', o));
                            // A template's `${…}` is code: `${name}` is the
                            // name, not a shorthand.
                            let splice = i > 0 && code[i - 1] == b'$';
                            stack.push(splice || brace_is_object(&code, i, around));
                        }
                        b'(' | b'[' => stack.push(true),
                        b'}' | b')' | b']' => {
                            stack.pop();
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
        }
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    #[test]
    fn a_local_is_renamed_and_a_key_kept() {
        let top: HashSet<&str> = ["helper", "counter"].into_iter().collect();
        let text = "  function counter(start, label) {\n    const total = start + 1;\n    \
                    return { total, label: label, start() { return helper(total); }, get size() { return 1e3; } };\n  }\n";
        let out = rename_in(text, &top);
        assert!(out.contains("{ total: "), "{out}");
        assert!(out.contains("label: "), "{out}");
        assert!(out.contains("start() {"), "{out}");
        assert!(out.contains("get size()"), "{out}");
        assert!(out.contains("helper("), "{out}");
        assert!(out.contains("1e3"), "{out}");
        assert!(!out.contains("const total"), "{out}");
        assert!(
            out.contains("function counter("),
            "the item's own name is the runtime's: {out}"
        );
    }
}
