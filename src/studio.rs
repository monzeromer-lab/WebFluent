//! Studio integration facade (Milestone 2 of the engine upgrade).
//!
//! One in-process call turns a parsed program into everything the studio's
//! preview webview needs: per-route pre-rendered HTML (with `data-wf-node`
//! stamps), the CSS, the JS bundle, and the node-identity map. This is the
//! studio-mode counterpart of `wf build` — it returns Rust structs instead of
//! writing files, and always stamps node ids (studio mode) so a preview click
//! can resolve to code.

use std::collections::HashMap;
use crate::parser::ast::{ComponentDecl, Declaration, Program, Statement};
use crate::config::ProjectConfig;
use crate::codegen::{generate_css_with, JsCodegen};
use crate::codegen::node_id::{build_node_map, NodeMap};
use crate::codegen::ssg::render_page_html_studio;

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

    let css = generate_css_with(
        &config.theme.name,
        &config.theme.tokens,
        config.theme.builtin,
    );

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
        if let Declaration::App(a) = d { Some(a.body.clone()) } else { None }
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
            let html = render_page_html_studio(
                page,
                config,
                app_body.as_deref(),
                translations,
                true, // studio mode: stamp data-wf-node
                &node_map,
                &components,
            );
            pages.push(CompiledPage { route: page.path.clone(), html });
        }
    }

    CompiledSite { pages, css, js, node_map }
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
