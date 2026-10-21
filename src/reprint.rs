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
        old.span.start as usize,
        old.span.end as usize,
        print::statement(new, block, depth),
    ));
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
                    o[0].span.start as usize,
                    o[0].span.end as usize,
                    unit_text(n, block, depth),
                ));
            }
            anchor = Some(o[0].span.end as usize);
        }
        for o in &old_gap[paired..] {
            out.push(removal(
                source,
                o[0].span.start as usize,
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
                let at = first[0].span.start as usize;
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
}
