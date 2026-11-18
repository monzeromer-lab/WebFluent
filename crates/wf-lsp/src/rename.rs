//! Rename, across the files of the project.
//!
//! What is renamed is whatever go-to-definition resolves at the cursor: a
//! component (its declaration, every call, every `layout:`, its parts'
//! owner), a store (`use`, `Store.member`), a store member, a local —
//! state, derived value, action, prop, parameter, loop variable, arm
//! binding — or any other declaration: a `const`, `data`, `image`, `type`,
//! `enum` case, `api` and endpoint, `animation`, or a record's field. Every identifier in the project that resolves to the same
//! definition is changed, inside string interpolations too; a built-in or
//! a browser global has no definition and is not renamed.

use std::collections::HashMap;

use lsp_types::{
    DocumentHighlight, DocumentHighlightKind, GotoDefinitionResponse, Location, Position,
    PrepareRenameResponse, Range, TextEdit, Url, WorkspaceEdit,
};
use webfluent::syntax::lexer::{Token, TokenType};

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
///
/// # Errors
///
/// A message for the editor to show when `new_name` is not a name, when
/// there is no name at `position` or it is not one the project declares,
/// and when the rename would change its case (a declaration's name is
/// capitalised, a value's is not).
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
    for (uri, range) in uses(project, &target, word) {
        changes.entry(uri).or_default().push(TextEdit {
            range,
            new_text: new_name.to_string(),
        });
    }
    for edits in changes.values_mut() {
        edits.sort_by_key(|e| (e.range.start.line, e.range.start.character));
    }
    Ok(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

/// Every place in the project where `word` names what `target` declares —
/// its uses (inside string splices too) and the declaration itself. What
/// rename changes, find-references lists and document-highlight marks.
pub fn uses(project: &Project, target: &Location, word: &str) -> Vec<(Url, Range)> {
    let mut out: Vec<(Url, Range)> = Vec::new();
    for (ix, other) in project.files.iter().enumerate() {
        let src: &str = &other.source;
        // A script is JavaScript, a page in Markdown is Markdown: neither is
        // WebFluent's to read for names.
        // What a script declares is named in scripts too, which only the
        // script's own reading finds.
        if crate::scripts::is_script(project, ix) {
            if crate::scripts::declaration(project, word).as_ref() == Some(target) {
                for (line, col) in webfluent::project_js::scan::uses_of(src, word) {
                    out.push((
                        other.uri.clone(),
                        other.index.word_range_at_line_col(src, line, col),
                    ));
                }
            }
            continue;
        }
        let Some(toks) = analysis::tokens_of(other) else {
            continue;
        };
        let mut ranges: Vec<Range> = Vec::new();
        for occurrence in occurrences(src, &toks, word) {
            let same = definition_at(project, ix, occurrence, &toks)
                .map(single)
                .is_some_and(|d| d == *target);
            if same {
                ranges.push(Range {
                    start: other.index.offset_to_position(src, occurrence),
                    end: other.index.offset_to_position(src, occurrence + word.len()),
                });
            }
        }
        // The declaration itself: the first whole word inside its range.
        if other.uri == target.uri
            && let Some(start) = other.index.position_to_offset(src, target.range.start)
            && let Some(end) = other.index.position_to_offset(src, target.range.end)
            && let Some(at) = whole_word(&src[start..end.min(src.len())], word).map(|i| start + i)
        {
            let range = Range {
                start: other.index.offset_to_position(src, at),
                end: other.index.offset_to_position(src, at + word.len()),
            };
            if !ranges.contains(&range) {
                ranges.push(range);
            }
        }
        ranges.sort_by_key(|r| (r.start.line, r.start.character));
        out.extend(ranges.into_iter().map(|r| (other.uri.clone(), r)));
    }
    out
}

/// The name at `position`, what declares it, and every place it is named:
/// for find-references and document-highlight.
pub fn references(
    project: &Project,
    file_ix: usize,
    position: Position,
    include_declaration: bool,
) -> Vec<Location> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let Some(offset) = file.index.position_to_offset(source, position) else {
        return Vec::new();
    };
    let tokens = analysis::tokens_of(file).unwrap_or_default();
    if analysis::in_comment(source, &tokens, offset)
        || (analysis::in_string(&tokens, offset) && !analysis::in_splice(source, &tokens, offset))
    {
        return Vec::new();
    }
    let Some((word, _)) = word_at(source, offset) else {
        return Vec::new();
    };
    let Some(target) = definition_at(project, file_ix, offset, &tokens).map(single) else {
        return Vec::new();
    };
    uses(project, &target, word)
        .into_iter()
        .filter(|(uri, range)| {
            include_declaration
                || !(uri == &target.uri
                    && range.start >= target.range.start
                    && range.end <= target.range.end)
        })
        .map(|(uri, range)| Location { uri, range })
        .collect()
}

/// The places in this file the name at `position` is named.
pub fn highlights(project: &Project, file_ix: usize, position: Position) -> Vec<DocumentHighlight> {
    let uri = &project.files[file_ix].uri;
    references(project, file_ix, position, true)
        .into_iter()
        .filter(|l| &l.uri == uri)
        .map(|l| DocumentHighlight {
            range: l.range,
            kind: Some(DocumentHighlightKind::TEXT),
        })
        .collect()
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
                let Some(text) = source.get(token.offset..token.end.min(source.len())) else {
                    continue;
                };
                // By character: a string holds any script, and a byte
                // inside one (Arabic, an emoji) is no place to look.
                let mut depth = 0usize;
                let mut skip_to = 0;
                for (i, c) in text.char_indices() {
                    if i < skip_to {
                        continue;
                    }
                    match c {
                        '{' => depth += 1,
                        '}' => depth = depth.saturating_sub(1),
                        _ if depth > 0
                            && text[i..].starts_with(word)
                            && whole_at(text, i, word.len()) =>
                        {
                            out.push(token.offset + i);
                            skip_to = i + word.len();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// A character a name may hold, in any script (`state عددٌ`): a letter, a
/// digit, `_`, or a mark of its letters.
fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || is_mark(c)
}

/// A combining mark — Arabic's short vowels among them — which belongs to
/// the letter before it.
fn is_mark(c: char) -> bool {
    matches!(c, '\u{0300}'..='\u{036F}' | '\u{0610}'..='\u{061A}' | '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{06D6}'..='\u{06ED}')
}

/// Whether `text[at..at + len]` stands as a whole name: no name character
/// on either side. `at` and `at + len` are character boundaries, or the
/// answer is no.
fn whole_at(text: &str, at: usize, len: usize) -> bool {
    let (Some(before), Some(after)) = (text.get(..at), text.get(at + len..)) else {
        return false;
    };
    !before.chars().next_back().is_some_and(is_name_char)
        && !after.chars().next().is_some_and(is_name_char)
}

/// The first whole-word occurrence of `word` in `text`.
fn whole_word(text: &str, word: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(i) = text.get(from..)?.find(word) {
        let at = from + i;
        if whole_at(text, at, word.len()) {
            return Some(at);
        }
        from = at + word.len();
    }
    None
}

pub(crate) fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}
