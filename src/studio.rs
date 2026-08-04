//! Studio integration facade (Milestone 2 of the engine upgrade).
//!
//! One in-process call turns a parsed program into everything the studio's
//! preview webview needs: per-route pre-rendered HTML (with `data-wf-node`
//! stamps), the CSS, the JS bundle, and the node-identity map. This is the
//! studio-mode counterpart of `wf build` — it returns Rust structs instead of
//! writing files, and always stamps node ids (studio mode) so a preview click
//! can resolve to code.

use crate::codegen::node_id::{NodeMap, build_node_map};
use crate::codegen::ssg::{SiteContext, render_page_html_studio};
use crate::codegen::{JsCodegen, generate_css_with};
use crate::config::ProjectConfig;
use crate::parser::ast::{ComponentDecl, Declaration, Program, Statement};
use std::collections::HashMap;

/// A pre-rendered page keyed by its route.
#[derive(Debug, Clone)]
pub struct CompiledPage {
    /// The page route, e.g. `/` or `/about`.
    pub route: String,
    /// SSG-rendered HTML (element roots carry `data-wf-node`).
    pub html: String,
}

/// Everything the studio preview needs from one compile.
#[derive(Debug, Clone, Default)]
pub struct CompiledSite {
    /// Pre-rendered static pages (the SSG paint), one per static route.
    pub pages: Vec<CompiledPage>,
    /// The stylesheet (`styles.css`).
    pub css: String,
    /// The JS bundle (`app.js`) — reactivity, routing, and `data-wf-node` stamps.
    pub js: String,
    /// `node_id ↔ span ↔ path` sidecar for click-to-code and structured edits.
    pub node_map: NodeMap,

    /// The design tokens this compile resolved, after the baseline, the
    /// project's `Theme` and any config overrides have been layered.
    ///
    /// The studio's inspector needs the values a swatch should actually show,
    /// which is not what any single one of those three sources says.
    pub tokens: HashMap<String, String>,

    /// The `Theme` declarations in the source, by name, with the span of each.
    ///
    /// Enough for the studio to list the themes a project has, jump to one, and
    /// know which it is previewing.
    pub themes: Vec<ThemeInfo>,

    /// Accessibility findings for this compile: the existing WCAG element checks
    /// plus contrast ratios over the resolved tokens.
    ///
    /// The studio can surface these against the node they belong to rather than
    /// waiting for the author to read build output they may never see.
    pub diagnostics: Vec<Diagnostic>,

    /// Why a theme failed to resolve, if it did.
    ///
    /// The preview falls back to the baseline rather than going blank, so
    /// without this the studio would show a working page and never mention that
    /// the author's theme is not the one on screen.
    pub theme_error: Option<String>,
}

/// A `Theme` declaration, as the studio needs to see it.
#[derive(Debug, Clone)]
pub struct ThemeInfo {
    pub name: String,
    /// Token names the theme sets, in source order.
    pub tokens: Vec<String>,
    /// Byte offset of the declaration, for click-to-code.
    pub offset: u32,
    pub line: u32,
}

/// One accessibility finding, flattened for the studio.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Rule identifier, e.g. `A11` or `A13`.
    pub rule: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub hint: String,
}

/// Compile a program for the studio preview: node ids stamped, SSG pages + CSS +
/// JS bundle, plus the node map.
///
/// Dynamic routes (paths containing a `:param`) are not pre-rendered — the SPA
/// runtime renders them after hydration — so they are omitted from `pages`.
pub fn compile_studio(
    program: &Program,
    config: &ProjectConfig,
    translations: &HashMap<String, HashMap<String, String>>,
) -> CompiledSite {
    let node_map = build_node_map(program);

    // A theme that fails to resolve must not take the preview down with it: the
    // studio shows a live page while the author is still typing, so fall back to
    // the baseline and let `wf build` be the one that refuses. The reason is
    // handed back rather than swallowed, so the studio can say why the page does
    // not look the way the source says it should.
    let (tokens, theme_error) = match crate::themes::resolve_tokens(program, &config.theme) {
        Ok(tokens) => (tokens, None),
        Err(e) => (crate::themes::tokens::default_tokens(), Some(e.to_string())),
    };
    let css = generate_css_with(&tokens, config.theme.builtin);

    let themes: Vec<ThemeInfo> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Theme(t) => Some(ThemeInfo {
                name: t.name.clone(),
                tokens: t.tokens.iter().map(|tok| tok.name.clone()).collect(),
                offset: t.span.start,
                line: t.span.line,
            }),
            _ => None,
        })
        .collect();

    let mut diagnostics: Vec<Diagnostic> = crate::linter::lint_accessibility(program)
        .into_iter()
        .chain(crate::linter::lint_contrast(program, &tokens))
        .map(|w| Diagnostic {
            rule: w.rule_id.clone(),
            message: w.message.clone(),
            file: w.file.clone(),
            line: w.line,
            hint: w.hint.clone(),
        })
        .collect();
    diagnostics.sort_by(|a, b| a.rule.cmp(&b.rule).then(a.line.cmp(&b.line)));

    let mut js_gen = JsCodegen::new();
    if let Some(i18n) = &config.i18n {
        js_gen.set_i18n(i18n.default_locale.clone(), translations.clone());
    }
    js_gen.set_ssg(true); // preview boots from the SSG paint, then hydrates
    if !config.build.base_path.is_empty() {
        js_gen.set_base_path(config.build.base_path.clone());
    }
    js_gen.set_studio(node_map.clone());
    let js = js_gen.generate(program);

    // The shared app shell (navbar/footer around the Router), if any.
    let app_body: Option<Vec<Statement>> = program.declarations.iter().find_map(|d| {
        if let Declaration::App(a) = d {
            Some(a.body.clone())
        } else {
            None
        }
    });

    // The component library, so the static paint EXPANDS calls to user components
    // instead of leaving `<!--wf-component-->` placeholders: a page built from
    // components used to paint empty until JS hydrated.
    let components: HashMap<String, ComponentDecl> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) => Some((c.name.clone(), c.clone())),
            _ => None,
        })
        .collect();

    let mut pages = Vec::new();
    for decl in &program.declarations {
        if let Declaration::Page(page) = decl {
            if page.path.contains(':') {
                continue; // dynamic route — not pre-rendered
            }
            let site = SiteContext {
                config,
                app_body: app_body.as_deref(),
                translations,
                components: &components,
                program,
            };
            let html = render_page_html_studio(page, &site, true, &node_map);
            pages.push(CompiledPage {
                route: page.path.clone(),
                html,
            });
        }
    }

    CompiledSite {
        pages,
        css,
        js,
        node_map,
        tokens,
        themes,
        diagnostics,
        theme_error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn program(src: &str) -> Program {
        let toks = Lexer::new(src, "<test>").tokenize().expect("lex");
        Parser::new(toks, "<test>").parse().expect("parse")
    }

    fn config() -> ProjectConfig {
        serde_json::from_str(r#"{"name":"test"}"#).expect("config")
    }

    #[test]
    fn compiles_a_page_to_a_site_with_node_map() {
        let src = "Page Home (path: \"/\") {\n\
                   \x20 Container {\n\
                   \x20   Heading(\"Hi\", h1)\n\
                   \x20   Button(\"Go\", primary)\n\
                   \x20 }\n\
                   }\n";
        let prog = program(src);
        let site = compile_studio(&prog, &config(), &HashMap::new());

        // One static page at "/".
        assert_eq!(site.pages.len(), 1);
        assert_eq!(site.pages[0].route, "/");

        // SSG HTML + JS both carry data-wf-node stamps; CSS is non-empty.
        assert!(site.pages[0].html.contains("data-wf-node="));
        assert!(site.js.contains("data-wf-node"));
        assert!(!site.css.is_empty());

        // The node map is populated and resolves ids to source spans.
        assert!(site.node_map.len() >= 3); // Container, Heading, Button
        assert!(site.node_map.info("Home:0").is_some());
    }

    #[test]
    fn skips_dynamic_routes_from_prerender() {
        let src = "Page Home (path: \"/\") { Text(\"home\") }\n\
                   Page User (path: \"/users/:id\") { Text(\"user\") }\n";
        let prog = program(src);
        let site = compile_studio(&prog, &config(), &HashMap::new());
        // Only the static "/" page is pre-rendered; the :id route is skipped.
        assert_eq!(site.pages.len(), 1);
        assert_eq!(site.pages[0].route, "/");
    }
}
