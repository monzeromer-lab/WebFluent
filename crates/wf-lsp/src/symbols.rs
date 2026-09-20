//! Document symbols (the outline of one file) and workspace symbols (every
//! declaration in the project, for "go to symbol in project").

use tower_lsp::lsp_types::*;
use webfluent::parser::ast::*;

use crate::line_index::LineIndex;
use crate::project::{Project, SourceFile};

#[allow(deprecated)]
fn symbol(
    name: String,
    detail: &str,
    kind: SymbolKind,
    range: Range,
    selection: Range,
    children: Vec<DocumentSymbol>,
) -> DocumentSymbol {
    DocumentSymbol {
        name,
        detail: Some(detail.to_string()),
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range: selection,
        children: if children.is_empty() {
            None
        } else {
            Some(children)
        },
    }
}

/// The outline of `file_ix`, nested when the client supports it.
pub fn document_symbols(
    project: &Project,
    file_ix: usize,
    hierarchical: bool,
) -> DocumentSymbolResponse {
    let file = &project.files[file_ix];
    let symbols: Vec<DocumentSymbol> = project
        .declarations_of(file_ix)
        .map(|(_, decl)| declaration_symbol(decl, &file.source, &file.index))
        .collect();
    if hierarchical {
        DocumentSymbolResponse::Nested(symbols)
    } else {
        let mut flat = Vec::new();
        flatten(&symbols, &file.uri, None, &mut flat);
        DocumentSymbolResponse::Flat(flat)
    }
}

fn declaration_symbol(decl: &Declaration, source: &str, index: &LineIndex) -> DocumentSymbol {
    match decl {
        Declaration::Page(page) => symbol(
            page.name.clone(),
            &format!("Page · {}", page.path),
            SymbolKind::CLASS,
            index.span_to_range(source, page.span),
            index.span_to_range(source, page.header_span),
            statement_symbols(&page.body, source, index),
        ),
        Declaration::Component(comp) => {
            let selection = index.span_to_range(source, comp.header_span);
            let mut children: Vec<DocumentSymbol> = comp
                .props
                .iter()
                .map(|prop| {
                    symbol(
                        prop.name.clone(),
                        &format!(
                            "prop · {:?}{}",
                            prop.prop_type,
                            if prop.optional { "?" } else { "" }
                        ),
                        SymbolKind::PROPERTY,
                        selection,
                        selection,
                        Vec::new(),
                    )
                })
                .collect();
            children.extend(statement_symbols(&comp.body, source, index));
            symbol(
                comp.name.clone(),
                "Component",
                SymbolKind::CLASS,
                index.span_to_range(source, comp.span),
                selection,
                children,
            )
        }
        Declaration::Store(store) => symbol(
            store.name.clone(),
            "Store",
            SymbolKind::MODULE,
            index.span_to_range(source, store.span),
            index.span_to_range(source, store.header_span),
            statement_symbols(&store.body, source, index),
        ),
        Declaration::Theme(theme) => {
            let range = index.span_to_range(source, theme.span);
            let children = theme
                .tokens
                .iter()
                .map(|token| {
                    let r = index.span_to_range(source, token.span);
                    symbol(
                        token.name.clone(),
                        &format!("token · var(--{})", token.name),
                        SymbolKind::CONSTANT,
                        r,
                        r,
                        Vec::new(),
                    )
                })
                .collect();
            symbol(
                theme.name.clone(),
                "Theme",
                SymbolKind::NAMESPACE,
                range,
                range,
                children,
            )
        }
        Declaration::App(app) => {
            let children = statement_symbols(&app.body, source, index);
            let range = match (app.body.first(), app.body.last()) {
                (Some(first), Some(last)) => Range::new(
                    index.span_to_range(source, first.span).start,
                    index.span_to_range(source, last.span).end,
                ),
                _ => Range::default(),
            };
            symbol(
                "App".to_string(),
                "App",
                SymbolKind::CLASS,
                range,
                range,
                children,
            )
        }
        Declaration::Type(t) => {
            let range = index.span_to_range(source, t.span);
            let children = t
                .fields
                .iter()
                .map(|f| {
                    let r = index.span_to_range(source, f.span);
                    symbol(
                        f.name.clone(),
                        &format!("field · {:?}", f.ty),
                        SymbolKind::FIELD,
                        r,
                        r,
                        Vec::new(),
                    )
                })
                .collect();
            symbol(
                t.name.clone(),
                "Type",
                SymbolKind::STRUCT,
                range,
                index.span_to_range(source, t.header_span),
                children,
            )
        }
        Declaration::Enum(e) => {
            let range = index.span_to_range(source, e.span);
            let children = e
                .cases
                .iter()
                .map(|c| {
                    let r = index.span_to_range(source, e.header_span);
                    symbol(
                        format!(".{c}"),
                        "case",
                        SymbolKind::ENUM_MEMBER,
                        r,
                        r,
                        Vec::new(),
                    )
                })
                .collect();
            symbol(
                e.name.clone(),
                "Enum",
                SymbolKind::ENUM,
                range,
                index.span_to_range(source, e.header_span),
                children,
            )
        }
    }
}

fn statement_symbols(stmts: &[Statement], source: &str, index: &LineIndex) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    for stmt in stmts {
        let range = index.span_to_range(source, stmt.span);
        match &stmt.kind {
            StatementKind::State(s) => symbols.push(symbol(
                s.name.clone(),
                "state",
                SymbolKind::VARIABLE,
                range,
                range,
                Vec::new(),
            )),
            StatementKind::Derived(d) => symbols.push(symbol(
                d.name.clone(),
                "derived",
                SymbolKind::VARIABLE,
                range,
                range,
                Vec::new(),
            )),
            StatementKind::Action(a) => {
                let params = a
                    .params
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                symbols.push(symbol(
                    format!("{}({params})", a.name),
                    "action",
                    SymbolKind::FUNCTION,
                    range,
                    range,
                    statement_symbols(&a.body, source, index),
                ));
            }
            StatementKind::Fetch(f) => symbols.push(symbol(
                f.variable.clone(),
                "fetch",
                SymbolKind::VARIABLE,
                range,
                range,
                Vec::new(),
            )),
            StatementKind::UIElement(el) => {
                // Elements that structure the file: the ones with a body.
                let children = statement_symbols(&el.children, source, index);
                if !children.is_empty() {
                    let name = crate::analysis::component_name(&el.component);
                    symbols.push(symbol(
                        name,
                        "element",
                        SymbolKind::OBJECT,
                        range,
                        range,
                        children,
                    ));
                } else {
                    symbols.extend(children);
                }
            }
            StatementKind::If(i) => {
                symbols.extend(statement_symbols(&i.then_body, source, index));
                for (_, body) in &i.else_if_branches {
                    symbols.extend(statement_symbols(body, source, index));
                }
                if let Some(body) = &i.else_body {
                    symbols.extend(statement_symbols(body, source, index));
                }
            }
            StatementKind::For(f) => symbols.extend(statement_symbols(&f.body, source, index)),
            StatementKind::Show(s) => symbols.extend(statement_symbols(&s.body, source, index)),
            StatementKind::Match(m) => {
                for arm in &m.arms {
                    symbols.extend(statement_symbols(&arm.body, source, index));
                }
            }
            StatementKind::Resource(r) => symbols.push(symbol(
                r.name.clone(),
                "resource",
                SymbolKind::VARIABLE,
                range,
                range,
                Vec::new(),
            )),
            _ => {}
        }
    }
    symbols
}

#[allow(deprecated)]
fn flatten(
    symbols: &[DocumentSymbol],
    uri: &Url,
    container: Option<&str>,
    out: &mut Vec<SymbolInformation>,
) {
    for sym in symbols {
        out.push(SymbolInformation {
            name: sym.name.clone(),
            kind: sym.kind,
            tags: None,
            deprecated: None,
            location: Location {
                uri: uri.clone(),
                range: sym.range,
            },
            container_name: container.map(str::to_string),
        });
        if let Some(children) = &sym.children {
            flatten(children, uri, Some(&sym.name), out);
        }
    }
}

/// Every declaration in the project whose name contains `query`
/// (case-insensitively); an empty query lists them all.
#[allow(deprecated)]
pub fn workspace_symbols(project: &Project, query: &str) -> Vec<SymbolInformation> {
    let query = query.to_lowercase();
    let mut out = Vec::new();
    for (ix, decl) in project.program.declarations.iter().enumerate() {
        let file: &SourceFile = project.file_of_declaration(ix);
        let (name, kind, span, container) = match decl {
            Declaration::Page(p) => (p.name.clone(), SymbolKind::CLASS, p.header_span, "Page"),
            Declaration::Component(c) => (
                c.name.clone(),
                SymbolKind::CLASS,
                c.header_span,
                "Component",
            ),
            Declaration::Store(s) => (s.name.clone(), SymbolKind::MODULE, s.header_span, "Store"),
            Declaration::Theme(t) => (t.name.clone(), SymbolKind::NAMESPACE, t.span, "Theme"),
            Declaration::Type(t) => (t.name.clone(), SymbolKind::STRUCT, t.header_span, "Type"),
            Declaration::Enum(e) => (e.name.clone(), SymbolKind::ENUM, e.header_span, "Enum"),
            Declaration::App(_) => continue,
        };
        if query.is_empty() || name.to_lowercase().contains(&query) {
            out.push(SymbolInformation {
                name,
                kind,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: file.uri.clone(),
                    range: file.index.span_to_range(&file.source, span),
                },
                container_name: Some(container.to_string()),
            });
        }
        if let Declaration::Store(s) = decl {
            for member in crate::analysis::store_members(s) {
                if query.is_empty() || member.name.to_lowercase().contains(&query) {
                    out.push(SymbolInformation {
                        name: member.name.clone(),
                        kind: match member.kind {
                            crate::analysis::BindingKind::Action => SymbolKind::FUNCTION,
                            _ => SymbolKind::VARIABLE,
                        },
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: file.uri.clone(),
                            range: file.index.span_to_range(&file.source, member.span),
                        },
                        container_name: Some(s.name.clone()),
                    });
                }
            }
        }
    }
    out
}
