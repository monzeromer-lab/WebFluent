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

    format!(
        r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
{}{}{}    <link rel="stylesheet" href="{root}/styles.css">
{}{}    <script src="{root}/app.js" defer></script>
{}</head>
<body>
{}    <div id="app"><main id="wf-main"></main></div>
</body>
</html>"#,
        lang,
        title,
        description_meta,
        csp_meta(config),
        head_links,
        entry_sheet,
        script_tags(config, &root),
        entry_chunk,
        SKIP_LINK,
    )
}

/// The `<script>` tags every page carries ahead of `app.js`, each `defer`
/// so the document parses first and the order holds: the plain libraries
/// `meta.scripts` names, the loader for its modules, and the project's own
/// scripts in path order — so a library is there for the scripts that use
/// it, and a name a script declares is there when the page's code runs.
/// A library marked `async` keeps no place in that order: it runs when it
/// arrives, and nothing waits for it.
pub fn script_tags(config: &ProjectConfig, root: &str) -> String {
    let mut out = String::new();
    let mut linked: Vec<&str> = Vec::new();
    for entry in config.meta.scripts.iter().filter(|s| !s.is_module()) {
        // One library listed twice — once plain, once for its globals — is
        // loaded once.
        if linked.contains(&entry.src()) {
            continue;
        }
        linked.push(entry.src());
        out.push_str(&format!(
            "    <script src=\"{}\" {}{}></script>\n",
            href_from(entry.src(), root),
            if entry.is_async() { "async" } else { "defer" },
            integrity_attrs(config, entry.src())
        ));
    }
    let modules = crate::codegen::js::external_modules(config);
    for (url, integrity) in &modules {
        if !url.contains("://") {
            continue;
        }
        // An ESM `import` takes no `integrity` attribute; a preload is
        // where the platform lets a module carry one.
        let hash = integrity
            .as_ref()
            .map(|h| format!(" integrity=\"{h}\" crossorigin=\"anonymous\""))
            .unwrap_or_default();
        out.push_str(&format!(
            "    <link rel=\"modulepreload\" href=\"{url}\"{hash}>\n"
        ));
    }
    if crate::codegen::js::externals_module(config).is_some() {
        out.push_str(&format!(
            "    <script type=\"module\" src=\"{root}/externals.js\"></script>\n"
        ));
    }
    for href in &config.build.scripts {
        out.push_str(&format!(
            "    <script src=\"{root}/{href}\" defer></script>\n"
        ));
    }
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
/// What a `meta.preload` file is, to the browser: its `as`, and the type
/// that lets a browser skip a format it cannot use.
pub struct PreloadKind {
    pub r#as: &'static str,
    pub mime: Option<&'static str>,
}

/// The kind of a file `meta.preload` names, by its extension — or `None`
/// for one no preload can say (`E111`).
pub fn preload_kind(path: &str) -> Option<PreloadKind> {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let ext = path.rsplit_once('.')?.1.to_ascii_lowercase();
    let (r#as, mime) = match ext.as_str() {
        "woff2" => ("font", Some("font/woff2")),
        "woff" => ("font", Some("font/woff")),
        "ttf" => ("font", Some("font/ttf")),
        "otf" => ("font", Some("font/otf")),
        "css" => ("style", None),
        "js" | "mjs" => ("script", None),
        "png" => ("image", Some("image/png")),
        "jpg" | "jpeg" => ("image", Some("image/jpeg")),
        "webp" => ("image", Some("image/webp")),
        "avif" => ("image", Some("image/avif")),
        "gif" => ("image", Some("image/gif")),
        "svg" => ("image", Some("image/svg+xml")),
        _ => return None,
    };
    Some(PreloadKind { r#as, mime })
}

pub fn head_links(config: &ProjectConfig, base: &str) -> String {
    use crate::config::project::url_origin;
    let mut out = String::new();
    // The icons, from the site's root under its `base_path` — `/favicon.svg`
    // is the base's, not the host's — and not from this page's depth: a
    // browser resolves an icon again when a page changes its address
    // without loading (the router), and `../favicon.svg` written for `/docs`
    // is `/docs/favicon.svg` once the page has moved on to `/docs/guide/x`.
    let icon_href = |url: &str| {
        if url.contains("://") || !url.starts_with('/') {
            href_from(url, base)
        } else {
            format!("{}{}", config.build.base_path.trim_end_matches('/'), url)
        }
    };
    if !config.meta.favicon.is_empty() {
        let icon = &config.meta.favicon;
        let kind = match icon.rsplit('.').next().map(|e| e.to_ascii_lowercase()) {
            Some(e) if e == "svg" => " type=\"image/svg+xml\"",
            Some(e) if e == "png" => " type=\"image/png\"",
            Some(e) if e == "ico" => " type=\"image/x-icon\"",
            _ => "",
        };
        out.push_str(&format!(
            "    <link rel=\"icon\" href=\"{}\"{kind}>\n",
            icon_href(icon)
        ));
    }
    if !config.meta.touch_icon.is_empty() {
        out.push_str(&format!(
            "    <link rel=\"apple-touch-icon\" href=\"{}\">\n",
            icon_href(&config.meta.touch_icon)
        ));
    }
    // Files the page will want, asked for now rather than when whatever
    // names them is read: a font is only found once the stylesheet that
    // declares it has arrived. A font preload must be `crossorigin`, or the
    // browser fetches it a second time for the stylesheet.
    for path in &config.meta.preload {
        if let Some(kind) = preload_kind(path) {
            out.push_str(&format!(
                "    <link rel=\"preload\" href=\"{}\" as=\"{}\"{}{}>\n",
                icon_href(path),
                kind.r#as,
                kind.mime
                    .map(|m| format!(" type=\"{m}\""))
                    .unwrap_or_default(),
                if kind.r#as == "font" {
                    " crossorigin"
                } else {
                    ""
                }
            ));
        }
    }
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
    fn an_async_script_is_async_and_the_rest_keep_their_order() {
        let config: ProjectConfig = serde_json::from_str(
            r#"{"name":"t","meta":{"scripts":[
                {"src":"https://tags.example/t.js","async":true},
                "https://cdn.example/lib.js"]}}"#,
        )
        .unwrap();
        let tags = script_tags(&config, ".");
        assert!(
            tags.contains(r#"<script src="https://tags.example/t.js" async></script>"#),
            "{tags}"
        );
        assert!(
            tags.contains(r#"<script src="https://cdn.example/lib.js" defer></script>"#),
            "{tags}"
        );
    }

    #[test]
    fn a_preload_is_asked_for_ahead_of_the_stylesheets() {
        let mut cfg = config(&[], &["/base.css"]);
        cfg.meta.preload = vec!["/fonts/inter.woff2".to_string(), "/hero.webp".to_string()];
        let links = head_links(&cfg, "../..");
        assert!(
            links.contains(r#"<link rel="preload" href="/fonts/inter.woff2" as="font" type="font/woff2" crossorigin>"#),
            "{links}"
        );
        assert!(
            links
                .contains(r#"<link rel="preload" href="/hero.webp" as="image" type="image/webp">"#),
            "{links}"
        );
        assert!(
            links.find("rel=\"preload\"") < links.find("rel=\"stylesheet\""),
            "a preload comes before the sheets: {links}"
        );
    }

    #[test]
    fn a_preload_is_under_the_base_path() {
        let mut cfg = config(&[], &[]);
        cfg.build.base_path = "/site".to_string();
        cfg.meta.preload = vec!["/fonts/inter.woff2".to_string()];
        assert!(head_links(&cfg, "..").contains(r#"href="/site/fonts/inter.woff2""#));
    }

    #[test]
    fn a_preload_kind_comes_from_the_extension() {
        assert_eq!(preload_kind("/a.woff2?v=2").unwrap().r#as, "font");
        assert_eq!(preload_kind("/a.css").unwrap().r#as, "style");
        assert_eq!(preload_kind("/a.mjs").unwrap().r#as, "script");
        assert_eq!(preload_kind("/a.JPG").unwrap().mime, Some("image/jpeg"));
        assert!(preload_kind("/a.txt").is_none());
        assert!(preload_kind("/fonts").is_none());
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

    /// A pre-rendered page never linked `meta.favicon`, and the single-page
    /// shell linked it as written, so under a `base_path` `/favicon.svg`
    /// was the host's, not the site's. Linked from the page's depth, it
    /// broke as soon as the router moved the page to another depth.
    #[test]
    fn the_icons_are_linked_from_the_site_root_under_the_base() {
        let mut config = config(&[], &[]);
        config.meta.favicon = "/favicon.svg".into();
        config.meta.touch_icon = "/apple-touch-icon.png".into();
        let lines = |config: &ProjectConfig| -> Vec<String> {
            head_links(config, "../..")
                .lines()
                .map(|l| l.trim().to_string())
                .collect()
        };
        assert_eq!(
            lines(&config),
            [
                r#"<link rel="icon" href="/favicon.svg" type="image/svg+xml">"#,
                r#"<link rel="apple-touch-icon" href="/apple-touch-icon.png">"#,
            ]
        );
        config.build.base_path = "/WebFluent".into();
        assert_eq!(
            lines(&config)[0],
            r#"<link rel="icon" href="/WebFluent/favicon.svg" type="image/svg+xml">"#
        );
        // A relative one is the page's to resolve, an absolute one is kept.
        config.meta.favicon = "https://cdn.example.com/i.png".into();
        assert!(lines(&config)[0].contains(r#"href="https://cdn.example.com/i.png""#));
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
