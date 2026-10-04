//! The cascade: which declarations reach each element, in what order, and the
//! computed style they make together with what the parent passes down.

use std::collections::HashMap;
use std::rc::Rc;

use super::css::{
    AttrOp, AttrTest, Combinator, Compound, Declaration, Pseudo, Selector, Sheet, parse_declarations,
    split_spaces, split_top_level,
};
use super::html::{Dom, Element, NodeKind};
use super::style::*;

/// What a browser draws before any author's rule: the display of each tag
/// and the few typographic defaults a page relies on.
pub const UA_SHEET: &str = r#"
html, body, div, section, article, header, footer, main, nav, aside, figure, figcaption,
p, h1, h2, h3, h4, h5, h6, blockquote, pre, ul, ol, dl, dd, dt, form, fieldset, address,
details, summary, hr, center, legend { display: block }
head, script, style, title, meta, link, template, noscript, [hidden] { display: none }
li { display: list-item }
table { display: table; border-collapse: separate }
thead { display: table-header-group }
tbody { display: table-row-group }
tfoot { display: table-footer-group }
tr { display: table-row }
td, th { display: table-cell; vertical-align: middle; padding: 1px }
caption { display: table-caption }
img, svg, video, canvas, input, button, select, textarea, progress, meter { display: inline-block }
h1 { font-size: 2em; font-weight: bold; margin: 0.67em 0 }
h2 { font-size: 1.5em; font-weight: bold; margin: 0.83em 0 }
h3 { font-size: 1.17em; font-weight: bold; margin: 1em 0 }
h4 { font-weight: bold; margin: 1.33em 0 }
h5 { font-size: 0.83em; font-weight: bold; margin: 1.67em 0 }
h6 { font-size: 0.67em; font-weight: bold; margin: 2.33em 0 }
p, ul, ol, blockquote, figure, dl, pre { margin: 1em 0 }
ul, ol { padding-left: 40px }
ol { list-style-type: decimal }
ul ul { list-style-type: circle }
b, strong, th { font-weight: bold }
i, em, cite, dfn, var, address { font-style: italic }
code, kbd, samp, pre, tt { font-family: monospace }
pre { white-space: pre }
small { font-size: 0.83em }
sub { vertical-align: sub; font-size: 0.83em }
sup { vertical-align: super; font-size: 0.83em }
u, ins { text-decoration: underline }
s, strike, del { text-decoration: line-through }
mark { background: #ff0; color: #000 }
hr { border-top: 1px solid #888; margin: 0.5em 0 }
blockquote { margin: 1em 40px }
center { text-align: center }
a { color: inherit }
[dir="rtl"] { direction: rtl }
[dir="ltr"] { direction: ltr }
"#;

/// The sheets of a page, indexed for matching.
pub struct Styler {
    rules: Vec<IndexedRule>,
    /// Rules by the rightmost compound's first class, id, or tag; rules with
    /// none of those (`*`, `[attr]`) in `universal`.
    by_class: HashMap<String, Vec<usize>>,
    by_id: HashMap<String, Vec<usize>>,
    by_tag: HashMap<String, Vec<usize>>,
    universal: Vec<usize>,
    pub units: Units,
}

struct IndexedRule {
    selector: Selector,
    declarations: Rc<Vec<Declaration>>,
    order: usize,
    /// 0 for the browser's defaults, 1 for the page's sheets.
    origin: u8,
}

impl Styler {
    pub fn new(ua: &Sheet, author: &Sheet, units: Units) -> Styler {
        let mut s = Styler {
            rules: Vec::new(),
            by_class: HashMap::new(),
            by_id: HashMap::new(),
            by_tag: HashMap::new(),
            universal: Vec::new(),
            units,
        };
        for (origin, sheet) in [(0u8, ua), (1u8, author)] {
            for rule in &sheet.rules {
                let decls = Rc::new(rule.declarations.clone());
                for sel in &rule.selectors {
                    let ix = s.rules.len();
                    let order = ix;
                    let last = &sel.parts.last().expect("a selector has a compound").0;
                    if let Some(id) = &last.id {
                        s.by_id.entry(id.clone()).or_default().push(ix);
                    } else if let Some(class) = last.classes.first() {
                        s.by_class.entry(class.clone()).or_default().push(ix);
                    } else if let Some(tag) = &last.tag {
                        s.by_tag.entry(tag.clone()).or_default().push(ix);
                    } else {
                        s.universal.push(ix);
                    }
                    s.rules.push(IndexedRule {
                        selector: sel.clone(),
                        declarations: decls.clone(),
                        order,
                        origin,
                    });
                }
            }
        }
        s
    }

    /// The declarations that reach `node`, in cascade order (last wins), for
    /// the element itself or one of its generated boxes.
    fn matched(&self, dom: &Dom, node: usize, pseudo: Option<Pseudo>) -> Vec<Declaration> {
        let Some(el) = dom.element(node) else {
            return Vec::new();
        };
        let mut candidates: Vec<usize> = Vec::new();
        if let Some(id) = el.attr("id") {
            if let Some(v) = self.by_id.get(id) {
                candidates.extend(v);
            }
        }
        for class in el.classes() {
            if let Some(v) = self.by_class.get(class) {
                candidates.extend(v);
            }
        }
        if let Some(v) = self.by_tag.get(&el.tag) {
            candidates.extend(v);
        }
        candidates.extend(&self.universal);
        candidates.sort_unstable();
        candidates.dedup();
        // (important, origin, specificity, order) — later wins.
        let mut hits: Vec<(bool, u8, (u32, u32, u32), usize, &Declaration)> = Vec::new();
        for ix in candidates {
            let rule = &self.rules[ix];
            if !matches(dom, node, &rule.selector, pseudo.as_ref()) {
                continue;
            }
            for d in rule.declarations.iter() {
                hits.push((d.important, rule.origin, rule.selector.specificity, rule.order, d));
            }
        }
        // The `style` attribute: above every sheet.
        let inline = if pseudo.is_none() {
            el.attr("style").map(parse_declarations).unwrap_or_default()
        } else {
            Vec::new()
        };
        for d in &inline {
            hits.push((d.important, 2, (1000, 0, 0), usize::MAX, d));
        }
        hits.sort_by(|a, b| (a.0, a.1, a.2, a.3).cmp(&(b.0, b.1, b.2, b.3)));
        hits.into_iter().map(|h| h.4.clone()).collect()
    }

    /// The computed style of `node`, given its parent's.
    pub fn compute(&self, dom: &Dom, node: usize, parent: &Style) -> Style {
        let decls = self.matched(dom, node, None);
        self.apply_all(parent, &decls)
    }

    /// The style of a `::before`/`::after` box, or `None` when no rule gives
    /// it content.
    pub fn compute_pseudo(&self, dom: &Dom, node: usize, parent: &Style, before: bool) -> Option<Style> {
        let pseudo = if before { Pseudo::Before } else { Pseudo::After };
        let decls = self.matched(dom, node, Some(pseudo));
        if decls.is_empty() {
            return None;
        }
        let style = self.apply_all(parent, &decls);
        style.content.as_ref()?;
        Some(style)
    }

    pub fn apply_all(&self, parent: &Style, decls: &[Declaration]) -> Style {
        let mut style = parent.inherit();
        // Custom properties first: every other value may read them.
        let mut vars: Option<HashMap<String, String>> = None;
        for d in decls {
            if d.name.starts_with("--") {
                vars.get_or_insert_with(|| (*parent.vars).clone())
                    .insert(d.name.clone(), d.value.clone());
            }
        }
        if let Some(v) = vars {
            style.vars = Rc::new(v);
        }
        // The font size next: `em` everywhere else means it.
        for d in decls.iter().filter(|d| d.name == "font-size" || d.name == "font") {
            let value = substitute_vars(&d.value, &style.vars);
            self.apply(&mut style, parent, &d.name, &value);
        }
        for d in decls {
            if d.name.starts_with("--") || d.name == "font-size" {
                continue;
            }
            let value = substitute_vars(&d.value, &style.vars);
            self.apply(&mut style, parent, &d.name, &value);
        }
        style
    }

    fn apply(&self, s: &mut Style, parent: &Style, name: &str, value: &str) {
        let value = value.trim();
        let u = Units {
            em: s.font_size,
            ..self.units
        };
        // `inherit`, `initial`, `unset` on the inherited properties that
        // matter; elsewhere they fall to the initial value.
        if value == "inherit" {
            inherit_one(s, parent, name);
            return;
        }
        let len = |v: &str| length(v, &u);
        let col = |v: &str| -> Option<Option<Rgba>> {
            if v.eq_ignore_ascii_case("currentcolor") {
                Some(None)
            } else {
                color(v).map(Some)
            }
        };
        match name {
            "color" => {
                if let Some(c) = col(value) {
                    s.color = c.unwrap_or(parent.color);
                }
            }
            "font-family" => s.font_family = Rc::new(parse_family_list(value)),
            "font-size" => {
                let pu = Units { em: parent.font_size, ..self.units };
                let size = match value {
                    "xx-small" => Some(Length::Pt(7.0)),
                    "x-small" => Some(Length::Pt(7.5)),
                    "small" => Some(Length::Pt(10.0)),
                    "medium" => Some(Length::Pt(12.0)),
                    "large" => Some(Length::Pt(13.5)),
                    "x-large" => Some(Length::Pt(18.0)),
                    "xx-large" => Some(Length::Pt(24.0)),
                    "smaller" => Some(Length::Pt(parent.font_size / 1.2)),
                    "larger" => Some(Length::Pt(parent.font_size * 1.2)),
                    v => length(v, &pu),
                };
                match size {
                    Some(Length::Pt(p)) => s.font_size = p.max(0.5),
                    Some(Length::Pct(p)) => s.font_size = parent.font_size * p / 100.0,
                    _ => {}
                }
            }
            "font-weight" => {
                if let Some(w) = font_weight(value, parent.font_weight) {
                    s.font_weight = w;
                }
            }
            "font-style" => s.italic = value.starts_with("italic") || value.starts_with("oblique"),
            "font" => self.font_shorthand(s, parent, value),
            "line-height" => {
                s.line_height = if value == "normal" {
                    LineHeight::Normal
                } else if let Ok(f) = value.parse::<f32>() {
                    LineHeight::Factor(f)
                } else {
                    match len(value) {
                        Some(Length::Pt(p)) => LineHeight::Pt(p),
                        Some(Length::Pct(p)) => LineHeight::Factor(p / 100.0),
                        _ => s.line_height,
                    }
                }
            }
            "letter-spacing" => {
                s.letter_spacing = if value == "normal" { 0.0 } else { length_pt(value, &u).unwrap_or(0.0) }
            }
            "word-spacing" => {
                s.word_spacing = if value == "normal" { 0.0 } else { length_pt(value, &u).unwrap_or(0.0) }
            }
            "text-align" => {
                s.text_align = match value {
                    "left" => TextAlign::Left,
                    "right" => TextAlign::Right,
                    "center" | "-webkit-center" => TextAlign::Center,
                    "justify" => TextAlign::Justify,
                    "end" => TextAlign::End,
                    _ => TextAlign::Start,
                }
            }
            "text-transform" => {
                s.text_transform = match value {
                    "uppercase" => TextTransform::Upper,
                    "lowercase" => TextTransform::Lower,
                    "capitalize" => TextTransform::Capitalize,
                    _ => TextTransform::None,
                }
            }
            "text-decoration" | "text-decoration-line" => {
                s.underline = value.contains("underline");
                s.line_through = value.contains("line-through");
                s.overline = value.contains("overline");
                if name == "text-decoration" {
                    for w in split_spaces(value) {
                        if let Some(c) = color(w) {
                            s.decoration_color = Some(c);
                        }
                    }
                }
            }
            "text-decoration-color" => s.decoration_color = color(value),
            "white-space" => {
                s.white_space = match value {
                    "nowrap" => WhiteSpace::NoWrap,
                    "pre" => WhiteSpace::Pre,
                    "pre-wrap" | "break-spaces" => WhiteSpace::PreWrap,
                    "pre-line" => WhiteSpace::PreLine,
                    _ => WhiteSpace::Normal,
                }
            }
            "direction" => s.direction = if value == "rtl" { Direction::Rtl } else { Direction::Ltr },
            "list-style" | "list-style-type" => {
                for w in split_spaces(value) {
                    if let Some(ls) = list_style(w) {
                        s.list_style = ls;
                    }
                }
            }
            "visibility" => s.visible = value == "visible",
            "orphans" => s.orphans = value.parse().unwrap_or(2),
            "widows" => s.widows = value.parse().unwrap_or(2),
            "border-collapse" => s.border_collapse = value == "collapse",
            "tab-size" => s.tab_size = value.parse().unwrap_or(8.0),

            "display" => {
                s.display = match value {
                    "none" => Display::None,
                    "inline" => Display::Inline,
                    "inline-block" => Display::InlineBlock,
                    "flex" => Display::Flex,
                    "inline-flex" => Display::InlineFlex,
                    "grid" => Display::Grid,
                    "inline-grid" => Display::InlineGrid,
                    "list-item" => Display::ListItem,
                    "table" | "inline-table" => Display::Table,
                    "table-header-group" => Display::TableHeaderGroup,
                    "table-row-group" => Display::TableRowGroup,
                    "table-footer-group" => Display::TableFooterGroup,
                    "table-row" => Display::TableRow,
                    "table-cell" => Display::TableCell,
                    "table-caption" => Display::TableCaption,
                    "contents" => Display::Contents,
                    _ => Display::Block,
                }
            }
            "position" => {
                s.position = match value {
                    "relative" | "sticky" => Position::Relative,
                    "absolute" => Position::Absolute,
                    "fixed" => Position::Fixed,
                    _ => Position::Static,
                }
            }
            "top" => s.inset[0] = len(value).unwrap_or(Length::Auto),
            "right" => s.inset[1] = len(value).unwrap_or(Length::Auto),
            "bottom" => s.inset[2] = len(value).unwrap_or(Length::Auto),
            "left" => s.inset[3] = len(value).unwrap_or(Length::Auto),
            "inset" => {
                let v: Vec<Length> = split_spaces(value).iter().filter_map(|w| len(w)).collect();
                if let Some(f) = four(&v) {
                    s.inset = f;
                }
            }
            "width" => s.width = len(value).unwrap_or(Length::Auto),
            "height" => s.height = len(value).unwrap_or(Length::Auto),
            "min-width" => s.min_width = len(value).unwrap_or(Length::Auto),
            "min-height" => s.min_height = len(value).unwrap_or(Length::Auto),
            "max-width" => s.max_width = len(value).unwrap_or(Length::Auto),
            "max-height" => s.max_height = len(value).unwrap_or(Length::Auto),
            "inline-size" => s.width = len(value).unwrap_or(Length::Auto),
            "block-size" => s.height = len(value).unwrap_or(Length::Auto),
            "box-sizing" => s.border_box = value == "border-box",
            "aspect-ratio" => {
                s.aspect_ratio = match value.split_once('/') {
                    Some((a, b)) => a.trim().parse::<f32>().ok().zip(b.trim().parse::<f32>().ok()).map(|(a, b)| a / b),
                    None => value.parse().ok(),
                }
            }
            "margin" | "padding" => {
                let v: Vec<Length> = split_spaces(value).iter().filter_map(|w| len(w)).collect();
                if let Some(f) = four(&v) {
                    if name == "margin" {
                        s.margin = f;
                    } else {
                        s.padding = f;
                    }
                }
            }
            "margin-top" => s.margin[0] = len(value).unwrap_or(Length::Pt(0.0)),
            "margin-right" => s.margin[1] = len(value).unwrap_or(Length::Pt(0.0)),
            "margin-bottom" => s.margin[2] = len(value).unwrap_or(Length::Pt(0.0)),
            "margin-left" => s.margin[3] = len(value).unwrap_or(Length::Pt(0.0)),
            "padding-top" => s.padding[0] = len(value).unwrap_or(Length::Pt(0.0)),
            "padding-right" => s.padding[1] = len(value).unwrap_or(Length::Pt(0.0)),
            "padding-bottom" => s.padding[2] = len(value).unwrap_or(Length::Pt(0.0)),
            "padding-left" => s.padding[3] = len(value).unwrap_or(Length::Pt(0.0)),
            // The logical properties, read for a left-to-right page.
            "margin-block" | "padding-block" | "margin-inline" | "padding-inline" => {
                let v: Vec<Length> = split_spaces(value).iter().filter_map(|w| len(w)).collect();
                let (a, b) = match v.as_slice() {
                    [a] => (*a, *a),
                    [a, b] => (*a, *b),
                    _ => return,
                };
                let sides = if name.ends_with("block") { (0, 2) } else { (3, 1) };
                let target = if name.starts_with("margin") { &mut s.margin } else { &mut s.padding };
                target[sides.0] = a;
                target[sides.1] = b;
            }
            "margin-block-start" => s.margin[0] = len(value).unwrap_or(Length::Pt(0.0)),
            "margin-block-end" => s.margin[2] = len(value).unwrap_or(Length::Pt(0.0)),
            "margin-inline-start" => s.margin[3] = len(value).unwrap_or(Length::Pt(0.0)),
            "margin-inline-end" => s.margin[1] = len(value).unwrap_or(Length::Pt(0.0)),
            "padding-block-start" => s.padding[0] = len(value).unwrap_or(Length::Pt(0.0)),
            "padding-block-end" => s.padding[2] = len(value).unwrap_or(Length::Pt(0.0)),
            "padding-inline-start" => s.padding[3] = len(value).unwrap_or(Length::Pt(0.0)),
            "padding-inline-end" => s.padding[1] = len(value).unwrap_or(Length::Pt(0.0)),

            "border" => {
                let side = border_side(value, &u);
                s.border = [side; 4];
            }
            "border-top" => s.border[0] = border_side(value, &u),
            "border-right" => s.border[1] = border_side(value, &u),
            "border-bottom" => s.border[2] = border_side(value, &u),
            "border-left" => s.border[3] = border_side(value, &u),
            "border-block" => {
                let b = border_side(value, &u);
                s.border[0] = b;
                s.border[2] = b;
            }
            "border-inline" => {
                let b = border_side(value, &u);
                s.border[1] = b;
                s.border[3] = b;
            }
            "border-block-start" => s.border[0] = border_side(value, &u),
            "border-block-end" => s.border[2] = border_side(value, &u),
            "border-inline-start" => s.border[3] = border_side(value, &u),
            "border-inline-end" => s.border[1] = border_side(value, &u),
            "border-width" => {
                let v: Vec<f32> = split_spaces(value).iter().filter_map(|w| border_width(w, &u)).collect();
                if let Some(f) = four(&v) {
                    for i in 0..4 {
                        s.border[i].width = f[i];
                    }
                }
            }
            "border-style" => {
                let v: Vec<BorderStyle> = split_spaces(value).iter().map(|w| border_style(w)).collect();
                if let Some(f) = four(&v) {
                    for i in 0..4 {
                        s.border[i].style = f[i];
                    }
                }
            }
            "border-color" => {
                let v: Vec<Option<Rgba>> = split_spaces(value).iter().filter_map(|w| col(w)).collect();
                if let Some(f) = four(&v) {
                    for i in 0..4 {
                        s.border[i].color = f[i];
                    }
                }
            }
            "border-top-color" | "border-right-color" | "border-bottom-color" | "border-left-color" => {
                if let Some(c) = col(value) {
                    s.border[side_index(name)].color = c;
                }
            }
            "border-top-width" | "border-right-width" | "border-bottom-width" | "border-left-width" => {
                if let Some(w) = border_width(value, &u) {
                    s.border[side_index(name)].width = w;
                }
            }
            "border-top-style" | "border-right-style" | "border-bottom-style" | "border-left-style" => {
                s.border[side_index(name)].style = border_style(value);
            }
            "border-radius" => {
                // `a b c d / e f` — the vertical radii are ignored.
                let horizontal = value.split('/').next().unwrap_or(value);
                let v: Vec<Length> = split_spaces(horizontal).iter().filter_map(|w| len(w)).collect();
                if let Some(f) = four(&v) {
                    s.radius = f;
                }
            }
            "border-top-left-radius" => s.radius[0] = len(value).unwrap_or(Length::Pt(0.0)),
            "border-top-right-radius" => s.radius[1] = len(value).unwrap_or(Length::Pt(0.0)),
            "border-bottom-right-radius" => s.radius[2] = len(value).unwrap_or(Length::Pt(0.0)),
            "border-bottom-left-radius" => s.radius[3] = len(value).unwrap_or(Length::Pt(0.0)),

            "background" => {
                s.background_images.clear();
                s.background_color = Rgba::TRANSPARENT;
                if value == "none" || value == "transparent" {
                    return;
                }
                for layer in split_top_level(value, b',') {
                    let layer = layer.trim();
                    if let Some(img) = image(layer) {
                        s.background_images.push(img);
                        continue;
                    }
                    // `#fff url(x) no-repeat` — the colour is a word of the
                    // last layer.
                    for w in split_spaces(layer) {
                        if let Some(img) = image(w) {
                            s.background_images.push(img);
                        } else if let Some(c) = col(w) {
                            s.background_color = c.unwrap_or(s.color);
                        } else if w == "cover" {
                            s.background_size_cover = true;
                        }
                    }
                }
            }
            "background-color" => {
                if let Some(c) = col(value) {
                    s.background_color = c.unwrap_or(s.color);
                }
            }
            "background-image" => {
                s.background_images = split_top_level(value, b',').iter().filter_map(|l| image(l)).collect();
            }
            "background-size" => s.background_size_cover = value.contains("cover"),
            "box-shadow" => s.shadows = shadows(value, &u),
            "opacity" => {
                s.opacity = value
                    .strip_suffix('%')
                    .and_then(|p| p.parse::<f32>().ok().map(|p| p / 100.0))
                    .or_else(|| value.parse().ok())
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0)
            }
            // On paper nothing scrolls: `auto` and `scroll` show everything,
            // as the box grows to hold it; only `hidden` and `clip` clip.
            "overflow" | "overflow-x" | "overflow-y" => {
                s.clip = split_spaces(value).iter().all(|w| matches!(*w, "hidden" | "clip"))
            }
            "transform" => {
                s.rotate = 0.0;
                s.scale = 1.0;
                s.translate = (Length::Pt(0.0), Length::Pt(0.0));
                for part in split_spaces(value) {
                    let lower = part.to_ascii_lowercase();
                    if let Some(a) = lower.strip_prefix("rotate(").and_then(|x| x.strip_suffix(')')) {
                        s.rotate = angle(a).unwrap_or(0.0);
                    } else if let Some(a) = lower.strip_prefix("scale(").and_then(|x| x.strip_suffix(')')) {
                        s.scale = a.split(',').next().and_then(|n| n.trim().parse().ok()).unwrap_or(1.0);
                    } else if let Some(a) = lower.strip_prefix("translate(").and_then(|x| x.strip_suffix(')')) {
                        let mut it = a.split(',');
                        let x = it.next().and_then(|v| len(v.trim())).unwrap_or(Length::Pt(0.0));
                        let y = it.next().and_then(|v| len(v.trim())).unwrap_or(Length::Pt(0.0));
                        s.translate = (x, y);
                    } else if let Some(a) = lower.strip_prefix("translatey(").and_then(|x| x.strip_suffix(')')) {
                        s.translate.1 = len(a).unwrap_or(Length::Pt(0.0));
                    } else if let Some(a) = lower.strip_prefix("translatex(").and_then(|x| x.strip_suffix(')')) {
                        s.translate.0 = len(a).unwrap_or(Length::Pt(0.0));
                    }
                }
            }
            "rotate" => s.rotate = angle(value).unwrap_or(0.0),

            "flex-direction" => {
                s.flex_direction = match value {
                    "column" => FlexDirection::Column,
                    "column-reverse" => FlexDirection::ColumnReverse,
                    "row-reverse" => FlexDirection::RowReverse,
                    _ => FlexDirection::Row,
                }
            }
            "flex-wrap" => s.flex_wrap = value.starts_with("wrap"),
            "flex-flow" => {
                for w in split_spaces(value) {
                    self.apply(s, parent, if w.contains("wrap") { "flex-wrap" } else { "flex-direction" }, w);
                }
            }
            "justify-content" => s.justify_content = align(value),
            "align-items" => s.align_items = align(value),
            "align-self" => s.align_self = if value == "auto" { None } else { align(value) },
            "align-content" => s.align_content = align(value),
            "justify-items" => s.justify_items = align(value),
            "place-items" => {
                let mut it = split_spaces(value).into_iter();
                let a = it.next().and_then(align);
                s.align_items = a;
                s.justify_items = it.next().and_then(align).or(a);
            }
            "place-content" => {
                let mut it = split_spaces(value).into_iter();
                let a = it.next().and_then(align);
                s.align_content = a;
                s.justify_content = it.next().and_then(align).or(a);
            }
            "flex" => {
                let words = split_spaces(value);
                match words.as_slice() {
                    ["none"] => {
                        s.flex_grow = 0.0;
                        s.flex_shrink = 0.0;
                        s.flex_basis = Length::Auto;
                    }
                    ["auto"] => {
                        s.flex_grow = 1.0;
                        s.flex_shrink = 1.0;
                        s.flex_basis = Length::Auto;
                    }
                    [g] if g.parse::<f32>().is_ok() => {
                        s.flex_grow = g.parse().unwrap_or(0.0);
                        s.flex_shrink = 1.0;
                        s.flex_basis = Length::Pct(0.0);
                    }
                    [b] => s.flex_basis = len(b).unwrap_or(Length::Auto),
                    [g, x] => {
                        s.flex_grow = g.parse().unwrap_or(0.0);
                        if let Ok(sh) = x.parse::<f32>() {
                            s.flex_shrink = sh;
                            s.flex_basis = Length::Pct(0.0);
                        } else {
                            s.flex_basis = len(x).unwrap_or(Length::Auto);
                        }
                    }
                    [g, sh, b, ..] => {
                        s.flex_grow = g.parse().unwrap_or(0.0);
                        s.flex_shrink = sh.parse().unwrap_or(1.0);
                        s.flex_basis = len(b).unwrap_or(Length::Auto);
                    }
                    [] => {}
                }
            }
            "flex-grow" => s.flex_grow = value.parse().unwrap_or(0.0),
            "flex-shrink" => s.flex_shrink = value.parse().unwrap_or(1.0),
            "flex-basis" => s.flex_basis = len(value).unwrap_or(Length::Auto),
            "order" => s.order = value.parse().unwrap_or(0),
            "gap" | "grid-gap" => {
                let v: Vec<Length> = split_spaces(value).iter().filter_map(|w| len(w)).collect();
                match v.as_slice() {
                    [a] => {
                        s.row_gap = *a;
                        s.column_gap = *a;
                    }
                    [a, b] => {
                        s.row_gap = *a;
                        s.column_gap = *b;
                    }
                    _ => {}
                }
            }
            "row-gap" | "grid-row-gap" => s.row_gap = len(value).unwrap_or(Length::Pt(0.0)),
            "column-gap" | "grid-column-gap" => s.column_gap = len(value).unwrap_or(Length::Pt(0.0)),
            "grid-template-columns" => s.grid_columns = tracks(value, &u),
            "grid-template-rows" => s.grid_rows = tracks(value, &u),
            "grid-auto-rows" => s.grid_auto_rows = tracks(value, &u),
            "grid-column" => s.grid_column = grid_placement(value),
            "grid-row" => s.grid_row = grid_placement(value),
            "grid-column-start" => s.grid_column.0 = grid_placement(value).0,
            "grid-column-end" => s.grid_column.1 = grid_placement(value).0,
            "grid-row-start" => s.grid_row.0 = grid_placement(value).0,
            "grid-row-end" => s.grid_row.1 = grid_placement(value).0,

            "vertical-align" => {
                s.vertical_align = match value {
                    "middle" => VerticalAlign::Middle,
                    "top" | "text-top" => VerticalAlign::Top,
                    "bottom" | "text-bottom" => VerticalAlign::Bottom,
                    "super" => VerticalAlign::Super,
                    "sub" => VerticalAlign::Sub,
                    _ => VerticalAlign::Baseline,
                }
            }
            "break-before" | "page-break-before" => s.break_before = page_break(value),
            "break-after" | "page-break-after" => s.break_after = page_break(value),
            "break-inside" | "page-break-inside" => s.break_inside = page_break(value),
            "content" => {
                s.content = match value {
                    "none" | "normal" => None,
                    v => Some(content_string(v)),
                }
            }
            "object-fit" => {
                s.object_fit = match value {
                    "contain" => ObjectFit::Contain,
                    "cover" => ObjectFit::Cover,
                    "none" => ObjectFit::None,
                    "scale-down" => ObjectFit::ScaleDown,
                    _ => ObjectFit::Fill,
                }
            }
            "text-overflow" => s.text_overflow_ellipsis = value == "ellipsis",
            _ => {}
        }
    }

    fn font_shorthand(&self, s: &mut Style, parent: &Style, value: &str) {
        // `italic 700 16px/1.4 Inter, sans-serif`: everything before the size
        // is style and weight, everything after it the family.
        let words = split_spaces(value);
        let Some(size_ix) = words.iter().position(|w| {
            let size = w.split('/').next().unwrap_or(w);
            length(size, &self.units).is_some() && size.chars().next().is_some_and(|c| c.is_ascii_digit() || c == '.')
                && !matches!(*w, "100" | "200" | "300" | "400" | "500" | "600" | "700" | "800" | "900")
        }) else {
            return;
        };
        s.italic = false;
        s.font_weight = 400;
        for w in &words[..size_ix] {
            if *w == "italic" || *w == "oblique" {
                s.italic = true;
            } else if let Some(fw) = font_weight(w, parent.font_weight) {
                s.font_weight = fw;
            }
        }
        let (size, lh) = match words[size_ix].split_once('/') {
            Some((a, b)) => (a, Some(b)),
            None => (words[size_ix], None),
        };
        self.apply(s, parent, "font-size", size);
        if let Some(lh) = lh {
            self.apply(s, parent, "line-height", lh);
        }
        let family = words[size_ix + 1..].join(" ");
        if !family.is_empty() {
            s.font_family = Rc::new(parse_family_list(&family));
        }
    }
}

fn inherit_one(s: &mut Style, parent: &Style, name: &str) {
    match name {
        "color" => s.color = parent.color,
        "font-family" => s.font_family = parent.font_family.clone(),
        "font-size" => s.font_size = parent.font_size,
        "font-weight" => s.font_weight = parent.font_weight,
        "line-height" => s.line_height = parent.line_height,
        "text-align" => s.text_align = parent.text_align,
        "background" | "background-color" => s.background_color = parent.background_color,
        "border-color" => {
            for i in 0..4 {
                s.border[i].color = Some(parent.color);
            }
        }
        _ => {}
    }
}

fn side_index(name: &str) -> usize {
    if name.contains("top") {
        0
    } else if name.contains("right") {
        1
    } else if name.contains("bottom") {
        2
    } else {
        3
    }
}

fn border_width(w: &str, u: &Units) -> Option<f32> {
    match w {
        "thin" => Some(0.75),
        "medium" => Some(2.25),
        "thick" => Some(3.75),
        _ => length_pt(w, u),
    }
}

fn border_style(w: &str) -> BorderStyle {
    match w {
        "solid" => BorderStyle::Solid,
        "dashed" => BorderStyle::Dashed,
        "dotted" => BorderStyle::Dotted,
        "double" => BorderStyle::Double,
        "groove" | "ridge" | "inset" | "outset" => BorderStyle::Solid,
        _ => BorderStyle::None,
    }
}

/// `1px solid #ccc`, `0`, `none`, `2px dashed var(--x)` (already substituted).
fn border_side(value: &str, u: &Units) -> BorderSide {
    let mut side = BorderSide {
        width: 2.25,
        style: BorderStyle::None,
        color: None,
    };
    if value == "none" || value == "0" || value == "hidden" {
        side.width = 0.0;
        return side;
    }
    for w in split_spaces(value) {
        if let Some(width) = border_width(w, u) {
            side.width = width;
        } else if matches!(w, "solid" | "dashed" | "dotted" | "double" | "groove" | "ridge" | "inset" | "outset" | "none" | "hidden") {
            side.style = border_style(w);
        } else if w.eq_ignore_ascii_case("currentcolor") {
            side.color = None;
        } else if let Some(c) = color(w) {
            side.color = Some(c);
        }
    }
    side
}

fn list_style(w: &str) -> Option<ListStyle> {
    Some(match w {
        "disc" => ListStyle::Disc,
        "circle" => ListStyle::Circle,
        "square" => ListStyle::Square,
        "decimal" | "decimal-leading-zero" => ListStyle::Decimal,
        "lower-alpha" | "lower-latin" => ListStyle::LowerAlpha,
        "upper-alpha" | "upper-latin" => ListStyle::UpperAlpha,
        "lower-roman" => ListStyle::LowerRoman,
        "upper-roman" => ListStyle::UpperRoman,
        "none" => ListStyle::None,
        _ => return None,
    })
}

fn page_break(value: &str) -> Break {
    match value {
        "page" | "always" | "left" | "right" | "recto" | "verso" => Break::Page,
        "avoid" | "avoid-page" => Break::Avoid,
        _ => Break::Auto,
    }
}

fn angle(v: &str) -> Option<f32> {
    let v = v.trim();
    if let Some(d) = v.strip_suffix("deg") {
        d.trim().parse().ok()
    } else if let Some(t) = v.strip_suffix("turn") {
        t.trim().parse::<f32>().ok().map(|t| t * 360.0)
    } else if let Some(r) = v.strip_suffix("rad") {
        r.trim().parse::<f32>().ok().map(f32::to_degrees)
    } else {
        v.parse().ok()
    }
}

/// `"→"`, `'a' "b"`, `"\2014"` → the generated text.
fn content_string(v: &str) -> String {
    let mut out = String::new();
    for part in split_spaces(v) {
        let p = part.trim();
        if (p.starts_with('"') && p.ends_with('"') && p.len() >= 2) || (p.starts_with('\'') && p.ends_with('\'') && p.len() >= 2) {
            let inner = &p[1..p.len() - 1];
            // `\2014` escapes.
            let mut chars = inner.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    let mut hex = String::new();
                    while let Some(&h) = chars.peek() {
                        if h.is_ascii_hexdigit() && hex.len() < 6 {
                            hex.push(h);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if hex.is_empty() {
                        if let Some(n) = chars.next() {
                            out.push(n);
                        }
                    } else {
                        if chars.peek() == Some(&' ') {
                            chars.next();
                        }
                        if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                            out.push(ch);
                        }
                    }
                } else {
                    out.push(c);
                }
            }
        }
    }
    out
}

// ─── Matching ───────────────────────────────────────────────────────

fn matches(dom: &Dom, node: usize, sel: &Selector, pseudo: Option<&Pseudo>) -> bool {
    let parts = &sel.parts;
    let last = parts.len() - 1;
    // The pseudo-element is on the rightmost compound; a selector with one
    // matches only that generated box, and one without never does.
    let wants = parts[last].0.pseudos.iter().find(|p| matches!(p, Pseudo::Before | Pseudo::After));
    match (wants, pseudo) {
        (None, None) => {}
        (Some(a), Some(b)) if a == b => {}
        _ => return false,
    }
    match_from(dom, node, parts, last)
}

fn match_from(dom: &Dom, node: usize, parts: &[(Compound, Option<Combinator>)], ix: usize) -> bool {
    if !compound_matches(dom, node, &parts[ix].0) {
        return false;
    }
    if ix == 0 {
        return true;
    }
    match parts[ix].1.unwrap_or(Combinator::Descendant) {
        Combinator::Child => parent_element(dom, node).is_some_and(|p| match_from(dom, p, parts, ix - 1)),
        Combinator::Descendant => {
            let mut cur = parent_element(dom, node);
            while let Some(p) = cur {
                if match_from(dom, p, parts, ix - 1) {
                    return true;
                }
                cur = parent_element(dom, p);
            }
            false
        }
        Combinator::Adjacent => {
            previous_element(dom, node).is_some_and(|p| match_from(dom, p, parts, ix - 1))
        }
        Combinator::Sibling => {
            let mut cur = previous_element(dom, node);
            while let Some(p) = cur {
                if match_from(dom, p, parts, ix - 1) {
                    return true;
                }
                cur = previous_element(dom, p);
            }
            false
        }
    }
}

fn parent_element(dom: &Dom, node: usize) -> Option<usize> {
    let p = dom.nodes[node].parent?;
    dom.element(p).map(|_| p)
}

fn siblings(dom: &Dom, node: usize) -> Vec<usize> {
    match dom.nodes[node].parent {
        Some(p) => dom.element_children(p).collect(),
        None => vec![node],
    }
}

fn previous_element(dom: &Dom, node: usize) -> Option<usize> {
    let sibs = siblings(dom, node);
    let pos = sibs.iter().position(|&s| s == node)?;
    pos.checked_sub(1).map(|i| sibs[i])
}

fn compound_matches(dom: &Dom, node: usize, c: &Compound) -> bool {
    let Some(el) = dom.element(node) else {
        return false;
    };
    if let Some(tag) = &c.tag {
        if &el.tag != tag {
            return false;
        }
    }
    if let Some(id) = &c.id {
        if el.attr("id") != Some(id.as_str()) {
            return false;
        }
    }
    if !c.classes.iter().all(|cl| el.has_class(cl)) {
        return false;
    }
    if !c.attrs.iter().all(|a| attr_matches(el, a)) {
        return false;
    }
    c.pseudos.iter().all(|p| pseudo_matches(dom, node, el, p))
}

fn attr_matches(el: &Element, test: &AttrTest) -> bool {
    let Some(v) = el.attr(&test.name) else {
        return false;
    };
    match &test.op {
        None => true,
        Some((op, want)) => match op {
            AttrOp::Equals => v == want,
            AttrOp::Includes => v.split_whitespace().any(|w| w == want),
            AttrOp::DashMatch => v == want || v.starts_with(&format!("{want}-")),
            AttrOp::Prefix => v.starts_with(want.as_str()),
            AttrOp::Suffix => v.ends_with(want.as_str()),
            AttrOp::Substring => v.contains(want.as_str()),
        },
    }
}

fn nth(a: i32, b: i32, pos: i32) -> bool {
    if a == 0 {
        return pos == b;
    }
    let n = (pos - b) as f32 / a as f32;
    n >= 0.0 && n.fract() == 0.0
}

fn pseudo_matches(dom: &Dom, node: usize, el: &Element, p: &Pseudo) -> bool {
    match p {
        Pseudo::Before | Pseudo::After => true,
        Pseudo::Root => dom.nodes[node].parent.is_some_and(|p| matches!(dom.nodes[p].kind, NodeKind::Root)),
        Pseudo::Empty => dom.nodes[node].children.iter().all(|&c| match &dom.nodes[c].kind {
            NodeKind::Text(t) => t.is_empty(),
            _ => false,
        }),
        Pseudo::Not(inner) => !inner.iter().any(|c| compound_matches(dom, node, c)),
        _ => {
            let sibs = siblings(dom, node);
            let pos = sibs.iter().position(|&s| s == node).unwrap_or(0);
            let same: Vec<usize> = sibs.iter().copied().filter(|&s| dom.tag(s) == Some(el.tag.as_str())).collect();
            let type_pos = same.iter().position(|&s| s == node).unwrap_or(0);
            match p {
                Pseudo::FirstChild => pos == 0,
                Pseudo::LastChild => pos + 1 == sibs.len(),
                Pseudo::OnlyChild => sibs.len() == 1,
                Pseudo::FirstOfType => type_pos == 0,
                Pseudo::LastOfType => type_pos + 1 == same.len(),
                Pseudo::NthChild(a, b) => nth(*a, *b, pos as i32 + 1),
                Pseudo::NthOfType(a, b) => nth(*a, *b, type_pos as i32 + 1),
                _ => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn styler(css: &str) -> Styler {
        Styler::new(
            &Sheet::parse(UA_SHEET),
            &Sheet::parse(css),
            Units { em: 12.0, rem: 12.0, vw: 600.0, vh: 800.0 },
        )
    }

    /// The computed style of the first element with `tag`, styled down the
    /// tree from the root.
    fn style_of(css: &str, html: &str, tag: &str) -> Style {
        let dom = Dom::parse(html);
        let s = styler(css);
        let target = dom.find_tag(tag).unwrap();
        // The chain of ancestors, root first.
        let mut chain = vec![target];
        let mut cur = dom.nodes[target].parent;
        while let Some(p) = cur {
            if dom.element(p).is_some() {
                chain.push(p);
            }
            cur = dom.nodes[p].parent;
        }
        let mut style = Style::root(12.0, "sans-serif");
        for n in chain.into_iter().rev() {
            style = s.compute(&dom, n, &style);
        }
        style
    }

    #[test]
    fn the_cascade_orders_by_specificity_then_source() {
        let css = ".a { color: #00f } p { color: #f00 } .a.a { font-weight: 700 } .b { font-weight: 300 }";
        let s = style_of(css, r#"<p class="a b">x</p>"#, "p");
        // A class beats a tag whatever the order; two classes beat one.
        assert_eq!(s.color, Rgba::rgb(0.0, 0.0, 1.0));
        assert_eq!(s.font_weight, 700);
        // `!important` beats everything; the style attribute beats sheets.
        let s = style_of(".a { color: #0f0 !important }", r#"<p class="a" style="color: #f00">x</p>"#, "p");
        assert_eq!(s.color, Rgba::rgb(0.0, 1.0, 0.0));
        let s = style_of(".a { color: #0f0 }", r#"<p class="a" style="color: #f00">x</p>"#, "p");
        assert_eq!(s.color, Rgba::rgb(1.0, 0.0, 0.0));
    }

    #[test]
    fn inheritance_vars_and_em() {
        let css = ":root { --brand: #ff6a2b } .card { font-size: 16px; color: var(--brand) } .card p { padding: 1em }";
        let s = style_of(css, r#"<html><div class="card"><p>x</p></div></html>"#, "p");
        // Colour and size are inherited; `em` is the element's own size.
        assert_eq!(s.font_size, 12.0);
        assert_eq!(s.padding[0], Length::Pt(12.0));
        assert!((s.color.r - 1.0).abs() < 0.01 && (s.color.g - 0.416).abs() < 0.01);
        // A custom property set on an element reaches its descendants.
        let s = style_of(".x { --gap: 20px } .y { margin-top: var(--gap) }", r#"<div class="x"><span class="y">a</span></div>"#, "span");
        assert_eq!(s.margin[0], Length::Pt(15.0));
    }

    #[test]
    fn combinators_and_pseudo_classes() {
        let css = ".list > li + li { border-top: 1px solid #ccc } li:first-child { color: #f00 } li:nth-child(even) { font-weight: 700 }";
        let html = r#"<ul class="list"><li>a</li><li id="b">b</li><li id="c">c</li></ul>"#;
        let dom = Dom::parse(html);
        let s = styler(css);
        let lis: Vec<usize> = dom.descendants(0).into_iter().filter(|&n| dom.tag(n) == Some("li")).collect();
        let root = Style::root(12.0, "serif");
        let ul = s.compute(&dom, dom.find_tag("ul").unwrap(), &root);
        let st: Vec<Style> = lis.iter().map(|&l| s.compute(&dom, l, &ul)).collect();
        assert_eq!(st[0].color, Rgba::rgb(1.0, 0.0, 0.0));
        assert!(!st[0].border[0].visible());
        assert!(st[1].border[0].visible() && st[2].border[0].visible());
        assert_eq!(st[1].font_weight, 700);
        assert_eq!(st[2].font_weight, 400);
        assert_eq!(st[0].display, Display::ListItem);
    }

    #[test]
    fn shorthands_expand() {
        let css = ".b { border: 1px solid #ccc; border-bottom-width: 3px; padding: 4px 8px; margin: 0 auto; border-radius: 50%; flex: 1; font: italic 700 16px/1.5 Inter, sans-serif; background: linear-gradient(90deg, #fff, #000) }";
        let s = style_of(css, r#"<div class="b">x</div>"#, "div");
        assert_eq!(s.border[0].width, 0.75);
        assert_eq!(s.border[2].width, 2.25);
        assert_eq!(s.padding, [Length::Pt(3.0), Length::Pt(6.0), Length::Pt(3.0), Length::Pt(6.0)]);
        assert_eq!(s.margin[1], Length::Auto);
        assert_eq!(s.radius[0], Length::Pct(50.0));
        assert_eq!((s.flex_grow, s.flex_basis), (1.0, Length::Pct(0.0)));
        assert!(s.italic);
        assert_eq!(s.font_weight, 700);
        assert_eq!(s.font_size, 12.0);
        assert_eq!(s.line_height, LineHeight::Factor(1.5));
        assert_eq!(s.font_family[0], "Inter");
        assert_eq!(s.background_images.len(), 1);
    }

    #[test]
    fn generated_content() {
        let css = r#".x::before { content: "\2014 " } .y::after { content: none }"#;
        let dom = Dom::parse(r#"<p class="x y">a</p>"#);
        let s = styler(css);
        let root = Style::root(12.0, "serif");
        let p = dom.find_tag("p").unwrap();
        let ps = s.compute(&dom, p, &root);
        assert_eq!(s.compute_pseudo(&dom, p, &ps, true).and_then(|s| s.content), Some("—".into()));
        assert!(s.compute_pseudo(&dom, p, &ps, false).is_none());
    }
}
