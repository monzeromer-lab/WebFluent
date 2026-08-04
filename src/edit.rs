//! Structured edit engine (Slice 3 of the studio engine upgrade).
//!
//! The AST is for *addressing*; the mutation is a **text patch on a span**, so
//! human-authored formatting outside the edit is preserved byte-for-byte and we
//! avoid a full AST→source pretty-printer.
//!
//! [`apply_edits`] takes source and a batch of typed [`EditOp`]s (the AI's tool
//! schema — serde-serializable), resolves each op's target node by the same id
//! the studio shows (`Home:2.0.3`, via [`crate::codegen::node_id::visit_nodes`]),
//! turns it into a `(byte range, replacement)` patch using the node's Slice-1
//! sub-spans, applies the patches right-to-left, and **reparses the result** so a
//! bad edit can never corrupt the file — it returns an error and the caller keeps
//! the original source.
//!
//! This first cut implements the text-replacement / whole-node / append ops. The
//! remaining ops (`AddModifier`/`RemoveModifier`, `InsertChild` at an index,
//! `SetStyle`/`RemoveStyle`, `MoveNode`) build on the same machinery and land next.

use crate::codegen::node_id;
use crate::error::{Result, WebFluentError};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::parser::ast::{Arg, Expr, Program, Span, UIElement};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Which argument of a node an op targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgRef {
    /// The n-th positional argument (0-based, counting only positional args).
    Positional(usize),
    /// A named argument (`name: value`).
    Named(String),
}

/// A structured edit targeting a node by its studio id (e.g. `"Home:2.0.3"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EditOp {
    /// Replace the node's first positional string argument with `value` (quoted).
    SetText { node: String, value: String },
    /// Replace an argument's value with `value` (raw `.wf` expression text).
    SetArg {
        node: String,
        arg: ArgRef,
        value: String,
    },
    /// Add a modifier (e.g. `primary`, `large`) to the node.
    AddModifier { node: String, modifier: String },
    /// Remove a modifier from the node.
    RemoveModifier { node: String, modifier: String },
    /// Set a style property (`prop: value`) on the node, creating the `style { }`
    /// block if needed.
    SetStyle {
        node: String,
        prop: String,
        value: String,
    },
    /// Remove a style property from the node's `style { }` block.
    RemoveStyle { node: String, prop: String },
    /// Insert the `wf` snippet as the node's child at `index` (appends if out of range).
    InsertChild {
        node: String,
        index: usize,
        wf: String,
    },
    /// Append the `wf` snippet as the last child of the node's block.
    AppendChild { node: String, wf: String },
    /// Replace the whole node with the `wf` snippet.
    ReplaceNode { node: String, wf: String },
    /// Remove the whole node (and its line).
    RemoveNode { node: String },
    /// Move the node to be the child at `index` of `new_parent`.
    MoveNode {
        node: String,
        new_parent: String,
        index: usize,
    },
}

impl EditOp {
    /// The id of the node this op targets.
    fn node(&self) -> &str {
        match self {
            EditOp::SetText { node, .. }
            | EditOp::SetArg { node, .. }
            | EditOp::AddModifier { node, .. }
            | EditOp::RemoveModifier { node, .. }
            | EditOp::SetStyle { node, .. }
            | EditOp::RemoveStyle { node, .. }
            | EditOp::InsertChild { node, .. }
            | EditOp::AppendChild { node, .. }
            | EditOp::ReplaceNode { node, .. }
            | EditOp::RemoveNode { node }
            | EditOp::MoveNode { node, .. } => node,
        }
    }
}

/// Apply a batch of structured edits to `.wf` source, returning the new source.
///
/// Every op is resolved and validated before any bytes change; the patched result
/// is reparsed as a final guard. On any error the original `source` is untouched
/// (the caller still holds it) — a malformed edit never corrupts the file.
pub fn apply_edits(source: &str, ops: &[EditOp]) -> Result<String> {
    let program = parse_program(source)?;

    // id -> element, using the SAME traversal the codegen ids come from.
    let mut index: HashMap<String, &UIElement> = HashMap::new();
    node_id::visit_nodes(&program, &mut |ui, id, _component| {
        index.insert(id.to_string(), ui);
    });

    let mut patches: Vec<Patch> = Vec::new();
    for op in ops {
        let ui = index
            .get(op.node())
            .ok_or_else(|| edit_err(format!("unknown node id '{}'", op.node())))?;
        patches.extend(compute_patches(op, ui, source, &index)?);
    }

    let result = apply_patches(source, patches)?;

    // Final guard: the edited source must still parse.
    parse_program(&result)
        .map_err(|e| edit_err(format!("edits produced invalid source: {}", e)))?;
    Ok(result)
}

// ─── Patch model ─────────────────────────────────────────

/// A single byte-range replacement.
struct Patch {
    start: usize,
    end: usize,
    text: String,
}

fn compute_patches(
    op: &EditOp,
    ui: &UIElement,
    source: &str,
    index: &HashMap<String, &UIElement>,
) -> Result<Vec<Patch>> {
    let one = |p: Patch| Ok(vec![p]);
    match op {
        EditOp::SetText { value, .. } => {
            let k = first_positional_string(ui)
                .ok_or_else(|| edit_err("set_text: node has no positional string argument"))?;
            one(replace(ui.arg_spans[k], quote_string(value)))
        }
        EditOp::SetArg { arg, value, .. } => match arg {
            ArgRef::Positional(i) => {
                let k = nth_positional(ui, *i).ok_or_else(|| {
                    edit_err(format!("set_arg: node has no positional argument #{}", i))
                })?;
                one(replace(ui.arg_spans[k], value.clone()))
            }
            ArgRef::Named(name) => {
                let k = named_arg_index(ui, name).ok_or_else(|| {
                    edit_err(format!("set_arg: node has no named argument '{}'", name))
                })?;
                one(replace(ui.arg_spans[k], format!("{}: {}", name, value)))
            }
        },
        EditOp::AddModifier { modifier, .. } => {
            if ui.modifiers.iter().any(|m| m == modifier) {
                return Ok(vec![]); // already present — no-op
            }
            if let Some(paren) = ui.paren_span {
                let close = paren.end as usize - 1; // the ')'
                let empty = ui.args.is_empty() && ui.modifiers.is_empty();
                let text = if empty {
                    modifier.clone()
                } else {
                    format!(", {}", modifier)
                };
                one(Patch {
                    start: close,
                    end: close,
                    text,
                })
            } else {
                // No `( … )` yet — add one right after the component name.
                let at = ident_end(source, ui.span.start as usize);
                one(Patch {
                    start: at,
                    end: at,
                    text: format!("({})", modifier),
                })
            }
        }
        EditOp::RemoveModifier { modifier, .. } => {
            let i = ui
                .modifiers
                .iter()
                .position(|m| m == modifier)
                .ok_or_else(|| {
                    edit_err(format!(
                        "remove_modifier: node has no modifier '{}'",
                        modifier
                    ))
                })?;
            let (start, end) = modifier_removal_range(source, ui.modifier_spans[i]);
            one(Patch {
                start,
                end,
                text: String::new(),
            })
        }
        EditOp::SetStyle { prop, value, .. } => {
            if let Some(sb) = &ui.style_block {
                if let Some(p) = sb.properties.iter().find(|p| &p.name == prop) {
                    one(replace(p.value_span, value.clone()))
                } else {
                    let indent = match sb.properties.last() {
                        Some(last) => " ".repeat(last.span.col.saturating_sub(1) as usize),
                        None => format!("{}  ", line_indent(source, sb.body_span.end as usize)),
                    };
                    let at = sb.body_span.end as usize;
                    one(Patch {
                        start: at,
                        end: at,
                        text: format!("{}{}: {}\n", indent, prop, value),
                    })
                }
            } else {
                // No style block — create one inside the element body.
                one(append_child_patch(
                    ui,
                    &format!("style {{ {}: {} }}", prop, value),
                    source,
                ))
            }
        }
        EditOp::RemoveStyle { prop, .. } => {
            let sb = ui
                .style_block
                .as_ref()
                .ok_or_else(|| edit_err("remove_style: node has no style block"))?;
            let p = sb
                .properties
                .iter()
                .find(|p| &p.name == prop)
                .ok_or_else(|| {
                    edit_err(format!(
                        "remove_style: node has no style property '{}'",
                        prop
                    ))
                })?;
            let (start, end) = removal_range(source, p.span);
            one(Patch {
                start,
                end,
                text: String::new(),
            })
        }
        EditOp::InsertChild { index: idx, wf, .. } => {
            validate_snippet(wf)?;
            one(insert_child_patch(ui, *idx, wf, source))
        }
        EditOp::AppendChild { wf, .. } => {
            validate_snippet(wf)?;
            one(append_child_patch(ui, wf, source))
        }
        EditOp::ReplaceNode { wf, .. } => {
            validate_snippet(wf)?;
            one(replace(ui.span, wf.clone()))
        }
        EditOp::RemoveNode { .. } => {
            let (start, end) = removal_range(source, ui.span);
            one(Patch {
                start,
                end,
                text: String::new(),
            })
        }
        EditOp::MoveNode {
            new_parent,
            index: idx,
            ..
        } => {
            let parent = index.get(new_parent.as_str()).ok_or_else(|| {
                edit_err(format!("move_node: unknown new_parent '{}'", new_parent))
            })?;
            let node_text = ui.span.slice(source).to_string();
            let (rm_start, rm_end) = removal_range(source, ui.span);
            let insert = insert_child_patch(parent, *idx, &node_text, source);
            Ok(vec![
                Patch {
                    start: rm_start,
                    end: rm_end,
                    text: String::new(),
                },
                insert,
            ])
        }
    }
}

fn replace(span: Span, text: String) -> Patch {
    Patch {
        start: span.start as usize,
        end: span.end as usize,
        text,
    }
}

/// A patch that appends `wf` as the last child of `ui`'s block (creating a block
/// if there is none).
fn append_child_patch(ui: &UIElement, wf: &str, source: &str) -> Patch {
    if let Some(body) = ui.body_span {
        let indent = match ui.children.last() {
            Some(last) => " ".repeat(last.span.col.saturating_sub(1) as usize),
            None => format!("{}  ", line_indent(source, body.end as usize)),
        };
        let at = body.end as usize;
        Patch {
            start: at,
            end: at,
            text: format!("{}{}\n", indent, wf),
        }
    } else {
        let at = ui.span.end as usize;
        Patch {
            start: at,
            end: at,
            text: format!(" {{ {} }}", wf),
        }
    }
}

/// A patch that inserts `wf` as `ui`'s child at `index` (appends if out of range).
fn insert_child_patch(ui: &UIElement, index: usize, wf: &str, source: &str) -> Patch {
    match ui.children.get(index) {
        Some(child) => {
            let at = child.span.start as usize;
            let indent = line_indent(source, at);
            Patch {
                start: at,
                end: at,
                text: format!("{}\n{}", wf, indent),
            }
        }
        None => append_child_patch(ui, wf, source),
    }
}

/// Apply patches right-to-left so earlier offsets stay valid; reject overlaps.
fn apply_patches(source: &str, mut patches: Vec<Patch>) -> Result<String> {
    patches.sort_by_key(|p| std::cmp::Reverse(p.start));
    for w in patches.windows(2) {
        let (later, earlier) = (&w[0], &w[1]); // later has the greater start
        if earlier.end > later.start {
            return Err(edit_err("overlapping edits target the same source region"));
        }
    }
    let mut result = source.to_string();
    for p in &patches {
        result.replace_range(p.start..p.end, &p.text);
    }
    Ok(result)
}

// ─── Argument lookup ─────────────────────────────────────

/// Array index of the first positional argument that is a string.
fn first_positional_string(ui: &UIElement) -> Option<usize> {
    ui.args.iter().position(|a| {
        matches!(a, Arg::Positional(e) if matches!(e, Expr::StringLiteral(_) | Expr::InterpolatedString(_)))
    })
}

/// Array index of the `n`-th positional argument (0-based among positionals).
fn nth_positional(ui: &UIElement, n: usize) -> Option<usize> {
    let mut count = 0;
    for (k, arg) in ui.args.iter().enumerate() {
        if let Arg::Positional(_) = arg {
            if count == n {
                return Some(k);
            }
            count += 1;
        }
    }
    None
}

/// Array index of the named argument `name`.
fn named_arg_index(ui: &UIElement, name: &str) -> Option<usize> {
    ui.args
        .iter()
        .position(|a| matches!(a, Arg::Named(n, _) if n == name))
}

// ─── Text helpers ────────────────────────────────────────

/// Quote and escape a string for `.wf` source.
fn quote_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// The byte offset just past the leading identifier (component name, incl. `.`
/// for sub-components) starting at `start`.
fn ident_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphanumeric() || b == b'_' || b == b'.' {
            i += 1;
        } else {
            break;
        }
    }
    i
}

/// The byte range to delete when removing one modifier: its span extended to
/// swallow the adjacent comma (trailing preferred, else leading), so the argument
/// list stays well-formed.
fn modifier_removal_range(source: &str, span: Span) -> (usize, usize) {
    let bytes = source.as_bytes();
    let start = span.start as usize;
    let end = span.end as usize;

    // Trailing ", " ?
    let mut e = end;
    while e < bytes.len() && bytes[e] == b' ' {
        e += 1;
    }
    if e < bytes.len() && bytes[e] == b',' {
        e += 1;
        while e < bytes.len() && bytes[e] == b' ' {
            e += 1;
        }
        return (start, e);
    }
    // Otherwise leading ", " ?
    let mut s = start;
    while s > 0 && bytes[s - 1] == b' ' {
        s -= 1;
    }
    if s > 0 && bytes[s - 1] == b',' {
        s -= 1;
        while s > 0 && bytes[s - 1] == b' ' {
            s -= 1;
        }
        return (s, end);
    }
    (start, end)
}

/// The leading whitespace of the line containing byte offset `pos`.
fn line_indent(source: &str, pos: usize) -> String {
    let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    source[line_start..pos]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

/// The byte range to delete for a whole-node removal: the node's span extended to
/// swallow its line's leading indentation and trailing newline, so no blank line
/// is left behind.
fn removal_range(source: &str, span: Span) -> (usize, usize) {
    let start = span.start as usize;
    let end = span.end as usize;
    let bytes = source.as_bytes();

    let line_start = source[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let lead_is_ws = source[line_start..start]
        .bytes()
        .all(|b| b == b' ' || b == b'\t');
    let new_start = if lead_is_ws { line_start } else { start };

    let mut new_end = end;
    while new_end < bytes.len() && (bytes[new_end] == b' ' || bytes[new_end] == b'\t') {
        new_end += 1;
    }
    if new_end < bytes.len() && bytes[new_end] == b'\n' {
        new_end += 1;
    }
    (new_start, new_end)
}

// ─── Parsing ─────────────────────────────────────────────

fn parse_program(source: &str) -> Result<Program> {
    let tokens = Lexer::new(source, "<edit>").tokenize()?;
    Parser::new(tokens, "<edit>").parse()
}

/// Validate a `.wf` child snippet by parsing it inside a throwaway page. A bad
/// snippet is rejected here, before any bytes are patched.
fn validate_snippet(wf: &str) -> Result<()> {
    let wrapped = format!("Page __EditValidate (path: \"/\") {{\n{}\n}}", wf);
    parse_program(&wrapped)
        .map(|_| ())
        .map_err(|e| edit_err(format!("invalid wf snippet: {}", e)))
}

fn edit_err(msg: impl Into<String>) -> WebFluentError {
    WebFluentError::EditError(msg.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The id of the first element whose source text starts with `needle`.
    fn id_of(source: &str, needle: &str) -> String {
        let program = parse_program(source).expect("parse");
        let map = node_id::build_node_map(&program);
        map.iter()
            .find(|(_, info)| info.span.slice(source).starts_with(needle))
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| panic!("no node starting with {:?}", needle))
    }

    const SRC: &str = "Page Home (path: \"/\") {\n\
                       \x20 Container {\n\
                       \x20   Heading(\"Welcome\", h1)\n\
                       \x20   Image(src: \"/a.png\", alt: \"a\")\n\
                       \x20   Text(\"Body\")\n\
                       \x20 }\n\
                       }\n";

    #[test]
    fn set_text_replaces_label_and_leaves_rest_identical() {
        let id = id_of(SRC, "Heading");
        let out = apply_edits(
            SRC,
            &[EditOp::SetText {
                node: id,
                value: "Goodbye".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Heading(\"Goodbye\", h1)"));
        assert!(!out.contains("Welcome"));
        // Everything except the one replaced token is byte-identical.
        assert_eq!(out.replace("Goodbye", "Welcome"), SRC);
        // And it still parses.
        parse_program(&out).unwrap();
    }

    #[test]
    fn set_arg_named_replaces_value() {
        let id = id_of(SRC, "Image");
        let out = apply_edits(
            SRC,
            &[EditOp::SetArg {
                node: id,
                arg: ArgRef::Named("src".into()),
                value: "\"/b.png\"".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Image(src: \"/b.png\", alt: \"a\")"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn set_arg_positional_replaces_expr() {
        let id = id_of(SRC, "Heading");
        let out = apply_edits(
            SRC,
            &[EditOp::SetArg {
                node: id,
                arg: ArgRef::Positional(0),
                value: "\"Hi\"".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Heading(\"Hi\", h1)"));
    }

    #[test]
    fn replace_node_swaps_whole_element() {
        let id = id_of(SRC, "Text(\"Body\")");
        let out = apply_edits(
            SRC,
            &[EditOp::ReplaceNode {
                node: id,
                wf: "Button(\"Click\", primary)".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Button(\"Click\", primary)"));
        assert!(!out.contains("Text(\"Body\")"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn remove_node_deletes_line_cleanly() {
        let id = id_of(SRC, "Text(\"Body\")");
        let out = apply_edits(SRC, &[EditOp::RemoveNode { node: id }]).unwrap();
        assert!(!out.contains("Text(\"Body\")"));
        assert!(!out.contains("\n\n")); // no blank line left behind
        parse_program(&out).unwrap();
    }

    #[test]
    fn append_child_adds_into_block() {
        let id = id_of(SRC, "Container");
        let out = apply_edits(
            SRC,
            &[EditOp::AppendChild {
                node: id,
                wf: "Button(\"New\")".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Button(\"New\")"));
        // Inserted after the existing children, still inside Container.
        let container = &out[out.find("Container").unwrap()..];
        assert!(container.find("Button(\"New\")").unwrap() < container.find('}').unwrap());
        parse_program(&out).unwrap();
    }

    #[test]
    fn malformed_snippet_is_rejected_without_touching_source() {
        let id = id_of(SRC, "Container");
        let err = apply_edits(
            SRC,
            &[EditOp::AppendChild {
                node: id,
                wf: "Button(\"oops\"".into(), // missing ')'
            }],
        );
        assert!(err.is_err());
    }

    #[test]
    fn unknown_node_is_rejected() {
        let err = apply_edits(
            SRC,
            &[EditOp::RemoveNode {
                node: "Nope:9".into(),
            }],
        );
        assert!(err.is_err());
    }

    #[test]
    fn batch_of_ops_applies_and_recompiles() {
        let heading = id_of(SRC, "Heading");
        let text = id_of(SRC, "Text(\"Body\")");
        let out = apply_edits(
            SRC,
            &[
                EditOp::SetText {
                    node: heading,
                    value: "Hi".into(),
                },
                EditOp::RemoveNode { node: text },
            ],
        )
        .unwrap();
        assert!(out.contains("Heading(\"Hi\", h1)"));
        assert!(!out.contains("Text(\"Body\")"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn add_modifier_appends_into_parens() {
        let src = "Page P (path: \"/\") {\n  Button(\"Go\", primary)\n}\n";
        let id = id_of(src, "Button");
        let out = apply_edits(
            src,
            &[EditOp::AddModifier {
                node: id,
                modifier: "large".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Button(\"Go\", primary, large)"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn add_modifier_is_idempotent() {
        let src = "Page P (path: \"/\") {\n  Button(\"Go\", primary)\n}\n";
        let id = id_of(src, "Button");
        let out = apply_edits(
            src,
            &[EditOp::AddModifier {
                node: id,
                modifier: "primary".into(),
            }],
        )
        .unwrap();
        assert_eq!(out, src); // already present → no change
    }

    #[test]
    fn add_modifier_creates_parens_when_none() {
        let src = "Page P (path: \"/\") {\n  Divider\n}\n";
        let id = id_of(src, "Divider");
        let out = apply_edits(
            src,
            &[EditOp::AddModifier {
                node: id,
                modifier: "large".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Divider(large)"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn remove_modifier_handles_comma_middle_and_last() {
        let src = "Page P (path: \"/\") {\n  Button(\"Go\", primary, large)\n}\n";
        // middle/first modifier
        let out = apply_edits(
            src,
            &[EditOp::RemoveModifier {
                node: id_of(src, "Button"),
                modifier: "primary".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Button(\"Go\", large)"), "got: {}", out);
        parse_program(&out).unwrap();
        // last modifier
        let out2 = apply_edits(
            src,
            &[EditOp::RemoveModifier {
                node: id_of(src, "Button"),
                modifier: "large".into(),
            }],
        )
        .unwrap();
        assert!(out2.contains("Button(\"Go\", primary)"), "got: {}", out2);
        parse_program(&out2).unwrap();
    }

    const STYLE_SRC: &str = "Page P (path: \"/\") {\n\
                             \x20 Card {\n\
                             \x20   style {\n\
                             \x20     color: red\n\
                             \x20   }\n\
                             \x20 }\n\
                             }\n";

    #[test]
    fn set_style_changes_existing_value() {
        let id = id_of(STYLE_SRC, "Card");
        let out = apply_edits(
            STYLE_SRC,
            &[EditOp::SetStyle {
                node: id,
                prop: "color".into(),
                value: "blue".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("color: blue"));
        assert!(!out.contains("color: red"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn set_style_adds_new_property() {
        let id = id_of(STYLE_SRC, "Card");
        // CSS values with units are quoted strings in WebFluent.
        let out = apply_edits(
            STYLE_SRC,
            &[EditOp::SetStyle {
                node: id,
                prop: "padding".into(),
                value: "\"10px\"".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("color: red"));
        assert!(out.contains("padding: \"10px\""));
        parse_program(&out).unwrap();
    }

    #[test]
    fn set_style_creates_block_when_none() {
        let src = "Page P (path: \"/\") {\n  Text(\"Hi\")\n}\n";
        let id = id_of(src, "Text");
        let out = apply_edits(
            src,
            &[EditOp::SetStyle {
                node: id,
                prop: "color".into(),
                value: "red".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("style"));
        assert!(out.contains("color: red"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn remove_style_deletes_property() {
        let src = "Page P (path: \"/\") {\n\
                   \x20 Card {\n\
                   \x20   style {\n\
                   \x20     color: red\n\
                   \x20     padding: \"10px\"\n\
                   \x20   }\n\
                   \x20 }\n\
                   }\n";
        let id = id_of(src, "Card");
        let out = apply_edits(
            src,
            &[EditOp::RemoveStyle {
                node: id,
                prop: "color".into(),
            }],
        )
        .unwrap();
        assert!(!out.contains("color: red"));
        assert!(out.contains("padding: \"10px\""));
        parse_program(&out).unwrap();
    }

    #[test]
    fn insert_child_at_index() {
        let src =
            "Page P (path: \"/\") {\n  Container {\n    Text(\"a\")\n    Text(\"b\")\n  }\n}\n";
        let id = id_of(src, "Container");
        let out = apply_edits(
            src,
            &[EditOp::InsertChild {
                node: id,
                index: 1,
                wf: "Button(\"x\")".into(),
            }],
        )
        .unwrap();
        let a = out.find("Text(\"a\")").unwrap();
        let btn = out.find("Button(\"x\")").unwrap();
        let b = out.find("Text(\"b\")").unwrap();
        assert!(a < btn && btn < b, "order wrong: {}", out);
        parse_program(&out).unwrap();
    }

    #[test]
    fn move_node_relocates_element() {
        let src = "Page P (path: \"/\") {\n\
                   \x20 Row {\n\
                   \x20   Text(\"item\")\n\
                   \x20 }\n\
                   \x20 Column {\n\
                   \x20 }\n\
                   }\n";
        let text_id = id_of(src, "Text(\"item\")");
        let col_id = id_of(src, "Column");
        let out = apply_edits(
            src,
            &[EditOp::MoveNode {
                node: text_id,
                new_parent: col_id,
                index: 0,
            }],
        )
        .unwrap();
        parse_program(&out).unwrap();
        // Text is now under Column, and gone from the Row block.
        let col = &out[out.find("Column").unwrap()..];
        assert!(col.contains("Text(\"item\")"));
        let row_block = &out[out.find("Row").unwrap()..out.find("Column").unwrap()];
        assert!(
            !row_block.contains("Text(\"item\")"),
            "still in row: {}",
            out
        );
    }

    #[test]
    fn ops_serialize_as_tagged_json() {
        let op = EditOp::SetText {
            node: "Home:0".into(),
            value: "Hi".into(),
        };
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("\"op\":\"set_text\""));
        let back: EditOp = serde_json::from_str(&json).unwrap();
        assert_eq!(op, back);
    }

    // ─── Adversarial / hardening: never panic, never corrupt ─────────

    #[test]
    fn multibyte_set_text_no_panic() {
        let src = "Page P (path: \"/\") {\n  Text(\"café ☕\")\n}\n";
        let id = id_of(src, "Text");
        let out = apply_edits(
            src,
            &[EditOp::SetText {
                node: id,
                value: "münchen 🍺".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Text(\"münchen 🍺\")"), "got: {}", out);
        parse_program(&out).unwrap();
    }

    #[test]
    fn multibyte_remove_sibling_no_panic() {
        let src = "Page P (path: \"/\") {\n  Text(\"café ☕ 日本語\")\n  Button(\"x\")\n}\n";
        let id = id_of(src, "Button");
        let out = apply_edits(src, &[EditOp::RemoveNode { node: id }]).unwrap();
        assert!(out.contains("café ☕ 日本語"));
        assert!(!out.contains("Button"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn multibyte_add_modifier_positions_correctly() {
        let src = "Page P (path: \"/\") {\n  Button(\"héllo café 🎉\", primary)\n}\n";
        let id = id_of(src, "Button");
        let out = apply_edits(
            src,
            &[EditOp::AddModifier {
                node: id,
                modifier: "large".into(),
            }],
        )
        .unwrap();
        assert!(
            out.contains("Button(\"héllo café 🎉\", primary, large)"),
            "got: {}",
            out
        );
        parse_program(&out).unwrap();
    }

    #[test]
    fn add_modifier_into_empty_parens() {
        let src = "Page P (path: \"/\") {\n  Button()\n}\n";
        let id = id_of(src, "Button");
        let out = apply_edits(
            src,
            &[EditOp::AddModifier {
                node: id,
                modifier: "large".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Button(large)"), "got: {}", out);
        parse_program(&out).unwrap();
    }

    #[test]
    fn add_modifier_after_named_arg() {
        let src = "Page P (path: \"/\") {\n  Image(src: \"/a.png\")\n}\n";
        let id = id_of(src, "Image");
        let out = apply_edits(
            src,
            &[EditOp::AddModifier {
                node: id,
                modifier: "rounded".into(),
            }],
        )
        .unwrap();
        assert!(
            out.contains("Image(src: \"/a.png\", rounded)"),
            "got: {}",
            out
        );
        parse_program(&out).unwrap();
    }

    #[test]
    fn remove_only_modifier_stays_valid() {
        let src = "Page P (path: \"/\") {\n  Button(primary)\n}\n";
        let id = id_of(src, "Button");
        let out = apply_edits(
            src,
            &[EditOp::RemoveModifier {
                node: id,
                modifier: "primary".into(),
            }],
        )
        .unwrap();
        assert!(!out.contains("primary"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn append_child_into_single_line_and_empty_body() {
        for src in [
            "Page P (path: \"/\") {\n  Container { Text(\"a\") }\n}\n",
            "Page P (path: \"/\") {\n  Container {}\n}\n",
        ] {
            let id = id_of(src, "Container");
            let out = apply_edits(
                src,
                &[EditOp::AppendChild {
                    node: id,
                    wf: "Button(\"b\")".into(),
                }],
            )
            .unwrap();
            assert!(out.contains("Button(\"b\")"), "got: {}", out);
            parse_program(&out).unwrap();
        }
    }

    #[test]
    fn insert_child_index_zero_into_empty() {
        let src = "Page P (path: \"/\") {\n  Container {}\n}\n";
        let id = id_of(src, "Container");
        let out = apply_edits(
            src,
            &[EditOp::InsertChild {
                node: id,
                index: 0,
                wf: "Text(\"x\")".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Text(\"x\")"));
        parse_program(&out).unwrap();
    }

    #[test]
    fn set_arg_positional_skips_named() {
        let src = "Page P (path: \"/\") {\n  Text(\"hi\", weight: bold)\n}\n";
        let id = id_of(src, "Text");
        let out = apply_edits(
            src,
            &[EditOp::SetArg {
                node: id,
                arg: ArgRef::Positional(0),
                value: "\"bye\"".into(),
            }],
        )
        .unwrap();
        assert!(out.contains("Text(\"bye\", weight: bold)"), "got: {}", out);
        parse_program(&out).unwrap();
    }

    #[test]
    fn overlapping_edits_on_same_node_rejected() {
        let src = "Page P (path: \"/\") {\n  Heading(\"Hello\", h1)\n}\n";
        let id = id_of(src, "Heading");
        // Both target the first positional arg → overlap → must reject (not corrupt).
        let res = apply_edits(
            src,
            &[
                EditOp::SetText {
                    node: id.clone(),
                    value: "A".into(),
                },
                EditOp::SetArg {
                    node: id,
                    arg: ArgRef::Positional(0),
                    value: "\"B\"".into(),
                },
            ],
        );
        assert!(res.is_err(), "expected overlap rejection");
    }

    #[test]
    fn move_node_within_same_parent_no_duplication_or_loss() {
        let src = "Page P (path: \"/\") {\n  Container {\n    Text(\"a\")\n    Text(\"b\")\n    Text(\"c\")\n  }\n}\n";
        let a = id_of(src, "Text(\"a\")");
        let container = id_of(src, "Container");
        let out = apply_edits(
            src,
            &[EditOp::MoveNode {
                node: a,
                new_parent: container,
                index: 2,
            }],
        )
        .unwrap();
        parse_program(&out).unwrap();
        // Each element must survive exactly once — no dup, no drop.
        assert_eq!(
            out.matches("Text(\"a\")").count(),
            1,
            "a duplicated/lost: {}",
            out
        );
        assert_eq!(out.matches("Text(\"b\")").count(), 1);
        assert_eq!(out.matches("Text(\"c\")").count(), 1);
    }

    #[test]
    fn move_node_into_own_subtree_never_corrupts() {
        let src = "Page P (path: \"/\") {\n  Container {\n    Text(\"a\")\n  }\n}\n";
        let container = id_of(src, "Container");
        let text = id_of(src, "Text");
        // Moving a node into its own descendant must never produce invalid output:
        // either an error, or a result that reparses.
        // Refusing is acceptable; producing something that will not reparse is not.
        if let Ok(out) = apply_edits(
            src,
            &[EditOp::MoveNode {
                node: container,
                new_parent: text,
                index: 0,
            }],
        ) {
            parse_program(&out).unwrap();
        }
    }
}
