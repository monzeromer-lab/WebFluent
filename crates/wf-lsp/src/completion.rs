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

use lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent, MarkupKind,
    Position,
};
use webfluent::config::OutputType;
use webfluent::linter;
use webfluent::registry::{self, Children, ComponentSig, PropType};
use webfluent::syntax::lexer::{Token, TokenType};
use webfluent::syntax::parser::ast::{
    ApiDecl, ArmPattern, ComponentDecl, ComponentRef, Declaration, EnumCase, EnumDecl,
    EventHandler, Expr, MatchStmt, Statement, StatementKind, TypeRef,
};

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
    let tokens = analysis::tokens_of(file)
        .unwrap_or_else(|| std::sync::Arc::new(tokens_until_error(source, &file.name())));
    // Inside `class: "…"`: the classes the project's stylesheets define.
    if crate::classes::in_class_value(&tokens, offset) {
        return crate::classes::completions(project);
    }
    // Inside a string's `{…}` splice: code — names in scope, and what a
    // value has after a dot, read from the text since the lexer has no
    // tokens there.
    if analysis::in_splice(source, &tokens, offset) {
        return splice_items(project, file_ix, source, offset);
    }
    // Inside a string, some strings name things: a key, a tag, a message.
    if analysis::in_string(&tokens, offset) {
        return string_value(project, &tokens, offset);
    }
    // In a comment, only an allow names anything: the codes it may name.
    if analysis::in_comment(source, &tokens, offset) {
        let line = &source[..offset];
        let line = &line[line.rfind('\n').map_or(0, |i| i + 1)..];
        if let Some(open) = line.rfind("wf-allow(")
            && !line[open..].contains(')')
        {
            return allow_codes();
        }
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
    let anchor = typing.map_or(offset, |t| t.offset);
    let previous: Vec<&Token> = tokens.iter().filter(|t| t.end <= anchor).collect();
    let last = previous.last().copied();
    let before_last = previous.len().checked_sub(2).map(|i| previous[i]);

    let line_prefix = &source[..offset];
    let line_prefix = &line_prefix[line_prefix.rfind('\n').map_or(0, |i| i + 1)..];
    let typed = line_prefix.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_' || c == '-');
    let in_style = matches!(
        enclosing_block(&previous),
        Some(Block::Style | Block::Keyframe)
    );

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
        Some(Block::Style) => return style_block(project, source, &previous, offset, &tokens),
        Some(Block::Transition) => return transition_block(),
        Some(Block::Animation) => return keyframes(),
        // A keyframe's declarations: what a style block's are, without the
        // nested rules and queries a keyframe cannot hold.
        Some(Block::Keyframe) => {
            return style_block(project, source, &previous, offset, &tokens)
                .into_iter()
                .filter(|i| i.kind != Some(CompletionItemKind::KEYWORD))
                .collect();
        }
        None => {}
    }

    match last.map(|t| &t.token_type) {
        Some(TokenType::Identifier(w)) if w == "use" && statement_start(&previous, 1) => {
            return stores(project);
        }
        Some(TokenType::Identifier(w)) if w == "on" && statement_start(&previous, 1) => {
            return events(project, file_ix, anchor, &previous);
        }
        Some(TokenType::Identifier(w)) if w == "emit" => {
            return declared_events(project, file_ix, anchor);
        }
        Some(TokenType::Dot | TokenType::OptionalChain) => {
            return after_dot(project, file_ix, &previous, anchor);
        }
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
    // `worker css = ‸`: the function that opens that kind of connection.
    if let Some(items) = connection_opener(&previous) {
        return items;
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

    // A type is being written: after `name:` in a declaration, in a
    // parameter list, in a `type`'s body, or after `->`.
    if type_position(&previous, decl) {
        return type_items(project);
    }
    // What a block holds, by the word that opened it.
    if open_paren_owner(&previous).is_none() {
        match block_word(&previous).as_deref() {
            Some("persist") => return persist_keys(),
            Some("validate") => return validate_rules(),
            Some("sequence") => return sequence_steps(),
            Some(word) if connection_events_of(word).is_some() => {
                return connection_block(word);
            }
            _ => {}
        }
        if matches!(decl, Declaration::Api(_)) {
            return api_body();
        }
    }
    match decl {
        // A value written at the top level: what it may read.
        Declaration::Const(_) | Declaration::Data(_) => {
            let mut items = top_level_names(project);
            items.extend(builtin_items());
            return items;
        }
        Declaration::Type(_) | Declaration::Enum(_) | Declaration::Animation(_) => {
            return Vec::new();
        }
        _ => {}
    }

    let body = analysis::body_of(decl);
    let path = analysis::statement_path(body, anchor.saturating_sub(1));
    let innermost = path.last().copied();
    let scope = analysis::scope_at(decl, anchor);

    // `Grid(columns: { ‸ }`: one value per breakpoint.
    if responsive_key_position(&previous) {
        return webfluent::codegen::scoped_css::RESPONSIVE_STEPS
            .iter()
            .map(|step| CompletionItem {
                label: step.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some(if *step == "base" {
                    "the value at every width, unless a wider step says otherwise".to_string()
                } else {
                    format!("the value from the `screen-{step}` breakpoint up")
                }),
                insert_text: Some(format!("{step}: ")),
                ..Default::default()
            })
            .collect();
    }

    if let Some(open) = open_paren_owner(&previous) {
        // Inside the parentheses of something. Whose?
        match open {
            ParenOwner::Element(name) => {
                items.extend(element_props(project, &name, &previous));
                items.extend(scope_items(&scope));
                // Where a value goes — after `prop:`, or first in the
                // parentheses of an element that takes a positional value —
                // the program's constants and the language's functions too.
                let value_position = match last.map(|t| &t.token_type) {
                    Some(TokenType::Colon) => true,
                    Some(TokenType::OpenParen) => takes_positional(project, &name),
                    _ => false,
                };
                if value_position {
                    items.extend(top_level_names(project));
                    items.extend(builtin_items());
                }
                items.extend(script_items(project));
                return items;
            }
            ParenOwner::PageHeader => {
                items.extend(args(reference::PAGE_ATTRIBUTES, "page attribute"));
                return items;
            }
            ParenOwner::Fetch | ParenOwner::Opener(_) | ParenOwner::Member(..) => {
                items.extend(call_options(project, &open, &previous));
                items.extend(scope_items(&scope));
                items.extend(top_level_names(project));
                items.extend(builtin_items());
                items.extend(script_items(project));
                return items;
            }
            ParenOwner::Call | ParenOwner::Unknown => {
                items.extend(scope_items(&scope));
                items.extend(top_level_names(project));
                items.extend(builtin_items());
                items.extend(script_items(project));
                return items;
            }
        }
    }

    // Directly inside a `match { }`: the arms it still lacks.
    if let Some(
        statement @ Statement {
            kind: StatementKind::Match(m),
            ..
        },
    ) = innermost
        && !inside_child_body(statement, anchor)
        && statement_start(&previous, 0)
    {
        return match_arms(project, decl_ix, m, &scope);
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
    let imperative = path.iter().any(|s| imperative_body(s, anchor));
    let context = if imperative {
        Context::Imperative
    } else {
        match decl {
            Declaration::Store(_) => Context::Store,
            Declaration::Component(_) => Context::Component,
            Declaration::Test(_) => Context::Test,
            _ => Context::Page,
        }
    };
    let spot = Spot {
        context,
        in_element: in_element_block(&previous),
        // `cleanup` closes an effect's own body, not a block inside it.
        in_effect: imperative && block_word(&previous).as_deref() == Some("effect"),
        after_try: after_try(&previous),
    };
    if !in_store && !imperative {
        // The element whose block the cursor is in decides what may go
        // there in a deck: slides in a `Presentation`, no slide in a slide.
        let enclosing = path.iter().rev().find_map(|s| match &s.kind {
            StatementKind::UIElement(el) => Some(&el.component),
            _ => None,
        });
        items.extend(components_in(project, enclosing));
    }
    items.extend(
        keywords(spot)
            .into_iter()
            .filter(|k| !paged(project) || !web_only_keyword(&k.label)),
    );
    items.extend(scope_items(&scope));
    if imperative {
        items.extend(top_level_names(project));
        items.extend(builtin_items());
    }
    items.extend(script_items(project));
    items
}

/// The functions the language gives a program and the values it reads from
/// the browser, with what each does: every name the compiler's tables hold
/// (`built_in_functions`, the browser's values, the language's own), with
/// this server's words for each where it has them.
fn builtin_items() -> Vec<CompletionItem> {
    let functions = webfluent::sema::types::built_in_functions();
    let values = webfluent::codegen::js::BROWSER_VALUES;
    let mut names: Vec<&str> = functions.to_vec();
    for name in values.iter().chain(reference::LANGUAGE_VALUES) {
        if !names.contains(name) {
            names.push(name);
        }
    }
    names
        .into_iter()
        .map(|name| {
            let value = values.contains(&name) || reference::LANGUAGE_VALUES.contains(&name);
            let callable = functions.contains(&name) && !value;
            let (usage, doc) = match reference::builtin(name) {
                Some((_, usage, doc)) => (usage.to_string(), doc.to_string()),
                None => (
                    if callable {
                        format!("{name}(…)")
                    } else {
                        name.to_string()
                    },
                    webfluent::sema::types::language_name(name).unwrap_or_default(),
                ),
            };
            builtin_item(name, callable, usage, doc)
        })
        .collect()
}

fn builtin_item(name: &str, callable: bool, usage: String, doc: String) -> CompletionItem {
    let insert = match name {
        "every" | "after" => format!("{name}(${{1:1000}}) {{\n\t$0\n}}"),
        _ if callable => format!("{name}($0)"),
        _ => name.to_string(),
    };
    CompletionItem {
        label: name.to_string(),
        kind: Some(if callable {
            CompletionItemKind::FUNCTION
        } else {
            CompletionItemKind::CONSTANT
        }),
        detail: Some(usage),
        documentation: (!doc.is_empty()).then_some(Documentation::String(doc)),
        insert_text: Some(insert),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("4{name}")),
        ..Default::default()
    }
}

/// After `worker css = `, `socket chat = `: the one function that opens a
/// connection of that kind.
fn connection_opener(previous: &[&Token]) -> Option<Vec<CompletionItem>> {
    let n = previous.len();
    let at = |back: usize| n.checked_sub(back).map(|i| &previous[i].token_type);
    let (Some(TokenType::Equals), Some(TokenType::Identifier(_)), Some(TokenType::Identifier(kw))) =
        (at(1), at(2), at(3))
    else {
        return None;
    };
    let (_, opener) = reference::CONNECTION_OPENERS
        .iter()
        .find(|(word, _)| word == kw)?;
    let (_, usage, doc) = reference::builtin(opener)?;
    Some(vec![builtin_item(
        opener,
        true,
        usage.to_string(),
        doc.to_string(),
    )])
}

/// The names the program declares at its top level that a value may read:
/// its constants, `data` and images.
fn top_level_names(project: &Project) -> Vec<CompletionItem> {
    project
        .program
        .declarations
        .iter()
        .filter_map(|d| {
            let (name, what) = match d {
                Declaration::Const(c) => (c.name.clone(), "const".to_string()),
                Declaration::Data(d) if d.is_image => {
                    (d.name.clone(), format!("image from `{}`", d.file))
                }
                Declaration::Data(d) => (d.name.clone(), format!("data from `{}`", d.file)),
                _ => return None,
            };
            Some(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::CONSTANT),
                detail: Some(what),
                sort_text: Some(format!("1{name}")),
                ..Default::default()
            })
        })
        .collect()
}

/// What the project's scripts make global, with how a call reads.
fn script_items(project: &Project) -> Vec<CompletionItem> {
    let mut out = Vec::new();
    for decl in &project.program.declarations {
        let Declaration::Script(script) = decl else {
            continue;
        };
        for n in &script.names {
            let callable = !matches!(n.kind, webfluent::project_js::scan::NameKind::Value);
            let doc = n
                .doc
                .as_deref()
                .map(webfluent::project_js::jsdoc::parse)
                .map(|d| d.summary)
                .unwrap_or_default();
            out.push(CompletionItem {
                label: n.name.clone(),
                kind: Some(if callable {
                    CompletionItemKind::FUNCTION
                } else {
                    CompletionItemKind::VARIABLE
                }),
                detail: Some(format!(
                    "{} — {}",
                    webfluent::project_js::signature(n),
                    script.path
                )),
                documentation: (!doc.is_empty()).then_some(Documentation::String(doc)),
                insert_text: callable.then(|| format!("{}($0)", n.name)),
                insert_text_format: callable.then_some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("3{}", n.name)),
                ..Default::default()
            });
        }
    }
    out
}

/// Tokens of a source that fails to lex, up to the failure, line by line
/// so a bad character late in the file does not blind the whole of it.
///
/// The file's name picks its layout, as the build's does. An indented
/// (`.wfx`) file's blocks come from its indentation, which one line alone
/// does not have: the longest run of whole lines from the top that lexes is
/// read in its layout, the braces its end implied dropped (the blocks are
/// still open where the cursor is), and only the lines after it one by one.
fn tokens_until_error(source: &str, file: &str) -> Vec<Token> {
    use webfluent::syntax::lexer::LexerV2;
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut tokens = Vec::new();
    let mut base = 0;
    let mut first = 0;
    if file.ends_with(".wfx") {
        let lexes = |k: usize| {
            let end: usize = lines[..k].iter().map(|l| l.len()).sum();
            LexerV2::for_file(&source[..end], file).tokenize().ok()
        };
        // The longest prefix of whole lines that lexes.
        let (mut lo, mut hi) = (0, lines.len());
        let mut best = Vec::new();
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            match lexes(mid) {
                Some(found) => {
                    best = found;
                    lo = mid;
                }
                None => hi = mid - 1,
            }
        }
        best.retain(|t| !matches!(t.token_type, TokenType::Eof));
        // The blocks the end of the prefix closed are open at the cursor.
        while best
            .last()
            .is_some_and(|t| matches!(t.token_type, TokenType::CloseBrace) && t.end == t.offset)
        {
            best.pop();
        }
        tokens = best;
        first = lo;
        base = lines[..lo].iter().map(|l| l.len()).sum();
    }
    for line in &lines[first..] {
        if let Ok(mut line_tokens) = LexerV2::new(line, file).tokenize() {
            line_tokens.retain(|t| !matches!(t.token_type, TokenType::Eof));
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

/// Whether `offset` is in the body of `stmt` that does something rather
/// than draws something: an action's, a handler's, an effect's or a
/// timer's, and an `on …` handler of an element or of a connection
/// (`socket … { on message { ‸ } }`). (Inside an action's parameter list,
/// the parenthesis decides before this is asked.)
fn imperative_body(stmt: &Statement, offset: usize) -> bool {
    let in_handler = |h: &EventHandler| {
        analysis::contains(h.span, offset.saturating_sub(1)) && h.span.end as usize > offset
    };
    match &stmt.kind {
        StatementKind::Action(_)
        | StatementKind::EventHandler(_)
        | StatementKind::Effect(_)
        | StatementKind::Timer(_) => true,
        StatementKind::UIElement(el) => el.events.iter().any(in_handler),
        StatementKind::Connection(c) => c.handlers.iter().any(in_handler),
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

/// What a request or a connection takes in its parentheses: `fetch(…)`'s
/// options; `ws(…)`, `sse(…)`, `rtc(…)`, `spawn(…)`'s; and for
/// `Backend.users(…)`, the endpoint's parameters and what a call of any
/// endpoint takes. Those already written are left out.
fn call_options(project: &Project, owner: &ParenOwner, previous: &[&Token]) -> Vec<CompletionItem> {
    let items = match owner {
        ParenOwner::Fetch => args(reference::RESOURCE_OPTIONS, "fetch option"),
        ParenOwner::Opener(name) => reference::CONNECTION_OPTIONS
            .iter()
            .find(|(opener, _)| opener == name)
            .map(|(_, options)| args(options, &format!("{name} option")))
            .unwrap_or_default(),
        ParenOwner::Member(owner, name) => {
            let Some(endpoint) = api_named(project, owner)
                .and_then(|api| api.endpoints.iter().find(|e| &e.name == name))
            else {
                return Vec::new();
            };
            let mut items: Vec<CompletionItem> = endpoint
                .params
                .iter()
                .map(|p| CompletionItem {
                    label: format!("{}:", p.name),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(format!(
                        "{} — parameter of {owner}.{name}",
                        type_name(&p.prop_type)
                    )),
                    insert_text: Some(format!("{}: ", p.name)),
                    sort_text: Some(format!("0{}", p.name)),
                    ..Default::default()
                })
                .collect();
            items.extend(
                args(reference::ENDPOINT_CALL_OPTIONS, "call option")
                    .into_iter()
                    .map(|mut item| {
                        item.sort_text = Some(format!("1{}", item.label));
                        item
                    }),
            );
            items
        }
        _ => Vec::new(),
    };
    unwritten(items, previous)
}

enum ParenOwner {
    Element(String),
    PageHeader,
    Fetch,
    /// `ws(`, `sse(`, `broadcast(`, `rtc(`, `spawn(`: what opens a connection.
    Opener(String),
    /// `owner.name(`: an endpoint of a service, or a method of a value.
    Member(String, String),
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
                let after_dot = matches!(before, Some(TokenType::Dot));
                return Some(match (name.as_str(), before) {
                    ("fetch", _) if !after_dot => ParenOwner::Fetch,
                    (opener, _)
                        if !after_dot
                            && reference::CONNECTION_OPTIONS
                                .iter()
                                .any(|(o, _)| *o == opener) =>
                    {
                        ParenOwner::Opener(opener.to_string())
                    }
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
                    // `Backend.users(` — an endpoint, or `Store.method(` — a call.
                    _ => match (before, before2) {
                        (Some(TokenType::Dot), Some(TokenType::Identifier(owner))) => {
                            ParenOwner::Member(owner.clone(), name.clone())
                        }
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
    /// An `animation Name { }` body: its keyframes.
    Animation,
    /// A keyframe of an animation, `50% { ‸ }`: CSS declarations.
    Keyframe,
}

/// The nearest enclosing block the tokens alone identify: a `style { … }`
/// (a nested rule inside one counts), a `transition { }`, or a `theme Name
/// { … }` body.
fn enclosing_block(previous: &[&Token]) -> Option<Block> {
    let mut depth = 0i32;
    // Whether a keyframe's or a nested rule's block was passed on the way out.
    let mut nested = false;
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
                    (Some(TokenType::Identifier(_)), Some(TokenType::Identifier(kw)))
                        if kw == "animation" =>
                    {
                        return Some(if nested {
                            Block::Keyframe
                        } else {
                            Block::Animation
                        });
                    }
                    // A nested rule or a keyframe: keep looking outward for
                    // the `style` or the `animation` that holds it. Any
                    // other opener is a body.
                    (Some(TokenType::RawSelector(_)), _) => nested = true,
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
            TokenType::RawValue(_) => {}
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
            {
                match open_paren_owner(&previous[..n - 1]) {
                    Some(ParenOwner::Element(el)) => return cases_of(project, &el, key),
                    // `fetch(…, on: .`, `Backend.users(cache: .`, `spawn(…, type: .`.
                    Some(ParenOwner::Fetch | ParenOwner::Member(..) | ParenOwner::Opener(_)) => {
                        if let Some(items) = option_cases(key) {
                            return items;
                        }
                    }
                    _ => {}
                }
            }
            opening_dot(project, file_ix, previous, anchor)
        }
        // `3.` — a number given a unit: a `Duration`.
        TokenType::NumberLiteral(_) => webfluent::syntax::parser::v2::DURATION_UNITS
            .iter()
            .map(|u| CompletionItem {
                label: u.to_string(),
                kind: Some(CompletionItemKind::UNIT),
                detail: Some(format!("a Duration: 3.{u}")),
                ..Default::default()
            })
            .collect(),
        TokenType::Identifier(word) => {
            // `Backend.users.` — what a service's endpoint offers besides
            // being called.
            if let (Some(TokenType::Dot), Some(TokenType::Identifier(api))) = (
                n.checked_sub(3).map(|i| &previous[i].token_type),
                n.checked_sub(4).map(|i| &previous[i].token_type),
            ) && api_named(project, api)
                .is_some_and(|a| a.endpoints.iter().any(|e| &e.name == word))
            {
                return endpoint_members();
            }
            if word.chars().next().is_some_and(char::is_uppercase) {
                // `Backend.` — its endpoints.
                if let Some(api) = api_named(project, word) {
                    return api_members(api);
                }
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
            // `env.` — the public names the config supplies.
            if word == "env" {
                return project
                    .env_names
                    .iter()
                    .map(|n| CompletionItem {
                        label: n.clone(),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some("a public env name, fixed at build time".to_string()),
                        ..Default::default()
                    })
                    .collect();
            }
            // `item.` — a value: its fields, or the methods of its kind,
            // when the checker knows what it is.
            value_members(project, file_ix, anchor, word)
        }
        // `= .`, `(.`, `== .`: a case of the enum the value is, or — not
        // knowing which — of any, and the animations an `animate:` plays.
        _ => opening_dot(project, file_ix, previous, anchor),
    }
}

fn api_named<'a>(project: &'a Project, name: &str) -> Option<&'a ApiDecl> {
    project.program.declarations.iter().find_map(|d| match d {
        Declaration::Api(a) if a.name == name => Some(a),
        _ => None,
    })
}

/// `Backend.`: the service's endpoints, and what the whole service offers.
fn api_members(api: &ApiDecl) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = api
        .endpoints
        .iter()
        .map(|e| {
            let params: Vec<String> = e
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, type_name(&p.prop_type)))
                .collect();
            let returns = e
                .returns
                .as_ref()
                .map(|t| format!(" -> {}", type_name(t)))
                .unwrap_or_default();
            CompletionItem {
                label: e.name.clone(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some(format!(
                    "{} {}({}){returns}",
                    e.method,
                    e.name,
                    params.join(", ")
                )),
                documentation: e.doc.clone().map(Documentation::String),
                insert_text: Some(format!("{}($0)", e.name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("0{}", e.name)),
                ..Default::default()
            }
        })
        .collect();
    items.push(CompletionItem {
        label: "invalidate".to_string(),
        kind: Some(CompletionItemKind::METHOD),
        detail: Some(format!("forget what every endpoint of {} cached", api.name)),
        insert_text: Some("invalidate()".to_string()),
        sort_text: Some("1invalidate".to_string()),
        ..Default::default()
    });
    items
}

/// `Backend.users.`: what an endpoint offers besides being called.
fn endpoint_members() -> Vec<CompletionItem> {
    [
        (
            "invalidate",
            true,
            "forget what it cached, for every argument or one",
        ),
        ("prefetch", true, "ask now for what will be wanted soon"),
        ("key", true, "the cache key of a call"),
        ("url", true, "the address a call goes to"),
        (
            "lines",
            true,
            "every line of a streamed response, as it arrives",
        ),
        ("pending", false, "whether a call of it is under way"),
        ("progress", false, "how far an upload has got, 0 to 1"),
    ]
    .iter()
    .map(|(name, call, doc)| CompletionItem {
        label: name.to_string(),
        kind: Some(if *call {
            CompletionItemKind::METHOD
        } else {
            CompletionItemKind::PROPERTY
        }),
        detail: Some(doc.to_string()),
        insert_text: Some(if *call {
            format!("{name}($0)")
        } else {
            name.to_string()
        }),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    })
    .collect()
}

/// A dot that opens a value: `state t: Tone = .`, `if t == .`, `animate: .`,
/// `format(n, .`. The cases of the enum the value is when that is known; a
/// `format` call's styles; otherwise every enum's cases and the program's
/// animations.
fn opening_dot(
    project: &Project,
    file_ix: usize,
    previous: &[&Token],
    anchor: usize,
) -> Vec<CompletionItem> {
    let n = previous.len();
    let at = |back: usize| n.checked_sub(back).map(|i| &previous[i].token_type);
    // `format(value, .`: the styles.
    if matches!(at(2), Some(TokenType::Comma))
        && let Some(ParenOwner::Call) = open_paren_owner(&previous[..n - 1])
        && call_name(&previous[..n - 1]).as_deref() == Some("format")
    {
        return webfluent::codegen::format::STYLES
            .iter()
            .map(|(style, doc)| CompletionItem {
                label: style.to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(doc.to_string()),
                ..Default::default()
            })
            .collect();
    }
    // The enum the value must be: `name: Enum = .`, or the type of the name
    // compared with — `t == .`, `t != .`.
    let expected: Option<String> = match (at(2), at(3), at(4)) {
        (Some(TokenType::Equals), Some(TokenType::Identifier(ty)), Some(TokenType::Colon)) => {
            Some(ty.clone())
        }
        (
            Some(TokenType::DoubleEquals | TokenType::NotEquals | TokenType::StrictNotEqual),
            Some(TokenType::Identifier(name)),
            _,
        ) => analysis::declaration_at(project, file_ix, anchor.saturating_sub(1)).and_then(
            |decl_ix| {
                let decl = &project.program.declarations[decl_ix];
                let binding = analysis::scope_at(decl, anchor)
                    .into_iter()
                    .find(|b| &b.name == name)?;
                match crate::hover::type_of_binding(project, decl_ix, &binding)? {
                    webfluent::sema::types::Type::Enum(e) => Some(e),
                    webfluent::sema::types::Type::Optional(inner) => match *inner {
                        webfluent::sema::types::Type::Enum(e) => Some(e),
                        _ => None,
                    },
                    _ => None,
                }
            },
        ),
        _ => None,
    };
    let case_item = |e: &EnumDecl, case: &EnumCase| CompletionItem {
        label: case.name.clone(),
        kind: Some(CompletionItemKind::ENUM_MEMBER),
        detail: Some(format!(".{} — {}", case.name, e.name)),
        sort_text: Some(format!("0{}", case.name)),
        ..Default::default()
    };
    if let Some(e) = expected
        .as_deref()
        .and_then(|name| find_enum(project, name))
    {
        return e.cases.iter().map(|c| case_item(e, c)).collect();
    }
    let mut items = Vec::new();
    for decl in &project.program.declarations {
        match decl {
            Declaration::Enum(e) => items.extend(e.cases.iter().map(|c| case_item(e, c))),
            Declaration::Animation(a) => items.push(CompletionItem {
                label: a.name.clone(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some("animation — `animate: .Name`".to_string()),
                sort_text: Some(format!("1{}", a.name)),
                ..Default::default()
            }),
            _ => {}
        }
    }
    items
}

/// Whether the cursor is where a type is written: `state x: ‸`,
/// `component C(name: ‸`, `action a(n: ‸`, `type T { field: ‸`,
/// `get users() -> ‸`, `items: [‸`.
fn type_position(previous: &[&Token], decl: &Declaration) -> bool {
    let n = previous.len();
    let at = |back: usize| n.checked_sub(back).map(|i| &previous[i].token_type);
    match at(1) {
        // `[‸` after a colon: a list of some type.
        Some(TokenType::OpenBracket) => return type_position(&previous[..n - 1], decl),
        // `-> ‸`
        Some(TokenType::GreaterThan) if matches!(at(2), Some(TokenType::Minus)) => return true,
        Some(TokenType::Colon) => {}
        _ => return false,
    }
    let Some(TokenType::Identifier(_)) = at(2) else {
        return false;
    };
    // `state x:`, `const N:`, `data d:`, `resource r:` — a declaration's type.
    if let Some(TokenType::Identifier(kw)) = at(3)
        && matches!(
            kw.as_str(),
            "state" | "derived" | "persist" | "const" | "data" | "resource"
        )
    {
        return true;
    }
    // A field of a `type`, outside any parenthesis.
    if matches!(decl, Declaration::Type(_)) && open_paren_owner(&previous[..n - 1]).is_none() {
        return true;
    }
    // A parameter of a declaration: the word before the `(`'s name says so.
    let mut depth = 0i32;
    for (ix, token) in previous[..n - 1].iter().enumerate().rev() {
        match token.token_type {
            TokenType::CloseParen => depth += 1,
            TokenType::OpenParen => {
                if depth > 0 {
                    depth -= 1;
                    continue;
                }
                let keyword = ix
                    .checked_sub(2)
                    .and_then(|i| previous.get(i))
                    .map(|t| &t.token_type);
                return matches!(keyword, Some(TokenType::Identifier(kw))
                    if matches!(kw.as_str(), "component" | "action" | "event" | "slot" | "part")
                        || webfluent::syntax::parser::v2::HTTP_VERBS.contains(&kw.as_str()));
            }
            TokenType::OpenBrace | TokenType::CloseBrace if depth == 0 => return false,
            _ => {}
        }
    }
    false
}

/// The types a program can write: its own, the language's, the plain ones.
fn type_items(project: &Project) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = project
        .program
        .declarations
        .iter()
        .filter_map(|d| {
            let (name, what) = match d {
                Declaration::Type(t) => (t.name.clone(), "type"),
                Declaration::Enum(e) => (e.name.clone(), "enum"),
                _ => return None,
            };
            Some(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some(what.to_string()),
                sort_text: Some(format!("0{name}")),
                ..Default::default()
            })
        })
        .collect();
    for name in webfluent::sema::types::PRIMITIVE_TYPES {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("type".to_string()),
            sort_text: Some(format!("1{name}")),
            ..Default::default()
        });
    }
    for scalar in webfluent::sema::types::Scalar::ALL {
        items.push(CompletionItem {
            label: scalar.name().to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("a type the language brings".to_string()),
            sort_text: Some(format!("2{}", scalar.name())),
            ..Default::default()
        });
    }
    items
}

/// The word the innermost open block's statement begins with: `persist`
/// for `persist n = 0 { ‸`, `validate` for `validate email { ‸`.
fn block_word(previous: &[&Token]) -> Option<String> {
    let mut depth = 0i32;
    let mut brace = None;
    for (ix, token) in previous.iter().enumerate().rev() {
        match token.token_type {
            TokenType::CloseBrace => depth += 1,
            TokenType::OpenBrace => {
                if depth == 0 {
                    brace = Some(ix);
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    let brace = brace?;
    // A statement ends at its line's end, so the one the brace opens begins
    // with the first token on the brace's line.
    let line = previous[brace].line;
    let start = previous[..brace]
        .iter()
        .rposition(|t| t.line != line)
        .map_or(0, |i| i + 1);
    match &previous.get(start)?.token_type {
        TokenType::Identifier(word) => Some(word.clone()),
        _ => None,
    }
}

fn persist_keys() -> Vec<CompletionItem> {
    reference::PERSIST_KEYS
        .iter()
        .map(|(name, body, doc)| CompletionItem {
            documentation: Some(Documentation::String(doc.to_string())),
            kind: Some(CompletionItemKind::PROPERTY),
            ..snippet(name, first_sentence(doc), body)
        })
        .collect()
}

fn validate_rules() -> Vec<CompletionItem> {
    webfluent::sema::types::VALIDATE_RULES
        .iter()
        .map(|rule| {
            let body = match *rule {
                "minLength" | "maxLength" => format!("{rule}(${{1:8}}) \"${{2:message}}\""),
                "min" | "max" => format!("{rule}(${{1:0}})"),
                "pattern" => "pattern(/${1:[0-9]}/) \"${2:message}\"".to_string(),
                "matches" => "matches(${1:other}) \"${2:message}\"".to_string(),
                "oneOf" => "oneOf([${1}])".to_string(),
                "custom" => "custom \"${1:message}\" { ${0:condition} }".to_string(),
                "async" => "async \"${1:message}\" { await ${0:call} }".to_string(),
                other => other.to_string(),
            };
            CompletionItem {
                kind: Some(CompletionItemKind::KEYWORD),
                ..snippet(rule, "validation rule", &body)
            }
        })
        .collect()
}

/// An `api` block: its settings, an endpoint by its verb, a hook.
fn api_body() -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = reference::API_SETTINGS
        .iter()
        .map(|(name, body, doc)| CompletionItem {
            documentation: Some(Documentation::String(doc.to_string())),
            kind: Some(CompletionItemKind::PROPERTY),
            ..snippet(name, first_sentence(doc), body)
        })
        .collect();
    for verb in webfluent::syntax::parser::v2::HTTP_VERBS {
        items.push(CompletionItem {
            kind: Some(CompletionItemKind::KEYWORD),
            ..snippet(
                verb,
                &format!(
                    "an endpoint the service answers with {}",
                    verb.to_uppercase()
                ),
                &format!("{verb} ${{1:name}}(${{2}}) -> ${{3:Map}}"),
            )
        });
    }
    items.push(snippet(
        "on",
        "a hook on every request, response or error",
        "on ${1|request,response,error|}(${2:r}) {\n\t$0\n}",
    ));
    items
}

/// Whether an element's first argument may be a value without a name.
fn takes_positional(project: &Project, name: &str) -> bool {
    match find_component(project, name) {
        Some(component) => component.props.iter().any(|p| p.positional),
        None => signature(name).is_some_and(|sig| sig.positional.is_some()),
    }
}

/// Whether the cursor is where a key of a responsive value goes: just after
/// `prop: {` or after a `,` inside one, in an element's parentheses.
fn responsive_key_position(previous: &[&Token]) -> bool {
    if !matches!(
        previous.last().map(|t| &t.token_type),
        Some(TokenType::OpenBrace | TokenType::Comma)
    ) {
        return false;
    }
    let mut depth = 0i32;
    for (ix, token) in previous.iter().enumerate().rev() {
        match token.token_type {
            TokenType::CloseBrace | TokenType::CloseParen | TokenType::CloseBracket => depth += 1,
            TokenType::OpenParen | TokenType::OpenBracket if depth == 0 => return false,
            TokenType::OpenBrace if depth == 0 => {
                return matches!(
                    (
                        ix.checked_sub(1).map(|i| &previous[i].token_type),
                        ix.checked_sub(2).map(|i| &previous[i].token_type)
                    ),
                    (Some(TokenType::Colon), Some(TokenType::Identifier(_)))
                ) && matches!(
                    open_paren_owner(&previous[..ix]),
                    Some(ParenOwner::Element(_))
                );
            }
            TokenType::OpenBrace | TokenType::OpenParen | TokenType::OpenBracket => depth -= 1,
            _ => {}
        }
    }
    false
}

/// Completion inside a string's `{…}` splice: after `name.`, what that
/// value has; otherwise the names in scope, the program's constants and the
/// language's functions.
fn splice_items(
    project: &Project,
    file_ix: usize,
    source: &str,
    offset: usize,
) -> Vec<CompletionItem> {
    let before = &source[..offset];
    let typed = before.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_');
    if let Some(owner_text) = typed.strip_suffix('.') {
        let owner_text = owner_text.strip_suffix('?').unwrap_or(owner_text);
        let owner: String = owner_text
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        if owner.is_empty() {
            return Vec::new();
        }
        if let Some(store) = project.program.declarations.iter().find_map(|d| match d {
            Declaration::Store(s) if s.name == owner => Some(s),
            _ => None,
        }) {
            return analysis::store_members(store)
                .into_iter()
                .map(|m| binding_item(&m, Some(&store.name)))
                .collect();
        }
        return value_members(project, file_ix, offset, &owner);
    }
    let mut items = Vec::new();
    if let Some(decl_ix) = analysis::declaration_at(project, file_ix, offset) {
        items.extend(scope_items(&analysis::scope_at(
            &project.program.declarations[decl_ix],
            offset,
        )));
    }
    items.extend(top_level_names(project));
    items.extend(builtin_items());
    items
}

/// What a string the cursor is in names, by where it is written:
/// `on key("‸")` a key, `Host(tag: "‸")` a tag, `t("‸")` a message.
fn string_value(project: &Project, tokens: &[Token], offset: usize) -> Vec<CompletionItem> {
    let Some(ix) = tokens.iter().position(|t| {
        matches!(t.token_type, TokenType::StringLiteral(_)) && t.offset < offset && offset < t.end
    }) else {
        return Vec::new();
    };
    let at = |back: usize| ix.checked_sub(back).map(|i| &tokens[i].token_type);
    let word = |back: usize| match at(back) {
        Some(TokenType::Identifier(w)) => Some(w.as_str()),
        _ => None,
    };
    let values = |names: Vec<(String, String)>, kind: CompletionItemKind| -> Vec<CompletionItem> {
        names
            .into_iter()
            .map(|(label, detail)| CompletionItem {
                label,
                kind: Some(kind),
                detail: Some(detail),
                ..Default::default()
            })
            .collect()
    };
    // `on key("…")`: the keys, and a modifier to begin with.
    if matches!(at(1), Some(TokenType::OpenParen))
        && word(2) == Some("key")
        && word(3) == Some("on")
    {
        let mut names: Vec<(String, String)> = webfluent::syntax::parser::v2::KEY_NAMES
            .iter()
            .map(|k| (k.to_string(), "a key".to_string()))
            .collect();
        names.extend(
            webfluent::syntax::parser::v2::KEY_MODIFIERS
                .iter()
                .map(|m| {
                    (
                        format!("{m}+"),
                        "a modifier, then the key: `ctrl+k`".to_string(),
                    )
                }),
        );
        return values(names, CompletionItemKind::CONSTANT);
    }
    // `Host(tag: "…")`: the elements a Host is made of.
    if matches!(at(1), Some(TokenType::Colon)) && word(2) == Some("tag") {
        let in_host = tokens[..ix].iter().rev().find_map(|t| match &t.token_type {
            TokenType::Identifier(w) if w.chars().next().is_some_and(char::is_uppercase) => {
                Some(w == "Host")
            }
            _ => None,
        });
        if in_host == Some(true) {
            return values(
                webfluent::codegen::builtin::HOST_TAGS
                    .iter()
                    .map(|t| (t.to_string(), format!("Host makes a <{t}>")))
                    .collect(),
                CompletionItemKind::VALUE,
            );
        }
    }
    // `t("…")`: the messages the project's translations hold.
    if matches!(at(1), Some(TokenType::OpenParen)) && word(2) == Some("t") {
        return values(
            project
                .messages
                .iter()
                .map(|k| (k.clone(), "a message in the translations".to_string()))
                .collect(),
            CompletionItemKind::TEXT,
        );
    }
    Vec::new()
}

/// The codes `// wf-allow(…)` may name: those `lints` could lower.
fn allow_codes() -> Vec<CompletionItem> {
    webfluent::diagnostics::codes::CODES
        .iter()
        .filter(|c| webfluent::diagnostics::codes::lowerable(c.code))
        .map(|c| CompletionItem {
            label: c.code.to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: Some(c.title.replace('`', "")),
            documentation: Some(Documentation::String(c.summary.to_string())),
            ..Default::default()
        })
        .collect()
}

/// The name of the call whose parenthesis is open at the end of `previous`.
fn call_name(previous: &[&Token]) -> Option<String> {
    let mut depth = 0i32;
    for (ix, token) in previous.iter().enumerate().rev() {
        match token.token_type {
            TokenType::CloseParen => depth += 1,
            TokenType::OpenParen => {
                if depth == 0 {
                    return match &previous.get(ix.checked_sub(1)?)?.token_type {
                        TokenType::Identifier(name) => Some(name.clone()),
                        _ => None,
                    };
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// What follows `name.` for a name: a record's fields, and everything else
/// the checker says a value of its type has (`webfluent::sema::types::
/// members`) — a list's, a string's, a date's, money's, a resource's, a
/// form's — for a name in scope, a constant, `data`, an image or a value the
/// browser gives; and a handle's members for an action (`.pending`), a
/// connection, a form or an element.
fn value_members(
    project: &Project,
    file_ix: usize,
    anchor: usize,
    name: &str,
) -> Vec<CompletionItem> {
    use webfluent::sema::types::{self as types, Type};
    let member_item = |name: &str, method: bool, detail: String, sort: &str| CompletionItem {
        label: name.to_string(),
        kind: Some(if method {
            CompletionItemKind::METHOD
        } else {
            CompletionItemKind::FIELD
        }),
        detail: Some(detail),
        insert_text: Some(if method {
            format!("{name}($0)")
        } else {
            name.to_string()
        }),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("{sort}{name}")),
        ..Default::default()
    };
    let handle = |names: &[(&str, bool, &str)]| -> Vec<CompletionItem> {
        names
            .iter()
            .map(|(n, m, d)| member_item(n, *m, d.to_string(), "0"))
            .collect()
    };
    let decl_ix = analysis::declaration_at(project, file_ix, anchor.saturating_sub(1));
    let binding = decl_ix.and_then(|ix| {
        analysis::scope_at(&project.program.declarations[ix], anchor)
            .into_iter()
            .find(|b| b.name == name)
    });
    if let Some(b) = &binding {
        let connection = [
            ("state", false, "connecting, open, closed or error"),
            ("messages", false, "what arrived, in order"),
            (
                "last",
                true,
                "the latest message, or the latest of one kind",
            ),
            ("error", false, "what went wrong"),
            ("close", true, "close it now"),
        ];
        match b.kind {
            BindingKind::Action => {
                return handle(&[("pending", false, "whether a call of it is under way")]);
            }
            BindingKind::Socket | BindingKind::Peer => {
                let mut items = handle(&connection);
                items.extend(handle(&[
                    ("send", true, "send a value"),
                    ("closure", false, "the close code and reason"),
                ]));
                if b.kind == BindingKind::Peer {
                    items.extend(handle(&[(
                        "signal",
                        true,
                        "hand it what the other side sent",
                    )]));
                }
                return items;
            }
            BindingKind::Stream => {
                let mut items = handle(&connection);
                items.extend(handle(&[("closure", false, "the close code and reason")]));
                return items;
            }
            BindingKind::Worker => {
                return handle(&[
                    ("request", true, "ask it, and await its answer"),
                    ("send", true, "post a value, expecting nothing back"),
                    (
                        "last",
                        true,
                        "the latest message, or the latest of one kind",
                    ),
                    ("pending", false, "how many requests are out"),
                    ("state", false, "open, error or closed"),
                    ("error", false, "why it stopped"),
                    ("closure", false, "how it was stopped"),
                    ("restart", true, "start it again"),
                    ("close", true, "stop it now"),
                ]);
            }
            BindingKind::Channel => {
                let mut items = handle(&connection);
                items.extend(handle(&[
                    ("post", true, "send a value to every tab"),
                    ("closure", false, "the close code and reason"),
                ]));
                return items;
            }
            BindingKind::ElementHandle => {
                return handle(&[
                    ("focus", true, "move focus to it"),
                    ("blur", true, "take focus from it"),
                    ("value", false, "a control's value"),
                    ("select", true, "select a field's text"),
                    ("click", true, "click it"),
                    ("scrollIntoView", true, "scroll it into view"),
                ]);
            }
            _ => {}
        }
    }
    let ty: Option<Type> = match &binding {
        Some(b) if b.kind == BindingKind::FormHandle => Some(types::form_type()),
        Some(b) => decl_ix.and_then(|ix| crate::hover::type_of_binding(project, ix, b)),
        None => top_level_type(project, name).or_else(|| types::browser_value_type(name)),
    };
    let Some(ty) = ty else {
        return Vec::new();
    };
    let mut items = Vec::new();
    // A record's fields, by the program's declaration.
    let record = match &ty {
        Type::Record(r) => Some(r.clone()),
        Type::Optional(inner) => match inner.as_ref() {
            Type::Record(r) => Some(r.clone()),
            _ => None,
        },
        _ => None,
    };
    if let Some(record) = record
        && let Some(t) = project.program.declarations.iter().find_map(|d| match d {
            Declaration::Type(t) if t.name == record => Some(t),
            _ => None,
        })
    {
        for f in t.all_fields(&|name| crate::hover::find_type(project, name)) {
            items.push(CompletionItem {
                label: f.name.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(format!("{} — field of {}", type_name(&f.ty), record)),
                documentation: f.doc.clone().map(Documentation::String),
                sort_text: Some(format!("0{}", f.name)),
                ..Default::default()
            });
        }
    }
    for m in types::members(&ty) {
        let detail = match &m.ty {
            Some(t) => format!("{t}"),
            None if m.method => "method".to_string(),
            None => String::new(),
        };
        items.push(member_item(&m.name, m.method, detail, "1"));
    }
    items
}

/// The type the checker gives a constant, a `data` constant or an image.
fn top_level_type(project: &Project, name: &str) -> Option<webfluent::sema::types::Type> {
    let ix = project.program.declarations.iter().position(|d| match d {
        Declaration::Const(c) => c.name == name,
        Declaration::Data(d) => d.name == name,
        _ => false,
    })?;
    let info = webfluent::sema::types::check(&project.program, &|_| String::new());
    info.bindings
        .iter()
        .find(|t| t.decl == ix && t.name == name)
        .map(|t| t.ty.clone())
        .filter(|t| !t.is_any())
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

/// The events an `on` handler can name where the cursor is: in an
/// element's block, the DOM events and the events the component declares;
/// in a connection's block, what that connection says; in an `api` block,
/// its hooks; anywhere else in a page, a component or the app, the events
/// the parser takes there (`PAGE_EVENTS`): the document's keys and what
/// only the window says.
fn events(
    project: &Project,
    file_ix: usize,
    anchor: usize,
    previous: &[&Token],
) -> Vec<CompletionItem> {
    let event_item = |name: &str, doc: &str, insert: String, sort: &str| CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::EVENT),
        detail: Some(doc.to_string()),
        insert_text: Some(insert),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("{sort}{name}")),
        ..Default::default()
    };
    // `socket chat = ws(…) { on ‸`: what the connection says.
    if let Some(word) = block_word(previous)
        && let Some(events) = connection_events_of(&word)
    {
        return events
            .iter()
            .map(|(name, doc)| event_item(name, doc, format!("{name}(${{1:m}}) {{\n\t$0\n}}"), "0"))
            .collect();
    }
    let decl = analysis::declaration_at(project, file_ix, anchor.saturating_sub(1))
        .map(|ix| &project.program.declarations[ix]);
    if let Some(Declaration::Api(_)) = decl {
        return [
            ("request", "Before every request: change its headers"),
            ("response", "After every response"),
            ("error", "When a request fails, before the caller hears it"),
        ]
        .iter()
        .map(|(name, doc)| event_item(name, doc, format!("{name}(${{1:r}}) {{\n\t$0\n}}"), "0"))
        .collect();
    }
    if in_element_block(previous) {
        let mut items = declared_events(project, file_ix, anchor);
        items.extend(
            reference::events()
                .iter()
                .map(|(name, doc)| event_item(name, doc, format!("{name} {{\n\t$0\n}}"), "1")),
        );
        return items;
    }
    webfluent::syntax::parser::v2::PAGE_EVENTS
        .iter()
        .map(|name| {
            let insert = if *name == "key" {
                "key(\"${1:mod+k}\") {\n\t$0\n}".to_string()
            } else {
                format!("{name} {{\n\t$0\n}}")
            };
            event_item(name, reference::page_event_doc(name), insert, "1")
        })
        .collect()
}

/// The events a connection declared with `word` (`socket`, `worker`, …)
/// may handle; `None` for any other word.
fn connection_events_of(word: &str) -> Option<&'static [(&'static str, &'static str)]> {
    reference::CONNECTION_EVENTS
        .iter()
        .find(|(kind, _)| *kind == word)
        .map(|(_, events)| *events)
}

/// A connection's block: its handlers, and — for a socket or a peer — the
/// types it sends and receives.
fn connection_block(word: &str) -> Vec<CompletionItem> {
    let mut items = vec![snippet(
        "on",
        "a handler for what the connection says",
        "on ${1:message}(${2:m}) {\n\t$0\n}",
    )];
    if matches!(word, "socket" | "peer") {
        items.push(snippet(
            "send",
            "the type of what is sent",
            "send ${1:Outgoing}",
        ));
        items.push(snippet(
            "receive",
            "the type of what arrives",
            "receive ${1:Incoming}",
        ));
    }
    items
}

/// A `sequence { }` body: its steps.
fn sequence_steps() -> Vec<CompletionItem> {
    let doc = reference::keyword("step").map_or("", |k| k.summary);
    vec![
        CompletionItem {
            kind: Some(CompletionItemKind::KEYWORD),
            documentation: Some(Documentation::String(doc.to_string())),
            ..snippet("step", "a step that starts at once", "step {\n\t$0\n}")
        },
        CompletionItem {
            kind: Some(CompletionItemKind::KEYWORD),
            documentation: Some(Documentation::String(doc.to_string())),
            filter_text: Some("step after".to_string()),
            ..snippet(
                "step(after:)",
                "a step that starts after the one before it",
                "step(after: \"${1:120ms}\") {\n\t$0\n}",
            )
        },
    ]
}

/// After `on: .`, `cache: .`, `as: .` in a request's parentheses: the cases
/// that option takes.
fn option_cases(key: &str) -> Option<Vec<CompletionItem>> {
    let (_, cases) = reference::OPTION_CASES.iter().find(|(k, _)| *k == key)?;
    Some(
        cases
            .iter()
            .map(|(name, insert, doc)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some(format!("{key}: .{name}")),
                documentation: Some(Documentation::String(doc.to_string())),
                insert_text: Some(insert.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect(),
    )
}

/// `items` without the argument names already written in the open
/// parenthesis.
fn unwritten(items: Vec<CompletionItem>, previous: &[&Token]) -> Vec<CompletionItem> {
    let written = written_arg_names(previous);
    items
        .into_iter()
        .filter(|item| {
            !written
                .iter()
                .any(|w| item.label.trim_end_matches(':') == w)
        })
        .collect()
}

/// Whether the innermost open block is an element's — `Button("x") { ‸` —
/// where `on`, `style` and `transition` belong, rather than a page's, a
/// component's or a branch's.
fn in_element_block(previous: &[&Token]) -> bool {
    block_word(previous).is_some_and(|w| w.starts_with(char::is_uppercase))
}

/// Whether the cursor follows the `}` that closes a `try`'s block, where
/// `catch` goes.
fn after_try(previous: &[&Token]) -> bool {
    if !matches!(
        previous.last().map(|t| &t.token_type),
        Some(TokenType::CloseBrace)
    ) {
        return false;
    }
    let mut depth = 0i32;
    for (ix, token) in previous.iter().enumerate().rev() {
        match token.token_type {
            TokenType::CloseBrace => depth += 1,
            TokenType::OpenBrace => {
                depth -= 1;
                if depth == 0 {
                    return matches!(
                        ix.checked_sub(1).map(|i| &previous[i].token_type),
                        Some(TokenType::Identifier(w)) if w == "try"
                    );
                }
            }
            _ => {}
        }
    }
    false
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

/// What a file's top level can declare: every top-level keyword the table
/// documents, each with the snippet that writes one.
fn top_level() -> Vec<CompletionItem> {
    reference::KEYWORDS
        .iter()
        .filter(|k| k.place == Place::TopLevel)
        .map(|k| {
            let body = match k.name {
                "page" => "page ${1:Name}(path: \"${2:/}\", title: \"${3:$1}\", description: \"${4}\") {\n\t$0\n}",
                "component" => "component ${1:Name}(${2:label}: ${3:String}) {\n\t$0\n}",
                "store" => "store ${1:Name}Store {\n\tstate ${2:items} = ${3:[]}\n\t$0\n}",
                "theme" => "theme ${1:Brand} {\n\tcolor-primary: ${2:#3B82F6}\n\t$0\n}",
                "app" => "app {\n\t$0\n\tRouter\n}",
                "type" => "type ${1:Name} {\n\t${2:id}: ${3:String}\n}",
                "enum" => "enum ${1:Name} { ${2:a}, ${3:b} }",
                "animation" => "animation ${1:Name} {\n\tfrom { ${2:opacity: 0} }\n\tto { ${3:opacity: 1} }\n}",
                "const" => "const ${1:NAME} = ${2:value}",
                "data" => "data ${1:name} = \"${2:data.json}\"",
                "image" => "image ${1:name} = \"${2:media/photo.jpg}\"",
                "api" => "api ${1:Backend}(base: \"${2:/api}\") {\n\tget ${3:items}() -> ${4:[Map]}\n}",
                "test" => "test \"${1:what it does}\" {\n\t$0\n}",
                other => other,
            };
            CompletionItem {
                documentation: Some(Documentation::String(k.summary.to_string())),
                ..snippet(k.name, first_sentence(k.summary), body)
            }
        })
        .collect()
}

/// The first sentence of a summary, for a completion's one-line detail.
fn first_sentence(text: &str) -> &str {
    text.split_once(". ").map_or(text, |(first, _)| first)
}

fn theme_body() -> Vec<CompletionItem> {
    reference::tokens()
        .iter()
        .map(|(name, doc)| CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: Some(doc.clone()),
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
    for (name, doc) in reference::tokens() {
        if seen.insert(name.clone()) {
            items.push(token_item(name, doc, "1"));
        }
    }
    if let Some(prop) = css_prop {
        let css_prop = webfluent::codegen::style_tokens::canonical_style_prop(&prop);
        for (name, _) in reference::tokens() {
            if let Some(short) = name.rsplit_once('-').map(|(_, s)| s)
                && webfluent::codegen::style_tokens::resolve_short_token(&css_prop, short, &|_| {
                    false
                })
                .as_deref()
                    == Some(name.as_str())
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

/// A connection's arm: its label, what it means, its snippet, and whether
/// a written arm is it.
type ConnectionArm = (
    &'static str,
    &'static str,
    &'static str,
    fn(&ArmPattern) -> bool,
);

/// Inside a `match x { }`: the arms not yet written — a resource's states,
/// an enum's cases (with names for a case's payload), or a connection's
/// states — and `else` while something is left.
fn match_arms(
    project: &Project,
    decl_ix: usize,
    m: &MatchStmt,
    scope: &[Binding],
) -> Vec<CompletionItem> {
    use webfluent::sema::types::Type;
    let written: Vec<&ArmPattern> = m.arms.iter().map(|a| &a.pattern).collect();
    let mut items = Vec::new();
    let has = |p: &dyn Fn(&ArmPattern) -> bool| written.iter().any(|w| p(w));
    let binding = match &m.scrutinee {
        Expr::Identifier(name) => scope.iter().find(|b| &b.name == name),
        _ => None,
    };
    let kind = binding.map(|b| b.kind);
    let is_connection = matches!(
        kind,
        Some(
            BindingKind::Socket
                | BindingKind::Stream
                | BindingKind::Channel
                | BindingKind::Peer
                | BindingKind::Worker
        )
    ) || has(&|p| matches!(p, ArmPattern::State(_)));
    // The enum the subject is, when the checker knows.
    let subject_enum = binding
        .filter(|_| !is_connection)
        .and_then(|b| crate::hover::type_of_binding(project, decl_ix, b))
        .and_then(|ty| match ty {
            Type::Enum(e) => Some(e),
            Type::Optional(inner) => match *inner {
                Type::Enum(e) => Some(e),
                _ => None,
            },
            _ => None,
        })
        .and_then(|name| find_enum(project, &name));
    let mut left = false;
    if let Some(e) = subject_enum {
        for case in &e.cases {
            if has(&|p| matches!(p, ArmPattern::Case(c) if *c == case.name)) {
                continue;
            }
            left = true;
            let names: Vec<String> = case
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| format!("${{{}:{}}}", i + 1, f.name))
                .collect();
            let head = if names.is_empty() {
                format!(".{}", case.name)
            } else {
                format!(".{}({})", case.name, names.join(", "))
            };
            items.push(CompletionItem {
                filter_text: Some(case.name.clone()),
                ..snippet(
                    &format!(".{}", case.name),
                    &format!("case of {}", e.name),
                    &format!("{head} {{\n\t$0\n}}"),
                )
            });
        }
    } else if is_connection {
        let arms: [ConnectionArm; 4] = [
            (
                "connecting",
                "While it opens",
                "connecting {\n\t$0\n}",
                |p| matches!(p, ArmPattern::State(s) if s == "connecting"),
            ),
            (
                "open",
                "Once it is open",
                "open {\n\t$0\n}",
                |p| matches!(p, ArmPattern::State(s) if s == "open"),
            ),
            (
                "closed",
                "Once it closed: the code and the reason",
                "closed(${1:c}) {\n\t$0\n}",
                |p| matches!(p, ArmPattern::State(s) if s == "closed"),
            ),
            ("error", "When it failed", "error(${1:e}) {\n\t$0\n}", |p| {
                matches!(p, ArmPattern::Error) || matches!(p, ArmPattern::State(s) if s == "error")
            }),
        ];
        for (label, detail, body, written_as) in arms {
            if !has(&written_as) {
                left = true;
                items.push(snippet(label, detail, body));
            }
        }
    } else if kind == Some(BindingKind::Resource)
        || written.is_empty()
        || has(&|p| {
            matches!(
                p,
                ArmPattern::Loading | ArmPattern::Error | ArmPattern::Ready
            )
        })
    {
        if !has(&|p| matches!(p, ArmPattern::Loading)) {
            left = true;
            items.push(snippet(
                "loading",
                "While the request is in flight",
                "loading {\n\t$0\n}",
            ));
        }
        if !has(&|p| matches!(p, ArmPattern::Error)) {
            left = true;
            items.push(snippet(
                "error",
                "When the request fails",
                "error(${1:e}) {\n\t$0\n}",
            ));
        }
        if !has(&|p| matches!(p, ArmPattern::Ready)) {
            left = true;
            items.push(snippet(
                "ready",
                "Once the data arrived",
                "ready(${1:value}) {\n\t$0\n}",
            ));
        }
    } else {
        left = true;
    }
    if left && !has(&|p| matches!(p, ArmPattern::Else)) {
        items.push(snippet("else", "Every other value", "else {\n\t$0\n}"));
    }
    items
}

fn style_block(
    project: &Project,
    source: &str,
    previous: &[&Token],
    offset: usize,
    tokens: &[Token],
) -> Vec<CompletionItem> {
    // In a value: the design tokens, as `$name`. A value ends with its
    // line, so a declaration on the line above leaves the cursor at a name.
    let in_value = previous.last().is_some_and(|t| {
        matches!(t.token_type, TokenType::StyleProp(_))
            || (matches!(t.token_type, TokenType::RawValue(_))
                && !source[t.end.min(offset)..offset].contains('\n'))
    }) || tokens.iter().any(|t| {
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

/// An `animation Name { }` body: the keyframes it may hold.
fn keyframes() -> Vec<CompletionItem> {
    [
        (
            "from",
            "The first frame",
            "from {\n\t${1:opacity}: ${2:0}\n}",
        ),
        ("to", "The last frame", "to {\n\t${1:opacity}: ${2:1}\n}"),
        (
            "50%",
            "A frame part of the way through",
            "${1:50}% {\n\t$0\n}",
        ),
    ]
    .iter()
    .map(|(label, detail, body)| CompletionItem {
        kind: Some(CompletionItemKind::KEYWORD),
        ..snippet(label, detail, body)
    })
    .collect()
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

    // `User(‸`: a record being built — the fields its type declares.
    if let Some(t) = crate::hover::find_type(project, name) {
        for f in t.all_fields(&|n| crate::hover::find_type(project, n)) {
            if written.contains(&f.name) {
                continue;
            }
            items.push(CompletionItem {
                label: format!("{}:", f.name),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(format!(
                    "{}{} — field of {name}",
                    type_name(&f.ty),
                    if f.default.is_some() { " = …" } else { "" }
                )),
                documentation: f.doc.clone().map(Documentation::String),
                insert_text: Some(format!("{}: ", f.name)),
                sort_text: Some(format!("{}{}", u8::from(f.default.is_some()), f.name)),
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

/// A type as it is written: the compiler's own printer, so a condition
/// reads as it was written (`Number(1..=30)`).
fn type_name(ty: &TypeRef) -> String {
    webfluent::syntax::print::type_ref(ty)
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

/// A call snippet for a built-in: its positional prop, if any, and every
/// prop it cannot be placed without (`Portal(into: null)`), so what is
/// inserted is never a `C01`.
fn builtin_snippet(sig: &ComponentSig) -> String {
    let mut args: Vec<String> = sig
        .positional
        .iter()
        .map(|p| format!("${{1:{}}}", p.name))
        .collect();
    for p in sig.props.iter().filter(|p| p.required) {
        let stop = args.len() + 1;
        args.push(format!("{}: ${{{stop}:{}}}", p.name, p.empty_value()));
    }
    let call = if args.is_empty() {
        sig.name.to_string()
    } else {
        format!("{}({})", sig.name, args.join(", "))
    };
    match sig.children {
        Children::Elements => format!("{call} {{\n\t$0\n}}"),
        Children::None => call,
    }
}

/// Whether the project writes paper — a PDF or a deck — where only what
/// can be drawn may be written.
fn paged(project: &Project) -> bool {
    matches!(project.output_type, OutputType::Pdf | OutputType::Slides)
}

/// A word that starts something only a browser runs: a handler, a timer, a
/// request, a connection, a value kept in storage.
fn web_only_keyword(word: &str) -> bool {
    matches!(
        word,
        "on" | "effect"
            | "every"
            | "after"
            | "resource"
            | "socket"
            | "stream"
            | "channel"
            | "peer"
            | "worker"
            | "persist"
            | "show"
            | "sequence"
            | "navigate"
            | "validate"
            | "head"
    )
}

fn components(project: &Project) -> Vec<CompletionItem> {
    components_in(project, None)
}

/// The elements a statement may start with: the built-ins this output
/// draws, then the project's components. In a deck, a `Presentation` holds
/// slides — the slide kinds and the components whose body is one — and a
/// slide holds anything else that draws.
fn components_in(project: &Project, enclosing: Option<&ComponentRef>) -> Vec<CompletionItem> {
    let output = project.output_type;
    let in_presentation =
        matches!(enclosing, Some(ComponentRef::BuiltIn(n)) if n == "Presentation");
    let in_slide = enclosing.is_some_and(|c| match c {
        ComponentRef::BuiltIn(n) => linter::is_slide(n),
        ComponentRef::UserDefined(n) => {
            find_component(project, n).is_some_and(|c| linter::renders_slides(&c.body))
        }
        ComponentRef::SubComponent(..) => false,
    });
    let slides = output == OutputType::Slides;
    let mut items: Vec<CompletionItem> = registry::components()
        .filter(|c| linter::offered_in(output, c.name))
        .filter(|c| {
            if !slides {
                return true;
            }
            let slide = linter::is_slide(c.name);
            if in_presentation {
                slide
            } else if in_slide || enclosing.is_some() {
                !slide && c.name != "Presentation"
            } else {
                !slide
            }
        })
        .map(|c| {
            // On a web page the paper-only elements come last.
            let paper_only =
                !paged(project) && matches!(c.group, "PDF" | "Slides") && c.name != "Section";
            let detail = if paper_only {
                format!("{} — {} (drawn in a PDF or a deck)", c.group, c.summary)
            } else {
                format!("{} — {}", c.group, c.summary)
            };
            CompletionItem {
                label: c.name.to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(detail),
                insert_text: Some(builtin_snippet(c)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("{}{}", if paper_only { 3 } else { 1 }, c.name)),
                ..Default::default()
            }
        })
        .collect();
    for (ix, decl) in project.program.declarations.iter().enumerate() {
        if let Declaration::Component(c) = decl {
            if slides && in_presentation != linter::renders_slides(&c.body) {
                continue;
            }
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

/// Where a statement is being written, which decides the words it can
/// begin with.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    Page,
    Component,
    Store,
    /// An action's, a handler's, an effect's or a timer's body.
    Imperative,
    /// A `test "…" { }` body.
    Test,
}

/// The place a statement is being written: the kind of body, and what
/// surrounds the cursor within it.
#[derive(Clone, Copy)]
struct Spot {
    context: Context,
    /// In an element's block, where `on`, `style` and `transition` go.
    in_element: bool,
    /// Directly in an effect's body, which `cleanup { }` may close.
    in_effect: bool,
    /// Just after a `try`'s block, where `catch` goes.
    after_try: bool,
}

/// The words a statement may begin with at `spot`, read from the parser's
/// own tables (`BODY_KEYWORDS`, `IMPERATIVE_KEYWORDS`), so a word the
/// parser learns is offered with no change here — and a word it refuses
/// where the cursor is is not.
fn keywords(spot: Spot) -> Vec<CompletionItem> {
    use webfluent::syntax::parser::v2::{BODY_KEYWORDS, IMPERATIVE_KEYWORDS};
    // A component's own words: declared in it, or placing what its caller gave.
    let component_only = |w: &str| matches!(w, "event" | "slot" | "part" | "children");
    // What a render body takes, where the cursor is. `navigate`, `log` and
    // `emit` are statements of an action or a handler; `style` and
    // `transition` belong in an element's block.
    let render = |w: &str, component: bool| -> bool {
        match w {
            "navigate" | "log" | "emit" => false,
            "style" | "transition" => spot.in_element,
            "head" => !component,
            _ if component_only(w) => component,
            _ => true,
        }
    };
    let mut words: Vec<&str> = match spot.context {
        Context::Page | Context::Test => BODY_KEYWORDS
            .iter()
            .copied()
            .filter(|w| render(w, false))
            .chain(["else"])
            .collect(),
        Context::Component => BODY_KEYWORDS
            .iter()
            .copied()
            .filter(|w| render(w, true))
            .chain(["else"])
            .collect(),
        // A store holds values, what changes them, and what it keeps open.
        Context::Store => BODY_KEYWORDS
            .iter()
            .copied()
            .filter(|w| {
                matches!(
                    *w,
                    "state"
                        | "persist"
                        | "derived"
                        | "action"
                        | "effect"
                        | "every"
                        | "after"
                        | "use"
                        | "resource"
                        | "socket"
                        | "stream"
                        | "channel"
                        | "peer"
                        | "worker"
                )
            })
            .collect(),
        Context::Imperative => {
            let mut words: Vec<&str> = IMPERATIVE_KEYWORDS.to_vec();
            words.extend(["await", "navigate", "log"]);
            if spot.in_effect {
                words.push("cleanup");
            }
            words
        }
    };
    if spot.after_try {
        words.insert(0, "catch");
    }
    let mut seen = std::collections::HashSet::new();
    words.retain(|w| seen.insert(*w));
    let mut items: Vec<CompletionItem> = words
        .into_iter()
        .map(|name| {
            let doc = reference::keyword(name).map(|k| k.summary.to_string());
            CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: doc,
                // Outside an element, a page hears the document's keys and
                // the window, not a click.
                insert_text: Some(if name == "on" && !spot.in_element {
                    "on ${1:keydown}(${2:e}) {\n\t$0\n}".to_string()
                } else {
                    keyword_snippet(name)
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("{}{name}", if name == "catch" { 0 } else { 2 })),
                ..Default::default()
            }
        })
        .collect();
    // A test's steps, each in the sense a test gives it (`type` types).
    if spot.context == Context::Test {
        items.extend(
            reference::KEYWORDS
                .iter()
                .filter(|k| k.place == Place::Test)
                .map(|k| CompletionItem {
                    label: k.name.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some(k.summary.to_string()),
                    insert_text: Some(keyword_snippet(k.name)),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    sort_text: Some(format!("2{}", k.name)),
                    ..Default::default()
                }),
        );
    }
    items
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
        "persist" => "persist ${1:name} = ${2:value}".into(),
        "migrate" => "migrate ${1:1} -> ${2:2} { ${0:old} }".into(),
        "sequence" => {
            "sequence {\n\tstep { $0 }\n\tstep(after: \"${1:120ms}\") { }\n}".into()
        }
        "navigate" => "navigate(\"${1:/}\")".into(),
        "log" => "log(${1:value})".into(),
        "emit" => "emit ${1:event}($0)".into(),
        "event" => "event ${1:name}(${2})".into(),
        "slot" => "slot ${1:name}".into(),
        "children" => "children".into(),
        "on" => "on ${1:click} {\n\t$0\n}".into(),
        "style" => "style {\n\t${1:padding}: ${2:1rem}\n\t$0\n}".into(),
        "transition" => "transition {\n\t${1:background}: ${2:200ms} ${3:ease}\n}".into(),
        "validate" => "validate ${1:name} {\n\t${2:required}\n}".into(),
        "socket" => "socket ${1:chat} = ws(\"${2:wss://}\") {\n\ton message(${3:m}) { $0 }\n}".into(),
        "stream" => "stream ${1:events} = sse(\"${2:/events}\")".into(),
        "channel" => "channel ${1:tabs} = broadcast(\"${2:name}\") {\n\ton message(${3:m}) { $0 }\n}".into(),
        "peer" => "peer ${1:link} = rtc(signal: ${2:m} => ${3:lobby}.post($2), initiator: ${4:true}) {\n\ton message(${5:m}) { $0 }\n}".into(),
        "worker" => "worker ${1:task} = spawn(\"${2:/workers/task.js}\") {\n\ton message(${3:m}) { $0 }\n}".into(),
        "cleanup" => "cleanup {\n\t$0\n}".into(),
        "catch" => "catch ${1:e} {\n\t$0\n}".into(),
        "every" => "every(${1:1000}) {\n\t$0\n}".into(),
        "after" => "after(${1:1000}) {\n\t$0\n}".into(),
        "head" => "head {\n\t${1:meta}(${2:name}: \"$3\", content: \"$4\")\n}".into(),
        "part" => "part ${1:Header} {\n\t$0\n}".into(),
        "let" => "let ${1:name} = ${2:value}".into(),
        "return" => "return ${1:value}".into(),
        "await" => "await ${1:call}".into(),
        "try" => "try {\n\t$1\n} catch ${2:e} {\n\t$0\n}".into(),
        "expect" => "expect \"${1:text}\"".into(),
        "click" => "click \"${1:Save}\"".into(),
        "type" => "type \"${1:text}\" into \"${2:Label}\"".into(),
        "press" => "press \"${1:Enter}\"".into(),
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
    match open_paren_owner(previous) {
        Some(ParenOwner::Element(name)) => return element_props(project, &name, previous),
        Some(owner @ (ParenOwner::Fetch | ParenOwner::Opener(_) | ParenOwner::Member(..))) => {
            let mut items = call_options(project, &owner, previous);
            items.extend(top_level_names(project));
            items.extend(builtin_items());
            return items;
        }
        _ => {}
    }
    let mut items = components(project);
    items.extend(keywords(Spot {
        context: Context::Page,
        in_element: in_element_block(previous),
        in_effect: false,
        after_try: false,
    }));
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
