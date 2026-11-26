//! `V13`: a call of a built-in on its way out. The registry says which
//! (`ComponentSig::deprecated`) and what replaces it; this module writes
//! the rewrite, where it is exact, as a fix `wf fix` and the editor apply.
//!
//! A rewrite is a set of byte edits at the call's own spans: the name, a
//! flag, an argument's key, the parts in its block. One that would change
//! what the page does is not offered, and the finding says why.

use crate::diagnostics::fixes::Plan;
use crate::registry::Deprecation;
use crate::syntax::parser::ast::{
    Arg, ComponentRef, Expr, Span, Statement, StatementKind, UIElement,
};

/// What a deprecated call is told: the finding's words, its hint, and the
/// rewrite when there is an exact one.
pub(super) struct Finding {
    pub message: String,
    pub hint: String,
    pub fix: Option<(String, Plan)>,
}

/// The finding for `el`, a call of the built-in `name` that `dep` says is
/// deprecated. `top` is whether it is written at the top of a page's or a
/// component's body, where a toast element is drawn once, as a call is.
pub(super) fn finding(name: &str, dep: &Deprecation, el: &UIElement, top: bool) -> Finding {
    let old = dep.spelling(name);
    let message = format!(
        "{old} is deprecated since {}: `{}` takes its place, and {old} is removed in 9.0",
        dep.since, dep.replacement
    );
    let fix = rewrite(name, el, top);
    let mut hint = dep.note.to_string();
    let why = match &fix {
        Ok(_) => "`wf fix --code V13` rewrites it".to_string(),
        Err(why) => format!("It is not rewritten for you: {why}"),
    };
    if !hint.is_empty() {
        hint.push(' ');
    }
    hint.push_str(&why);
    Finding {
        message,
        hint,
        fix: fix
            .ok()
            .map(|plan| (format!("Write `{}`", dep.replacement), plan)),
    }
}

/// The word `from` at `at` becomes `to`.
fn swap(at: u32, from: &str, to: &str) -> Plan {
    Plan::Swap {
        start: at as usize,
        from: from.to_string(),
        to: to.to_string(),
    }
}

/// `text` goes in at `at`.
fn insert(at: u32, text: &str) -> Plan {
    Plan::Replace {
        start: at as usize,
        end: at as usize,
        text: text.to_string(),
    }
}

/// The flag `.word` at `span` goes.
fn drop_flag(span: Span, word: &str) -> Plan {
    swap(span.start, &format!(".{word}"), "")
}

/// Every part of `owner` written in `body`, its owner's name at each
/// renamed to `to`. A nested call of the owner keeps its own parts: its
/// own finding renames them.
fn parts(owner: &str, to: &str, body: &[Statement], out: &mut Vec<Plan>) {
    for stmt in body {
        if let StatementKind::UIElement(el) = &stmt.kind {
            match &el.component {
                ComponentRef::SubComponent(o, _) if o == owner && el.span.line > 0 => {
                    out.push(swap(el.span.start, owner, to));
                }
                ComponentRef::BuiltIn(n) if n == owner => continue,
                _ => {}
            }
            parts(owner, to, &el.children, out);
            for fill in &el.slot_fills {
                parts(owner, to, &fill.body, out);
            }
        }
        for b in stmt.kind.bodies() {
            parts(owner, to, b, out);
        }
    }
}

/// The span of the argument called `key`, and where its value is.
fn named<'a>(el: &'a UIElement, key: &str) -> Option<(Span, &'a Expr)> {
    el.args
        .iter()
        .zip(&el.arg_spans)
        .find_map(|(a, s)| match a {
            Arg::Named(k, v) if k == key => Some((*s, v)),
            _ => None,
        })
}

fn positional(el: &UIElement) -> Option<(Span, &Expr)> {
    el.args
        .iter()
        .zip(&el.arg_spans)
        .find_map(|(a, s)| match a {
            Arg::Positional(v) => Some((*s, v)),
            Arg::Named(..) => None,
        })
}

/// The exact rewrite of a call of `name`, or why there is none.
fn rewrite(name: &str, el: &UIElement, top: bool) -> Result<Plan, &'static str> {
    // A call the compiler wrote has no place in the source to rewrite.
    if el.span.line == 0 {
        return Err("the compiler wrote this call");
    }
    let at = el.span.start;
    let mut edits = Vec::new();
    match name {
        "Dialog" => {
            edits.push(swap(at, name, "Modal"));
            parts("Dialog", "Modal", &el.children, &mut edits);
        }
        "Dropdown" => {
            let first = positional(el);
            let label = named(el, "label");
            if first.is_some() && label.is_some() {
                return Err(
                    "it says its label twice, and a `Menu`'s `label:` is the list's name, not its button's words",
                );
            }
            edits.push(swap(at, name, "Menu"));
            if let Some((span, _)) = first {
                edits.push(insert(span.start, "trigger: "));
            }
            if let Some((span, _)) = label {
                edits.push(swap(span.start, "label", "trigger"));
            }
            parts("Dropdown", "Menu", &el.children, &mut edits);
        }
        "Tag" => {
            if positional(el).is_none() && named(el, "label").is_none() {
                return Err("a `Chip` needs its words, and this tag has none");
            }
            if !el.events.is_empty() {
                return Err(
                    "a `Chip` with a handler is a button, where this tag is a label with a handler on it",
                );
            }
            edits.push(swap(at, name, "Chip"));
        }
        "Toast" => {
            const TONES: &[&str] = &["success", "danger", "warning", "info"];
            if !top {
                return Err(
                    "it is drawn each time the branch or item it is in is drawn; call `toast(…)` where that becomes true",
                );
            }
            let only_text = matches!(el.args.as_slice(), [Arg::Positional(_)]);
            if !only_text
                || el.modifiers.len() > 1
                || el.modifiers.iter().any(|m| !TONES.contains(&m.as_str()))
                || !el.events.is_empty()
                || el.style_block.is_some()
            {
                return Err("it says more than its words and its tone, which a call does not take");
            }
            let Some(paren) = el.paren_span else {
                return Err("it has no words");
            };
            edits.push(swap(at, name, "toast"));
            if let (Some(tone), Some(span)) = (el.modifiers.first(), el.modifier_spans.first()) {
                edits.push(insert(paren.end - 1, &format!(", tone: .{tone}")));
                edits.push(drop_flag(*span, tone));
            }
        }
        "Code" => {
            let code = positional(el)
                .map(|(_, e)| e)
                .or_else(|| named(el, "content").map(|(_, e)| e));
            match code {
                None => return Err("it has no code"),
                Some(Expr::NumberLiteral(_) | Expr::BoolLiteral(_)) => {
                    return Err("a `CodeBlock`'s code is text");
                }
                Some(_) => {}
            }
            edits.push(swap(at, name, "CodeBlock"));
            if let Some((span, _)) = named(el, "content") {
                edits.push(swap(span.start, "content", "code"));
            }
            for (m, span) in el.modifiers.iter().zip(&el.modifier_spans) {
                if m == "block" {
                    edits.push(drop_flag(*span, m));
                }
            }
        }
        "CircularProgress" => match el.paren_span {
            Some(paren) => {
                edits.push(swap(at, name, "Progress"));
                edits.push(insert(paren.end, ".circular"));
            }
            None => edits.push(swap(at, name, "Progress.circular")),
        },
        _ => return Err("there is no rewrite for it"),
    }
    Ok(Plan::All(edits))
}
