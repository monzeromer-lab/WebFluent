//! The compiler's findings, as LSP diagnostics, per file.
//!
//! Everything the compiler would say on `wf build` — a lexer or parser error,
//! the semantic checks, the vocabulary, accessibility and contrast lints — is
//! run over the merged project once and routed to the file each finding is
//! about. Severity follows the compiler: a broken reference stops a build, so
//! it is an error; a lint is a warning.

use tower_lsp::lsp_types::*;
use webfluent::error::{Diagnostic as WfDiagnostic, WebFluentError};
use webfluent::linter::{
    lint_accessibility_in, lint_contrast_in, lint_vocabulary_with, validate_semantics_in,
};
use webfluent::themes::resolve_tokens;

use crate::line_index::LineIndex;
use crate::project::Project;

/// Diagnostics for every file in the project, indexed like `project.files`.
pub fn project_diagnostics(project: &Project) -> Vec<Vec<Diagnostic>> {
    let mut out: Vec<Vec<Diagnostic>> = project.files.iter().map(|_| Vec::new()).collect();

    // A file that does not parse reports its one error and contributes no
    // declarations; the rest of the project is still checked without it.
    for (ix, file) in project.files.iter().enumerate() {
        if let Err(error) = &file.parsed
            && let Some(diagnostic) = parse_error(error, &file.source, &file.index)
        {
            out[ix].push(diagnostic);
        }
    }

    // The linters label each finding with the declaration's file; the label
    // here is the file's index, which routes it back.
    let file_count = out.len();
    let file_of = |decl_ix: usize| project.decl_file[decl_ix].to_string();
    let route = move |label: &str| label.parse::<usize>().ok().filter(|&ix| ix < file_count);

    for finding in validate_semantics_in(&project.program, &file_of) {
        if let Some(ix) = route(&finding.file) {
            let file = &project.files[ix];
            out[ix].push(diagnostic(
                &file.source,
                &file.index,
                finding.line,
                finding.column,
                &finding.message,
                finding.hint.as_deref(),
                DiagnosticSeverity::ERROR,
                None,
            ));
        }
    }

    let mut findings = webfluent::sema::check(&project.program, &file_of);
    let typed = webfluent::sema::types::check(&project.program, &file_of);
    findings.errors.extend(typed.findings.errors);
    findings.warnings.extend(typed.findings.warnings);
    for (finding, severity) in findings
        .errors
        .iter()
        .map(|d| (d, DiagnosticSeverity::ERROR))
        .chain(
            findings
                .warnings
                .iter()
                .map(|d| (d, DiagnosticSeverity::WARNING)),
        )
    {
        if let Some(ix) = route(&finding.file) {
            let file = &project.files[ix];
            out[ix].push(diagnostic(
                &file.source,
                &file.index,
                finding.line,
                finding.column,
                &finding.message,
                finding.hint.as_deref(),
                severity,
                None,
            ));
        }
    }

    // The linters read the vocabulary the generators do: a `.sm` flag is
    // judged by the class it produces. Lowering keeps every span.
    let lowered = webfluent::sema::lower(project.program.clone());
    for warning in lint_vocabulary_with(&lowered, &project.stylesheets, &file_of) {
        if let Some(ix) = route(&warning.file) {
            let file = &project.files[ix];
            out[ix].push(diagnostic(
                &file.source,
                &file.index,
                warning.line,
                warning.column,
                &warning.message,
                warning.hint.as_deref(),
                DiagnosticSeverity::WARNING,
                Some(&warning.rule_id),
            ));
        }
    }

    let mut a11y = lint_accessibility_in(&lowered, &file_of);
    if let Ok(tokens) = resolve_tokens(&lowered, &project.theme) {
        a11y.extend(lint_contrast_in(&lowered, &tokens, &file_of));
    }
    for warning in a11y {
        if let Some(ix) = route(&warning.file) {
            let file = &project.files[ix];
            out[ix].push(diagnostic(
                &file.source,
                &file.index,
                warning.line,
                warning.column,
                &warning.message,
                Some(&warning.hint),
                DiagnosticSeverity::WARNING,
                Some(&warning.rule_id),
            ));
        }
    }

    for diagnostics in &mut out {
        diagnostics.sort_by_key(|d| (d.range.start, d.range.end));
    }
    out
}

fn parse_error(error: &WebFluentError, source: &str, index: &LineIndex) -> Option<Diagnostic> {
    let (finding, what): (&WfDiagnostic, &str) = match error {
        WebFluentError::LexerError(d) => (d, "lexer"),
        WebFluentError::ParseError(d) => (d, "parser"),
        // The other variants come from stages the server never runs.
        _ => return None,
    };
    let mut diagnostic = self::diagnostic(
        source,
        index,
        finding.line,
        finding.column,
        &finding.message,
        finding.hint.as_deref(),
        DiagnosticSeverity::ERROR,
        None,
    );
    diagnostic.source = Some(format!("webfluent {what}"));
    Some(diagnostic)
}

#[allow(clippy::too_many_arguments)]
fn diagnostic(
    source: &str,
    index: &LineIndex,
    line: usize,
    column: usize,
    message: &str,
    hint: Option<&str>,
    severity: DiagnosticSeverity,
    code: Option<&str>,
) -> Diagnostic {
    let message = match hint {
        Some(hint) if !hint.is_empty() => format!("{message}\n{hint}"),
        _ => message.to_string(),
    };
    Diagnostic {
        range: index.word_range_at_line_col(source, line, column),
        severity: Some(severity),
        code: code.map(|c| NumberOrString::String(c.to_string())),
        source: Some("webfluent".to_string()),
        message,
        ..Default::default()
    }
}
