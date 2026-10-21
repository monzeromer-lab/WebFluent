//! What a program says about the project around it: files it names that
//! the project does not have, messages it asks for that no translation
//! holds, and a persisted value whose shape changed since the last build.

use crate::diagnostics::Diagnostic;
use crate::parser::ast::*;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Every expression in `stmts`, with the place of the statement or the
/// argument it is in.
pub(crate) fn visit(stmts: &[Statement], f: &mut dyn FnMut(&Expr, Span)) {
    fn expr(e: &Expr, span: Span, f: &mut dyn FnMut(&Expr, Span)) {
        f(e, span);
        for child in e.children() {
            expr(child, span, f);
        }
    }
    for stmt in stmts {
        // An element's arguments are visited below, each at its own place.
        if !matches!(stmt.kind, StatementKind::UIElement(_)) {
            for e in stmt.kind.exprs() {
                expr(e, stmt.span, f);
            }
        }
        if let StatementKind::UIElement(ui) = &stmt.kind {
            for (i, arg) in ui.args.iter().enumerate() {
                let at = ui.arg_spans.get(i).copied().unwrap_or(ui.span);
                match arg {
                    Arg::Positional(e) | Arg::Named(_, e) => expr(e, at, f),
                }
            }
            visit(&ui.children, f);
            for fill in &ui.slot_fills {
                visit(&fill.body, f);
            }
            for h in &ui.events {
                visit(&h.body, f);
            }
        }
        for body in stmt.kind.bodies() {
            visit(body, f);
        }
    }
}

fn bodies(program: &Program) -> impl Iterator<Item = (usize, &[Statement])> {
    program
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(i, d)| match d {
            Declaration::Page(p) => Some((i, p.body.as_slice())),
            Declaration::Component(c) => Some((i, c.body.as_slice())),
            Declaration::App(a) => Some((i, a.body.as_slice())),
            Declaration::Store(s) => Some((i, s.body.as_slice())),
            _ => None,
        })
}

/// D01: a site-relative `src:`, `poster:`, `captions:` or `transcript:`
/// literal naming a file `public/` does not have — the page asks for it
/// and gets a 404.
pub fn missing_assets(
    program: &Program,
    dir: &Path,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    // What `public/` has, as the site asks for it: `/img/hero.png`.
    let mut have = Vec::new();
    list_public(&dir.join("public"), "", &mut have);
    let nearest_file = |path: &str| {
        crate::diagnostics::fixes::nearest(path, have.iter().map(String::as_str)).map(|n| {
            (
                format!("Change to `{n}`"),
                crate::diagnostics::fixes::Plan::Rename {
                    from: path.to_string(),
                    to: n.to_string(),
                },
            )
        })
    };
    for (index, body) in bodies(program) {
        let file = file_of(index);
        // `worker lang = spawn("/workers/lang.js")`: the script is a file of
        // `public/` too.
        walk_statements(body, &mut |stmt| {
            let StatementKind::Connection(c) = &stmt.kind else {
                return;
            };
            if c.kind != ConnectionKind::Worker {
                return;
            }
            let Some(Expr::StringLiteral(url)) = &c.url else {
                return;
            };
            if !url.starts_with('/') || url.starts_with("//") {
                return;
            }
            let path = url.split(['?', '#']).next().unwrap_or("");
            if path.len() > 1
                && !dir
                    .join("public")
                    .join(path.trim_start_matches('/'))
                    .exists()
            {
                out.push(
                    Diagnostic::coded(
                        "D01",
                        format!("`spawn(\"{url}\")` names a script `public/` does not have, so the worker never starts"),
                        &file,
                        stmt.span.line.max(1) as usize,
                        stmt.span.col.max(1) as usize,
                    )
                    .with_hint(format!("Put the worker at `public{path}`, or correct the path"))
                    .with_plans(nearest_file(path)),
                );
            }
        });
        walk_elements(body, &mut |ui| {
            // A frame's `src` is a page — this site's or another's — not a
            // file of `public/`.
            if matches!(&ui.component, ComponentRef::BuiltIn(name) if name == "Frame") {
                return;
            }
            for (i, arg) in ui.args.iter().enumerate() {
                let Arg::Named(key, Expr::StringLiteral(url)) = arg else {
                    continue;
                };
                if !names_a_file(&ui.component, key) {
                    continue;
                }
                if !url.starts_with('/') || url.starts_with("//") {
                    continue;
                }
                let path = url.split(['?', '#']).next().unwrap_or("");
                let on_disk = dir.join("public").join(path.trim_start_matches('/'));
                if path.len() > 1 && !on_disk.exists() {
                    let at = ui.arg_spans.get(i).copied().unwrap_or(ui.span);
                    out.push(
                        Diagnostic::coded(
                            "D01",
                            format!("`{key}: \"{url}\"` names a file `public/` does not have, so the page asks for it and gets nothing"),
                            &file,
                            at.line.max(1) as usize,
                            at.col.max(1) as usize,
                        )
                        .with_hint(format!("Put the file at `public{path}`, or correct the path"))
                        .with_plans(nearest_file(path)),
                    );
                }
            }
        });
    }
    out
}

/// Whether `key` on `component` names a file of `public/`: on a built-in,
/// a prop the registry types as a file; on the project's own component,
/// a prop called as the built-ins call theirs.
pub fn names_a_file(component: &ComponentRef, key: &str) -> bool {
    match component {
        ComponentRef::BuiltIn(name) => {
            crate::registry::file_props().any(|(c, p, _)| c.name == name && p.name == key)
        }
        _ => matches!(key, "src" | "poster" | "captions" | "transcript"),
    }
}

/// Every file under `dir`, as `/<prefix>/<name>`.
fn list_public(dir: &Path, prefix: &str, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        if path.is_dir() {
            list_public(&path, &format!("{prefix}/{name}"), out);
        } else {
            out.push(format!("{prefix}/{name}"));
        }
    }
}

fn walk_statements(stmts: &[Statement], f: &mut dyn FnMut(&Statement)) {
    for stmt in stmts {
        f(stmt);
        if let StatementKind::UIElement(ui) = &stmt.kind {
            walk_statements(&ui.children, f);
            for fill in &ui.slot_fills {
                walk_statements(&fill.body, f);
            }
        }
        for body in stmt.kind.bodies() {
            walk_statements(body, f);
        }
    }
}

fn walk_elements(stmts: &[Statement], f: &mut dyn FnMut(&UIElement)) {
    for stmt in stmts {
        if let StatementKind::UIElement(ui) = &stmt.kind {
            f(ui);
            walk_elements(&ui.children, f);
            for fill in &ui.slot_fills {
                walk_elements(&fill.body, f);
            }
        }
        for body in stmt.kind.bodies() {
            walk_elements(body, f);
        }
    }
}

/// The placeholders a message uses: `{name}`.
fn placeholders(message: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = message;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            break;
        };
        let name = &after[..close];
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            out.push(name.to_string());
        }
        rest = &after[close + 1..];
    }
    out
}

/// I01–I04: what a program asks of its translations, against what they
/// hold.
pub fn translations(
    program: &Program,
    config: &crate::config::project::I18nConfig,
    messages: &HashMap<String, HashMap<String, String>>,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let has = |key: &str| -> Option<(&String, &HashMap<String, String>)> {
        messages.iter().find(|(_, m)| {
            m.contains_key(key)
                || m.keys().any(|k| {
                    k.strip_prefix(key).is_some_and(|r| {
                        matches!(r, ".zero" | ".one" | ".two" | ".few" | ".many" | ".other")
                    })
                })
        })
    };
    let default = messages.get(&config.default_locale);
    // Every `t()` the program writes — in bodies, and in a page's header,
    // layout and head — as the translations table counts them.
    for u in crate::i18n::table::uses(program, file_of) {
        if u.dynamic {
            continue;
        }
        let (file, line, col) = (u.file.clone(), u.line as usize, u.col as usize);
        let key = &u.key;
        let Some((_, holder)) = has(key) else {
            let mut keys: Vec<&String> = messages.values().flat_map(|m| m.keys()).collect();
            keys.sort();
            keys.dedup();
            let near = keys
                .iter()
                .map(|k| (crate::linter::vocabulary::levenshtein(key, k), *k))
                .filter(|(d, _)| *d <= 3)
                .min()
                .map(|(_, k)| format!("Did you mean `{k}`?"))
                .unwrap_or_else(|| {
                    format!(
                        "Add `\"{key}\": \"…\"` to `{}/{}.json`",
                        config.dir, config.default_locale
                    )
                });
            out.push(
                            Diagnostic::coded(
                                "I01",
                                format!("`t(\"{key}\")` names a message no translation has, so the page shows `{key}`"),
                                &file,
                                line,
                                col,
                            )
                            .with_hint(near),
                        );
            continue;
        };
        // I02: what the call passes against what the message uses.
        let source = default.unwrap_or(holder);
        let mut used: Vec<String> = Vec::new();
        let mut plural = false;
        for (k, m) in source {
            let form = k.strip_prefix(key.as_str());
            // The message itself and its plural forms — not
            // `key.in`, a message of its own under the same name.
            let plural_form =
                |r: &str| matches!(r, ".zero" | ".one" | ".two" | ".few" | ".many" | ".other");
            if form == Some("") || form.is_some_and(plural_form) {
                plural |= form != Some("");
                used.extend(placeholders(m));
            }
        }
        used.sort();
        used.dedup();
        let Some(given) = u.passes.clone() else {
            continue;
        };
        for g in &given {
            if !used.contains(g) && !(plural && g == "count") {
                out.push(
                    Diagnostic::coded(
                        "I02",
                        format!("`t(\"{key}\")` passes `{g}`, which the message does not use"),
                        &file,
                        line,
                        col,
                    )
                    .with_hint(format!(
                        "The message uses {}",
                        if used.is_empty() {
                            "no placeholder".to_string()
                        } else {
                            used.iter()
                                .map(|u| format!("`{{{u}}}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    )),
                );
            }
        }
        for u in &used {
            if !given.contains(u) {
                out.push(Diagnostic::coded(
                                "I02",
                                format!("the message `{key}` shows `{{{u}}}`, which `t(\"{key}\")` does not pass, so it shows as written"),
                                &file,
                                line,
                                col,
                            ).with_hint(format!("Pass it: `t(\"{key}\", {{ {u}: … }})`")));
            }
        }
    }
    for (index, body) in bodies(program) {
        let file = file_of(index);
        visit(body, &mut |e, span| {
            let (line, col) = (span.line.max(1) as usize, span.col.max(1) as usize);
            let Expr::FunctionCall(name, args) = e else {
                return;
            };
            match (name.as_str(), args.first()) {
                ("setLocale", Some(Expr::StringLiteral(locale)))
                    if !config.locales.contains(locale) =>
                {
                    let near = crate::diagnostics::fixes::nearest(
                        locale,
                        config.locales.iter().map(String::as_str),
                    )
                    .map(str::to_string);
                    out.push(
                        Diagnostic::coded(
                            "I04",
                            format!(
                                "`setLocale(\"{locale}\")` names a locale the project does not have"
                            ),
                            &file,
                            line,
                            col,
                        )
                        .with_plans(near.map(|n| {
                            (
                                format!("Change to `{n}`"),
                                crate::diagnostics::fixes::Plan::Rename {
                                    from: format!("\"{locale}\""),
                                    to: format!("\"{n}\""),
                                },
                            )
                        }))
                        .with_hint(format!(
                            "The locales are {}; add it to `i18n.locales` with a `{locale}.json`",
                            config
                                .locales
                                .iter()
                                .map(|l| format!("`{l}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                    );
                }
                _ => {}
            }
        });
    }
    // I03: a key one locale has and another does not.
    let mut all: Vec<&String> = messages.values().flat_map(|m| m.keys()).collect();
    all.sort();
    all.dedup();
    for locale in &config.locales {
        let file = format!("{}/{locale}.json", config.dir.trim_end_matches('/'));
        let Some(own) = messages.get(locale) else {
            out.push(
                Diagnostic::coded("I03", format!("the locale `{locale}` has no `{locale}.json`, so every message shows its key"), &file, 1, 1)
                    .with_hint(format!("Add `{file}`")),
            );
            continue;
        };
        // A plural form one locale's rules need and another's do not —
        // Arabic's `.two` and `.few` — is not missing where the message has
        // its `.other`: that is what the form falls back to.
        let falls_back = |k: &str| {
            [".zero", ".one", ".two", ".few", ".many"]
                .iter()
                .find_map(|f| k.strip_suffix(f))
                .is_some_and(|base| {
                    own.contains_key(&format!("{base}.other")) || own.contains_key(base)
                })
        };
        let missing: Vec<&&String> = all
            .iter()
            .filter(|k| !own.contains_key(**k) && !falls_back(k))
            .collect();
        if !missing.is_empty() {
            let shown: Vec<String> = missing.iter().take(8).map(|k| format!("`{k}`")).collect();
            let more = if missing.len() > 8 {
                format!(" and {} more", missing.len() - 8)
            } else {
                String::new()
            };
            out.push(
                Diagnostic::coded(
                    "I03",
                    format!("`{locale}.json` has no {}{more}, which another locale has, so a reader in `{locale}` sees {}", shown.join(", "), if *locale == config.default_locale { "the key".to_string() } else { format!("the `{}` message", config.default_locale) }),
                    &file,
                    1,
                    1,
                )
                .with_hint("Translate each, or remove it from the locales that have it"),
            );
        }
    }
    out
}

/// What each `persist` holds, as its declared type or the shape of its
/// Where the build keeps what each persisted value starts as, and where it
/// moves the previous build's when they differ.
pub const PERSIST_VALUES: &str = ".wf-cache/persist-values.json";
pub const PERSIST_VALUES_BEFORE: &str = ".wf-cache/persist-values.previous.json";

/// Each persisted value's storage key (`wf:Owner.name`) and what storage
/// holds for it on a first visit — its initial value, in the envelope its
/// `version:` writes — where the value is known at build time.
pub fn persisted_values(
    program: &Program,
    env: &BTreeMap<String, serde_json::Value>,
) -> BTreeMap<String, serde_json::Value> {
    let scope = crate::codegen::static_eval::Scope::from_program_with_env(program, &[], env);
    let mut out = BTreeMap::new();
    for decl in &program.declarations {
        let (owner, body) = match decl {
            Declaration::Page(p) => (p.name.as_str(), &p.body),
            Declaration::Component(c) => (c.name.as_str(), &c.body),
            Declaration::Store(s) => (s.name.as_str(), &s.body),
            _ => continue,
        };
        for stmt in body {
            let StatementKind::State(st) = &stmt.kind else {
                continue;
            };
            if !st.persist || st.policy.as_ref().is_some_and(|p| p.key.is_some()) {
                continue;
            }
            let Some(value) = crate::codegen::static_eval::eval(&st.value, &scope) else {
                continue;
            };
            let value = value.to_json();
            let version = st.policy.as_ref().and_then(|p| p.version).unwrap_or(0);
            let stored = if version > 0 {
                serde_json::json!({ "wf:v": version, "wf:d": value })
            } else {
                value
            };
            out.insert(format!("wf:{owner}.{}", st.name), stored);
        }
    }
    out
}

/// Write [`persisted_values`] to [`PERSIST_VALUES`], first moving what the
/// last build wrote there to [`PERSIST_VALUES_BEFORE`] when it differs.
pub fn keep_persisted_values(
    project_dir: &Path,
    program: &Program,
    env: &BTreeMap<String, serde_json::Value>,
) {
    let values = persisted_values(program, env);
    let now = project_dir.join(PERSIST_VALUES);
    let text = serde_json::to_string_pretty(&values).unwrap_or_default();
    let before = std::fs::read_to_string(&now).ok();
    if values.is_empty() && before.is_none() {
        return;
    }
    if let Some(parent) = now.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Some(before) = before
        && before != text
    {
        let _ = std::fs::write(project_dir.join(PERSIST_VALUES_BEFORE), before);
    }
    let _ = std::fs::write(now, text);
}

/// first value, with its version — `Owner.name` → (shape, version).
pub fn persist_shapes(program: &Program) -> BTreeMap<String, (String, u32)> {
    fn shape(e: &Expr) -> String {
        match e {
            Expr::NumberLiteral(_) => "Number".into(),
            Expr::StringLiteral(_) | Expr::InterpolatedString(_) => "String".into(),
            Expr::BoolLiteral(_) => "Bool".into(),
            Expr::ListLiteral(items) => {
                format!("[{}]", items.first().map(shape).unwrap_or_default())
            }
            Expr::MapLiteral(pairs) => {
                let mut fields: Vec<String> = pairs
                    .iter()
                    .filter(|(k, _)| k != "...")
                    .map(|(k, v)| format!("{}: {}", k.trim_matches('"'), shape(v)))
                    .collect();
                fields.sort();
                format!("{{{}}}", fields.join(", "))
            }
            Expr::Record(name, _) => name.clone(),
            _ => String::new(),
        }
    }
    let mut out = BTreeMap::new();
    for decl in &program.declarations {
        let (owner, body) = match decl {
            Declaration::Page(p) => (p.name.as_str(), &p.body),
            Declaration::Component(c) => (c.name.as_str(), &c.body),
            Declaration::Store(s) => (s.name.as_str(), &s.body),
            _ => continue,
        };
        for stmt in body {
            if let StatementKind::State(st) = &stmt.kind
                && st.persist
            {
                let ty = match &st.ty {
                    Some(t) => format!("{t:?}"),
                    None => shape(&st.value),
                };
                if ty.is_empty() {
                    continue;
                }
                let version = st.policy.as_ref().and_then(|p| p.version).unwrap_or(0);
                out.insert(format!("{owner}.{}", st.name), (ty, version));
            }
        }
    }
    out
}

/// Where the shapes a build saw are kept, for the next to compare with.
pub const PERSIST_CACHE: &str = ".wf-cache/persist.json";

/// D04: a persisted value whose shape changed since the last build with no
/// change of `version:` — a returning reader's browser holds the old shape,
/// and the page reads it as the new one.
pub fn persist_changes(
    program: &Program,
    dir: &Path,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<Diagnostic> {
    let Ok(text) = std::fs::read_to_string(dir.join(PERSIST_CACHE)) else {
        return Vec::new();
    };
    let Ok(before) = serde_json::from_str::<BTreeMap<String, (String, u32)>>(&text) else {
        return Vec::new();
    };
    let now = persist_shapes(program);
    let mut out = Vec::new();
    for (index, decl) in program.declarations.iter().enumerate() {
        let (owner, body) = match decl {
            Declaration::Page(p) => (p.name.as_str(), &p.body),
            Declaration::Component(c) => (c.name.as_str(), &c.body),
            Declaration::Store(s) => (s.name.as_str(), &s.body),
            _ => continue,
        };
        for stmt in body {
            let StatementKind::State(st) = &stmt.kind else {
                continue;
            };
            let key = format!("{owner}.{}", st.name);
            if let (Some((old, old_v)), Some((new, new_v))) = (before.get(&key), now.get(&key))
                && old != new
                && old_v == new_v
            {
                // The version raised, and the step that brings the old one
                // forward, as one change: a version with no way forward is
                // P01's loss of data.
                let from = (*new_v).max(1);
                let to = from + 1;
                let step = format!("migrate {from} -> {to} {{ old }}");
                let raise = match st.policy.as_ref() {
                    Some(policy) if policy.version.is_some() => {
                        Some(crate::diagnostics::fixes::Plan::All(vec![
                            crate::diagnostics::fixes::Plan::Rename {
                                from: format!("version: {new_v}"),
                                to: format!("version: {to}"),
                            },
                            crate::diagnostics::fixes::Plan::InsertInBlockClosingAt {
                                close: (stmt.span.end as usize).saturating_sub(1),
                                text: step.clone(),
                            },
                        ]))
                    }
                    Some(_) => Some(crate::diagnostics::fixes::Plan::InsertInBlockClosingAt {
                        close: (stmt.span.end as usize).saturating_sub(1),
                        text: format!("version: {to}\n{step}"),
                    }),
                    None => {
                        let indent = " ".repeat(stmt.span.col.saturating_sub(1) as usize);
                        Some(crate::diagnostics::fixes::Plan::InsertIn {
                            file: file_of(index),
                            at: stmt.span.end as usize,
                            text: format!(
                                " {{\n{indent}    version: {to}\n{indent}    {step}\n{indent}}}"
                            ),
                        })
                    }
                };
                out.push(
                    Diagnostic::coded(
                        "D04",
                        format!("`persist {}` holds a new shape and the same `version:`, so a returning reader's stored value is read as the new one", st.name),
                        file_of(index),
                        stmt.span.line.max(1) as usize,
                        stmt.span.col.max(1) as usize,
                    )
                    .with_hint(format!(
                        "Raise it — `{{ version: {} }}` — and say how an old value comes forward with `migrate {} -> {} {{ … }}`",
                        new_v + 1,
                        new_v,
                        new_v + 1
                    ))
                    .with_plans(raise.map(|p| (format!("Raise to version {to}, and bring {from} forward"), p))),
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_names_its_placeholders() {
        assert_eq!(
            placeholders("Hello, {name}! {count} new"),
            vec!["name", "count"]
        );
        assert!(placeholders("no {} here { x }").is_empty());
    }
}
