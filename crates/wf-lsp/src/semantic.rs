//! Semantic tokens: every name coloured by what it resolves to — a
//! component, a type, an enum's case, a store, a state, a prop or a
//! parameter, an action, a function or value the language gives — which a
//! grammar, reading only the text, cannot know.

use std::collections::HashMap;

use tower_lsp::lsp_types::*;
use webfluent::lexer::TokenType;
use webfluent::parser::ast::*;
use webfluent::registry;

use crate::analysis::{self, BindingKind};
use crate::project::Project;

/// The kinds, in the order the legend lists them.
pub const TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::NAMESPACE,
    SemanticTokenType::TYPE,
    SemanticTokenType::CLASS,
    SemanticTokenType::ENUM,
    SemanticTokenType::ENUM_MEMBER,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::METHOD,
];

/// The modifiers, in the order the legend lists them.
pub const MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::READONLY,
    SemanticTokenModifier::DEFAULT_LIBRARY,
];

pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TYPES.to_vec(),
        token_modifiers: MODIFIERS.to_vec(),
    }
}

const NAMESPACE: u32 = 0;
const TYPE: u32 = 1;
const CLASS: u32 = 2;
const ENUM: u32 = 3;
const ENUM_MEMBER: u32 = 4;
const PARAMETER: u32 = 5;
const VARIABLE: u32 = 6;
const PROPERTY: u32 = 7;
const FUNCTION: u32 = 8;
const METHOD: u32 = 9;

const READONLY: u32 = 1 << 1;
const LIBRARY: u32 = 1 << 2;

/// One classified name: where it is and what it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Classified {
    pub start: Position,
    pub length: u32,
    pub kind: u32,
    pub modifiers: u32,
}

/// What each name of the file is, in order.
pub fn classify(project: &Project, file_ix: usize) -> Vec<Classified> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let Some(tokens) = analysis::tokens_of(file) else {
        return Vec::new();
    };
    // The program's own names, once.
    let mut declared: HashMap<&str, (u32, u32)> = HashMap::new();
    for decl in &project.program.declarations {
        match decl {
            Declaration::Component(c) => {
                declared.insert(&c.name, (CLASS, 0));
            }
            Declaration::Page(p) => {
                declared.insert(&p.name, (CLASS, 0));
            }
            Declaration::Store(s) => {
                declared.insert(&s.name, (NAMESPACE, 0));
            }
            Declaration::Api(a) => {
                declared.insert(&a.name, (NAMESPACE, 0));
            }
            Declaration::Type(t) => {
                declared.insert(&t.name, (TYPE, 0));
            }
            Declaration::Enum(e) => {
                declared.insert(&e.name, (ENUM, 0));
            }
            Declaration::Const(c) => {
                declared.insert(&c.name, (VARIABLE, READONLY));
            }
            Declaration::Data(d) => {
                declared.insert(&d.name, (VARIABLE, READONLY));
            }
            Declaration::Animation(a) => {
                declared.insert(&a.name, (ENUM_MEMBER, 0));
            }
            Declaration::Script(s) => {
                for n in &s.names {
                    let callable = !matches!(n.kind, webfluent::project_js::scan::NameKind::Value);
                    declared.insert(&n.name, (if callable { FUNCTION } else { VARIABLE }, 0));
                }
            }
            _ => {}
        }
    }
    let is_component = |w: &str| {
        registry::component(w).is_some() || declared.get(w).is_some_and(|(k, _)| *k == CLASS)
    };
    let keywords: Vec<&str> = crate::reference::KEYWORDS.iter().map(|k| k.name).collect();
    let functions = webfluent::sema::types::built_in_functions();
    let mut out = Vec::new();
    for (ix, token) in tokens.iter().enumerate() {
        let TokenType::Identifier(word) = &token.token_type else {
            continue;
        };
        let prev = ix.checked_sub(1).map(|i| &tokens[i].token_type);
        let next = tokens.get(ix + 1).map(|t| &t.token_type);
        let kind: Option<(u32, u32)> =
            if matches!(prev, Some(TokenType::Dot | TokenType::OptionalChain)) {
                // After a dot: a store's member, a service's endpoint, a part,
                // a flag or a case, or a field.
                let owner = ix.checked_sub(2).map(|i| &tokens[i].token_type);
                match owner {
                    Some(TokenType::Identifier(o))
                        if declared
                            .get(o.as_str())
                            .is_some_and(|(k, _)| *k == NAMESPACE) =>
                    {
                        Some(if matches!(next, Some(TokenType::OpenParen)) {
                            (METHOD, 0)
                        } else {
                            (PROPERTY, 0)
                        })
                    }
                    Some(TokenType::Identifier(o)) if o.starts_with(char::is_uppercase) => {
                        if word.starts_with(char::is_uppercase) {
                            Some((
                                CLASS,
                                if registry::part(o, word).is_some() {
                                    LIBRARY
                                } else {
                                    0
                                },
                            ))
                        } else {
                            Some((ENUM_MEMBER, 0))
                        }
                    }
                    Some(TokenType::CloseParen | TokenType::Identifier(_))
                        if !matches!(next, Some(TokenType::OpenParen))
                            && is_flag_chain(&tokens, ix, &is_component) =>
                    {
                        Some((ENUM_MEMBER, LIBRARY))
                    }
                    Some(
                        TokenType::Identifier(_) | TokenType::CloseParen | TokenType::CloseBracket,
                    ) => Some(if matches!(next, Some(TokenType::OpenParen)) {
                        (METHOD, 0)
                    } else {
                        (PROPERTY, 0)
                    }),
                    // `.calm`, `animate: .fadeIn`: a case.
                    _ => Some((ENUM_MEMBER, 0)),
                }
            } else if matches!(next, Some(TokenType::Colon))
                && matches!(
                    prev,
                    Some(TokenType::OpenParen | TokenType::Comma | TokenType::OpenBrace)
                )
            {
                // `name:` — a prop, a field, a key.
                Some((PROPERTY, 0))
            } else if keywords.contains(&word.as_str()) && !declared.contains_key(word.as_str()) {
                None
            } else if let Some(found) = declared.get(word.as_str()) {
                Some(*found)
            } else if registry::component(word).is_some() {
                Some((CLASS, LIBRARY))
            } else if webfluent::sema::types::Scalar::of_name(word).is_some()
                || webfluent::sema::types::PRIMITIVE_TYPES.contains(&word.as_str())
            {
                Some((TYPE, LIBRARY))
            } else {
                // A name in scope where it stands.
                let binding =
                    analysis::declaration_at(project, file_ix, token.offset).and_then(|decl_ix| {
                        analysis::scope_at(&project.program.declarations[decl_ix], token.offset)
                            .into_iter()
                            .find(|b| &b.name == word)
                    });
                match binding.map(|b| b.kind) {
                    Some(BindingKind::State) => Some((VARIABLE, 0)),
                    Some(BindingKind::Derived) => Some((VARIABLE, READONLY)),
                    Some(BindingKind::Action) => Some((FUNCTION, 0)),
                    Some(BindingKind::Store) => Some((NAMESPACE, 0)),
                    Some(
                        BindingKind::Prop
                        | BindingKind::Param
                        | BindingKind::RouteParam
                        | BindingKind::LoopItem
                        | BindingKind::LoopIndex
                        | BindingKind::ArmBinding
                        | BindingKind::FetchResult
                        | BindingKind::FetchError,
                    ) => Some((PARAMETER, READONLY)),
                    Some(_) => Some((VARIABLE, 0)),
                    None if functions.contains(&word.as_str()) => Some((FUNCTION, LIBRARY)),
                    None if webfluent::codegen::js::BROWSER_VALUES.contains(&word.as_str()) => {
                        Some((VARIABLE, LIBRARY | READONLY))
                    }
                    None => None,
                }
            };
        if let Some((kind, modifiers)) = kind {
            let start = file.index.offset_to_position(source, token.offset);
            let end = file.index.offset_to_position(source, token.end);
            if end.line == start.line {
                out.push(Classified {
                    start,
                    length: end.character - start.character,
                    kind,
                    modifiers,
                });
            }
        }
    }
    out
}

/// Whether the name at `ix` is a flag: a `.word` after an element's call or
/// its other flags — `Button("x").primary.lg` — rather than a value's field.
fn is_flag_chain(
    tokens: &[webfluent::lexer::Token],
    ix: usize,
    is_component: &dyn Fn(&str) -> bool,
) -> bool {
    // Back over `.word` links to what the chain hangs from.
    let mut i = ix;
    while i >= 3
        && matches!(&tokens[i - 2].token_type, TokenType::Identifier(w) if !w.starts_with(char::is_uppercase))
        && matches!(tokens[i - 3].token_type, TokenType::Dot)
    {
        i -= 2;
    }
    let Some(anchor) = i.checked_sub(2) else {
        return false;
    };
    match &tokens[anchor].token_type {
        TokenType::Identifier(w) => is_component(w),
        TokenType::CloseParen => {
            // The `(` it closes, and the name before that.
            let mut depth = 0i32;
            let mut j = anchor;
            loop {
                match tokens[j].token_type {
                    TokenType::CloseParen => depth += 1,
                    TokenType::OpenParen => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                if j == 0 {
                    return false;
                }
                j -= 1;
            }
            // `Card.Header(…)` and `Text(…)` alike: the name before the `(`.
            matches!(j.checked_sub(1).map(|k| &tokens[k].token_type),
                Some(TokenType::Identifier(w)) if w.starts_with(char::is_uppercase))
        }
        _ => false,
    }
}

/// The file's tokens, encoded as the protocol sends them: each relative to
/// the one before.
pub fn semantic_tokens(project: &Project, file_ix: usize) -> SemanticTokens {
    let mut data = Vec::new();
    let (mut line, mut col) = (0u32, 0u32);
    for c in classify(project, file_ix) {
        let delta_line = c.start.line - line;
        let delta_start = if delta_line == 0 {
            c.start.character - col
        } else {
            c.start.character
        };
        data.push(SemanticToken {
            delta_line,
            delta_start,
            length: c.length,
            token_type: c.kind,
            token_modifiers_bitset: c.modifiers,
        });
        line = c.start.line;
        col = c.start.character;
    }
    SemanticTokens {
        result_id: None,
        data,
    }
}
