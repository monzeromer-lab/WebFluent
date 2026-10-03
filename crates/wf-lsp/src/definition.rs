//! Go to definition, across the files of the project.
//!
//! A component call jumps to the `component` in whichever file declares it;
//! a page's `layout:` to that component; `use CartStore` and
//! `CartStore.total` to the store and its member; a name to the state,
//! derived value, action, prop, parameter, loop variable or arm binding that
//! declares it in the enclosing declaration — the nearest one, not the
//! first one in the file. Every other declaration is reached too: a
//! `const`, `data`, `image`, `type` (in an annotation or a record built),
//! `enum` and each `.case`, `api` and each endpoint (`Backend.users`), an
//! `animation`, a theme, a project script's function, and a record's field
//! (`post.title`, `Post(title: …)`) in whichever type declares it. A
//! string's `{…}` splices are code, and are read as such.

use tower_lsp::lsp_types::*;
use webfluent::lexer::Token;
use webfluent::parser::ast::*;

use crate::analysis::{self, Binding, ElementPart};
use crate::line_index::word_at;
use crate::project::Project;

pub fn find_definition(
    project: &Project,
    file_ix: usize,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let offset = file.index.position_to_offset(source, position)?;
    let tokens = analysis::tokens_of(file).unwrap_or_default();
    // A class in `class: "…"`: its rule, in the stylesheet that defines it.
    if crate::classes::in_class_value(&tokens, offset) {
        return crate::classes::class_word_at(source, offset)
            .and_then(|class| crate::classes::location(project, class))
            .map(GotoDefinitionResponse::Scalar);
    }
    // A string's text names nothing; its `{…}` splices are code.
    if (analysis::in_string(&tokens, offset) && !analysis::in_splice(source, &tokens, offset))
        || analysis::in_comment(source, &tokens, offset)
    {
        return None;
    }
    definition_at(project, file_ix, offset, &tokens)
}

/// The definition of the name at `offset`, a byte offset into the file,
/// whatever surrounds it: what rename reads, inside interpolations too.
pub fn definition_at(
    project: &Project,
    file_ix: usize,
    offset: usize,
    tokens: &[Token],
) -> Option<GotoDefinitionResponse> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let (word, range) = word_at(source, offset)?;
    // A key — `{ count: 2 }`, `Foo(count: 1)` — names nothing in scope.
    let after = source[range.end..].trim_start();
    let before = source[..range.start].trim_end();
    if after.starts_with(':')
        && !after.starts_with("::")
        && (before.ends_with('{') || before.ends_with(',') || before.ends_with('('))
    {
        // …unless it is a field of a record being built: `User(name: "Ada")`.
        return callee_of(source, range.start).and_then(|callee| {
            field_location(project, &webfluent::sema::types::Type::Record(callee), word)
        });
    }

    if let Some(decl_ix) = analysis::declaration_at(project, file_ix, offset) {
        let decl = &project.program.declarations[decl_ix];

        // `layout: Shell` in a page header: the component.
        if let Declaration::Page(page) = decl
            && let Some(layout) = &page.layout
            && analysis::contains(layout.span, offset)
            && layout.name == word
        {
            return declaration_location(project, word, Kind::Component);
        }

        if let Some(el) = analysis::element_at(analysis::body_of(decl), offset) {
            if let Some(ElementPart::Name) = analysis::element_part_at(el, source, offset) {
                return match &el.component {
                    ComponentRef::UserDefined(name) => {
                        declaration_location(project, name, Kind::Component)
                    }
                    _ => None,
                };
            }
            // `Route(page: Home)`: the page.
            if let ComponentRef::BuiltIn(name) = &el.component
                && name == "Route"
            {
                let is_page_arg = el.args.iter().zip(&el.arg_spans).any(|(arg, span)| {
                    matches!(arg, Arg::Named(k, Expr::Identifier(v)) if k == "page" && v == word)
                        && analysis::contains(*span, offset)
                });
                if is_page_arg {
                    return declaration_location(project, word, Kind::Page);
                }
            }
        }

        // `Store.member` — by the tokens, or by the text inside a string's
        // `{…}` splice, where there are none.
        let owner = analysis::member_owner(project, tokens, offset).or_else(|| {
            let (_, range) = word_at(source, offset)?;
            let before = source[..range.start].trim_end();
            let store_name = before.strip_suffix('.').map(|b| {
                b.trim_end()
                    .chars()
                    .rev()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>()
            })?;
            project
                .program
                .declarations
                .iter()
                .enumerate()
                .find_map(|(ix, d)| match d {
                    Declaration::Store(s) if s.name == store_name => Some((ix, s)),
                    _ => None,
                })
        });
        if let Some((store_ix, store)) = owner
            && let Some(member) = analysis::store_members(store)
                .into_iter()
                .find(|m| m.name == word)
        {
            let store_file = project.decl_file[store_ix];
            return Some(location(project, store_file, member.span).into());
        }
        // After a dot and not a store's member: an endpoint of a service,
        // a part of a component (`Owner.Part`), or — with nothing before the
        // dot that owns it — a case of an enum or an animation.
        if let Some((_, range)) = word_at(source, offset)
            && source[..range.start].trim_end().ends_with('.')
        {
            let before = source[..range.start].trim_end();
            // `a?.b` reads through null: the owner is still `a`.
            let before_dot = before[..before.len() - 1].trim_end();
            let before_dot = before_dot.strip_suffix('?').unwrap_or(before_dot);
            let owner_name: String = before_dot
                .chars()
                .rev()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            if let Some(found) = endpoint_location(project, &owner_name, word) {
                return Some(found);
            }
            // `post.title`, for a value the checker types as a record: the
            // field, in whichever type declares it.
            if let Some(binding) = analysis::scope_at(decl, offset)
                .into_iter()
                .find(|b| b.name == owner_name)
                && let Some(found) = crate::hover::type_of_binding(project, decl_ix, &binding)
                    .and_then(|ty| field_location(project, &ty, word))
            {
                return Some(found);
            }
            // `.calm`, `animate: .Wobble`: the dot opens the value. After a
            // `)`, a name or a `]` it is a flag or a member instead.
            let opens_value =
                owner_name.is_empty() && !before_dot.ends_with(')') && !before_dot.ends_with(']');
            if opens_value {
                return case_location(project, word)
                    .or_else(|| declaration_location(project, word, Kind::Animation));
            }
            let qualified = format!("{owner_name}.{word}");
            return declaration_location(project, &qualified, Kind::Component);
        }

        if let Some(binding) = analysis::scope_at(decl, offset)
            .into_iter()
            .find(|b| b.name == word)
        {
            return match binding.kind {
                analysis::BindingKind::Store => declaration_location(project, word, Kind::Store),
                _ => Some(location(project, file_ix, binding_span(&binding)).into()),
            };
        }
    }

    // A name a project script declares: where it is written.
    if let Some((script_ix, name)) = project.script_name(word) {
        let script = &project.files[script_ix];
        return Some(
            Location {
                uri: script.uri.clone(),
                range: script
                    .index
                    .word_range_at_line_col(&script.source, name.line, name.col),
            }
            .into(),
        );
    }

    // A name used where the tree gives no context: any declaration of it.
    declaration_location(project, word, Kind::Any)
}

/// The span to land on: the name of a declaration when the whole statement
/// is known, since the statement may be long.
fn binding_span(binding: &Binding) -> Span {
    binding.span
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Component,
    Page,
    Store,
    Animation,
    Any,
}

/// `Backend.users`: the endpoint `users` of the service `Backend`.
fn endpoint_location(project: &Project, api: &str, name: &str) -> Option<GotoDefinitionResponse> {
    project
        .program
        .declarations
        .iter()
        .enumerate()
        .find_map(|(ix, decl)| match decl {
            Declaration::Api(a) if a.name == api => a
                .endpoints
                .iter()
                .find(|e| e.name == name)
                .map(|e| location(project, project.decl_file[ix], e.span).into()),
            _ => None,
        })
}

/// The name called by the parenthesis the argument at `at` is inside:
/// `User` for `User(id: "1", name‸: …)`.
fn callee_of(source: &str, at: usize) -> Option<String> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut i = at;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b')' | b']' | b'}' => depth += 1,
            b'[' | b'{' if depth == 0 => return None,
            b'(' if depth == 0 => {
                let name: String = source[..i]
                    .chars()
                    .rev()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                return (!name.is_empty()).then_some(name);
            }
            b'(' | b'[' | b'{' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// The field `name` of a value of type `ty`, when `ty` is a record (or may
/// be null and holds one): in the type that declares it, following
/// `extends`.
fn field_location(
    project: &Project,
    ty: &webfluent::sema::types::Type,
    name: &str,
) -> Option<GotoDefinitionResponse> {
    use webfluent::sema::types::Type;
    let mut record = match ty {
        Type::Record(r) => r.clone(),
        Type::Optional(inner) => match inner.as_ref() {
            Type::Record(r) => r.clone(),
            _ => return None,
        },
        _ => return None,
    };
    for _ in 0..16 {
        let (ix, decl) =
            project
                .program
                .declarations
                .iter()
                .enumerate()
                .find_map(|(ix, d)| match d {
                    Declaration::Type(t) if t.name == record => Some((ix, t)),
                    _ => None,
                })?;
        if let Some(field) = decl.fields.iter().find(|f| f.name == name) {
            return Some(location(project, project.decl_file[ix], field.span).into());
        }
        record = decl.extends.clone()?;
    }
    None
}

/// `.calm`: the case of whichever enum declares one by that name — its name
/// inside the enum's declaration.
fn case_location(project: &Project, case: &str) -> Option<GotoDefinitionResponse> {
    project
        .program
        .declarations
        .iter()
        .enumerate()
        .find_map(|(ix, decl)| match decl {
            Declaration::Enum(e) if e.cases.iter().any(|c| c.name == case) => {
                word_in_span(project, project.decl_file[ix], e.span, case, 0)
            }
            _ => None,
        })
}

/// The `nth` (from 0) whole-word occurrence of `word` inside `span` of a
/// file — a name a declaration holds but keeps no span for.
fn word_in_span(
    project: &Project,
    file_ix: usize,
    span: Span,
    word: &str,
    nth: usize,
) -> Option<GotoDefinitionResponse> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let text = source.get(span.start as usize..(span.end as usize).min(source.len()))?;
    let bytes = text.as_bytes();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let at = text
        .match_indices(word)
        .filter(|(i, _)| {
            (*i == 0 || !is_word(bytes[i - 1]))
                && bytes.get(i + word.len()).is_none_or(|b| !is_word(*b))
        })
        .nth(nth)?
        .0;
    let start = span.start as usize + at;
    Some(
        Location {
            uri: file.uri.clone(),
            range: Range {
                start: file.index.offset_to_position(source, start),
                end: file.index.offset_to_position(source, start + word.len()),
            },
        }
        .into(),
    )
}

fn declaration_location(
    project: &Project,
    name: &str,
    kind: Kind,
) -> Option<GotoDefinitionResponse> {
    let (ix, span) = project
        .program
        .declarations
        .iter()
        .enumerate()
        .find_map(|(ix, decl)| match decl {
            Declaration::Component(c)
                if c.name == name && matches!(kind, Kind::Component | Kind::Any) =>
            {
                Some((ix, c.header_span))
            }
            Declaration::Page(p) if p.name == name && matches!(kind, Kind::Page | Kind::Any) => {
                Some((ix, p.header_span))
            }
            Declaration::Store(s) if s.name == name && matches!(kind, Kind::Store | Kind::Any) => {
                Some((ix, s.header_span))
            }
            Declaration::Theme(t) if t.name == name && kind == Kind::Any => Some((ix, t.span)),
            Declaration::Animation(a)
                if a.name == name && matches!(kind, Kind::Animation | Kind::Any) =>
            {
                Some((ix, a.span))
            }
            Declaration::Type(t) if t.name == name && kind == Kind::Any => {
                Some((ix, t.header_span))
            }
            Declaration::Enum(e) if e.name == name && kind == Kind::Any => {
                Some((ix, e.header_span))
            }
            Declaration::Const(c) if c.name == name && kind == Kind::Any => Some((ix, c.span)),
            Declaration::Data(d) if d.name == name && kind == Kind::Any => Some((ix, d.span)),
            Declaration::Api(a) if a.name == name && kind == Kind::Any => Some((ix, a.span)),
            _ => None,
        })?;
    Some(location(project, project.decl_file[ix], span).into())
}

fn location(project: &Project, file_ix: usize, span: Span) -> Location {
    let file = &project.files[file_ix];
    Location {
        uri: file.uri.clone(),
        range: file.index.span_to_range(&file.source, span),
    }
}
