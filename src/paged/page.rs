//! The page box: how big the paper is and how wide its margins, page by
//! page.
//!
//! Three places say it, and the more particular wins, as in a browser:
//!
//! 1. the configuration (`build.pdf.page_size`, `margins`) — the default, as
//!    a print dialog's paper is;
//! 2. the sheets' `@page { size; margin }`, which a browser lets override
//!    that default;
//! 3. the document's own `Document(size:, landscape, margin:)` — written on
//!    the element, as an inline style is over a sheet.
//!
//! Then `@page :left` and `:right` refine the margins of the pages on each
//! side, and `:first` the first page's (it is more specific than either, as
//! the cascade of page selectors has it). A document runs left to right,
//! so its first page is a right-hand page, unless it is right to left.
//! `size` in a `:first`, `:left` or `:right` rule is ignored, as CSS Paged
//! Media says.
//!
//! The content is one strip laid out at one width — the pages with no
//! side or first rule of their own; a page whose margins would leave it
//! another width is laid out at that one and placed at its own left margin,
//! with a note.

use super::Paper;
use super::css::{Declaration, PageRule, PageSelector, split_spaces};
use super::paginate::Bands;
use super::style::{Length, Units, length};

/// The paper and each page's margins (top, right, bottom, left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub width: f32,
    pub height: f32,
    /// The margins of a page no `:first`, `:left` or `:right` rule reaches:
    /// what the content is laid out at the width of.
    pub base: [f32; 4],
    pub first: [f32; 4],
    /// Pages 2, 4, 6…
    pub even: [f32; 4],
    /// Pages 3, 5, 7…
    pub odd: [f32; 4],
}

impl Geometry {
    /// Every page alike.
    pub fn uniform(paper: Paper) -> Geometry {
        Geometry {
            width: paper.width,
            height: paper.height,
            base: paper.margin,
            first: paper.margin,
            even: paper.margin,
            odd: paper.margin,
        }
    }

    /// The paper with the base margins.
    pub fn paper(&self) -> Paper {
        Paper {
            width: self.width,
            height: self.height,
            margin: self.base,
        }
    }

    /// The margins of page `i` (from 0).
    pub fn margins(&self, i: usize) -> [f32; 4] {
        match i {
            0 => self.first,
            _ if i % 2 == 1 => self.even,
            _ => self.odd,
        }
    }

    /// The height of each page's content area.
    pub fn bands(&self) -> Bands {
        let h = |m: [f32; 4]| (self.height - m[0] - m[2]).max(10.0);
        Bands {
            first: h(self.first),
            even: h(self.even),
            odd: h(self.odd),
        }
    }
}

/// The page box from the configured `paper`, the sheets' `@page` rules and
/// the document's own size (`document`, applied over the rules), with a
/// note for each thing the reader should know. `rtl` makes the first page a
/// left-hand one; `em` is what an `em` in a margin is.
pub fn resolve(
    paper: Paper,
    rules: &[PageRule],
    document: impl FnOnce(Paper) -> Paper,
    rtl: bool,
    em: f32,
) -> (Geometry, Vec<String>) {
    let mut notes = Vec::new();
    let mut base = paper;
    for rule in rules.iter().filter(|r| r.selector == PageSelector::All) {
        for d in &rule.declarations {
            if d.name == "size" {
                size(&d.value, &mut base);
            }
        }
        margins(&rule.declarations, &mut base, em);
    }
    let base = document(base);
    let side = |which: PageSelector| {
        let mut p = base;
        for rule in rules.iter().filter(|r| r.selector == which) {
            margins(&rule.declarations, &mut p, em);
        }
        p.margin
    };
    // Left to right, the first page is a right-hand page, and so every odd
    // one; right to left the other way round.
    let (odd_side, even_side) = if rtl {
        (PageSelector::Left, PageSelector::Right)
    } else {
        (PageSelector::Right, PageSelector::Left)
    };
    let odd = side(odd_side);
    let even = side(even_side);
    let mut first = Paper {
        margin: odd,
        ..base
    };
    for rule in rules.iter().filter(|r| r.selector == PageSelector::First) {
        margins(&rule.declarations, &mut first, em);
    }
    let g = Geometry {
        width: base.width,
        height: base.height,
        base: base.margin,
        first: first.margin,
        even,
        odd,
    };
    let width = |m: [f32; 4]| g.width - m[1] - m[3];
    for (name, m) in [(":first", g.first), (":left/:right", g.even), (":left/:right", g.odd)] {
        let note = format!(
            "`@page {name}` leaves its pages a content width other than the rest; they are laid out at the width of the rest"
        );
        if (width(m) - width(g.base)).abs() > 0.5 && !notes.contains(&note) {
            notes.push(note);
        }
    }
    (g, notes)
}

/// `size: A4`, `A4 landscape`, `landscape`, `210mm 297mm`, `8in`, `auto`.
fn size(value: &str, paper: &mut Paper) {
    let units = Units {
        em: 12.0,
        rem: 12.0,
        vw: 0.0,
        vh: 0.0,
    };
    let mut lengths = Vec::new();
    let mut orientation = None;
    for w in split_spaces(value) {
        let lower = w.to_ascii_lowercase();
        match lower.as_str() {
            "auto" => {}
            "portrait" | "landscape" => orientation = Some(lower),
            _ => {
                if let Some(l) = super::style::length_pt(w, &units).filter(|l| *l > 0.0) {
                    lengths.push(l);
                } else if let Some((pw, ph)) = super::paper_size(w) {
                    paper.width = pw;
                    paper.height = ph;
                }
            }
        }
    }
    match lengths.as_slice() {
        [s] => {
            paper.width = *s;
            paper.height = *s;
        }
        [w, h] => {
            paper.width = *w;
            paper.height = *h;
        }
        _ => {}
    }
    let landscape = paper.width > paper.height;
    match orientation.as_deref() {
        Some("landscape") if !landscape => std::mem::swap(&mut paper.width, &mut paper.height),
        Some("portrait") if landscape => std::mem::swap(&mut paper.width, &mut paper.height),
        _ => {}
    }
}

/// `margin`, `margin-top` … in an `@page` rule. A percentage is of the
/// page's width for the left and right, of its height for the top and
/// bottom; `auto` keeps what was there.
fn margins(decls: &[Declaration], paper: &mut Paper, em: f32) {
    let units = Units {
        em,
        rem: em,
        vw: paper.width,
        vh: paper.height,
    };
    let (w, h) = (paper.width, paper.height);
    let of = |side: usize| if side.is_multiple_of(2) { h } else { w };
    let read = |v: &str, side: usize| -> Option<f32> {
        match length(v, &units)? {
            Length::Auto => None,
            l => l.resolve(of(side)).map(|v| v.max(0.0)),
        }
    };
    for d in decls {
        let side = match d.name.as_str() {
            "margin" => {
                let parts = split_spaces(&d.value);
                let pick: [usize; 4] = match parts.len() {
                    1 => [0, 0, 0, 0],
                    2 => [0, 1, 0, 1],
                    3 => [0, 1, 2, 1],
                    4 => [0, 1, 2, 3],
                    _ => continue,
                };
                for (s, &p) in pick.iter().enumerate() {
                    if let Some(v) = read(parts[p], s) {
                        paper.margin[s] = v;
                    }
                }
                continue;
            }
            "margin-top" => 0,
            "margin-right" => 1,
            "margin-bottom" => 2,
            "margin-left" => 3,
            _ => continue,
        };
        if let Some(v) = read(&d.value, side) {
            paper.margin[side] = v;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paged::css::Sheet;

    fn a4() -> Paper {
        Paper {
            width: 595.28,
            height: 841.89,
            margin: [72.0; 4],
        }
    }

    fn geometry(css: &str, rtl: bool) -> Geometry {
        resolve(a4(), &Sheet::parse(css).pages, |p| p, rtl, 12.0).0
    }

    #[test]
    fn page_rules_set_the_paper_and_each_sides_margins() {
        let g = geometry(
            "@page { size: letter landscape; margin: 1in 36pt }
             @page :first { margin-top: 2in }
             @page :left { margin-left: 54pt; margin-right: 18pt }
             @page :right { margin-left: 18pt; margin-right: 54pt }",
            false,
        );
        assert_eq!((g.width, g.height), (792.0, 612.0));
        assert_eq!(g.base, [72.0, 36.0, 72.0, 36.0]);
        // The first page is a right-hand page, with its own top.
        assert_eq!(g.first, [144.0, 54.0, 72.0, 18.0]);
        assert_eq!(g.even, [72.0, 18.0, 72.0, 54.0]);
        assert_eq!(g.odd, [72.0, 54.0, 72.0, 18.0]);
        // Right to left, the first page is a left-hand page.
        let g = geometry("@page :left { margin-left: 10pt }", true);
        assert_eq!(g.first[3], 10.0);
        assert_eq!(g.odd[3], 10.0);
        assert_eq!(g.even[3], 72.0);
    }

    #[test]
    fn the_document_wins_over_the_sheet_and_the_sheet_over_the_configuration() {
        let rules = Sheet::parse("@page { size: A5; margin: 10mm }").pages;
        let (g, _) = resolve(a4(), &rules, |p| p, false, 12.0);
        assert_eq!((g.width, g.height), (419.53, 595.28));
        assert!((g.base[0] - 28.346).abs() < 0.01);
        let (g, _) = resolve(
            a4(),
            &rules,
            |mut p| {
                p.width = 612.0;
                p.height = 792.0;
                p
            },
            false,
            12.0,
        );
        assert_eq!((g.width, g.height), (612.0, 792.0));
        // `size` on a side is ignored; a percentage margin is of the page.
        let g = geometry("@page :first { size: A3; margin-left: 10% }", false);
        assert_eq!(g.width, 595.28);
        assert!((g.first[3] - 59.528).abs() < 0.01);
    }

    #[test]
    fn the_bands_follow_each_pages_margins() {
        let g = geometry("@page :first { margin-top: 272pt }", false);
        let b = g.bands();
        assert!((b.first - (841.89 - 344.0)).abs() < 0.01);
        assert!((b.even - (841.89 - 144.0)).abs() < 0.01);
        assert_eq!(b.index_of(b.first - 1.0), 0);
        assert_eq!(b.index_of(b.first + 1.0), 1);
        assert_eq!(b.index_of(b.top(3) + 1.0), 3);
        assert_eq!(b.top(2), b.first + b.even);
    }
}
