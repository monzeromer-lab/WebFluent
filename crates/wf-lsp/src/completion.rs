//! Completions for the position, read from the tree and the project.
//!
//! What is offered depends on where the cursor is:
//!
//! - top level: the declaration snippets;
//! - a `Theme` body: `token` and the baseline token names;
//! - a `fetch` body: `loading`, `error`, `success`;
//! - a `style` block: CSS properties, pseudo-states, `@media`, and — after a
//!   colon — the design tokens as `var(--…)`;
//! - after `use`: the project's stores; after `on:`: the events;
//! - after `Store.`: that store's members; after `Card.`: its sub-components;
//! - inside an element's parentheses: the arguments and modifiers that
//!   element takes, and the names in scope;
//! - inside a body: the built-in and project components, the statement
//!   keywords, and the names in scope.
//!
//! Nothing is offered inside a string or a comment.

use tower_lsp::lsp_types::*;
use webfluent::lexer::{Token, TokenType};
use webfluent::parser::ast::*;
use webfluent::parser::vocabulary::MODIFIER_KEYWORDS;

use crate::analysis::{self, Binding, BindingKind};
use crate::project::Project;
use crate::reference::{self, ArgDoc, Place};

pub fn provide_completions(
    project: &Project,
    file_ix: usize,
    position: Position,
) -> Vec<CompletionItem> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let Some(offset) = file.index.position_to_offset(source, position) else {
        return Vec::new();
    };
    let tokens = analysis::tokens_of(file).unwrap_or_else(|| tokens_until_error(source));
    if analysis::in_string(&tokens, offset) || analysis::in_comment(source, &tokens, offset) {
        return Vec::new();
    }

    // The word being typed is part of the query the editor filters with; the
    // context is decided by what comes before it.
    let typing = analysis::token_at(&tokens, offset)
        .filter(|t| {
            t.offset < offset && matches!(t.token_type, TokenType::Identifier(_))
                || is_word_token(t)
        })
        .filter(|t| t.offset < offset && t.end >= offset);
    let anchor = typing.map(|t| t.offset).unwrap_or(offset);
    let previous: Vec<&Token> = tokens.iter().filter(|t| t.end <= anchor).collect();
    let last = previous.last().copied();
    let before_last = previous.len().checked_sub(2).map(|i| previous[i]);
    let source: &str = source;

    // `on:` — the lexer refuses `on:` with no name after it, so this is read
    // from the text rather than the tokens.
    let line_prefix = &source[..offset];
    let line_prefix = &line_prefix[line_prefix.rfind('\n').map(|i| i + 1).unwrap_or(0)..];
    let typed_word = line_prefix.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_');
    if typed_word.ends_with("on:") {
        return events();
    }

    if let Some(Token {
        token_type: TokenType::Use,
        ..
    }) = last
    {
        return stores(project);
    }

    if let Some(Token {
        token_type: TokenType::Dot,
        ..
    }) = last
    {
        return after_dot(project, before_last);
    }

    let mut items = Vec::new();

    // Blocks the tokens alone identify, whether or not the file parses.
    match enclosing_block(&previous) {
        Some(Block::Theme) => return theme_body(),
        Some(Block::Style) => return style_block(&previous),
        _ => {}
    }

    let Some(decl_ix) = analysis::declaration_at(project, file_ix, anchor.saturating_sub(1))
        .or_else(|| analysis::declaration_at(project, file_ix, anchor))
    else {
        // Outside every declaration: the file's top level — or a file that
        // does not parse, where the tree is no help.
        if file.parsed.is_ok() || previous.is_empty() {
            return top_level();
        }
        return fallback(project, &previous);
    };
    let decl = &project.program.declarations[decl_ix];

    let body = analysis::body_of(decl);
    let path = analysis::statement_path(body, anchor.saturating_sub(1));
    let innermost = path.last().copied();

    // Inside a `fetch { }` but not in one of its blocks: its three keywords.
    if let Some(Statement {
        kind: StatementKind::Fetch(f),
        ..
    }) = innermost
    {
        let in_block = analysis::child_bodies(innermost.unwrap())
            .into_iter()
            .any(|b| {
                b.iter()
                    .any(|s| analysis::contains(s.span, anchor.saturating_sub(1)))
            });
        if !in_block && fetch_body_open(&previous) {
            return fetch_blocks(f);
        }
    }

    let scope = analysis::scope_at(decl, anchor);

    if let Some(open) = open_paren_owner(&previous) {
        // Inside the parentheses of something. Whose?
        match open {
            ParenOwner::Element(name) => {
                items.extend(element_arguments(project, &name));
                items.extend(scope_items(&scope));
                return items;
            }
            ParenOwner::PageHeader => {
                items.extend(args(reference::PAGE_ATTRIBUTES, "Page attribute"));
                return items;
            }
            ParenOwner::Animate => {
                items.extend(animations());
                items.extend(args(reference::ANIMATE_ARGUMENTS, "animate option"));
                return items;
            }
            ParenOwner::FetchOptions => {
                items.extend(args(reference::FETCH_OPTIONS, "fetch option"));
                items.extend(scope_items(&scope));
                return items;
            }
            ParenOwner::Call | ParenOwner::Unknown => {
                items.extend(scope_items(&scope));
                return items;
            }
        }
    }

    // In a body: what a statement can start with, and the names in scope.
    let in_store = matches!(decl, Declaration::Store(_));
    if !in_store {
        items.extend(components(project));
    }
    items.extend(keywords(in_store));
    items.extend(scope_items(&scope));
    items
}

fn is_word_token(t: &Token) -> bool {
    matches!(
        t.token_type,
        TokenType::Identifier(_)
            | TokenType::Error
            | TokenType::Loading
            | TokenType::Success
            | TokenType::State
            | TokenType::Style
            | TokenType::Show
            | TokenType::Action
    ) || webfluent::lexer::token::component_name(&t.token_type).is_some()
}

/// Tokens of a source that fails to lex, up to the failure.
fn tokens_until_error(source: &str) -> Vec<Token> {
    // Lex line by line so a bad character late in the file does not blind
    // the whole of it; offsets are corrected to the file.
    let mut tokens = Vec::new();
    let mut base = 0;
    for line in source.split_inclusive('\n') {
        if let Ok(mut line_tokens) = webfluent::lexer::Lexer::new(line, "").tokenize() {
            line_tokens.retain(|t| !matches!(t.token_type, TokenType::EOF));
            for t in &mut line_tokens {
                t.offset += base;
                t.end += base;
            }
            tokens.extend(line_tokens);
        }
        base += line.len();
    }
    tokens
}

enum ParenOwner {
    Element(String),
    PageHeader,
    Animate,
    FetchOptions,
    Call,
    Unknown,
}

/// If the cursor is inside an unclosed `(`, what opened it.
fn open_paren_owner(previous: &[&Token]) -> Option<ParenOwner> {
    let mut depth = 0i32;
    for (ix, token) in previous.iter().enumerate().rev() {
        match token.token_type {
            TokenType::CloseParen => depth += 1,
            TokenType::OpenParen => {
                if depth > 0 {
                    depth -= 1;
                    continue;
                }
                let owner = previous.get(ix.checked_sub(1)?)?;
                return Some(match &owner.token_type {
                    TokenType::Animate => ParenOwner::Animate,
                    TokenType::Identifier(name) => {
                        // `Page Name (` — the page header.
                        if let Some(Token {
                            token_type: TokenType::Page,
                            ..
                        }) = ix.checked_sub(2).and_then(|i| previous.get(i)).copied()
                        {
                            ParenOwner::PageHeader
                        } else if name.chars().next().is_some_and(char::is_uppercase) {
                            // `Store.member(` is a call; a bare capitalised name is an element.
                            match ix.checked_sub(2).and_then(|i| previous.get(i)) {
                                Some(Token {
                                    token_type: TokenType::Dot,
                                    ..
                                }) => ParenOwner::Call,
                                _ => ParenOwner::Element(name.clone()),
                            }
                        } else {
                            ParenOwner::Call
                        }
                    }
                    TokenType::TypeList => ParenOwner::Element("List".to_string()),
                    // A built-in component keyword, or `Card.Header(`.
                    other => match webfluent::lexer::token::component_name(other) {
                        Some(name) => {
                            let full = match (
                                ix.checked_sub(2).and_then(|i| previous.get(i)),
                                ix.checked_sub(3).and_then(|i| previous.get(i)),
                            ) {
                                (
                                    Some(Token {
                                        token_type: TokenType::Dot,
                                        ..
                                    }),
                                    Some(parent),
                                ) => {
                                    match webfluent::lexer::token::component_name(
                                        &parent.token_type,
                                    ) {
                                        Some(p) => format!("{p}.{name}"),
                                        None => name.to_string(),
                                    }
                                }
                                _ => name.to_string(),
                            };
                            ParenOwner::Element(full)
                        }
                        None => {
                            // `fetch x from url (` — options.
                            if matches!(other, TokenType::StringLiteral(_) | TokenType::CloseParen)
                                && previous[..ix]
                                    .iter()
                                    .rev()
                                    .take(8)
                                    .any(|t| matches!(t.token_type, TokenType::From))
                            {
                                ParenOwner::FetchOptions
                            } else {
                                ParenOwner::Unknown
                            }
                        }
                    },
                });
            }
            TokenType::OpenBrace | TokenType::CloseBrace if depth == 0 => return None,
            _ => {}
        }
    }
    None
}

/// Whether the cursor is directly inside a `fetch … {` body.
fn fetch_body_open(previous: &[&Token]) -> bool {
    let mut depth = 0i32;
    for token in previous.iter().rev() {
        match token.token_type {
            TokenType::CloseBrace => depth += 1,
            TokenType::OpenBrace => {
                if depth > 0 {
                    depth -= 1;
                } else {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

enum Block {
    Style,
    Theme,
}

/// The nearest enclosing block the tokens alone identify: a `style { … }`
/// (at any depth: a pseudo-state or `@media` block inside one counts), or a
/// `Theme Name { … }` body.
fn enclosing_block(previous: &[&Token]) -> Option<Block> {
    let mut depth = 0i32;
    for (ix, token) in previous.iter().enumerate().rev() {
        match token.token_type {
            TokenType::CloseBrace => depth += 1,
            TokenType::OpenBrace => {
                if depth > 0 {
                    depth -= 1;
                    continue;
                }
                let opener = ix
                    .checked_sub(1)
                    .and_then(|i| previous.get(i))
                    .map(|t| &t.token_type);
                let before_opener = ix
                    .checked_sub(2)
                    .and_then(|i| previous.get(i))
                    .map(|t| &t.token_type);
                match (opener, before_opener) {
                    (Some(TokenType::Style), _) => return Some(Block::Style),
                    (Some(TokenType::Identifier(_)), Some(TokenType::Theme)) => {
                        return Some(Block::Theme);
                    }
                    // A pseudo-state or `@media` block: keep looking outward
                    // for the `style` that holds it. Any other opener is a
                    // body, and ends the search.
                    (Some(TokenType::Identifier(_)), _) | (Some(TokenType::CloseParen), _) => {}
                    _ => return None,
                }
            }
            _ => {}
        }
    }
    None
}

fn after_dot(project: &Project, owner: Option<&Token>) -> Vec<CompletionItem> {
    let Some(owner) = owner else {
        return Vec::new();
    };
    let name = match &owner.token_type {
        TokenType::Identifier(n) => n.clone(),
        other => match webfluent::lexer::token::component_name(other) {
            Some(n) => n.to_string(),
            None => return Vec::new(),
        },
    };
    if let Some(doc) = reference::component(&name) {
        return doc
            .children
            .iter()
            .map(|child| {
                let short = child.split('.').nth(1).unwrap_or(child);
                CompletionItem {
                    label: short.to_string(),
                    kind: Some(CompletionItemKind::CLASS),
                    detail: Some(format!("{child} — part of {}", doc.name)),
                    ..Default::default()
                }
            })
            .collect();
    }
    if let Some(store) = project.program.declarations.iter().find_map(|d| match d {
        Declaration::Store(s) if s.name == name => Some(s),
        _ => None,
    }) {
        return analysis::store_members(store)
            .into_iter()
            .map(|m| binding_item(&m, Some(&store.name)))
            .collect();
    }
    Vec::new()
}

fn events() -> Vec<CompletionItem> {
    reference::EVENTS
        .iter()
        .map(|(name, doc)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::EVENT),
            detail: Some(doc.to_string()),
            insert_text: Some(format!("{name} {{\n\t$0\n}}")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

fn stores(project: &Project) -> Vec<CompletionItem> {
    project
        .program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Store(s) => Some(CompletionItem {
                label: s.name.clone(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Store".to_string()),
                ..Default::default()
            }),
            _ => None,
        })
        .collect()
}

fn top_level() -> Vec<CompletionItem> {
    vec![
        snippet(
            "Page",
            "A routed page",
            "Page ${1:Name} (path: \"${2:/}\", title: \"${3:$1}\") {\n\t$0\n}",
        ),
        snippet(
            "Component",
            "A reusable component",
            "Component ${1:Name} (${2:label}: ${3:String}) {\n\t$0\n}",
        ),
        snippet(
            "Store",
            "Shared state",
            "Store ${1:Name}Store {\n\tstate ${2:items} = ${3:[]}\n\t$0\n}",
        ),
        snippet(
            "Theme",
            "Design tokens",
            "Theme ${1:Brand} {\n\ttoken color-primary: \"${2:#3B82F6}\"\n\t$0\n}",
        ),
        snippet(
            "App",
            "The root of the site",
            "App {\n\tRouter {\n\t\tRoute(path: \"/\", page: ${1:Home})\n\t}\n}",
        ),
    ]
}

fn theme_body() -> Vec<CompletionItem> {
    let mut items = vec![snippet(
        "token",
        "A design token",
        "token ${1:name}: \"${2:value}\"",
    )];
    items.extend(reference::TOKENS.iter().map(|(name, doc)| CompletionItem {
        label: format!("token {name}"),
        kind: Some(CompletionItemKind::CONSTANT),
        detail: Some(doc.to_string()),
        insert_text: Some(format!("token {name}: \"${{1:value}}\"")),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        filter_text: Some(name.to_string()),
        ..Default::default()
    }));
    items
}

fn fetch_blocks(fetch: &FetchDecl) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    if fetch.loading_block.is_none() {
        items.push(snippet(
            "loading",
            "While the request is in flight",
            "loading {\n\t$0\n}",
        ));
    }
    if fetch.error_block.is_none() {
        items.push(snippet(
            "error",
            "When the request fails",
            "error (${1:err}) {\n\t$0\n}",
        ));
    }
    if fetch.success_block.is_none() {
        items.push(snippet(
            "success",
            "Once the data arrived",
            "success {\n\t$0\n}",
        ));
    }
    items
}

fn style_block(previous: &[&Token]) -> Vec<CompletionItem> {
    // After `name:` a value is wanted: the design tokens, quoted.
    let after_colon = matches!(
        previous.last().map(|t| &t.token_type),
        Some(TokenType::Colon)
    );
    if after_colon {
        return reference::TOKENS
            .iter()
            .map(|(name, doc)| CompletionItem {
                label: format!("var(--{name})"),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some(doc.to_string()),
                insert_text: Some(format!("\"var(--{name})\"")),
                filter_text: Some(format!("var {name} {}", name.replace('-', " "))),
                ..Default::default()
            })
            .collect();
    }
    let mut items: Vec<CompletionItem> = reference::CSS_PROPERTIES
        .iter()
        .map(|prop| CompletionItem {
            label: prop.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some("CSS property".to_string()),
            insert_text: Some(format!("{prop}: \"$0\"")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("1{prop}")),
            ..Default::default()
        })
        .collect();
    items.extend(
        reference::pseudo_states()
            .iter()
            .map(|state| CompletionItem {
                label: state.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Pseudo-state rule".to_string()),
                insert_text: Some(format!("{state} {{\n\t$0\n}}")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("2{state}")),
                ..Default::default()
            }),
    );
    items.push(CompletionItem {
        label: "@media".to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some("Responsive rule".to_string()),
        insert_text: Some("@media (max-width: ${1:768px}) {\n\t$0\n}".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some("3@media".to_string()),
        ..Default::default()
    });
    items
}

fn element_arguments(project: &Project, name: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // A user component: its props, by name.
    if let Some(Declaration::Component(c)) = project
        .program
        .declarations
        .iter()
        .find(|d| matches!(d, Declaration::Component(c) if c.name == name))
    {
        for prop in &c.props {
            items.push(CompletionItem {
                label: format!("{}:", prop.name),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some(format!(
                    "{:?}{} — prop of {}",
                    prop.prop_type,
                    if prop.optional { "?" } else { "" },
                    c.name
                )),
                insert_text: Some(format!("{}: ", prop.name)),
                sort_text: Some(format!("0{}", prop.name)),
                ..Default::default()
            });
        }
        return items;
    }

    let doc = reference::component(name).or_else(|| reference::sub_component(name).map(|(d, _)| d));
    let (args_docs, modifiers): (&[ArgDoc], &[&str]) = match doc {
        Some(doc) if doc.name == name => (doc.args, doc.modifiers),
        _ => (&[], &[]),
    };
    if name == "Sidebar.Item" {
        items.extend(args(
            &[
                ArgDoc {
                    name: "to",
                    doc: "Route to navigate to",
                },
                ArgDoc {
                    name: "icon",
                    doc: "One of the built-in icon names",
                },
                ArgDoc {
                    name: "active",
                    doc: "`\"prefix\"` also matches routes beneath `to`",
                },
            ],
            "Sidebar.Item argument",
        ));
    } else if name == "Breadcrumb.Item" {
        items.extend(args(
            &[ArgDoc {
                name: "to",
                doc: "Route to navigate to; omit on the current page",
            }],
            "Breadcrumb.Item argument",
        ));
    }
    items.extend(args(args_docs, &format!("{name} argument")));

    // The modifiers it lists first, then the rest of the vocabulary.
    for m in modifiers {
        items.push(modifier_item(m, name, true));
    }
    for m in MODIFIER_KEYWORDS {
        if !modifiers.contains(m) {
            items.push(modifier_item(m, name, false));
        }
    }
    if name == "Icon" {
        items.extend(icons());
    }
    items
}

fn args(docs: &[ArgDoc], detail: &str) -> Vec<CompletionItem> {
    docs.iter()
        .map(|a| CompletionItem {
            label: format!("{}:", a.name),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(detail.to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: a.doc.to_string(),
            })),
            insert_text: Some(format!("{}: ", a.name)),
            sort_text: Some(format!("0{}", a.name)),
            ..Default::default()
        })
        .collect()
}

fn modifier_item(m: &str, component: &str, listed: bool) -> CompletionItem {
    let (group, doc) = reference::modifier_doc(m).unwrap_or(("Modifier", ""));
    CompletionItem {
        label: m.to_string(),
        kind: Some(CompletionItemKind::ENUM_MEMBER),
        detail: Some(if listed {
            format!("{group} — {component}")
        } else {
            group.to_string()
        }),
        documentation: (!doc.is_empty()).then(|| Documentation::String(doc.to_string())),
        sort_text: Some(format!("{}{m}", if listed { "1" } else { "3" })),
        ..Default::default()
    }
}

fn animations() -> Vec<CompletionItem> {
    [
        "fadeIn",
        "fadeOut",
        "slideUp",
        "slideDown",
        "slideLeft",
        "slideRight",
        "scaleIn",
        "scaleOut",
        "bounce",
        "shake",
        "pulse",
        "spin",
    ]
    .iter()
    .map(|a| CompletionItem {
        label: a.to_string(),
        kind: Some(CompletionItemKind::ENUM_MEMBER),
        detail: Some("Animation".to_string()),
        ..Default::default()
    })
    .collect()
}

fn icons() -> Vec<CompletionItem> {
    reference::ICONS
        .iter()
        .map(|icon| CompletionItem {
            label: format!("\"{icon}\""),
            kind: Some(CompletionItemKind::VALUE),
            detail: Some("Built-in icon".to_string()),
            filter_text: Some(icon.to_string()),
            sort_text: Some(format!("2{icon}")),
            ..Default::default()
        })
        .collect()
}

fn components(project: &Project) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = reference::COMPONENTS
        .iter()
        .map(|c| CompletionItem {
            label: c.name.to_string(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(format!("{} — {}", c.group, c.summary)),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```wf\n{}\n```", c.usage),
            })),
            sort_text: Some(format!("1{}", c.name)),
            ..Default::default()
        })
        .collect();
    for (ix, decl) in project.program.declarations.iter().enumerate() {
        if let Declaration::Component(c) = decl {
            let call = if c.props.is_empty() {
                c.name.clone()
            } else {
                format!(
                    "{}({})",
                    c.name,
                    c.props
                        .iter()
                        .enumerate()
                        .map(|(i, p)| format!("{}: ${{{}:{}}}", p.name, i + 1, p.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            items.push(CompletionItem {
                label: c.name.clone(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(format!(
                    "Component — {}",
                    project.label_of(project.decl_file[ix])
                )),
                insert_text: Some(call),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("0{}", c.name)),
                ..Default::default()
            });
        }
    }
    items
}

fn keywords(in_store: bool) -> Vec<CompletionItem> {
    reference::KEYWORDS
        .iter()
        .filter(|k| k.place == Place::Body)
        .filter(|k| {
            !in_store
                || matches!(
                    k.name,
                    "state"
                        | "derived"
                        | "action"
                        | "effect"
                        | "return"
                        | "if"
                        | "else"
                        | "for"
                        | "in"
                        | "log"
                        | "navigate"
                )
        })
        .map(|k| CompletionItem {
            label: k.name.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(k.summary.to_string()),
            insert_text: Some(keyword_snippet(k.name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("2{}", k.name)),
            ..Default::default()
        })
        .collect()
}

fn keyword_snippet(name: &str) -> String {
    match name {
        "state" => "state ${1:name} = ${2:0}".into(),
        "derived" => "derived ${1:name} = ${2:expression}".into(),
        "effect" => "effect {\n\t$0\n}".into(),
        "action" => "action ${1:name}(${2}) {\n\t$0\n}".into(),
        "use" => "use ${1:Store}".into(),
        "if" => "if ${1:condition} {\n\t$0\n}".into(),
        "else" => "else {\n\t$0\n}".into(),
        "for" => "for ${1:item} in ${2:items} {\n\t$0\n}".into(),
        "show" => "show ${1:visible} {\n\t$0\n}".into(),
        "fetch" => "fetch ${1:data} from \"${2:/api/}\" {\n\tloading { Spinner() }\n\terror (err) { Alert(\"{err.message}\", danger) }\n\tsuccess {\n\t\t$0\n\t}\n}".into(),
        "navigate" => "navigate(\"${1:/}\")".into(),
        "log" => "log(${1:value})".into(),
        "return" => "return $0".into(),
        "animate" => "animate(${1:target}, ${2:fadeIn})".into(),
        "style" => "style {\n\t${1:padding}: \"${2:1rem}\"\n\t$0\n}".into(),
        "transition" => "transition {\n\t${1:background} ${2:200ms} ${3:ease}\n}".into(),
        other => other.into(),
    }
}

fn scope_items(scope: &[Binding]) -> Vec<CompletionItem> {
    let mut seen = std::collections::HashSet::new();
    scope
        .iter()
        .filter(|b| seen.insert(b.name.clone()))
        .map(|b| binding_item(b, None))
        .collect()
}

fn binding_item(b: &Binding, store: Option<&str>) -> CompletionItem {
    let kind = match b.kind {
        BindingKind::Action => CompletionItemKind::FUNCTION,
        BindingKind::Store => CompletionItemKind::MODULE,
        BindingKind::Prop | BindingKind::Param => CompletionItemKind::VARIABLE,
        _ => CompletionItemKind::VARIABLE,
    };
    let detail = match store {
        Some(s) => format!("{} of {s}", b.kind.label()),
        None => b.kind.label().to_string(),
    };
    CompletionItem {
        label: b.name.clone(),
        kind: Some(kind),
        detail: Some(detail),
        insert_text: (b.kind == BindingKind::Action).then(|| format!("{}($0)", b.name)),
        insert_text_format: (b.kind == BindingKind::Action).then_some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("0{}", b.name)),
        ..Default::default()
    }
}

/// When the file does not parse, offer by the nearest shape: components and
/// keywords in a body, arguments of the element whose parenthesis is open.
fn fallback(project: &Project, previous: &[&Token]) -> Vec<CompletionItem> {
    if let Some(ParenOwner::Element(name)) = open_paren_owner(previous) {
        return element_arguments(project, &name);
    }
    let mut items = components(project);
    items.extend(keywords(false));
    items
}

fn snippet(label: &str, detail: &str, body: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(detail.to_string()),
        insert_text: Some(body.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}
