//! Go to definition, across the files of the project.
//!
//! A component call jumps to the `component` in whichever file declares it;
//! a page's `layout:` to that component; `use CartStore` and
//! `CartStore.total` to the store and its member; a name to the state,
//! derived value, action, prop, parameter, loop variable or arm binding that
//! declares it in the enclosing declaration — the nearest one, not the
//! first one in the file.

use tower_lsp::lsp_types::*;
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
    if analysis::in_string(&tokens, offset) || analysis::in_comment(source, &tokens, offset) {
        return None;
    }
    let (word, _) = word_at(source, offset)?;

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

        // `Store.member`
        if let Some((store_ix, store)) = analysis::member_owner(project, &tokens, offset)
            && let Some(member) = analysis::store_members(store)
                .into_iter()
                .find(|m| m.name == word)
        {
            let store_file = project.decl_file[store_ix];
            return Some(location(project, store_file, member.span).into());
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
    Any,
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
