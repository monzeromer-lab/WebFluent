use tower_lsp::lsp_types::*;
use webfluent::parser::ast::*;

use crate::line_index::LineIndex;

/// Build hierarchical document symbols (or flat symbols) from a parsed program.
#[allow(deprecated)]
pub fn build_document_symbols(
    program: &Program,
    source: &str,
    index: &LineIndex,
    uri: &Url,
    supports_hierarchical: bool,
) -> DocumentSymbolResponse {
    let mut nested_symbols = Vec::new();

    for decl in &program.declarations {
        match decl {
            Declaration::Page(page) => {
                let range = index.span_to_range(source, page.span);
                let selection_range = index.span_to_range(source, page.header_span);
                let children = collect_statement_symbols(&page.body, source, index);

                nested_symbols.push(DocumentSymbol {
                    name: page.name.clone(),
                    detail: Some(format!("Route: {}", page.path)),
                    kind: SymbolKind::CLASS,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range,
                    children: if children.is_empty() { None } else { Some(children) },
                });
            }

            Declaration::Component(comp) => {
                let range = index.span_to_range(source, comp.span);
                let selection_range = index.span_to_range(source, comp.header_span);
                let mut children = Vec::new();

                // Props
                for prop in &comp.props {
                    children.push(DocumentSymbol {
                        name: prop.name.clone(),
                        detail: Some(format!("{:?}{}", prop.prop_type, if prop.optional { "?" } else { "" })),
                        kind: SymbolKind::PROPERTY,
                        tags: None,
                        deprecated: None,
                        range: selection_range,
                        selection_range,
                        children: None,
                    });
                }

                children.extend(collect_statement_symbols(&comp.body, source, index));

                nested_symbols.push(DocumentSymbol {
                    name: comp.name.clone(),
                    detail: Some("Component".to_string()),
                    kind: SymbolKind::CLASS,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range,
                    children: if children.is_empty() { None } else { Some(children) },
                });
            }

            Declaration::Store(store) => {
                let range = index.span_to_range(source, store.span);
                let selection_range = index.span_to_range(source, store.header_span);
                let children = collect_statement_symbols(&store.body, source, index);

                nested_symbols.push(DocumentSymbol {
                    name: store.name.clone(),
                    detail: Some("Store".to_string()),
                    kind: SymbolKind::MODULE,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range,
                    children: if children.is_empty() { None } else { Some(children) },
                });
            }

            Declaration::Theme(theme) => {
                let range = index.span_to_range(source, theme.span);
                let mut children = Vec::new();

                for token in &theme.tokens {
                    let token_range = index.span_to_range(source, token.span);
                    children.push(DocumentSymbol {
                        name: token.name.clone(),
                        detail: Some(format!("var(--{})", token.name)),
                        kind: SymbolKind::CONSTANT,
                        tags: None,
                        deprecated: None,
                        range: token_range,
                        selection_range: token_range,
                        children: None,
                    });
                }

                nested_symbols.push(DocumentSymbol {
                    name: theme.name.clone(),
                    detail: Some("Theme".to_string()),
                    kind: SymbolKind::NAMESPACE,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: if children.is_empty() { None } else { Some(children) },
                });
            }

            Declaration::App(app) => {
                let children = collect_statement_symbols(&app.body, source, index);
                let range = children.first().map(|c| c.range).unwrap_or_default();

                nested_symbols.push(DocumentSymbol {
                    name: "App".to_string(),
                    detail: Some("Root Application".to_string()),
                    kind: SymbolKind::CLASS,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: if children.is_empty() { None } else { Some(children) },
                });
            }
        }
    }

    if supports_hierarchical {
        DocumentSymbolResponse::Nested(nested_symbols)
    } else {
        // Flatten nested symbols for older flat SymbolInformation clients
        let mut flat_symbols = Vec::new();
        flatten_symbols(&nested_symbols, uri, None, &mut flat_symbols);
        DocumentSymbolResponse::Flat(flat_symbols)
    }
}

#[allow(deprecated)]
fn collect_statement_symbols(
    stmts: &[Statement],
    source: &str,
    index: &LineIndex,
) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    for stmt in stmts {
        let stmt_range = index.span_to_range(source, stmt.span);
        match &stmt.kind {
            StatementKind::State(s) => {
                symbols.push(DocumentSymbol {
                    name: s.name.clone(),
                    detail: Some("state".to_string()),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    deprecated: None,
                    range: stmt_range,
                    selection_range: stmt_range,
                    children: None,
                });
            }
            StatementKind::Derived(d) => {
                symbols.push(DocumentSymbol {
                    name: d.name.clone(),
                    detail: Some("derived".to_string()),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    deprecated: None,
                    range: stmt_range,
                    selection_range: stmt_range,
                    children: None,
                });
            }
            StatementKind::Action(a) => {
                let params = a.params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ");
                let children = collect_statement_symbols(&a.body, source, index);
                symbols.push(DocumentSymbol {
                    name: format!("{}({})", a.name, params),
                    detail: Some("action".to_string()),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    deprecated: None,
                    range: stmt_range,
                    selection_range: stmt_range,
                    children: if children.is_empty() { None } else { Some(children) },
                });
            }
            StatementKind::UIElement(el) => {
                let name = match &el.component {
                    ComponentRef::BuiltIn(n) => n.clone(),
                    ComponentRef::SubComponent(p, c) => format!("{p}.{c}"),
                    ComponentRef::UserDefined(n) => n.clone(),
                };
                let children = collect_statement_symbols(&el.children, source, index);
                symbols.push(DocumentSymbol {
                    name,
                    detail: None,
                    kind: SymbolKind::FIELD,
                    tags: None,
                    deprecated: None,
                    range: stmt_range,
                    selection_range: stmt_range,
                    children: if children.is_empty() { None } else { Some(children) },
                });
            }
            _ => {}
        }
    }

    symbols
}

#[allow(deprecated)]
fn flatten_symbols(
    nested: &[DocumentSymbol],
    uri: &Url,
    container: Option<&str>,
    out: &mut Vec<SymbolInformation>,
) {
    for sym in nested {
        out.push(SymbolInformation {
            name: sym.name.clone(),
            kind: sym.kind,
            tags: None,
            deprecated: None,
            location: Location {
                uri: uri.clone(),
                range: sym.range,
            },
            container_name: container.map(ToString::to_string),
        });

        if let Some(children) = &sym.children {
            flatten_symbols(children, uri, Some(&sym.name), out);
        }
    }
}
