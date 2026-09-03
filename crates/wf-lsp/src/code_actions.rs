use std::collections::HashMap;
use tower_lsp::lsp_types::*;

/// Generate QuickFix code actions for diagnostics that have actionable suggestions.
pub fn provide_code_actions(
    uri: &Url,
    params: CodeActionParams,
) -> Vec<CodeActionOrCommand> {
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
