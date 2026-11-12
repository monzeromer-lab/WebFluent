//! Document symbols (the outline of one file) and workspace symbols (every
//! declaration in the project, for "go to symbol in project").
#![expect(
    deprecated,
    reason = "lsp-types 0.94 still requires the `deprecated` field when building a symbol; it is set to None"
)]

use lsp_types::*;
use webfluent::syntax::parser::ast::*;

use crate::line_index::LineIndex;
use crate::project::{Project, SourceFile};

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
    let mut symbols: Vec<DocumentSymbol> = Vec::new();
    for (_, decl) in project.declarations_of(file_ix) {
        let Some(mut sym) = declaration_symbol(decl, &file.source, &file.index) else {
            continue;
        };
        // A part (`Panel.Header`) is a component of its own, declared inside
        // its owner: the outline shows it there.
        if let Declaration::Component(c) = decl
            && let Some((owner, part)) = c.name.split_once('.')
            && let Some(owner_sym) = symbols
                .iter_mut()
                .rev()
                .find(|s| s.name == owner && s.kind == SymbolKind::CLASS)
        {
            sym.name = part.to_string();
            sym.detail = Some(format!("part of {owner}"));
            owner_sym.children.get_or_insert_with(Vec::new).push(sym);
            continue;
        }
        symbols.push(sym);
    }
    if hierarchical {
        DocumentSymbolResponse::Nested(symbols)
    } else {
        let mut flat = Vec::new();
        flatten(&symbols, &file.uri, None, &mut flat);
        DocumentSymbolResponse::Flat(flat)
    }
}

/// The outline entry of a declaration; `None` for a kind of declaration
/// this server does not know yet.
fn declaration_symbol(
    decl: &Declaration,
    source: &str,
    index: &LineIndex,
) -> Option<DocumentSymbol> {
    Some(match decl {
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
                    let at = index.span_to_range(source, prop.span);
                    let ty = crate::hover::type_name(&prop.prop_type);
                    let optional = prop.optional && !ty.ends_with('?');
                    symbol(
                        prop.name.clone(),
                        &format!("prop · {ty}{}", if optional { "?" } else { "" }),
                        SymbolKind::PROPERTY,
                        at,
                        at,
                        Vec::new(),
                    )
                })
                .collect();
            children.extend(comp.events.iter().map(|e| {
                let at = index.span_to_range(source, e.span);
                symbol(
                    e.name.clone(),
                    &format!("event · ({})", params_text(&e.params)),
                    SymbolKind::EVENT,
                    at,
                    at,
                    Vec::new(),
                )
            }));
            children.extend(comp.slots.iter().map(|slot| {
                let at = index.span_to_range(source, slot.span);
                let detail = if slot.params.is_empty() {
                    "slot".to_string()
                } else {
                    format!("slot · ({})", params_text(&slot.params))
                };
                symbol(
                    slot.name.clone().unwrap_or_else(|| "children".to_string()),
                    &detail,
                    SymbolKind::FIELD,
                    at,
                    at,
                    Vec::new(),
                )
            }));
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
                        &format!("field · {}", crate::hover::type_name(&f.ty)),
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
                        format!(".{}", c.name),
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
        Declaration::Const(c) => {
            let range = index.span_to_range(source, c.span);
            symbol(
                c.name.clone(),
                "Const",
                SymbolKind::CONSTANT,
                range,
                range,
                Vec::new(),
            )
        }
        Declaration::Animation(a) => {
            let range = index.span_to_range(source, a.span);
            symbol(
                a.name.clone(),
                "Animation",
                SymbolKind::OBJECT,
                range,
                range,
                Vec::new(),
            )
        }
        // A script's names, each where it is written.
        Declaration::Script(script) => {
            let children = script
                .names
                .iter()
                .map(|n| {
                    let at = index.word_range_at_line_col(source, n.line, n.col);
                    symbol(
                        n.name.clone(),
                        "script",
                        SymbolKind::FUNCTION,
                        at,
                        at,
                        Vec::new(),
                    )
                })
                .collect();
            let top = index.word_range_at_line_col(source, 1, 1);
            symbol(
                script.path.clone(),
                "Script",
                SymbolKind::FILE,
                top,
                top,
                children,
            )
        }
        // A service, and the endpoints it has, so the outline lists them.
        Declaration::Api(a) => {
            let range = index.span_to_range(source, a.span);
            symbol(
                a.name.clone(),
                "Api",
                SymbolKind::INTERFACE,
                range,
                range,
                a.endpoints
                    .iter()
                    .map(|e| {
                        let at = index.span_to_range(source, e.span);
                        symbol(
                            e.name.clone(),
                            &format!("{} {}", e.method, e.path),
                            SymbolKind::METHOD,
                            at,
                            at,
                            Vec::new(),
                        )
                    })
                    .collect(),
            )
        }
        Declaration::Data(d) => {
            let range = index.span_to_range(source, d.span);
            symbol(
                d.name.clone(),
                "Data",
                SymbolKind::CONSTANT,
                range,
                range,
                Vec::new(),
            )
        }
        Declaration::Test(t) => {
            let range = index.span_to_range(source, t.span);
            symbol(
                format!("test \"{}\"", t.name),
                "Test",
                SymbolKind::EVENT,
                range,
                range,
                Vec::new(),
            )
        }
        _ => return None,
    })
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
                if children.is_empty() {
                    symbols.extend(children);
                } else {
                    let name = crate::analysis::component_name(&el.component);
                    symbols.push(symbol(
                        name,
                        "element",
                        SymbolKind::OBJECT,
                        range,
                        range,
                        children,
                    ));
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
            StatementKind::Connection(c) => {
                let mut children = Vec::new();
                for h in &c.handlers {
                    children.push(handler_symbol(h, source, index));
                }
                symbols.push(symbol(
                    c.name.clone(),
                    connection_word(c.kind),
                    SymbolKind::VARIABLE,
                    range,
                    range,
                    children,
                ));
            }
            StatementKind::Validate(v) => symbols.push(symbol(
                format!("validate {}", v.name),
                &v.rules
                    .iter()
                    .map(|r| r.name.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                SymbolKind::PROPERTY,
                range,
                range,
                Vec::new(),
            )),
            StatementKind::Effect(e) => {
                let mut children = statement_symbols(&e.body, source, index);
                children.extend(statement_symbols(&e.cleanup, source, index));
                symbols.push(symbol(
                    "effect".to_string(),
                    "effect",
                    SymbolKind::EVENT,
                    range,
                    range,
                    children,
                ));
            }
            StatementKind::Timer(t) => symbols.push(symbol(
                if t.every { "every" } else { "after" }.to_string(),
                "timer",
                SymbolKind::EVENT,
                range,
                range,
                statement_symbols(&t.body, source, index),
            )),
            StatementKind::EventHandler(h) => symbols.push(handler_symbol(h, source, index)),
            StatementKind::Use(u) => symbols.push(symbol(
                format!("use {}", u.store_name),
                "use",
                SymbolKind::MODULE,
                range,
                range,
                Vec::new(),
            )),
            StatementKind::Try(t) => {
                symbols.extend(statement_symbols(&t.body, source, index));
                symbols.extend(statement_symbols(&t.catch_body, source, index));
            }
            _ => {}
        }
    }
    symbols
}

/// An `on …` handler, with what its body declares.
fn handler_symbol(h: &EventHandler, source: &str, index: &LineIndex) -> DocumentSymbol {
    let range = index.span_to_range(source, h.span);
    let name = match &h.key {
        Some(key) => format!("on key(\"{key}\")"),
        None => format!("on {}", h.event),
    };
    symbol(
        name,
        "handler",
        SymbolKind::EVENT,
        range,
        range,
        statement_symbols(&h.body, source, index),
    )
}

/// The word a connection is declared with.
fn connection_word(kind: ConnectionKind) -> &'static str {
    match kind {
        ConnectionKind::Socket => "socket",
        ConnectionKind::Stream => "stream",
        ConnectionKind::Channel => "channel",
        ConnectionKind::Peer => "peer",
        ConnectionKind::Worker => "worker",
        _ => "connection",
    }
}

/// `name: Type, other: Type`, for an event's or a slot's parameters.
fn params_text(params: &[ParamDecl]) -> String {
    params
        .iter()
        .map(|p| format!("{}: {}", p.name, crate::hover::type_name(&p.param_type)))
        .collect::<Vec<_>>()
        .join(", ")
}

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
            Declaration::Const(c) => (c.name.clone(), SymbolKind::CONSTANT, c.span, "Const"),
            Declaration::Animation(a) => (a.name.clone(), SymbolKind::OBJECT, a.span, "Animation"),
            Declaration::Test(t) => (
                format!("test \"{}\"", t.name),
                SymbolKind::EVENT,
                t.span,
                "Test",
            ),
            Declaration::Data(d) => (d.name.clone(), SymbolKind::CONSTANT, d.span, "Data"),
            Declaration::Api(a) => (a.name.clone(), SymbolKind::INTERFACE, a.span, "Api"),
            Declaration::App(app) => {
                let Some(first) = app.body.first() else {
                    continue;
                };
                let span = match app.body.last() {
                    Some(last) => Span {
                        end: last.span.end,
                        ..first.span
                    },
                    None => first.span,
                };
                ("App".to_string(), SymbolKind::CLASS, span, "App")
            }
            // A script's names, each where it is written.
            Declaration::Script(script) => {
                for n in &script.names {
                    if query.is_empty() || n.name.to_lowercase().contains(&query) {
                        out.push(SymbolInformation {
                            name: n.name.clone(),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            deprecated: None,
                            location: Location {
                                uri: file.uri.clone(),
                                range: file.index.word_range_at_line_col(
                                    &file.source,
                                    n.line,
                                    n.col,
                                ),
                            },
                            container_name: Some(script.path.clone()),
                        });
                    }
                }
                continue;
            }
            _ => continue,
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
