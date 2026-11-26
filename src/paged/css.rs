//! The stylesheets a page ships, read for print: rules, selectors and
//! declarations, with `@media` queries asked of the page, `@page` and
//! `@font-face` kept, and every rule that only means something on a screen
//! left out.

/// A whole stylesheet, in source order.
#[derive(Debug, Clone, Default)]
pub struct Sheet {
    pub rules: Vec<Rule>,
    pub font_faces: Vec<FontFace>,
    /// The `@page` rules, in source order.
    pub pages: Vec<PageRule>,
    /// The pseudo-classes and pseudo-elements a selector named that the
    /// engine does not know, each once: their rules are left out, as a
    /// browser leaves out a selector it cannot read.
    pub unknown: Vec<String>,
    /// What else the sheet asked for that paper does not draw, each once
    /// (an `@page` margin box).
    pub unsupported: Vec<String>,
}

/// What a media query is asked of: the page area a document is laid out in,
/// in CSS pixels (`1px` is `0.75pt`). A query that names a width or a
/// height is answered against it; with no page (`None`) it is false.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Media {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Media {
    /// The page area `width` × `height` points wide and tall.
    pub fn page(width: f32, height: f32) -> Media {
        Media {
            width: Some(width / 0.75),
            height: Some(height / 0.75),
        }
    }
}

/// Which pages an `@page` rule is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSelector {
    All,
    First,
    Left,
    Right,
}

/// `@page :first { margin-top: 2cm }`.
#[derive(Debug, Clone, PartialEq)]
pub struct PageRule {
    pub selector: PageSelector,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: String,
    pub important: bool,
}

/// `@font-face { font-family: X; src: url(…); font-weight; font-style }`.
#[derive(Debug, Clone, PartialEq)]
pub struct FontFace {
    pub family: String,
    pub sources: Vec<String>,
    pub weight: Option<String>,
    pub style: Option<String>,
}

/// A complex selector: compounds joined by combinators, read right to left
/// when matched.
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    /// `(compound, combinator to the compound before it)`; the first has none.
    pub parts: Vec<(Compound, Option<Combinator>)>,
    pub specificity: (u32, u32, u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    Descendant,
    Child,
    Adjacent,
    Sibling,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Compound {
    /// `None` for `*` or no tag.
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attrs: Vec<AttrTest>,
    pub pseudos: Vec<Pseudo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttrTest {
    pub name: String,
    /// `None` for `[name]`.
    pub op: Option<(AttrOp, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrOp {
    Equals,
    Includes,
    DashMatch,
    Prefix,
    Suffix,
    Substring,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pseudo {
    FirstChild,
    LastChild,
    OnlyChild,
    FirstOfType,
    LastOfType,
    OnlyOfType,
    /// `:nth-child(an+b)`.
    NthChild(i32, i32),
    NthOfType(i32, i32),
    /// `:nth-last-child(an+b)`, counted from the end.
    NthLastChild(i32, i32),
    NthLastOfType(i32, i32),
    /// `:link`, `:any-link`: an `a` or `area` with an `href` (on paper no
    /// link has been visited).
    Link,
    /// `:checked`: a box or radio with `checked`, an option `selected`.
    Checked,
    /// `:disabled` and `:enabled`, from the `disabled` attribute.
    Disabled,
    Enabled,
    /// `:dir(rtl)`: whether the element's direction, from the nearest
    /// `dir` attribute on it or above it, is right to left.
    Dir(bool),
    /// `:lang(ar, en)`: the nearest `lang` on it or above it is one of these
    /// or a region of one.
    Lang(Vec<String>),
    /// `:has(> img, + p)`: relative selectors, each with the combinator that
    /// joins it to the element (descendant when none is written).
    Has(Vec<(Combinator, Selector)>),
    Empty,
    Root,
    Not(Vec<Compound>),
    /// `:is(a, b)`: any of them, as specific as the most specific.
    Is(Vec<Compound>),
    /// `:where(a, b)`: any of them, adding nothing to the specificity — how a
    /// default is written that any rule of the author's overrides.
    Where(Vec<Compound>),
    /// `::before` / `::after`: kept so the rule is known, matched only when a
    /// generated box is asked for.
    Before,
    After,
}

impl Sheet {
    /// A sheet read with no page to ask a width query of: such a query is
    /// false.
    pub fn parse(css: &str) -> Sheet {
        Sheet::parse_for(css, Media::default())
    }

    /// A sheet read for a page area: a `@media` query on its width, height
    /// or orientation is answered against it.
    pub fn parse_for(css: &str, media: Media) -> Sheet {
        let mut sheet = Sheet::default();
        let css = strip_comments(css);
        parse_block_contents(&css, &mut sheet, media);
        sheet
    }

    /// Another sheet's rules after this one's.
    pub fn extend(&mut self, other: Sheet) {
        self.rules.extend(other.rules);
        self.font_faces.extend(other.font_faces);
        self.pages.extend(other.pages);
        for u in other.unknown {
            if !self.unknown.contains(&u) {
                self.unknown.push(u);
            }
        }
        for u in other.unsupported {
            if !self.unsupported.contains(&u) {
                self.unsupported.push(u);
            }
        }
    }
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        if let Some(end) = rest[start + 2..].find("*/") {
            rest = &rest[start + 2 + end + 2..];
        } else {
            rest = "";
            break;
        }
    }
    out.push_str(rest);
    out
}

/// The index of the `}` that closes the block whose `{` is at `open`,
/// skipping strings.
fn matching_brace(s: &str, open: usize) -> usize {
    let b = s.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    let mut quote: Option<u8> = None;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = quote {
            if c == b'\\' {
                i += 1;
            } else if c == q {
                quote = None;
            }
        } else {
            match c {
                b'"' | b'\'' => quote = Some(c),
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    b.len()
}

fn parse_block_contents(css: &str, sheet: &mut Sheet, media: Media) {
    let mut i = 0;
    let b = css.as_bytes();
    while i < b.len() {
        while i < b.len() && (b[i].is_ascii_whitespace() || b[i] == b';') {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        let Some(brace) = css[i..].find('{').map(|p| i + p) else {
            break;
        };
        // An at-rule with no block (`@import …;`, `@charset`).
        if b[i] == b'@'
            && let Some(semi) = css[i..].find(';').map(|p| i + p)
            && semi < brace
        {
            i = semi + 1;
            continue;
        }
        let prelude = css[i..brace].trim();
        let close = matching_brace(css, brace);
        let body = &css[brace + 1..close.min(css.len())];
        i = close + 1;
        if let Some(at) = prelude.strip_prefix('@') {
            let (name, condition) = at
                .split_once(|c: char| c.is_whitespace())
                .unwrap_or((at, ""));
            match name.to_ascii_lowercase().as_str() {
                "media" => {
                    if media_applies(condition, media) {
                        parse_block_contents(body, sheet, media);
                    }
                }
                // A layer or a supported feature is still the page's style.
                "layer" | "supports" => parse_block_contents(body, sheet, media),
                "font-face" => {
                    if let Some(face) = font_face(body) {
                        sheet.font_faces.push(face);
                    }
                }
                "page" => page_rule(condition, body, sheet),
                // `@keyframes`, `@container`, `@property`: nothing a printed
                // box takes.
                _ => {}
            }
            continue;
        }
        let selectors: Vec<Selector> = split_top_level(prelude, b',')
            .into_iter()
            .filter_map(|s| match read_selector(s.trim()) {
                Ok(sel) => Some(sel),
                Err(Some(unknown)) => {
                    if !sheet.unknown.contains(&unknown) {
                        sheet.unknown.push(unknown);
                    }
                    None
                }
                Err(None) => None,
            })
            .collect();
        if selectors.is_empty() {
            continue;
        }
        let declarations = parse_declarations(body);
        if !declarations.is_empty() {
            sheet.rules.push(Rule {
                selectors,
                declarations,
            });
        }
    }
}

/// An `@page` rule: which pages it is for, and its declarations. A named
/// page (`@page chapter`) is never asked for — nothing gives an element a
/// `page:` — so its rule is left out; a margin box inside one (`@top-center
/// { … }`) is noted and left out, since a document draws its `Header` and
/// `Footer` there instead.
fn page_rule(prelude: &str, body: &str, sheet: &mut Sheet) {
    let selector = match prelude.trim().to_ascii_lowercase().as_str() {
        "" => PageSelector::All,
        ":first" => PageSelector::First,
        ":left" => PageSelector::Left,
        ":right" => PageSelector::Right,
        _ => return,
    };
    let mut flat = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(at) = rest.find('@') {
        flat.push_str(&rest[..at]);
        let Some(open) = rest[at..].find('{').map(|o| at + o) else {
            rest = "";
            break;
        };
        let note = format!("the `@page` margin box `@{}`", rest[at + 1..open].trim());
        if !sheet.unsupported.contains(&note) {
            sheet.unsupported.push(note);
        }
        let close = matching_brace(rest, open);
        flat.push(';');
        rest = rest.get(close + 1..).unwrap_or("");
    }
    flat.push_str(rest);
    let declarations = parse_declarations(&flat);
    if !declarations.is_empty() {
        sheet.pages.push(PageRule {
            selector,
            declarations,
        });
    }
}

/// Whether a media query list holds for a printed page: `print` and `all`
/// do, `screen` does not, and a feature is answered against the page area —
/// `min-width`, `max-width`, `width` and their range forms (`(width >=
/// 600px)`, `(400px <= width <= 700px)`), the same of `height`, and
/// `orientation`. A feature only a screen has is answered as paper would
/// (`(hover: none)`, `(prefers-color-scheme: light)`), and one nothing knows
/// is false.
pub fn media_applies(condition: &str, media: Media) -> bool {
    let c = condition.to_ascii_lowercase();
    split_top_level(&c, b',')
        .iter()
        .any(|q| media_query(q.trim(), media))
}

fn media_query(q: &str, media: Media) -> bool {
    let (negated, q) = match q.strip_prefix("not ") {
        Some(rest) => (true, rest.trim()),
        None => (false, q.strip_prefix("only ").unwrap_or(q).trim()),
    };
    let mut ok = true;
    let mut rest = q;
    let mut first = true;
    while !rest.is_empty() {
        let part;
        if rest.starts_with('(') {
            let Some(close) = matching_paren(rest, 0) else {
                return false;
            };
            part = &rest[..=close];
            rest = rest[close + 1..].trim_start();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            part = &rest[..end];
            rest = rest[end..].trim_start();
        }
        match part {
            "and" => continue,
            // `a or b`: what came before holds, or what follows must.
            "or" => {
                if ok {
                    return !negated;
                }
                ok = true;
                continue;
            }
            "print" | "all" if first => {}
            _ if part.starts_with('(') => ok &= media_feature(&part[1..part.len() - 1], media),
            // `screen`, `speech`, a media type nothing knows.
            _ => ok = false,
        }
        first = false;
    }
    ok != negated
}

/// A length in a media query, in CSS pixels; `em` and `rem` there are the
/// initial font size, 16px.
fn query_px(v: &str) -> Option<f32> {
    let v = v.trim();
    if v == "0" {
        return Some(0.0);
    }
    for (unit, factor) in [
        ("px", 1.0),
        ("rem", 16.0),
        ("em", 16.0),
        ("pt", 1.0 / 0.75),
        ("pc", 16.0),
        ("in", 96.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
    ] {
        if let Some(n) = v.strip_suffix(unit) {
            return n.trim().parse::<f32>().ok().map(|n| n * factor);
        }
    }
    None
}

/// One parenthesised feature: `min-width: 600px`, `width >= 40em`,
/// `400px <= width <= 700px`, `orientation: portrait`, `not (…)`.
fn media_feature(f: &str, media: Media) -> bool {
    let f = f.trim();
    if let Some(inner) = f.strip_prefix("not ") {
        let inner = inner.trim();
        let inner = inner
            .strip_prefix('(')
            .and_then(|i| i.strip_suffix(')'))
            .unwrap_or(inner);
        return !media_feature(inner, media);
    }
    if f.starts_with('(') {
        return media_query(f, media);
    }
    if let Some((name, value)) = f.split_once(':') {
        let (name, value) = (name.trim(), value.trim());
        let cmp = |have: Option<f32>, test: fn(f32, f32) -> bool| {
            have.zip(query_px(value)).is_some_and(|(h, v)| test(h, v))
        };
        let min = |h: f32, v: f32| h + 0.01 >= v;
        let max = |h: f32, v: f32| h <= v + 0.01;
        let eq = |h: f32, v: f32| (h - v).abs() < 0.5;
        return match name {
            "orientation" => match (media.width, media.height) {
                (Some(w), Some(h)) => (value == "portrait") == (h >= w),
                _ => false,
            },
            "min-width" | "min-device-width" => cmp(media.width, min),
            "max-width" | "max-device-width" => cmp(media.width, max),
            "width" | "device-width" => cmp(media.width, eq),
            "min-height" | "min-device-height" => cmp(media.height, min),
            "max-height" | "max-device-height" => cmp(media.height, max),
            "height" | "device-height" => cmp(media.height, eq),
            // Paper shows colour, and nothing on it moves, hovers or is
            // pointed at.
            "prefers-color-scheme" => value == "light",
            "hover" | "any-hover" | "pointer" | "any-pointer" => value == "none",
            "prefers-reduced-motion" => value == "reduce",
            _ => false,
        };
    }
    if f == "color" {
        return true;
    }
    // The range forms: `width >= 600px`, `400px < width <= 700px`.
    let mut tokens: Vec<&str> = Vec::new();
    let mut rest = f;
    loop {
        let at = rest.find(['<', '>', '=']).unwrap_or(rest.len());
        tokens.push(rest[..at].trim());
        rest = &rest[at..];
        if rest.is_empty() {
            break;
        }
        let len = if rest[1..].starts_with('=') { 2 } else { 1 };
        tokens.push(&rest[..len]);
        rest = &rest[len..];
    }
    let value_of = |t: &str| match t {
        "width" | "device-width" => media.width,
        "height" | "device-height" => media.height,
        _ => query_px(t),
    };
    let is_dim = |t: &str| matches!(t, "width" | "device-width" | "height" | "device-height");
    if tokens.len() < 3 || !tokens.iter().any(|t| is_dim(t)) {
        return false;
    }
    tokens.windows(3).step_by(2).all(|w| {
        let (Some(a), Some(b)) = (value_of(w[0]), value_of(w[2])) else {
            return false;
        };
        match w[1] {
            "<" => a < b,
            "<=" => a <= b + 0.01,
            ">" => a > b,
            ">=" => a + 0.01 >= b,
            "=" => (a - b).abs() < 0.5,
            _ => false,
        }
    })
}

fn font_face(body: &str) -> Option<FontFace> {
    let decls = parse_declarations(body);
    let get = |n: &str| decls.iter().find(|d| d.name == n).map(|d| d.value.clone());
    let family = get("font-family")?
        .trim()
        .trim_matches(['"', '\''])
        .to_string();
    let sources = get("src")
        .map(|src| {
            split_top_level(&src, b',')
                .into_iter()
                .filter_map(|part| {
                    let part = part.trim();
                    let start = part.find("url(")? + 4;
                    let end = part[start..].find(')')? + start;
                    Some(
                        part[start..end]
                            .trim()
                            .trim_matches(['"', '\''])
                            .to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    Some(FontFace {
        family,
        sources,
        weight: get("font-weight"),
        style: get("font-style"),
    })
}

/// `a: 1; b: url(x;y) !important` → the declarations, in order. A nested
/// rule (CSS nesting) inside a declaration block is skipped.
pub fn parse_declarations(body: &str) -> Vec<Declaration> {
    let mut out = Vec::new();
    for part in split_top_level(body, b';') {
        let part = part.trim();
        if part.is_empty() || part.contains('{') {
            continue;
        }
        let Some((name, value)) = part.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() || name.contains(char::is_whitespace) {
            continue;
        }
        let mut value = value.trim().to_string();
        let mut important = false;
        if let Some(stripped) = value
            .strip_suffix("!important")
            .or_else(|| value.strip_suffix("! important"))
        {
            value = stripped.trim().to_string();
            important = true;
        }
        out.push(Declaration {
            name,
            value,
            important,
        });
    }
    out
}

/// Split on `sep` where it is not inside parentheses, brackets, braces or a
/// string.
pub fn split_top_level(s: &str, sep: u8) -> Vec<&str> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    let mut start = 0;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = quote {
            if c == b'\\' {
                i += 1;
            } else if c == q {
                quote = None;
            }
        } else {
            match c {
                b'"' | b'\'' => quote = Some(c),
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                _ if c == sep && depth == 0 => {
                    out.push(&s[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        i += 1;
    }
    out.push(&s[start..]);
    out
}

/// Split on whitespace outside parentheses: `1px solid rgb(0, 0, 0)` →
/// three parts.
pub fn split_spaces(s: &str) -> Vec<&str> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    let mut start: Option<usize> = None;
    for (i, &c) in b.iter().enumerate() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            b'"' | b'\'' => {
                quote = Some(c);
                start.get_or_insert(i);
            }
            b'(' => {
                depth += 1;
                start.get_or_insert(i);
            }
            b')' => depth -= 1,
            _ if c.is_ascii_whitespace() && depth == 0 => {
                if let Some(s0) = start.take() {
                    out.push(&s[s0..i]);
                }
            }
            _ => {
                start.get_or_insert(i);
            }
        }
    }
    if let Some(s0) = start {
        out.push(&s[s0..]);
    }
    out
}

// ─── Selectors ──────────────────────────────────────────────────────

/// A selector, or `None` when it names something a printed page never is
/// (`:hover`, `:focus`, `::placeholder`) or something the engine does not
/// know — the rule then never applies.
pub fn parse_selector(s: &str) -> Option<Selector> {
    read_selector(s).ok()
}

/// What no printed page ever is, or what only a screen draws: a selector
/// naming one is left out without a word. Any other name the engine does
/// not know is reported, so an author learns why a rule did nothing.
const SCREEN_ONLY: &[&str] = &[
    "hover",
    "focus",
    "focus-visible",
    "focus-within",
    "active",
    "visited",
    "target",
    "target-within",
    "placeholder-shown",
    "placeholder",
    "selection",
    "marker",
    "backdrop",
    "file-selector-button",
    "first-letter",
    "first-line",
    "cue",
    "popover-open",
    "modal",
    "open",
    "closed",
    "fullscreen",
    "picture-in-picture",
    "playing",
    "paused",
    "autofill",
    "invalid",
    "valid",
    "user-invalid",
    "user-valid",
    "in-range",
    "out-of-range",
    "required",
    "optional",
    "read-only",
    "read-write",
    "indeterminate",
    "default",
    "defined",
    "host",
    "host-context",
    "slotted",
    "part",
    "state",
    "scope",
    "view-transition",
    "view-transition-group",
    "view-transition-image-pair",
    "view-transition-old",
    "view-transition-new",
    "active-view-transition",
    "scroll-marker",
    "scroll-button",
    "details-content",
    "search-text",
    "target-text",
    "spelling-error",
    "grammar-error",
    "highlight",
    "picker",
    "picker-icon",
    "checkmark",
];

/// A selector; `Err(Some(name))` when it names a pseudo-class or
/// pseudo-element the engine does not know, `Err(None)` when it cannot be
/// read or only a screen ever matches it.
pub fn read_selector(s: &str) -> Result<Selector, Option<String>> {
    let parts = read_complex(s.trim())?;
    if parts.is_empty() {
        return Err(None);
    }
    let mut spec = (0, 0, 0);
    for (c, _) in &parts {
        let (a, b, d) = compound_specificity(c);
        spec = (spec.0 + a, spec.1 + b, spec.2 + d);
    }
    Ok(Selector {
        parts,
        specificity: spec,
    })
}

/// The compounds of a complex selector and the combinators between them.
fn read_complex(s: &str) -> Result<Vec<(Compound, Option<Combinator>)>, Option<String>> {
    let mut parts: Vec<(Compound, Option<Combinator>)> = Vec::new();
    let mut pending: Option<Combinator> = None;
    let mut rest = s;
    while !rest.is_empty() {
        let first = rest.as_bytes()[0];
        if first.is_ascii_whitespace() {
            rest = rest.trim_start();
            if pending.is_none() && !parts.is_empty() {
                pending = Some(Combinator::Descendant);
            }
            continue;
        }
        let combinator = match first {
            b'>' => Some(Combinator::Child),
            b'+' => Some(Combinator::Adjacent),
            b'~' => Some(Combinator::Sibling),
            _ => None,
        };
        if let Some(c) = combinator {
            pending = Some(c);
            rest = rest[1..].trim_start();
            continue;
        }
        let (compound, used) = parse_compound(rest)?;
        rest = &rest[used..];
        let join = if parts.is_empty() {
            None
        } else {
            pending.take().or(Some(Combinator::Descendant))
        };
        parts.push((compound, join));
    }
    Ok(parts)
}

/// `> img`, `+ p`, `.note`: a relative selector of `:has()`, and the
/// combinator that joins it to the element `:has()` is on.
fn read_relative(s: &str) -> Result<(Combinator, Selector), Option<String>> {
    let s = s.trim();
    let (lead, rest) = match s.as_bytes().first() {
        Some(b'>') => (Combinator::Child, &s[1..]),
        Some(b'+') => (Combinator::Adjacent, &s[1..]),
        Some(b'~') => (Combinator::Sibling, &s[1..]),
        _ => (Combinator::Descendant, s),
    };
    let sel = read_selector(rest)?;
    // `:has()` does not nest, and a pseudo-element is never in one.
    let nested = sel.parts.iter().any(|(c, _)| {
        c.pseudos
            .iter()
            .any(|p| matches!(p, Pseudo::Has(_) | Pseudo::Before | Pseudo::After))
    });
    if nested {
        return Err(None);
    }
    Ok((lead, sel))
}

fn compound_specificity(c: &Compound) -> (u32, u32, u32) {
    let mut a = u32::from(c.id.is_some());
    let mut b = (c.classes.len() + c.attrs.len()) as u32;
    let mut d = u32::from(c.tag.is_some());
    for p in &c.pseudos {
        match p {
            // `:not()` and `:is()` count as their most specific argument.
            Pseudo::Not(inner) | Pseudo::Is(inner) => {
                let (x, y, z) = inner
                    .iter()
                    .map(compound_specificity)
                    .max()
                    .unwrap_or_default();
                a += x;
                b += y;
                d += z;
            }
            // So does `:has()`.
            Pseudo::Has(inner) => {
                let (x, y, z) = inner
                    .iter()
                    .map(|(_, s)| s.specificity)
                    .max()
                    .unwrap_or_default();
                a += x;
                b += y;
                d += z;
            }
            Pseudo::Where(_) => {}
            Pseudo::Before | Pseudo::After => d += 1,
            _ => b += 1,
        }
    }
    (a, b, d)
}

fn ident_end(s: &str) -> usize {
    s.find(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_' || c == '\\' || !c.is_ascii()))
        .unwrap_or(s.len())
}

/// One compound selector from the start of `s`, and how much of `s` it took.
fn parse_compound(s: &str) -> Result<(Compound, usize), Option<String>> {
    let mut c = Compound::default();
    let mut i = 0;
    let b = s.as_bytes();
    if b[0] == b'*' {
        i = 1;
    } else if b[0].is_ascii_alphabetic() {
        let end = ident_end(s);
        c.tag = Some(s[..end].to_ascii_lowercase());
        i = end;
    }
    while i < b.len() {
        match b[i] {
            b'.' => {
                let end = i + 1 + ident_end(&s[i + 1..]);
                c.classes.push(unescape(&s[i + 1..end]));
                i = end;
            }
            b'#' => {
                let end = i + 1 + ident_end(&s[i + 1..]);
                c.id = Some(s[i + 1..end].to_string());
                i = end;
            }
            b'[' => {
                let close = s[i..].find(']').ok_or(None)? + i;
                c.attrs.push(parse_attr(&s[i + 1..close]));
                i = close + 1;
            }
            b':' => {
                let element = s[i..].starts_with("::");
                let start = i + if element { 2 } else { 1 };
                let end = start + ident_end(&s[start..]);
                let name = s[start..end].to_ascii_lowercase();
                let mut arg = None;
                let mut next = end;
                if s[end..].starts_with('(') {
                    let close = matching_paren(s, end).ok_or(None)?;
                    arg = Some(&s[end + 1..close]);
                    next = close + 1;
                }
                let nth = |a: &str| parse_nth(a).ok_or(None);
                let pseudo = match (name.as_str(), arg) {
                    ("first-child", None) => Pseudo::FirstChild,
                    ("last-child", None) => Pseudo::LastChild,
                    ("only-child", None) => Pseudo::OnlyChild,
                    ("first-of-type", None) => Pseudo::FirstOfType,
                    ("last-of-type", None) => Pseudo::LastOfType,
                    ("only-of-type", None) => Pseudo::OnlyOfType,
                    ("empty", None) => Pseudo::Empty,
                    ("root", None) => Pseudo::Root,
                    ("link" | "any-link", None) => Pseudo::Link,
                    ("checked", None) => Pseudo::Checked,
                    ("disabled", None) => Pseudo::Disabled,
                    ("enabled", None) => Pseudo::Enabled,
                    ("nth-child", Some(a)) => {
                        let (x, y) = nth(a)?;
                        Pseudo::NthChild(x, y)
                    }
                    ("nth-of-type", Some(a)) => {
                        let (x, y) = nth(a)?;
                        Pseudo::NthOfType(x, y)
                    }
                    ("nth-last-child", Some(a)) => {
                        let (x, y) = nth(a)?;
                        Pseudo::NthLastChild(x, y)
                    }
                    ("nth-last-of-type", Some(a)) => {
                        let (x, y) = nth(a)?;
                        Pseudo::NthLastOfType(x, y)
                    }
                    ("not" | "is" | "where" | "matches", Some(a)) => {
                        let mut inner = Vec::new();
                        for part in split_top_level(a, b',') {
                            let part = part.trim();
                            let (comp, used) = parse_compound(part)?;
                            if used != part.len() {
                                return Err(None);
                            }
                            inner.push(comp);
                        }
                        match name.as_str() {
                            "not" => Pseudo::Not(inner),
                            "where" => Pseudo::Where(inner),
                            _ => Pseudo::Is(inner),
                        }
                    }
                    ("dir", Some(a)) => match a.trim().to_ascii_lowercase().as_str() {
                        "rtl" => Pseudo::Dir(true),
                        "ltr" => Pseudo::Dir(false),
                        _ => return Err(None),
                    },
                    ("lang", Some(a)) => Pseudo::Lang(
                        split_top_level(a, b',')
                            .into_iter()
                            .map(|l| l.trim().trim_matches(['"', '\'']).to_ascii_lowercase())
                            .filter(|l| !l.is_empty())
                            .collect(),
                    ),
                    ("has", Some(a)) => Pseudo::Has(
                        split_top_level(a, b',')
                            .into_iter()
                            .map(read_relative)
                            .collect::<Result<_, _>>()?,
                    ),
                    ("before", None) => Pseudo::Before,
                    ("after", None) => Pseudo::After,
                    (n, _) if SCREEN_ONLY.contains(&n) || n.starts_with('-') => {
                        return Err(None);
                    }
                    (n, _) => {
                        return Err(Some(format!("{}{n}", if element { "::" } else { ":" })));
                    }
                };
                c.pseudos.push(pseudo);
                i = next;
            }
            _ => break,
        }
    }
    if i == 0 {
        return Err(None);
    }
    Ok((c, i))
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

fn unescape(s: &str) -> String {
    s.replace('\\', "")
}

fn parse_attr(inner: &str) -> AttrTest {
    let ops = [
        ("~=", AttrOp::Includes),
        ("|=", AttrOp::DashMatch),
        ("^=", AttrOp::Prefix),
        ("$=", AttrOp::Suffix),
        ("*=", AttrOp::Substring),
        ("=", AttrOp::Equals),
    ];
    for (text, op) in ops {
        if let Some((name, value)) = inner.split_once(text) {
            let value = value.trim();
            // `[a="b" i]`: the flag is ignored.
            let value = value
                .strip_suffix(" i")
                .or_else(|| value.strip_suffix(" s"))
                .unwrap_or(value)
                .trim()
                .trim_matches(['"', '\''])
                .to_string();
            return AttrTest {
                name: name.trim().to_ascii_lowercase(),
                op: Some((op, value)),
            };
        }
    }
    AttrTest {
        name: inner.trim().to_ascii_lowercase(),
        op: None,
    }
}

/// `odd`, `even`, `3`, `2n+1`, `-n+3` → `(a, b)`.
fn parse_nth(s: &str) -> Option<(i32, i32)> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    match s.as_str() {
        "odd" => return Some((2, 1)),
        "even" => return Some((2, 0)),
        _ => {}
    }
    if let Some(npos) = s.find('n') {
        let a = match &s[..npos] {
            "" | "+" => 1,
            "-" => -1,
            n => n.parse().ok()?,
        };
        let b = if s[npos + 1..].is_empty() {
            0
        } else {
            s[npos + 1..].trim_start_matches('+').parse().ok()?
        };
        Some((a, b))
    } else {
        Some((0, s.parse().ok()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rules_and_leaves_out_the_screens() {
        let sheet = Sheet::parse(
            r#"
            /* a comment { with braces } */
            .wf-card { background: var(--color-surface); border: 1px solid var(--color-border) }
            .wf-badge--primary, .wf-tag--primary { color: #fff !important; }
            .wf-button:hover { color: red }
            @media (max-width: 768px) { .wf-row { flex-direction: column } }
            @media print { .no-print { display: none } }
            @keyframes spin { from { transform: rotate(0) } to { transform: rotate(1turn) } }
            @font-face { font-family: "Manrope"; src: url("/fonts/Manrope.ttf") format("truetype"); font-weight: 400 }
            .wf-markdown > * + * { margin-top: 1rem }
            "#,
        );
        let selectors: Vec<usize> = sheet.rules.iter().map(|r| r.selectors.len()).collect();
        // The hover rule never applies, the screen query is skipped, the
        // print query is taken in.
        assert_eq!(selectors, vec![1, 2, 1, 1]);
        assert!(sheet.rules[1].declarations[0].important);
        assert_eq!(sheet.font_faces[0].family, "Manrope");
        assert_eq!(sheet.font_faces[0].sources, vec!["/fonts/Manrope.ttf"]);
        let last = &sheet.rules[3].selectors[0];
        assert_eq!(last.parts.len(), 3);
        assert_eq!(last.parts[1].1, Some(Combinator::Child));
        assert_eq!(last.parts[2].1, Some(Combinator::Adjacent));
    }

    #[test]
    fn specificity_counts_ids_classes_and_tags() {
        let s = |t: &str| parse_selector(t).unwrap().specificity;
        assert_eq!(s("h1.wf-heading"), (0, 1, 1));
        assert_eq!(s(".a.a.a"), (0, 3, 0));
        assert_eq!(s("#x .y > p:first-child"), (1, 2, 1));
        assert_eq!(s(".wf-grid[data-cols=\"3\"]"), (0, 2, 0));
        assert_eq!(s("li:not(.done)"), (0, 1, 1));
        assert_eq!(s(":where(.wf-slide) h1"), (0, 0, 1));
        assert_eq!(s(":is(.a, #b) p"), (1, 0, 1));
        assert_eq!(s("li:not(.a, .b.c)"), (0, 2, 1));
        assert!(parse_selector("a:hover").is_none());
        assert!(parse_selector("input::placeholder").is_none());
    }

    #[test]
    fn splits_values_where_css_does() {
        assert_eq!(
            split_spaces("1px solid rgb(0, 0, 0)"),
            vec!["1px", "solid", "rgb(0, 0, 0)"]
        );
        assert_eq!(
            split_top_level("linear-gradient(90deg, #fff, #000), url(a,b)", b','),
            vec!["linear-gradient(90deg, #fff, #000)", " url(a,b)"]
        );
        assert_eq!(parse_nth("2n+1"), Some((2, 1)));
        assert_eq!(parse_nth("-n+3"), Some((-1, 3)));
        assert_eq!(parse_nth("even"), Some((2, 0)));
    }
}
