use tower_lsp::Client;
use tower_lsp::lsp_types::*;
use webfluent::error::{A11yWarning, Diagnostic as WfDiagnostic, VocabWarning, WebFluentError};

use crate::line_index::LineIndex;

/// Convert all WebFluent compiler errors, semantic validations, accessibility warnings,
/// and vocabulary warnings into LSP diagnostics and publish them to the client.
pub async fn publish_all_diagnostics(
    client: &Client,
    uri: &Url,
    source: &str,
    index: &LineIndex,
    errors: &[WebFluentError],
    semantic_diagnostics: &[WfDiagnostic],
    a11y_warnings: &[A11yWarning],
    vocab_warnings: &[VocabWarning],
    contrast_warnings: &[A11yWarning],
) {
    let mut diagnostics = Vec::new();

    // 1. Lexer & Parser & General Errors
    for err in errors {
        match err {
            WebFluentError::LexerError(diag) => {
                diagnostics.push(create_diagnostic(
                    source,
                    index,
                    diag.line,
                    diag.column,
                    1,
                    &diag.message,
                    diag.hint.as_deref(),
                    DiagnosticSeverity::ERROR,
                    "webfluent-lexer",
                    None,
                ));
            }
            WebFluentError::ParseError(diag) => {
                diagnostics.push(create_diagnostic(
                    source,
                    index,
                    diag.line,
                    diag.column,
                    1,
                    &diag.message,
                    diag.hint.as_deref(),
                    DiagnosticSeverity::ERROR,
                    "webfluent-parser",
                    None,
                ));
            }
            WebFluentError::EditError(msg) => {
                diagnostics.push(simple_diagnostic(msg, DiagnosticSeverity::ERROR, "webfluent-edit"));
            }
            WebFluentError::CodegenError(msg) => {
                diagnostics.push(simple_diagnostic(msg, DiagnosticSeverity::ERROR, "webfluent-codegen"));
            }
            WebFluentError::ConfigError(msg) => {
                diagnostics.push(simple_diagnostic(msg, DiagnosticSeverity::ERROR, "webfluent-config"));
            }
            WebFluentError::IoError(msg) => {
                diagnostics.push(simple_diagnostic(msg, DiagnosticSeverity::ERROR, "webfluent-io"));
            }
        }
    }

    // 2. Semantic Diagnostics (S01-S04: undefined components, duplicate declarations, broken routes)
    for diag in semantic_diagnostics {
        diagnostics.push(create_diagnostic(
            source,
            index,
            diag.line,
            diag.column,
            1,
            &diag.message,
            diag.hint.as_deref(),
            DiagnosticSeverity::ERROR,
            "webfluent-semantics",
            None,
        ));
    }

    // 3. Vocabulary Warnings (V01 dead bare words, V02 dead modifier classes)
    for warning in vocab_warnings {
        diagnostics.push(create_diagnostic(
            source,
            index,
            warning.line,
            warning.column,
            1,
            &warning.message,
            warning.hint.as_deref(),
            DiagnosticSeverity::WARNING,
            "webfluent-vocabulary",
            Some(&warning.rule_id),
        ));
    }

    // 4. Accessibility Warnings (A01-A12)
    for warning in a11y_warnings {
        diagnostics.push(create_diagnostic(
            source,
            index,
            warning.line,
            warning.column,
            1,
            &warning.message,
            Some(&warning.hint),
            DiagnosticSeverity::WARNING,
            "webfluent-a11y",
            Some(&warning.rule_id),
        ));
    }

    // 5. Contrast Warnings (A13)
    for warning in contrast_warnings {
        diagnostics.push(create_diagnostic(
            source,
            index,
            warning.line,
            warning.column,
            1,
            &warning.message,
            Some(&warning.hint),
            DiagnosticSeverity::WARNING,
            "webfluent-contrast",
            Some(&warning.rule_id),
        ));
    }

    client.publish_diagnostics(uri.clone(), diagnostics, None).await;
}

fn create_diagnostic(
    source: &str,
    index: &LineIndex,
    line: usize,
    column: usize,
    default_len: usize,
    message: &str,
    hint: Option<&str>,
    severity: DiagnosticSeverity,
    source_name: &str,
    code: Option<&str>,
) -> Diagnostic {
    // Try to determine word length at (line, column)
    let word_len = find_word_len(source, line, column).unwrap_or(default_len);
    let range = index.line_col_to_range(source, line, column, word_len);

    let full_message = if let Some(h) = hint {
        format!("{message}\n  Hint: {h}")
    } else {
        message.to_string()
    };

    Diagnostic {
        range,
        severity: Some(severity),
        code: code.map(|c| NumberOrString::String(c.to_string())),
        source: Some(source_name.to_string()),
        message: full_message,
        ..Default::default()
    }
}

fn simple_diagnostic(message: &str, severity: DiagnosticSeverity, source: &str) -> Diagnostic {
    Diagnostic {
        range: Range::default(),
        severity: Some(severity),
        source: Some(source.to_string()),
        message: message.to_string(),
        ..Default::default()
    }
}

/// Helper to measure the length of an identifier at (1-based line, 1-based col).
fn find_word_len(source: &str, line: usize, column: usize) -> Option<usize> {
    if line == 0 || column == 0 {
        return None;
    }
    let target_line = source.lines().nth(line - 1)?;
    let col_idx = column - 1;
    if col_idx >= target_line.len() {
        return None;
    }

    let remainder = &target_line[col_idx..];
    let len = remainder
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .map(|c| c.len_utf8())
        .sum::<usize>();

    if len > 0 { Some(len) } else { None }
}
