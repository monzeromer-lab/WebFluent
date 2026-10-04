//! The stylesheets a page ships, read for print: rules, selectors and
//! declarations, with `@media print` taken in, `@font-face` kept, and every
//! rule that only means something on a screen left out.

/// A whole stylesheet, in source order.
#[derive(Debug, Clone, Default)]
pub struct Sheet {
    pub rules: Vec<Rule>,
    pub font_faces: Vec<FontFace>,
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
    /// `:nth-child(an+b)`.
    NthChild(i32, i32),
    NthOfType(i32, i32),
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
    pub fn parse(css: &str) -> Sheet {
        let mut sheet = Sheet::default();
        let css = strip_comments(css);
        parse_block_contents(&css, &mut sheet);
        sheet
    }

    /// Another sheet's rules after this one's.
    pub fn extend(&mut self, other: Sheet) {
        self.rules.extend(other.rules);
        self.font_faces.extend(other.font_faces);
    }
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => {
                rest = "";
                break;
            }
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

fn parse_block_contents(css: &str, sheet: &mut Sheet) {
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
                    if media_applies_to_print(condition) {
                        parse_block_contents(body, sheet);
                    }
                }
                // A layer or a supported feature is still the page's style.
                "layer" | "supports" => parse_block_contents(body, sheet),
                "font-face" => {
                    if let Some(face) = font_face(body) {
                        sheet.font_faces.push(face);
                    }
                }
                // `@keyframes`, `@container`, `@page` (read elsewhere),
                // `@property`: nothing a printed box takes.
                _ => {}
            }
            continue;
        }
        let selectors: Vec<Selector> = split_top_level(prelude, b',')
            .into_iter()
            .filter_map(|s| parse_selector(s.trim()))
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

/// Whether a media query's rules belong on paper: `print`, `all`, or a
/// query that names no medium and no width (a width is a screen's).
fn media_applies_to_print(condition: &str) -> bool {
    let c = condition.to_ascii_lowercase();
    split_top_level(&c, b',').iter().any(|q| {
        let q = q.trim();
        if q.starts_with("not ") || q.contains("screen") {
            return false;
        }
        if q.contains("print") || q == "all" {
            return true;
        }
        // `(prefers-reduced-motion)`, `(hover: hover)`, widths: a screen's.
        false
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
/// (`:hover`, `:focus`, `::placeholder`) — the rule then never applies.
pub fn parse_selector(s: &str) -> Option<Selector> {
    let mut parts: Vec<(Compound, Option<Combinator>)> = Vec::new();
    let mut pending: Option<Combinator> = None;
    let mut rest = s.trim();
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
    if parts.is_empty() {
        return None;
    }
    let mut spec = (0, 0, 0);
    for (c, _) in &parts {
        let (a, b, d) = compound_specificity(c);
        spec = (spec.0 + a, spec.1 + b, spec.2 + d);
    }
    Some(Selector {
        parts,
        specificity: spec,
    })
}

fn compound_specificity(c: &Compound) -> (u32, u32, u32) {
    let mut a = c.id.is_some() as u32;
    let mut b = (c.classes.len() + c.attrs.len()) as u32;
    let mut d = c.tag.is_some() as u32;
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
fn parse_compound(s: &str) -> Option<(Compound, usize)> {
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
                let close = s[i..].find(']')? + i;
                c.attrs.push(parse_attr(&s[i + 1..close])?);
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
                    let close = matching_paren(s, end)?;
                    arg = Some(&s[end + 1..close]);
                    next = close + 1;
                }
                let pseudo = match (name.as_str(), arg) {
                    ("first-child", None) => Pseudo::FirstChild,
                    ("last-child", None) => Pseudo::LastChild,
                    ("only-child", None) => Pseudo::OnlyChild,
                    ("first-of-type", None) => Pseudo::FirstOfType,
                    ("last-of-type", None) => Pseudo::LastOfType,
                    ("empty", None) => Pseudo::Empty,
                    ("root", None) => Pseudo::Root,
                    ("nth-child", Some(a)) => {
                        let (x, y) = parse_nth(a)?;
                        Pseudo::NthChild(x, y)
                    }
                    ("nth-of-type", Some(a)) => {
                        let (x, y) = parse_nth(a)?;
                        Pseudo::NthOfType(x, y)
                    }
                    ("not" | "is" | "where" | "matches", Some(a)) => {
                        let mut inner = Vec::new();
                        for part in split_top_level(a, b',') {
                            let part = part.trim();
                            let (comp, used) = parse_compound(part)?;
                            if used != part.len() {
                                return None;
                            }
                            inner.push(comp);
                        }
                        match name.as_str() {
                            "not" => Pseudo::Not(inner),
                            "where" => Pseudo::Where(inner),
                            _ => Pseudo::Is(inner),
                        }
                    }
                    ("before", None) => Pseudo::Before,
                    ("after", None) => Pseudo::After,
                    // Anything else is a screen's (hover, focus, visited,
                    // placeholder, selection, marker…).
                    _ => return None,
                };
                c.pseudos.push(pseudo);
                i = next;
            }
            _ => break,
        }
    }
    if i == 0 {
        return None;
    }
    Some((c, i))
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

fn parse_attr(inner: &str) -> Option<AttrTest> {
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
            return Some(AttrTest {
                name: name.trim().to_ascii_lowercase(),
                op: Some((op, value)),
            });
        }
    }
    Some(AttrTest {
        name: inner.trim().to_ascii_lowercase(),
        op: None,
    })
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
