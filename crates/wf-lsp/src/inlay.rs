//! Inlay hints: the type the checker gives a `state`, `derived` value or
//! `let` written without one — `state count = 0` shows `: Number`.

use tower_lsp::lsp_types::*;
use webfluent::parser::ast::*;

use crate::analysis;
use crate::project::Project;

/// The most of a type a hint shows before it is cut short.
const LONGEST: usize = 40;

pub fn inlay_hints(project: &Project, file_ix: usize, range: Range) -> Vec<InlayHint> {
    let file = &project.files[file_ix];
    if file.script || file.markdown || file.parsed.is_err() {
        return Vec::new();
    }
    let source: &str = &file.source;
    let info = webfluent::sema::types::check(&project.program, &|_| String::new());
    let mut out = Vec::new();
    fn walk<'a>(stmts: &'a [Statement], found: &mut Vec<(&'a Statement, &'a str)>) {
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::State(s) if s.ty.is_none() && !s.name.starts_with("d__") => {
                    found.push((stmt, &s.name))
                }
                StatementKind::Derived(d) if d.ty.is_none() => found.push((stmt, &d.name)),
                _ => {}
            }
            for body in analysis::child_bodies(stmt) {
                walk(body, found);
            }
        }
    }
    for (decl_ix, decl) in project.declarations_of(file_ix) {
        let mut found = Vec::new();
        walk(analysis::body_of(decl), &mut found);
        for (stmt, name) in found {
            let Some(ty) = info.type_at(decl_ix, name, stmt.span) else {
                continue;
            };
            if ty.is_any() {
                continue;
            }
            // Just after the name, which follows the statement's keyword.
            let start = stmt.span.start as usize;
            let Some(text) = source.get(start..(stmt.span.end as usize).min(source.len())) else {
                continue;
            };
            let Some(at) = text
                .match_indices(name)
                .find(|(i, _)| {
                    let b = text.as_bytes();
                    (*i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_'))
                        && b.get(i + name.len())
                            .is_none_or(|c| !(c.is_ascii_alphanumeric() || *c == b'_'))
                })
                .map(|(i, _)| start + i + name.len())
            else {
                continue;
            };
            let position = file.index.offset_to_position(source, at);
            if position < range.start || position > range.end {
                continue;
            }
            // A long record shape is cut short; the tooltip has all of it.
            let full = ty.to_string();
            let (label, tooltip) = if full.chars().count() > LONGEST {
                let cut: String = full.chars().take(LONGEST - 1).collect();
                (format!(": {cut}…"), Some(InlayHintTooltip::String(full)))
            } else {
                (format!(": {full}"), None)
            };
            out.push(InlayHint {
                position,
                label: InlayHintLabel::String(label),
                kind: Some(InlayHintKind::TYPE),
                text_edits: None,
                tooltip,
                padding_left: Some(false),
                padding_right: Some(false),
                data: None,
            });
        }
    }
    out
}
