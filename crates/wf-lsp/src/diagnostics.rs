//! The compiler's findings, as LSP diagnostics, per file.
//!
//! The server runs the build's own pipeline —
//! [`webfluent::diagnostics::check::check_project`] — over the merged
//! project, so the editor shows exactly what `wf build` refuses: the
//! scripts, the config, `env`, the output mode and every lint included.
//! Each finding is routed to its file by path, and carries its code, a link
//! to its entry in the guide, its whole range, the other places it
//! concerns, and an `Unnecessary` tag where it points at code nothing needs.

use std::collections::HashMap;
use tower_lsp::lsp_types::*;
use webfluent::diagnostics::{Diagnostic as WfDiagnostic, Severity, Tag};

use crate::line_index::LineIndex;
use crate::project::Project;

/// Diagnostics for every file in the project, indexed like `project.files`.
pub fn project_diagnostics(project: &Project) -> Vec<Vec<Diagnostic>> {
    let mut out: Vec<Vec<Diagnostic>> = project.files.iter().map(|_| Vec::new()).collect();
    let labels: Vec<String> = (0..project.files.len())
        .map(|ix| project.label_of(ix))
        .collect();
    let by_label: HashMap<&str, usize> = labels
        .iter()
        .enumerate()
        .map(|(ix, l)| (l.as_str(), ix))
        .collect();

    // A file that does not parse reports each of its mistakes; the rest of
    // the project is still checked without it.
    let mut findings: Vec<(usize, WfDiagnostic)> = Vec::new();
    let mut incomplete = false;
    for (ix, file) in project.files.iter().enumerate() {
        if file.script || file.parsed.is_ok() {
            continue;
        }
        if !file.stale {
            incomplete = true;
        }
        let (_, errors) = webfluent::syntax::parse_source_recovering(&file.source, &labels[ix]);
        findings.extend(errors.into_iter().map(|d| (ix, d)));
    }

    let file_of = |decl_ix: usize| labels[project.decl_file[decl_ix]].clone();
    let source_of = |decl_ix: usize| {
        project
            .files
            .get(project.decl_file[decl_ix])
            .map(|f| f.source.to_string())
    };
    let config = project
        .root
        .as_ref()
        .and_then(|root| webfluent::config::ProjectConfig::load(root).ok());
    let scripts = project
        .root
        .as_ref()
        .and_then(|root| webfluent::project_js::load(root, &root.join("src")).ok())
        .unwrap_or_default();
    let declaration_files: Vec<String> = project
        .decl_file
        .iter()
        .map(|&ix| labels[ix].clone())
        .collect();
    let checked =
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &project.program,
            file_of: &file_of,
            source_of: &source_of,
            dir: project.root.as_deref(),
            config: config.as_ref(),
            declaration_files: &declaration_files,
            scripts: &scripts,
            stylesheets: &project.stylesheets,
            incomplete,
        });
    for finding in checked.diagnostics {
        if let Some(&ix) = by_label.get(finding.file.as_str()) {
            findings.push((ix, finding));
        }
    }

    for (ix, finding) in findings {
        let file = &project.files[ix];
        let related = finding
            .related
            .iter()
            .filter_map(|r| {
                let &rix = by_label.get(r.file.as_str())?;
                let other = &project.files[rix];
                Some(DiagnosticRelatedInformation {
                    location: Location::new(
                        other.uri.clone(),
                        other
                            .index
                            .word_range_at_line_col(&other.source, r.line, r.column),
                    ),
                    message: r.message.clone(),
                })
            })
            .collect();
        out[ix].push(to_lsp(&finding, &file.source, &file.index, related));
    }

    for diagnostics in &mut out {
        diagnostics.sort_by_key(|d| (d.range.start, d.range.end));
    }
    out
}

/// The range a finding covers: from its start to its end when the compiler
/// knows the end, the word at its start when it does not.
pub fn range_of(finding: &WfDiagnostic, source: &str, index: &LineIndex) -> Range {
    if finding.end_line > 0
        && let (Some(start), Some(end)) = (
            index.line_col_to_offset(source, finding.line, finding.column),
            index.line_col_to_offset(source, finding.end_line, finding.end_column),
        )
        && end > start
    {
        return Range::new(
            index.offset_to_position(source, start),
            index.offset_to_position(source, end),
        );
    }
    index.word_range_at_line_col(source, finding.line, finding.column)
}

/// One finding as the editor reads it.
pub fn to_lsp(
    finding: &WfDiagnostic,
    source: &str,
    index: &LineIndex,
    related: Vec<DiagnosticRelatedInformation>,
) -> Diagnostic {
    let message = match &finding.hint {
        Some(hint) if !hint.is_empty() => format!("{}\n{hint}", finding.message),
        _ => finding.message.clone(),
    };
    let severity = match finding.severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Info => DiagnosticSeverity::INFORMATION,
    };
    let tags: Vec<DiagnosticTag> = finding
        .tags
        .iter()
        .map(|t| match t {
            Tag::Unnecessary => DiagnosticTag::UNNECESSARY,
            Tag::Deprecated => DiagnosticTag::DEPRECATED,
        })
        .collect();
    let code = (!finding.code.is_empty()).then(|| NumberOrString::String(finding.code.to_string()));
    let code_description = finding
        .docs_url()
        .and_then(|url| Url::parse(&url).ok())
        .map(|href| CodeDescription { href });
    Diagnostic {
        range: range_of(finding, source, index),
        severity: Some(severity),
        code,
        code_description,
        source: Some("webfluent".to_string()),
        message,
        related_information: (!related.is_empty()).then_some(related),
        tags: (!tags.is_empty()).then_some(tags),
        data: fixes(finding, source, index),
    }
}

/// A finding's fixes as the editor applies them — a title and LSP text
/// edits — carried in the diagnostic's `data`, which a client hands back
/// with a code-action request.
fn fixes(finding: &WfDiagnostic, source: &str, index: &LineIndex) -> Option<serde_json::Value> {
    let fixes: Vec<serde_json::Value> = finding
        .fixes
        .iter()
        .map(|fix| {
            let edits: Vec<TextEdit> = fix
                .edits
                .iter()
                .filter_map(|e| {
                    let start = index.line_col_to_offset(source, e.line, e.column)?;
                    let end = index.line_col_to_offset(source, e.end_line, e.end_column)?;
                    Some(TextEdit {
                        range: Range::new(
                            index.offset_to_position(source, start),
                            index.offset_to_position(source, end),
                        ),
                        new_text: e.text.clone(),
                    })
                })
                .collect();
            serde_json::json!({ "title": fix.title, "edits": edits })
        })
        .collect();
    (!fixes.is_empty()).then_some(serde_json::Value::Array(fixes))
}
