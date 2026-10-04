//! Folding ranges: every declaration, and every statement with a block of
//! its own, that runs over more than one line.

use tower_lsp::lsp_types::*;
use webfluent::parser::ast::*;

use crate::analysis;
use crate::project::Project;

pub fn folding_ranges(project: &Project, file_ix: usize) -> Vec<FoldingRange> {
    let file = &project.files[file_ix];
    if file.script || file.markdown {
        return Vec::new();
    }
    let source: &str = &file.source;
    let mut out = Vec::new();
    let fold = |span: Span, out: &mut Vec<FoldingRange>| {
        let range = file.index.span_to_range(source, span);
        // The closing line stays visible, as an editor folds a block.
        if range.end.line > range.start.line {
            out.push(FoldingRange {
                start_line: range.start.line,
                start_character: None,
                end_line: range.end.line.saturating_sub(1).max(range.start.line),
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        }
    };
    fn walk(stmts: &[Statement], fold: &mut dyn FnMut(Span)) {
        for stmt in stmts {
            let bodies = analysis::child_bodies(stmt);
            let has_block = !bodies.is_empty()
                || matches!(&stmt.kind, StatementKind::UIElement(el) if el.style_block.is_some() || el.body_span.is_some());
            if has_block {
                fold(stmt.span);
            }
            for body in bodies {
                walk(body, fold);
            }
        }
    }
    for (_, decl) in project.declarations_of(file_ix) {
        if let Some(span) = analysis::decl_span(decl) {
            fold(span, &mut out);
        }
        let mut statements = Vec::new();
        walk(analysis::body_of(decl), &mut |span| statements.push(span));
        for span in statements {
            fold(span, &mut out);
        }
    }
    out.sort_by_key(|r| (r.start_line, r.end_line));
    out.dedup_by_key(|r| (r.start_line, r.end_line));
    out
}
