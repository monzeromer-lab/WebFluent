//! A file's change as lines: the hunks between two texts, each with the
//! lines around it, and the text that accepting some of them gives — the
//! shape a reviewer stages by hunk.

use serde::Serialize;

/// What a line of a hunk is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LineKind {
    Context,
    Added,
    Removed,
}

/// One line of a hunk, with its newline as the file has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Line {
    pub kind: LineKind,
    pub text: String,
}

/// One run of changes and the lines around it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hunk {
    /// Its place among the file's hunks, from 0: what an accept names.
    pub id: usize,
    /// Where it starts in the old text and the new, from 1, and how many
    /// lines it covers in each — as a unified diff's `@@` line says.
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<Line>,
}

impl Hunk {
    pub fn added(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| l.kind == LineKind::Added)
            .count()
    }

    pub fn removed(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| l.kind == LineKind::Removed)
            .count()
    }
}

/// How many unchanged lines a hunk keeps on each side.
pub const CONTEXT: usize = 3;

fn lines(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

/// The edit script from `a` to `b`, line by line: Myers' shortest one.
fn script<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<(LineKind, &'a str)> {
    let (n, m) = (a.len() as isize, b.len() as isize);
    let max = (n + m) as usize;
    let offset = max as isize;
    let mut v = vec![0isize; 2 * max + 2];
    let mut trace: Vec<Vec<isize>> = Vec::new();
    'search: for d in 0..=max as isize {
        trace.push(v.clone());
        let mut k = -d;
        while k <= d {
            let i = (k + offset) as usize;
            let mut x = if k == -d || (k != d && v[i - 1] < v[i + 1]) {
                v[i + 1]
            } else {
                v[i - 1] + 1
            };
            let mut y = x - k;
            while x < n && y < m && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }
            v[i] = x;
            if x >= n && y >= m {
                trace.push(v.clone());
                break 'search;
            }
            k += 2;
        }
    }
    // Walk back through the trace.
    let mut out = Vec::new();
    let (mut x, mut y) = (n, m);
    for d in (1..trace.len() as isize - 1).rev() {
        let v = &trace[d as usize];
        let k = x - y;
        let i = (k + offset) as usize;
        let prev_k = if k == -d || (k != d && v[i - 1] < v[i + 1]) {
            k + 1
        } else {
            k - 1
        };
        let prev_x = v[(prev_k + offset) as usize];
        let prev_y = prev_x - prev_k;
        while x > prev_x && y > prev_y {
            out.push((LineKind::Context, a[(x - 1) as usize]));
            x -= 1;
            y -= 1;
        }
        if x == prev_x {
            out.push((LineKind::Added, b[(y - 1) as usize]));
            y -= 1;
        } else {
            out.push((LineKind::Removed, a[(x - 1) as usize]));
            x -= 1;
        }
    }
    while x > 0 && y > 0 {
        out.push((LineKind::Context, a[(x - 1) as usize]));
        x -= 1;
        y -= 1;
    }
    while y > 0 {
        out.push((LineKind::Added, b[(y - 1) as usize]));
        y -= 1;
    }
    while x > 0 {
        out.push((LineKind::Removed, a[(x - 1) as usize]));
        x -= 1;
    }
    out.reverse();
    out
}

/// The hunks that take `old` to `new`.
pub fn hunks(old: &str, new: &str) -> Vec<Hunk> {
    let (a, b) = (lines(old), lines(new));
    let ops = script(&a, &b);
    // Which ops are changes, and the runs of context between them.
    let changed: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, (k, _))| *k != LineKind::Context)
        .map(|(i, _)| i)
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < changed.len() {
        let first = changed[i];
        let mut last = first;
        // Changes within twice the context of each other share a hunk.
        while i + 1 < changed.len() && changed[i + 1] - last <= 2 * CONTEXT + 1 {
            i += 1;
            last = changed[i];
        }
        let start = first.saturating_sub(CONTEXT);
        let end = (last + CONTEXT + 1).min(ops.len());
        // Where `start` is in each text.
        let (mut old_at, mut new_at) = (0, 0);
        for (k, _) in &ops[..start] {
            match k {
                LineKind::Context => {
                    old_at += 1;
                    new_at += 1;
                }
                LineKind::Removed => old_at += 1,
                LineKind::Added => new_at += 1,
            }
        }
        let body: Vec<Line> = ops[start..end]
            .iter()
            .map(|(k, t)| Line {
                kind: *k,
                text: t.to_string(),
            })
            .collect();
        let old_lines = body.iter().filter(|l| l.kind != LineKind::Added).count();
        let new_lines = body.iter().filter(|l| l.kind != LineKind::Removed).count();
        out.push(Hunk {
            id: out.len(),
            old_start: old_at + 1,
            old_lines,
            new_start: new_at + 1,
            new_lines,
            lines: body,
        });
        i += 1;
    }
    out
}

/// `old` with the hunks whose ids are in `accepted` applied, and the rest
/// left as they were: accepting every hunk gives the new text, none the
/// old.
pub fn apply(old: &str, hunks: &[Hunk], accepted: &[usize]) -> String {
    let a = lines(old);
    let mut out = String::new();
    let mut at = 0; // the next old line to copy, from 0
    for h in hunks {
        let start = h.old_start - 1;
        for line in &a[at..start.min(a.len())] {
            out.push_str(line);
        }
        let take = accepted.contains(&h.id);
        for l in &h.lines {
            match (l.kind, take) {
                (LineKind::Context, _) | (LineKind::Added, true) | (LineKind::Removed, false) => {
                    out.push_str(&l.text)
                }
                _ => {}
            }
        }
        at = start + h.old_lines;
    }
    for line in a.iter().skip(at) {
        out.push_str(line);
    }
    out
}

/// The hunks as a unified diff of `path`.
pub fn unified(path: &str, hunks: &[Hunk]) -> String {
    let mut out = format!("--- a/{path}\n+++ b/{path}\n");
    for h in hunks {
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            h.old_start, h.old_lines, h.new_start, h.new_lines
        ));
        for l in &h.lines {
            let sign = match l.kind {
                LineKind::Context => ' ',
                LineKind::Added => '+',
                LineKind::Removed => '-',
            };
            out.push(sign);
            out.push_str(&l.text);
            if !l.text.ends_with('\n') {
                out.push_str("\n\\ No newline at end of file\n");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_subset_of_hunks_applies() {
        let old = (1..=60).map(|i| format!("line {i}\n")).collect::<String>();
        let mut new = old
            .replace("line 3\n", "line three\n")
            .replace("line 20\n", "")
            .replace("line 35\n", "line 35\nline 35b\n");
        new.push_str("tail");
        let h = hunks(&old, &new);
        assert_eq!(h.len(), 4, "{h:#?}");
        let ids: Vec<usize> = h.iter().map(|x| x.id).collect();
        assert_eq!(apply(&old, &h, &ids), new);
        assert_eq!(apply(&old, &h, &[]), old);
        for mask in 0..16u32 {
            let pick: Vec<usize> = ids
                .iter()
                .copied()
                .filter(|i| mask & (1 << i) != 0)
                .collect();
            let got = apply(&old, &h, &pick);
            // Each hunk applied alone changes only its own lines.
            assert_eq!(got.contains("line three"), pick.contains(&0));
            assert_eq!(!got.contains("line 20\n"), pick.contains(&1));
            assert_eq!(got.contains("35b"), pick.contains(&2));
            assert_eq!(got.ends_with("tail"), pick.contains(&3));
        }
        assert_eq!((h[0].removed(), h[0].added()), (1, 1));
    }

    #[test]
    fn nothing_changed_is_no_hunk_and_a_new_file_is_one() {
        assert!(hunks("a\nb\n", "a\nb\n").is_empty());
        let h = hunks("", "a\nb\n");
        assert_eq!(
            (h.len(), h[0].added(), h[0].old_start, h[0].new_start),
            (1, 2, 1, 1)
        );
        assert_eq!(apply("", &h, &[0]), "a\nb\n");
        let gone = hunks("a\nb\n", "");
        assert_eq!(apply("a\nb\n", &gone, &[0]), "");
        assert!(unified("x.wf", &hunks("a\n", "b\n")).contains("@@ -1,1 +1,1 @@\n-a\n+b\n"));
    }
}
