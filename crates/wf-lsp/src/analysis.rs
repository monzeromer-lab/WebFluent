//! What is under the cursor, read from the syntax tree rather than guessed
//! from the text.
//!
//! The parser records a byte span for every declaration, statement, element,
//! argument and modifier, so the server can tell an element name from a
//! modifier from a named argument from a string that happens to contain the
//! word `Button`, and can tell which page or component a `count` belongs to.

use webfluent::lexer::{Token, TokenType};
use webfluent::parser::ast::*;

use crate::project::{Project, SourceFile};

/// The declaration in `file` whose source contains `offset`, as an index into
/// the merged program.
///
/// `App` carries no span of its own; it is the declaration whose body
/// statements bracket the offset, or, failing that, the one file-level gap
/// not covered by any spanned declaration.
pub fn declaration_at(project: &Project, file_ix: usize, offset: usize) -> Option<usize> {
    let mut app: Option<usize> = None;
    for (ix, decl) in project.declarations_of(file_ix) {
        match decl {
            Declaration::Page(p) if contains(p.span, offset) => return Some(ix),
            Declaration::Component(c) if contains(c.span, offset) => return Some(ix),
            Declaration::Store(s) if contains(s.span, offset) => return Some(ix),
            Declaration::Theme(t) if contains(t.span, offset) => return Some(ix),
            Declaration::App(a) => {
                let start = a.body.first().map(|s| s.span.start as usize);
                let end = a.body.last().map(|s| s.span.end as usize);
                if let (Some(start), Some(end)) = (start, end)
                    && start <= offset
                    && offset <= end
                {
                    return Some(ix);
                }
                app = Some(ix);
            }
            _ => {}
        }
    }
    app
}

pub fn contains(span: Span, offset: usize) -> bool {
    (span.start as usize) <= offset && offset < (span.end as usize)
}

/// The statements a declaration's body holds.
pub fn body_of(decl: &Declaration) -> &[Statement] {
    match decl {
        Declaration::Page(p) => &p.body,
        Declaration::Component(c) => &c.body,
        Declaration::Store(s) => &s.body,
        Declaration::App(a) => &a.body,
        Declaration::Theme(_) | Declaration::Type(_) | Declaration::Enum(_) => &[],
    }
}

/// Every statement list nested in a statement: bodies, branches, handlers.
pub fn child_bodies(stmt: &Statement) -> Vec<&[Statement]> {
    match &stmt.kind {
        StatementKind::UIElement(el) => {
            let mut bodies: Vec<&[Statement]> = vec![&el.children];
            bodies.extend(el.events.iter().map(|e| e.body.as_slice()));
            bodies.extend(el.slot_fills.iter().map(|f| f.body.as_slice()));
            bodies
        }
        StatementKind::If(i) => {
            let mut bodies: Vec<&[Statement]> = vec![&i.then_body];
            bodies.extend(i.else_if_branches.iter().map(|(_, b)| b.as_slice()));
            bodies.extend(i.else_body.as_deref());
            bodies
        }
        StatementKind::For(f) => vec![&f.body],
        StatementKind::Show(s) => vec![&s.body],
        StatementKind::Fetch(f) => {
            let mut bodies: Vec<&[Statement]> = Vec::new();
            bodies.extend(f.loading_block.as_deref());
            bodies.extend(f.error_block.as_ref().map(|(_, b)| b.as_slice()));
            bodies.extend(f.success_block.as_deref());
            bodies
        }
        StatementKind::Match(m) => m.arms.iter().map(|a| a.body.as_slice()).collect(),
        StatementKind::Effect(e) => vec![&e.body],
        StatementKind::Action(a) => vec![&a.body],
        StatementKind::EventHandler(h) => vec![&h.body],
        _ => Vec::new(),
    }
}

/// The chain of statements from the body down to the innermost one that
/// contains `offset`, outermost first.
pub fn statement_path(stmts: &[Statement], offset: usize) -> Vec<&Statement> {
    let mut path = Vec::new();
    let mut current = stmts;
    'descend: loop {
        for stmt in current {
            if contains(stmt.span, offset) {
                path.push(stmt);
                for body in child_bodies(stmt) {
                    if body.iter().any(|s| contains(s.span, offset)) {
                        current = body;
                        continue 'descend;
                    }
                }
                break 'descend;
            }
        }
        break;
    }
    path
}

/// The innermost element that contains `offset`.
pub fn element_at(stmts: &[Statement], offset: usize) -> Option<&UIElement> {
    statement_path(stmts, offset)
        .into_iter()
        .rev()
        .find_map(|stmt| match &stmt.kind {
            StatementKind::UIElement(el) => Some(el),
            _ => None,
        })
}

/// What the cursor is on inside an element, when it is on a specific part.
pub enum ElementPart<'a> {
    /// The component name itself.
    Name,
    /// A modifier word, by index into `modifiers`.
    Modifier(usize),
    /// The name of a named argument, by index into `args`.
    ArgumentName(&'a str),
    /// The `style { … }` block.
    Style,
}

/// The name a component reference is written as: `Card`, `Card.Header`,
/// `UserCard`.
pub fn component_name(reference: &ComponentRef) -> String {
    match reference {
        ComponentRef::BuiltIn(n) | ComponentRef::UserDefined(n) => n.clone(),
        ComponentRef::SubComponent(p, c) => format!("{p}.{c}"),
    }
}

/// Which part of `el` sits at `offset`, if a specific one does.
pub fn element_part_at<'a>(
    el: &'a UIElement,
    source: &str,
    offset: usize,
) -> Option<ElementPart<'a>> {
    let name_len = component_name(&el.component).len();
    let name_start = el.span.start as usize;
    if name_start <= offset && offset <= name_start + name_len {
        return Some(ElementPart::Name);
    }
    if let Some(ix) = el
        .modifier_spans
        .iter()
        .position(|span| contains(*span, offset) || span.end as usize == offset)
    {
        return Some(ElementPart::Modifier(ix));
    }
    for (arg, span) in el.args.iter().zip(&el.arg_spans) {
        if let Arg::Named(name, _) = arg {
            let start = span.start as usize;
            let text = source.get(start..span.end as usize).unwrap_or("");
            // The name runs up to the colon.
            let name_end = start + text.find(':').unwrap_or(name.len());
            if start <= offset && offset <= name_end {
                return Some(ElementPart::ArgumentName(name));
            }
        }
    }
    if el.style_span.is_some_and(|span| contains(span, offset)) {
        return Some(ElementPart::Style);
    }
    None
}

/// A name in scope and what declared it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub name: String,
    pub kind: BindingKind,
    /// Byte range of the declaring statement or header, in its own file.
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    State,
    Derived,
    Action,
    Prop,
    Param,
    LoopItem,
    LoopIndex,
    FetchResult,
    FetchError,
    Store,
    /// A `resource`, rendered with `match`.
    Resource,
    /// The value or error a `match` arm binds, or an `if let` name.
    ArmBinding,
}

impl BindingKind {
    pub fn label(self) -> &'static str {
        match self {
            BindingKind::State => "state",
            BindingKind::Derived => "derived",
            BindingKind::Action => "action",
            BindingKind::Prop => "prop",
            BindingKind::Param => "parameter",
            BindingKind::LoopItem => "loop item",
            BindingKind::LoopIndex => "loop index",
            BindingKind::FetchResult => "fetch result",
            BindingKind::FetchError => "fetch error",
            BindingKind::Store => "store",
            BindingKind::Resource => "resource",
            BindingKind::ArmBinding => "binding",
        }
    }
}

/// Everything a name at `offset` inside `decl` can refer to, innermost first:
/// loop variables and fetch bindings of the enclosing statements, then the
/// declaration's own props, state, derived values and actions (hoisted —
/// the compiler resolves them regardless of order), then the stores it
/// `use`s.
pub fn scope_at(decl: &Declaration, offset: usize) -> Vec<Binding> {
    let mut scope = Vec::new();
    let body = body_of(decl);

    for stmt in statement_path(body, offset).into_iter().rev() {
        match &stmt.kind {
            StatementKind::For(f) => {
                scope.push(Binding {
                    name: f.item.clone(),
                    kind: BindingKind::LoopItem,
                    span: stmt.span,
                });
                if let Some(index) = &f.index {
                    scope.push(Binding {
                        name: index.clone(),
                        kind: BindingKind::LoopIndex,
                        span: stmt.span,
                    });
                }
            }
            StatementKind::Fetch(f) => {
                scope.push(Binding {
                    name: f.variable.clone(),
                    kind: BindingKind::FetchResult,
                    span: stmt.span,
                });
                if let Some((err, _)) = &f.error_block {
                    scope.push(Binding {
                        name: err.clone(),
                        kind: BindingKind::FetchError,
                        span: stmt.span,
                    });
                }
            }
            StatementKind::Action(a) => {
                for p in &a.params {
                    scope.push(Binding {
                        name: p.name.clone(),
                        kind: BindingKind::Param,
                        span: stmt.span,
                    });
                }
            }
            StatementKind::If(i) => {
                if let Some(name) = &i.binding {
                    scope.push(Binding {
                        name: name.clone(),
                        kind: BindingKind::ArmBinding,
                        span: stmt.span,
                    });
                }
            }
            StatementKind::Match(m) => {
                for arm in &m.arms {
                    if let Some(name) = &arm.binding
                        && contains(arm.span, offset)
                    {
                        scope.push(Binding {
                            name: name.clone(),
                            kind: BindingKind::ArmBinding,
                            span: arm.span,
                        });
                    }
                }
            }
            StatementKind::EventHandler(h) => {
                if let Some(param) = &h.param {
                    scope.push(Binding {
                        name: param.clone(),
                        kind: BindingKind::Param,
                        span: h.span,
                    });
                }
            }
            _ => {}
        }
    }

    if let Declaration::Component(c) = decl {
        for p in &c.props {
            scope.push(Binding {
                name: p.name.clone(),
                kind: BindingKind::Prop,
                span: c.header_span,
            });
        }
    }

    hoisted(body, &mut scope);
    scope
}

/// The state, derived values, actions and `use`d stores declared anywhere in
/// a body, at any depth — the compiler hoists them all.
pub fn hoisted(stmts: &[Statement], scope: &mut Vec<Binding>) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::State(s) => scope.push(Binding {
                name: s.name.clone(),
                kind: BindingKind::State,
                span: stmt.span,
            }),
            StatementKind::Derived(d) => scope.push(Binding {
                name: d.name.clone(),
                kind: BindingKind::Derived,
                span: stmt.span,
            }),
            StatementKind::Action(a) => scope.push(Binding {
                name: a.name.clone(),
                kind: BindingKind::Action,
                span: stmt.span,
            }),
            StatementKind::Use(u) => scope.push(Binding {
                name: u.store_name.clone(),
                kind: BindingKind::Store,
                span: stmt.span,
            }),
            StatementKind::Resource(r) => scope.push(Binding {
                name: r.name.clone(),
                kind: BindingKind::Resource,
                span: stmt.span,
            }),
            _ => {}
        }
        for body in child_bodies(stmt) {
            hoisted(body, scope);
        }
    }
}

/// The members of a store: its state, derived values and actions.
pub fn store_members(store: &StoreDecl) -> Vec<Binding> {
    let mut members = Vec::new();
    hoisted(&store.body, &mut members);
    members.retain(|b| b.kind != BindingKind::Store);
    members
}

/// The lexer's view of a file: `None` when it does not tokenize. The
/// file's name picks its layout: a `.wfx` is tokenized by indentation.
pub fn tokens_of(file: &SourceFile) -> Option<Vec<Token>> {
    webfluent::syntax::tokens(&file.source, &file.path.to_string_lossy()).ok()
}

/// The index of the token whose extent contains `offset`, or that ends
/// exactly there (the cursor sits after the last character while typing).
pub fn token_index_at(tokens: &[Token], offset: usize) -> Option<usize> {
    tokens
        .iter()
        .position(|t| {
            !matches!(t.token_type, TokenType::EOF) && t.offset <= offset && offset < t.end
        })
        .or_else(|| {
            tokens
                .iter()
                .rposition(|t| t.end == offset && t.offset < t.end)
        })
}

/// The token at `offset`; see [`token_index_at`].
pub fn token_at(tokens: &[Token], offset: usize) -> Option<&Token> {
    token_index_at(tokens, offset).map(|ix| &tokens[ix])
}

/// The store whose member the token at `offset` is: the tokens read
/// `Name . word`, and `Name` is a store in the project. Returns the store's
/// index in the merged program.
pub fn member_owner<'a>(
    project: &'a Project,
    tokens: &[Token],
    offset: usize,
) -> Option<(usize, &'a StoreDecl)> {
    let word_ix = token_index_at(tokens, offset)?;
    if !matches!(
        tokens.get(word_ix.checked_sub(1)?)?.token_type,
        TokenType::Dot
    ) {
        return None;
    }
    let TokenType::Identifier(name) = &tokens.get(word_ix.checked_sub(2)?)?.token_type else {
        return None;
    };
    project
        .program
        .declarations
        .iter()
        .enumerate()
        .find_map(|(ix, d)| match d {
            Declaration::Store(s) if &s.name == name => Some((ix, s)),
            _ => None,
        })
}

/// The last token that ends at or before `offset`.
pub fn token_before(tokens: &[Token], offset: usize) -> Option<&Token> {
    tokens
        .iter()
        .rev()
        .find(|t| !matches!(t.token_type, TokenType::EOF) && t.end <= offset)
}

/// Whether `offset` lies inside a comment: between tokens, in text the lexer
/// skipped. Whitespace between tokens is not a comment; the check reads the
/// gap.
pub fn in_comment(source: &str, tokens: &[Token], offset: usize) -> bool {
    let gap_start = token_before(tokens, offset).map(|t| t.end).unwrap_or(0);
    let gap_end = tokens
        .iter()
        .find(|t| t.offset >= offset)
        .map(|t| t.offset)
        .unwrap_or(source.len());
    let gap = source.get(gap_start..gap_end).unwrap_or("");
    let before = source.get(gap_start..offset).unwrap_or("");
    gap.trim_start().starts_with("//") && before.contains("//")
        || gap.trim_start().starts_with("/*") && before.contains("/*")
}

/// Whether `offset` lies inside a string literal's quotes.
pub fn in_string(tokens: &[Token], offset: usize) -> bool {
    tokens.iter().any(|t| {
        matches!(t.token_type, TokenType::StringLiteral(_)) && t.offset < offset && offset < t.end
    })
}
