use crate::config::ProjectConfig;
use crate::parser::ast::{Declaration, Program};

/// Generate the HTML entry point (`index.html`) for a WebFluent project.
pub fn generate_html(config: &ProjectConfig, program: &Program) -> String {
    let title = if config.meta.title.is_empty() {
        &config.name
    } else {
        &config.meta.title
    };

    let lang = if config.meta.lang.is_empty() {
        "en"
    } else {
        &config.meta.lang
    };

    // A single-page app serves every route from this one document, so the tags
    // here describe the entry route. A crawler that runs no JavaScript sees only
    // this — which is the argument for building such a site with SSG on.
    let description_meta = program
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Page(p) if p.path == "/" => Some(p),
            _ => None,
        })
        .or_else(|| {
            program.declarations.iter().find_map(|d| match d {
                Declaration::Page(p) => Some(p),
                _ => None,
            })
        })
        .map(|page| crate::codegen::seo::head_tags(page, config, program))
        .unwrap_or_default();

    let head_links = head_links(config, ".");

    let favicon_link = if config.meta.favicon.is_empty() {
        String::new()
    } else {
        format!(r#"    <link rel="icon" href="{}">"#, config.meta.favicon)
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
{}{}{}{}    <link rel="stylesheet" href="styles.css">
    <script src="app.js" defer></script>
</head>
<body>
{}    <div id="app"><main id="wf-main"></main></div>
</body>
</html>"#,
        lang,
        title,
        description_meta,
        if favicon_link.is_empty() {
            String::new()
        } else {
            format!("{}\n", favicon_link)
        },
        csp_meta(config),
        head_links,
        SKIP_LINK,
    )
}

/// The `<link>` tags for the fonts and stylesheets the config declares, each
/// on its own line, ahead of `styles.css` so the engine's sheet wins ties.
///
/// A font stylesheet gets a `preconnect` to each origin it will pull from —
/// the stylesheet's own, and for Google Fonts the file origin as well — which
/// is the difference between the font arriving with the page and a second
/// round trip after it. `base` is the relative prefix a site-relative path
/// needs from this page's depth (`"."` at the root, `".."` one level down).
pub fn head_links(config: &ProjectConfig, base: &str) -> String {
    use crate::config::project::url_origin;
    let mut out = String::new();
    let mut preconnected: Vec<String> = Vec::new();
    let mut preconnect = |origin: String, out: &mut String| {
        if !preconnected.contains(&origin) {
            out.push_str(&format!(
                "    <link rel=\"preconnect\" href=\"{}\" crossorigin>\n",
                origin
            ));
            preconnected.push(origin);
        }
    };
    for url in &config.meta.fonts {
        if let Some(origin) = url_origin(url) {
            preconnect(origin.clone(), &mut out);
            if origin == "https://fonts.googleapis.com" {
                preconnect("https://fonts.gstatic.com".to_string(), &mut out);
            }
        }
        out.push_str(&format!(
            "    <link rel=\"stylesheet\" href=\"{}\">\n",
            href_from(url, base)
        ));
    }
    for url in &config.meta.stylesheets {
        out.push_str(&format!(
            "    <link rel=\"stylesheet\" href=\"{}\">\n",
            href_from(url, base)
        ));
    }
    out
}

/// A URL as written, or a site-relative path resolved from `base`.
fn href_from(url: &str, base: &str) -> String {
    if url.contains("://") {
        url.to_string()
    } else {
        format!("{}/{}", base, url.trim_start_matches('/'))
    }
}

/// A skip link, first in the tab order and visible only when focused.
///
/// Without one, a keyboard or screen-reader user walks the whole navigation on
/// every page before reaching the content they came for.
pub const SKIP_LINK: &str = "    <a class=\"wf-skip-link\" href=\"#wf-main\">Skip to content</a>\n";

/// The `Content-Security-Policy` meta tag, when the project asks for one.
///
/// The generated output is already the shape a strict policy wants: script and
/// style are external files, handlers bind through `addEventListener`, and
/// nothing is injected as HTML. That makes `'self'` achievable without any
/// `'unsafe-inline'` escape hatch — but a policy nobody opted into would break
/// the first third-party embed someone adds, so it is off by default.
pub fn csp_meta(config: &ProjectConfig) -> String {
    if !config.build.csp {
        return String::new();
    }
    format!(
        "    <meta http-equiv=\"Content-Security-Policy\" content=\"{}\">\n",
        crate::config::project::csp_policy(&config.meta)
    )
}

#[cfg(test)]
mod head_link_tests {
    use super::*;
    use crate::config::project::MetaConfig;

    fn config(fonts: &[&str], sheets: &[&str]) -> ProjectConfig {
        let mut config: ProjectConfig = serde_json::from_str(r#"{"name":"t"}"#).unwrap();
        config.meta = MetaConfig {
            fonts: fonts.iter().map(|s| s.to_string()).collect(),
            stylesheets: sheets.iter().map(|s| s.to_string()).collect(),
            ..MetaConfig::default()
        };
        config
    }

    #[test]
    fn a_google_font_is_preconnected_to_both_origins_then_linked() {
        let url = "https://fonts.googleapis.com/css2?family=Manrope:wght@400..800&display=swap";
        let links = head_links(&config(&[url], &[]), ".");
        let lines: Vec<&str> = links.lines().map(str::trim).collect();
        assert_eq!(
            lines,
            [
                r#"<link rel="preconnect" href="https://fonts.googleapis.com" crossorigin>"#,
                r#"<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>"#,
                &format!(r#"<link rel="stylesheet" href="{url}">"#),
            ]
        );
    }

    #[test]
    fn a_site_relative_stylesheet_is_resolved_from_the_pages_depth() {
        let links = head_links(&config(&[], &["/base.css"]), "../..");
        assert_eq!(
            links.trim(),
            r#"<link rel="stylesheet" href="../../base.css">"#
        );
    }

    #[test]
    fn the_links_come_before_the_engine_stylesheet() {
        let program = Program {
            declarations: Vec::new(),
        };
        let html = generate_html(&config(&[], &["/base.css"]), &program);
        let base = html.find("base.css").expect("base.css linked");
        let styles = html.find("styles.css").expect("styles.css linked");
        assert!(base < styles, "{html}");
    }

    #[test]
    fn nothing_declared_adds_nothing() {
        assert_eq!(head_links(&config(&[], &[]), "."), "");
    }
}
