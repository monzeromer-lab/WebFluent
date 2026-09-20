//! Rename, across the files of the project.
//!
//! What is renamed is whatever go-to-definition resolves at the cursor: a
//! component (its declaration, every call, every `layout:`, its parts'
//! owner), a store (`use`, `Store.member`), a store member, or a local —
//! state, derived value, action, prop, parameter, loop variable, arm
//! binding. Every identifier in the project that resolves to the same
//! definition is changed, inside string interpolations too; a built-in or
//! a browser global has no definition and is not renamed.

use std::collections::HashMap;

use tower_lsp::lsp_types::*;
use webfluent::lexer::{Token, TokenType};

use crate::analysis;
use crate::definition::definition_at;
use crate::line_index::word_at;
use crate::project::Project;

/// The range of the name at `position` and the name, when it can be
/// renamed.
pub fn prepare_rename(
    project: &Project,
    file_ix: usize,
    position: Position,
) -> Option<PrepareRenameResponse> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let offset = file.index.position_to_offset(source, position)?;
    let tokens = analysis::tokens_of(file).unwrap_or_default();
    if analysis::in_comment(source, &tokens, offset) {
        return None;
    }
    let (word, range) = word_at(source, offset)?;
    definition_at(project, file_ix, offset, &tokens)?;
    Some(PrepareRenameResponse::RangeWithPlaceholder {
        range: Range {
            start: file.index.offset_to_position(source, range.start),
            end: file.index.offset_to_position(source, range.end),
        },
        placeholder: word.to_string(),
    })
}

/// The edits that rename the name at `position` to `new_name` everywhere.
pub fn rename(
    project: &Project,
    file_ix: usize,
    position: Position,
    new_name: &str,
) -> Result<WorkspaceEdit, String> {
    if !is_identifier(new_name) {
        return Err(format!(
            "`{new_name}` is not a name: letters, digits and `_`, not starting with a digit"
        ));
    }
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let Some(offset) = file.index.position_to_offset(source, position) else {
        return Err("Nothing to rename here".to_string());
    };
    let tokens = analysis::tokens_of(file).unwrap_or_default();
    let Some((word, _)) = word_at(source, offset) else {
        return Err("Nothing to rename here".to_string());
    };
    let Some(target) = definition_at(project, file_ix, offset, &tokens) else {
        return Err(format!("`{word}` is not something this project declares"));
    };
    let target = single(target);
    let capitalised = word.starts_with(char::is_uppercase);
    if capitalised != new_name.starts_with(char::is_uppercase) {
        return Err(if capitalised {
            format!("`{word}` is a declaration's name; `{new_name}` should be capitalised too")
        } else {
            format!("`{word}` is a value's name; `{new_name}` should start in lower case")
        });
    }

    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    for (ix, other) in project.files.iter().enumerate() {
        let src: &str = &other.source;
        let Some(toks) = analysis::tokens_of(other) else {
            continue;
        };
        let mut edits: Vec<TextEdit> = Vec::new();
        for occurrence in occurrences(src, &toks, word) {
            let same = definition_at(project, ix, occurrence, &toks)
                .map(single)
                .is_some_and(|d| d == target);
            if !same {
                continue;
            }
            edits.push(TextEdit {
                range: Range {
                    start: other.index.offset_to_position(src, occurrence),
                    end: other.index.offset_to_position(src, occurrence + word.len()),
                },
                new_text: new_name.to_string(),
            });
        }
        // The declaration itself: the first whole word inside its range.
        if other.uri == target.uri
            && let Some(start) = other.index.position_to_offset(src, target.range.start)
            && let Some(end) = other.index.position_to_offset(src, target.range.end)
            && let Some(at) = whole_word(&src[start..end.min(src.len())], word).map(|i| start + i)
            && !edits
                .iter()
                .any(|e| e.range.start == other.index.offset_to_position(src, at))
        {
            edits.push(TextEdit {
                range: Range {
                    start: other.index.offset_to_position(src, at),
                    end: other.index.offset_to_position(src, at + word.len()),
                },
                new_text: new_name.to_string(),
            });
        }
        if !edits.is_empty() {
            edits.sort_by_key(|e| (e.range.start.line, e.range.start.character));
            changes.insert(other.uri.clone(), edits);
        }
    }
    Ok(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

fn single(response: GotoDefinitionResponse) -> Location {
    match response {
        GotoDefinitionResponse::Scalar(l) => l,
        GotoDefinitionResponse::Array(mut ls) => ls.remove(0),
        GotoDefinitionResponse::Link(mut ls) => {
            let l = ls.remove(0);
            Location {
                uri: l.target_uri,
                range: l.target_selection_range,
            }
        }
    }
}

/// Every offset at which `word` stands as a whole identifier: an
/// identifier token, or a word inside a `{…}` splice of a string.
fn occurrences(source: &str, tokens: &[Token], word: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for token in tokens {
        match &token.token_type {
            TokenType::Identifier(name) if name == word => out.push(token.offset),
            TokenType::StringLiteral(_) => {
                let text = &source[token.offset..token.end.min(source.len())];
                let mut depth = 0usize;
                let mut i = 0;
                let bytes = text.as_bytes();
                while i < bytes.len() {
                    match bytes[i] {
                        b'{' => depth += 1,
                        b'}' => depth = depth.saturating_sub(1),
                        _ if depth > 0
                            && text[i..].starts_with(word)
                            && whole_at(text, i, word.len()) =>
                        {
                            out.push(token.offset + i);
                            i += word.len();
                            continue;
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            _ => {}
        }
    }
    out
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn whole_at(text: &str, at: usize, len: usize) -> bool {
    let bytes = text.as_bytes();
    let before = at == 0 || !is_word_byte(bytes[at - 1]);
    let after = at + len >= bytes.len() || !is_word_byte(bytes[at + len]);
    before && after
}

/// The first whole-word occurrence of `word` in `text`.
fn whole_word(text: &str, word: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(i) = text[from..].find(word) {
        let at = from + i;
        if whole_at(text, at, word.len()) {
            return Some(at);
        }
        from = at + word.len();
    }
    None
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}
