//! Which design tokens a build ships, and where they came from.
//!
//! The engine used to carry four palettes as Rust maps — `default`, `dark`,
//! `minimal`, `brutalist` — selected by name from `webfluent.app.json`. They were
//! not design systems; `dark` was five token overrides and `brutalist` fifteen.
//! Anyone doing real design ignored them and wrote `theme.tokens` in JSON, which
//! put the look of a site in a config file, away from the `style { }` blocks it
//! sits behind.
//!
//! A theme is now written in the language:
//!
//! ```wf
//! Theme Brand {
//!     token color-primary: "#0F766E"
//!     token radius-md: "14px"
//! }
//! ```
//!
//! Resolution layers three sources, each overriding the last:
//!
//! 1. the baseline token set, so every `var(--…)` the stylesheet references
//!    always resolves;
//! 2. the selected `Theme` declaration;
//! 3. `theme.tokens` from the config, which stays for machine-written overrides
//!    (a deploy pipeline injecting a brand colour, the studio's inspector).

use std::collections::HashMap;

use crate::config::project::ThemeConfig;
use crate::error::{Result, WebFluentError};
use crate::parser::ast::{Declaration, Expr, Program, ThemeDecl};

/// The palettes that used to ship with the engine, and where each went.
///
/// Naming one is a hard error rather than a silent fallback: a project that says
/// `"name": "brutalist"` and quietly renders as `default` is worse off than one
/// that is told what happened.
const REMOVED_PALETTES: &[(&str, &str)] = &[
    ("dark", "examples/themes/dark.wf"),
    ("minimal", "examples/themes/minimal.wf"),
    ("brutalist", "examples/themes/brutalist.wf"),
];

/// Every `Theme` the program declares.
pub fn declared_themes(program: &Program) -> Vec<&ThemeDecl> {
    program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Theme(t) => Some(t),
            _ => None,
        })
        .collect()
}

/// The tokens a build should emit, given its source and its config.
///
/// `config.name` selects among several declared themes. With exactly one
/// declaration it can be left unset — the obvious case should not need
/// ceremony. With none, the baseline is used.
pub fn resolve_tokens(program: &Program, config: &ThemeConfig) -> Result<HashMap<String, String>> {
    let mut tokens = super::tokens::default_tokens();

    if let Some(theme) = select_theme(program, config)? {
        for token in &theme.tokens {
            let Some(value) = literal_value(&token.value) else {
                return Err(WebFluentError::CodegenError(format!(
                    "Theme {}: token `{}` must be a literal value, since it becomes a CSS custom \
                     property at build time",
                    theme.name, token.name
                )));
            };
            tokens.insert(token.name.clone(), value);
        }
    }

    // Config overrides sit on top, for values a machine supplies.
    for (k, v) in &config.tokens {
        tokens.insert(k.clone(), v.clone());
    }

    Ok(tokens)
}

/// The `Theme` declaration this build uses, if any.
fn select_theme<'a>(program: &'a Program, config: &ThemeConfig) -> Result<Option<&'a ThemeDecl>> {
    let themes = declared_themes(program);
    let requested = config.name.as_deref().filter(|n| !n.is_empty());

    match requested {
        // `default` names the baseline token set, which needs no declaration.
        Some("default") | None if themes.is_empty() => Ok(None),

        Some(name) => {
            if let Some(found) = themes.iter().find(|t| t.name == name) {
                return Ok(Some(found));
            }
            if let Some((_, example)) = REMOVED_PALETTES.iter().find(|(p, _)| *p == name) {
                return Err(WebFluentError::ConfigError(format!(
                    "The built-in `{name}` theme was removed. Themes are now written in \
                     WebFluent: copy `{example}` into your `src/` and keep \
                     `\"theme\": {{ \"name\": \"{name}\" }}` pointing at it."
                )));
            }
            if name == "default" {
                // A declaration is present but the config still asks for the baseline.
                return Ok(None);
            }
            Err(WebFluentError::ConfigError(format!(
                "No `Theme {name}` is declared in this project. Declare one — \
                 `Theme {name} {{ token color-primary: \"#0F766E\" }}` — or remove \
                 `theme.name` to use the baseline tokens.{}",
                available(&themes)
            )))
        }

        None => match themes.len() {
            0 => Ok(None),
            1 => Ok(Some(themes[0])),
            _ => Err(WebFluentError::ConfigError(format!(
                "This project declares {} themes, so the build cannot tell which to use. \
                 Set `\"theme\": {{ \"name\": \"…\" }}` in webfluent.app.json.{}",
                themes.len(),
                available(&themes)
            ))),
        },
    }
}

fn available(themes: &[&ThemeDecl]) -> String {
    if themes.is_empty() {
        String::new()
    } else {
        let names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        format!(" Declared: {}.", names.join(", "))
    }
}

/// A token's value as it will appear in the stylesheet.
///
/// Tokens become `--name: value` at build time, so the value has to be knowable
/// then. Anything reactive is a mistake worth naming rather than emitting.
fn literal_value(expr: &Expr) -> Option<String> {
    match expr {
        Expr::StringLiteral(s) => Some(s.clone()),
        Expr::NumberLiteral(n) => Some(if *n == (*n as i64) as f64 {
            format!("{}", *n as i64)
        } else {
            format!("{}", n)
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse(src: &str) -> Program {
        let tokens = Lexer::new(src, "<t>").tokenize().expect("lex");
        Parser::new(tokens, "<t>").parse().expect("parse")
    }

    fn config(name: Option<&str>) -> ThemeConfig {
        ThemeConfig {
            name: name.map(str::to_string),
            tokens: HashMap::new(),
            builtin: Default::default(),
        }
    }

    const PAGE: &str = "Page P (path: \"/\") { Text(\"x\") }";

    #[test]
    fn a_project_with_no_theme_gets_the_baseline() {
        let tokens = resolve_tokens(&parse(PAGE), &config(None)).unwrap();
        assert_eq!(
            tokens.get("color-primary").map(String::as_str),
            Some("#3B82F6")
        );
        assert!(
            tokens.contains_key("spacing-md"),
            "the baseline must be complete"
        );
    }

    #[test]
    fn a_single_declared_theme_is_used_without_naming_it() {
        let program = parse(&format!(
            "Theme Brand {{ token color-primary: \"#0F766E\" }}\n{PAGE}"
        ));
        let tokens = resolve_tokens(&program, &config(None)).unwrap();
        assert_eq!(
            tokens.get("color-primary").map(String::as_str),
            Some("#0F766E")
        );
    }

    #[test]
    fn a_theme_overrides_only_what_it_names() {
        let program = parse(&format!(
            "Theme Brand {{ token color-primary: \"#0F766E\" }}\n{PAGE}"
        ));
        let tokens = resolve_tokens(&program, &config(None)).unwrap();
        assert_eq!(
            tokens.get("spacing-md").map(String::as_str),
            Some("1rem"),
            "an unnamed token must keep its baseline value"
        );
    }

    #[test]
    fn config_tokens_win_over_the_declaration() {
        let program = parse(&format!(
            "Theme Brand {{ token color-primary: \"#0F766E\" }}\n{PAGE}"
        ));
        let mut cfg = config(None);
        cfg.tokens.insert("color-primary".into(), "#FF0000".into());
        let tokens = resolve_tokens(&program, &cfg).unwrap();
        assert_eq!(
            tokens.get("color-primary").map(String::as_str),
            Some("#FF0000")
        );
    }

    #[test]
    fn several_themes_are_selected_by_name() {
        let program = parse(&format!(
            "Theme Light {{ token color-text: \"#111111\" }}\n\
             Theme Dark {{ token color-text: \"#F1F5F9\" }}\n{PAGE}"
        ));
        let tokens = resolve_tokens(&program, &config(Some("Dark"))).unwrap();
        assert_eq!(
            tokens.get("color-text").map(String::as_str),
            Some("#F1F5F9")
        );
    }

    #[test]
    fn several_themes_with_no_selection_is_an_error_not_a_guess() {
        let program = parse(&format!(
            "Theme Light {{ token color-text: \"#111\" }}\n\
             Theme Dark {{ token color-text: \"#EEE\" }}\n{PAGE}"
        ));
        let err = resolve_tokens(&program, &config(None))
            .unwrap_err()
            .to_string();
        assert!(err.contains("declares 2 themes"), "{err}");
        assert!(
            err.contains("Light") && err.contains("Dark"),
            "the error should list them: {err}"
        );
    }

    #[test]
    fn a_removed_palette_name_says_where_it_went() {
        for (palette, example) in REMOVED_PALETTES {
            let err = resolve_tokens(&parse(PAGE), &config(Some(palette)))
                .unwrap_err()
                .to_string();
            assert!(err.contains(palette), "{err}");
            assert!(
                err.contains(example),
                "the error must name the replacement: {err}"
            );
        }
    }

    #[test]
    fn naming_a_theme_that_is_not_declared_is_an_error() {
        let err = resolve_tokens(&parse(PAGE), &config(Some("Nonexistent")))
            .unwrap_err()
            .to_string();
        assert!(err.contains("No `Theme Nonexistent`"), "{err}");
    }

    #[test]
    fn default_still_names_the_baseline() {
        let tokens = resolve_tokens(&parse(PAGE), &config(Some("default"))).unwrap();
        assert_eq!(
            tokens.get("color-primary").map(String::as_str),
            Some("#3B82F6")
        );
    }

    #[test]
    fn a_non_literal_token_value_is_rejected_with_its_name() {
        let program = parse(&format!(
            "Theme Brand {{ token color-primary: someState }}\n{PAGE}"
        ));
        let err = resolve_tokens(&program, &config(None))
            .unwrap_err()
            .to_string();
        assert!(err.contains("color-primary"), "{err}");
        assert!(err.contains("literal"), "{err}");
    }
}
