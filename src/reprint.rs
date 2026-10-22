//! The smallest edit: given a node as it was parsed and the same node as an
//! editor changed it, the patches that turn the one's source into the
//! other's — and nothing else in the file moves.
//!
//! The two trees are compared from the node down. What is the same keeps
//! its bytes, comments and spacing and all. Where they part, the narrowest
//! piece that differs is printed ([`crate::print`]) in its place: one
//! argument of a call, one flag, one style value, the head of an `if`, one
//! statement of a block. A statement added is written on its own line
//! among its neighbours; one removed takes its line with it. Only where a
//! piece cannot be addressed by its own span is the node around it printed
//! whole — an element whose name changed, a block whose shape did.
//!
//! The same holds a level up: `program` compares two trees of a file,
//! declaration by declaration — a component's header, one prop, an event,
//! a part, its `///`; a page's header and `head { … }`; a type's fields; a
//! theme's tokens — and adds and removes whole declarations with their
//! comments and the blank line between them.
//!
//! [`crate::edit`] puts this behind its rewrites and holds the result to
//! the tree the editor asked for.

use crate::parser::ast::*;
use crate::print::{self, Block};

/// Replace `start..end` of the source with `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl Patch {
    fn new(start: usize, end: usize, text: impl Into<String>) -> Self {
        Patch {
            start,
            end,
            text: text.into(),
        }
    }
}

/// The source with `patches` applied. Two that overlap are a mistake in
/// whoever made them, and nothing is changed.
pub fn apply(source: &str, mut patches: Vec<Patch>) -> Result<String, String> {
    patches.sort_by_key(|p| (p.start, p.end));
    for pair in patches.windows(2) {
        if pair[1].start < pair[0].end {
            return Err(format!(
                "two changes overlap, at {}..{} and {}..{}",
                pair[0].start, pair[0].end, pair[1].start, pair[1].end
            ));
        }
    }
    let mut out = source.to_string();
    for p in patches.iter().rev() {
        if p.end > out.len()
            || p.start > p.end
            || !out.is_char_boundary(p.start)
            || !out.is_char_boundary(p.end)
        {
            return Err(format!(
                "a change at {}..{} is outside the source",
                p.start, p.end
            ));
        }
        out.replace_range(p.start..p.end, &p.text);
    }
    Ok(out)
}

fn shape_of<T: std::fmt::Debug>(node: &T) -> String {
    print::without_places(&format!("{node:?}"))
}

/// Whether two nodes are the same, wherever each was written.
pub fn same<T: std::fmt::Debug>(a: &T, b: &T) -> bool {
    shape_of(a) == shape_of(b)
}

/// The depth of the line `at` is on, by its leading spaces.
fn depth_at(source: &str, at: usize) -> usize {
    let line_start = source[..at.min(source.len())]
        .rfind('\n')
        .map_or(0, |n| n + 1);
    let spaces = source[line_start..]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum::<usize>();
    spaces / 4
}

fn pad(depth: usize) -> String {
    "    ".repeat(depth)
}

/// The one patch that turns `before` into `after`: what lies between what
/// they share at the front and at the back. For following a span across an
/// edit made elsewhere, as [`follow`] does with an edit's own patches.
pub fn between(before: &str, after: &str) -> Patch {
    let (a, b) = (before.as_bytes(), after.as_bytes());
    let mut front = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    while !before.is_char_boundary(front) {
        front -= 1;
    }
    let mut back = a[front..]
        .iter()
        .rev()
        .zip(b[front..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    while !before.is_char_boundary(a.len() - back) || !after.is_char_boundary(b.len() - back) {
        back -= 1;
    }
    Patch::new(front, a.len() - back, &after[front..b.len() - back])
}

/// Where `start..end` is once `patches` are applied — a selection kept
/// across an edit. A patch before it moves it, one inside it grows or
/// shrinks it; one that overlaps an edge, or covers it, leaves it nowhere.
pub fn follow(start: usize, end: usize, patches: &[Patch]) -> Option<(usize, usize)> {
    let (mut s, mut e) = (start as isize, end as isize);
    for p in patches {
        let delta = p.text.len() as isize - (p.end as isize - p.start as isize);
        if p.end <= start && p.start < start {
            s += delta;
            e += delta;
        } else if p.end <= start && p.start == p.end && p.start == start {
            // Text written right where it starts is before it.
            s += delta;
            e += delta;
        } else if p.start >= end {
            // After it: nothing moves.
        } else if p.start >= start && p.end <= end {
            e += delta;
        } else {
            return None;
        }
    }
    (s >= 0 && e >= s).then_some((s as usize, e as usize))
}

// ─── Finding the braces ──────────────────────────────────

/// The `{ … }` groups at the top level of `source[start..end]`, as the
/// offsets of each `{` and its `}`: strings, comments and what is inside
/// parentheses or brackets are passed over.
pub fn blocks_in(source: &str, start: usize, end: usize) -> Vec<(usize, usize)> {
    let bytes = source.as_bytes();
    let end = end.min(bytes.len());
    let mut out = Vec::new();
    let mut i = start;
    let mut parens = 0usize;
    let mut braces: Vec<usize> = Vec::new();
    while i < end {
        let c = bytes[i];
        match c {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < end && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            b'#' if raw_string_at(bytes, i) => {
                let hashes = bytes[i..].iter().take_while(|b| **b == b'#').count();
                i += hashes + 1;
                let close: Vec<u8> = std::iter::once(b'"')
                    .chain(std::iter::repeat_n(b'#', hashes))
                    .collect();
                while i < end && !bytes[i..].starts_with(&close) {
                    i += 1;
                }
                i += close.len();
                continue;
            }
            b'"' => {
                i = string_end(bytes, i, end);
                continue;
            }
            // `/` after an operand divides; anywhere else it opens a
            // regular expression, as the lexer reads it.
            b'/' if !after_operand(bytes, start, i) => {
                i = regex_end(bytes, i, end);
                continue;
            }
            b'(' | b'[' => parens += 1,
            b')' | b']' => parens = parens.saturating_sub(1),
            b'{' if parens == 0 => braces.push(i),
            b'}' if parens == 0 => {
                if let Some(open) = braces.pop()
                    && braces.is_empty()
                {
                    out.push((open, i));
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// Whether what comes before `i` (back to `from`) ends an operand.
fn after_operand(bytes: &[u8], from: usize, i: usize) -> bool {
    let before = bytes[from..i]
        .iter()
        .rev()
        .find(|b| !b.is_ascii_whitespace());
    before
        .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b')' | b']' | b'"' | b'}'))
}

/// Past the regular expression that opens at `i`, and its flags.
fn regex_end(bytes: &[u8], i: usize, end: usize) -> usize {
    let mut j = i + 1;
    let mut class = false;
    while j < end {
        match bytes[j] {
            b'\\' => j += 1,
            b'[' => class = true,
            b']' => class = false,
            b'/' if !class => {
                j += 1;
                while j < end && bytes[j].is_ascii_alphabetic() {
                    j += 1;
                }
                return j;
            }
            b'\n' => return j,
            _ => {}
        }
        j += 1;
    }
    j
}

fn raw_string_at(bytes: &[u8], i: usize) -> bool {
    let hashes = bytes[i..].iter().take_while(|b| **b == b'#').count();
    bytes.get(i + hashes) == Some(&b'"')
}

/// Past the string that opens at `i`: a block string, or a plain one with
/// its escapes and its splices — a splice may hold a string of its own.
fn string_end(bytes: &[u8], i: usize, end: usize) -> usize {
    if bytes[i..].starts_with(b"\"\"\"") {
        let mut j = i + 3;
        while j + 2 < end && !bytes[j..].starts_with(b"\"\"\"") {
            j += 1;
        }
        return j + 3;
    }
    let mut j = i + 1;
    let mut depth = 0usize;
    while j < end {
        match bytes[j] {
            b'\\' => j += 1,
            b'{' => depth += 1,
            b'}' => depth = depth.saturating_sub(1),
            b'"' if depth > 0 => j = string_end(bytes, j, end) - 1,
            b'"' => return j + 1,
            b'\n' if depth == 0 => return j,
            _ => {}
        }
        j += 1;
    }
    j
}

// ─── Statements ──────────────────────────────────────────

/// What turns `old`, as it is written in `source`, into `new`.
pub fn statement(source: &str, old: &Statement, new: &Statement, block: Block) -> Vec<Patch> {
    let mut out = Vec::new();
    diff_statement(source, old, new, block, &mut out);
    out
}

fn whole(source: &str, old: &Statement, new: &Statement, block: Block, out: &mut Vec<Patch>) {
    let depth = depth_at(source, old.span.start as usize);
    out.push(Patch::new(
        written_start(source, old),
        old.span.end as usize,
        print::statement(new, block, depth),
    ));
}

/// Where a statement is written from: its `///`, when it has one (a
/// member's documentation goes where it goes, and is printed with it), else
/// its first word.
fn written_start(source: &str, s: &Statement) -> usize {
    let start = s.span.start as usize;
    if s.doc().is_none() || start > source.len() {
        return start;
    }
    let line = print::doc_start(source, start);
    let indent = source[line..start].len() - source[line..start].trim_start().len();
    line + indent
}

fn diff_statement(
    source: &str,
    old: &Statement,
    new: &Statement,
    block: Block,
    out: &mut Vec<Patch>,
) {
    if same(old, new) {
        return;
    }
    let before = out.len();
    let done = match (&old.kind, &new.kind) {
        (StatementKind::UIElement(a), StatementKind::UIElement(b)) => {
            diff_element(source, a, b, out)
        }
        (StatementKind::If(a), StatementKind::If(b)) => diff_if(source, old, a, b, block, out),
        (StatementKind::For(a), StatementKind::For(b)) => {
            let head = |f: &ForStmt| {
                let item = match &f.index {
                    Some(i) => format!("{}, {i}", f.item),
                    None => f.item.clone(),
                };
                let key = f
                    .key
                    .as_ref()
                    .map(|k| format!(" by {}", print::expr(k)))
                    .unwrap_or_default();
                format!("for {item} in {}{key}", print::expr(&f.iterable))
            };
            one_block(
                source,
                old,
                &head(a),
                &head(b),
                &a.body,
                &b.body,
                block,
                out,
            )
        }
        (StatementKind::Show(a), StatementKind::Show(b)) => {
            let head = |s: &ShowStmt| format!("show {}", print::expr(&s.condition));
            one_block(
                source,
                old,
                &head(a),
                &head(b),
                &a.body,
                &b.body,
                Block::Render,
                out,
            )
        }
        (StatementKind::EventHandler(a), StatementKind::EventHandler(b)) => {
            diff_handler(source, a, b, out)
        }
        (StatementKind::Match(a), StatementKind::Match(b)) => diff_match(source, old, a, b, out),
        _ => false,
    };
    if !done {
        out.truncate(before);
        whole(source, old, new, block, out);
    }
}

/// A statement of one block — `for`, `show` — whose head and body are
/// each compared on their own.
#[allow(clippy::too_many_arguments)]
fn one_block(
    source: &str,
    old: &Statement,
    old_head: &str,
    new_head: &str,
    old_body: &[Statement],
    new_body: &[Statement],
    block: Block,
    out: &mut Vec<Patch>,
) -> bool {
    let (start, end) = (old.span.start as usize, old.span.end as usize);
    let blocks = blocks_in(source, start, end);
    let Some(&(open, close)) = blocks.first() else {
        return false;
    };
    if old_head != new_head {
        out.push(Patch::new(start, open, format!("{new_head} ")));
    }
    diff_body(
        source,
        old_body,
        new_body,
        (open + 1, close),
        block,
        depth_at(source, start) + 1,
        out,
    );
    true
}

fn diff_if(
    source: &str,
    old: &Statement,
    a: &IfStmt,
    b: &IfStmt,
    block: Block,
    out: &mut Vec<Patch>,
) -> bool {
    // The same branches, each compared on its own; a branch added or
    // removed is the whole `if` again.
    if a.else_if_branches.len() != b.else_if_branches.len()
        || a.else_body.is_some() != b.else_body.is_some()
    {
        return false;
    }
    let (start, end) = (old.span.start as usize, old.span.end as usize);
    let blocks = blocks_in(source, start, end);
    let wanted = 1 + a.else_if_branches.len() + usize::from(a.else_body.is_some());
    if blocks.len() != wanted {
        return false;
    }
    let depth = depth_at(source, start) + 1;
    let head = |i: &IfStmt| match &i.binding {
        Some(name) => format!("if let {name} = {}", print::expr(&i.condition)),
        None => format!("if {}", print::expr(&i.condition)),
    };
    if head(a) != head(b) {
        out.push(Patch::new(start, blocks[0].0, format!("{} ", head(b))));
    }
    diff_body(
        source,
        &a.then_body,
        &b.then_body,
        (blocks[0].0 + 1, blocks[0].1),
        block,
        depth,
        out,
    );
    for (k, ((ca, ba), (cb, bb))) in a
        .else_if_branches
        .iter()
        .zip(&b.else_if_branches)
        .enumerate()
    {
        let (prev_close, (open, close)) = (blocks[k].1, blocks[k + 1]);
        if !same(ca, cb) {
            out.push(Patch::new(
                prev_close + 1,
                open,
                format!(" else if {} ", print::expr(cb)),
            ));
        }
        diff_body(source, ba, bb, (open + 1, close), block, depth, out);
    }
    if let (Some(ea), Some(eb)) = (&a.else_body, &b.else_body) {
        let (open, close) = blocks[blocks.len() - 1];
        diff_body(source, ea, eb, (open + 1, close), block, depth, out);
    }
    true
}

fn diff_match(
    source: &str,
    old: &Statement,
    a: &MatchStmt,
    b: &MatchStmt,
    out: &mut Vec<Patch>,
) -> bool {
    if a.arms.len() != b.arms.len()
        || a.arms.iter().zip(&b.arms).any(|(x, y)| {
            x.pattern != y.pattern || x.binding != y.binding || x.bindings != y.bindings
        })
    {
        return false;
    }
    let (start, end) = (old.span.start as usize, old.span.end as usize);
    let Some(&(open, _)) = blocks_in(source, start, end).first() else {
        return false;
    };
    if !same(&a.scrutinee, &b.scrutinee) {
        out.push(Patch::new(
            start,
            open,
            format!("match {} ", print::expr(&b.scrutinee)),
        ));
    }
    for (x, y) in a.arms.iter().zip(&b.arms) {
        let (s, e) = (x.span.start as usize, x.span.end as usize);
        let Some(&(o, c)) = blocks_in(source, s, e).first() else {
            return false;
        };
        diff_body(
            source,
            &x.body,
            &y.body,
            (o + 1, c),
            Block::Render,
            depth_at(source, s) + 1,
            out,
        );
    }
    true
}

fn diff_handler(source: &str, a: &EventHandler, b: &EventHandler, out: &mut Vec<Patch>) -> bool {
    let (start, end) = (a.span.start as usize, a.span.end as usize);
    if a.event != b.event || a.param != b.param || a.key != b.key {
        out.push(Patch::new(
            start,
            end,
            print::handler(b, depth_at(source, start)),
        ));
        return true;
    }
    let Some(&(open, close)) = blocks_in(source, start, end).first() else {
        return false;
    };
    diff_body(
        source,
        &a.body,
        &b.body,
        (open + 1, close),
        Block::Imperative,
        depth_at(source, start) + 1,
        out,
    );
    true
}

// ─── Blocks of statements ────────────────────────────────

/// Statements as the source holds them: a destructuring `let` is several
/// over one span, and one unit here.
fn units(stmts: &[Statement]) -> Vec<&[Statement]> {
    let mut out: Vec<&[Statement]> = Vec::new();
    let mut i = 0;
    while i < stmts.len() {
        let span = stmts[i].span;
        let mut j = i + 1;
        while j < stmts.len() && stmts[j].span == span && span.end > span.start {
            j += 1;
        }
        out.push(&stmts[i..j]);
        i = j;
    }
    out
}

fn unit_text(unit: &[Statement], block: Block, depth: usize) -> String {
    match print::destructure(unit) {
        Some((text, n)) if n == unit.len() => text,
        _ => print::statements(unit, block, depth).join(&format!("\n{}", pad(depth))),
    }
}

/// What turns the block `old` into `new`. `interior` is the block between
/// its braces, and `depth` the depth of its statements.
pub fn body(
    source: &str,
    old: &[Statement],
    new: &[Statement],
    interior: (usize, usize),
    block: Block,
) -> Vec<Patch> {
    let depth = old
        .first()
        .map(|s| depth_at(source, s.span.start as usize))
        .unwrap_or_else(|| depth_at(source, interior.0) + 1);
    let mut out = Vec::new();
    diff_body(source, old, new, interior, block, depth, &mut out);
    out
}

fn diff_body(
    source: &str,
    old: &[Statement],
    new: &[Statement],
    interior: (usize, usize),
    block: Block,
    depth: usize,
    out: &mut Vec<Patch>,
) {
    if same(&old, &new) {
        return;
    }
    let old_units = units(old);
    let new_units = new.iter().map(std::slice::from_ref).collect::<Vec<_>>();
    // A new destructuring `let` is one unit too.
    let new_units = regroup(new, new_units);
    let depth = old_units
        .first()
        .map(|u| depth_at(source, u[0].span.start as usize))
        .unwrap_or(depth);
    let pairs = lcs(&old_units, &new_units);

    // Walk the two in step: between two units that match, what is left of
    // the old is changed into what is left of the new, one for one, and
    // the rest removed or added.
    let mut inserts: Vec<(usize, Vec<String>)> = Vec::new();
    let (mut oi, mut ni) = (0, 0);
    let mut anchor: Option<usize> = None;
    for &(om, nm) in pairs
        .iter()
        .chain(std::iter::once(&(old_units.len(), new_units.len())))
    {
        let old_gap = &old_units[oi..om];
        let new_gap = &new_units[ni..nm];
        let paired = old_gap.len().min(new_gap.len());
        for k in 0..paired {
            let (o, n) = (old_gap[k], new_gap[k]);
            if o.len() == 1 && n.len() == 1 {
                diff_statement(source, &o[0], &n[0], block, out);
            } else {
                out.push(Patch::new(
                    written_start(source, &o[0]),
                    o[0].span.end as usize,
                    unit_text(n, block, depth),
                ));
            }
            anchor = Some(o[0].span.end as usize);
        }
        for o in &old_gap[paired..] {
            // With its `///`, which would otherwise document the next one.
            out.push(removal(
                source,
                written_start(source, &o[0]),
                o[0].span.end as usize,
            ));
        }
        if new_gap.len() > paired {
            let texts: Vec<String> = new_gap[paired..]
                .iter()
                .map(|n| unit_text(n, block, depth))
                .collect();
            // After the last old statement before here that stays, or at
            // the front of the block.
            let at = anchor.or_else(|| old_gap.get(paired).map(|_| usize::MAX));
            inserts.push((at.unwrap_or(usize::MAX), texts));
        }
        if om < old_units.len() {
            anchor = Some(old_units[om][0].span.end as usize);
        }
        oi = om + 1;
        ni = nm + 1;
    }

    for (at, texts) in inserts {
        if at != usize::MAX {
            let at = end_of_line(source, at);
            let text: String = texts
                .iter()
                .map(|t| format!("\n{}{t}", pad(depth)))
                .collect();
            out.push(Patch::new(at, at, text));
            continue;
        }
        // At the front: before the first statement there is, or into an
        // empty block.
        match old_units.first() {
            Some(first) => {
                let at = written_start(source, &first[0]);
                let text: String = texts
                    .iter()
                    .map(|t| format!("{t}\n{}", pad(depth)))
                    .collect();
                out.push(Patch::new(at, at, text));
            }
            None => {
                let (start, end) = interior;
                let inside = &source[start..end];
                if inside.trim().is_empty() {
                    let lines: String = texts
                        .iter()
                        .map(|t| format!("\n{}{t}", pad(depth)))
                        .collect();
                    out.push(Patch::new(
                        start,
                        end,
                        format!("{lines}\n{}", pad(depth.saturating_sub(1))),
                    ));
                } else {
                    // Only comments are there: after them.
                    let at = start + inside.trim_end().len();
                    let text: String = texts
                        .iter()
                        .map(|t| format!("\n{}{t}", pad(depth)))
                        .collect();
                    out.push(Patch::new(at, at, text));
                }
            }
        }
    }
}

/// `new`'s statements as units, a destructuring `let` the printer would
/// write as one kept together.
fn regroup<'a>(new: &'a [Statement], singles: Vec<&'a [Statement]>) -> Vec<&'a [Statement]> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < new.len() {
        match print::destructure(&new[i..]) {
            Some((_, n)) => {
                out.push(&new[i..i + n]);
                i += n;
            }
            None => {
                out.push(singles[i]);
                i += 1;
            }
        }
    }
    out
}

/// The longest run of units the two blocks have in common, in order, as
/// pairs of indices.
fn lcs(a: &[&[Statement]], b: &[&[Statement]]) -> Vec<(usize, usize)> {
    let sa: Vec<String> = a.iter().map(shape_of).collect();
    let sb: Vec<String> = b.iter().map(shape_of).collect();
    let (n, m) = (sa.len(), sb.len());
    let mut table = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            table[i][j] = if sa[i] == sb[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if sa[i] == sb[j] {
            out.push((i, j));
            i += 1;
            j += 1;
        } else if table[i + 1][j] >= table[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

/// Where a line that comes after the statement ending at `at` goes: after
/// the comment that ends its line, when one does — the comment is the
/// statement's.
fn end_of_line(source: &str, at: usize) -> usize {
    let line_end = source[at..].find('\n').map_or(source.len(), |n| at + n);
    let rest = source[at..line_end].trim();
    if rest.is_empty() || rest.starts_with("//") {
        line_end
    } else {
        at
    }
}

/// Taking `start..end` out, and its line with it when nothing else is on
/// that line but a comment of its own.
fn removal(source: &str, start: usize, end: usize) -> Patch {
    let line_start = source[..start].rfind('\n').map_or(0, |n| n + 1);
    let line_end = source[end..].find('\n').map_or(source.len(), |n| end + n);
    let rest = source[end..line_end].trim();
    let alone =
        source[line_start..start].trim().is_empty() && (rest.is_empty() || rest.starts_with("//"));
    if alone {
        // The line and its newline; the last line of the file, the newline
        // before it.
        if line_end < source.len() {
            Patch::new(line_start, line_end + 1, "")
        } else {
            Patch::new(line_start.saturating_sub(1), line_end, "")
        }
    } else {
        Patch::new(start, end, "")
    }
}

// ─── Elements ────────────────────────────────────────────

/// What turns the element `old` into `new`, when it can be said piece by
/// piece; `false` when the element must be printed whole.
fn diff_element(source: &str, a: &UIElement, b: &UIElement, out: &mut Vec<Patch>) -> bool {
    if !same(&a.component, &b.component) {
        return false;
    }
    if matches!(&a.component, ComponentRef::BuiltIn(n) if n == "Children" || n.starts_with("__")) {
        return false;
    }
    let start = a.span.start as usize;
    let name = print::component(&a.component);
    if !source[start..].starts_with(&name) {
        return false;
    }
    let name_end = start + name.len();
    let depth = depth_at(source, start);

    // The arguments.
    if !same(&a.args, &b.args) {
        let inside_parens = a.paren_span.is_some_and(|p| {
            a.arg_spans
                .iter()
                .all(|s| s.start >= p.start && s.end <= p.end)
        });
        let aligned = a.args.len() == b.args.len()
            && a.arg_spans.len() == a.args.len()
            && inside_parens
            && a.args.iter().zip(&b.args).all(|(x, y)| match (x, y) {
                (Arg::Positional(_), Arg::Positional(_)) => true,
                (Arg::Named(k1, _), Arg::Named(k2, _)) => k1 == k2,
                _ => false,
            });
        if aligned {
            for (i, (x, y)) in a.args.iter().zip(&b.args).enumerate() {
                if !same(x, y) {
                    let s = a.arg_spans[i];
                    out.push(Patch::new(s.start as usize, s.end as usize, print::arg(y)));
                }
            }
        } else {
            let text = if b.args.is_empty() {
                String::new()
            } else {
                format!(
                    "({})",
                    b.args.iter().map(print::arg).collect::<Vec<_>>().join(", ")
                )
            };
            match a.paren_span {
                Some(p) if inside_parens => {
                    out.push(Patch::new(p.start as usize, p.end as usize, text))
                }
                None if a.args.is_empty() => out.push(Patch::new(name_end, name_end, text)),
                _ => return false,
            }
        }
    }

    // The flags.
    let header_end = a
        .modifier_spans
        .last()
        .map(|s| s.end as usize)
        .or(a.paren_span.map(|p| p.end as usize))
        .unwrap_or(name_end);
    if a.modifiers != b.modifiers {
        if a.modifiers.len() == b.modifiers.len() && a.modifier_spans.len() == a.modifiers.len() {
            for (i, (x, y)) in a.modifiers.iter().zip(&b.modifiers).enumerate() {
                if x != y {
                    let s = a.modifier_spans[i];
                    out.push(Patch::new(
                        s.start as usize,
                        s.end as usize,
                        format!(".{y}"),
                    ));
                }
            }
        } else {
            let text: String = b.modifiers.iter().map(|m| format!(".{m}")).collect();
            match (a.modifier_spans.first(), a.modifier_spans.last()) {
                (Some(f), Some(l)) => out.push(Patch::new(f.start as usize, l.end as usize, text)),
                _ => out.push(Patch::new(header_end, header_end, text)),
            }
        }
    }

    // The block.
    let has_block = |e: &UIElement| {
        e.style_block.is_some()
            || e.transition_block.is_some()
            || !e.events.is_empty()
            || !e.slot_fills.is_empty()
            || !e.children.is_empty()
    };
    let Some(interior) = a.body_span.map(|s| (s.start as usize, s.end as usize)) else {
        if has_block(b) {
            // A block where there was none: written after the call.
            let printed = print::element(b, depth);
            let call = print::call(b);
            out.push(Patch::new(
                header_end,
                header_end,
                printed[call.len()..].to_string(),
            ));
        }
        return true;
    };
    diff_element_block(source, a, b, interior, depth + 1, out)
}

/// One item of an element's block, where it is written.
fn item_spans(a: &UIElement) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    if let Some(s) = a.style_span {
        spans.push((s.start as usize, s.end as usize));
    }
    if let Some(t) = &a.transition_block {
        spans.push((t.span.start as usize, t.span.end as usize));
    }
    spans.extend(
        a.events
            .iter()
            .map(|h| (h.span.start as usize, h.span.end as usize)),
    );
    spans.extend(
        a.slot_fills
            .iter()
            .map(|f| (f.span.start as usize, f.span.end as usize)),
    );
    spans
}

fn diff_element_block(
    source: &str,
    a: &UIElement,
    b: &UIElement,
    interior: (usize, usize),
    depth: usize,
    out: &mut Vec<Patch>,
) -> bool {
    // Where something new before the children goes: after what is there.
    let mut cursor = interior.0;
    let mut ahead: Vec<String> = Vec::new();
    let flush = |cursor: usize, ahead: &mut Vec<String>, out: &mut Vec<Patch>| {
        if !ahead.is_empty() {
            let text: String = ahead
                .drain(..)
                .map(|t| format!("\n{}{t}", pad(depth)))
                .collect();
            let at = end_of_line(source, cursor);
            out.push(Patch::new(at, at, text));
        }
    };

    // The style block.
    match (&a.style_block, &b.style_block, a.style_span) {
        (Some(x), Some(y), Some(span)) => {
            if !same(x, y) {
                diff_style(
                    source,
                    x,
                    y,
                    (span.start as usize, span.end as usize),
                    depth,
                    out,
                );
            }
            cursor = span.end as usize;
        }
        (Some(_), None, Some(span)) => {
            out.push(removal(source, span.start as usize, span.end as usize));
            cursor = span.end as usize;
        }
        (None, Some(y), _) => ahead.push(print::style_block(y, depth)),
        (None, None, _) => {}
        _ => return false,
    }

    // The transition.
    match (&a.transition_block, &b.transition_block) {
        (Some(x), Some(y)) => {
            flush(cursor, &mut ahead, out);
            if !same(x, y) {
                out.push(Patch::new(
                    x.span.start as usize,
                    x.span.end as usize,
                    print::transition(y, depth),
                ));
            }
            cursor = x.span.end as usize;
        }
        (Some(x), None) => {
            flush(cursor, &mut ahead, out);
            out.push(removal(source, x.span.start as usize, x.span.end as usize));
            cursor = x.span.end as usize;
        }
        (None, Some(y)) => ahead.push(print::transition(y, depth)),
        (None, None) => {}
    }

    // The handlers and the fills: the same ones, each compared on its own;
    // otherwise all of them again.
    let same_handlers = a.events.len() == b.events.len();
    if same_handlers {
        if !a.events.is_empty() {
            flush(cursor, &mut ahead, out);
        }
        for (x, y) in a.events.iter().zip(&b.events) {
            if !same(x, y) && !diff_handler(source, x, y, out) {
                return false;
            }
            cursor = x.span.end as usize;
        }
    } else {
        flush(cursor, &mut ahead, out);
        for x in &a.events {
            out.push(removal(source, x.span.start as usize, x.span.end as usize));
            cursor = x.span.end as usize;
        }
        ahead.extend(b.events.iter().map(|h| print::handler(h, depth)));
    }
    let same_fills = a.slot_fills.len() == b.slot_fills.len()
        && a.slot_fills
            .iter()
            .zip(&b.slot_fills)
            .all(|(x, y)| x.name == y.name && x.params == y.params);
    if same_fills {
        if !a.slot_fills.is_empty() {
            flush(cursor, &mut ahead, out);
        }
        for (x, y) in a.slot_fills.iter().zip(&b.slot_fills) {
            let interior = (x.body_span.start as usize, x.body_span.end as usize);
            diff_body(
                source,
                &x.body,
                &y.body,
                interior,
                Block::Render,
                depth + 1,
                out,
            );
            cursor = x.span.end as usize;
        }
    } else {
        flush(cursor, &mut ahead, out);
        for x in &a.slot_fills {
            out.push(removal(source, x.span.start as usize, x.span.end as usize));
            cursor = x.span.end as usize;
        }
        ahead.extend(b.slot_fills.iter().map(|f| print::slot_fill(f, depth)));
    }

    // The children, after whatever comes before them.
    if a.children.is_empty() {
        let inside = &source[interior.0..interior.1];
        let empty = item_spans(a).is_empty() && inside.trim().is_empty();
        let mut texts: Vec<String> = std::mem::take(&mut ahead);
        texts.extend(print::statements(&b.children, Block::Render, depth));
        if texts.is_empty() {
            return true;
        }
        let lines: String = texts
            .iter()
            .map(|t| format!("\n{}{t}", pad(depth)))
            .collect();
        if empty {
            // `Card {}` becomes a block of lines.
            out.push(Patch::new(
                interior.0,
                interior.1,
                format!("{lines}\n{}", pad(depth - 1)),
            ));
        } else {
            // After the last thing in the block — an item, or a comment.
            let at = if item_spans(a).is_empty() {
                interior.0 + inside.trim_end().len()
            } else {
                cursor
            };
            let at = end_of_line(source, at);
            out.push(Patch::new(at, at, lines));
        }
    } else {
        flush(cursor, &mut ahead, out);
        diff_body(
            source,
            &a.children,
            &b.children,
            (cursor, interior.1),
            Block::Render,
            depth,
            out,
        );
    }
    true
}

/// A style block's declarations, each compared on its own: a value that
/// changed is written in its place, a declaration added goes after the
/// last, one removed takes its line. A nested rule that changed is the
/// whole block again.
fn diff_style(
    source: &str,
    a: &StyleBlock,
    b: &StyleBlock,
    span: (usize, usize),
    depth: usize,
    out: &mut Vec<Patch>,
) {
    let whole =
        |out: &mut Vec<Patch>| out.push(Patch::new(span.0, span.1, print::style_block(b, depth)));
    if !same(&a.media_queries, &b.media_queries) || !same(&a.pseudo_blocks, &b.pseudo_blocks) {
        return whole(out);
    }
    let names_a: Vec<&str> = a.properties.iter().map(|p| p.name.as_str()).collect();
    let names_b: Vec<&str> = b.properties.iter().map(|p| p.name.as_str()).collect();
    // The same declarations in the same order, some with new values.
    if names_a == names_b {
        for (x, y) in a.properties.iter().zip(&b.properties) {
            if !same(&x.value, &y.value) {
                out.push(Patch::new(
                    x.value_span.start as usize,
                    x.value_span.end as usize,
                    print::style_value(&y.value),
                ));
            }
        }
        return;
    }
    // Declarations added at the end, or taken out, the rest as they were.
    let kept: Vec<&str> = names_b
        .iter()
        .copied()
        .filter(|n| names_a.contains(n))
        .collect();
    let still: Vec<&str> = names_a
        .iter()
        .copied()
        .filter(|n| names_b.contains(n))
        .collect();
    if kept != still
        || names_a
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            != names_a.len()
    {
        return whole(out);
    }
    let mut last_end = None;
    for x in &a.properties {
        match b.properties.iter().find(|y| y.name == x.name) {
            Some(y) => {
                if !same(&x.value, &y.value) {
                    out.push(Patch::new(
                        x.value_span.start as usize,
                        x.value_span.end as usize,
                        print::style_value(&y.value),
                    ));
                }
                last_end = Some(x.span.end as usize);
            }
            None => out.push(removal(source, x.span.start as usize, x.span.end as usize)),
        }
    }
    // New ones go where the order puts them; written after the last kept
    // when they come after it, otherwise the whole block again.
    let new_ones: Vec<&StyleProperty> = b
        .properties
        .iter()
        .filter(|y| !names_a.contains(&y.name.as_str()))
        .collect();
    if new_ones.is_empty() {
        return;
    }
    let first_new = b
        .properties
        .iter()
        .position(|y| !names_a.contains(&y.name.as_str()))
        .unwrap_or(0);
    let last_kept = b
        .properties
        .iter()
        .rposition(|y| names_a.contains(&y.name.as_str()));
    if last_kept.is_some_and(|k| k > first_new) {
        out.retain(|p| p.start < span.0 || p.end > span.1);
        return whole(out);
    }
    let text: String = new_ones
        .iter()
        .map(|p| format!("\n{}{}", pad(depth + 1), print::style_property(p)))
        .collect();
    match last_end {
        Some(at) if a.media_queries.is_empty() && a.pseudo_blocks.is_empty() => {
            let at = end_of_line(source, at);
            out.push(Patch::new(at, at, text))
        }
        Some(_) | None => {
            out.retain(|p| p.start < span.0 || p.end > span.1);
            whole(out)
        }
    }
}

// ─── Declarations ────────────────────────────────────────

/// One member of a declaration as the source holds it — a prop, an event,
/// a field, a token — where it is written (from its `///`), and how it
/// would be printed now.
struct Item {
    start: usize,
    end: usize,
    text: String,
}

/// Where a member written at `start..end` begins: its `///` above it, from
/// the start of that line, when nothing else is before it on its line.
fn item_start(source: &str, start: usize) -> usize {
    let line_start = source[..start].rfind('\n').map_or(0, |n| n + 1);
    if source[line_start..start].trim().is_empty() {
        print::doc_start(source, start)
    } else {
        start
    }
}

/// What turns the members `old` into `new` (each new one already printed,
/// its `///` with it, at `depth`). A new member with nothing before it goes
/// at `front` — right after a block's `{`.
fn diff_items(
    source: &str,
    old: &[Item],
    new: &[String],
    front: usize,
    depth: usize,
    out: &mut Vec<Patch>,
) {
    let olds: Vec<&str> = old.iter().map(|i| i.text.as_str()).collect();
    let news: Vec<&str> = new.iter().map(String::as_str).collect();
    let pairs = lcs_by(&olds, &news);
    let (mut oi, mut ni) = (0, 0);
    let mut anchor: Option<usize> = None;
    let line = |text: &str| format!("\n{}", indent_text(text, depth));
    for &(om, nm) in pairs.iter().chain(std::iter::once(&(old.len(), new.len()))) {
        let og = &old[oi..om];
        let ng = &new[ni..nm];
        let paired = og.len().min(ng.len());
        for k in 0..paired {
            let o = &og[k];
            let at_line_start = o.start == 0 || source.as_bytes()[o.start - 1] == b'\n';
            let text = if at_line_start {
                indent_text(&ng[k], depth)
            } else {
                ng[k].trim_start().to_string()
            };
            out.push(Patch::new(o.start, o.end, text));
            anchor = Some(o.end);
        }
        for o in &og[paired..] {
            out.push(removal(source, o.start, o.end));
        }
        if ng.len() > paired {
            let at = end_of_line(source, anchor.unwrap_or(front));
            let text: String = ng[paired..].iter().map(|t| line(t)).collect();
            out.push(Patch::new(at, at, text));
        }
        if om < old.len() {
            anchor = Some(old[om].end);
        }
        oi = om + 1;
        ni = nm + 1;
    }
}

/// `text`, whose lines after the first are already at `depth` (as the
/// printer writes them), with its first line — and a `///` before it — at
/// `depth` too.
fn indent_text(text: &str, depth: usize) -> String {
    let mut out = String::new();
    for (i, l) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if i == 0 || l.trim_start().starts_with("///") && !l.starts_with(&pad(depth)) {
            out.push_str(&pad(depth));
            out.push_str(l.trim_start());
        } else {
            out.push_str(l);
        }
    }
    out
}

fn lcs_by(a: &[&str], b: &[&str]) -> Vec<(usize, usize)> {
    let (n, m) = (a.len(), b.len());
    let mut table = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            table[i][j] = if a[i] == b[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if a[i] == b[j] {
            out.push((i, j));
            i += 1;
            j += 1;
        } else if table[i + 1][j] >= table[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

fn with_doc(doc: &Option<String>, text: &str, depth: usize) -> String {
    print::documented(doc, text, depth)
}

fn parts_of<'a>(program: &'a Program, owner: &str) -> Vec<&'a ComponentDecl> {
    program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c)
                if c.name.rsplit_once('.').is_some_and(|(o, _)| o == owner) =>
            {
                Some(c)
            }
            _ => None,
        })
        .collect()
}

/// What a declaration is as a whole: its own shape, and its parts'.
fn declaration_key(d: &Declaration, program: &Program) -> String {
    let mut key = shape_of(d);
    if let Declaration::Component(c) = d {
        for p in parts_of(program, &c.name) {
            key.push_str(&shape_of(p));
        }
    }
    key
}

/// The declarations of a file, as a reader of the file sees them: a part is
/// its component's, and a project script is not source.
fn top_level(program: &Program) -> Vec<&Declaration> {
    program
        .declarations
        .iter()
        .filter(|d| {
            !matches!(d, Declaration::Component(c) if c.name.contains('.'))
                && !matches!(d, Declaration::Script(_))
        })
        .collect()
}

/// What turns a file whose tree is `old` into one whose tree is `new`: a
/// declaration added after the one before it, one removed with its `///`
/// and the blank line after it, and one changed in the narrowest piece
/// that says so.
pub fn program(source: &str, old: &Program, new: &Program) -> Vec<Patch> {
    let mut out = Vec::new();
    let a = top_level(old);
    let b = top_level(new);
    let ka: Vec<String> = a.iter().map(|d| declaration_key(d, old)).collect();
    let kb: Vec<String> = b.iter().map(|d| declaration_key(d, new)).collect();
    let ra: Vec<&str> = ka.iter().map(String::as_str).collect();
    let rb: Vec<&str> = kb.iter().map(String::as_str).collect();
    let pairs = lcs_by(&ra, &rb);
    let (mut oi, mut ni) = (0, 0);
    let mut anchor: Option<usize> = None;
    let mut front: Vec<String> = Vec::new();
    for &(om, nm) in pairs.iter().chain(std::iter::once(&(a.len(), b.len()))) {
        let og = &a[oi..om];
        let ng = &b[ni..nm];
        let paired = og.len().min(ng.len());
        for k in 0..paired {
            if !diff_declaration(source, og[k], ng[k], old, new, &mut out)
                && let Some((s, e)) = print::declaration_span(source, og[k])
            {
                out.push(Patch::new(s, e, print::declaration(ng[k], new)));
            }
            anchor = print::declaration_span(source, og[k])
                .map(|s| s.1)
                .or(anchor);
        }
        for o in &og[paired..] {
            if let Some((s, e)) = print::declaration_span(source, o) {
                // The declaration, and the blank line that set it apart.
                let mut p = removal(source, s, e);
                if source[p.end..].starts_with('\n') {
                    p.end += 1;
                } else if p.start > 0 && source[..p.start].ends_with("\n\n") {
                    p.start -= 1;
                }
                out.push(p);
            }
        }
        let texts: Vec<String> = ng[paired..]
            .iter()
            .map(|d| print::declaration(d, new))
            .collect();
        if !texts.is_empty() {
            match anchor {
                Some(at) => {
                    let text: String = texts.iter().map(|t| format!("\n\n{t}")).collect();
                    out.push(Patch::new(at, at, text));
                }
                None => front.extend(texts),
            }
        }
        if om < a.len() {
            anchor = print::declaration_span(source, a[om])
                .map(|s| s.1)
                .or(anchor);
        }
        oi = om + 1;
        ni = nm + 1;
    }
    if !front.is_empty() {
        let first = a
            .iter()
            .find_map(|d| print::declaration_span(source, d))
            .map(|s| s.0);
        match first {
            Some(at) => out.push(Patch::new(at, at, format!("{}\n\n", front.join("\n\n")))),
            None => {
                let sep = if source.trim().is_empty() { "" } else { "\n\n" };
                let at = source.trim_end().len();
                out.push(Patch::new(
                    at,
                    source.len(),
                    format!("{sep}{}\n", front.join("\n\n")),
                ));
            }
        }
    }
    out
}

/// A declaration changed in place: its `///`, its header, its members and
/// its body each compared on their own. `false` when it must be printed
/// whole.
fn diff_declaration(
    source: &str,
    a: &Declaration,
    b: &Declaration,
    pa: &Program,
    pb: &Program,
    out: &mut Vec<Patch>,
) -> bool {
    let before = out.len();
    let done = match (a, b) {
        (Declaration::Component(x), Declaration::Component(y)) => {
            diff_component(source, x, y, pa, pb, out)
        }
        (Declaration::Page(x), Declaration::Page(y)) => diff_page(source, x, y, out),
        (Declaration::Store(x), Declaration::Store(y)) => {
            if print::store_header(x) != print::store_header(y) {
                out.push(Patch::new(
                    x.header_span.start as usize,
                    x.header_span.end as usize,
                    print::store_header(y),
                ));
            }
            diff_body(
                source,
                &x.body,
                &y.body,
                (x.body_span.start as usize, x.body_span.end as usize),
                Block::Render,
                depth_at(source, x.span.start as usize) + 1,
                out,
            );
            true
        }
        (Declaration::Type(x), Declaration::Type(y)) => {
            if !same(&x.doc, &y.doc) {
                diff_doc(source, x.span.start as usize, &y.doc, out);
            }
            if print::type_header(x) != print::type_header(y) {
                out.push(Patch::new(
                    x.header_span.start as usize,
                    x.header_span.end as usize,
                    print::type_header(y),
                ));
            }
            let (s, e) = (x.span.start as usize, x.span.end as usize);
            match blocks_in(source, s, e).first() {
                Some(&(open, _)) => {
                    let depth = depth_at(source, s) + 1;
                    let old: Vec<Item> = x
                        .fields
                        .iter()
                        .map(|f| Item {
                            start: item_start(source, f.span.start as usize),
                            end: f.span.end as usize,
                            text: with_doc(&f.doc, &print::field(f), depth),
                        })
                        .collect();
                    let new: Vec<String> = y
                        .fields
                        .iter()
                        .map(|f| with_doc(&f.doc, &print::field(f), depth))
                        .collect();
                    diff_items(source, &old, &new, open + 1, depth, out);
                    true
                }
                None => false,
            }
        }
        (Declaration::Theme(x), Declaration::Theme(y)) if x.name == y.name => {
            let (s, e) = (x.span.start as usize, x.span.end as usize);
            match blocks_in(source, s, e).first() {
                Some(&(open, _)) => {
                    let depth = depth_at(source, s) + 1;
                    let old: Vec<Item> = x
                        .tokens
                        .iter()
                        .map(|t| Item {
                            start: item_start(source, t.span.start as usize),
                            end: t.span.end as usize,
                            text: print::token(t),
                        })
                        .collect();
                    let new: Vec<String> = y.tokens.iter().map(print::token).collect();
                    diff_items(source, &old, &new, open + 1, depth, out);
                    true
                }
                None => false,
            }
        }
        _ => false,
    };
    if !done {
        out.truncate(before);
    }
    done
}

/// The `///` above the declaration at `start` made `doc`.
fn diff_doc(source: &str, start: usize, doc: &Option<String>, out: &mut Vec<Patch>) {
    let line_start = source[..start].rfind('\n').map_or(0, |n| n + 1);
    let from = print::doc_start(source, start);
    out.push(Patch::new(
        from,
        line_start,
        print::doc(doc, depth_at(source, start)),
    ));
}

fn diff_component(
    source: &str,
    x: &ComponentDecl,
    y: &ComponentDecl,
    pa: &Program,
    pb: &Program,
    out: &mut Vec<Patch>,
) -> bool {
    if x.name != y.name {
        return false;
    }
    let start = x.span.start as usize;
    let depth = depth_at(source, start);
    if !same(&x.doc, &y.doc) {
        diff_doc(source, start, &y.doc, out);
    }
    // The header: one prop at a time where the list is the same props,
    // each with the `///` it had; otherwise the header again.
    if !same(&x.props, &y.props) {
        let aligned = x.props.len() == y.props.len()
            && x.props
                .iter()
                .zip(&y.props)
                .all(|(p, q)| p.name == q.name && p.doc == q.doc);
        if aligned {
            for (p, q) in x.props.iter().zip(&y.props) {
                if !same(p, q) {
                    out.push(Patch::new(
                        p.span.start as usize,
                        p.span.end as usize,
                        print::prop(q),
                    ));
                }
            }
        } else {
            out.push(Patch::new(
                x.header_span.start as usize,
                x.header_span.end as usize,
                print::component_header(y, depth),
            ));
        }
    }
    let interior = (x.body_span.start as usize, x.body_span.end as usize);
    let inner = depth + 1;
    // The events, the slots and the parts, each a list of its own.
    let events_old: Vec<Item> = x
        .events
        .iter()
        .map(|e| Item {
            start: item_start(source, e.span.start as usize),
            end: e.span.end as usize,
            text: with_doc(&e.doc, &print::event_decl(e), inner),
        })
        .collect();
    let events_new: Vec<String> = y
        .events
        .iter()
        .map(|e| with_doc(&e.doc, &print::event_decl(e), inner))
        .collect();
    if events_old
        .iter()
        .map(|i| i.text.as_str())
        .ne(events_new.iter().map(String::as_str))
    {
        diff_items(source, &events_old, &events_new, interior.0, inner, out);
    }
    let slots_old: Vec<Item> = x
        .slots
        .iter()
        .map(|s| Item {
            start: s.span.start as usize,
            end: s.span.end as usize,
            text: print::slot_decl(s),
        })
        .collect();
    let slots_new: Vec<String> = y.slots.iter().map(print::slot_decl).collect();
    if slots_old
        .iter()
        .map(|i| i.text.as_str())
        .ne(slots_new.iter().map(String::as_str))
    {
        let front = events_old.last().map(|i| i.end).unwrap_or(interior.0);
        diff_items(source, &slots_old, &slots_new, front, inner, out);
    }
    let parts_a = parts_of(pa, &x.name);
    let parts_b = parts_of(pb, &y.name);
    let part_text =
        |p: &ComponentDecl| with_doc(&p.doc, &print::component_decl(p, &[], inner), inner);
    let parts_old: Vec<Item> = x
        .parts
        .iter()
        .filter_map(|n| parts_a.iter().find(|p| p.name == format!("{}.{n}", x.name)))
        .map(|p| Item {
            start: item_start(source, p.span.start as usize),
            end: p.span.end as usize,
            text: part_text(p),
        })
        .collect();
    let parts_new: Vec<String> = y
        .parts
        .iter()
        .filter_map(|n| parts_b.iter().find(|p| p.name == format!("{}.{n}", y.name)))
        .map(|p| part_text(p))
        .collect();
    if parts_old
        .iter()
        .map(|i| i.text.as_str())
        .ne(parts_new.iter().map(String::as_str))
    {
        let front = slots_old
            .last()
            .map(|i| i.end)
            .or(events_old.last().map(|i| i.end))
            .unwrap_or(interior.0);
        diff_items(source, &parts_old, &parts_new, front, inner, out);
    }
    diff_body(
        source,
        &x.body,
        &y.body,
        interior,
        Block::Render,
        inner,
        out,
    );
    true
}

fn diff_page(source: &str, x: &PageDecl, y: &PageDecl, out: &mut Vec<Patch>) -> bool {
    if print::page_header(x) != print::page_header(y) {
        out.push(Patch::new(
            x.header_span.start as usize,
            x.header_span.end as usize,
            print::page_header(y),
        ));
    }
    let interior = (x.body_span.start as usize, x.body_span.end as usize);
    let depth = depth_at(source, x.span.start as usize) + 1;
    if !same(&x.head, &y.head) {
        let tag = |t: &HeadTag| {
            format!(
                "{}({})",
                t.tag,
                t.attrs
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", print::expr(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        if x.head.is_empty() {
            // A head where there was none: first in the body.
            let tags: Vec<String> = y.head.iter().map(tag).collect();
            let block = format!(
                "head {{\n{}\n{}}}",
                tags.iter()
                    .map(|t| format!("{}{t}", pad(depth + 1)))
                    .collect::<Vec<_>>()
                    .join("\n"),
                pad(depth)
            );
            let at = end_of_line(source, interior.0);
            out.push(Patch::new(at, at, format!("\n{}{block}", pad(depth))));
        } else {
            // The `{ … }` the tags are in.
            let first = x.head[0].span.start as usize;
            let Some(word) = source[interior.0..first].rfind("head") else {
                return false;
            };
            let head_start = interior.0 + word;
            let Some(&(open, close)) = blocks_in(source, head_start, interior.1).first() else {
                return false;
            };
            if y.head.is_empty() {
                out.push(removal(source, head_start, close + 1));
            } else {
                let old: Vec<Item> = x
                    .head
                    .iter()
                    .map(|t| Item {
                        start: t.span.start as usize,
                        end: t.span.end as usize,
                        text: tag(t),
                    })
                    .collect();
                let new: Vec<String> = y.head.iter().map(tag).collect();
                diff_items(source, &old, &new, open + 1, depth + 1, out);
            }
        }
    }
    diff_body(
        source,
        &x.body,
        &y.body,
        interior,
        Block::Render,
        depth,
        out,
    );
    true
}

#[cfg(test)]
mod tests {
    use crate::edit::{rewrite_body, rewrite_element, rewrite_statement};
    use crate::parser::ast::*;

    const PAGE: &str = r#"page Home(path: "/", title: "Home", description: "Home.") {
    // The deploys, newest first.
    Stack(gap: .md) {
        Heading("Deploys").h2   // the title
        Row(gap: .sm,   align: .center) {
            Badge("live").success
        }
        Card {
            style {
                padding: 12px   16px
                color: $text
            }
            Text("Last deploy")
        }
        Spacer
    }
}
"#;

    fn id_of(source: &str, needle: &str) -> String {
        let program = crate::syntax::parse_source(source, "t.wf").unwrap();
        let mut found = None;
        crate::codegen::node_id::visit_nodes(&program, &mut |ui, id, _| {
            if found.is_none() && source[ui.span.start as usize..].starts_with(needle) {
                found = Some(id.to_string());
            }
        });
        found.unwrap_or_else(|| panic!("no element starts `{needle}`"))
    }

    fn text_el(t: &str) -> Statement {
        let program = crate::syntax::parse_source(
            &format!("page P(path: \"/\") {{\n    Text(\"{t}\")\n}}\n"),
            "t.wf",
        )
        .unwrap();
        let Declaration::Page(p) = &program.declarations[0] else {
            unreachable!()
        };
        p.body[0].clone()
    }

    #[test]
    fn one_argument_changes_and_the_rest_is_as_it_was() {
        let id = id_of(PAGE, "Row(");
        let out = rewrite_element(PAGE, "t.wf", &id, |e| {
            e.args[0] = Arg::Named("gap".into(), Expr::EnumCase("lg".into()));
        })
        .unwrap();
        // The other argument keeps its odd spacing; the comments stay.
        assert_eq!(
            out,
            PAGE.replace("Row(gap: .sm,   align", "Row(gap: .lg,   align")
        );
    }

    #[test]
    fn a_flag_is_added_after_the_others() {
        let id = id_of(PAGE, "Heading(");
        let out = rewrite_element(PAGE, "t.wf", &id, |e| e.modifiers.push("muted".into())).unwrap();
        assert_eq!(
            out,
            PAGE.replace(".h2   // the title", ".h2.muted   // the title")
        );
        let out = rewrite_element(PAGE, "t.wf", &id, |e| e.modifiers.clear()).unwrap();
        assert_eq!(
            out,
            PAGE.replace("Heading(\"Deploys\").h2", "Heading(\"Deploys\")")
        );
    }

    #[test]
    fn a_style_value_is_written_where_it_was_and_a_new_one_after_the_last() {
        let id = id_of(PAGE, "Card {");
        let out = rewrite_element(PAGE, "t.wf", &id, |e| {
            let style = e.style_block.as_mut().unwrap();
            style.properties[0].value = Expr::StringLiteral("8px".into());
        })
        .unwrap();
        assert_eq!(out, PAGE.replace("padding: 12px   16px", "padding: 8px"));
        let out = rewrite_element(PAGE, "t.wf", &id, |e| {
            let style = e.style_block.as_mut().unwrap();
            let mut p = style.properties[1].clone();
            p.name = "gap".into();
            p.value = Expr::Token("space-sm".into());
            style.properties.push(p);
        })
        .unwrap();
        assert_eq!(
            out,
            PAGE.replace(
                "color: $text\n",
                "color: $text\n                gap: $space-sm\n"
            )
        );
        let out = rewrite_element(PAGE, "t.wf", &id, |e| {
            e.style_block.as_mut().unwrap().properties.remove(1);
        })
        .unwrap();
        assert_eq!(out, PAGE.replace("                color: $text\n", ""));
    }

    #[test]
    fn a_child_added_goes_on_its_own_line_and_one_removed_takes_its_line() {
        let id = id_of(PAGE, "Stack(");
        let out =
            rewrite_element(PAGE, "t.wf", &id, |e| e.children.insert(1, text_el("New"))).unwrap();
        assert_eq!(
            out,
            PAGE.replace("// the title\n", "// the title\n        Text(\"New\")\n")
        );
        let out = rewrite_element(PAGE, "t.wf", &id, |e| {
            e.children.retain(|c| !matches!(&c.kind, StatementKind::UIElement(u) if matches!(&u.component, ComponentRef::BuiltIn(n) if n == "Spacer")));
        })
        .unwrap();
        assert_eq!(out, PAGE.replace("        Spacer\n", ""));
        // A comment at the end of its line goes with it.
        let out = rewrite_element(PAGE, "t.wf", &id, |e| {
            e.children.remove(0);
        })
        .unwrap();
        assert_eq!(
            out,
            PAGE.replace("        Heading(\"Deploys\").h2   // the title\n", "")
        );
        // Into an element with no block: one is written.
        let spacer = id_of(PAGE, "Spacer");
        let out =
            rewrite_element(PAGE, "t.wf", &spacer, |e| e.children.push(text_el("In"))).unwrap();
        assert_eq!(
            out,
            PAGE.replace(
                "        Spacer\n",
                "        Spacer {\n            Text(\"In\")\n        }\n"
            )
        );
    }

    #[test]
    fn moving_a_child_moves_only_it() {
        let id = id_of(PAGE, "Stack(");
        let out = rewrite_element(PAGE, "t.wf", &id, |e| {
            let last = e.children.pop().unwrap();
            e.children.insert(0, last);
        })
        .unwrap();
        assert_eq!(
            out,
            PAGE.replace("        Spacer\n", "").replace(
                "    Stack(gap: .md) {\n",
                "    Stack(gap: .md) {\n        Spacer\n"
            )
        );
    }

    #[test]
    fn wrapping_a_child_in_a_condition_keeps_its_bytes_inside() {
        let id = id_of(PAGE, "Stack(");
        let out = rewrite_element(PAGE, "t.wf", &id, |e| {
            let card = e.children.remove(2);
            let wrapped = Statement::new(
                StatementKind::If(IfStmt {
                    condition: Expr::Identifier("open".into()),
                    binding: None,
                    animate: None,
                    animate_span: None,
                    then_body: vec![card],
                    else_if_branches: Vec::new(),
                    else_body: None,
                }),
                Span::dummy(),
            );
            e.children.insert(2, wrapped);
        })
        .unwrap();
        assert!(
            out.contains("        if open {\n            Card {\n"),
            "{out}"
        );
        assert!(out.contains("// the title"));
        assert!(crate::syntax::parse_source(&out, "t.wf").is_ok());
    }

    #[test]
    fn a_head_changes_and_its_body_does_not() {
        let src = "page P(path: \"/\") {\n    if live.length == 0 {   // none yet\n        Text(\"None\")\n    } else {\n        Text(\"Some\")\n    }\n}\n";
        let start = src.find("if ").unwrap();
        let end = src.rfind("    }").unwrap() + 5;
        let out = rewrite_statement(src, "t.wf", start, end, |s| {
            let StatementKind::If(i) = &mut s.kind else {
                panic!()
            };
            i.condition = Expr::Identifier("empty".into());
        })
        .unwrap();
        assert_eq!(out, src.replace("if live.length == 0 {", "if empty {"));
    }

    #[test]
    fn a_body_gains_a_statement_at_its_end_and_loses_its_first() {
        let out = rewrite_body(PAGE, "t.wf", "Home", |b| b.push(text_el("End"))).unwrap();
        assert_eq!(
            out,
            PAGE.replace("    }\n}\n", "    }\n    Text(\"End\")\n}\n")
        );
        let out = rewrite_body(PAGE, "t.wf", "Home", |b| {
            b.remove(0);
        })
        .unwrap();
        // The comment above it stays: it is not part of the statement.
        assert!(out.starts_with("page Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n    // The deploys, newest first.\n}"), "{out}");
    }

    #[test]
    fn what_cannot_be_read_back_is_not_written() {
        let id = id_of(PAGE, "Row(");
        // A flag that is not a word would not read back as one.
        let refused = rewrite_element(PAGE, "t.wf", &id, |e| e.modifiers.push("not a flag".into()));
        assert!(refused.is_err());
        let refused = rewrite_element(PAGE, "t.wf", "Home:9.9", |_| {});
        assert!(refused.unwrap_err().to_string().contains("unknown node id"));
    }

    const FILE: &str = r#"// Cards, and what they show.

/// A card.
component Card(_ title: String,   tone: String = "calm") {
    event opened

    Stack {
        Text(title)   // the title
    }
}

theme Brand {
    color-primary: #0F766E
    radius-md: 14px
}

const LIMIT = 10
"#;

    use crate::edit::{rewrite_declaration, rewrite_program};

    fn card(d: &mut Declaration) -> &mut ComponentDecl {
        match d {
            Declaration::Component(c) => c,
            _ => panic!("a component"),
        }
    }

    #[test]
    fn a_prop_changes_in_its_own_place() {
        let out = rewrite_declaration(FILE, "t.wf", "Card", |d| {
            card(d).props[1].default = Some(Expr::StringLiteral("loud".into()));
        })
        .unwrap();
        assert_eq!(
            out,
            FILE.replace("tone: String = \"calm\"", "tone: String = \"loud\"")
        );
        let out = rewrite_declaration(FILE, "t.wf", "Card", |d| {
            let mut p = card(d).props[1].clone();
            p.name = "size".into();
            p.default = None;
            card(d).props.push(p);
        })
        .unwrap();
        // A prop added is the header again; the rest of the file as it was.
        assert_eq!(
            out,
            FILE.replace(
                "component Card(_ title: String,   tone: String = \"calm\")",
                "component Card(_ title: String, tone: String = \"calm\", size: String)"
            )
        );
    }

    #[test]
    fn an_event_and_its_doc_go_after_the_last() {
        let out = rewrite_declaration(FILE, "t.wf", "Card", |d| {
            card(d).events.push(EventDecl {
                name: "picked".into(),
                params: vec![ParamDecl {
                    name: "id".into(),
                    param_type: TypeRef::String,
                }],
                doc: Some("When one is picked.".into()),
                span: Span::dummy(),
            });
        })
        .unwrap();
        assert_eq!(
            out,
            FILE.replace(
                "    event opened\n",
                "    event opened\n    /// When one is picked.\n    event picked(id: String)\n"
            )
        );
    }

    /// A store whose members are documented, for the tests below.
    const STORE: &str = "store Deploys {\n    /// What is deployed.\n    state items = []\n    /// How many.\n    derived count = items.length\n\n    /// Empty it.\n    action clear {\n        items = []\n    }\n}\n";

    fn store_body(d: &mut Declaration) -> &mut Vec<Statement> {
        let Declaration::Store(s) = d else {
            panic!("a store")
        };
        &mut s.body
    }

    #[test]
    fn a_member_s_doc_stays_once_as_its_value_changes() {
        let out = crate::edit::rewrite_declaration(STORE, "t.wf", "Deploys", |d| {
            let StatementKind::Derived(c) = &mut store_body(d)[1].kind else {
                panic!("count")
            };
            c.value = Expr::NumberLiteral(0.0);
        })
        .unwrap();
        assert_eq!(
            out,
            STORE.replace("derived count = items.length", "derived count = 0")
        );
    }

    #[test]
    fn a_member_removed_takes_its_doc_and_one_added_brings_its_own() {
        let out = crate::edit::rewrite_declaration(STORE, "t.wf", "Deploys", |d| {
            store_body(d).remove(1);
        })
        .unwrap();
        assert_eq!(
            out,
            STORE.replace("    /// How many.\n    derived count = items.length\n", "")
        );
        let out = crate::edit::rewrite_declaration(STORE, "t.wf", "Deploys", |d| {
            let mut first = store_body(d)[0].clone();
            let StatementKind::State(st) = &mut first.kind else {
                panic!("items")
            };
            st.name = "envs".into();
            st.doc = Some("Where to.".into());
            store_body(d).insert(0, first);
        })
        .unwrap();
        // At the front: above the first member's doc, not between it and
        // its member.
        assert_eq!(
            out,
            STORE.replace(
                "    /// What is deployed.\n",
                "    /// Where to.\n    state envs = []\n    /// What is deployed.\n"
            )
        );
        // A doc changed is written where the old one was.
        let out = crate::edit::rewrite_declaration(STORE, "t.wf", "Deploys", |d| {
            let StatementKind::Action(a) = &mut store_body(d)[2].kind else {
                panic!("clear")
            };
            a.doc = Some("Empty it.\nEvery one.".into());
        })
        .unwrap();
        assert_eq!(
            out,
            STORE.replace(
                "    /// Empty it.\n",
                "    /// Empty it.\n    /// Every one.\n"
            )
        );
    }

    #[test]
    fn a_doc_is_written_above_and_replaced_where_it_is() {
        let out = rewrite_declaration(FILE, "t.wf", "Card", |d| {
            card(d).doc = Some("A card.\n\nIt shows one thing.".into());
        })
        .unwrap();
        assert_eq!(
            out,
            FILE.replace(
                "/// A card.\n",
                "/// A card.\n///\n/// It shows one thing.\n"
            )
        );
        let out = rewrite_declaration(FILE, "t.wf", "Card", |d| card(d).doc = None).unwrap();
        assert_eq!(out, FILE.replace("/// A card.\n", ""));
    }

    #[test]
    fn a_part_is_declared_in_its_component() {
        let out = rewrite_program(FILE, "t.wf", |p| {
            let mut part = match &p.declarations[0] {
                Declaration::Component(c) => c.clone(),
                _ => unreachable!(),
            };
            part.name = "Card.Header".into();
            part.props.truncate(1);
            part.events.clear();
            part.doc = None;
            part.body = Vec::new();
            card(&mut p.declarations[0]).parts.push("Header".into());
            p.declarations.insert(1, Declaration::Component(part));
        })
        .unwrap();
        assert_eq!(
            out,
            FILE.replace(
                "    event opened\n",
                "    event opened\n    part Header(_ title: String) {}\n"
            )
        );
    }

    #[test]
    fn a_declaration_comes_and_goes_with_its_doc_and_blank_line() {
        let out = rewrite_program(FILE, "t.wf", |p| {
            p.declarations
                .retain(|d| !matches!(d, Declaration::Theme(_)));
        })
        .unwrap();
        assert_eq!(
            out,
            FILE.replace(
                "theme Brand {\n    color-primary: #0F766E\n    radius-md: 14px\n}\n\n",
                ""
            )
        );
        let out = rewrite_program(FILE, "t.wf", |p| {
            p.declarations
                .retain(|d| !matches!(d, Declaration::Component(_)));
        })
        .unwrap();
        assert!(
            out.starts_with("// Cards, and what they show.\n\ntheme Brand {"),
            "{out}"
        );
        let out = rewrite_program(FILE, "t.wf", |p| {
            p.declarations.push(Declaration::Const(ConstDecl {
                name: "PAGE".into(),
                ty: None,
                value: Expr::NumberLiteral(20.0),
                doc: Some("How many a page shows.".into()),
                span: Span::dummy(),
            }));
        })
        .unwrap();
        assert_eq!(
            out,
            format!("{FILE}\n/// How many a page shows.\nconst PAGE = 20\n")
                .replace("const LIMIT = 10\n\n", "const LIMIT = 10\n\n")
        );
    }

    #[test]
    fn a_token_changes_in_its_own_place() {
        let out = rewrite_declaration(FILE, "t.wf", "Brand", |d| {
            let Declaration::Theme(t) = d else { panic!() };
            t.tokens[1].value = Expr::StringLiteral("8px".into());
        })
        .unwrap();
        assert_eq!(out, FILE.replace("radius-md: 14px", "radius-md: 8px"));
    }

    #[test]
    fn a_page_gains_a_head_and_loses_it() {
        let src = "page Home(path: \"/\", title: \"Home\") {\n    Text(\"Hi\")\n}\n";
        let out = rewrite_declaration(src, "t.wf", "Home", |d| {
            let Declaration::Page(p) = d else { panic!() };
            p.head.push(HeadTag {
                tag: "meta".into(),
                attrs: vec![
                    ("name".into(), Expr::StringLiteral("theme-color".into())),
                    ("content".into(), Expr::StringLiteral("#fff".into())),
                ],
                span: Span::dummy(),
            });
        })
        .unwrap();
        assert_eq!(
            out,
            "page Home(path: \"/\", title: \"Home\") {\n    head {\n        meta(name: \"theme-color\", content: \"#fff\")\n    }\n    Text(\"Hi\")\n}\n"
        );
        let back = rewrite_declaration(&out, "t.wf", "Home", |d| {
            let Declaration::Page(p) = d else { panic!() };
            p.head.clear();
        })
        .unwrap();
        assert_eq!(back, src);
        let titled = rewrite_declaration(src, "t.wf", "Home", |d| {
            let Declaration::Page(p) = d else { panic!() };
            p.title = Some("Start".into());
        })
        .unwrap();
        assert_eq!(titled, src.replace("title: \"Home\"", "title: \"Start\""));
    }
}
