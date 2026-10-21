//! A theme as an editor sees it: every token the build resolves, by the
//! compiler's own groups, each declared by the theme or inherited from the
//! baseline, with the properties that draw from it and every place the
//! project names it; the colour pairings the stylesheet composes with
//! their contrast; and, when the config names a dark theme, the same for
//! it beside — with the tokens one of the two declares and the other not.
//!
//! It is read through a `Vfs`, so a theme being edited and not yet saved
//! is the theme it reads. What it writes goes through `edit`:
//! `set_token` changes or declares one token, `reset_token` takes one
//! back to the baseline, and nothing else in the file moves.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use crate::config::project::ThemeConfig;
use crate::linter::contrast::{Pairing, pairings};
use crate::parser::ast::*;

/// One token.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Token {
    /// Without its `--`: `color-primary`, `surface-raised`.
    pub name: String,
    /// `color`, `spacing`, `font-size`, `line-height`, `radius`, `shadow`,
    /// `transition`, `font`, `motion`, `other` — or `declared` for one the
    /// baseline has not and the theme invents.
    pub group: &'static str,
    /// What the build gives it: the theme's value, or the baseline's.
    pub value: String,
    /// The engine's baseline value, when it has one.
    pub baseline: Option<String>,
    /// Whether the theme names it; an inherited one keeps the baseline, and
    /// editing it declares it.
    pub declared: bool,
    /// The line of the theme it is declared on.
    pub line: Option<u32>,
    /// The dark theme's value, when it declares one.
    pub dark: Option<String>,
    /// The properties whose group resolves it from a bare `$name`:
    /// `spacing-md` is drawn by `padding`, `margin`, `gap` …
    pub drawn_by: Vec<&'static str>,
    /// Every place the project names it.
    pub uses: Vec<Use>,
}

/// A place a token is named: `$md` on `padding`, `var(--color-primary)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Use {
    pub file: String,
    pub line: u32,
    /// The property it is the value of, or `""` in a stylesheet.
    pub property: String,
}

/// A theme, and its dark pair.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct View {
    /// The theme the build uses, when the project declares one.
    pub theme: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    /// The dark theme `theme.dark` names.
    pub dark: Option<String>,
    pub tokens: Vec<Token>,
    /// The pairings over the resolved tokens.
    pub pairs: Vec<Pairing>,
    /// The same with the dark theme over them.
    pub dark_pairs: Vec<Pairing>,
    /// Tokens one of the pair declares and the other does not.
    pub unpaired: Vec<String>,
}

/// The groups, in the order a theme panel lists them.
pub const GROUPS: &[&str] = &[
    "color",
    "spacing",
    "font-size",
    "line-height",
    "radius",
    "shadow",
    "transition",
    "font",
    "motion",
    "other",
    "declared",
];

/// The group a token belongs to, by the name it has in the baseline.
pub fn group_of(name: &str, in_baseline: bool) -> &'static str {
    if !in_baseline {
        return "declared";
    }
    for (prefix, group) in [
        ("color-", "color"),
        ("spacing-", "spacing"),
        ("font-size-", "font-size"),
        ("line-height-", "line-height"),
        ("radius-", "radius"),
        ("shadow-", "shadow"),
        ("transition-", "transition"),
        ("font-", "font"),
        ("ease-", "motion"),
        ("animation-", "motion"),
    ] {
        if name.starts_with(prefix) {
            return group;
        }
    }
    "other"
}

/// The properties a group's tokens are drawn by, as the compiler resolves
/// a bare `$name` (`style_tokens`): every curated property, and its sides.
pub fn drawn_by(group: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for (_, props) in crate::style_value::CURATED {
        for p in *props {
            if crate::codegen::style_tokens::token_group(p) == Some(group) && !out.contains(p) {
                out.push(p);
            }
        }
    }
    out
}

/// The theme `config` selects and its dark pair, from the program.
fn select<'a>(
    program: &'a Program,
    config: &ThemeConfig,
) -> (Option<(usize, &'a ThemeDecl)>, Option<&'a ThemeDecl>) {
    let dark_name = config.dark.as_deref().filter(|n| !n.is_empty());
    let themes: Vec<(usize, &ThemeDecl)> = program
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(i, d)| match d {
            Declaration::Theme(t) => Some((i, t)),
            _ => None,
        })
        .collect();
    let dark = dark_name.and_then(|n| themes.iter().find(|(_, t)| t.name == n).map(|(_, t)| *t));
    let wanted = config
        .name
        .as_deref()
        .filter(|n| !n.is_empty() && *n != "default");
    let own: Vec<&(usize, &ThemeDecl)> = themes
        .iter()
        .filter(|(_, t)| Some(t.name.as_str()) != dark_name)
        .collect();
    let main = match wanted {
        Some(n) => own.iter().find(|(_, t)| t.name == n).map(|x| **x),
        None if own.len() == 1 => Some(*own[0]),
        None => None,
    };
    (main, dark)
}

fn literal(e: &Expr) -> String {
    crate::print::style_value(e)
}

/// Every place each token is named in `program` (and in `stylesheets`, the
/// project's CSS by file), by full token name.
fn uses_of(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    declared: &dyn Fn(&str) -> bool,
    stylesheets: &[(String, String)],
) -> BTreeMap<String, Vec<Use>> {
    let mut out: BTreeMap<String, Vec<Use>> = BTreeMap::new();
    let var = regex::Regex::new(r"var\(--([A-Za-z0-9_-]+)").expect("a regex");
    let mut add = |name: String, u: Use| out.entry(name).or_default().push(u);
    fn props(stmts: &[Statement], f: &mut dyn FnMut(&StyleProperty)) {
        for s in stmts {
            if let StatementKind::UIElement(ui) = &s.kind {
                if let Some(b) = &ui.style_block {
                    for p in b
                        .properties
                        .iter()
                        .chain(b.media_queries.iter().flat_map(|m| m.properties.iter()))
                        .chain(b.pseudo_blocks.iter().flat_map(|q| q.properties.iter()))
                    {
                        f(p);
                    }
                }
                props(&ui.children, f);
                for fill in &ui.slot_fills {
                    props(&fill.body, f);
                }
            }
            for body in s.kind.bodies() {
                props(body, f);
            }
        }
    }
    for (i, d) in program.declarations.iter().enumerate() {
        let Some(body) = crate::print::body_of(d) else {
            continue;
        };
        let file = file_of(i);
        props(body, &mut |p| {
            let prop = crate::codegen::style_tokens::canonical_style_prop(&p.name);
            let mut tokens = Vec::new();
            match &p.value {
                Expr::Token(t) => tokens.push(t.clone()),
                Expr::InterpolatedString(parts) => {
                    for part in parts {
                        if let StringPart::Expression(Expr::Token(t)) = part {
                            tokens.push(t.clone());
                        }
                    }
                }
                _ => {}
            }
            for t in tokens {
                let full = crate::codegen::style_tokens::resolve_short_token(&prop, &t, declared)
                    .unwrap_or(t);
                add(
                    full,
                    Use {
                        file: file.clone(),
                        line: p.span.line,
                        property: prop.clone(),
                    },
                );
            }
            for c in var.captures_iter(&literal(&p.value)) {
                add(
                    c[1].to_string(),
                    Use {
                        file: file.clone(),
                        line: p.span.line,
                        property: prop.clone(),
                    },
                );
            }
        });
    }
    for (file, css) in stylesheets {
        for (n, line) in css.lines().enumerate() {
            for c in var.captures_iter(line) {
                add(
                    c[1].to_string(),
                    Use {
                        file: file.clone(),
                        line: n as u32 + 1,
                        property: String::new(),
                    },
                );
            }
        }
    }
    for v in out.values_mut() {
        v.sort();
        v.dedup();
    }
    out
}

/// The theme of `program` as `config` selects it: every token, the
/// pairings, and the dark pair. `stylesheets` are the project's own CSS
/// files, by name, for where a token is named outside a style block.
pub fn view(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    config: &ThemeConfig,
    stylesheets: &[(String, String)],
) -> View {
    let baseline = super::tokens::default_tokens();
    let (main, dark) = select(program, config);
    let theme_values: HashMap<String, (String, u32)> = main
        .map(|(_, t)| {
            t.tokens
                .iter()
                .map(|k| (k.name.clone(), (literal(&k.value), k.span.line)))
                .collect()
        })
        .unwrap_or_default();
    let dark_values: HashMap<String, String> = dark
        .map(|t| {
            t.tokens
                .iter()
                .map(|k| (k.name.clone(), literal(&k.value)))
                .collect()
        })
        .unwrap_or_default();
    let mut resolved = baseline.clone();
    for (k, (v, _)) in &theme_values {
        resolved.insert(k.clone(), v.clone());
    }
    for (k, v) in &config.tokens {
        resolved.insert(k.clone(), v.clone());
    }
    let declared = |n: &str| theme_values.contains_key(n) || dark_values.contains_key(n);
    let uses = uses_of(program, file_of, &declared, stylesheets);

    let mut names: Vec<&String> = resolved.keys().chain(dark_values.keys()).collect();
    names.sort();
    names.dedup();
    let mut tokens: Vec<Token> = names
        .into_iter()
        .map(|name| {
            let in_baseline = baseline.contains_key(name);
            let group = group_of(name, in_baseline);
            Token {
                name: name.clone(),
                group,
                value: resolved.get(name).cloned().unwrap_or_default(),
                baseline: baseline.get(name).cloned(),
                declared: theme_values.contains_key(name),
                line: theme_values.get(name).map(|(_, l)| *l),
                dark: dark_values.get(name).cloned(),
                drawn_by: if in_baseline {
                    drawn_by(group)
                } else {
                    Vec::new()
                },
                uses: uses.get(name).cloned().unwrap_or_default(),
            }
        })
        .collect();
    // By the compiler's groups, in the panel's order, then by name.
    tokens.sort_by_key(|t| {
        (
            GROUPS
                .iter()
                .position(|g| *g == t.group)
                .unwrap_or(GROUPS.len()),
            t.name.clone(),
        )
    });

    let pairs = pairings(program, &resolved);
    let mut dark_resolved = resolved.clone();
    for (k, v) in &dark_values {
        dark_resolved.insert(k.clone(), v.clone());
    }
    let dark_pairs = if dark.is_some() {
        pairings(program, &dark_resolved)
    } else {
        Vec::new()
    };
    let mut unpaired: Vec<String> = Vec::new();
    if dark.is_some() {
        for k in theme_values.keys() {
            if !dark_values.contains_key(k) && k.starts_with("color-") {
                unpaired.push(k.clone());
            }
        }
        for k in dark_values.keys() {
            if !theme_values.contains_key(k) {
                unpaired.push(k.clone());
            }
        }
        unpaired.sort();
    }
    View {
        theme: main.map(|(_, t)| t.name.clone()),
        file: main.map(|(i, _)| file_of(i)),
        line: main.map(|(_, t)| t.span.line),
        dark: dark.map(|t| t.name.clone()),
        tokens,
        pairs,
        dark_pairs,
        unpaired,
    }
}

/// [`view`] of the project at `dir`, read through `vfs`: an unsaved theme
/// is the theme it reads.
pub fn view_via(vfs: &dyn crate::vfs::Vfs, dir: &std::path::Path) -> crate::error::Result<View> {
    let (program, files, _) = crate::build::read_project_collecting_via(vfs, dir, None)?;
    let config = crate::config::ProjectConfig::load_via(vfs, dir)?;
    let mut sheets = Vec::new();
    let mut stack = vec![dir.join("src")];
    while let Some(d) = stack.pop() {
        let Ok(mut entries) = vfs.read_dir(&d) else {
            continue;
        };
        entries.sort();
        for p in entries {
            if vfs.is_dir(&p) {
                stack.push(p);
            } else if p.extension().is_some_and(|e| e == "css")
                && let Ok(text) = vfs.read_to_string(&p)
            {
                sheets.push((
                    p.strip_prefix(dir)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .to_string(),
                    text,
                ));
            }
        }
    }
    let file_of = |i: usize| files.get(i).cloned().unwrap_or_default();
    Ok(view(&program, &file_of, &config.theme, &sheets))
}

/// The token `name` of `theme` set to `value` (a style value, as written:
/// `#0F766E`, `14px`) — changed where it is declared, or declared after the
/// theme's last token — and nothing else in `source` moved.
pub fn set_token(
    source: &str,
    file: &str,
    theme: &str,
    name: &str,
    value: &str,
) -> crate::error::Result<String> {
    let probe = format!("theme Probe {{\n    {name}: {value}\n}}\n");
    let parsed = crate::syntax::parse_source(&probe, "probe.wf")?;
    let Some(Declaration::Theme(t)) = parsed.declarations.into_iter().next() else {
        return Err(crate::error::WebFluentError::EditError(format!(
            "`{name}: {value}` is not a token"
        )));
    };
    let Some(token) = t.tokens.into_iter().next() else {
        return Err(crate::error::WebFluentError::EditError(format!(
            "`{name}: {value}` is not a token"
        )));
    };
    crate::edit::rewrite_declaration(source, file, theme, |d| {
        if let Declaration::Theme(t) = d {
            match t.tokens.iter_mut().find(|k| k.name == name) {
                Some(k) => k.value = token.value,
                None => t.tokens.push(token),
            }
        }
    })
}

/// The token `name` taken out of `theme`, so it is the baseline's again.
pub fn reset_token(
    source: &str,
    file: &str,
    theme: &str,
    name: &str,
) -> crate::error::Result<String> {
    crate::edit::rewrite_declaration(source, file, theme, |d| {
        if let Declaration::Theme(t) = d {
            t.tokens.retain(|k| k.name != name);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "theme Brand {\n    color-primary: #0F766E\n    color-surface: #ffffff\n    color-text: #f5f5f5\n    surface-raised: #fafafa\n}\n\ntheme Night {\n    color-background: #0B1220\n    color-text: #e5e7eb\n}\n\npage Home(path: \"/\") {\n    Card {\n        style {\n            padding: $md\n            background: $surface-raised\n            color: var(--color-primary)\n        }\n        Button(\"Go\").primary\n    }\n}\n";

    fn config() -> ThemeConfig {
        serde_json::from_str(r#"{ "name": "Brand", "dark": "Night" }"#).unwrap()
    }

    fn the_view() -> View {
        let p = crate::syntax::parse_source(SRC, "t.wf").unwrap();
        view(
            &p,
            &|_| "src/theme.wf".into(),
            &config(),
            &[(
                "src/site.css".into(),
                ".x { border-color: var(--color-border); }\n".into(),
            )],
        )
    }

    #[test]
    fn every_token_says_what_it_is_and_where_it_is_used() {
        let v = the_view();
        assert_eq!(
            (v.theme.as_deref(), v.dark.as_deref(), v.line),
            (Some("Brand"), Some("Night"), Some(1))
        );
        let t = |n: &str| {
            v.tokens
                .iter()
                .find(|t| t.name == n)
                .unwrap_or_else(|| panic!("{n}"))
        };
        let primary = t("color-primary");
        assert!(primary.declared && primary.group == "color" && primary.line == Some(2));
        assert_eq!(primary.value, "#0F766E");
        assert_ne!(primary.baseline.as_deref(), Some("#0F766E"));
        assert!(primary.drawn_by.contains(&"color") && primary.drawn_by.contains(&"background"));
        assert_eq!(primary.uses.len(), 1, "var(--color-primary) on color");
        let md = t("spacing-md");
        assert!(!md.declared, "inherited");
        assert_eq!(md.value, md.baseline.clone().unwrap());
        assert!(md.drawn_by.contains(&"padding") && md.drawn_by.contains(&"gap"));
        assert_eq!(
            md.uses,
            [Use {
                file: "src/theme.wf".into(),
                line: 16,
                property: "padding".into()
            }]
        );
        let raised = t("surface-raised");
        assert_eq!((raised.group, raised.declared), ("declared", true));
        assert_eq!(raised.uses.len(), 1);
        assert_eq!(t("color-border").uses[0].file, "src/site.css");
        assert_eq!(t("color-text").dark.as_deref(), Some("#e5e7eb"));
    }

    #[test]
    fn the_pairings_are_the_ones_a13_is_drawn_from() {
        let v = the_view();
        let text_on_surface = v
            .pairs
            .iter()
            .find(|p| p.description == "text on a card or surface")
            .unwrap();
        assert!(!text_on_surface.passes(), "light text on white");
        let white_on_primary = v
            .pairs
            .iter()
            .find(|p| p.background == "color-primary" && p.foreground == "white")
            .unwrap();
        assert!(white_on_primary.drawn, "a `.primary` button is on the page");
        // What fails here, the build says, with A13.
        let p = crate::syntax::parse_source(SRC, "t.wf").unwrap();
        let resolved: HashMap<String, String> = {
            let mut r = super::super::tokens::default_tokens();
            for t in &v.tokens {
                r.insert(t.name.clone(), t.value.clone());
            }
            r
        };
        let warned = crate::linter::contrast::lint_contrast(&p, &resolved);
        let failing = v.pairs.iter().filter(|p| p.drawn && !p.passes()).count();
        assert!(warned.len() <= failing && !warned.is_empty());
        for w in &warned {
            assert!(
                v.pairs
                    .iter()
                    .any(|p| !p.passes() && w.message.starts_with(p.description)),
                "{}",
                w.message
            );
        }
        // The dark pair has its own, and the tokens one declares alone.
        assert!(!v.dark_pairs.is_empty());
        assert!(v.unpaired.contains(&"color-background".to_string()));
        assert!(
            v.unpaired.contains(&"color-primary".to_string()),
            "a light colour with no dark one"
        );
    }

    #[test]
    fn a_token_is_written_where_it_is_and_reset_to_the_baseline() {
        let out = set_token(SRC, "t.wf", "Brand", "color-primary", "#115E59").unwrap();
        assert_eq!(
            out,
            SRC.replace("color-primary: #0F766E", "color-primary: #115E59")
        );
        let out = set_token(SRC, "t.wf", "Brand", "radius-md", "14px").unwrap();
        assert_eq!(
            out,
            SRC.replace(
                "    surface-raised: #fafafa\n}",
                "    surface-raised: #fafafa\n    radius-md: 14px\n}"
            )
        );
        let out = reset_token(SRC, "t.wf", "Brand", "color-text").unwrap();
        assert_eq!(out, SRC.replacen("    color-text: #f5f5f5\n", "", 1));
        assert!(set_token(SRC, "t.wf", "Brand", "bad name!", "x").is_err());
        assert!(set_token(SRC, "t.wf", "Nope", "color-primary", "#000").is_err());
    }
}
