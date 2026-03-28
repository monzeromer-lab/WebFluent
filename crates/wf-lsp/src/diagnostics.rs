use tower_lsp::Client;
use tower_lsp::lsp_types::*;
use webfluent::error::{A11yWarning, WebFluentError};

/// Convert WebFluent errors and accessibility warnings into LSP diagnostics
/// and publish them to the client.
pub async fn publish_diagnostics(
    client: &Client,
    uri: &Url,
    errors: &[WebFluentError],
    a11y_warnings: &[A11yWarning],
) {
    let mut diagnostics = Vec::new();

    for err in errors {
        match err {
            WebFluentError::LexerError(diag) => {
                diagnostics.push(wf_diagnostic_to_lsp(
                    &diag.message,
                    diag.line,
                    diag.column,
                    DiagnosticSeverity::ERROR,
                    "webfluent-lexer",
                    diag.hint.as_deref(),
                ));
            }
            WebFluentError::ParseError(diag) => {
                diagnostics.push(wf_diagnostic_to_lsp(
                    &diag.message,
                    diag.line,
                    diag.column,
                    DiagnosticSeverity::ERROR,
                    "webfluent-parser",
                    diag.hint.as_deref(),
                ));
            }
            WebFluentError::CodegenError(msg) => {
                diagnostics.push(simple_diagnostic(
                    msg,
                    DiagnosticSeverity::ERROR,
                    "webfluent-codegen",
                ));
            }
            WebFluentError::ConfigError(msg) => {
                diagnostics.push(simple_diagnostic(
                    msg,
                    DiagnosticSeverity::ERROR,
                    "webfluent-config",
                ));
            }
            WebFluentError::IoError(msg) => {
                diagnostics.push(simple_diagnostic(
                    msg,
                    DiagnosticSeverity::ERROR,
                    "webfluent-io",
                ));
            }
        }
    }

    for warning in a11y_warnings {
        let message = format!("[{}] {}\n  Hint: {}", warning.rule_id, warning.message, warning.hint);
        diagnostics.push(wf_diagnostic_to_lsp(
            &message,
            warning.line,
            warning.column,
            DiagnosticSeverity::WARNING,
            "webfluent-a11y",
            None,
        ));
    }

    client
        .publish_diagnostics(uri.clone(), diagnostics, None)
        .await;
}

/// Convert a WebFluent diagnostic (1-based line/column) to an LSP diagnostic (0-based).
fn wf_diagnostic_to_lsp(
    message: &str,
    line: usize,
    column: usize,
    severity: DiagnosticSeverity,
    source: &str,
    hint: Option<&str>,
) -> Diagnostic {
    // WebFluent uses 1-based; LSP uses 0-based. Guard against 0 values.
    let lsp_line = if line > 0 { line - 1 } else { 0 } as u32;
    let lsp_col = if column > 0 { column - 1 } else { 0 } as u32;

    let full_message = if let Some(h) = hint {
        format!("{}\n  Hint: {}", message, h)
    } else {
        message.to_string()
    };

    Diagnostic {
        range: Range {
            start: Position::new(lsp_line, lsp_col),
            end: Position::new(lsp_line, lsp_col + 1),
        },
        severity: Some(severity),
        source: Some(source.to_string()),
        message: full_message,
        ..Default::default()
    }
}

/// Create a simple diagnostic at line 0, col 0 (for errors without location info).
fn simple_diagnostic(message: &str, severity: DiagnosticSeverity, source: &str) -> Diagnostic {
    Diagnostic {
        range: Range::default(),
        severity: Some(severity),
        source: Some(source.to_string()),
        message: message.to_string(),
        ..Default::default()
    }
}
