//! A box's computed style, and the CSS values it is computed from.
//!
//! Lengths are points. `px` is CSS's (¾ pt), so a style written for the web is
//! the same size on paper; `em` is the element's font size, `rem` the root's;
//! a percentage stays one until layout knows what it is a percentage of.

use std::collections::HashMap;
use std::rc::Rc;

use super::css::{split_spaces, split_top_level};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const BLACK: Rgba = Rgba::rgb(0.0, 0.0, 0.0);
    pub const TRANSPARENT: Rgba = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const fn rgb(r: f32, g: f32, b: f32) -> Rgba {
        Rgba { r, g, b, a: 1.0 }
    }
    pub fn is_visible(&self) -> bool {
        self.a > 0.001
    }
    pub fn with_alpha(self, a: f32) -> Rgba {
        Rgba { a: self.a * a, ..self }
    }
    fn mix(self, other: Rgba, t: f32) -> Rgba {
        Rgba {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
}

/// A length as layout needs it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Length {
    #[default]
    Auto,
    Pt(f32),
    Pct(f32),
}

impl Length {
    pub fn resolve(self, of: f32) -> Option<f32> {
        match self {
            Length::Auto => None,
            Length::Pt(v) => Some(v),
            Length::Pct(p) => Some(of * p / 100.0),
        }
    }
    pub fn or_zero(self, of: f32) -> f32 {
        self.resolve(of).unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    None,
    Block,
    Inline,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    ListItem,
    Table,
    TableHeaderGroup,
    TableRowGroup,
    TableFooterGroup,
    TableRow,
    TableCell,
    TableCaption,
    Contents,
}

impl Display {
    pub fn is_inline_level(self) -> bool {
        matches!(
            self,
            Display::Inline | Display::InlineBlock | Display::InlineFlex | Display::InlineGrid
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextTransform {
    #[default]
    None,
    Upper,
    Lower,
    Capitalize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhiteSpace {
    #[default]
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineHeight {
    #[default]
    Normal,
    Factor(f32),
    Pt(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderSide {
    pub width: f32,
    pub style: BorderStyle,
    /// `None` is `currentColor`.
    pub color: Option<Rgba>,
}

impl BorderSide {
    pub fn visible(&self) -> bool {
        self.width > 0.0 && self.style != BorderStyle::None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GradientStop {
    pub color: Rgba,
    /// 0–1, or `None` to be spread evenly.
    pub at: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Image {
    Linear { angle: f32, stops: Vec<GradientStop> },
    Radial { stops: Vec<GradientStop> },
    Url(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Rgba,
    pub inset: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Track {
    Pt(f32),
    Pct(f32),
    Fr(f32),
    Auto,
    MinContent,
    MaxContent,
    MinMax(Box<Track>, Box<Track>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridLine {
    #[default]
    Auto,
    Line(i16),
    Span(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Break {
    #[default]
    Auto,
    Page,
    Avoid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListStyle {
    #[default]
    Disc,
    Circle,
    Square,
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectFit {
    #[default]
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Middle,
    Top,
    Bottom,
    Super,
    Sub,
}

/// A box's computed style.
#[derive(Debug, Clone)]
pub struct Style {
    // ─── Inherited ──────────────────────────────────────────────
    pub color: Rgba,
    pub font_family: Rc<Vec<String>>,
    pub font_size: f32,
    pub font_weight: u16,
    pub italic: bool,
    pub line_height: LineHeight,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub text_align: TextAlign,
    pub text_transform: TextTransform,
    pub white_space: WhiteSpace,
    pub direction: Direction,
    pub list_style: ListStyle,
    pub visible: bool,
    pub tab_size: f32,
    pub orphans: u32,
    pub widows: u32,
    pub border_collapse: bool,
    /// Custom properties: inherited, set by any rule.
    pub vars: Rc<HashMap<String, String>>,

    // ─── Not inherited ──────────────────────────────────────────
    pub display: Display,
    pub position: Position,
    pub inset: [Length; 4],
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    pub border_box: bool,
    pub margin: [Length; 4],
    pub padding: [Length; 4],
    pub border: [BorderSide; 4],
    /// Corners: top-left, top-right, bottom-right, bottom-left.
    pub radius: [Length; 4],
    pub background_color: Rgba,
    pub background_images: Vec<Image>,
    pub background_size_cover: bool,
    pub shadows: Vec<Shadow>,
    pub opacity: f32,
    pub clip: bool,
    pub rotate: f32,
    pub scale: f32,
    pub translate: (Length, Length),
    pub flex_direction: FlexDirection,
    pub flex_wrap: bool,
    pub justify_content: Option<Align>,
    pub align_items: Option<Align>,
    pub align_self: Option<Align>,
    pub align_content: Option<Align>,
    pub justify_items: Option<Align>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Length,
    pub order: i32,
    pub row_gap: Length,
    pub column_gap: Length,
    pub grid_columns: Vec<Track>,
    pub grid_rows: Vec<Track>,
    pub grid_auto_rows: Vec<Track>,
    pub grid_column: (GridLine, GridLine),
    pub grid_row: (GridLine, GridLine),
    pub underline: bool,
    pub line_through: bool,
    pub overline: bool,
    pub decoration_color: Option<Rgba>,
    pub vertical_align: VerticalAlign,
    pub break_before: Break,
    pub break_after: Break,
    pub break_inside: Break,
    pub content: Option<String>,
    pub object_fit: ObjectFit,
    pub aspect_ratio: Option<f32>,
    pub text_overflow_ellipsis: bool,
}

impl Style {
    /// The initial values, as a printed page starts.
    pub fn root(font_size: f32, family: &str) -> Style {
        Style {
            color: Rgba::BLACK,
            font_family: Rc::new(parse_family_list(family)),
            font_size,
            font_weight: 400,
            italic: false,
            line_height: LineHeight::Normal,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            text_align: TextAlign::Start,
            text_transform: TextTransform::None,
            white_space: WhiteSpace::Normal,
            direction: Direction::Ltr,
            list_style: ListStyle::Disc,
            visible: true,
            tab_size: 8.0,
            orphans: 2,
            widows: 2,
            border_collapse: false,
            vars: Rc::new(HashMap::new()),
            // CSS's initial value: an element nothing styles is inline.
            display: Display::Inline,
            position: Position::Static,
            inset: [Length::Auto; 4],
            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Auto,
            min_height: Length::Auto,
            max_width: Length::Auto,
            max_height: Length::Auto,
            border_box: false,
            margin: [Length::Pt(0.0); 4],
            padding: [Length::Pt(0.0); 4],
            border: [BorderSide::default(); 4],
            radius: [Length::Pt(0.0); 4],
            background_color: Rgba::TRANSPARENT,
            background_images: Vec::new(),
            background_size_cover: false,
            shadows: Vec::new(),
            opacity: 1.0,
            clip: false,
            rotate: 0.0,
            scale: 1.0,
            translate: (Length::Pt(0.0), Length::Pt(0.0)),
            flex_direction: FlexDirection::Row,
            flex_wrap: false,
            justify_content: None,
            align_items: None,
            align_self: None,
            align_content: None,
            justify_items: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Length::Auto,
            order: 0,
            row_gap: Length::Pt(0.0),
            column_gap: Length::Pt(0.0),
            grid_columns: Vec::new(),
            grid_rows: Vec::new(),
            grid_auto_rows: Vec::new(),
            grid_column: (GridLine::Auto, GridLine::Auto),
            grid_row: (GridLine::Auto, GridLine::Auto),
            underline: false,
            line_through: false,
            overline: false,
            decoration_color: None,
            vertical_align: VerticalAlign::Baseline,
            break_before: Break::Auto,
            break_after: Break::Auto,
            break_inside: Break::Auto,
            content: None,
            object_fit: ObjectFit::Fill,
            aspect_ratio: None,
            text_overflow_ellipsis: false,
        }
    }

    /// A child's starting style: the inherited properties carried, the rest
    /// at their initial values.
    pub fn inherit(&self) -> Style {
        let mut s = Style::root(self.font_size, "");
        s.color = self.color;
        s.font_family = self.font_family.clone();
        s.font_weight = self.font_weight;
        s.italic = self.italic;
        s.line_height = self.line_height;
        s.letter_spacing = self.letter_spacing;
        s.word_spacing = self.word_spacing;
        s.text_align = self.text_align;
        s.text_transform = self.text_transform;
        s.white_space = self.white_space;
        s.direction = self.direction;
        s.list_style = self.list_style;
        s.visible = self.visible;
        s.tab_size = self.tab_size;
        s.orphans = self.orphans;
        s.widows = self.widows;
        s.border_collapse = self.border_collapse;
        s.vars = self.vars.clone();
        s
    }

    /// The line height in points.
    pub fn line_height_pt(&self) -> f32 {
        match self.line_height {
            LineHeight::Normal => self.font_size * 1.2,
            LineHeight::Factor(f) => self.font_size * f,
            LineHeight::Pt(p) => p,
        }
    }

    pub fn border_color(&self, side: usize) -> Rgba {
        self.border[side].color.unwrap_or(self.color)
    }
}

// ─── Values ─────────────────────────────────────────────────────────

/// What a value is resolved against.
#[derive(Debug, Clone, Copy)]
pub struct Units {
    /// The element's font size, for `em`.
    pub em: f32,
    /// The root font size, for `rem`.
    pub rem: f32,
    /// The page's width and height, for `vw`/`vh`.
    pub vw: f32,
    pub vh: f32,
}

/// Replace every `var(--x, fallback)` with the custom property's value.
pub fn substitute_vars(value: &str, vars: &HashMap<String, String>) -> String {
    let mut out = value.to_string();
    // A value may name a property whose value names another.
    for _ in 0..8 {
        let Some(start) = out.find("var(") else {
            break;
        };
        let Some(end) = matching_paren(&out, start + 3) else {
            break;
        };
        let inner = &out[start + 4..end];
        let (name, fallback) = match split_top_level(inner, b',').as_slice() {
            [name] => (name.trim().to_string(), None),
            [name, rest @ ..] => (name.trim().to_string(), Some(rest.join(",").trim().to_string())),
            [] => break,
        };
        let replacement = vars
            .get(&name)
            .cloned()
            .or(fallback)
            .unwrap_or_default();
        out = format!("{}{}{}", &out[..start], replacement, &out[end + 1..]);
    }
    out
}

fn matching_paren(s: &str, open: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// A length: `12px`, `1.5rem`, `50%`, `auto`, `0`, `calc(100% - 24px)`,
/// `clamp(1rem, 2vw, 2rem)` (its maximum: paper is a wide screen).
pub fn length(value: &str, u: &Units) -> Option<Length> {
    let v = value.trim().to_ascii_lowercase();
    if v == "auto" || v == "none" || v == "fit-content" || v == "max-content" || v == "min-content" {
        return Some(Length::Auto);
    }
    if let Some(inner) = v.strip_prefix("calc(").and_then(|s| s.strip_suffix(')')) {
        return calc(inner, u);
    }
    for f in ["clamp(", "max(", "min("] {
        if let Some(inner) = v.strip_prefix(f).and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<Length> = split_top_level(inner, b',')
                .iter()
                .filter_map(|p| length(p, u))
                .collect();
            let pts: Vec<f32> = parts
                .iter()
                .filter_map(|l| match l {
                    Length::Pt(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if pts.is_empty() {
                return parts.first().copied();
            }
            return Some(Length::Pt(match f {
                "clamp(" => *pts.last().unwrap(),
                "max(" => pts.iter().cloned().fold(f32::MIN, f32::max),
                _ => pts.iter().cloned().fold(f32::MAX, f32::min),
            }));
        }
    }
    let num_end = v
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e'))
        .unwrap_or(v.len());
    // `1e3px` is rare; `em` must not be read as an exponent.
    let (num, unit) = if v[..num_end].ends_with('e') && v[num_end..].starts_with('m') {
        (&v[..num_end - 1], &v[num_end - 1..])
    } else {
        (&v[..num_end], &v[num_end..])
    };
    let n: f32 = num.parse().ok()?;
    Some(match unit.trim() {
        "" | "pt" => Length::Pt(n),
        "px" => Length::Pt(n * 0.75),
        "em" => Length::Pt(n * u.em),
        "rem" => Length::Pt(n * u.rem),
        "%" => Length::Pct(n),
        "mm" => Length::Pt(n * 72.0 / 25.4),
        "cm" => Length::Pt(n * 72.0 / 2.54),
        "in" => Length::Pt(n * 72.0),
        "pc" => Length::Pt(n * 12.0),
        "vw" => Length::Pt(n * u.vw / 100.0),
        "vh" => Length::Pt(n * u.vh / 100.0),
        "ch" => Length::Pt(n * u.em * 0.5),
        "ex" => Length::Pt(n * u.em * 0.5),
        _ => return None,
    })
}

/// `calc()` over points and at most one percentage: `100% - 24px`.
fn calc(inner: &str, u: &Units) -> Option<Length> {
    let mut pt = 0.0f32;
    let mut pct = 0.0f32;
    let mut sign = 1.0f32;
    let mut product: Option<f32> = None;
    for tok in split_spaces(inner) {
        match tok {
            "+" => sign = 1.0,
            "-" => sign = -1.0,
            "*" => product = Some(1.0),
            "/" => product = Some(-1.0),
            t => {
                let t = t.trim_start_matches('(').trim_end_matches(')');
                if let Some(op) = product.take() {
                    let n: f32 = t.parse().ok()?;
                    let k = if op < 0.0 { 1.0 / n } else { n };
                    pt *= k;
                    pct *= k;
                    continue;
                }
                match length(t, u)? {
                    Length::Pt(p) => pt += sign * p,
                    Length::Pct(p) => pct += sign * p,
                    Length::Auto => return None,
                }
                sign = 1.0;
            }
        }
    }
    if pct != 0.0 && pt == 0.0 {
        return Some(Length::Pct(pct));
    }
    if pct != 0.0 {
        // A mixed value: the layout cannot take both, so the percentage is
        // read against the page width it will most likely meet.
        return Some(Length::Pt(pt + pct / 100.0 * u.vw));
    }
    Some(Length::Pt(pt))
}

pub fn length_pt(value: &str, u: &Units) -> Option<f32> {
    match length(value, u)? {
        Length::Pt(p) => Some(p),
        Length::Pct(p) => Some(p / 100.0 * u.em),
        Length::Auto => None,
    }
}

/// `#rgb`, `#rrggbbaa`, `rgb()`, `rgba()`, `hsl()`, a name, `transparent`,
/// `color-mix(in srgb, a 30%, b)`. `currentColor` is `None` in an
/// `Option<Option<…>>` sense: the caller decides.
pub fn color(value: &str) -> Option<Rgba> {
    let v = value.trim().to_ascii_lowercase();
    if let Some(hex) = v.strip_prefix('#') {
        let digit = |i: usize, len: usize| -> Option<f32> {
            let s = &hex[i..i + len];
            let n = u8::from_str_radix(s, 16).ok()? as f32;
            Some(if len == 1 { n * 17.0 / 255.0 } else { n / 255.0 })
        };
        return match hex.len() {
            3 => Some(Rgba::rgb(digit(0, 1)?, digit(1, 1)?, digit(2, 1)?)),
            4 => Some(Rgba {
                r: digit(0, 1)?,
                g: digit(1, 1)?,
                b: digit(2, 1)?,
                a: digit(3, 1)?,
            }),
            6 => Some(Rgba::rgb(digit(0, 2)?, digit(2, 2)?, digit(4, 2)?)),
            8 => Some(Rgba {
                r: digit(0, 2)?,
                g: digit(2, 2)?,
                b: digit(4, 2)?,
                a: digit(6, 2)?,
            }),
            _ => None,
        };
    }
    if v == "transparent" {
        return Some(Rgba::TRANSPARENT);
    }
    let func = |name: &str| -> Option<Vec<String>> {
        let inner = v.strip_prefix(name)?.trim_start().strip_prefix('(')?.strip_suffix(')')?;
        let inner = inner.replace('/', " ").replace(',', " ");
        Some(inner.split_whitespace().map(|s| s.to_string()).collect())
    };
    let channel = |s: &str, scale: f32| -> Option<f32> {
        if let Some(p) = s.strip_suffix('%') {
            Some(p.parse::<f32>().ok()? / 100.0)
        } else {
            Some(s.parse::<f32>().ok()? / scale)
        }
    };
    if let Some(parts) = func("rgba").or_else(|| func("rgb")) {
        if parts.len() >= 3 {
            let a = parts.get(3).map(|s| channel(s, 1.0)).unwrap_or(Some(1.0))?;
            return Some(Rgba {
                r: channel(&parts[0], 255.0)?,
                g: channel(&parts[1], 255.0)?,
                b: channel(&parts[2], 255.0)?,
                a,
            });
        }
    }
    if let Some(parts) = func("hsla").or_else(|| func("hsl")) {
        if parts.len() >= 3 {
            let h = parts[0].trim_end_matches("deg").parse::<f32>().ok()? / 360.0;
            let s = channel(&parts[1], 100.0)?;
            let l = channel(&parts[2], 100.0)?;
            let a = parts.get(3).map(|s| channel(s, 1.0)).unwrap_or(Some(1.0))?;
            let (r, g, b) = hsl_to_rgb(h, s, l);
            return Some(Rgba { r, g, b, a });
        }
    }
    if let Some(inner) = v
        .strip_prefix("color-mix(")
        .and_then(|s| s.strip_suffix(')'))
    {
        // `in srgb, a 30%, b` — the space is ignored, the mix is linear.
        let parts: Vec<&str> = split_top_level(inner, b',');
        if parts.len() == 3 {
            let side = |s: &str| -> Option<(Rgba, Option<f32>)> {
                let words = split_spaces(s.trim());
                let (c, p) = match words.as_slice() {
                    [c] => (*c, None),
                    [c, p] => (*c, p.strip_suffix('%').and_then(|n| n.parse::<f32>().ok())),
                    _ => return None,
                };
                Some((color(c)?, p))
            };
            let (a, pa) = side(parts[1])?;
            let (b, pb) = side(parts[2])?;
            let wa = pa.unwrap_or_else(|| 100.0 - pb.unwrap_or(50.0)) / 100.0;
            return Some(b.mix(a, wa));
        }
    }
    named_color(&v)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hue = |mut t: f32| {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    (hue(h + 1.0 / 3.0), hue(h), hue(h - 1.0 / 3.0))
}

fn named_color(name: &str) -> Option<Rgba> {
    let hex = match name {
        "black" => "000000",
        "white" => "ffffff",
        "red" => "ff0000",
        "green" => "008000",
        "blue" => "0000ff",
        "yellow" => "ffff00",
        "orange" => "ffa500",
        "purple" => "800080",
        "pink" => "ffc0cb",
        "gray" | "grey" => "808080",
        "silver" => "c0c0c0",
        "maroon" => "800000",
        "olive" => "808000",
        "lime" => "00ff00",
        "aqua" | "cyan" => "00ffff",
        "teal" => "008080",
        "navy" => "000080",
        "fuchsia" | "magenta" => "ff00ff",
        "brown" => "a52a2a",
        "gold" => "ffd700",
        "indigo" => "4b0082",
        "violet" => "ee82ee",
        "coral" => "ff7f50",
        "salmon" => "fa8072",
        "tomato" => "ff6347",
        "crimson" => "dc143c",
        "khaki" => "f0e68c",
        "beige" => "f5f5dc",
        "ivory" => "fffff0",
        "lavender" => "e6e6fa",
        "tan" => "d2b48c",
        "chocolate" => "d2691e",
        "skyblue" => "87ceeb",
        "steelblue" => "4682b4",
        "slategray" | "slategrey" => "708090",
        "darkgray" | "darkgrey" => "a9a9a9",
        "lightgray" | "lightgrey" => "d3d3d3",
        "gainsboro" => "dcdcdc",
        "whitesmoke" => "f5f5f5",
        "dimgray" | "dimgrey" => "696969",
        "darkblue" => "00008b",
        "darkgreen" => "006400",
        "darkred" => "8b0000",
        "lightblue" => "add8e6",
        "lightgreen" => "90ee90",
        "orangered" => "ff4500",
        "seagreen" => "2e8b57",
        "royalblue" => "4169e1",
        "rebeccapurple" => "663399",
        "turquoise" => "40e0d0",
        "mintcream" => "f5fffa",
        "honeydew" => "f0fff0",
        "aliceblue" => "f0f8ff",
        "ghostwhite" => "f8f8ff",
        "snow" => "fffafa",
        "linen" => "faf0e6",
        _ => return None,
    };
    color(&format!("#{hex}"))
}

pub fn parse_family_list(value: &str) -> Vec<String> {
    split_top_level(value, b',')
        .into_iter()
        .map(|f| f.trim().trim_matches(['"', '\'']).to_string())
        .filter(|f| !f.is_empty())
        .collect()
}

pub fn font_weight(value: &str, parent: u16) -> Option<u16> {
    Some(match value.trim() {
        "normal" => 400,
        "bold" => 700,
        "bolder" => (parent + 300).min(900),
        "lighter" => parent.saturating_sub(300).max(100),
        n => n.parse::<f32>().ok()?.clamp(1.0, 1000.0) as u16,
    })
}

/// `linear-gradient(…)` / `radial-gradient(…)` / `url(…)`.
pub fn image(value: &str) -> Option<Image> {
    let v = value.trim();
    let lower = v.to_ascii_lowercase();
    if let Some(inner) = lower
        .strip_prefix("url(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let original = &v[4..4 + inner.len()];
        return Some(Image::Url(original.trim().trim_matches(['"', '\'']).to_string()));
    }
    let (radial, inner) = if let Some(i) = lower
        .strip_prefix("linear-gradient(")
        .or_else(|| lower.strip_prefix("repeating-linear-gradient("))
    {
        (false, i)
    } else if let Some(i) = lower
        .strip_prefix("radial-gradient(")
        .or_else(|| lower.strip_prefix("repeating-radial-gradient("))
    {
        (true, i)
    } else {
        return None;
    };
    let inner = inner.strip_suffix(')')?;
    let mut parts: Vec<&str> = split_top_level(inner, b',').into_iter().map(str::trim).collect();
    let mut angle = 180.0f32;
    if let Some(first) = parts.first() {
        let first = *first;
        let directional = if radial {
            first.starts_with("circle") || first.starts_with("ellipse") || first.starts_with("at ")
                || first.starts_with("closest") || first.starts_with("farthest")
        } else if let Some(deg) = first.strip_suffix("deg") {
            angle = deg.trim().parse().ok()?;
            true
        } else if let Some(turn) = first.strip_suffix("turn") {
            angle = turn.trim().parse::<f32>().ok()? * 360.0;
            true
        } else if let Some(dir) = first.strip_prefix("to ") {
            angle = match dir.trim() {
                "top" => 0.0,
                "right" => 90.0,
                "bottom" => 180.0,
                "left" => 270.0,
                "top right" | "right top" => 45.0,
                "bottom right" | "right bottom" => 135.0,
                "bottom left" | "left bottom" => 225.0,
                "top left" | "left top" => 315.0,
                _ => 180.0,
            };
            true
        } else {
            false
        };
        if directional {
            parts.remove(0);
        }
    }
    let mut stops = Vec::new();
    for p in parts {
        let words = split_spaces(p);
        let Some(c) = words.first().and_then(|w| color(w)) else {
            continue;
        };
        let at = words.get(1).and_then(|w| w.strip_suffix('%')).and_then(|n| n.parse::<f32>().ok()).map(|n| n / 100.0);
        stops.push(GradientStop { color: c, at });
        // `red 20% 40%`: a second position starts a hard stop.
        if let Some(second) = words.get(2).and_then(|w| w.strip_suffix('%')).and_then(|n| n.parse::<f32>().ok()) {
            stops.push(GradientStop { color: c, at: Some(second / 100.0) });
        }
    }
    if stops.len() < 2 {
        return None;
    }
    Some(if radial {
        Image::Radial { stops }
    } else {
        Image::Linear { angle, stops }
    })
}

/// `0 4px 6px -1px rgba(0,0,0,0.1), inset 0 1px 0 #fff`.
pub fn shadows(value: &str, u: &Units) -> Vec<Shadow> {
    if value.trim() == "none" {
        return Vec::new();
    }
    split_top_level(value, b',')
        .into_iter()
        .filter_map(|s| {
            let mut lengths = Vec::new();
            let mut col = Rgba::BLACK.with_alpha(0.25);
            let mut inset = false;
            for w in split_spaces(s.trim()) {
                if w == "inset" {
                    inset = true;
                } else if let Some(l) = length_pt(w, u) {
                    lengths.push(l);
                } else if let Some(c) = color(w) {
                    col = c;
                }
            }
            if lengths.len() < 2 {
                return None;
            }
            Some(Shadow {
                x: lengths[0],
                y: lengths[1],
                blur: lengths.get(2).copied().unwrap_or(0.0),
                spread: lengths.get(3).copied().unwrap_or(0.0),
                color: col,
                inset,
            })
        })
        .collect()
}

/// `repeat(3, 1fr)`, `200px 1fr`, `minmax(0, 1fr) auto`.
pub fn tracks(value: &str, u: &Units) -> Vec<Track> {
    let mut out = Vec::new();
    for word in split_spaces(value) {
        let lower = word.to_ascii_lowercase();
        if let Some(inner) = lower.strip_prefix("repeat(").and_then(|s| s.strip_suffix(')')) {
            if let Some((count, pattern)) = inner.split_once(',') {
                // `auto-fill`/`auto-fit` cannot be counted before layout:
                // one repetition stands for them.
                let n: usize = count.trim().parse().unwrap_or(1);
                let one = tracks(pattern.trim(), u);
                for _ in 0..n.min(64) {
                    out.extend(one.iter().cloned());
                }
            }
            continue;
        }
        if let Some(t) = track(&lower, u) {
            out.push(t);
        }
    }
    out
}

fn track(word: &str, u: &Units) -> Option<Track> {
    if let Some(fr) = word.strip_suffix("fr") {
        return fr.parse().ok().map(Track::Fr);
    }
    if let Some(inner) = word.strip_prefix("minmax(").and_then(|s| s.strip_suffix(')')) {
        let (a, b) = inner.split_once(',')?;
        return Some(Track::MinMax(
            Box::new(track(a.trim(), u)?),
            Box::new(track(b.trim(), u)?),
        ));
    }
    Some(match word {
        "auto" => Track::Auto,
        "min-content" => Track::MinContent,
        "max-content" => Track::MaxContent,
        w => match length(w, u)? {
            Length::Pt(p) => Track::Pt(p),
            Length::Pct(p) => Track::Pct(p),
            Length::Auto => Track::Auto,
        },
    })
}

/// `span 2`, `1 / 3`, `1 / -1`, `2`.
pub fn grid_placement(value: &str) -> (GridLine, GridLine) {
    let one = |s: &str| -> GridLine {
        let s = s.trim();
        if let Some(n) = s.strip_prefix("span") {
            return GridLine::Span(n.trim().parse().unwrap_or(1));
        }
        s.parse().map(GridLine::Line).unwrap_or(GridLine::Auto)
    };
    match value.split_once('/') {
        Some((a, b)) => (one(a), one(b)),
        None => (one(value), GridLine::Auto),
    }
}

pub fn align(value: &str) -> Option<Align> {
    Some(match value.trim() {
        "flex-start" | "start" | "left" | "self-start" | "normal" => Align::Start,
        "flex-end" | "end" | "right" | "self-end" => Align::End,
        "center" => Align::Center,
        "stretch" => Align::Stretch,
        "baseline" | "first baseline" | "last baseline" => Align::Baseline,
        "space-between" => Align::SpaceBetween,
        "space-around" => Align::SpaceAround,
        "space-evenly" => Align::SpaceEvenly,
        _ => return None,
    })
}

/// One to four values in CSS's top-right-bottom-left order, expanded.
pub fn four<T: Copy>(values: &[T]) -> Option<[T; 4]> {
    Some(match values {
        [a] => [*a, *a, *a, *a],
        [a, b] => [*a, *b, *a, *b],
        [a, b, c] => [*a, *b, *c, *b],
        [a, b, c, d] => [*a, *b, *c, *d],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: Units = Units {
        em: 12.0,
        rem: 12.0,
        vw: 600.0,
        vh: 800.0,
    };

    #[test]
    fn lengths_are_points() {
        assert_eq!(length("16px", &U), Some(Length::Pt(12.0)));
        assert_eq!(length("2em", &U), Some(Length::Pt(24.0)));
        assert_eq!(length("1.5rem", &U), Some(Length::Pt(18.0)));
        assert_eq!(length("50%", &U), Some(Length::Pct(50.0)));
        assert_eq!(length("10", &U), Some(Length::Pt(10.0)));
        assert_eq!(length("25.4mm", &U), Some(Length::Pt(72.0)));
        assert_eq!(length("auto", &U), Some(Length::Auto));
        // A clamp is its maximum: paper is a wide screen.
        assert_eq!(length("clamp(1rem, 1rem + 1vw, 2rem)", &U), Some(Length::Pt(24.0)));
        assert_eq!(length("calc(8px * 2)", &U), Some(Length::Pt(12.0)));
        assert_eq!(length("calc(100%)", &U), Some(Length::Pct(100.0)));
        assert_eq!(length("bogus", &U), None);
    }

    #[test]
    fn colours_in_every_spelling() {
        let red = Rgba::rgb(1.0, 0.0, 0.0);
        assert_eq!(color("#f00"), Some(red));
        assert_eq!(color("#FF0000"), Some(red));
        assert_eq!(color("rgb(255, 0, 0)"), Some(red));
        assert_eq!(color("rgb(255 0 0 / 50%)"), Some(red.with_alpha(0.5)));
        assert_eq!(color("rgba(255,0,0,0.5)"), Some(red.with_alpha(0.5)));
        assert_eq!(color("red"), Some(red));
        let hsl = color("hsl(0, 100%, 50%)").unwrap();
        assert!((hsl.r - 1.0).abs() < 0.01 && hsl.g.abs() < 0.01);
        let mix = color("color-mix(in srgb, #ffffff 50%, #000000)").unwrap();
        assert!((mix.r - 0.5).abs() < 0.01);
        assert_eq!(color("transparent").map(|c| c.a), Some(0.0));
    }

    #[test]
    fn vars_resolve_through_each_other() {
        let mut vars = HashMap::new();
        vars.insert("--brand".to_string(), "#ff6a2b".to_string());
        vars.insert("--edge".to_string(), "var(--brand)".to_string());
        assert_eq!(substitute_vars("1px solid var(--edge)", &vars), "1px solid #ff6a2b");
        assert_eq!(substitute_vars("var(--nope, 4px)", &vars), "4px");
    }

    #[test]
    fn gradients_shadows_and_tracks() {
        let Some(Image::Linear { angle, stops }) =
            image("linear-gradient(90deg, #fff 0%, #000 100%)")
        else {
            panic!()
        };
        assert_eq!(angle, 90.0);
        assert_eq!(stops.len(), 2);
        assert!(matches!(image("radial-gradient(circle at top, red, blue)"), Some(Image::Radial { .. })));
        assert_eq!(image("url(\"/a.png\")"), Some(Image::Url("/a.png".into())));
        let s = shadows("0 4px 6px -1px rgba(0,0,0,0.1), inset 0 1px 0 #fff", &U);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].y, 3.0);
        assert!(s[1].inset);
        assert_eq!(tracks("repeat(3, 1fr)", &U), vec![Track::Fr(1.0); 3]);
        assert_eq!(tracks("200px auto", &U), vec![Track::Pt(150.0), Track::Auto]);
        assert_eq!(grid_placement("span 2"), (GridLine::Span(2), GridLine::Auto));
        assert_eq!(grid_placement("1 / -1"), (GridLine::Line(1), GridLine::Line(-1)));
    }
}
