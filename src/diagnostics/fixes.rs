//! Fixes a finder knows the shape of but not the text of. A lint walks the
//! tree and knows *what* would fix a finding — this name becomes that one,
//! this call takes one more argument — but not the bytes; the pipeline has
//! every file's source, and turns each [`Plan`] into the [`Edit`]s an editor
//! applies. A plan that does not fit the text it finds (the source moved on,
//! the call has no parentheses where one was expected) yields no fix rather
//! than a wrong one.

use super::{Diagnostic, Edit, Fix};

/// What would fix a finding, in terms of the text at its place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// The first occurrence of `from` at or after the finding's start, as a
    /// whole word where its edge is a word character, becomes `to`.
    Rename { from: String, to: String },
    /// `text` becomes the last argument of the call that starts at the
    /// finding — `Image(src: x)` → `Image(src: x, alt: "")`, `Card` →
    /// `Card(title: "")`.
    AddArgument { text: String },
    /// The bare word at the finding, written among a call's arguments,
    /// becomes the flag `.flag` after its parentheses — `Text("a", center)`
    /// → `Text("a").center`.
    Flag { word: String, flag: String },
    /// Arms for the cases a `match` at or after the finding leaves out,
    /// before its closing brace, one a line.
    AddArms { arms: Vec<String> },
}

/// A plan and the words an editor shows for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    pub title: String,
    pub plan: Plan,
}

impl Diagnostic {
    /// A fix this finding offers, to be written out against the source by
    /// [`realise`].
    pub fn with_plan(mut self, title: impl Into<String>, plan: Plan) -> Self {
        self.plans.push(Planned {
            title: title.into(),
            plan,
        });
        self
    }
}

/// Turn every plan of `d` into a fix against `source`, the text of `d.file`.
pub fn realise(d: &mut Diagnostic, source: &str) {
    let plans = std::mem::take(&mut d.plans);
    for planned in plans {
        if let Some(edits) = edits(d, source, &planned.plan) {
            d.fixes.push(Fix {
                title: planned.title,
                edits,
            });
        }
    }
}

/// The nearest of `candidates` to `word` — close enough to be a slip of the
/// hand, never `word` itself.
pub fn nearest<'a>(word: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let limit = match word.chars().count() {
        0..=2 => 1,
        3..=5 => 2,
        _ => 3,
    };
    candidates
        .into_iter()
        .filter(|c| *c != word)
        .map(|c| (crate::linter::vocabulary::levenshtein(word, c), c))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, c)| (*d, c.len(), *c))
        .map(|(_, c)| c)
}

/// The byte offset of a 1-based line and column (in characters); the
/// column one past the line's last character is its end.
fn offset(source: &str, line: usize, column: usize) -> Option<usize> {
    let mut start = 0;
    for _ in 1..line {
        start += source[start..].find('\n')? + 1;
    }
    let text = &source[start..];
    let line_text = &text[..text.find('\n').unwrap_or(text.len())];
    let index = column.checked_sub(1)?;
    match line_text.char_indices().nth(index) {
        Some((byte, _)) => Some(start + byte),
        None if index == line_text.chars().count() => Some(start + line_text.len()),
        None => None,
    }
}

/// The 1-based line and column of a byte offset.
fn place(source: &str, at: usize) -> (usize, usize) {
    let before = &source[..at];
    let line = before.matches('\n').count() + 1;
    let column = match before.rfind('\n') {
        Some(nl) => before[nl + 1..].chars().count() + 1,
        None => before.chars().count() + 1,
    };
    (line, column)
}

fn edit(d: &Diagnostic, source: &str, from: usize, to: usize, text: impl Into<String>) -> Edit {
    let (line, column) = place(source, from);
    let (end_line, end_column) = place(source, to);
    Edit {
        file: d.file.clone(),
        line,
        column,
        end_line,
        end_column,
        text: text.into(),
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The first occurrence of `needle` at or after `from`, as a whole word at
/// the edges that are word characters, within `limit` bytes.
fn find_word(source: &str, from: usize, needle: &str, limit: usize) -> Option<usize> {
    let end = (from + limit).min(source.len());
    let mut at = from;
    while at < end {
        let i = at + source[at..end].find(needle)?;
        let before = source[..i].chars().next_back();
        let after = source[i + needle.len()..].chars().next();
        let starts_word = needle.chars().next().is_some_and(is_word);
        let ends_word = needle.chars().next_back().is_some_and(is_word);
        let clear_before = !starts_word || before.is_none_or(|c| !is_word(c));
        let clear_after = !ends_word || after.is_none_or(|c| !is_word(c));
        if clear_before && clear_after {
            return Some(i);
        }
        at = i + needle.len().max(1);
    }
    None
}

/// The offset just past the bracket matching the one at `open`, skipping
/// strings and comments.
fn matching(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let (left, right) = match bytes.get(open)? {
        b'(' => (b'(', b')'),
        b'{' => (b'{', b'}'),
        b'[' => (b'[', b']'),
        _ => return None,
    };
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            c if c == left => depth += 1,
            c if c == right => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// The `(` that opens the argument list around `at`: the nearest unmatched
/// one before it.
fn enclosing_paren(source: &str, at: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut i = at;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b')' | b']' | b'}' => depth += 1,
            b'(' | b'[' | b'{' => {
                if depth == 0 {
                    return (bytes[i] == b'(').then_some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// The indentation of the line `at` is on.
fn indent_of(source: &str, at: usize) -> &str {
    let start = source[..at].rfind('\n').map_or(0, |n| n + 1);
    let line = &source[start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

fn edits(d: &Diagnostic, source: &str, plan: &Plan) -> Option<Vec<Edit>> {
    let start = offset(source, d.line, d.column)?;
    match plan {
        Plan::Rename { from, to } => {
            // A finding placed on the name of `.name` starts after the dot
            // its rename names: the search starts that far back.
            let lead = from.len() - from.trim_start_matches(['.', '?']).len();
            let from_at = if source[..start].ends_with(&from[..lead]) {
                start - lead
            } else {
                start
            };
            let at = find_word(source, from_at, from, 400)?;
            Some(vec![edit(d, source, at, at + from.len(), to.clone())])
        }
        Plan::AddArgument { text } => {
            // The callee's name — `Image`, `Panel.Header` — then its
            // arguments, if it has any.
            let rest = &source[start..];
            let name_len = rest
                .find(|c: char| !(is_word(c) || c == '.'))
                .unwrap_or(rest.len());
            // Flags after a bare call (`Spinner.lg`) are part of the name
            // scan; a call with arguments has them after the `)`.
            let after_name = start + name_len;
            if source[after_name..].starts_with('(') {
                let close = matching(source, after_name)? - 1;
                let inside = source[after_name + 1..close].trim_end();
                let insert_at = after_name + 1 + inside.len();
                let text = if inside.trim().is_empty() {
                    text.clone()
                } else if inside.ends_with(',') {
                    format!(" {text}")
                } else {
                    format!(", {text}")
                };
                Some(vec![edit(d, source, insert_at, insert_at, text)])
            } else {
                // No parentheses: they go right after the component's name,
                // before any flag.
                let name = &rest[..name_len];
                let bare = name
                    .split('.')
                    .take_while(|s| s.chars().next().is_some_and(char::is_uppercase))
                    .collect::<Vec<_>>()
                    .join(".");
                let at = start + bare.len();
                Some(vec![edit(d, source, at, at, format!("({text})"))])
            }
        }
        Plan::Flag { word, flag } => {
            if !source[start..].starts_with(word.as_str()) {
                return None;
            }
            let word_end = start + word.len();
            let open = enclosing_paren(source, start)?;
            let close = matching(source, open)? - 1;
            let inside = &source[open + 1..close];
            let others: Vec<&str> = split_args(inside)
                .into_iter()
                .map(str::trim)
                .filter(|a| !a.is_empty() && *a != word.as_str())
                .collect();
            // What the word leaves behind: the argument list without it, or
            // nothing at all when it was the only argument.
            let mut out = Vec::new();
            if others.is_empty() {
                out.push(edit(d, source, open, close + 1, format!(".{flag}")));
            } else {
                // The word and the comma that joins it to its neighbour.
                let before = source[open + 1..start].trim_end();
                let (from, to) = if before.ends_with(',') {
                    (open + 1 + before.len() - 1, word_end)
                } else {
                    let after = &source[word_end..close];
                    let comma = after.find(',').map_or(0, |c| c + 1);
                    let spaces = after[comma..].len() - after[comma..].trim_start().len();
                    (start, word_end + comma + spaces)
                };
                out.push(edit(d, source, from, to, ""));
                out.push(edit(d, source, close + 1, close + 1, format!(".{flag}")));
            }
            Some(out)
        }
        Plan::AddArms { arms } => {
            let at = find_word(source, start, "match", 400).unwrap_or(start);
            let open = at + source[at..].find('{')?;
            // A match whose block is written by indentation (`.wfx`) has no
            // brace on its line: the one found is somebody else's.
            if source[at..open].contains('\n') {
                return None;
            }
            let end = matching(source, open)?;
            let close = end - 1;
            let indent = indent_of(source, close).to_string();
            // Inside the braces, one level in from the closing one.
            let arm_indent = format!("{indent}    ");
            let line_start = source[..close].rfind('\n').map_or(0, |n| n + 1);
            if source[line_start..close].trim().is_empty() {
                // `}` on its own line: the arms go on lines of their own
                // above it.
                let text: String = arms.iter().map(|a| format!("{arm_indent}{a}\n")).collect();
                Some(vec![edit(d, source, line_start, line_start, text)])
            } else {
                // A match on one line: the arms join it.
                let text: String = arms.iter().map(|a| format!(" {a}")).collect();
                let insert_at = open + 1 + source[open + 1..close].trim_end().len();
                Some(vec![edit(d, source, insert_at, insert_at, text)])
            }
        }
    }
}

/// An argument list split at its top-level commas.
fn split_args(inside: &str) -> Vec<&str> {
    let bytes = inside.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                out.push(&inside[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&inside[start..]);
    out
}

/// What `source` reads after every edit of `fix` is applied — for tests,
/// and for a tool that applies a fix without an editor.
pub fn apply(source: &str, fix: &Fix) -> String {
    let mut spans: Vec<(usize, usize, &str)> = fix
        .edits
        .iter()
        .filter_map(|e| {
            Some((
                offset(source, e.line, e.column)?,
                offset(source, e.end_line, e.end_column)?,
                e.text.as_str(),
            ))
        })
        .collect();
    spans.sort_by_key(|s| std::cmp::Reverse(s.0));
    let mut out = source.to_string();
    for (from, to, text) in spans {
        out.replace_range(from..to, text);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(source: &str, line: usize, column: usize, plan: Plan) -> String {
        let mut d = Diagnostic::coded("T05", "x", "a.wf", line, column).with_plan("fix", plan);
        realise(&mut d, source);
        assert_eq!(d.fixes.len(), 1, "no fix for {source:?}");
        apply(source, &d.fixes[0])
    }

    #[test]
    fn a_rename_takes_the_whole_word() {
        let src = "page P(path: \"/\") {\n    Text(user.nmae)\n}\n";
        let out = fixed(
            src,
            2,
            10,
            Plan::Rename {
                from: "nmae".into(),
                to: "name".into(),
            },
        );
        assert!(out.contains("Text(user.name)"), "{out}");
        // `s` inside `state` is not the word `s`.
        let src = "    state s = 1\n";
        let out = fixed(
            src,
            1,
            5,
            Plan::Rename {
                from: "s".into(),
                to: "_s".into(),
            },
        );
        assert_eq!(out, "    state _s = 1\n");
    }

    #[test]
    fn a_member_rename_finds_the_dot_before_the_finding() {
        let src = "    Text(u.nmae)\n";
        let out = fixed(
            src,
            1,
            12,
            Plan::Rename {
                from: ".nmae".into(),
                to: ".name".into(),
            },
        );
        assert_eq!(out, "    Text(u.name)\n");
    }

    #[test]
    fn an_argument_joins_the_call() {
        let add = |text: &str| Plan::AddArgument { text: text.into() };
        assert_eq!(
            fixed("Image(src: \"/a.png\")\n", 1, 1, add("alt: \"\"")),
            "Image(src: \"/a.png\", alt: \"\")\n"
        );
        assert_eq!(
            fixed("Card().elevated\n", 1, 1, add("title: \"\"")),
            "Card(title: \"\").elevated\n"
        );
        assert_eq!(
            fixed("Card.elevated {\n", 1, 1, add("title: \"\"")),
            "Card(title: \"\").elevated {\n"
        );
        assert_eq!(
            fixed("Panel.Header\n", 1, 1, add("text: \"\"")),
            "Panel.Header(text: \"\")\n"
        );
    }

    #[test]
    fn a_bare_word_becomes_a_flag() {
        let flag = |w: &str, f: &str| Plan::Flag {
            word: w.into(),
            flag: f.into(),
        };
        assert_eq!(
            fixed("Text(\"a\", center)\n", 1, 11, flag("center", "center")),
            "Text(\"a\").center\n"
        );
        assert_eq!(
            fixed("Spinner(lg)\n", 1, 9, flag("lg", "lg")),
            "Spinner.lg\n"
        );
        assert_eq!(
            fixed(
                "Text(centered, \"a\").bold\n",
                1,
                6,
                flag("centered", "center")
            ),
            "Text(\"a\").center.bold\n"
        );
    }

    #[test]
    fn arms_go_before_the_closing_brace() {
        let src = "    match tone {\n        .calm { Text(\"c\") }\n    }\n";
        let out = fixed(
            src,
            1,
            5,
            Plan::AddArms {
                arms: vec![".loud { }".into()],
            },
        );
        assert_eq!(
            out,
            "    match tone {\n        .calm { Text(\"c\") }\n        .loud { }\n    }\n"
        );
        let src = "derived n = match t { .a { 1 } }\n";
        let out = fixed(
            src,
            1,
            13,
            Plan::AddArms {
                arms: vec![".b { null }".into()],
            },
        );
        assert_eq!(out, "derived n = match t { .a { 1 } .b { null } }\n");
    }

    #[test]
    fn nearest_is_a_slip_never_the_word() {
        assert_eq!(nearest("nmae", ["name", "id"]), Some("name"));
        assert_eq!(nearest("name", ["name"]), None);
        assert_eq!(nearest("xyz", ["name"]), None);
    }
}
