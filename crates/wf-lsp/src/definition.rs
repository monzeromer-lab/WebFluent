use tower_lsp::lsp_types::*;
use webfluent::parser::ast::*;

use crate::line_index::LineIndex;

/// Find definition location for the symbol under cursor.
pub fn find_definition(
    program: &Program,
    source: &str,
    index: &LineIndex,
    uri: &Url,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let word = word_at_position(source, position)?;

    // 1. Check for Component declaration
    for decl in &program.declarations {
        if let Declaration::Component(c) = decl {
            if c.name == word {
                let range = index.span_to_range(source, c.header_span);
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range,
                }));
            }
            // Check props
            for p in &c.props {
                if p.name == word {
                    let range = index.span_to_range(source, c.header_span);
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: uri.clone(),
                        range,
                    }));
                }
            }
        }
    }

    // 2. Check for Page declaration
    for decl in &program.declarations {
        if let Declaration::Page(p) = decl {
            if p.name == word {
                let range = index.span_to_range(source, p.header_span);
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range,
                }));
            }
        }
    }

    // 3. Check for Store declaration
    for decl in &program.declarations {
        if let Declaration::Store(s) = decl {
            if s.name == word {
                let range = index.span_to_range(source, s.header_span);
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range,
                }));
            }
        }
    }

    // 4. Check for Theme and Theme tokens
    for decl in &program.declarations {
        if let Declaration::Theme(t) = decl {
            if t.name == word {
                let range = index.span_to_range(source, t.span);
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range,
                }));
            }
            for token in &t.tokens {
                if token.name == word {
                    let range = index.span_to_range(source, token.span);
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: uri.clone(),
                        range,
                    }));
                }
            }
        }
    }

    // 5. Check in-scope statements (state, derived, action)
    for decl in &program.declarations {
        let stmts = match decl {
            Declaration::Page(p) => &p.body,
            Declaration::Component(c) => &c.body,
            Declaration::Store(s) => &s.body,
            Declaration::App(a) => &a.body,
            Declaration::Theme(_) => continue,
        };

        if let Some(loc) = find_stmt_def(stmts, source, index, uri, &word) {
            return Some(GotoDefinitionResponse::Scalar(loc));
        }
    }

    None
}

fn find_stmt_def(
    stmts: &[Statement],
    source: &str,
    index: &LineIndex,
    uri: &Url,
    word: &str,
) -> Option<Location> {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::State(s) if s.name == word => {
                return Some(Location {
                    uri: uri.clone(),
                    range: index.span_to_range(source, stmt.span),
                });
            }
            StatementKind::Derived(d) if d.name == word => {
                return Some(Location {
                    uri: uri.clone(),
                    range: index.span_to_range(source, stmt.span),
                });
            }
            StatementKind::Action(a) if a.name == word => {
                return Some(Location {
                    uri: uri.clone(),
                    range: index.span_to_range(source, stmt.span),
                });
            }
            StatementKind::UIElement(el) => {
                if let Some(loc) = find_stmt_def(&el.children, source, index, uri, word) {
                    return Some(loc);
                }
            }
            StatementKind::If(i) => {
                if let Some(loc) = find_stmt_def(&i.then_body, source, index, uri, word) {
                    return Some(loc);
                }
                if let Some(else_body) = &i.else_body {
                    if let Some(loc) = find_stmt_def(else_body, source, index, uri, word) {
                        return Some(loc);
                    }
                }
            }
            StatementKind::For(f) => {
                if let Some(loc) = find_stmt_def(&f.body, source, index, uri, word) {
                    return Some(loc);
                }
            }
            _ => {}
        }
    }
    None
}

fn word_at_position(source: &str, position: Position) -> Option<String> {
    let line = source.lines().nth(position.line as usize)?;
    let col = position.character as usize;

    if col > line.len() {
        return None;
    }

    let bytes = line.as_bytes();

    let mut start = col;
    while start > 0 {
        let ch = bytes[start - 1] as char;
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            start -= 1;
        } else {
            break;
        }
    }

    let mut end = col;
    while end < bytes.len() {
        let ch = bytes[end] as char;
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            end += 1;
        } else {
            break;
        }
    }

    if start == end {
        return None;
    }

    Some(line[start..end].to_string())
}
