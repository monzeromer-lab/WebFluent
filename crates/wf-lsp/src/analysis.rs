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

pub use webfluent::scope::{
    Binding, BindingKind, body_of, child_bodies, contains, decl_span, hoisted, scope_at,
    statement_path, store_members,
};

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
            Declaration::Test(t) if contains(t.span, offset) => return Some(ix),
            Declaration::Type(t) if contains(t.span, offset) => return Some(ix),
            Declaration::Enum(e) if contains(e.span, offset) => return Some(ix),
            Declaration::Api(a) if contains(a.span, offset) => return Some(ix),
            Declaration::Const(c) if contains(c.span, offset) => return Some(ix),
            Declaration::Data(d) if contains(d.span, offset) => return Some(ix),
            Declaration::Animation(a) if contains(a.span, offset) => return Some(ix),
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

/// The lexer's view of a file: `None` when it does not tokenize. The
/// file's name picks its layout: a `.wfx` is tokenized by indentation.
pub fn tokens_of(file: &SourceFile) -> Option<Vec<Token>> {
    // A script is JavaScript and a page in Markdown is Markdown: neither
    // has WebFluent tokens to find a name among.
    if file.script || file.markdown {
        return None;
    }
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

/// Whether `offset` lies inside a `{…}` splice of a string — code, where a
/// name is a name — rather than in the string's text. A raw string
/// (`#"…"#`) has no splices.
pub fn in_splice(source: &str, tokens: &[Token], offset: usize) -> bool {
    let Some(token) = tokens.iter().find(|t| {
        matches!(t.token_type, TokenType::StringLiteral(_)) && t.offset < offset && offset < t.end
    }) else {
        return false;
    };
    let Some(text) = source.get(token.offset..offset) else {
        return false;
    };
    if text.starts_with('#') {
        return false;
    }
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'{' => depth += 1,
            b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        i += 1;
    }
    depth > 0
}

/// Whether `offset` lies inside a string literal's quotes.
pub fn in_string(tokens: &[Token], offset: usize) -> bool {
    tokens.iter().any(|t| {
        matches!(t.token_type, TokenType::StringLiteral(_)) && t.offset < offset && offset < t.end
    })
}
