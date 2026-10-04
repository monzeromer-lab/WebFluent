//! Inline layout: a paragraph's runs of text (and the inline-blocks among
//! them) shaped, broken into lines, ordered for display and aligned.
//!
//! A paragraph is shaped once — every character given a face that has it,
//! every run shaped by rustybuzz in its own direction — and broken into lines
//! as often as layout asks, at whatever width.

use std::rc::Rc;

use unicode_bidi::{BidiInfo, Level};
use unicode_linebreak::{BreakOpportunity, linebreaks};

use super::fonts::{FontDb, Pick};
use super::style::{Direction, Style, TextAlign, TextTransform, VerticalAlign, WhiteSpace};

/// The object replacement character: where an inline-block sits in the text.
pub const ATOM: char = '\u{FFFC}';

/// What a paragraph is made of, in logical order.
#[derive(Clone, Debug)]
pub enum Item {
    Text {
        text: String,
        style: Rc<Style>,
        /// The inline element the text belongs to, for backgrounds and links.
        span: Option<usize>,
    },
    /// An inline-block: a box laid out on its own and placed on the line.
    Atom { boxed: usize, style: Rc<Style> },
    /// A forced line break (`<br>`).
    Break { style: Rc<Style> },
}

/// One glyph, ready to draw.
#[derive(Clone, Debug)]
pub struct Glyph {
    pub id: u16,
    /// Points.
    pub advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    /// Byte range of its cluster in the paragraph's text.
    pub range: std::ops::Range<usize>,
}

/// The smallest unit a line is made of: one or more characters that shape
/// together, in one face, at one level.
#[derive(Clone, Debug)]
pub struct Cluster {
    pub range: std::ops::Range<usize>,
    pub glyphs: Vec<Glyph>,
    pub width: f32,
    pub pick: Option<Pick>,
    pub size: f32,
    /// Which item it came from.
    pub item: usize,
    pub level: u8,
    pub space: bool,
    pub break_after: bool,
    pub mandatory_after: bool,
    pub atom: Option<usize>,
    /// The face's vertical metrics at this size, in points.
    pub ascent: f32,
    pub descent: f32,
}

/// A paragraph shaped and ready to be broken into lines.
pub struct Shaped {
    pub text: String,
    pub items: Vec<Item>,
    pub clusters: Vec<Cluster>,
    pub block: Rc<Style>,
    /// Characters no face had, for the findings.
    pub missing: Vec<char>,
}

/// A run of glyphs placed on a line: one face, one size, one style.
#[derive(Clone, Debug)]
pub struct Run {
    pub pick: Pick,
    pub size: f32,
    pub item: usize,
    /// Points from the line's start edge to the run's first glyph.
    pub x: f32,
    pub width: f32,
    pub glyphs: Vec<Glyph>,
    /// Shift of the baseline (superscript, subscript).
    pub rise: f32,
}

#[derive(Clone, Debug)]
pub struct PlacedAtom {
    pub boxed: usize,
    pub x: f32,
    /// From the line's top.
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// A span's extent on one line, for its background, border or link.
#[derive(Clone, Debug)]
pub struct SpanExtent {
    pub span: usize,
    pub x: f32,
    pub width: f32,
}

#[derive(Clone, Debug)]
pub struct Line {
    /// From the paragraph's top.
    pub top: f32,
    pub height: f32,
    /// From the line's top.
    pub baseline: f32,
    pub width: f32,
    pub runs: Vec<Run>,
    pub atoms: Vec<PlacedAtom>,
    pub spans: Vec<SpanExtent>,
}

/// The size of an atom laid out at an available width: width, height, and
/// its baseline from its top.
pub type AtomSizer<'a> = dyn FnMut(usize, f32) -> (f32, f32, f32) + 'a;

// ─── Whitespace ─────────────────────────────────────────────────────

fn collapses(ws: WhiteSpace) -> bool {
    matches!(
        ws,
        WhiteSpace::Normal | WhiteSpace::NoWrap | WhiteSpace::PreLine
    )
}

fn transform(text: &str, t: TextTransform, at_word_start: &mut bool) -> String {
    match t {
        TextTransform::None => text.to_string(),
        TextTransform::Upper => text.to_uppercase(),
        TextTransform::Lower => text.to_lowercase(),
        TextTransform::Capitalize => {
            let mut out = String::with_capacity(text.len());
            for ch in text.chars() {
                if *at_word_start && ch.is_alphanumeric() {
                    out.extend(ch.to_uppercase());
                    *at_word_start = false;
                } else {
                    if ch.is_whitespace() {
                        *at_word_start = true;
                    }
                    out.push(ch);
                }
            }
            out
        }
    }
}

/// Collapse whitespace across items as CSS does: a run of spaces and
/// newlines is one space, none at the start of the paragraph, none after
/// a space in an earlier item. Returns the items with their text processed;
/// an item left empty is dropped.
pub fn normalise(items: Vec<Item>) -> Vec<Item> {
    let mut out = Vec::with_capacity(items.len());
    let mut last_space = true;
    let mut word_start = true;
    for item in items {
        match item {
            Item::Text { text, style, span } => {
                let mut t = String::with_capacity(text.len());
                let text = text.replace('\u{ad}', "");
                match style.white_space {
                    ws if collapses(ws) => {
                        for ch in text.chars() {
                            let is_newline = ch == '\n' || ch == '\r';
                            if ws == WhiteSpace::PreLine && is_newline {
                                // Trailing space before a kept newline goes.
                                while t.ends_with(' ') {
                                    t.pop();
                                }
                                t.push('\n');
                                last_space = true;
                            } else if ch == ' ' || ch == '\t' || is_newline || ch == '\u{c}' {
                                if !last_space {
                                    t.push(' ');
                                    last_space = true;
                                }
                            } else {
                                t.push(ch);
                                last_space = false;
                            }
                        }
                    }
                    _ => {
                        let tab = " ".repeat(style.tab_size.max(1.0) as usize);
                        t = text.replace('\t', &tab).replace("\r\n", "\n");
                        last_space = t.ends_with([' ', '\n']);
                    }
                }
                let t = transform(&t, style.text_transform, &mut word_start);
                if !t.is_empty() {
                    out.push(Item::Text {
                        text: t,
                        style,
                        span,
                    });
                }
            }
            Item::Atom { .. } => {
                last_space = false;
                word_start = false;
                out.push(item);
            }
            Item::Break { .. } => {
                // A space before a forced break hangs; after it, none starts
                // the line.
                if let Some(Item::Text { text, .. }) = out.last_mut() {
                    while text.ends_with(' ') {
                        text.pop();
                    }
                }
                last_space = true;
                out.push(item);
            }
        }
    }
    // No space at the very end.
    while let Some(Item::Text { text, .. }) = out.last_mut() {
        if collapses_last(text) {
            break;
        }
        out.pop();
    }
    out
}

/// Trim trailing collapsible spaces; true when text is left.
fn collapses_last(text: &mut String) -> bool {
    while text.ends_with(' ') {
        text.pop();
    }
    !text.is_empty()
}

// ─── Shaping ────────────────────────────────────────────────────────

impl Shaped {
    pub fn new(items: Vec<Item>, block: Rc<Style>, db: &mut FontDb) -> Shaped {
        let items = normalise(items);
        // The paragraph's text, and which item each byte belongs to.
        let mut text = String::new();
        let mut owner: Vec<(usize, usize)> = Vec::new(); // (start byte, item)
        for (ix, item) in items.iter().enumerate() {
            owner.push((text.len(), ix));
            match item {
                Item::Text { text: t, .. } => text.push_str(t),
                Item::Atom { .. } => text.push(ATOM),
                Item::Break { .. } => text.push('\n'),
            }
        }
        let item_at = |byte: usize| -> usize {
            match owner.binary_search_by(|(s, _)| s.cmp(&byte)) {
                Ok(i) => owner[i].1,
                Err(i) => owner[i.saturating_sub(1)].1,
            }
        };
        let base = match block.direction {
            Direction::Rtl => Some(Level::rtl()),
            Direction::Ltr => Some(Level::ltr()),
        };
        let bidi = BidiInfo::new(&text, base);
        let levels = bidi.levels.clone();

        // Break opportunities: a break is allowed *before* each index.
        let mut allowed = vec![false; text.len() + 1];
        let mut mandatory = vec![false; text.len() + 1];
        for (ix, op) in linebreaks(&text) {
            match op {
                BreakOpportunity::Mandatory => {
                    if ix < text.len() {
                        mandatory[ix] = true;
                    }
                }
                BreakOpportunity::Allowed => allowed[ix] = true,
            }
        }

        // Each character's face: the style's family, or a fallback that
        // has the glyph.
        let mut missing = Vec::new();
        let mut char_picks: Vec<(usize, Option<Pick>)> = Vec::new();
        let mut family_pick_cache: Vec<Option<Pick>> = vec![None; items.len()];
        for (byte, ch) in text.char_indices() {
            let ix = item_at(byte);
            let pick = match &items[ix] {
                Item::Text { style, .. } => {
                    let primary = *family_pick_cache[ix].get_or_insert_with(|| {
                        db.pick(&style.font_family, style.font_weight, style.italic)
                    });
                    if ch == '\n' || is_default_ignorable(ch) || db.covers(primary.face, ch) {
                        Some(primary)
                    } else if let Some(f) =
                        other_family_covering(db, &style.font_family, primary, ch, style)
                    {
                        Some(f)
                    } else if let Some(f) = db.fallback(ch, style.font_weight, style.italic) {
                        Some(f)
                    } else {
                        if !missing.contains(&ch) && !ch.is_whitespace() {
                            missing.push(ch);
                        }
                        Some(primary)
                    }
                }
                _ => None,
            };
            char_picks.push((byte, pick));
        }

        // Runs of one item, one level, one face.
        let mut clusters: Vec<Cluster> = Vec::new();
        let mut i = 0;
        while i < char_picks.len() {
            let (start, pick) = char_picks[i];
            let item = item_at(start);
            let level = levels[start].number();
            let mut j = i + 1;
            while j < char_picks.len() {
                let (b, p) = char_picks[j];
                if item_at(b) != item || levels[b].number() != level || p != pick {
                    break;
                }
                j += 1;
            }
            let end = char_picks.get(j).map(|c| c.0).unwrap_or(text.len());
            match &items[item] {
                Item::Atom { boxed, .. } => clusters.push(Cluster {
                    range: start..end,
                    glyphs: Vec::new(),
                    width: 0.0,
                    pick: None,
                    size: 0.0,
                    item,
                    level,
                    space: false,
                    break_after: false,
                    mandatory_after: false,
                    atom: Some(*boxed),
                    ascent: 0.0,
                    descent: 0.0,
                }),
                Item::Break { style } => clusters.push(Cluster {
                    range: start..end,
                    glyphs: Vec::new(),
                    width: 0.0,
                    pick: Some(db.pick(&style.font_family, style.font_weight, style.italic)),
                    size: style.font_size,
                    item,
                    level,
                    space: false,
                    break_after: false,
                    mandatory_after: true,
                    atom: None,
                    ascent: 0.0,
                    descent: 0.0,
                }),
                Item::Text { style, .. } => {
                    let pick = pick.expect("text has a face");
                    shape_run(
                        db,
                        &text,
                        start..end,
                        pick,
                        style,
                        level,
                        item,
                        &mut clusters,
                    );
                }
            }
            i = j;
        }

        // Break flags, and the metrics of the faces.
        for c in &mut clusters {
            c.break_after = allowed.get(c.range.end).copied().unwrap_or(false);
            if mandatory.get(c.range.end).copied().unwrap_or(false)
                || text[c.range.clone()].ends_with('\n')
            {
                c.mandatory_after = true;
            }
            if let Some(p) = c.pick {
                let f = db.face(p);
                let k = c.size / f.units_per_em;
                c.ascent = f.ascent * k;
                c.descent = f.descent * k;
            }
        }
        // An atom may break on either side, as a word does.
        let atom_positions: Vec<usize> = clusters
            .iter()
            .enumerate()
            .filter(|(_, c)| c.atom.is_some())
            .map(|(i, _)| i)
            .collect();
        for i in atom_positions {
            clusters[i].break_after = true;
            if i > 0 {
                clusters[i - 1].break_after = true;
            }
        }
        // No-wrap text breaks only where it must.
        for c in &mut clusters {
            if let Item::Text { style, .. } = &items[c.item] {
                if matches!(style.white_space, WhiteSpace::NoWrap | WhiteSpace::Pre) {
                    c.break_after = false;
                }
            }
        }
        Shaped {
            text,
            items,
            clusters,
            block,
            missing,
        }
    }

    pub fn style_of(&self, item: usize) -> &Rc<Style> {
        match &self.items[item] {
            Item::Text { style, .. } | Item::Atom { style, .. } | Item::Break { style } => style,
        }
    }

    /// The widest unbreakable piece: what the paragraph cannot be narrower
    /// than.
    pub fn min_content(&self, atoms: &mut AtomSizer) -> f32 {
        let mut best = 0.0f32;
        let mut cur = 0.0f32;
        for c in &self.clusters {
            let w = if let Some(a) = c.atom {
                atoms(a, 0.0).0
            } else {
                c.width
            };
            if !c.space {
                cur += w;
            }
            if c.break_after || c.mandatory_after || c.space {
                best = best.max(cur);
                cur = 0.0;
            }
        }
        best.max(cur)
    }

    /// The paragraph on one line (or as many as its forced breaks make).
    pub fn max_content(&self, atoms: &mut AtomSizer) -> f32 {
        let mut best = 0.0f32;
        let mut cur = 0.0f32;
        for c in &self.clusters {
            cur += if let Some(a) = c.atom {
                atoms(a, f32::INFINITY).0
            } else {
                c.width
            };
            if c.mandatory_after {
                best = best.max(cur);
                cur = 0.0;
            }
        }
        best.max(cur)
    }

    /// Break into lines of at most `width` points.
    pub fn lines(&self, width: f32, atoms: &mut AtomSizer) -> Vec<Line> {
        // The width of each cluster at this width (an atom's may depend on it).
        let widths: Vec<f32> = self
            .clusters
            .iter()
            .map(|c| {
                if let Some(a) = c.atom {
                    atoms(a, width).0
                } else {
                    c.width
                }
            })
            .collect();
        let mut ranges: Vec<(usize, usize)> = Vec::new();
        let mut start = 0;
        while start < self.clusters.len() {
            // Spaces at the start of a line (after a soft break) collapse.
            while start < self.clusters.len()
                && self.clusters[start].space
                && collapses(self.style_of(self.clusters[start].item).white_space)
                && !ranges.is_empty()
            {
                start += 1;
            }
            if start >= self.clusters.len() {
                break;
            }
            let mut w = 0.0f32;
            let mut last_break: Option<usize> = None;
            let mut end = start;
            let mut forced = false;
            while end < self.clusters.len() {
                let c = &self.clusters[end];
                let cw = widths[end];
                // A space never makes a line overflow: it hangs.
                if w + cw > width + 0.01 && !c.space && end > start {
                    break;
                }
                w += cw;
                end += 1;
                if c.mandatory_after {
                    forced = true;
                    break;
                }
                if c.break_after {
                    last_break = Some(end);
                }
            }
            if !forced && end < self.clusters.len() {
                if let Some(b) = last_break.filter(|&b| b > start) {
                    end = b;
                }
                // No opportunity at all: the word is cut where it overflows
                // (CSS's `overflow-wrap: anywhere`), so text never leaves its box.
            }
            if end == start {
                end = start + 1;
            }
            ranges.push((start, end));
            start = end;
        }
        let mut lines = Vec::new();
        let mut top = 0.0f32;
        let count = ranges.len();
        for (n, (a, b)) in ranges.into_iter().enumerate() {
            let last = n + 1 == count || self.clusters[b - 1].mandatory_after;
            let line = self.build_line(a, b, width, &widths, last, top, atoms);
            top += line.height;
            lines.push(line);
        }
        if lines.is_empty() {
            // An empty paragraph is a strut high, as a browser's empty
            // line is not: only when it holds a forced break.
            let _ = top;
        }
        lines
    }

    #[allow(clippy::too_many_arguments)]
    fn build_line(
        &self,
        a: usize,
        b: usize,
        width: f32,
        widths: &[f32],
        last: bool,
        top: f32,
        atoms: &mut AtomSizer,
    ) -> Line {
        // Trailing spaces hang outside the line.
        let mut content_end = b;
        while content_end > a
            && (self.clusters[content_end - 1].space
                || self.clusters[content_end - 1].mandatory_after
                    && self.clusters[content_end - 1].glyphs.is_empty()
                    && self.clusters[content_end - 1].atom.is_none())
        {
            content_end -= 1;
        }
        let logical: Vec<usize> = (a..content_end).collect();
        let used: f32 = logical.iter().map(|&i| widths[i]).sum();

        // Visual order: reverse every maximal run at or above each level,
        // from the highest down to the lowest odd one (UAX #9, L2).
        let mut order = logical.clone();
        let max_level = order
            .iter()
            .map(|&i| self.clusters[i].level)
            .max()
            .unwrap_or(0);
        let min_odd = order
            .iter()
            .map(|&i| self.clusters[i].level)
            .filter(|l| l % 2 == 1)
            .min()
            .unwrap_or(max_level + 1);
        let mut level = max_level;
        while level >= min_odd && level > 0 {
            let mut i = 0;
            while i < order.len() {
                if self.clusters[order[i]].level >= level {
                    let mut j = i;
                    while j < order.len() && self.clusters[order[j]].level >= level {
                        j += 1;
                    }
                    order[i..j].reverse();
                    i = j;
                } else {
                    i += 1;
                }
            }
            level -= 1;
        }

        // Alignment.
        let rtl = self.block.direction == Direction::Rtl;
        let align = match (self.block.text_align, rtl) {
            (TextAlign::Start, false) | (TextAlign::End, true) | (TextAlign::Left, _) => 0.0,
            (TextAlign::Start, true) | (TextAlign::End, false) | (TextAlign::Right, _) => 1.0,
            (TextAlign::Center, _) => 0.5,
            (TextAlign::Justify, _) => {
                if rtl {
                    1.0
                } else {
                    0.0
                }
            }
        };
        let free = (width - used).max(0.0);
        let justify =
            self.block.text_align == TextAlign::Justify && !last && free > 0.0 && width.is_finite();
        let spaces = if justify {
            order.iter().filter(|&&i| self.clusters[i].space).count()
        } else {
            0
        };
        let extra_per_space = if spaces > 0 {
            free / spaces as f32
        } else {
            0.0
        };
        let mut x = if justify && spaces > 0 || !width.is_finite() {
            0.0
        } else {
            free * align
        };

        // Vertical metrics: the strut, then every piece on the line.
        let strut_lh = self.block.line_height_pt();
        let (strut_a, strut_d) = self.strut_metrics();
        let half = (strut_lh - (strut_a + strut_d)) / 2.0;
        let mut above = strut_a + half;
        let mut below = strut_d + half;

        let mut runs: Vec<Run> = Vec::new();
        let mut placed: Vec<(usize, f32, f32, f32, f32, VerticalAlign)> = Vec::new(); // atom, x, w, h, baseline, valign
        let mut spans: Vec<SpanExtent> = Vec::new();
        for &ci in &order {
            let c = &self.clusters[ci];
            let w = widths[ci] + if c.space { extra_per_space } else { 0.0 };
            let style = self.style_of(c.item);
            if let Some(boxed) = c.atom {
                let (aw, ah, abase) = atoms(boxed, width);
                placed.push((boxed, x, aw, ah, abase, style.vertical_align));
                match style.vertical_align {
                    VerticalAlign::Middle => {
                        let mid = strut_a * 0.3;
                        above = above.max(ah / 2.0 + mid);
                        below = below.max(ah / 2.0 - mid);
                    }
                    VerticalAlign::Top | VerticalAlign::Bottom => {
                        above = above.max(ah);
                    }
                    _ => {
                        above = above.max(abase);
                        below = below.max(ah - abase);
                    }
                }
            } else if let Some(pick) = c.pick {
                let rise = match style.vertical_align {
                    VerticalAlign::Super => style.font_size * 0.4,
                    VerticalAlign::Sub => -style.font_size * 0.2,
                    _ => 0.0,
                };
                let lh = style.line_height_pt();
                let hl = (lh - (c.ascent + c.descent)) / 2.0;
                above = above.max(c.ascent + hl + rise);
                below = below.max(c.descent + hl - rise);
                let same = runs.last().is_some_and(|r| {
                    r.pick == pick
                        && r.item == c.item
                        && (r.x + r.width - x).abs() < 0.01
                        && r.size == c.size
                });
                if !same {
                    runs.push(Run {
                        pick,
                        size: c.size,
                        item: c.item,
                        x,
                        width: 0.0,
                        glyphs: Vec::new(),
                        rise,
                    });
                }
                let run = runs.last_mut().unwrap();
                // Justification widens the space glyph's advance.
                let mut glyphs = c.glyphs.clone();
                if c.space && extra_per_space > 0.0 {
                    if let Some(g) = glyphs.last_mut() {
                        g.advance += extra_per_space;
                    }
                }
                run.glyphs.extend(glyphs);
                run.width += w;
            }
            if let Item::Text {
                span: Some(span), ..
            } = &self.items[c.item]
            {
                match spans.last_mut() {
                    Some(s) if s.span == *span && (s.x + s.width - x).abs() < 0.01 => s.width += w,
                    _ => spans.push(SpanExtent {
                        span: *span,
                        x,
                        width: w,
                    }),
                }
            }
            x += w;
        }
        let height = (above + below).max(0.0);
        let baseline = above;
        let atoms_out = placed
            .into_iter()
            .map(|(boxed, ax, w, h, abase, valign)| PlacedAtom {
                boxed,
                x: ax,
                y: match valign {
                    VerticalAlign::Middle => baseline - strut_a * 0.3 - h / 2.0,
                    VerticalAlign::Top => 0.0,
                    VerticalAlign::Bottom => height - h,
                    _ => baseline - abase,
                },
                width: w,
                height: h,
            })
            .collect();
        Line {
            top,
            height,
            baseline,
            width: used,
            runs,
            atoms: atoms_out,
            spans,
        }
    }

    fn strut_metrics(&self) -> (f32, f32) {
        // The block's first face at its size.
        for c in &self.clusters {
            if c.pick.is_some() && c.size == self.block.font_size {
                return (c.ascent, c.descent);
            }
        }
        (self.block.font_size * 0.9, self.block.font_size * 0.212)
    }
}

/// A later family in the list that has the character, at the style's weight.
fn other_family_covering(
    db: &mut FontDb,
    families: &[String],
    primary: Pick,
    ch: char,
    style: &Style,
) -> Option<Pick> {
    for (i, _) in families.iter().enumerate().skip(1) {
        let p = db.pick(&families[i..], style.font_weight, style.italic);
        if p.face != primary.face && db.covers(p.face, ch) {
            return Some(p);
        }
    }
    None
}

fn is_default_ignorable(ch: char) -> bool {
    matches!(ch, '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{2064}' | '\u{fe00}'..='\u{fe0f}' | '\u{feff}')
}

#[allow(clippy::too_many_arguments)]
fn shape_run(
    db: &FontDb,
    text: &str,
    range: std::ops::Range<usize>,
    pick: Pick,
    style: &Style,
    level: u8,
    item: usize,
    out: &mut Vec<Cluster>,
) {
    let face_data = db.face(pick);
    let Some(mut face) = rustybuzz::Face::from_slice(&face_data.data, face_data.index) else {
        return;
    };
    if face_data.variable {
        face.set_variations(&[rustybuzz::Variation {
            tag: rustybuzz::ttf_parser::Tag::from_bytes(b"wght"),
            value: pick.weight as f32,
        }]);
    }
    let size = style.font_size;
    let scale = size / face_data.units_per_em;
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    let slice = &text[range.clone()];
    // A newline is shaped as nothing: it only ends the line.
    buffer.push_str(&slice.replace('\n', " "));
    buffer.set_direction(if level % 2 == 1 {
        rustybuzz::Direction::RightToLeft
    } else {
        rustybuzz::Direction::LeftToRight
    });
    buffer.guess_segment_properties();
    let features: Vec<rustybuzz::Feature> = Vec::new();
    let shaped = rustybuzz::shape(&face, &features, buffer);
    let infos = shaped.glyph_infos();
    let positions = shaped.glyph_positions();
    // Group the glyphs by cluster, in logical order.
    let mut groups: Vec<(usize, Vec<Glyph>)> = Vec::new();
    for (info, pos) in infos.iter().zip(positions) {
        let cluster = info.cluster as usize;
        let glyph = Glyph {
            id: info.glyph_id as u16,
            advance: pos.x_advance as f32 * scale,
            x_offset: pos.x_offset as f32 * scale,
            y_offset: pos.y_offset as f32 * scale,
            range: 0..0,
        };
        match groups.iter_mut().find(|(c, _)| *c == cluster) {
            Some((_, g)) => g.push(glyph),
            None => groups.push((cluster, vec![glyph])),
        }
    }
    groups.sort_by_key(|(c, _)| *c);
    for (n, (cluster, mut glyphs)) in groups.iter().cloned().enumerate() {
        let start = range.start + cluster;
        let end = groups
            .get(n + 1)
            .map(|(c, _)| range.start + c)
            .unwrap_or(range.end);
        let piece = &text[start..end];
        let space = piece
            .chars()
            .all(|c| c == ' ' || c == '\u{a0}' || c == '\n')
            && !piece.is_empty();
        let mut width: f32 = glyphs.iter().map(|g| g.advance).sum();
        let chars = piece.chars().count() as f32;
        width += style.letter_spacing * chars;
        if space && piece.contains(' ') {
            width += style.word_spacing;
        }
        // The glyphs carry the spacing, so what is drawn is what is measured.
        if let Some(g) = glyphs.last_mut() {
            g.advance +=
                style.letter_spacing * chars + if space { style.word_spacing } else { 0.0 };
        }
        for g in &mut glyphs {
            g.range = start..end;
        }
        out.push(Cluster {
            range: start..end,
            glyphs,
            width,
            pick: Some(pick),
            size,
            item,
            level,
            space,
            break_after: false,
            mandatory_after: piece.ends_with('\n'),
            atom: None,
            ascent: 0.0,
            descent: 0.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> Rc<Style> {
        Rc::new(Style::root(12.0, "sans-serif"))
    }

    fn text(t: &str) -> Item {
        Item::Text {
            text: t.to_string(),
            style: style(),
            span: None,
        }
    }

    fn no_atoms() -> impl FnMut(usize, f32) -> (f32, f32, f32) {
        |_, _| (0.0, 0.0, 0.0)
    }

    fn line_texts(s: &Shaped, lines: &[Line]) -> Vec<String> {
        lines
            .iter()
            .map(|l| {
                let mut out = String::new();
                for r in &l.runs {
                    let mut last = usize::MAX;
                    for g in &r.glyphs {
                        if g.range.start != last {
                            out.push_str(&s.text[g.range.clone()]);
                            last = g.range.start;
                        }
                    }
                }
                out.trim_end().to_string()
            })
            .collect()
    }

    #[test]
    fn whitespace_collapses_across_items() {
        let items = normalise(vec![text("  Hello   "), text("  world \n"), text("!")]);
        let joined: Vec<String> = items
            .iter()
            .map(|i| match i {
                Item::Text { text, .. } => text.clone(),
                _ => String::new(),
            })
            .collect();
        assert_eq!(joined.concat(), "Hello world !");
    }

    #[test]
    fn breaks_at_spaces_and_measures_what_it_draws() {
        let mut db = FontDb::new(false);
        let s = Shaped::new(
            vec![text("The quick brown fox jumps over the lazy dog")],
            style(),
            &mut db,
        );
        let mut atoms = no_atoms();
        let one = s.lines(10_000.0, &mut atoms);
        assert_eq!(one.len(), 1);
        let full = one[0].width;
        // Helvetica-metric text at 12pt: the pangram is about 240pt wide.
        assert!((200.0..280.0).contains(&full), "{full}");
        let lines = s.lines(full / 2.0 + 10.0, &mut atoms);
        assert_eq!(lines.len(), 2);
        let texts = line_texts(&s, &lines);
        assert!(texts[0].starts_with("The quick"));
        assert!(lines.iter().all(|l| l.width <= full / 2.0 + 10.0 + 0.01));
        // Lines stack, each a line-height apart.
        assert!((lines[1].top - lines[0].height).abs() < 0.01);
        assert!(s.min_content(&mut atoms) < 60.0);
        assert!((s.max_content(&mut atoms) - full).abs() < 0.5);
    }

    #[test]
    fn a_word_too_long_for_the_line_is_cut_not_overflowed() {
        let mut db = FontDb::new(false);
        let s = Shaped::new(
            vec![text(
                "https://example.com/a/very/long/path/that/never/breaks",
            )],
            style(),
            &mut db,
        );
        let mut atoms = no_atoms();
        let lines = s.lines(80.0, &mut atoms);
        assert!(lines.len() > 2);
        assert!(
            lines.iter().all(|l| l.width <= 80.01),
            "{:?}",
            lines.iter().map(|l| l.width).collect::<Vec<_>>()
        );
    }

    #[test]
    fn alignment_and_justification() {
        let mut db = FontDb::new(false);
        let mut st = Style::root(12.0, "sans-serif");
        st.text_align = TextAlign::Right;
        let s = Shaped::new(vec![text("right")], Rc::new(st.clone()), &mut db);
        let mut atoms = no_atoms();
        let l = &s.lines(200.0, &mut atoms)[0];
        assert!((l.runs[0].x + l.width - 200.0).abs() < 0.5);
        st.text_align = TextAlign::Justify;
        let s = Shaped::new(
            vec![text("aa bb cc dd ee ff gg hh ii jj kk ll mm nn")],
            Rc::new(st),
            &mut db,
        );
        let lines = s.lines(100.0, &mut atoms);
        // Every line but the last fills the width.
        let first = &lines[0];
        let end = first.runs.last().map(|r| r.x + r.width).unwrap();
        assert!(end > 98.0 && end < 100.5, "{end}");
    }

    #[test]
    fn forced_breaks_and_atoms() {
        let mut db = FontDb::new(false);
        let items = vec![
            text("one"),
            Item::Break { style: style() },
            text("two "),
            Item::Atom {
                boxed: 7,
                style: style(),
            },
            text(" three"),
        ];
        let s = Shaped::new(items, style(), &mut db);
        let mut atoms = |_: usize, _: f32| (30.0f32, 20.0f32, 15.0f32);
        let lines = s.lines(1000.0, &mut atoms);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].atoms.len(), 1);
        let a = &lines[1].atoms[0];
        assert_eq!(a.boxed, 7);
        // The atom sits on the baseline: its baseline at the line's.
        assert!((a.y + 15.0 - lines[1].baseline).abs() < 0.01);
        assert!(lines[1].height >= 20.0);
    }

    #[test]
    fn right_to_left_runs_are_reversed_for_display() {
        let mut db = FontDb::new(false);
        // Hebrew is in Liberation; the RTL run's clusters display reversed.
        let s = Shaped::new(vec![text("abc שלום def")], style(), &mut db);
        let mut atoms = no_atoms();
        let line = &s.lines(1000.0, &mut atoms)[0];
        let mut visual = String::new();
        for r in &line.runs {
            let mut last = usize::MAX;
            for g in &r.glyphs {
                if g.range.start != last {
                    visual.push_str(&s.text[g.range.clone()]);
                    last = g.range.start;
                }
            }
        }
        assert_eq!(visual, "abc םולש def");
        assert!(s.missing.is_empty());
    }

    #[test]
    fn a_character_no_face_has_is_reported() {
        let mut db = FontDb::new(false);
        let s = Shaped::new(vec![text("مرحبا")], style(), &mut db);
        assert!(s.missing.contains(&'م'));
    }
}
