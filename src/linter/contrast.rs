//! WCAG contrast checking over a project's design tokens.
//!
//! This check is possible because a theme is now WebFluent source rather than
//! CSS the compiler never reads. Every other toolchain has to run a browser and
//! sample rendered pixels to learn what a palette does; here the values are in
//! the AST, so the pairing that will fail an audit can be named at build time,
//! before anyone has written a page with it.
//!
//! Ratios and thresholds follow WCAG 2.2 §1.4.3 (Contrast Minimum) and §1.4.11
//! (Non-text Contrast): 4.5:1 for body text, 3:1 for large text and for the
//! boundaries of user interface components.

use crate::error::A11yWarning;
use crate::parser::ast::{ComponentRef, Declaration, Expr, Program, Statement, StatementKind};

/// WCAG AA for body text.
const AA_TEXT: f64 = 4.5;
/// WCAG AA for large text and non-text UI boundaries.
const AA_LARGE: f64 = 3.0;

/// Token pairs that end up on top of each other in the built-in stylesheet, with
/// the ratio each has to clear.
///
/// Only pairs the engine's own CSS actually composes are listed. A warning about
/// a combination no component renders would be noise, and noise is how a lint
/// gets switched off.
const PAIRS: &[(&str, &str, f64, &str)] = &[
    (
        "color-text",
        "color-background",
        AA_TEXT,
        "body text on the page background",
    ),
    (
        "color-text",
        "color-surface",
        AA_TEXT,
        "text on a card or surface",
    ),
    (
        "color-text-muted",
        "color-background",
        AA_TEXT,
        "muted text on the page background",
    ),
    (
        "color-text-muted",
        "color-surface",
        AA_TEXT,
        "muted text on a surface",
    ),
    (
        "color-primary",
        "color-background",
        AA_LARGE,
        "a primary link or icon on the page background",
    ),
];

// `color-border` against `color-background` is deliberately absent. WCAG 1.4.11
// covers boundaries *required to identify* a control, and this token draws both
// those and purely decorative rules — card edges, dividers, table lines. A
// subtle 1px divider is standard practice and not a failure, so checking the
// token wholesale would report every well-made design. Distinguishing the two
// needs to know which component the border is on, which a token cannot say.

/// Foreground tokens the stylesheet pairs with white text, and the modifier
/// whose variant does so. A site that never writes `Button("x", primary)`
/// has no white label on `--color-primary`, so the pairing is not checked:
/// in structural mode that token paints only the focus ring, and a warning
/// about a button that does not exist is noise the author cannot act on.
const ON_WHITE: &[(&str, &str, &str)] = &[
    ("color-primary", "primary", "a primary button's white label"),
    (
        "color-secondary",
        "secondary",
        "a secondary button's white label",
    ),
    ("color-success", "success", "a success badge's white label"),
    ("color-danger", "danger", "a danger button's white label"),
    ("color-info", "info", "an info badge's white label"),
];

/// Every modifier word written on a builtin element anywhere in the program.
fn used_modifiers(program: &Program) -> std::collections::HashSet<String> {
    fn walk(stmts: &[Statement], out: &mut std::collections::HashSet<String>) {
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::UIElement(el) => {
                    if matches!(el.component, ComponentRef::BuiltIn(_)) {
                        out.extend(el.modifiers.iter().cloned());
                    }
                    walk(&el.children, out);
                }
                StatementKind::If(i) => {
                    walk(&i.then_body, out);
                    for (_, body) in &i.else_if_branches {
                        walk(body, out);
                    }
                    if let Some(body) = &i.else_body {
                        walk(body, out);
                    }
                }
                StatementKind::For(f) => walk(&f.body, out),
                StatementKind::Show(s) => walk(&s.body, out),
                StatementKind::Fetch(f) => {
                    if let Some(body) = &f.loading_block {
                        walk(body, out);
                    }
                    if let Some((_, body)) = &f.error_block {
                        walk(body, out);
                    }
                    if let Some(body) = &f.success_block {
                        walk(body, out);
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = std::collections::HashSet::new();
    for decl in &program.declarations {
        match decl {
            Declaration::Page(p) => walk(&p.body, &mut out),
            Declaration::Component(c) => walk(&c.body, &mut out),
            Declaration::App(a) => walk(&a.body, &mut out),
            _ => {}
        }
    }
    out
}

/// Check the declared theme's tokens, layered over the baseline.
///
/// Warnings name the file by convention (`src/theme.wf`); a caller that
/// merged several files knows the real one and should use [`lint_contrast_in`].
pub fn lint_contrast(
    program: &Program,
    resolved: &std::collections::HashMap<String, String>,
) -> Vec<A11yWarning> {
    lint_contrast_in(program, resolved, &|_| "src/theme.wf".to_string())
}

/// [`lint_contrast`] for a program merged from several files: `file_of` names
/// the file the declaration at that index came from.
pub fn lint_contrast_in(
    program: &Program,
    resolved: &std::collections::HashMap<String, String>,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<A11yWarning> {
    // Only report on projects that actually declare a theme: the baseline is the
    // engine's own problem, and warning about it on every build of every project
    // would say nothing the author can act on.
    let Some((index, theme)) = program
        .declarations
        .iter()
        .enumerate()
        .find_map(|(i, d)| match d {
            Declaration::Theme(t) => Some((i, t)),
            _ => None,
        })
    else {
        return Vec::new();
    };

    let file = file_of(index);
    let file = file.as_str();
    let touched: Vec<&str> = theme.tokens.iter().map(|t| t.name.as_str()).collect();
    let mut warnings = Vec::new();

    for (fg_name, bg_name, threshold, description) in PAIRS {
        // Report only pairings the author's own theme is responsible for.
        if !touched.contains(fg_name) && !touched.contains(bg_name) {
            continue;
        }
        let (Some(fg), Some(bg)) = (resolved.get(*fg_name), resolved.get(*bg_name)) else {
            continue;
        };
        let (Some(fg), Some(bg)) = (parse_colour(fg), parse_colour(bg)) else {
            continue;
        };
        let ratio = contrast_ratio(fg, bg);
        if ratio < *threshold {
            warnings.push(A11yWarning::new(
                "A13",
                format!(
                    "{description} has a contrast ratio of {ratio:.2}:1, below the {threshold}:1 WCAG AA minimum"
                ),
                file,
                line_of(theme, fg_name).max(line_of(theme, bg_name)),
                1,
                format!("Darken --{fg_name} or lighten --{bg_name} until they clear {threshold}:1"),
            ));
        }
    }

    let modifiers = used_modifiers(program);
    for (token, modifier, description) in ON_WHITE {
        if !touched.contains(token) || !modifiers.contains(*modifier) {
            continue;
        }
        let Some(value) = resolved.get(*token).and_then(|v| parse_colour(v)) else {
            continue;
        };
        let ratio = contrast_ratio((255, 255, 255), value);
        if ratio < AA_TEXT {
            warnings.push(A11yWarning::new(
                "A13",
                format!("{description} has a contrast ratio of {ratio:.2}:1, below the {AA_TEXT}:1 WCAG AA minimum"),
                file,
                line_of(theme, token),
                1,
                format!("Darken --{token}: the stylesheet puts white text on it"),
            ));
        }
    }

    warnings
}

fn line_of(theme: &crate::parser::ast::ThemeDecl, token: &str) -> usize {
    theme
        .tokens
        .iter()
        .find(|t| t.name == token)
        .map(|t| t.span.line as usize)
        .unwrap_or(theme.span.line as usize)
}

/// Parse `#rgb`, `#rrggbb` or `rgb(r, g, b)`.
///
/// Anything else — a `var()`, a gradient, a named colour — is left alone rather
/// than guessed at.
fn parse_colour(value: &str) -> Option<(u8, u8, u8)> {
    let v = value.trim();
    if let Some(hex) = v.strip_prefix('#') {
        return match hex.len() {
            3 => {
                let d = |i: usize| u8::from_str_radix(&hex[i..i + 1].repeat(2), 16).ok();
                Some((d(0)?, d(1)?, d(2)?))
            }
            6 => {
                let d = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
                Some((d(0)?, d(2)?, d(4)?))
            }
            _ => None,
        };
    }
    if let Some(rest) = v.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')')) {
        let parts: Vec<&str> = rest.split(',').map(str::trim).collect();
        if parts.len() == 3 {
            return Some((
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            ));
        }
    }
    None
}

/// Relative luminance, per WCAG 2.2.
fn luminance((r, g, b): (u8, u8, u8)) -> f64 {
    let channel = |c: u8| {
        let c = c as f64 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// The WCAG contrast ratio between two colours, from 1:1 to 21:1.
fn contrast_ratio(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (lighter, darker) = if la > lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Whether an expression is a literal colour this check understands.
pub fn is_colour_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::StringLiteral(s) if parse_colour(s).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Program {
        crate::syntax::parse_source(src, "<t>").expect("parse")
    }

    fn check(theme_src: &str) -> Vec<A11yWarning> {
        let program = parse(&format!(
            "{theme_src}\nPage P (path: \"/\") {{ Text(\"x\") }}"
        ));
        let resolved =
            crate::themes::resolve_tokens(&program, &Default::default()).expect("resolve");
        lint_contrast(&program, &resolved)
    }

    /// The reference values from WCAG: black on white is 21:1, and a colour
    /// against itself is 1:1.
    #[test]
    fn the_ratio_matches_the_wcag_definition() {
        assert!((contrast_ratio((0, 0, 0), (255, 255, 255)) - 21.0).abs() < 0.01);
        assert!((contrast_ratio((0x77, 0x77, 0x77), (0x77, 0x77, 0x77)) - 1.0).abs() < 0.01);
        // #767676 on white is the canonical "just passes AA" grey.
        assert!(contrast_ratio((0x76, 0x76, 0x76), (255, 255, 255)) >= 4.5);
    }

    #[test]
    fn colours_parse_in_every_spelling_the_engine_accepts() {
        assert_eq!(parse_colour("#fff"), Some((255, 255, 255)));
        assert_eq!(parse_colour("#0F766E"), Some((0x0F, 0x76, 0x6E)));
        assert_eq!(parse_colour("rgb(15, 118, 110)"), Some((15, 118, 110)));
        assert_eq!(
            parse_colour("var(--x)"),
            None,
            "a reference is not guessed at"
        );
        assert_eq!(parse_colour("rebeccapurple"), None);
    }

    #[test]
    fn a_readable_theme_draws_no_warning() {
        let warnings = check(
            "Theme T {\n  token color-text: \"#111111\"\n  token color-background: \"#FFFFFF\"\n}",
        );
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn grey_on_white_below_the_minimum_is_reported() {
        let warnings = check(
            "Theme T {\n  token color-text-muted: \"#AAAAAA\"\n  token color-background: \"#FFFFFF\"\n}",
        );
        // Both pairings the token appears in are genuinely below the minimum.
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        let joined = warnings
            .iter()
            .map(|w| w.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            joined.contains("muted text on the page background"),
            "{joined}"
        );
        assert!(joined.contains("muted text on a surface"), "{joined}");
        assert!(
            joined.contains("4.5"),
            "the message should name the threshold: {joined}"
        );
    }

    #[test]
    fn a_pale_primary_under_white_text_is_reported() {
        // The stylesheet puts white text on `--color-primary`, so a pale primary
        // is unreadable however good it looks against the page.
        let program = parse(
            "Theme T {\n  token color-primary: \"#FFE066\"\n}\nPage P (path: \"/\") { Button(\"Go\", primary) }",
        );
        let resolved =
            crate::themes::resolve_tokens(&program, &Default::default()).expect("resolve");
        let warnings = lint_contrast(&program, &resolved);
        assert!(
            warnings
                .iter()
                .any(|w| w.to_string().contains("white label")),
            "{warnings:?}"
        );
    }

    #[test]
    fn a_pale_primary_is_not_reported_when_nothing_is_painted_primary() {
        // No `primary` modifier anywhere: the token paints no white label, so
        // there is nothing for the author to fix.
        let warnings = check("Theme T {\n  token color-primary: \"#FFE066\"\n}");
        assert!(
            warnings
                .iter()
                .all(|w| !w.to_string().contains("white label")),
            "{warnings:?}"
        );
    }

    #[test]
    fn a_project_with_no_theme_is_not_lectured_about_the_baseline() {
        let program = parse("Page P (path: \"/\") { Text(\"x\") }");
        let resolved = crate::themes::resolve_tokens(&program, &Default::default()).unwrap();
        assert!(lint_contrast(&program, &resolved).is_empty());
    }

    #[test]
    fn only_pairings_the_theme_touches_are_reported() {
        // The theme sets a readable text colour and nothing else; the baseline's
        // own muted grey is not this author's problem.
        let warnings = check("Theme T {\n  token color-text: \"#000000\"\n}");
        assert!(
            warnings.iter().all(|w| !w.to_string().contains("muted")),
            "{warnings:?}"
        );
    }
}
