//! Pagination: the laid-out strip cut into pages.
//!
//! The strip is walked in document order. Wherever a piece that cannot be
//! split — a line of text, a table row, a row of cards, a picture — would
//! straddle the end of a page, it moves to the top of the next, and
//! everything after it moves with it. Boxes that hold such a gap grow by it,
//! so a card that runs onto the next page paints on both, sliced at the page
//! edge. Side-by-side columns break in parallel; a table's header rows are
//! drawn again at the top of every page its body continues on.

use super::boxes::{Kind, Tree};
use super::layout::Frag;
use super::style::{Break, Position};

/// Less than this from a page's top is "at the top".
const EPS: f32 = 0.5;

pub struct Pages {
    pub count: usize,
    /// Frags drawn on a page beyond the flow: repeated table headers.
    pub extra: Vec<Frag>,
}

struct State<'a> {
    tree: &'a Tree,
    h: f32,
    off: f32,
    force: bool,
    extra: Vec<Frag>,
}

pub fn paginate(root: &mut Frag, tree: &Tree, page_height: f32) -> Pages {
    let mut st = State { tree, h: page_height.max(10.0), off: 0.0, force: false, extra: Vec::new() };
    st.process(root);
    let bottom = root.y + root.h;
    let count = ((bottom - EPS) / st.h).ceil().max(1.0) as usize;
    Pages { count, extra: st.extra }
}

impl State<'_> {
    fn page_top(&self, y: f32) -> f32 {
        (y / self.h).floor() * self.h
    }

    fn next_top(&self, y: f32) -> f32 {
        self.page_top(y + EPS) + self.h
    }

    fn at_top(&self, y: f32) -> bool {
        y - self.page_top(y + EPS) < EPS
    }

    /// Move a piece that straddles a page end to the next page.
    fn fit(&mut self, top: f32, height: f32) -> f32 {
        let bottom = top + height;
        let end = self.page_top(top + EPS) + self.h;
        if bottom > end + EPS && height <= self.h && !self.at_top(top) {
            let d = end - top;
            self.off += d;
            return d;
        }
        0.0
    }

    fn forced(&mut self, top: f32) -> f32 {
        if self.at_top(top) {
            return 0.0;
        }
        let d = self.next_top(top) - top;
        self.off += d;
        d
    }

    fn breakable(&self, f: &Frag) -> bool {
        let lb = &self.tree.boxes[f.boxed];
        let s = &lb.style;
        if s.position == Position::Absolute || s.position == Position::Fixed {
            return false;
        }
        let tall = f.h > self.h;
        if s.break_inside == Break::Avoid && !tall {
            return false;
        }
        match &lb.kind {
            Kind::Inline(_) => f.lines.len() > 1,
            Kind::Table { .. } => true,
            // A box as tall as its content breaks between what it holds;
            // one given a height is a unit, unless it cannot fit a page.
            Kind::Container => !f.children.is_empty() && (s.height == super::style::Length::Auto || tall),
            _ => false,
        }
    }

    fn process(&mut self, f: &mut Frag) {
        let tree = self.tree;
        let lb = &tree.boxes[f.boxed];
        let style = lb.style.clone();
        let entry = self.off;
        if style.break_before == Break::Page || std::mem::take(&mut self.force) {
            let top = f.y + self.off;
            self.forced(top);
        }
        if matches!(lb.kind, Kind::PageBreak) {
            f.shift(0.0, self.off);
            self.force = true;
            return;
        }
        if !self.breakable(f) {
            f.shift(0.0, self.off);
            let d = self.fit(f.y, f.h);
            if d > 0.0 {
                f.shift(0.0, d);
            }
        } else {
            let before = self.off;
            f.y += self.off;
            // How far the first content sits below the box's top.
            let gap = f.children.iter().map(|c| c.y + before).fold(f32::MAX, f32::min) - f.y;
            match &lb.kind {
                Kind::Inline(_) => self.lines(f),
                Kind::Table { header_rows, .. } => self.table(f, *header_rows),
                _ => self.children(f, before),
            }
            f.h += self.off - before;
            // A box whose first content moved to a new page goes with it,
            // rather than leaving an empty slice of itself behind.
            if !matches!(lb.kind, Kind::Inline(_)) && !f.children.is_empty() {
                let now = f.children.iter().map(|c| c.y).fold(f32::MAX, f32::min) - f.y;
                if now > gap + EPS && gap.is_finite() {
                    let moved = now - gap;
                    f.y += moved;
                    f.h -= moved;
                }
            }
        }
        if style.break_after == Break::Page {
            self.force = true;
        }
        let _ = entry;
    }

    /// The children of a breakable box, in rows of what overlaps.
    fn children(&mut self, f: &mut Frag, entry: f32) {
        let tree = self.tree;
        // Absolutely placed children move with the box's top, nothing else.
        let mut rows: Vec<Vec<usize>> = Vec::new();
        let mut row_bottom = f32::MIN;
        let order: Vec<usize> = {
            let mut o: Vec<usize> = (0..f.children.len()).collect();
            o.sort_by(|&a, &b| f.children[a].y.partial_cmp(&f.children[b].y).unwrap_or(std::cmp::Ordering::Equal));
            o
        };
        for i in order {
            let c = &f.children[i];
            let pos = tree.boxes[c.boxed].style.position;
            if pos == Position::Absolute || pos == Position::Fixed {
                continue;
            }
            if c.y < row_bottom - EPS && !rows.is_empty() {
                rows.last_mut().unwrap().push(i);
                row_bottom = row_bottom.max(c.y + c.h);
            } else {
                rows.push(vec![i]);
                row_bottom = c.y + c.h;
            }
        }
        for (i, c) in f.children.iter_mut().enumerate() {
            let pos = tree.boxes[c.boxed].style.position;
            if pos == Position::Absolute || pos == Position::Fixed {
                let _ = i;
                c.shift(0.0, entry);
            }
        }
        let row_count = rows.len();
        for (r, row) in rows.iter().enumerate() {
            if row.len() == 1 {
                let ix = row[0];
                let heading = self.is_heading_frag(&f.children[ix]);
                self.process(&mut f.children[ix]);
                // A heading never ends a page: it goes with what follows.
                if heading && r + 1 < row_count {
                    let next = &f.children[rows[r + 1][0]];
                    let need = next.h.min(self.first_unit_height(next)).min(self.h / 4.0);
                    let c = &f.children[ix];
                    let end = self.page_top(c.y + EPS) + self.h;
                    if c.y + c.h + need > end + EPS && !self.at_top(c.y) {
                        let d = end - c.y;
                        f.children[ix].shift(0.0, d);
                        self.off += d;
                    }
                }
                continue;
            }
            let top = row.iter().map(|&i| f.children[i].y).fold(f32::MAX, f32::min);
            let bottom = row.iter().map(|&i| f.children[i].y + f.children[i].h).fold(f32::MIN, f32::max);
            let height = bottom - top;
            let any_breakable = row.iter().any(|&i| self.breakable(&f.children[i]));
            if height <= self.h || !any_breakable {
                // A row that fits a page moves as one.
                for &i in row {
                    f.children[i].shift(0.0, self.off);
                }
                let d = self.fit(top + self.off, height);
                if d > 0.0 {
                    for &i in row {
                        f.children[i].shift(0.0, d);
                    }
                }
            } else {
                // Taller than a page: each column breaks on its own, and
                // what follows waits for the longest.
                let base = self.off;
                let mut most = base;
                for &i in row {
                    self.off = base;
                    self.process(&mut f.children[i]);
                    most = most.max(self.off);
                }
                self.off = most;
            }
        }
    }

    fn is_heading_frag(&self, f: &Frag) -> bool {
        self.tree.headings.iter().any(|(b, _)| *b == f.boxed)
    }

    fn first_unit_height(&self, f: &Frag) -> f32 {
        if let Some(l) = f.lines.first() {
            return l.height;
        }
        match f.children.first() {
            Some(c) if self.breakable(f) => self.first_unit_height(c) + (c.y - f.y).max(0.0),
            _ => f.h,
        }
    }

    /// A paragraph's lines: each moves to the next page whole, keeping at
    /// least `orphans` lines before a break and `widows` after it.
    fn lines(&mut self, f: &mut Frag) {
        let style = self.tree.boxes[f.boxed].style.clone();
        let n = f.lines.len();
        let orphans = style.orphans.max(1) as usize;
        let widows = style.widows.max(1) as usize;
        // The atoms were placed before the frag's own shift.
        for a in &mut f.atoms {
            a.shift(0.0, self.off);
        }
        let mut extra = 0.0f32;
        let mut i = 0;
        while i < n {
            // A line's top already carries every move before it.
            let top = f.y + f.lines[i].top;
            let height = f.lines[i].height;
            let page = self.page_top(top + EPS);
            let straddles = top + height > page + self.h + EPS && height <= self.h && !self.at_top(top);
            if !straddles {
                i += 1;
                continue;
            }
            let on_page = (0..i).filter(|&j| f.y + f.lines[j].top >= page - EPS).count();
            let mut break_at = i;
            if on_page < orphans && on_page == i && !self.at_top(f.y) {
                // Too few lines would stay behind: the paragraph moves whole.
                break_at = 0;
            } else if n - i < widows {
                // Too few would go ahead: break earlier, keeping enough behind.
                let earlier = n.saturating_sub(widows);
                if earlier < i && earlier >= i - on_page + orphans {
                    break_at = earlier;
                }
            }
            let at = f.y + f.lines[break_at].top;
            let d = self.next_top(at) - at;
            extra += d;
            for l in &mut f.lines[break_at..] {
                l.top += d;
            }
            let first_moved_atom: usize = f.lines[..break_at].iter().map(|l| l.atoms.len()).sum();
            for a in &mut f.atoms[first_moved_atom..] {
                a.shift(0.0, d);
            }
            i = break_at + 1;
        }
        self.off += extra;
        // A paragraph that moved whole starts where its first line does.
        if let Some(first) = f.lines.first().map(|l| l.top) {
            if first > EPS {
                f.y += first;
                for l in &mut f.lines {
                    l.top -= first;
                }
                f.h -= first;
            }
        }
    }

    /// A table: rows move whole; a row moved to a new page brings the
    /// header rows with it.
    fn table(&mut self, f: &mut Frag, header_rows: usize) {
        let tree = self.tree;
        let row_of = |c: &Frag| match tree.boxes[c.boxed].kind {
            Kind::Cell { row, .. } => row,
            _ => 0,
        };
        let rows = f.children.iter().map(row_of).max().map(|r| r + 1).unwrap_or(0);
        // The header as it stands, before anything moves.
        let header: Vec<Frag> = f.children.iter().filter(|c| row_of(c) < header_rows).cloned().collect();
        let header_top = header.iter().map(|c| c.y).fold(f32::MAX, f32::min);
        let header_h = header.iter().map(|c| c.y + c.h).fold(f32::MIN, f32::max) - header_top;
        for r in 0..rows {
            let idx: Vec<usize> = (0..f.children.len()).filter(|&i| row_of(&f.children[i]) == r).collect();
            if idx.is_empty() {
                continue;
            }
            for &i in &idx {
                f.children[i].shift(0.0, self.off);
            }
            let top = idx.iter().map(|&i| f.children[i].y).fold(f32::MAX, f32::min);
            let bottom = idx.iter().map(|&i| f.children[i].y + f.children[i].h).fold(f32::MIN, f32::max);
            let end = self.page_top(top + EPS) + self.h;
            let height = bottom - top;
            let repeat = header_rows > 0 && r >= header_rows && !header.is_empty();
            if bottom > end + EPS && height <= self.h && !self.at_top(top) {
                let mut d = end - top;
                if repeat {
                    d += header_h;
                    for h in &header {
                        let mut copy = h.clone();
                        copy.shift(0.0, end - header_top + (h.y - header_top) * 0.0);
                        self.extra.push(copy);
                    }
                }
                for &i in &idx {
                    f.children[i].shift(0.0, d);
                }
                self.off += d;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::boxes::Builder;
    use super::super::cascade::{Styler, UA_SHEET};
    use super::super::css::Sheet;
    use super::super::fonts::FontDb;
    use super::super::html::Dom;
    use super::super::layout::Layout;
    use super::super::style::{Style, Units};
    use super::*;

    fn pages(html: &str, css: &str, page_h: f32) -> (Tree, Frag, Pages) {
        let dom = Dom::parse(html);
        let styler = Styler::new(&Sheet::parse(UA_SHEET), &Sheet::parse(css), Units { em: 12.0, rem: 12.0, vw: 300.0, vh: page_h });
        let mut db = FontDb::new(false);
        let read = |_: &str| -> Option<Vec<u8>> { None };
        let tree = Builder::new(&dom, &styler, &mut db, &read).build(&Style::root(12.0, "sans-serif"));
        let mut frag = Layout::new(&tree, &db).layout(tree.flow, 300.0);
        let p = paginate(&mut frag, &tree, page_h);
        (tree, frag, p)
    }

    /// Every unit (line, leaf) and the page it lands on: none may straddle.
    fn assert_no_straddle(f: &Frag, h: f32) {
        for l in &f.lines {
            let top = f.y + l.top;
            let page_end = ((top + EPS) / h).floor() * h + h;
            assert!(top + l.height <= page_end + EPS, "a line straddles a page: {top}..{} past {page_end}", top + l.height);
        }
        for c in &f.children {
            assert_no_straddle(c, h);
        }
    }

    #[test]
    fn paragraphs_break_between_lines() {
        let para = "word ".repeat(400);
        let (_, f, p) = pages(&format!("<body><p>{para}</p></body>"), "p { margin: 0; line-height: 15pt }", 200.0);
        assert!(p.count >= 3, "{} pages", p.count);
        assert_no_straddle(&f, 200.0);
    }

    #[test]
    fn a_forced_break_starts_a_page() {
        let (_, f, p) = pages(
            r#"<body><p>one</p><div class="wf-page-break"></div><p>two</p></body>"#,
            "p { margin: 0 }",
            500.0,
        );
        assert_eq!(p.count, 2);
        let two = f.children.last().unwrap();
        assert!((two.y - 500.0).abs() < 0.01, "{}", two.y);
    }

    #[test]
    fn a_row_of_cards_moves_together() {
        let filler = "<p>x</p>".repeat(9);
        let (_, f, _) = pages(
            &format!(r#"<body>{filler}<div class="row"><div class="c">A</div><div class="c">B</div></div></body>"#),
            "p { margin: 0; height: 20pt } .row { display: flex } .c { height: 40pt; flex: 1 }",
            200.0,
        );
        let row = f.children.last().unwrap();
        // 9 × 20 = 180: the 40pt row would straddle 200, so it starts page 2.
        assert!((row.y - 200.0).abs() < 0.01, "{}", row.y);
        assert!(row.children.iter().all(|c| (c.y - 200.0).abs() < 0.01));
    }

    #[test]
    fn a_table_repeats_its_header_on_every_page() {
        let rows: String = (0..40).map(|i| format!("<tr><td>{i}</td><td>row {i}</td></tr>")).collect();
        let (_, f, p) = pages(
            &format!("<body><table><thead><tr><th>#</th><th>Item</th></tr></thead><tbody>{rows}</tbody></table></body>"),
            "td, th { padding: 2pt; height: 20pt }",
            200.0,
        );
        assert!(p.count >= 4, "{}", p.count);
        // One copy of the header per continued page, at its top.
        let copies = p.extra.len() / 2;
        assert_eq!(copies, p.count - 1);
        for c in &p.extra {
            assert!((c.y % 200.0).abs() < 0.01, "{}", c.y);
        }
        assert_no_straddle(&f, 200.0);
    }
}
