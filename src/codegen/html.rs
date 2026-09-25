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

    // The shell is served for every route, at any depth, so its assets are
    // addressed from the site root — `/deploys/8f2c` used to fetch
    // `/deploys/app.js` and get the shell again.
    let root = config.build.base_path.trim_end_matches('/').to_string();
    let head_links = head_links(config, &root);

    // The shell serves every route, so it cannot know which page chunk the
    // reader wants; the one for "/" is the usual first visit and is linked
    // beside app.js so that visit needs no second round trip. Any other
    // route's chunk is fetched by the router when it shows.
    let entry = if config.build.split {
        program.declarations.iter().find_map(|d| match d {
            Declaration::Page(p) if p.path == "/" => Some(p.name.as_str()),
            _ => None,
        })
    } else {
        None
    };
    let entry_chunk = entry
        .map(|name| {
            format!(
                "    <script src=\"{root}/pages/{}.js\" data-wf-page=\"{}\" defer></script>\n",
                name, name
            )
        })
        .unwrap_or_default();
    // And that page's own sheet, when it has one, so the first paint has
    // every rule it needs.
    let entry_sheet = entry
        .filter(|name| {
            crate::codegen::scoped_css::split_rules(program)
                .pages
                .contains_key(*name)
        })
        .map(|name| {
            format!(
                "    <link rel=\"stylesheet\" href=\"{root}/pages/{}.css\" data-wf-page-css=\"{}\">\n",
                name, name
            )
        })
        .unwrap_or_default();

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
{}{}{}{}    <link rel="stylesheet" href="{root}/styles.css">
{}{}    <script src="{root}/app.js" defer></script>
{}</head>
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
        entry_sheet,
        externals_tags(config, program, &root),
        entry_chunk,
        SKIP_LINK,
    )
}

/// The tags a program's `external` modules need: the module that imports
/// them, and a `modulepreload` carrying the hash for each one that
/// declared it.
///
/// A module script and a deferred classic script run in the order they
/// appear, so this has finished before `app.js` reads any of it. The
/// preload is also where subresource integrity can be applied: an ESM
/// `import` takes no `integrity` attribute, and a `<link rel=modulepreload
/// integrity>` is the way the platform gives you one.
pub fn externals_tags(config: &ProjectConfig, program: &Program, root: &str) -> String {
    let modules = crate::codegen::js::external_modules(program);
    if modules.is_empty() {
        return String::new();
    }
    let _ = config;
    let mut out = String::new();
    for (url, integrity) in &modules {
        if !url.contains("://") {
            continue;
        }
        let hash = integrity
            .as_ref()
            .map(|h| format!(" integrity=\"{h}\" crossorigin=\"anonymous\""))
            .unwrap_or_default();
        out.push_str(&format!(
            "    <link rel=\"modulepreload\" href=\"{url}\"{hash}>\n"
        ));
    }
    out.push_str(&format!(
        "    <script type=\"module\" src=\"{root}/externals.js\"></script>\n"
    ));
    out
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
            "    <link rel=\"stylesheet\" href=\"{}\"{}>\n",
            href_from(url, base),
            integrity_attrs(config, url)
        ));
    }
    for url in &config.meta.stylesheets {
        out.push_str(&format!(
            "    <link rel=\"stylesheet\" href=\"{}\"{}>\n",
            href_from(url, base),
            integrity_attrs(config, url)
        ));
    }
    out
}

/// The `integrity` and `crossorigin` attributes for a declared asset, when
/// the config names its hash. A file served from this site needs neither.
fn integrity_attrs(config: &ProjectConfig, url: &str) -> String {
    match config.meta.integrity.get(url) {
        Some(hash) if url.contains("://") => {
            format!(" integrity=\"{hash}\" crossorigin=\"anonymous\"")
        }
        _ => String::new(),
    }
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
/// `'unsafe-inline'` escape hatch. `wf init` turns it on; an existing project
/// opts in, because a policy nobody asked for would break the first
/// third-party embed someone adds.
///
/// `frame-ancestors` is not here: a browser ignores it in a meta tag. It is
/// in `_headers`, where it is honoured.
pub fn csp_meta(config: &ProjectConfig) -> String {
    if !config.build.csp {
        return String::new();
    }
    format!(
        "    <meta http-equiv=\"Content-Security-Policy\" content=\"{}\">\n",
        crate::config::project::csp_meta_policy(config)
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
    fn the_shell_addresses_its_assets_from_the_site_root() {
        let program = Program {
            declarations: Vec::new(),
        };
        let html = generate_html(&config(&[], &["/base.css"]), &program);
        assert!(
            html.contains("href=\"/styles.css\"") && html.contains("src=\"/app.js\""),
            "{html}"
        );
        assert!(html.contains("href=\"/base.css\""), "{html}");
        let mut cfg = config(&[], &[]);
        cfg.build.base_path = "/site/".to_string();
        let html = generate_html(&cfg, &program);
        assert!(
            html.contains("href=\"/site/styles.css\"") && html.contains("src=\"/site/app.js\""),
            "{html}"
        );
    }

    #[test]
    fn nothing_declared_adds_nothing() {
        assert_eq!(head_links(&config(&[], &[]), "."), "");
    }
}
