//! Style values as data: what kind of value a property takes, the value
//! read into its parts, and the narrowest declaration written back.
//!
//! A style value is raw CSS to the end of its line, so a property is not
//! one field. Each property has a **shape** — a box of one to four sides,
//! corners, a length, a unitless number, a colour, a paint (a colour or a
//! gradient), a composite of fixed parts (`border`), a list of composites
//! (`box-shadow`), a set of words, a track list, a time, an easing, or free
//! CSS — and `parse` reads a written value into that shape's parts,
//! `print` writes the parts back in the narrowest form that says them:
//! `padding: 12px` when the four sides agree, `12px 16px` when the pairs
//! do, `repeat(3, 1fr)` for three equal tracks. `box_declarations` says
//! when one side alone is `padding-left` rather than a shorthand.
//!
//! What cannot be read into its shape is `Value::Free`, kept as it was
//! written: a `{splice}`, a `calc()`, `var()`, anything the property's
//! shape does not cover. Nothing is guessed. And `print(parse(x))` reads
//! back as the same parts — `tests/style_values.rs` holds every style value
//! of every project in the repository to it.
//!
//! `property` is the catalogue an inspector is built from: each curated
//! property's group, shape, token group and suffixes, the words an enum
//! takes, the range a number has, and whether a PDF or a deck draws it.

/// What kind of value a property takes: the control an inspector draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shape {
    /// One to four sides: `padding`, `margin`, `inset`.
    Box,
    /// One to four corners: `border-radius`.
    Corners,
    /// A number with a unit, or a keyword: `width`, `gap`, `font-size`.
    Length,
    /// A number with no unit: `opacity`, `z-index`, `line-height`.
    Number,
    Color,
    /// A colour or a gradient: `background`.
    Paint,
    /// Fixed parts, one shorthand: `border: 1px solid $border`.
    Composite,
    /// Composites that stack: `box-shadow`, `transition`.
    List,
    /// One of a fixed set of words: `display`, `text-align`.
    Enum,
    /// Grid tracks: `grid-template-columns`.
    Tracks,
    /// A length of time: `transition-duration`.
    Time,
    /// How a motion is paced: `transition-timing-function`.
    Easing,
    /// Anything else, as written.
    Free,
}

/// One property, as an inspector sees it.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Property {
    /// As the CSS has it: `radius` is `border-radius`.
    pub name: String,
    /// The panel's group: `Layout`, `Spacing`, `Size`, `Typography`,
    /// `Fill and border`, `Effects`, `Motion`, or `Other`.
    pub group: &'static str,
    pub shape: Shape,
    /// The token group a bare `$name` on it resolves through, and that
    /// group's names: `spacing` and `xs` … `3xl`.
    pub tokens: Option<&'static str>,
    pub suffixes: &'static [&'static str],
    /// For an enum, the words it takes.
    pub cases: &'static [&'static str],
    /// For a number, the range a slider covers, where it is known.
    pub range: Option<(f64, f64)>,
    /// Whether a PDF or a deck draws it.
    pub on_paper: bool,
}

/// What an inspector says of a property a PDF or a deck leaves out.
pub const NOT_ON_PAPER: &str =
    "A PDF and a deck do not draw this property; it is the page's alone.";

const LAYOUT: &str = "Layout";
const SPACING: &str = "Spacing";
const SIZE: &str = "Size";
const TYPOGRAPHY: &str = "Typography";
const FILL: &str = "Fill and border";
const EFFECTS: &str = "Effects";
const MOTION: &str = "Motion";
const OTHER: &str = "Other";

const BORDER_STYLES: &[&str] = &[
    "none", "solid", "dashed", "dotted", "double", "groove", "ridge", "inset", "outset", "hidden",
];

/// The curated properties, by group, in the panel's order.
pub const CURATED: &[(&str, &[&str])] = &[
    (
        LAYOUT,
        &[
            "display",
            "position",
            "flex-direction",
            "flex-wrap",
            "align-items",
            "align-self",
            "justify-content",
            "justify-items",
            "place-items",
            "flex",
            "flex-grow",
            "flex-shrink",
            "flex-basis",
            "order",
            "grid-template-columns",
            "grid-template-rows",
            "gap",
            "row-gap",
            "column-gap",
            "inset",
            "top",
            "right",
            "bottom",
            "left",
            "z-index",
            "overflow",
            "overflow-x",
            "overflow-y",
            "box-sizing",
            "visibility",
        ],
    ),
    (
        SPACING,
        &["padding", "margin", "scroll-padding", "scroll-margin"],
    ),
    (
        SIZE,
        &[
            "width",
            "height",
            "min-width",
            "max-width",
            "min-height",
            "max-height",
            "aspect-ratio",
            "object-fit",
        ],
    ),
    (
        TYPOGRAPHY,
        &[
            "color",
            "font-size",
            "font-weight",
            "font-style",
            "line-height",
            "letter-spacing",
            "text-align",
            "text-transform",
            "text-decoration",
            "text-indent",
            "white-space",
            "vertical-align",
            "list-style-type",
        ],
    ),
    (
        FILL,
        &[
            "background",
            "background-color",
            "background-size",
            "background-repeat",
            "background-position",
            "border",
            "border-color",
            "border-style",
            "border-radius",
            "outline",
            "outline-color",
            "outline-offset",
            "fill",
            "stroke",
            "caret-color",
            "accent-color",
        ],
    ),
    (
        EFFECTS,
        &[
            "opacity",
            "box-shadow",
            "filter",
            "transform",
            "cursor",
            "pointer-events",
        ],
    ),
    (
        MOTION,
        &[
            "transition",
            "transition-duration",
            "transition-timing-function",
            "animation",
            "animation-duration",
            "animation-delay",
            "animation-timing-function",
        ],
    ),
];

fn cases_of(name: &str) -> &'static [&'static str] {
    match name {
        "display" => &[
            "block",
            "inline",
            "inline-block",
            "flex",
            "inline-flex",
            "grid",
            "inline-grid",
            "contents",
            "none",
        ],
        "position" => &["static", "relative", "absolute", "fixed", "sticky"],
        "flex-direction" => &["row", "row-reverse", "column", "column-reverse"],
        "flex-wrap" => &["nowrap", "wrap", "wrap-reverse"],
        "align-items" | "align-self" | "justify-items" => &[
            "stretch",
            "start",
            "center",
            "end",
            "baseline",
            "flex-start",
            "flex-end",
            "auto",
        ],
        "justify-content" => &[
            "start",
            "center",
            "end",
            "space-between",
            "space-around",
            "space-evenly",
            "flex-start",
            "flex-end",
            "stretch",
        ],
        "text-align" => &["left", "center", "right", "justify", "start", "end"],
        "text-transform" => &["none", "uppercase", "lowercase", "capitalize"],
        "font-style" => &["normal", "italic", "oblique"],
        "white-space" => &[
            "normal",
            "nowrap",
            "pre",
            "pre-wrap",
            "pre-line",
            "break-spaces",
        ],
        "overflow" | "overflow-x" | "overflow-y" => {
            &["visible", "hidden", "clip", "scroll", "auto"]
        }
        "object-fit" => &["fill", "contain", "cover", "none", "scale-down"],
        "cursor" => &[
            "auto",
            "default",
            "pointer",
            "text",
            "move",
            "grab",
            "grabbing",
            "not-allowed",
            "wait",
            "help",
            "crosshair",
            "col-resize",
            "row-resize",
        ],
        "pointer-events" => &["auto", "none"],
        "visibility" => &["visible", "hidden", "collapse"],
        "box-sizing" => &["content-box", "border-box"],
        "border-style" => BORDER_STYLES,
        "background-size" => &["auto", "cover", "contain"],
        "background-repeat" => &[
            "repeat",
            "no-repeat",
            "repeat-x",
            "repeat-y",
            "space",
            "round",
        ],
        "background-position" => &["center", "top", "right", "bottom", "left"],
        "list-style-type" => &["disc", "circle", "square", "decimal", "none"],
        "vertical-align" => &[
            "baseline",
            "top",
            "middle",
            "bottom",
            "text-top",
            "text-bottom",
            "sub",
            "super",
        ],
        _ => &[],
    }
}

fn shape_of(name: &str) -> Shape {
    match name {
        "padding" | "margin" | "inset" | "scroll-padding" | "scroll-margin" => Shape::Box,
        "border-radius" => Shape::Corners,
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height" | "top"
        | "right" | "bottom" | "left" | "gap" | "row-gap" | "column-gap" | "flex-basis"
        | "font-size" | "letter-spacing" | "text-indent" | "outline-offset" => Shape::Length,
        n if n.starts_with("padding-") || n.starts_with("margin-") || n.starts_with("inset-") => {
            Shape::Length
        }
        n if n.starts_with("border-") && n.ends_with("-radius") => Shape::Length,
        "opacity" | "z-index" | "flex-grow" | "flex-shrink" | "order" | "line-height"
        | "font-weight" | "aspect-ratio" => Shape::Number,
        "color"
        | "background-color"
        | "border-color"
        | "outline-color"
        | "fill"
        | "stroke"
        | "caret-color"
        | "accent-color"
        | "text-decoration-color" => Shape::Color,
        "background" => Shape::Paint,
        "border" | "outline" | "text-decoration" | "flex" | "place-items" | "border-top"
        | "border-right" | "border-bottom" | "border-left" => Shape::Composite,
        "box-shadow" | "transition" | "transform" | "filter" | "animation" => Shape::List,
        "grid-template-columns" | "grid-template-rows" => Shape::Tracks,
        "transition-duration" | "animation-duration" | "animation-delay" | "transition-delay" => {
            Shape::Time
        }
        "transition-timing-function" | "animation-timing-function" => Shape::Easing,
        n if !cases_of(n).is_empty() => Shape::Enum,
        _ => Shape::Free,
    }
}

/// The property called `name` — `radius` is `border-radius` — as an
/// inspector draws it. A property the catalogue does not curate is `Other`
/// and `Free`, and still written as typed.
pub fn property(name: &str) -> Property {
    let name = crate::codegen::style_tokens::canonical_style_prop(name);
    let group = CURATED
        .iter()
        .find(|(_, props)| props.contains(&name.as_str()))
        .map(|(g, _)| *g)
        .unwrap_or(OTHER);
    let tokens = crate::codegen::style_tokens::token_group(&name);
    Property {
        group,
        shape: shape_of(&name),
        suffixes: tokens
            .map(crate::codegen::style_tokens::group_suffixes)
            .unwrap_or(&[]),
        tokens,
        cases: cases_of(&name),
        range: match name.as_str() {
            "opacity" | "flex-grow" => Some((0.0, if name == "opacity" { 1.0 } else { 10.0 })),
            _ => None,
        },
        on_paper: crate::paged::cascade::drawn_on_paper(&name),
        name,
    }
}

// ─── Parts ───────────────────────────────────────────────

/// One part of a value, as it is written.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Part {
    /// `12px`, `1.5rem`, `0` — a number with its unit (`""` for none).
    Number { value: f64, unit: String },
    /// `$brand`, `$space-md`.
    Token { name: String },
    /// A word: `auto`, `solid`, `center`, a named colour.
    Word { text: String },
    /// `#3d6fa3`.
    Hex { text: String },
    /// A function call kept whole: `rgba(0,0,0,.4)`, `minmax(0, 1fr)`.
    Call { text: String },
    /// `{expr}`: a value read from the program.
    Splice { text: String },
}

impl Part {
    /// As written back.
    pub fn text(&self) -> String {
        match self {
            Part::Number { value, unit } => {
                if *value == 0.0 && !unit.is_empty() && unit != "%" && unit != "fr" {
                    // A zero needs no unit, but keeps the one a time or a
                    // ratio would lose its meaning without.
                    if matches!(unit.as_str(), "s" | "ms" | "deg" | "turn" | "rad") {
                        format!("0{unit}")
                    } else {
                        "0".to_string()
                    }
                } else {
                    format!("{}{unit}", number(*value))
                }
            }
            Part::Token { name } => format!("${name}"),
            Part::Word { text } | Part::Hex { text } | Part::Call { text } => text.clone(),
            Part::Splice { text } => format!("{{{text}}}"),
        }
    }

    fn is_length(&self) -> bool {
        matches!(self, Part::Number { .. } | Part::Token { .. })
            || matches!(self, Part::Call { text } if text.starts_with("calc(") || text.starts_with("var(") || text.starts_with("clamp(") || text.starts_with("min(") || text.starts_with("max("))
            || matches!(self, Part::Word { text } if text == "auto")
    }

    fn is_color(&self) -> bool {
        matches!(self, Part::Hex { .. } | Part::Token { .. })
            || matches!(self, Part::Call { text } if ["rgb(", "rgba(", "hsl(", "hsla(", "var(", "color-mix(", "oklch(", "oklab("].iter().any(|f| text.starts_with(f)))
            || matches!(self, Part::Word { text } if is_named_color(text))
    }
}

fn is_named_color(word: &str) -> bool {
    crate::paged::style::color(word).is_some() && !word.starts_with('#')
        || matches!(
            word,
            "transparent" | "currentColor" | "currentcolor" | "inherit"
        )
}

/// A number as the shortest text that reads back as itself.
fn number(n: f64) -> String {
    crate::print::number(n)
}

/// The words of `text` at its top level: split at spaces (and, with
/// `commas`, at commas into groups), with a call's parentheses, a string's
/// quotes and a `{splice}` kept whole.
fn split(text: &str, commas: bool) -> Option<Vec<Vec<String>>> {
    let mut groups: Vec<Vec<String>> = vec![Vec::new()];
    let mut word = String::new();
    let mut depth = 0i32;
    let mut braces = 0i32;
    let mut quote: Option<char> = None;
    for c in text.chars() {
        if let Some(q) = quote {
            word.push(c);
            if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                quote = Some(c);
                word.push(c);
            }
            '(' => {
                depth += 1;
                word.push(c);
            }
            ')' => {
                depth -= 1;
                word.push(c);
            }
            '{' => {
                braces += 1;
                word.push(c);
            }
            '}' => {
                braces -= 1;
                word.push(c);
            }
            ',' if commas && depth == 0 && braces == 0 => {
                if !word.is_empty() {
                    groups.last_mut()?.push(std::mem::take(&mut word));
                }
                groups.push(Vec::new());
            }
            c if c.is_whitespace() && depth == 0 && braces == 0 => {
                if !word.is_empty() {
                    groups.last_mut()?.push(std::mem::take(&mut word));
                }
            }
            c => word.push(c),
        }
    }
    if depth != 0 || braces != 0 || quote.is_some() {
        return None;
    }
    if !word.is_empty() {
        groups.last_mut()?.push(word);
    }
    Some(groups)
}

/// One word as a part.
pub fn part(word: &str) -> Option<Part> {
    let w = word.trim();
    if w.is_empty() {
        return None;
    }
    if let Some(name) = w.strip_prefix('$') {
        return name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            .then(|| Part::Token {
                name: name.to_string(),
            });
    }
    if w.starts_with('{') && w.ends_with('}') {
        return Some(Part::Splice {
            text: w[1..w.len() - 1].to_string(),
        });
    }
    if let Some(digits) = w.strip_prefix('#') {
        return digits
            .chars()
            .all(|c| c.is_ascii_hexdigit())
            .then(|| Part::Hex {
                text: w.to_string(),
            });
    }
    if w.ends_with(')') && w.contains('(') {
        return Some(Part::Call {
            text: w.to_string(),
        });
    }
    // A number and its unit.
    let digits = w
        .char_indices()
        .take_while(|(i, c)| {
            c.is_ascii_digit() || *c == '.' || (*i == 0 && (*c == '-' || *c == '+'))
        })
        .count();
    if digits > 0 && w[..digits].chars().any(|c| c.is_ascii_digit()) {
        let (n, unit) = w.split_at(digits);
        let value: f64 = n.parse().ok()?;
        if unit.chars().all(|c| c.is_ascii_alphabetic() || c == '%') {
            // `.4` and `0.4` read alike, so neither is kept as written.
            return Some(Part::Number {
                value,
                unit: unit.to_string(),
            });
        }
        return None;
    }
    w.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        .then(|| Part::Word {
            text: w.to_string(),
        })
}

// ─── Values ──────────────────────────────────────────────

/// A value read into its shape's parts.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "shape", rename_all = "kebab-case")]
pub enum Value {
    /// Top, right, bottom, left.
    Box { sides: [Part; 4] },
    /// Top-left, top-right, bottom-right, bottom-left.
    Corners { corners: [Part; 4] },
    /// A length, a number, a colour, a word, a time, an easing.
    One { part: Part },
    /// `linear-gradient(to bottom, $a, $b 40%)`.
    Gradient {
        /// `linear` or `radial`, and `repeating-` before it when it is one.
        kind: String,
        /// `to bottom`, `45deg`, `circle at center` — as written.
        direction: Option<String>,
        stops: Vec<Stop>,
    },
    /// A shorthand's parts in the order it writes them: a border's width,
    /// style and colour.
    Composite { parts: Vec<Part> },
    /// Layers, each its parts: a shadow's `x y blur spread colour`, a
    /// transition's `property duration easing delay`.
    Layers { layers: Vec<Vec<Part>> },
    /// One per track; equal tracks were `repeat()`.
    Tracks { tracks: Vec<Part> },
    /// As written: a splice, `calc()`, `var()`, what no shape covers.
    Free { text: String },
}

/// One stop of a gradient: a colour, and where it sits when it says.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Stop {
    pub color: Part,
    pub at: Option<Part>,
}

/// The value written for `property`, read into its shape. What the shape
/// cannot hold is [`Value::Free`].
pub fn parse(property: &str, text: &str) -> Value {
    let free = || Value::Free {
        text: text.trim().to_string(),
    };
    let p = self::property(property);
    let text = text.trim();
    if text.is_empty() || text.contains("!important") {
        return free();
    }
    let Some(groups) = split(text, true) else {
        return free();
    };
    let single = groups.len() == 1;
    let words: Vec<&str> = groups.iter().flatten().map(String::as_str).collect();
    let parts: Option<Vec<Part>> = words.iter().map(|w| part(w)).collect();
    // A splice alone is the program's value, whatever the property.
    if words.len() == 1 && words[0].starts_with('{') {
        return free();
    }
    match p.shape {
        Shape::Box | Shape::Corners if single => {
            let Some(parts) =
                parts.filter(|v| (1..=4).contains(&v.len()) && v.iter().all(Part::is_length))
            else {
                return free();
            };
            let four = expand(&parts);
            if p.shape == Shape::Box {
                Value::Box { sides: four }
            } else {
                Value::Corners { corners: four }
            }
        }
        Shape::Length if single && words.len() == 1 => match parts {
            Some(mut v) if v[0].is_length() || matches!(v[0], Part::Word { .. }) => {
                Value::One { part: v.remove(0) }
            }
            _ => free(),
        },
        Shape::Number if single && words.len() == 1 => match parts {
            // `line-height` takes a length as well as a ratio.
            Some(mut v)
                if matches!(v[0], Part::Number { ref unit, .. } if unit.is_empty() || p.name == "line-height")
                    || matches!(v[0], Part::Token { .. } | Part::Word { .. }) =>
            {
                Value::One { part: v.remove(0) }
            }
            _ => free(),
        },
        Shape::Color if single && words.len() == 1 => match parts {
            Some(mut v) if v[0].is_color() => Value::One { part: v.remove(0) },
            _ => free(),
        },
        Shape::Enum if single && words.len() == 1 => match parts {
            Some(mut v) if matches!(&v[0], Part::Word { text } if p.cases.contains(&text.as_str())) => {
                Value::One { part: v.remove(0) }
            }
            _ => free(),
        },
        Shape::Time if single && words.len() == 1 => match parts {
            Some(mut v)
                if matches!(&v[0], Part::Number { unit, .. } if unit == "s" || unit == "ms")
                    || matches!(v[0], Part::Token { .. }) =>
            {
                Value::One { part: v.remove(0) }
            }
            _ => free(),
        },
        Shape::Easing if single && words.len() == 1 => match parts {
            Some(mut v)
                if matches!(v[0], Part::Word { .. } | Part::Token { .. })
                    || matches!(&v[0], Part::Call { text } if text.starts_with("cubic-bezier(") || text.starts_with("steps(") || text.starts_with("linear(")) =>
            {
                Value::One { part: v.remove(0) }
            }
            _ => free(),
        },
        Shape::Paint if single && words.len() == 1 => {
            let word = words[0];
            if let Some(g) = gradient(word) {
                return g;
            }
            match parts {
                Some(mut v) if v[0].is_color() => Value::One { part: v.remove(0) },
                _ => free(),
            }
        }
        Shape::Composite if single => match parts {
            Some(v) if !v.is_empty() && composite_reads(&p.name, &v) => {
                Value::Composite { parts: v }
            }
            _ => free(),
        },
        Shape::List => {
            let mut layers = Vec::new();
            for g in &groups {
                if g.is_empty() {
                    return free();
                }
                let Some(ps) = g.iter().map(|w| part(w)).collect::<Option<Vec<Part>>>() else {
                    return free();
                };
                if ps.iter().any(|p| matches!(p, Part::Splice { .. })) {
                    return free();
                }
                layers.push(ps);
            }
            // `transform` and `filter` are one space-separated list, not
            // comma-separated layers.
            if matches!(p.name.as_str(), "transform" | "filter") && layers.len() != 1 {
                return free();
            }
            Value::Layers { layers }
        }
        Shape::Tracks if single => {
            let mut tracks = Vec::new();
            for w in &words {
                match repeat(w) {
                    Some(r) => tracks.extend(r),
                    None if w.starts_with("repeat(") => return free(),
                    None => match part(w) {
                        Some(t)
                            if t.is_length()
                                || matches!(t, Part::Call { .. } | Part::Word { .. }) =>
                        {
                            tracks.push(t)
                        }
                        _ => return free(),
                    },
                }
            }
            if tracks.is_empty() {
                return free();
            }
            Value::Tracks { tracks }
        }
        _ => free(),
    }
}

/// One to four values as the four a box has: `a` is all four, `a b` the
/// pairs, `a b c` the middle pair shared.
fn expand(parts: &[Part]) -> [Part; 4] {
    let p = |i: usize| parts[i].clone();
    match parts.len() {
        1 => [p(0), p(0), p(0), p(0)],
        2 => [p(0), p(1), p(0), p(1)],
        3 => [p(0), p(1), p(2), p(1)],
        _ => [p(0), p(1), p(2), p(3)],
    }
}

/// The narrowest of the four: one when they agree, two when the pairs do,
/// three when the sides do.
pub fn narrowest(four: &[Part; 4]) -> Vec<Part> {
    let [t, r, b, l] = four;
    if t == r && r == b && b == l {
        vec![t.clone()]
    } else if t == b && r == l {
        vec![t.clone(), r.clone()]
    } else if r == l {
        vec![t.clone(), r.clone(), b.clone()]
    } else {
        four.to_vec()
    }
}

/// Whether the parts are ones the composite takes, each at most once: a
/// border's width, style and colour.
fn composite_reads(name: &str, parts: &[Part]) -> bool {
    match name {
        "border" | "outline" | "border-top" | "border-right" | "border-bottom" | "border-left" => {
            let widths = parts.iter().filter(|p| matches!(p, Part::Number { .. }) || matches!(p, Part::Word { text } if matches!(text.as_str(), "thin" | "medium" | "thick"))).count();
            let styles = parts
                .iter()
                .filter(
                    |p| matches!(p, Part::Word { text } if BORDER_STYLES.contains(&text.as_str())),
                )
                .count();
            let colors = parts
                .iter()
                .filter(|p| p.is_color() && !matches!(p, Part::Number { .. }))
                .count();
            parts.len() <= 3
                && widths <= 1
                && styles <= 1
                && colors <= 1
                && widths + styles + colors == parts.len()
        }
        _ => parts.len() <= 3,
    }
}

/// `repeat(3, 1fr)` as its tracks.
fn repeat(word: &str) -> Option<Vec<Part>> {
    let inner = word.strip_prefix("repeat(")?.strip_suffix(')')?;
    let (n, track) = inner.split_once(',')?;
    let n: usize = n.trim().parse().ok()?;
    let groups = split(track.trim(), false)?;
    let words = groups.into_iter().next()?;
    if words.len() != 1 || n == 0 || n > 64 {
        return None;
    }
    let t = part(&words[0])?;
    Some(vec![t; n])
}

/// `linear-gradient(…)` read into its direction and stops.
fn gradient(word: &str) -> Option<Value> {
    let (kind, inner) = ["repeating-linear", "repeating-radial", "linear", "radial"]
        .iter()
        .find_map(|k| {
            word.strip_prefix(&format!("{k}-gradient("))
                .and_then(|r| r.strip_suffix(')'))
                .map(|r| (k.to_string(), r))
        })?;
    let groups = split(inner, true)?;
    let mut direction = None;
    let mut stops = Vec::new();
    for (i, g) in groups.iter().enumerate() {
        let parts: Vec<Part> = g.iter().map(|w| part(w)).collect::<Option<_>>()?;
        match parts.as_slice() {
            [c] if c.is_color() => stops.push(Stop {
                color: c.clone(),
                at: None,
            }),
            [c, at] if c.is_color() && matches!(at, Part::Number { .. }) => stops.push(Stop {
                color: c.clone(),
                at: Some(at.clone()),
            }),
            _ if i == 0 => direction = Some(g.join(" ")),
            _ => return None,
        }
    }
    // A stop where it would be anyway — the first at the start, the last
    // at the end — says nothing, and is read as saying nothing.
    let at = |s: &Stop, v: f64| matches!(&s.at, Some(Part::Number { value, unit }) if unit == "%" && *value == v);
    if stops.first().is_some_and(|s| at(s, 0.0)) {
        stops[0].at = None;
    }
    if let Some(last) = stops.last_mut()
        && at(last, 100.0)
    {
        last.at = None;
    }
    (stops.len() >= 2).then_some(Value::Gradient {
        kind,
        direction,
        stops,
    })
}

/// The value written back for `property`, in the narrowest form that says
/// its parts.
pub fn print(property: &str, value: &Value) -> String {
    let join = |ps: &[Part]| ps.iter().map(Part::text).collect::<Vec<_>>().join(" ");
    let _ = property;
    match value {
        Value::Box { sides } => join(&narrowest(sides)),
        Value::Corners { corners } => join(&narrowest(corners)),
        Value::One { part } => part.text(),
        Value::Composite { parts } => join(parts),
        Value::Layers { layers } => layers
            .iter()
            .map(|l| join(l))
            .collect::<Vec<_>>()
            .join(", "),
        Value::Tracks { tracks } => {
            let all_same = tracks.windows(2).all(|w| w[0] == w[1]);
            if tracks.len() > 1 && all_same {
                format!("repeat({}, {})", tracks.len(), tracks[0].text())
            } else {
                join(tracks)
            }
        }
        Value::Gradient {
            kind,
            direction,
            stops,
        } => {
            let last = stops.len().saturating_sub(1);
            let mut items: Vec<String> = direction.iter().cloned().collect();
            for (i, s) in stops.iter().enumerate() {
                // A stop where it would be anyway — the first at the start,
                // the last at the end — says nothing.
                let at = match &s.at {
                    Some(Part::Number { value, unit })
                        if unit == "%"
                            && ((i == 0 && *value == 0.0) || (i == last && *value == 100.0)) =>
                    {
                        None
                    }
                    other => other.clone(),
                };
                items.push(match at {
                    Some(at) => format!("{} {}", s.color.text(), at.text()),
                    None => s.color.text(),
                });
            }
            format!("{kind}-gradient({})", items.join(", "))
        }
        Value::Free { text } => text.clone(),
    }
}

/// The side longhands of a box property, top, right, bottom, left.
pub fn sides_of(property: &str) -> Option<[String; 4]> {
    let p = crate::codegen::style_tokens::canonical_style_prop(property);
    Some(match p.as_str() {
        "inset" => ["top", "right", "bottom", "left"].map(String::from),
        "border-radius" => [
            "border-top-left-radius",
            "border-top-right-radius",
            "border-bottom-right-radius",
            "border-bottom-left-radius",
        ]
        .map(String::from),
        "padding" | "margin" | "scroll-padding" | "scroll-margin" => {
            ["top", "right", "bottom", "left"].map(|s| format!("{p}-{s}"))
        }
        _ => return None,
    })
}

/// What a box property's sides — each set, or not — are written as: the
/// shorthand, narrowest, when all four are set; each set side's longhand
/// alone otherwise (`padding-left: 16px` and nothing else).
pub fn box_declarations(property: &str, sides: &[Option<Part>; 4]) -> Vec<(String, String)> {
    let name = crate::codegen::style_tokens::canonical_style_prop(property);
    if let [Some(t), Some(r), Some(b), Some(l)] = sides {
        let four = [t.clone(), r.clone(), b.clone(), l.clone()];
        let value = if name == "border-radius" {
            Value::Corners { corners: four }
        } else {
            Value::Box { sides: four }
        };
        return vec![(name.clone(), print(&name, &value))];
    }
    let Some(longhands) = sides_of(&name) else {
        return Vec::new();
    };
    sides
        .iter()
        .zip(longhands)
        .filter_map(|(s, l)| s.as_ref().map(|p| (l, p.text())))
        .collect()
}

/// An element's sides for a box property, from the declarations it has
/// in order — the shorthand, then any longhand after it: what each side is
/// set to, and `None` where nothing sets it.
pub fn read_box(declarations: &[(&str, &str)], property: &str) -> [Option<Part>; 4] {
    let name = crate::codegen::style_tokens::canonical_style_prop(property);
    let mut sides: [Option<Part>; 4] = [None, None, None, None];
    let Some(longhands) = sides_of(&name) else {
        return sides;
    };
    for (n, v) in declarations {
        let n = crate::codegen::style_tokens::canonical_style_prop(n);
        if n == name {
            match parse(&name, v) {
                Value::Box { sides: four } | Value::Corners { corners: four } => {
                    sides = four.map(Some);
                }
                _ => sides = [None, None, None, None],
            }
        } else if let Some(i) = longhands.iter().position(|l| *l == n) {
            sides[i] = part(v).filter(Part::is_length);
        }
    }
    sides
}

/// A declaration an edit sets, or takes out.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "do", rename_all = "kebab-case")]
pub enum StyleEdit {
    Set { name: String, value: String },
    Remove { name: String },
}

/// One side of a box property set to `value` (`side` 0 to 3: top, right,
/// bottom, left), on an element whose declarations are these: with the
/// shorthand there, the shorthand written again, narrowest, and the side
/// longhands it now holds taken out; without it, that side's longhand
/// alone — until all four are set, when they are one shorthand.
pub fn edit_box(
    declarations: &[(&str, &str)],
    property: &str,
    side: usize,
    value: Part,
) -> Vec<StyleEdit> {
    let name = crate::codegen::style_tokens::canonical_style_prop(property);
    let Some(longhands) = sides_of(&name) else {
        return Vec::new();
    };
    let has_shorthand = declarations
        .iter()
        .any(|(n, _)| crate::codegen::style_tokens::canonical_style_prop(n) == name);
    let mut sides = read_box(declarations, &name);
    sides[side.min(3)] = Some(value);
    let present: Vec<&String> = longhands
        .iter()
        .filter(|l| declarations.iter().any(|(n, _)| n == *l))
        .collect();
    let all_set = sides.iter().all(Option::is_some);
    if has_shorthand || all_set {
        let mut out: Vec<StyleEdit> = box_declarations(&name, &sides)
            .into_iter()
            .map(|(name, value)| StyleEdit::Set { name, value })
            .collect();
        out.extend(
            present
                .into_iter()
                .map(|l| StyleEdit::Remove { name: l.clone() }),
        );
        return out;
    }
    let l = &longhands[side.min(3)];
    vec![StyleEdit::Set {
        name: l.clone(),
        value: sides[side.min(3)]
            .as_ref()
            .map(Part::text)
            .unwrap_or_default(),
    }]
}

/// Every curated property, by group, as an inspector lists them.
pub fn catalogue() -> Vec<Property> {
    CURATED
        .iter()
        .flat_map(|(_, props)| props.iter().map(|p| property(p)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(v: f64, unit: &str) -> Part {
        Part::Number {
            value: v,
            unit: unit.into(),
        }
    }

    fn round(property: &str, written: &str) -> String {
        let v = parse(property, written);
        assert!(
            !matches!(v, Value::Free { .. }),
            "{property}: {written} is free"
        );
        let printed = print(property, &v);
        assert_eq!(
            parse(property, &printed),
            v,
            "{property}: {written} → {printed}"
        );
        printed
    }

    /// What the builder's every control writes (board B09).
    #[test]
    fn each_control_writes_what_the_board_says() {
        // padding, linked: one value, every side.
        assert_eq!(
            box_declarations(
                "padding",
                &[
                    Some(n(12.0, "px")),
                    Some(n(12.0, "px")),
                    Some(n(12.0, "px")),
                    Some(n(12.0, "px"))
                ]
            ),
            [("padding".into(), "12px".into())]
        );
        // per side, the pairs match: two values.
        assert_eq!(
            box_declarations(
                "padding",
                &[
                    Some(n(14.0, "px")),
                    Some(n(16.0, "px")),
                    Some(n(14.0, "px")),
                    Some(n(16.0, "px"))
                ]
            ),
            [("padding".into(), "14px 16px".into())]
        );
        // one side alone: its longhand, nothing else.
        assert_eq!(
            box_declarations("padding", &[None, None, None, Some(n(16.0, "px"))]),
            [("padding-left".into(), "16px".into())]
        );
        // corners in the order CSS reads them.
        assert_eq!(
            box_declarations(
                "radius",
                &[
                    Some(n(10.0, "px")),
                    Some(n(10.0, "px")),
                    Some(n(0.0, "")),
                    Some(n(0.0, ""))
                ]
            ),
            [("border-radius".into(), "10px 10px 0 0".into())]
        );
        assert_eq!(round("width", "240px"), "240px");
        assert_eq!(round("opacity", "0.45"), "0.45");
        assert_eq!(round("color", "$text"), "$text");
        assert_eq!(round("border-color", "#3d6fa3"), "#3d6fa3");
        assert_eq!(
            round(
                "background",
                "linear-gradient(to bottom, $surface 0%, $surface-raised 100%)"
            ),
            "linear-gradient(to bottom, $surface, $surface-raised)"
        );
        assert_eq!(round("border", "1px solid $border"), "1px solid $border");
        let Value::Composite { mut parts } = parse("border", "1px solid $border") else {
            panic!()
        };
        parts.pop();
        assert_eq!(
            print("border", &Value::Composite { parts }),
            "1px solid",
            "the colour cleared: the two parts kept"
        );
        assert_eq!(
            round(
                "box-shadow",
                "0 1px 2px rgba(0,0,0,.4), 0 8px 24px rgba(0,0,0,.4)"
            ),
            "0 1px 2px rgba(0,0,0,.4), 0 8px 24px rgba(0,0,0,.4)"
        );
        assert_eq!(round("text-align", "left"), "left");
        assert_eq!(round("display", "flex"), "flex");
        assert_eq!(
            round("grid-template-columns", "1fr 1fr 1fr"),
            "repeat(3, 1fr)"
        );
        assert_eq!(
            round("transition", "background 200ms ease-spring"),
            "background 200ms ease-spring"
        );
        assert_eq!(
            parse("mix-blend-mode", "multiply"),
            Value::Free {
                text: "multiply".into()
            }
        );
        assert_eq!(
            print("mix-blend-mode", &parse("mix-blend-mode", "multiply")),
            "multiply"
        );
    }

    #[test]
    fn a_box_reads_one_to_four_and_writes_the_fewest() {
        assert_eq!(round("padding", "1rem 2rem 1rem 2rem"), "1rem 2rem");
        assert_eq!(round("padding", "1rem 2rem 3rem 2rem"), "1rem 2rem 3rem");
        assert_eq!(round("margin", "0 auto"), "0 auto");
        assert_eq!(round("padding", "$sm $md"), "$sm $md");
        assert_eq!(round("inset", "0"), "0");
        assert!(
            matches!(parse("padding", "{gap}px"), Value::Free { .. }),
            "a splice is the program's"
        );
        assert!(matches!(
            parse("padding", "1px 2px 3px 4px 5px"),
            Value::Free { .. }
        ));
        assert!(matches!(
            parse("padding", "calc(1rem + 2px)"),
            Value::Box { .. }
        ));
        assert_eq!(
            box_declarations(
                "margin",
                &[Some(n(0.0, "")), None, Some(n(8.0, "px")), None]
            ),
            [
                ("margin-top".into(), "0".into()),
                ("margin-bottom".into(), "8px".into())
            ]
        );
    }

    #[test]
    fn a_catalogue_says_what_each_property_is() {
        let p = property("padding");
        assert_eq!(
            (p.group, p.shape, p.tokens),
            ("Spacing", Shape::Box, Some("spacing"))
        );
        assert!(p.suffixes.contains(&"md"));
        assert_eq!(property("radius").name, "border-radius");
        assert_eq!(property("radius").shape, Shape::Corners);
        assert_eq!(property("opacity").range, Some((0.0, 1.0)));
        assert_eq!(property("z-index").range, None);
        assert_eq!(property("display").shape, Shape::Enum);
        assert!(property("display").cases.contains(&"flex"));
        assert_eq!(property("mix-blend-mode").group, "Other");
        assert_eq!(property("mix-blend-mode").shape, Shape::Free);
        assert!(property("padding").on_paper);
        assert!(!property("cursor").on_paper);
        for (_, props) in CURATED {
            for p in *props {
                assert_ne!(property(p).group, "Other", "{p}");
            }
        }
    }

    #[test]
    fn what_no_shape_holds_is_kept_as_written() {
        for (prop, text) in [
            ("width", "calc(100% - 2rem)"),
            ("color", "{accent}"),
            ("display", "table-cell"),
            ("background", "url(/x.png) center / cover no-repeat"),
            ("grid-template-columns", "[full-start] 1fr [full-end]"),
            ("padding", "1rem !important"),
        ] {
            let v = parse(prop, text);
            assert_eq!(print(prop, &v), text, "{prop}");
        }
        assert!(matches!(
            parse("width", "calc(100% - 2rem)"),
            Value::One { .. }
        ));
        assert!(matches!(parse("display", "table-cell"), Value::Free { .. }));
    }

    #[test]
    fn one_side_edited_writes_the_narrowest_it_can() {
        let px = |v: f64| n(v, "px");
        // Nothing set: that side's longhand, and nothing else.
        assert_eq!(
            edit_box(&[], "padding", 3, px(16.0)),
            [StyleEdit::Set {
                name: "padding-left".into(),
                value: "16px".into()
            }]
        );
        // The shorthand there: written again, narrowest.
        assert_eq!(
            edit_box(&[("padding", "14px 16px")], "padding", 0, px(20.0)),
            [StyleEdit::Set {
                name: "padding".into(),
                value: "20px 16px 14px".into()
            }]
        );
        assert_eq!(
            edit_box(&[("padding", "14px 16px 20px")], "padding", 2, px(14.0)),
            [StyleEdit::Set {
                name: "padding".into(),
                value: "14px 16px".into()
            }]
        );
        // A longhand after the shorthand is folded in, and taken out.
        assert_eq!(
            edit_box(
                &[("margin", "0"), ("margin-left", "8px")],
                "margin",
                1,
                px(8.0)
            ),
            [
                StyleEdit::Set {
                    name: "margin".into(),
                    value: "0 8px".into()
                },
                StyleEdit::Remove {
                    name: "margin-left".into()
                }
            ]
        );
        // The fourth longhand set makes them one shorthand.
        let three = [
            ("padding-top", "4px"),
            ("padding-right", "4px"),
            ("padding-bottom", "4px"),
        ];
        let out = edit_box(&three, "padding", 3, px(4.0));
        assert_eq!(
            out[0],
            StyleEdit::Set {
                name: "padding".into(),
                value: "4px".into()
            }
        );
        assert_eq!(out.len(), 4, "{out:?}");
        // What a box reads from.
        assert_eq!(
            read_box(&[("padding", "1px 2px"), ("padding-top", "9px")], "padding"),
            [Some(px(9.0)), Some(px(2.0)), Some(px(1.0)), Some(px(2.0))]
        );
        assert_eq!(
            read_box(&[("radius", "4px")], "border-radius")[2],
            Some(px(4.0))
        );
        assert_eq!(
            read_box(&[("padding", "{p}")], "padding"),
            [None, None, None, None]
        );
        assert!(
            catalogue()
                .iter()
                .any(|p| p.name == "box-shadow" && p.shape == Shape::List)
        );
    }
}
