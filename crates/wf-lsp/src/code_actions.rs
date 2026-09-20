use std::collections::HashMap;
use tower_lsp::lsp_types::*;

/// Generate QuickFix code actions for diagnostics that have actionable suggestions.
pub fn provide_code_actions(uri: &Url, params: CodeActionParams) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    for diag in params.context.diagnostics {
        if let Some(suggestion) = extract_suggestion(&diag.message) {
            let mut changes = HashMap::new();
            changes.insert(
                uri.clone(),
                vec![TextEdit {
                    range: diag.range,
                    new_text: suggestion.clone(),
                }],
            );

            let action = CodeAction {
                title: format!("Change to `{suggestion}`"),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diag.clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
                command: None,
            };

            actions.push(CodeActionOrCommand::CodeAction(action));
        }
    }

    actions
}

/// Extract suggested replacement from message text like "did you mean `foo`?" or "did you mean 'foo'?"
fn extract_suggestion(message: &str) -> Option<String> {
    let lower = message.to_lowercase();
    let marker = "did you mean ";
    let idx = lower.find(marker)?;
    let remainder = &message[idx + marker.len()..];

    // Check for backticks `foo`
    if let Some(start) = remainder.find('`') {
        let after_start = &remainder[start + 1..];
        if let Some(end) = after_start.find('`') {
            return Some(after_start[..end].to_string());
        }
    }

    // Check for single quotes 'foo'
    if let Some(start) = remainder.find('\'') {
        let after_start = &remainder[start + 1..];
        if let Some(end) = after_start.find('\'') {
            return Some(after_start[..end].to_string());
        }
    }

    // Check for double quotes "foo"
    if let Some(start) = remainder.find('"') {
        let after_start = &remainder[start + 1..];
        if let Some(end) = after_start.find('"') {
            return Some(after_start[..end].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_backtick_suggestion() {
        let msg = "'centered' is not a modifier; did you mean `center`?";
        assert_eq!(extract_suggestion(msg), Some("center".to_string()));
    }

    #[test]
    fn extract_single_quote_suggestion() {
        let msg = "unknown keyword; did you mean 'Store'?";
        assert_eq!(extract_suggestion(msg), Some("Store".to_string()));
    }
}

// ─── Extract component ──────────────────────────────────

use webfluent::parser::ast::*;
use webfluent::sema::types::Type;

use crate::analysis::{self, Binding, BindingKind};
use crate::project::Project;

/// `Extract component`: the whole statements the selection covers — elements
/// and the branches and loops around them — become a component declared at
/// the end of the file, and the selection its call. What the statements
/// read from the enclosing scope becomes a prop, typed where the checker
/// knows the type; the stores they use are `use`d again. A selection that
/// writes to the enclosing scope, or calls one of its actions, is not
/// offered, since a prop cannot carry that back.
pub fn extract_component_action(
    project: &Project,
    file_ix: usize,
    range: Range,
) -> Vec<CodeActionOrCommand> {
    let Some(edit) = extract_component_edit(project, file_ix, range) else {
        return Vec::new();
    };
    vec![CodeActionOrCommand::CodeAction(CodeAction {
        title: "Extract component".to_string(),
        kind: Some(CodeActionKind::REFACTOR_EXTRACT),
        diagnostics: None,
        edit: Some(edit),
        is_preferred: None,
        disabled: None,
        data: None,
        command: None,
    })]
}

fn extract_component_edit(
    project: &Project,
    file_ix: usize,
    range: Range,
) -> Option<WorkspaceEdit> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let start = file.index.position_to_offset(source, range.start)?;
    let end = file.index.position_to_offset(source, range.end)?;
    if end <= start {
        return None;
    }
    let decl_ix = analysis::declaration_at(project, file_ix, start)?;
    let decl = &project.program.declarations[decl_ix];
    if !matches!(decl, Declaration::Page(_) | Declaration::Component(_)) {
        return None;
    }
    let body = analysis::body_of(decl);

    // The list of statements the selection lives in: the innermost body
    // whose statements the range covers whole.
    let path = analysis::statement_path(body, start);
    let mut lists: Vec<&[Statement]> = vec![body];
    for stmt in &path {
        for child in analysis::child_bodies(stmt) {
            if child.iter().any(|s| analysis::contains(s.span, start)) {
                lists.push(child);
            }
        }
    }
    let list = lists.into_iter().rev().find(|l| {
        l.iter().any(|s| {
            s.span.start as usize >= start
                && (s.span.end as usize) <= end.max(start + 1)
                && extractable(s)
        })
    })?;
    let selected: Vec<&Statement> = list
        .iter()
        .filter(|s| s.span.start as usize >= start && (s.span.end as usize) <= end)
        .collect();
    if selected.is_empty() || !selected.iter().all(|s| extractable(s)) {
        return None;
    }
    let first = selected[0];
    let last = selected[selected.len() - 1];

    // What the selection reads, writes and calls.
    let mut uses = Uses::default();
    for stmt in &selected {
        walk_statement(stmt, &mut uses);
    }
    // The scope just before the selection: what the selection binds itself
    // — a loop's item, an arm's name — is its own.
    let scope = analysis::scope_at(decl, (first.span.start as usize).saturating_sub(1));
    // Names the selection itself declares are its own.
    let mut own: Vec<Binding> = Vec::new();
    for stmt in &selected {
        analysis::hoisted(std::slice::from_ref(stmt), &mut own);
    }
    let outer = |name: &str| {
        scope
            .iter()
            .find(|b| b.name == name && !own.iter().any(|o| o.name == name))
    };
    if uses.written.iter().any(|w| outer(w).is_some())
        || uses
            .called
            .iter()
            .any(|c| outer(c).is_some_and(|b| b.kind == BindingKind::Action))
    {
        return None;
    }
    let mut props: Vec<(String, String)> = Vec::new();
    let mut stores: Vec<String> = Vec::new();
    for name in &uses.read {
        let Some(binding) = outer(name) else { continue };
        match binding.kind {
            BindingKind::Store => {
                if !stores.contains(name) {
                    stores.push(name.clone());
                }
            }
            BindingKind::Action => return None,
            _ => {
                if !props.iter().any(|(n, _)| n == name) {
                    let ty = crate::hover::type_of_binding(project, decl_ix, binding)
                        .map(|t| type_text(&t))
                        .unwrap_or_else(|| "Any".to_string());
                    props.push((name.clone(), ty));
                }
            }
        }
    }
    props.sort();

    // A name no declaration has.
    let taken: Vec<&str> = project
        .program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) => Some(c.name.as_str()),
            Declaration::Page(p) => Some(p.name.as_str()),
            Declaration::Store(s) => Some(s.name.as_str()),
            _ => None,
        })
        .collect();
    let mut name = "Extracted".to_string();
    let mut n = 1;
    while taken.contains(&name.as_str()) {
        n += 1;
        name = format!("Extracted{n}");
    }

    // The declaration: the selected text, dedented and set one level in.
    let text = &source[first.span.start as usize..last.span.end as usize];
    let lines: Vec<&str> = text.lines().collect();
    let common = lines
        .iter()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut body_text = String::new();
    for (i, line) in lines.iter().enumerate() {
        let line = if i == 0 {
            line.trim_start()
        } else if line.len() >= common {
            &line[common..]
        } else {
            line.trim_start()
        };
        body_text.push_str("    ");
        body_text.push_str(line);
        body_text.push('\n');
    }
    let params = if props.is_empty() {
        String::new()
    } else {
        format!(
            "({})",
            props
                .iter()
                .map(|(n, t)| format!("{n}: {t}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let mut declaration = format!("\n\ncomponent {name}{params} {{\n");
    for store in &stores {
        declaration.push_str(&format!("    use {store}\n"));
    }
    declaration.push_str(&body_text);
    declaration.push_str("}\n");

    let call = if props.is_empty() {
        name.clone()
    } else {
        format!(
            "{name}({})",
            props
                .iter()
                .map(|(n, _)| format!("{n}: {n}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let trimmed_end = source.trim_end().len();
    let edits = vec![
        TextEdit {
            range: Range {
                start: file
                    .index
                    .offset_to_position(source, first.span.start as usize),
                end: file
                    .index
                    .offset_to_position(source, last.span.end as usize),
            },
            new_text: call,
        },
        TextEdit {
            range: Range {
                start: file.index.offset_to_position(source, trimmed_end),
                end: file.index.offset_to_position(source, trimmed_end),
            },
            new_text: declaration,
        },
    ];
    let mut changes = HashMap::new();
    changes.insert(file.uri.clone(), edits);
    Some(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

/// A statement a component body may hold as its content.
fn extractable(stmt: &Statement) -> bool {
    matches!(
        stmt.kind,
        StatementKind::UIElement(_)
            | StatementKind::If(_)
            | StatementKind::For(_)
            | StatementKind::Show(_)
            | StatementKind::Match(_)
    )
}

/// The written spelling of a type, as a prop takes it.
fn type_text(ty: &Type) -> String {
    match ty {
        Type::String | Type::Number | Type::Bool => ty.to_string(),
        Type::List(inner) => format!("[{}]", type_text(inner)),
        Type::Optional(inner) => format!("{}?", type_text(inner)),
        Type::Record(name) | Type::Enum(name) => name.clone(),
        Type::Map | Type::Shape(_) => "Map".to_string(),
        _ => "Any".to_string(),
    }
}

#[derive(Default)]
struct Uses {
    read: Vec<String>,
    written: Vec<String>,
    called: Vec<String>,
}

fn walk_statement(stmt: &Statement, uses: &mut Uses) {
    match &stmt.kind {
        StatementKind::Assignment(a) => {
            match &a.target {
                Expr::Identifier(name) => uses.written.push(name.clone()),
                other => walk_expr(other, uses),
            }
            walk_expr(&a.value, uses);
        }
        StatementKind::UIElement(el) => {
            for arg in &el.args {
                match arg {
                    Arg::Positional(e) | Arg::Named(_, e) => walk_expr(e, uses),
                }
            }
            if let Some(style) = &el.style_block {
                for p in &style.properties {
                    walk_expr(&p.value, uses);
                }
                for b in &style.pseudo_blocks {
                    for p in &b.properties {
                        walk_expr(&p.value, uses);
                    }
                }
                for m in &style.media_queries {
                    for p in &m.properties {
                        walk_expr(&p.value, uses);
                    }
                }
            }
            for h in &el.events {
                for s in &h.body {
                    walk_statement(s, uses);
                }
            }
            for f in &el.slot_fills {
                for s in &f.body {
                    walk_statement(s, uses);
                }
            }
            for s in &el.children {
                walk_statement(s, uses);
            }
        }
        StatementKind::MethodCall(m) => {
            if let Expr::Identifier(name) = &m.object {
                uses.read.push(name.clone());
            } else {
                walk_expr(&m.object, uses);
            }
            for a in &m.args {
                walk_expr(a, uses);
            }
        }
        StatementKind::For(f) => {
            walk_expr(&f.iterable, uses);
            if let Some(k) = &f.key {
                walk_expr(k, uses);
            }
        }
        other => {
            for e in other.exprs() {
                walk_expr(e, uses);
            }
        }
    }
    for body in stmt.kind.bodies() {
        for s in body {
            walk_statement(s, uses);
        }
    }
}

fn walk_expr(expr: &Expr, uses: &mut Uses) {
    match expr {
        Expr::Identifier(name) => uses.read.push(name.clone()),
        Expr::FunctionCall(name, _) => uses.called.push(name.clone()),
        _ => {}
    }
    for child in expr.children() {
        walk_expr(child, uses);
    }
}
