//! Signature help: inside a call's parentheses, what it takes and which
//! argument the cursor is on — an element's or a component's props, a
//! service's endpoint, an action, a project script's function, or one the
//! language gives.

use tower_lsp::lsp_types::*;
use webfluent::lexer::{Token, TokenType};
use webfluent::parser::ast::*;
use webfluent::registry::{self, PropType};

use crate::analysis;
use crate::project::Project;

/// One parameter of a signature, and what it says about itself.
struct Param {
    label: String,
    name: String,
    doc: Option<String>,
}

pub fn signature_help(
    project: &Project,
    file_ix: usize,
    position: Position,
) -> Option<SignatureHelp> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let offset = file.index.position_to_offset(source, position)?;
    let tokens = analysis::tokens_of(file).unwrap_or_default();
    if analysis::in_comment(source, &tokens, offset)
        || (analysis::in_string(&tokens, offset) && !analysis::in_splice(source, &tokens, offset))
    {
        return None;
    }
    let previous: Vec<&Token> = tokens.iter().filter(|t| t.end <= offset).collect();
    let (open, commas, named) = open_call(&previous)?;
    let callee = callee(&previous, open)?;
    let (label, params, doc) = describe(project, file_ix, offset, &callee)?;
    // The argument the cursor is on: the one named last when it is named,
    // otherwise the one counted by the commas before it.
    let active = match &named {
        Some(name) => params.iter().position(|p| &p.name == name),
        None => Some(commas.min(params.len().saturating_sub(1))),
    }
    .map(|i| i as u32);
    let parameters = params
        .iter()
        .map(|p| ParameterInformation {
            label: ParameterLabel::Simple(p.label.clone()),
            documentation: p.doc.clone().map(Documentation::String),
        })
        .collect();
    Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label,
            documentation: doc.map(Documentation::String),
            parameters: Some(parameters),
            active_parameter: active,
        }],
        active_signature: Some(0),
        active_parameter: active,
    })
}

/// The innermost unclosed `(` before the cursor: its index, how many
/// top-level commas follow it, and the name of the argument being written
/// when it is named (`gap: ‸`).
fn open_call(previous: &[&Token]) -> Option<(usize, usize, Option<String>)> {
    let mut depth = 0i32;
    let mut commas = 0;
    let mut named: Option<String> = None;
    let mut seen_comma = false;
    for (ix, token) in previous.iter().enumerate().rev() {
        match &token.token_type {
            TokenType::CloseParen | TokenType::CloseBracket | TokenType::CloseBrace => depth += 1,
            TokenType::OpenBracket | TokenType::OpenBrace if depth > 0 => depth -= 1,
            TokenType::OpenBrace if depth == 0 => return None,
            TokenType::OpenParen if depth > 0 => depth -= 1,
            TokenType::OpenParen => return Some((ix, commas, named)),
            TokenType::Comma if depth == 0 => {
                commas += 1;
                seen_comma = true;
            }
            // `name:` in the argument being written (after the last comma).
            TokenType::Colon if depth == 0 && !seen_comma && named.is_none() => {
                if let Some(TokenType::Identifier(name)) =
                    ix.checked_sub(1).map(|i| &previous[i].token_type)
                {
                    named = Some(name.clone());
                }
            }
            _ => {}
        }
    }
    None
}

/// What is being called by the `(` at `open`: `Card`, `Card.Header`,
/// `Backend.users`, `save`, `format`.
fn callee(previous: &[&Token], open: usize) -> Option<String> {
    let TokenType::Identifier(name) = &previous.get(open.checked_sub(1)?)?.token_type else {
        return None;
    };
    if let (Some(TokenType::Dot), Some(TokenType::Identifier(owner))) = (
        open.checked_sub(2).map(|i| &previous[i].token_type),
        open.checked_sub(3).map(|i| &previous[i].token_type),
    ) {
        return Some(format!("{owner}.{name}"));
    }
    Some(name.clone())
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
        TypeRef::Refined(inner, _) => type_name(inner),
    }
}

fn prop_params(props: &[PropDecl]) -> Vec<Param> {
    props
        .iter()
        .map(|p| Param {
            label: format!(
                "{}{}: {}{}",
                if p.positional { "_ " } else { "" },
                p.name,
                type_name(&p.prop_type),
                if p.default.is_some() { " = …" } else { "" }
            ),
            name: p.name.clone(),
            doc: p.doc.clone(),
        })
        .collect()
}

fn describe(
    project: &Project,
    file_ix: usize,
    offset: usize,
    callee: &str,
) -> Option<(String, Vec<Param>, Option<String>)> {
    let joined = |name: &str, params: &[Param]| {
        format!(
            "{name}({})",
            params
                .iter()
                .map(|p| p.label.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let declarations = &project.program.declarations;
    // The project's own component, or one of its parts.
    if let Some(c) = declarations.iter().find_map(|d| match d {
        Declaration::Component(c) if c.name == callee => Some(c),
        _ => None,
    }) {
        let params = prop_params(&c.props);
        return Some((joined(callee, &params), params, c.doc.clone()));
    }
    // A service's endpoint.
    if let Some((api, endpoint)) = callee.split_once('.')
        && let Some(e) = declarations.iter().find_map(|d| match d {
            Declaration::Api(a) if a.name == api => a.endpoints.iter().find(|e| e.name == endpoint),
            _ => None,
        })
    {
        let params = prop_params(&e.params);
        let returns = e
            .returns
            .as_ref()
            .map(|t| format!(" -> {}", type_name(t)))
            .unwrap_or_default();
        return Some((
            format!("{}{returns}", joined(callee, &params)),
            params,
            e.doc.clone(),
        ));
    }
    // A built-in element or part: its own props, the positional first.
    let sig = match callee.split_once('.') {
        Some((owner, part)) => registry::part(owner, part),
        None => registry::component(callee),
    };
    if let Some(sig) = sig {
        let mut params: Vec<Param> = Vec::new();
        if let Some(p) = &sig.positional {
            params.push(Param {
                label: p.name.to_string(),
                name: p.name.to_string(),
                doc: Some(p.summary.to_string()),
            });
        }
        for p in sig.props.iter() {
            let ty = match p.ty {
                PropType::Enum(cases) => cases
                    .iter()
                    .filter(|c| !c.name.is_empty())
                    .map(|c| format!(".{}", c.name))
                    .collect::<Vec<_>>()
                    .join(" | "),
                PropType::Bool => "Bool".to_string(),
                _ => "…".to_string(),
            };
            params.push(Param {
                label: format!("{}: {ty}", p.name),
                name: p.name.to_string(),
                doc: Some(p.summary.to_string()),
            });
        }
        return Some((
            joined(callee, &params),
            params,
            Some(sig.summary.to_string()),
        ));
    }
    // An action of the page, the component or a store.
    let action = |a: &ActionDecl| {
        let params: Vec<Param> = a
            .params
            .iter()
            .map(|p| Param {
                label: format!("{}: {}", p.name, type_name(&p.param_type)),
                name: p.name.clone(),
                doc: None,
            })
            .collect();
        (joined(callee, &params), params, None)
    };
    let find_action = |body: &[Statement], name: &str| {
        body.iter().find_map(|s| match &s.kind {
            StatementKind::Action(a) if a.name == name => Some(a.clone()),
            _ => None,
        })
    };
    if let Some((store, name)) = callee.split_once('.')
        && let Some(a) = declarations.iter().find_map(|d| match d {
            Declaration::Store(s) if s.name == store => find_action(&s.body, name),
            _ => None,
        })
    {
        return Some(action(&a));
    }
    if let Some(decl_ix) = analysis::declaration_at(project, file_ix, offset)
        && let Some(a) = find_action(analysis::body_of(&declarations[decl_ix]), callee)
    {
        return Some(action(&a));
    }
    // A project script's function: its JSDoc-typed signature.
    if let Some((_, name)) = project.script_name(callee) {
        let signature = webfluent::project_js::signature(name);
        return Some((signature.clone(), split_params(&signature), None));
    }
    // One the language gives.
    if let Some((_, usage, doc)) = crate::reference::builtin(callee) {
        return Some((
            usage.to_string(),
            split_params(usage),
            Some(doc.to_string()),
        ));
    }
    None
}

/// The parameters written between a signature's parentheses.
fn split_params(signature: &str) -> Vec<Param> {
    let Some(open) = signature.find('(') else {
        return Vec::new();
    };
    let Some(close) = signature.rfind(')') else {
        return Vec::new();
    };
    let inside = &signature[open + 1..close];
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, c) in inside.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&inside[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&inside[start..]);
    out.into_iter()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| Param {
            label: p.to_string(),
            name: p
                .trim_start_matches(['.', '_', ' '])
                .split([':', '?', ' ', '='])
                .next()
                .unwrap_or(p)
                .to_string(),
            doc: None,
        })
        .collect()
}
