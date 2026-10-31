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
    /// The bytes `start..end` of the finding's file — a span the finder
    /// holds — become `text`.
    Replace {
        start: usize,
        end: usize,
        text: String,
    },
    /// `text` goes in at the finding's start — `await ` before a call.
    Insert { text: String },
    /// `text` goes in just before the first `anchor` at or after the
    /// finding on its line — ` by t.id` before a `for`'s `{`.
    InsertBefore { anchor: String, text: String },
    /// The statement that starts on the finding's line goes, with its
    /// block, its brackets and its line: an unused state, a dead line.
    RemoveStatement,
    /// `text` becomes the last line inside the block whose `{` is the first
    /// at or after the finding, one level in — a `Router` in an `app`, a
    /// `migrate` in a `persist` block.
    InsertInBlock { text: String },
    /// [`InsertInBlock`](Plan::InsertInBlock) for the block whose closing
    /// `}` is at byte `close` — a finder that knows the statement's end
    /// says which block it means, so a `{` in a value before it is never
    /// mistaken for it.
    InsertInBlockClosingAt { close: usize, text: String },
    /// Every one of these, as one fix: a version raised and the step that
    /// brings the old one forward.
    All(Vec<Plan>),
    /// An edit in another file, or another place of this one, the finder
    /// knows by its byte offset: `text` goes in at `at` of `file`.
    InsertIn {
        file: String,
        at: usize,
        text: String,
    },
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

    /// The fix, when there is one.
    pub fn with_plans(self, plan: Option<(String, Plan)>) -> Self {
        match plan {
            Some((title, plan)) => self.with_plan(title, plan),
            None => self,
        }
    }
}

/// Turn every plan of `d` into a fix against `source`, the text of `d.file`.
pub fn realise(d: &mut Diagnostic, source: &str) {
    let file = d.file.clone();
    let own = source.to_string();
    realise_with(d, &|f: &str| (f == file).then(|| own.clone()));
}

/// Turn every plan of `d` into a fix, reading each file a plan edits
/// through `text_of`. A plan whose file cannot be read, or whose text does
/// not fit it, yields no fix.
pub fn realise_with(d: &mut Diagnostic, text_of: &dyn Fn(&str) -> Option<String>) {
    let plans = std::mem::take(&mut d.plans);
    let Some(source) = text_of(&d.file) else {
        return;
    };
    for planned in plans {
        let edits = match &planned.plan {
            Plan::InsertIn { file, at, text } => text_of(file).and_then(|other| {
                (*at <= other.len() && other.is_char_boundary(*at)).then(|| {
                    let (line, column) = place(&other, *at);
                    vec![Edit {
                        file: file.clone(),
                        line,
                        column,
                        end_line: line,
                        end_column: column,
                        text: text.clone(),
                    }]
                })
            }),
            Plan::All(plans) => plans
                .iter()
                .map(|p| edits(d, &source, p))
                .collect::<Option<Vec<_>>>()
                .map(|all| all.concat()),
            plan => edits(d, &source, plan),
        };
        if let Some(edits) = edits {
            d.fixes.push(Fix {
                title: planned.title,
                edits,
            });
        }
    }
}

/// The first fix of each of `diagnostics`, merged into one set of edits:
/// a fix that would overlap one already taken is left out whole, for the
/// next round. Returns the edits, in order, and the findings they fix.
pub fn merge<'a>(
    diagnostics: impl IntoIterator<Item = &'a Diagnostic>,
) -> (Vec<Edit>, Vec<&'a Diagnostic>) {
    let mut taken: Vec<Edit> = Vec::new();
    let mut fixed = Vec::new();
    let overlaps = |a: &Edit, b: &Edit| {
        a.file == b.file
            && ((a.line, a.column) == (b.line, b.column)
                || ((a.line, a.column) < (b.end_line, b.end_column)
                    && (b.line, b.column) < (a.end_line, a.end_column)))
    };
    for d in diagnostics {
        let Some(fix) = d.fixes.first() else {
            continue;
        };
        if fix
            .edits
            .iter()
            .any(|e| taken.iter().any(|t| overlaps(e, t)))
        {
            continue;
        }
        taken.extend(fix.edits.iter().cloned());
        fixed.push(d);
    }
    taken.sort_by_key(|e| (e.file.clone(), e.line, e.column));
    (taken, fixed)
}

/// Every finding of `code` in `file` fixed at once, as one change set: the
/// first fix of each, its edits merged, a fix that would overlap one
/// already taken left out whole. `None` when there is nothing to fix.
pub fn fix_all(diagnostics: &[Diagnostic], code: &str, file: &str) -> Option<Fix> {
    let (taken, fixed) = merge(
        diagnostics
            .iter()
            .filter(|d| d.code == code && d.file == file),
    );
    let count = fixed.len();
    if count == 0 {
        return None;
    }
    let short = file.rsplit('/').next().unwrap_or(file);
    Some(Fix {
        title: format!("Fix all {count} {code} in {short}"),
        edits: taken,
    })
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
    // A limit in bytes may end inside a character: it stops before it.
    let mut end = (from + limit).min(source.len());
    while !source.is_char_boundary(end) {
        end -= 1;
    }
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
        Plan::Replace { start, end, text } => {
            if *end > source.len()
                || start > end
                || !source.is_char_boundary(*start)
                || !source.is_char_boundary(*end)
            {
                return None;
            }
            Some(vec![edit(d, source, *start, *end, text.clone())])
        }
        Plan::Insert { text } => Some(vec![edit(d, source, start, start, text.clone())]),
        Plan::InsertBefore { anchor, text } => {
            let line_end = source[start..]
                .find('\n')
                .map_or(source.len(), |n| start + n);
            let at = start + source[start..line_end].find(anchor.as_str())?;
            // ` by t.id ` before `{`: no double space where one was.
            let trimmed = source[..at].trim_end_matches(' ').len();
            Some(vec![edit(d, source, trimmed, at, format!("{text} "))])
        }
        Plan::RemoveStatement => {
            let line_start = source[..start].rfind('\n').map_or(0, |n| n + 1);
            // Only a statement that is the whole of its line: whatever is
            // before the finding is its keyword, never another statement.
            let before = &source[line_start..start];
            if before.contains(['{', '}', '(', ')', ';', '"', '=']) {
                return None;
            }
            let mut i = start;
            let bytes = source.as_bytes();
            while i < bytes.len() && bytes[i] != b'\n' {
                match bytes[i] {
                    b'(' | b'{' | b'[' => i = matching(source, i)?,
                    b'"' => {
                        i += 1;
                        while i < bytes.len() && bytes[i] != b'"' {
                            if bytes[i] == b'\\' {
                                i += 1;
                            }
                            i += 1;
                        }
                        i += 1;
                    }
                    b'/' if bytes.get(i + 1) == Some(&b'/') => {
                        while i < bytes.len() && bytes[i] != b'\n' {
                            i += 1;
                        }
                    }
                    b'}' | b')' | b']' => return None,
                    _ => i += 1,
                }
            }
            let end = if i < bytes.len() { i + 1 } else { i };
            // Its `///` doc comment goes with it: left, it would document
            // whatever comes next.
            let mut from = line_start;
            while from > 0 {
                let prev_start = source[..from - 1].rfind('\n').map_or(0, |n| n + 1);
                if source[prev_start..from].trim_start().starts_with("///") {
                    from = prev_start;
                } else {
                    break;
                }
            }
            Some(vec![edit(d, source, from, end, "")])
        }
        Plan::InsertInBlockClosingAt { close, text } => {
            if source.as_bytes().get(*close) != Some(&b'}') {
                return None;
            }
            let open = opening(source, *close)?;
            insert_in_block(d, source, open, text)
        }
        Plan::All(plans) => plans
            .iter()
            .map(|p| edits(d, source, p))
            .collect::<Option<Vec<_>>>()
            .map(|all| all.concat()),
        Plan::InsertInBlock { text } => {
            let open = start + source[start..(start + 4000).min(source.len())].find('{')?;
            insert_in_block(d, source, open, text)
        }
        Plan::InsertIn { .. } => None,
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

/// `text` as the last line(s) inside the block that opens at `open`, one
/// level in from the line the `{` is on.
fn insert_in_block(d: &Diagnostic, source: &str, open: usize, text: &str) -> Option<Vec<Edit>> {
    let end = matching(source, open)?;
    let close = end - 1;
    let outer = indent_of(source, open).to_string();
    let inner = format!("{outer}    ");
    let line_start = source[..close].rfind('\n').map_or(0, |n| n + 1);
    let lines: String = text
        .lines()
        .map(|l| {
            if l.is_empty() {
                "\n".to_string()
            } else {
                format!("{inner}{l}\n")
            }
        })
        .collect();
    if source[line_start..close].trim().is_empty() && line_start > open {
        Some(vec![edit(d, source, line_start, line_start, lines)])
    } else {
        // `{ }` on one line: the block opens out onto lines.
        let inside = source[open + 1..close].trim();
        let mut body = String::new();
        if !inside.is_empty() {
            body.push_str(&format!("{inner}{inside}\n"));
        }
        body.push_str(&lines);
        Some(vec![edit(
            d,
            source,
            open + 1,
            close,
            format!("\n{body}{outer}"),
        )])
    }
}

/// The `{` matching the `}` at `close`, scanning back over strings.
fn opening(source: &str, close: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut i = close + 1;
    let mut in_string = false;
    while i > 0 {
        i -= 1;
        let c = bytes[i];
        if in_string {
            if c == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
                in_string = false;
            }
            continue;
        }
        match c {
            b'"' => in_string = true,
            b'}' => depth += 1,
            b'{' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
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
    fn a_statement_goes_with_its_block_and_its_line() {
        let src = "page P {\n    state unused = 1\n    Text(\"a\")\n}\n";
        assert_eq!(
            fixed(src, 2, 5, Plan::RemoveStatement),
            "page P {\n    Text(\"a\")\n}\n"
        );
        let src =
            "store S {\n    action old(x: Number) {\n        log(x)\n    }\n    state n = 0\n}\n";
        assert_eq!(
            fixed(src, 2, 5, Plan::RemoveStatement),
            "store S {\n    state n = 0\n}\n"
        );
        // A list written over lines goes whole.
        let src = "    state rows = [\n        1,\n        2,\n    ]\n    Text(\"x\")\n";
        assert_eq!(fixed(src, 1, 5, Plan::RemoveStatement), "    Text(\"x\")\n");
        // Its doc comment goes with it; a plain comment above is not its.
        let src = "store S {\n    // the store's\n    /// What was asked.\n    /// Over two lines.\n    state q = \"\"\n    state n = 0\n}\n";
        assert_eq!(
            fixed(src, 5, 5, Plan::RemoveStatement),
            "store S {\n    // the store's\n    state n = 0\n}\n"
        );
        // Not a statement of its own line: no fix rather than a wrong one.
        let src = "    Row { state a = 1 }\n";
        let mut d =
            Diagnostic::coded("U01", "x", "a.wf", 1, 15).with_plan("fix", Plan::RemoveStatement);
        realise(&mut d, src);
        assert!(d.fixes.is_empty());
    }

    #[test]
    fn a_line_goes_into_a_block_one_level_in() {
        let src = "app {\n    Navbar { Text(\"x\") }\n}\n";
        assert_eq!(
            fixed(
                src,
                1,
                1,
                Plan::InsertInBlock {
                    text: "Router".into()
                }
            ),
            "app {\n    Navbar { Text(\"x\") }\n    Router\n}\n"
        );
        let src = "    persist n = 0 { version: 2 }\n";
        assert_eq!(
            fixed(
                src,
                1,
                5,
                Plan::InsertInBlock {
                    text: "migrate 1 -> 2 { old }".into()
                }
            ),
            "    persist n = 0 {\n        version: 2\n        migrate 1 -> 2 { old }\n    }\n"
        );
    }

    #[test]
    fn text_goes_in_at_the_finding_or_before_an_anchor() {
        assert_eq!(
            fixed(
                "    x = load()\n",
                1,
                9,
                Plan::Insert {
                    text: "await ".into()
                }
            ),
            "    x = await load()\n"
        );
        assert_eq!(
            fixed(
                "    for t in rows {\n",
                1,
                5,
                Plan::InsertBefore {
                    anchor: "{".into(),
                    text: " by t.id".into()
                }
            ),
            "    for t in rows by t.id {\n"
        );
        assert_eq!(
            fixed(
                "    Text(n)\n",
                1,
                10,
                Plan::Replace {
                    start: 9,
                    end: 10,
                    text: "\"{n}\"".into()
                }
            ),
            "    Text(\"{n}\")\n"
        );
    }

    #[test]
    fn an_edit_may_be_in_another_file() {
        let mut d = Diagnostic::coded("C06", "x", "page.wf", 1, 1).with_plan(
            "mark it",
            Plan::InsertIn {
                file: "panel.wf".into(),
                at: 16,
                text: "_ ".into(),
            },
        );
        let panel = "component Panel(title: String) {}\n".to_string();
        realise_with(&mut d, &|f| match f {
            "page.wf" => Some("Panel(\"x\")\n".to_string()),
            "panel.wf" => Some(panel.clone()),
            _ => None,
        });
        assert_eq!(d.fixes.len(), 1);
        assert_eq!(d.fixes[0].edits[0].file, "panel.wf");
        assert_eq!(
            apply(&panel, &d.fixes[0]),
            "component Panel(_ title: String) {}\n"
        );
    }

    #[test]
    fn a_family_fixed_at_once_is_one_change_set() {
        let src = "Text(\"a\", center)\nText(\"b\", center)\nText(\"c\")\n";
        let mut found = Vec::new();
        for line in [1, 2] {
            let mut d = Diagnostic::coded("V01", "x", "a.wf", line, 11).with_plan(
                "flag",
                Plan::Flag {
                    word: "center".into(),
                    flag: "center".into(),
                },
            );
            realise(&mut d, src);
            found.push(d);
        }
        // Another file's and another code's are not this family in this file.
        found.push(Diagnostic::coded("V01", "x", "b.wf", 1, 1));
        found.push(Diagnostic::coded("U01", "x", "a.wf", 1, 1));
        let all = fix_all(&found, "V01", "a.wf").expect("a change set");
        assert_eq!(all.title, "Fix all 2 V01 in a.wf");
        assert_eq!(
            apply(src, &all),
            "Text(\"a\").center\nText(\"b\").center\nText(\"c\")\n"
        );
        assert!(fix_all(&found, "T05", "a.wf").is_none());
        // Two fixes of one place: the second is left out, not merged into it.
        let twice = vec![found[0].clone(), found[0].clone()];
        assert_eq!(
            fix_all(&twice, "V01", "a.wf").unwrap().title,
            "Fix all 1 V01 in a.wf"
        );
    }

    #[test]
    fn a_search_that_would_end_inside_a_character_stops_before_it() {
        // The 400 bytes a search looks through end inside the dash.
        let source = format!("{}word {}—", " ".repeat(10), "x".repeat(384));
        assert_eq!(find_word(&source, 0, "word", 400), Some(10));
        assert_eq!(find_word(&source, 0, "nowhere", 400), None);
    }

    #[test]
    fn nearest_is_a_slip_never_the_word() {
        assert_eq!(nearest("nmae", ["name", "id"]), Some("name"));
        assert_eq!(nearest("name", ["name"]), None);
        assert_eq!(nearest("xyz", ["name"]), None);
    }
}
