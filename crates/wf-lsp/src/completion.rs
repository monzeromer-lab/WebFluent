//! Completions for the position, read from the tokens, the tree and the
//! project.
//!
//! What is offered depends on where the cursor is:
//!
//! - the top level: the declaration snippets;
//! - a `theme` body: the baseline token names;
//! - a `style` block: CSS properties, nested selectors, `@media` — and in a
//!   value, the design tokens as `$name`;
//! - after `use`: the project's stores; after `on`: the events; after
//!   `emit`: the component's own events;
//! - after `)` and `.`, or a flag and `.`: the element's flags; after
//!   `Card.`: its parts and flags; after `Store.`: that store's members;
//!   after `tone: .`: the cases the prop takes;
//! - inside an element's parentheses: the props it takes and the names in
//!   scope; a page header: its attributes; `layout:`: the components;
//! - a `match` body: the arms it still lacks;
//! - a component's block: the slots it declares;
//! - a body: the built-in and project components, the statement keywords
//!   and the names in scope.
//!
//! Nothing is offered inside a string or a comment.

use tower_lsp::lsp_types::*;
use webfluent::lexer::{Token, TokenType};
use webfluent::parser::ast::*;
use webfluent::registry::{self, Children, ComponentSig, PropType};
use webfluent::sema::types::Type;

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
            matches!(
                t.token_type,
                TokenType::Identifier(_) | TokenType::DesignToken(_)
            )
        })
        .filter(|t| t.offset < offset && t.end >= offset);
    let anchor = typing.map(|t| t.offset).unwrap_or(offset);
    let previous: Vec<&Token> = tokens.iter().filter(|t| t.end <= anchor).collect();
    let last = previous.last().copied();
    let before_last = previous.len().checked_sub(2).map(|i| previous[i]);

    let line_prefix = &source[..offset];
    let line_prefix = &line_prefix[line_prefix.rfind('\n').map(|i| i + 1).unwrap_or(0)..];
    let typed = line_prefix.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_' || c == '-');
    let in_style = matches!(enclosing_block(&previous), Some(Block::Style));

    // `$` — a design token, wherever it is typed; the lexer refuses a bare
    // `$`, so this reads the text.
    if typed.ends_with('$') {
        return design_tokens(
            project,
            in_style.then(|| style_property_of(&previous)).flatten(),
        );
    }

    match enclosing_block(&previous) {
        Some(Block::Theme) => return theme_body(),
        Some(Block::Style) => return style_block(project, &previous, offset, &tokens),
        Some(Block::Transition) => return transition_block(),
        None => {}
    }

    match last.map(|t| &t.token_type) {
        Some(TokenType::Identifier(w)) if w == "use" && statement_start(&previous, 1) => {
            return stores(project);
        }
        Some(TokenType::Identifier(w)) if w == "on" && statement_start(&previous, 1) => {
            return events(project, file_ix, anchor);
        }
        Some(TokenType::Identifier(w)) if w == "emit" => {
            return declared_events(project, file_ix, anchor);
        }
        Some(TokenType::Dot) => return after_dot(project, file_ix, &previous, anchor),
        _ => {}
    }

    // `layout: ` in a page header, or `page: ` on a route.
    if let (Some(TokenType::Colon), Some(TokenType::Identifier(key))) = (
        last.map(|t| &t.token_type),
        before_last.map(|t| &t.token_type),
    ) && key == "layout"
    {
        return layouts(project);
    }

    let mut items = Vec::new();

    let Some(decl_ix) = analysis::declaration_at(project, file_ix, anchor.saturating_sub(1))
        .or_else(|| analysis::declaration_at(project, file_ix, anchor))
    else {
        // Outside every declaration: the file's top level — or a file that
        // does not parse, where the tree is no help.
        if file.parsed.is_ok()
            || previous.is_empty()
            || !previous
                .iter()
                .any(|t| matches!(t.token_type, TokenType::OpenBrace))
        {
            return top_level();
        }
        return fallback(project, &previous);
    };
    let decl = &project.program.declarations[decl_ix];
    let body = analysis::body_of(decl);
    let path = analysis::statement_path(body, anchor.saturating_sub(1));
    let innermost = path.last().copied();
    let scope = analysis::scope_at(decl, anchor);

    if let Some(open) = open_paren_owner(&previous) {
        // Inside the parentheses of something. Whose?
        match open {
            ParenOwner::Element(name) => {
                items.extend(element_props(project, &name, &previous));
                items.extend(scope_items(&scope));
                return items;
            }
            ParenOwner::PageHeader => {
                items.extend(args(reference::PAGE_ATTRIBUTES, "page attribute"));
                return items;
            }
            ParenOwner::Fetch => {
                items.extend(args(reference::RESOURCE_OPTIONS, "fetch option"));
                items.extend(scope_items(&scope));
                return items;
            }
            ParenOwner::Call | ParenOwner::Unknown => {
                items.extend(scope_items(&scope));
                return items;
            }
        }
    }

    // Directly inside a `match { }`: the arms it still lacks.
    if let Some(Statement {
        kind: StatementKind::Match(m),
        ..
    }) = innermost
        && !inside_child_body(innermost.unwrap(), anchor)
        && statement_start(&previous, 0)
    {
        return match_arms(m, &scope);
    }

    // Directly inside a component's block: the slots it declares.
    if let Some(Statement {
        kind: StatementKind::UIElement(el),
        ..
    }) = innermost
        && let ComponentRef::UserDefined(name) = &el.component
        && statement_start(&previous, 0)
        && let Some(component) = find_component(project, name)
    {
        for slot in &component.slots {
            if let Some(slot_name) = &slot.name
                && !el.slot_fills.iter().any(|f| &f.name == slot_name)
            {
                items.push(snippet(
                    slot_name,
                    &format!("slot of {name}"),
                    &format!("{slot_name} {{\n\t$0\n}}"),
                ));
            }
        }
    }

    // In a body: what a statement can start with, and the names in scope.
    let in_store = matches!(decl, Declaration::Store(_));
    let in_component = matches!(decl, Declaration::Component(_));
    if !in_store {
        items.extend(components(project));
    }
    items.extend(keywords(in_store, in_component));
    items.extend(scope_items(&scope));
    items
}

/// Tokens of a source that fails to lex, up to the failure, line by line
/// so a bad character late in the file does not blind the whole of it.
fn tokens_until_error(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut base = 0;
    for line in source.split_inclusive('\n') {
        if let Ok(mut line_tokens) = webfluent::lexer::LexerV2::new(line, "").tokenize() {
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

/// Whether the token `back` places before the end of `previous` starts a
/// statement: the first token of a block, or the one after another
/// statement's closing brace or parenthesis.
fn statement_start(previous: &[&Token], back: usize) -> bool {
    let Some(ix) = previous.len().checked_sub(back + 1) else {
        return true;
    };
    match previous.get(ix).map(|t| &t.token_type) {
        None => true,
        Some(TokenType::OpenBrace | TokenType::CloseBrace | TokenType::CloseParen) => true,
        // `Text("x")` then a new line: the previous statement ended with a
        // literal or a word, which a flag chain also does.
        Some(TokenType::StringLiteral(_) | TokenType::NumberLiteral(_)) => true,
        Some(TokenType::Identifier(_)) => !matches!(
            previous.get(ix - 1).map(|t| &t.token_type),
            Some(TokenType::Colon | TokenType::Comma | TokenType::OpenParen)
        ),
        _ => false,
    }
}

/// Whether `offset` lies inside one of the statement's own blocks.
fn inside_child_body(stmt: &Statement, offset: usize) -> bool {
    analysis::child_bodies(stmt).into_iter().any(|b| {
        b.iter()
            .any(|s| analysis::contains(s.span, offset.saturating_sub(1)))
    })
}

enum ParenOwner {
    Element(String),
    PageHeader,
    Fetch,
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
                let TokenType::Identifier(name) = &owner.token_type else {
                    return Some(ParenOwner::Unknown);
                };
                let before = ix
                    .checked_sub(2)
                    .and_then(|i| previous.get(i))
                    .map(|t| &t.token_type);
                let before2 = ix
                    .checked_sub(3)
                    .and_then(|i| previous.get(i))
                    .map(|t| &t.token_type);
                return Some(match (name.as_str(), before) {
                    ("fetch", _) => ParenOwner::Fetch,
                    (_, Some(TokenType::Identifier(kw))) if kw == "page" => ParenOwner::PageHeader,
                    (_, Some(TokenType::Identifier(kw))) if kw == "component" => {
                        ParenOwner::Unknown
                    }
                    _ if name.chars().next().is_some_and(char::is_uppercase) => {
                        match (before, before2) {
                            // `Card.Header(`: the part; `Store.method(`: a call.
                            (Some(TokenType::Dot), Some(TokenType::Identifier(owner_name))) => {
                                if owner_name.chars().next().is_some_and(char::is_uppercase) {
                                    ParenOwner::Element(format!("{owner_name}.{name}"))
                                } else {
                                    ParenOwner::Call
                                }
                            }
                            _ => ParenOwner::Element(name.clone()),
                        }
                    }
                    _ => match before {
                        // `Store.method(` — a call.
                        Some(TokenType::Dot) => ParenOwner::Call,
                        _ => ParenOwner::Call,
                    },
                });
            }
            TokenType::OpenBrace | TokenType::CloseBrace if depth == 0 => return None,
            _ => {}
        }
    }
    None
}

enum Block {
    Style,
    Transition,
    Theme,
}

/// The nearest enclosing block the tokens alone identify: a `style { … }`
/// (a nested rule inside one counts), a `transition { }`, or a `theme Name
/// { … }` body.
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
                    (Some(TokenType::Identifier(w)), _) if w == "style" => {
                        return Some(Block::Style);
                    }
                    (Some(TokenType::Identifier(w)), _) if w == "transition" => {
                        return Some(Block::Transition);
                    }
                    (Some(TokenType::Identifier(_)), Some(TokenType::Identifier(kw)))
                        if kw == "theme" =>
                    {
                        return Some(Block::Theme);
                    }
                    // A nested rule: keep looking outward for the `style`
                    // that holds it. Any other opener is a body.
                    (Some(TokenType::RawSelector(_)), _) => {}
                    _ => return None,
                }
            }
            _ => {}
        }
    }
    None
}

/// The CSS property whose value the cursor is in, when the tokens say so.
fn style_property_of(previous: &[&Token]) -> Option<String> {
    for token in previous.iter().rev() {
        match &token.token_type {
            TokenType::StyleProp(name) => return Some(name.clone()),
            TokenType::RawValue(_) => continue,
            _ => return None,
        }
    }
    None
}

fn after_dot(
    project: &Project,
    file_ix: usize,
    previous: &[&Token],
    anchor: usize,
) -> Vec<CompletionItem> {
    let n = previous.len();
    let owner = n.checked_sub(2).map(|i| previous[i]);
    let Some(owner) = owner else {
        return Vec::new();
    };
    match &owner.token_type {
        // `Button("x").` or `Row(gap: .sm).`: the flags of that element.
        TokenType::CloseParen => {
            let name = element_before_paren(previous, n - 2);
            name.map(|n| flags(project, &n)).unwrap_or_default()
        }
        // `tone: .` — the cases of the prop.
        TokenType::Colon => {
            if let Some(TokenType::Identifier(key)) =
                n.checked_sub(3).map(|i| &previous[i].token_type)
                && let Some(ParenOwner::Element(el)) = open_paren_owner(&previous[..n - 1])
            {
                return cases_of(project, &el, key);
            }
            Vec::new()
        }
        TokenType::Identifier(word) => {
            if word.chars().next().is_some_and(char::is_uppercase) {
                // `Card.` — its parts, and its flags; `Store.` — its members.
                if let Some(store) = project.program.declarations.iter().find_map(|d| match d {
                    Declaration::Store(s) if &s.name == word => Some(s),
                    _ => None,
                }) {
                    return analysis::store_members(store)
                        .into_iter()
                        .map(|m| binding_item(&m, Some(&store.name)))
                        .collect();
                }
                let mut items = parts(word);
                items.extend(flags(project, word));
                return items;
            }
            // `.primary.` — another flag of the same element.
            if matches!(
                n.checked_sub(3).map(|i| &previous[i].token_type),
                Some(TokenType::Dot)
            ) {
                let mut ix = n - 2;
                while ix >= 2
                    && matches!(previous[ix].token_type, TokenType::Identifier(_))
                    && matches!(previous[ix - 1].token_type, TokenType::Dot)
                {
                    ix -= 2;
                }
                let name = match &previous[ix].token_type {
                    TokenType::CloseParen => element_before_paren(previous, ix),
                    TokenType::Identifier(w) => Some(w.clone()),
                    _ => None,
                };
                return name.map(|n| flags(project, &n)).unwrap_or_default();
            }
            // `item.` — a value: its fields, or the methods of its kind,
            // when the checker knows what it is.
            value_members(project, file_ix, anchor, word)
        }
        _ => Vec::new(),
    }
}

/// What follows `name.` for a name in scope: a record's fields, an enum's
/// nothing, a list's or a string's methods.
fn value_members(
    project: &Project,
    file_ix: usize,
    anchor: usize,
    name: &str,
) -> Vec<CompletionItem> {
    let Some(decl_ix) = analysis::declaration_at(project, file_ix, anchor.saturating_sub(1)) else {
        return Vec::new();
    };
    let decl = &project.program.declarations[decl_ix];
    let Some(binding) = analysis::scope_at(decl, anchor)
        .into_iter()
        .find(|b| b.name == name)
    else {
        return Vec::new();
    };
    let Some(ty) = crate::hover::type_of_binding(project, decl_ix, &binding) else {
        return Vec::new();
    };
    let method = |m: &str, detail: &str| CompletionItem {
        label: m.to_string(),
        kind: Some(CompletionItemKind::METHOD),
        detail: Some(detail.to_string()),
        insert_text: Some(format!("{m}($0)")),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("1{m}")),
        ..Default::default()
    };
    // A value that may be null offers what it holds; the checker says so.
    let ty = match ty {
        Type::Optional(inner) => *inner,
        other => other,
    };
    match ty {
        Type::Record(record) => project
            .program
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Type(t) if t.name == record => Some(t),
                _ => None,
            })
            .map(|t| {
                t.all_fields(&|name| crate::hover::find_type(project, name))
                    .into_iter()
                    .map(|f| CompletionItem {
                        label: f.name.clone(),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some(format!("{} — field of {}", type_name(&f.ty), record)),
                        documentation: f.doc.clone().map(Documentation::String),
                        sort_text: Some(format!("0{}", f.name)),
                        ..Default::default()
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Type::List(_) => {
            let mut items = vec![CompletionItem {
                label: "length".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Number".to_string()),
                sort_text: Some("0length".to_string()),
                ..Default::default()
            }];
            for (m, d) in [
                ("map", "a new list, one result per item"),
                ("filter", "the items the test holds for"),
                ("find", "the first item the test holds for, or null"),
                ("some", "whether the test holds for any item"),
                ("every", "whether the test holds for every item"),
                ("includes", "whether the list holds the value"),
                ("indexOf", "where the value is, or -1"),
                ("concat", "this list and another"),
                ("slice", "a part of the list"),
                ("join", "the items as one string"),
                ("reduce", "one value folded from the items"),
                ("sum", "the total of the numbers"),
            ] {
                items.push(method(m, d));
            }
            items
        }
        Type::String => {
            let mut items = vec![CompletionItem {
                label: "length".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Number".to_string()),
                sort_text: Some("0length".to_string()),
                ..Default::default()
            }];
            for (m, d) in [
                ("toUpperCase", "in upper case"),
                ("toLowerCase", "in lower case"),
                ("trim", "without surrounding whitespace"),
                ("includes", "whether it holds the text"),
                ("startsWith", "whether it begins with the text"),
                ("endsWith", "whether it ends with the text"),
                ("split", "the parts between a separator"),
                ("replace", "with a part swapped"),
                ("slice", "a part of it"),
            ] {
                items.push(method(m, d));
            }
            items
        }
        Type::Resource(_) => ["state", "data", "error"]
            .iter()
            .map(|f| CompletionItem {
                label: f.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("of the resource".to_string()),
                ..Default::default()
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The name of the element whose `)` sits at `close_ix`: the word before
/// the matching `(`, with its owner when it is a part.
fn element_before_paren(previous: &[&Token], close_ix: usize) -> Option<String> {
    let mut depth = 0i32;
    let mut ix = close_ix;
    loop {
        match previous[ix].token_type {
            TokenType::CloseParen => depth += 1,
            TokenType::OpenParen => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        ix = ix.checked_sub(1)?;
    }
    let TokenType::Identifier(name) = &previous.get(ix.checked_sub(1)?)?.token_type else {
        return None;
    };
    if let (Some(TokenType::Dot), Some(TokenType::Identifier(owner))) = (
        ix.checked_sub(2).map(|i| &previous[i].token_type),
        ix.checked_sub(3).map(|i| &previous[i].token_type),
    ) && owner.chars().next().is_some_and(char::is_uppercase)
    {
        return Some(format!("{owner}.{name}"));
    }
    Some(name.clone())
}

fn find_component<'a>(project: &'a Project, name: &str) -> Option<&'a ComponentDecl> {
    project.program.declarations.iter().find_map(|d| match d {
        Declaration::Component(c) if c.name == name => Some(c),
        _ => None,
    })
}

fn find_enum<'a>(project: &'a Project, name: &str) -> Option<&'a EnumDecl> {
    project.program.declarations.iter().find_map(|d| match d {
        Declaration::Enum(e) if e.name == name => Some(e),
        _ => None,
    })
}

fn signature(name: &str) -> Option<&'static ComponentSig> {
    match name.split_once('.') {
        Some((owner, part)) => registry::part(owner, part),
        None => registry::component(name),
    }
}

/// The flags an element takes: its boolean props and the cases of its enum
/// props, then the universal ones.
fn flags(project: &Project, name: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    if let Some(sig) = signature(name) {
        for prop in sig.all_props() {
            if !prop.shorthand {
                continue;
            }
            match prop.ty {
                PropType::Bool => items.push(CompletionItem {
                    label: prop.name.to_string(),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    detail: Some(format!("{}: true", prop.name)),
                    documentation: Some(Documentation::String(prop.summary.to_string())),
                    sort_text: Some(format!("{}{}", rank(prop), prop.name)),
                    ..Default::default()
                }),
                PropType::Enum(cases) => {
                    for case in cases.iter().filter(|c| !c.name.is_empty()) {
                        items.push(CompletionItem {
                            label: case.name.to_string(),
                            kind: Some(CompletionItemKind::ENUM_MEMBER),
                            detail: Some(format!("{}: .{}", prop.name, case.name)),
                            documentation: Some(Documentation::String(case.summary.to_string())),
                            sort_text: Some(format!("{}{}", rank(prop), case.name)),
                            ..Default::default()
                        });
                    }
                }
                _ => {}
            }
        }
        return items;
    }
    if let Some(component) = find_component(project, name) {
        for prop in &component.props {
            match &prop.prop_type {
                TypeRef::Bool => items.push(CompletionItem {
                    label: prop.name.clone(),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    detail: Some(format!("{}: true — prop of {}", prop.name, component.name)),
                    sort_text: Some(format!("0{}", prop.name)),
                    ..Default::default()
                }),
                TypeRef::Named(enum_name) => {
                    if let Some(e) = find_enum(project, enum_name) {
                        for case in e.case_names() {
                            items.push(CompletionItem {
                                label: case.clone(),
                                kind: Some(CompletionItemKind::ENUM_MEMBER),
                                detail: Some(format!("{}: .{case} — {enum_name}", prop.name)),
                                sort_text: Some(format!("0{case}")),
                                ..Default::default()
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
    items
}

/// Universal props sort after the component's own.
fn rank(prop: &registry::PropSig) -> &'static str {
    if registry::UNIVERSAL_PROPS
        .iter()
        .any(|u| std::ptr::eq(u, prop))
    {
        "2"
    } else {
        "1"
    }
}

/// The parts of a built-in: `Card.` → `Header`, `Body`, `Footer`.
fn parts(owner: &str) -> Vec<CompletionItem> {
    registry::parts_of(owner)
        .map(|part| CompletionItem {
            label: part.name.to_string(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(format!("{}.{} — part of {}", owner, part.name, owner)),
            documentation: Some(Documentation::String(part.summary.to_string())),
            sort_text: Some(format!("0{}", part.name)),
            ..Default::default()
        })
        .collect()
}

/// The cases the prop `key` of `element` takes, offered after `key: .`.
fn cases_of(project: &Project, element: &str, key: &str) -> Vec<CompletionItem> {
    if let Some(sig) = signature(element)
        && let Some(prop) = sig.prop(key)
        && let PropType::Enum(cases) = prop.ty
    {
        return cases
            .iter()
            .filter(|c| !c.name.is_empty())
            .map(|c| CompletionItem {
                label: c.name.to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(format!("{key}: .{} on {element}", c.name)),
                documentation: Some(Documentation::String(c.summary.to_string())),
                ..Default::default()
            })
            .collect();
    }
    if let Some(component) = find_component(project, element)
        && let Some(prop) = component.props.iter().find(|p| p.name == key)
        && let TypeRef::Named(enum_name) = &prop.prop_type
        && let Some(e) = find_enum(project, enum_name)
    {
        return e
            .case_names()
            .into_iter()
            .map(|c| CompletionItem {
                label: c.clone(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(format!("{key}: .{c} — {enum_name}")),
                ..Default::default()
            })
            .collect();
    }
    Vec::new()
}

/// The events an `on` handler can name where the cursor is: the DOM events,
/// and the events the component declares when the cursor is in a
/// component's block.
fn events(project: &Project, file_ix: usize, anchor: usize) -> Vec<CompletionItem> {
    let mut items = declared_events(project, file_ix, anchor);
    items.extend(
        reference::events()
            .iter()
            .map(|(name, doc)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some(doc.to_string()),
                insert_text: Some(format!("{name} {{\n\t$0\n}}")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("1{name}")),
                ..Default::default()
            }),
    );
    items
}

/// The events declared by the component whose block the cursor is in (for
/// `on`), or by the component being written (for `emit`).
fn declared_events(project: &Project, file_ix: usize, anchor: usize) -> Vec<CompletionItem> {
    let Some(decl_ix) = analysis::declaration_at(project, file_ix, anchor.saturating_sub(1)) else {
        return Vec::new();
    };
    let decl = &project.program.declarations[decl_ix];
    let path = analysis::statement_path(analysis::body_of(decl), anchor.saturating_sub(1));
    let component = path
        .iter()
        .rev()
        .find_map(|s| match &s.kind {
            StatementKind::UIElement(el) => match &el.component {
                ComponentRef::UserDefined(name) => find_component(project, name),
                _ => None,
            },
            _ => None,
        })
        .or(match decl {
            Declaration::Component(c) => Some(c),
            _ => None,
        });
    let Some(component) = component else {
        return Vec::new();
    };
    component
        .events
        .iter()
        .map(|e| {
            let params = e
                .params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let head = if params.is_empty() {
                e.name.clone()
            } else {
                format!("{}({params})", e.name)
            };
            CompletionItem {
                label: e.name.clone(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some(format!("event of {}", component.name)),
                insert_text: Some(format!("{head} {{\n\t$0\n}}")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("0{}", e.name)),
                ..Default::default()
            }
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
                detail: Some("store".to_string()),
                ..Default::default()
            }),
            _ => None,
        })
        .collect()
}

/// The components a page can name as its `layout:`: those with a default slot.
fn layouts(project: &Project) -> Vec<CompletionItem> {
    project
        .program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) if c.slots.iter().any(|s| s.name.is_none()) => {
                Some(CompletionItem {
                    label: c.name.clone(),
                    kind: Some(CompletionItemKind::CLASS),
                    detail: Some("layout — a component with a default slot".to_string()),
                    ..Default::default()
                })
            }
            _ => None,
        })
        .collect()
}

fn top_level() -> Vec<CompletionItem> {
    vec![
        snippet(
            "page",
            "A routed page",
            "page ${1:Name}(path: \"${2:/}\", title: \"${3:$1}\") {\n\t$0\n}",
        ),
        snippet(
            "component",
            "A reusable component",
            "component ${1:Name}(${2:label}: ${3:String}) {\n\t$0\n}",
        ),
        snippet(
            "store",
            "Shared state",
            "store ${1:Name}Store {\n\tstate ${2:items} = ${3:[]}\n\t$0\n}",
        ),
        snippet(
            "theme",
            "Design tokens",
            "theme ${1:Brand} {\n\tcolor-primary: ${2:#3B82F6}\n\t$0\n}",
        ),
        snippet("app", "The root of the site", "app {\n\t$0\n\tRouter\n}"),
        snippet(
            "type",
            "A record type",
            "type ${1:Name} {\n\t${2:id}: ${3:String}\n}",
        ),
        snippet(
            "enum",
            "A set of cases",
            "enum ${1:Name} { ${2:a}, ${3:b} }",
        ),
    ]
}

fn theme_body() -> Vec<CompletionItem> {
    reference::TOKENS
        .iter()
        .map(|(name, doc)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: Some(doc.to_string()),
            insert_text: Some(format!("{name}: ${{1:value}}")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

/// The design tokens as `$name`: the project's themes' and the baseline,
/// and — in a style value — the short names the property's group resolves.
fn design_tokens(project: &Project, css_prop: Option<String>) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for decl in &project.program.declarations {
        if let Declaration::Theme(t) = decl {
            for token in &t.tokens {
                if seen.insert(token.name.clone()) {
                    items.push(token_item(
                        &token.name,
                        &format!("token of theme {}", t.name),
                        "0",
                    ));
                }
            }
        }
    }
    for (name, doc) in reference::TOKENS {
        if seen.insert(name.to_string()) {
            items.push(token_item(name, doc, "1"));
        }
    }
    if let Some(prop) = css_prop {
        let css_prop = webfluent::codegen::style_tokens::canonical_style_prop(&prop);
        for (name, _) in reference::TOKENS {
            if let Some(short) = name.rsplit_once('-').map(|(_, s)| s)
                && webfluent::codegen::style_tokens::resolve_short_token(&css_prop, short, &|_| {
                    false
                })
                .as_deref()
                    == Some(name)
                && seen.insert(short.to_string())
            {
                items.push(token_item(
                    short,
                    &format!("${name}, by the property's group"),
                    "0",
                ));
            }
        }
    }
    items
}

fn token_item(name: &str, doc: &str, sort: &str) -> CompletionItem {
    CompletionItem {
        label: format!("${name}"),
        kind: Some(CompletionItemKind::VALUE),
        detail: Some(doc.to_string()),
        insert_text: Some(format!("${name}")),
        filter_text: Some(format!("${name} {name} {}", name.replace('-', " "))),
        sort_text: Some(format!("{sort}{name}")),
        ..Default::default()
    }
}

/// Inside a `match x { }`: the arms not yet written.
fn match_arms(m: &MatchStmt, scope: &[Binding]) -> Vec<CompletionItem> {
    let written: Vec<&ArmPattern> = m.arms.iter().map(|a| &a.pattern).collect();
    let mut items = Vec::new();
    let is_resource = match &m.scrutinee {
        Expr::Identifier(name) => scope
            .iter()
            .any(|b| &b.name == name && b.kind == BindingKind::Resource),
        _ => false,
    };
    let has = |p: fn(&ArmPattern) -> bool| written.iter().any(|w| p(w));
    if is_resource
        || written.is_empty()
        || has(|p| {
            matches!(
                p,
                ArmPattern::Loading | ArmPattern::Error | ArmPattern::Ready
            )
        })
    {
        if !has(|p| matches!(p, ArmPattern::Loading)) {
            items.push(snippet(
                "loading",
                "While the request is in flight",
                "loading {\n\t$0\n}",
            ));
        }
        if !has(|p| matches!(p, ArmPattern::Error)) {
            items.push(snippet(
                "error",
                "When the request fails",
                "error(${1:e}) {\n\t$0\n}",
            ));
        }
        if !has(|p| matches!(p, ArmPattern::Ready)) {
            items.push(snippet(
                "ready",
                "Once the data arrived",
                "ready(${1:value}) {\n\t$0\n}",
            ));
        }
    }
    if !has(|p| matches!(p, ArmPattern::Else)) {
        items.push(snippet("else", "Every other value", "else {\n\t$0\n}"));
    }
    items
}

fn style_block(
    project: &Project,
    previous: &[&Token],
    offset: usize,
    tokens: &[Token],
) -> Vec<CompletionItem> {
    // In a value: the design tokens, as `$name`.
    let in_value = matches!(
        previous.last().map(|t| &t.token_type),
        Some(TokenType::StyleProp(_) | TokenType::RawValue(_))
    ) || tokens.iter().any(|t| {
        matches!(t.token_type, TokenType::RawValue(_)) && t.offset < offset && offset <= t.end
    });
    if in_value {
        return design_tokens(project, style_property_of(previous));
    }
    let mut items: Vec<CompletionItem> = reference::CSS_PROPERTIES
        .iter()
        .map(|prop| CompletionItem {
            label: prop.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some("CSS property".to_string()),
            insert_text: Some(format!("{prop}: $0")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("1{prop}")),
            ..Default::default()
        })
        .collect();
    items.extend(
        reference::SELECTORS
            .iter()
            .map(|(selector, doc)| CompletionItem {
                label: selector.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(doc.to_string()),
                insert_text: Some(format!("{selector} {{\n\t$0\n}}")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                filter_text: Some(selector.trim_start_matches('&').to_string()),
                sort_text: Some(format!("2{selector}")),
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

fn transition_block() -> Vec<CompletionItem> {
    [
        "background",
        "color",
        "transform",
        "opacity",
        "box-shadow",
        "border-color",
        "all",
    ]
    .iter()
    .map(|prop| CompletionItem {
        label: prop.to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some("property to animate".to_string()),
        insert_text: Some(format!("{prop}: ${{1:200ms}} ${{2:ease}}")),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    })
    .collect()
}

/// The props an element takes, by name, the component's own first.
fn element_props(project: &Project, name: &str, previous: &[&Token]) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let written: Vec<String> = written_arg_names(previous);

    if let Some(component) = find_component(project, name) {
        for prop in &component.props {
            if written.contains(&prop.name) {
                continue;
            }
            items.push(CompletionItem {
                label: format!("{}:", prop.name),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some(format!(
                    "{} — prop of {}",
                    type_name(&prop.prop_type),
                    component.name
                )),
                documentation: prop.doc.clone().map(Documentation::String),
                insert_text: Some(format!("{}: ", prop.name)),
                sort_text: Some(format!("0{}", prop.name)),
                ..Default::default()
            });
        }
        return items;
    }

    let Some(sig) = signature(name) else {
        return items;
    };
    for prop in sig.all_props() {
        if written.iter().any(|w| w == prop.name) {
            continue;
        }
        let (insert, detail) = match prop.ty {
            PropType::Enum(_) => (
                format!("{}: .", prop.name),
                format!("{} — one of its cases", prop.name),
            ),
            PropType::Bool => (
                format!("{}: ", prop.name),
                format!("{}: true | false", prop.name),
            ),
            PropType::State => (format!("{}: ", prop.name), "a state variable".to_string()),
            _ => (
                format!("{}: ", prop.name),
                format!("{} prop", sig.qualified()),
            ),
        };
        items.push(CompletionItem {
            label: format!("{}:", prop.name),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(detail),
            documentation: Some(Documentation::String(prop.summary.to_string())),
            insert_text: Some(insert),
            sort_text: Some(format!("{}{}", rank(prop), prop.name)),
            ..Default::default()
        });
    }
    if name == "Icon" || name == "IconButton" {
        items.extend(icons());
    }
    items
}

/// The argument names already written in the open parenthesis group.
fn written_arg_names(previous: &[&Token]) -> Vec<String> {
    let mut names = Vec::new();
    let mut depth = 0i32;
    for (ix, token) in previous.iter().enumerate().rev() {
        match token.token_type {
            TokenType::CloseParen => depth += 1,
            TokenType::OpenParen => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            TokenType::Colon if depth == 0 => {
                if let Some(TokenType::Identifier(name)) =
                    ix.checked_sub(1).map(|i| &previous[i].token_type)
                {
                    names.push(name.clone());
                }
            }
            _ => {}
        }
    }
    names
}

fn type_name(ty: &TypeRef) -> String {
    match ty {
        TypeRef::String => "String".into(),
        TypeRef::Number => "Number".into(),
        TypeRef::Bool => "Bool".into(),
        TypeRef::Map => "Map".into(),
        TypeRef::Any => "Any".into(),
        TypeRef::List(inner) => format!("[{}]", type_name(inner)),
        TypeRef::Optional(inner) => format!("{}?", type_name(inner)),
        TypeRef::Named(name) => name.clone(),
    }
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

fn icons() -> Vec<CompletionItem> {
    reference::ICONS
        .iter()
        .map(|icon| CompletionItem {
            label: format!("\"{icon}\""),
            kind: Some(CompletionItemKind::VALUE),
            detail: Some("Built-in icon".to_string()),
            filter_text: Some(icon.to_string()),
            sort_text: Some(format!("3{icon}")),
            ..Default::default()
        })
        .collect()
}

/// A call snippet for a built-in: its positional prop, if any.
fn builtin_snippet(sig: &ComponentSig) -> String {
    match (&sig.positional, sig.children) {
        (Some(p), Children::Elements) => format!("{}(${{1:{}}}) {{\n\t$0\n}}", sig.name, p.name),
        (Some(p), Children::None) => format!("{}(${{1:{}}})", sig.name, p.name),
        (None, Children::Elements) => format!("{} {{\n\t$0\n}}", sig.name),
        (None, Children::None) => sig.name.to_string(),
    }
}

fn components(project: &Project) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = registry::components()
        .map(|c| CompletionItem {
            label: c.name.to_string(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(format!("{} — {}", c.group, c.summary)),
            insert_text: Some(builtin_snippet(c)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("1{}", c.name)),
            ..Default::default()
        })
        .collect();
    for (ix, decl) in project.program.declarations.iter().enumerate() {
        if let Declaration::Component(c) = decl {
            let call = if c.props.is_empty() {
                c.name.clone()
            } else {
                let mut n = 0;
                let args: Vec<String> = c
                    .props
                    .iter()
                    .map(|p| {
                        n += 1;
                        if p.positional {
                            format!("${{{n}:{}}}", p.name)
                        } else {
                            format!("{}: ${{{n}:{}}}", p.name, p.name)
                        }
                    })
                    .collect();
                format!("{}({})", c.name, args.join(", "))
            };
            items.push(CompletionItem {
                label: c.name.clone(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(format!(
                    "component — {}",
                    project.label_of(project.decl_file[ix])
                )),
                documentation: c.doc.clone().map(Documentation::String),
                insert_text: Some(call),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("0{}", c.name)),
                ..Default::default()
            });
        }
    }
    items
}

fn keywords(in_store: bool, in_component: bool) -> Vec<CompletionItem> {
    reference::KEYWORDS
        .iter()
        .filter(|k| k.place == Place::Body)
        .filter(|k| {
            if in_store {
                matches!(
                    k.name,
                    "state"
                        | "derived"
                        | "action"
                        | "effect"
                        | "return"
                        | "if"
                        | "else"
                        | "let"
                        | "log"
                        | "navigate"
                        | "await"
                )
            } else if in_component {
                !matches!(k.name, "let" | "return" | "await" | "null" | "by")
            } else {
                !matches!(
                    k.name,
                    "let"
                        | "return"
                        | "await"
                        | "null"
                        | "by"
                        | "event"
                        | "slot"
                        | "children"
                        | "emit"
                )
            }
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
        "resource" => "resource ${1:data} = fetch(\"${2:/api/}\")\nmatch $1 {\n\tloading { Spinner }\n\terror(e) { Alert(e.message).danger }\n\tready(${3:value}) {\n\t\t$0\n\t}\n}".into(),
        "match" => "match ${1:value} {\n\t$0\n}".into(),
        "if" => "if ${1:condition} {\n\t$0\n}".into(),
        "else" => "else {\n\t$0\n}".into(),
        "for" => "for ${1:item} in ${2:items} by ${1}.${3:id} {\n\t$0\n}".into(),
        "show" => "show ${1:visible} {\n\t$0\n}".into(),
        "navigate" => "navigate(\"${1:/}\")".into(),
        "log" => "log(${1:value})".into(),
        "emit" => "emit ${1:event}($0)".into(),
        "event" => "event ${1:name}(${2})".into(),
        "slot" => "slot ${1:name}".into(),
        "children" => "children".into(),
        "on" => "on ${1:click} {\n\t$0\n}".into(),
        "style" => "style {\n\t${1:padding}: ${2:1rem}\n\t$0\n}".into(),
        "transition" => "transition {\n\t${1:background}: ${2:200ms} ${3:ease}\n}".into(),
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
/// keywords in a body, props of the element whose parenthesis is open.
fn fallback(project: &Project, previous: &[&Token]) -> Vec<CompletionItem> {
    if let Some(ParenOwner::Element(name)) = open_paren_owner(previous) {
        return element_props(project, &name, previous);
    }
    let mut items = components(project);
    items.extend(keywords(false, false));
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
