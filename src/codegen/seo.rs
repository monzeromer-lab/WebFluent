//! The `<head>` tags a search engine and a link preview read, and the two files
//! a crawler looks for.
//!
//! A WebFluent page used to ship a `<title>` and, if the project set one, a
//! description. Everything else a search result is assembled from — the canonical
//! URL, the sharing card, the language alternates, the machine-readable
//! description of what the page *is* — had no way to be expressed at all.
//!
//! Everything here is derived from what the source already says. A page has a
//! title, a path and a body; a project has a name, a locale set and a URL. The
//! engine knows the route table, so it can write a sitemap without being told
//! one. The only thing an author must supply is `meta.site_url`, because an
//! absolute URL cannot be inferred — and where it is missing, the tags that
//! require it are omitted rather than guessed at, since Google's guidance is
//! explicit that a relative canonical causes problems later.

use crate::config::ProjectConfig;
use crate::parser::ast::{Declaration, PageDecl, Program};

/// Escape text for an HTML attribute value.
fn attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Escape text for a JSON string.
fn json_str(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            // The JSON sits inside a `<script>`: an unescaped `<` lets a title
            // containing `</script>` close the element and turn the rest of the
            // document into markup. Escaping it keeps the JSON valid and inert.
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            c => out.push(c),
        }
    }
    out
}

/// The site origin with any trailing slash removed, or `None` if unset.
pub fn site_origin(config: &ProjectConfig) -> Option<&str> {
    let url = config.meta.site_url.trim_end_matches('/');
    if url.is_empty() { None } else { Some(url) }
}

/// The absolute URL of a route.
pub fn absolute_url(config: &ProjectConfig, path: &str) -> Option<String> {
    let origin = site_origin(config)?;
    let base = config.build.base_path.trim_end_matches('/');
    let route = path.trim_start_matches('/');
    Some(if route.is_empty() {
        format!("{origin}{base}/")
    } else {
        format!("{origin}{base}/{route}")
    })
}

/// Resolve a possibly-relative asset reference to an absolute URL.
fn absolute_asset(config: &ProjectConfig, reference: &str) -> Option<String> {
    if reference.starts_with("http://") || reference.starts_with("https://") {
        return Some(reference.to_string());
    }
    let origin = site_origin(config)?;
    let base = config.build.base_path.trim_end_matches('/');
    Some(format!(
        "{origin}{base}/{}",
        reference.trim_start_matches('/')
    ))
}

/// The page's own description, falling back to the project's.
fn description<'a>(page: &'a PageDecl, config: &'a ProjectConfig) -> Option<&'a str> {
    page.description
        .as_deref()
        .or(Some(config.meta.description.as_str()))
        .filter(|d| !d.is_empty())
}

fn title(page: &PageDecl, config: &ProjectConfig) -> String {
    page.title
        .clone()
        .or_else(|| Some(config.meta.title.clone()).filter(|t| !t.is_empty()))
        .unwrap_or_else(|| config.name.clone())
}

fn site_name(config: &ProjectConfig) -> String {
    if config.meta.site_name.is_empty() {
        config.name.clone()
    } else {
        config.meta.site_name.clone()
    }
}

/// Every `<head>` tag for a page beyond charset, viewport and the stylesheet.
///
/// Returns pre-indented lines ready to splice into the document shell.
pub fn head_tags(page: &PageDecl, config: &ProjectConfig, program: &Program) -> String {
    let mut out = String::new();
    let mut push = |line: String| {
        out.push_str("    ");
        out.push_str(&line);
        out.push('\n');
    };

    let page_title = title(page, config);
    let desc = description(page, config).map(str::to_string);
    let canonical = absolute_url(config, &page.path);

    if let Some(d) = &desc {
        push(format!(
            r#"<meta name="description" content="{}">"#,
            attr(d)
        ));
    }

    // A page kept out of search says so; one that is indexable says nothing,
    // because indexing is the default and a `robots: index` tag is noise.
    if page.noindex {
        push(r#"<meta name="robots" content="noindex, follow">"#.to_string());
    }

    // Self-referencing and absolute. Google's guidance calls a self-canonical on
    // every indexable page the thing that stops duplicate clustering.
    if let Some(url) = &canonical {
        if !page.noindex {
            push(format!(r#"<link rel="canonical" href="{}">"#, attr(url)));
        }
    }

    // Language alternates. Every variant must list itself as well as the others,
    // or the set is ignored.
    if let (Some(i18n), Some(origin)) = (&config.i18n, site_origin(config)) {
        if i18n.locales.len() > 1 {
            let base = config.build.base_path.trim_end_matches('/');
            let route = page.path.trim_start_matches('/');
            for locale in &i18n.locales {
                let href = if route.is_empty() {
                    format!("{origin}{base}/?lang={locale}")
                } else {
                    format!("{origin}{base}/{route}?lang={locale}")
                };
                push(format!(
                    r#"<link rel="alternate" hreflang="{}" href="{}">"#,
                    attr(locale),
                    attr(&href)
                ));
            }
            if let Some(url) = &canonical {
                push(format!(
                    r#"<link rel="alternate" hreflang="x-default" href="{}">"#,
                    attr(url)
                ));
            }
        }
    }

    // ── Sharing card ────────────────────────────────────
    let og_type = page.page_type.as_deref().unwrap_or("website");
    push(format!(
        r#"<meta property="og:type" content="{}">"#,
        attr(og_type)
    ));
    push(format!(
        r#"<meta property="og:title" content="{}">"#,
        attr(&page_title)
    ));
    push(format!(
        r#"<meta property="og:site_name" content="{}">"#,
        attr(&site_name(config))
    ));
    if let Some(d) = &desc {
        push(format!(
            r#"<meta property="og:description" content="{}">"#,
            attr(d)
        ));
    }
    if let Some(url) = &canonical {
        push(format!(
            r#"<meta property="og:url" content="{}">"#,
            attr(url)
        ));
    }
    if !config.meta.lang.is_empty() {
        push(format!(
            r#"<meta property="og:locale" content="{}">"#,
            attr(&config.meta.lang)
        ));
    }

    let image = page
        .image
        .as_deref()
        .filter(|i| !i.is_empty())
        .or(Some(config.meta.image.as_str()))
        .filter(|i| !i.is_empty())
        .and_then(|i| absolute_asset(config, i));
    if let Some(img) = &image {
        push(format!(
            r#"<meta property="og:image" content="{}">"#,
            attr(img)
        ));
    }

    // A card with an image is worth showing large; one without would render as a
    // blank panel, so it stays a summary.
    push(format!(
        r#"<meta name="twitter:card" content="{}">"#,
        if image.is_some() {
            "summary_large_image"
        } else {
            "summary"
        }
    ));
    push(format!(
        r#"<meta name="twitter:title" content="{}">"#,
        attr(&page_title)
    ));
    if let Some(d) = &desc {
        push(format!(
            r#"<meta name="twitter:description" content="{}">"#,
            attr(d)
        ));
    }
    if let Some(img) = &image {
        push(format!(
            r#"<meta name="twitter:image" content="{}">"#,
            attr(img)
        ));
    }

    out.push_str(&structured_data(page, config, program));
    out
}

/// JSON-LD describing the page and the site.
///
/// Google recommends JSON-LD over microdata because it does not interleave with
/// the markup. Only facts the page actually states are emitted — the rule is that
/// structured data must match visible content, so a title and a description that
/// appear on the page are safe, and anything invented would not be.
fn structured_data(page: &PageDecl, config: &ProjectConfig, program: &Program) -> String {
    let Some(url) = absolute_url(config, &page.path) else {
        // Every schema.org node here is identified by URL. Without an origin
        // there is nothing to identify them with.
        return String::new();
    };
    let origin = site_origin(config).unwrap_or_default();
    let name = site_name(config);
    let page_title = title(page, config);

    let mut graph = Vec::new();

    graph.push(format!(
        r#"{{"@type":"WebSite","@id":"{origin}/#website","url":"{origin}/","name":"{}"}}"#,
        json_str(&name)
    ));

    graph.push(format!(
        r#"{{"@type":"Organization","@id":"{origin}/#organization","name":"{}","url":"{origin}/"}}"#,
        json_str(&name)
    ));

    let page_type = match page.page_type.as_deref() {
        Some("article") => "Article",
        _ => "WebPage",
    };
    let mut web_page = format!(
        r#"{{"@type":"{page_type}","@id":"{}#webpage","url":"{}","name":"{}","isPartOf":{{"@id":"{origin}/#website"}}"#,
        json_str(&url),
        json_str(&url),
        json_str(&page_title)
    );
    if let Some(d) = description(page, config) {
        web_page.push_str(&format!(r#","description":"{}""#, json_str(d)));
    }
    web_page.push('}');
    graph.push(web_page);

    // A breadcrumb trail from the route, which the engine already knows. Only
    // for nested routes: a one-item trail on the home page says nothing.
    if let Some(crumbs) = breadcrumbs(page, config, program, origin) {
        graph.push(crumbs);
    }

    format!(
        "    <script type=\"application/ld+json\">{{\"@context\":\"https://schema.org\",\"@graph\":[{}]}}</script>\n",
        graph.join(",")
    )
}

/// A `BreadcrumbList` derived from the route's own segments.
fn breadcrumbs(
    page: &PageDecl,
    config: &ProjectConfig,
    program: &Program,
    origin: &str,
) -> Option<String> {
    let segments: Vec<&str> = page.path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return None;
    }

    let base = config.build.base_path.trim_end_matches('/');
    let mut items = vec![format!(
        r#"{{"@type":"ListItem","position":1,"name":"Home","item":"{origin}{base}/"}}"#
    )];

    let mut accumulated = String::new();
    for (i, segment) in segments.iter().enumerate() {
        accumulated.push('/');
        accumulated.push_str(segment);
        // Prefer the declared page's own title over the URL segment.
        let name = program
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Page(p) if p.path.trim_end_matches('/') == accumulated => {
                    p.title.clone()
                }
                _ => None,
            })
            .unwrap_or_else(|| humanise(segment));
        items.push(format!(
            r#"{{"@type":"ListItem","position":{},"name":"{}","item":"{origin}{base}{}"}}"#,
            i + 2,
            json_str(&name),
            accumulated
        ));
    }

    Some(format!(
        r#"{{"@type":"BreadcrumbList","@id":"{origin}{base}{}#breadcrumb","itemListElement":[{}]}}"#,
        page.path,
        items.join(",")
    ))
}

/// `getting-started` → `Getting Started`.
fn humanise(segment: &str) -> String {
    segment
        .split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ─── Site-level files ───────────────────────────────────────────────────

/// `sitemap.xml` for every indexable static route.
///
/// `priority` and `changefreq` are deliberately absent: Google ignores both, so
/// emitting them is noise that implies a control the author does not have.
pub fn sitemap(config: &ProjectConfig, program: &Program) -> Option<String> {
    site_origin(config)?;

    let mut urls = String::new();
    for decl in &program.declarations {
        let Declaration::Page(page) = decl else {
            continue;
        };
        // A dynamic route has no single URL, a wildcard is not a page, and a
        // page excluded from search does not belong in a sitemap.
        if page.path.contains(':') || page.path.contains('*') || page.noindex {
            continue;
        }
        let Some(url) = absolute_url(config, &page.path) else {
            continue;
        };
        urls.push_str(&format!(
            "  <url>\n    <loc>{}</loc>\n  </url>\n",
            attr(&url)
        ));
    }

    if urls.is_empty() {
        return None;
    }

    Some(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{urls}</urlset>\n"
    ))
}

/// `robots.txt`, pointing at the sitemap.
pub fn robots_txt(config: &ProjectConfig, has_sitemap: bool) -> String {
    let mut out = String::from("User-agent: *\nAllow: /\n");
    if has_sitemap {
        if let Some(origin) = site_origin(config) {
            let base = config.build.base_path.trim_end_matches('/');
            out.push_str(&format!("\nSitemap: {origin}{base}/sitemap.xml\n"));
        }
    }
    out
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

    fn config(json: &str) -> ProjectConfig {
        serde_json::from_str(json).expect("config")
    }

    fn head(src: &str, cfg: &str) -> String {
        let program = parse(src);
        let config = config(cfg);
        let page = program
            .declarations
            .iter()
            .find_map(|d| {
                if let Declaration::Page(p) = d {
                    Some(p)
                } else {
                    None
                }
            })
            .expect("a page");
        head_tags(page, &config, &program)
    }

    const SITE: &str = r#"{"name":"Ledger","meta":{"site_url":"https://ledger.example","description":"Site desc"}}"#;

    #[test]
    fn a_page_gets_a_self_referencing_absolute_canonical() {
        let out = head(
            r#"Page About (path: "/about", title: "About") { Text("x") }"#,
            SITE,
        );
        assert!(
            out.contains(r#"<link rel="canonical" href="https://ledger.example/about">"#),
            "{out}"
        );
    }

    /// A relative canonical is worse than none: Google's guidance says it causes
    /// problems later, so with no origin the tag is left out.
    #[test]
    fn no_site_url_means_no_canonical_rather_than_a_relative_one() {
        let out = head(
            r#"Page About (path: "/about", title: "About") { Text("x") }"#,
            r#"{"name":"L"}"#,
        );
        assert!(!out.contains("canonical"), "{out}");
        assert!(!out.contains("og:url"), "{out}");
    }

    #[test]
    fn a_noindex_page_says_so_and_gets_no_canonical() {
        let out = head(
            r#"Page Draft (path: "/draft", title: "Draft", noindex) { Text("x") }"#,
            SITE,
        );
        assert!(
            out.contains(r#"<meta name="robots" content="noindex, follow">"#),
            "{out}"
        );
        assert!(
            !out.contains("rel=\"canonical\""),
            "a hidden page needs no canonical: {out}"
        );
    }

    #[test]
    fn an_indexable_page_does_not_state_the_default() {
        let out = head(r#"Page P (path: "/", title: "P") { Text("x") }"#, SITE);
        assert!(
            !out.contains("name=\"robots\""),
            "index,follow is the default: {out}"
        );
    }

    #[test]
    fn the_sharing_card_carries_the_page_title_and_description() {
        let out = head(
            r#"Page P (path: "/", title: "Invoicing", description: "Nine seconds") { Text("x") }"#,
            SITE,
        );
        assert!(
            out.contains(r#"<meta property="og:title" content="Invoicing">"#),
            "{out}"
        );
        assert!(
            out.contains(r#"<meta property="og:description" content="Nine seconds">"#),
            "{out}"
        );
        assert!(
            out.contains(r#"<meta property="og:site_name" content="Ledger">"#),
            "{out}"
        );
        assert!(
            out.contains(r#"content="summary""#),
            "no image, so not a large card: {out}"
        );
    }

    #[test]
    fn a_page_image_upgrades_the_card_and_is_made_absolute() {
        let out = head(
            r#"Page P (path: "/", title: "P", image: "/card.png") { Text("x") }"#,
            SITE,
        );
        assert!(
            out.contains(r#"<meta property="og:image" content="https://ledger.example/card.png">"#),
            "{out}"
        );
        assert!(out.contains("summary_large_image"), "{out}");
    }

    #[test]
    fn the_page_description_wins_over_the_project_one() {
        let out = head(
            r#"Page P (path: "/", title: "P", description: "Page desc") { Text("x") }"#,
            SITE,
        );
        assert!(out.contains(r#"content="Page desc""#), "{out}");
        assert!(!out.contains("Site desc"), "{out}");
    }

    #[test]
    fn structured_data_is_json_ld_and_describes_the_page() {
        let out = head(r#"Page P (path: "/", title: "Home") { Text("x") }"#, SITE);
        assert!(
            out.contains(r#"<script type="application/ld+json">"#),
            "{out}"
        );
        assert!(out.contains(r#""@type":"WebSite""#), "{out}");
        assert!(out.contains(r#""@type":"Organization""#), "{out}");
        assert!(out.contains(r#""@type":"WebPage""#), "{out}");
        // Valid JSON, not just a string that looks like it.
        let json = out
            .split_once("ld+json\">")
            .and_then(|(_, r)| r.split_once("</script>"))
            .expect("a script body")
            .0;
        serde_json::from_str::<serde_json::Value>(json)
            .expect("structured data must be valid JSON");
    }

    #[test]
    fn an_article_page_is_typed_as_one() {
        let out = head(
            r#"Page Post (path: "/post", title: "Post", type: "article") { Text("x") }"#,
            SITE,
        );
        assert!(
            out.contains(r#"<meta property="og:type" content="article">"#),
            "{out}"
        );
        assert!(out.contains(r#""@type":"Article""#), "{out}");
    }

    #[test]
    fn a_nested_route_gets_a_breadcrumb_trail_and_the_home_page_does_not() {
        let nested = head(
            r#"Page Guide (path: "/docs/getting-started", title: "Getting Started") { Text("x") }"#,
            SITE,
        );
        assert!(nested.contains(r#""@type":"BreadcrumbList""#), "{nested}");
        assert!(nested.contains(r#""name":"Getting Started""#), "{nested}");
        assert!(
            nested.contains(r#""name":"Docs""#),
            "a URL segment becomes a readable name: {nested}"
        );

        let home = head(r#"Page P (path: "/", title: "Home") { Text("x") }"#, SITE);
        assert!(
            !home.contains("BreadcrumbList"),
            "a one-item trail says nothing: {home}"
        );
    }

    #[test]
    fn a_multilingual_site_lists_every_variant_including_itself() {
        let out = head(
            r#"Page P (path: "/", title: "P") { Text("x") }"#,
            r#"{"name":"L","meta":{"site_url":"https://l.example"},
                "i18n":{"default_locale":"en","locales":["en","ar"]}}"#,
        );
        assert!(out.contains(r#"hreflang="en""#), "{out}");
        assert!(
            out.contains(r#"hreflang="ar""#),
            "a variant must list its siblings: {out}"
        );
        assert!(out.contains(r#"hreflang="x-default""#), "{out}");
    }

    #[test]
    fn a_single_language_site_emits_no_alternates() {
        let out = head(
            r#"Page P (path: "/", title: "P") { Text("x") }"#,
            r#"{"name":"L","meta":{"site_url":"https://l.example"},
                "i18n":{"default_locale":"en","locales":["en"]}}"#,
        );
        assert!(
            !out.contains("hreflang"),
            "one language needs no alternates: {out}"
        );
    }

    #[test]
    fn text_is_escaped_everywhere_it_lands() {
        let out = head(
            r#"Page P (path: "/", title: "Say \"hi\" & <b>bye</b>") { Text("x") }"#,
            SITE,
        );
        assert!(
            !out.contains("<b>bye</b>"),
            "unescaped markup reached an attribute: {out}"
        );
        let json = out
            .split_once("ld+json\">")
            .and_then(|(_, r)| r.split_once("</script>"))
            .expect("a script body")
            .0;
        serde_json::from_str::<serde_json::Value>(json)
            .expect("a quote in a title must not break the JSON-LD");
    }

    // ─── Site files ─────────────────────────────────────

    #[test]
    fn the_sitemap_lists_static_routes_only() {
        let program = parse(
            r#"Page Home (path: "/", title: "H") { Text("x") }
               Page About (path: "/about", title: "A") { Text("x") }
               Page User (path: "/user/:id", title: "U") { Text("x") }
               Page Missing (path: "*", title: "M") { Text("x") }
               Page Draft (path: "/draft", title: "D", noindex) { Text("x") }"#,
        );
        let xml = sitemap(&config(SITE), &program).expect("a sitemap");

        assert!(xml.contains("<loc>https://ledger.example/</loc>"), "{xml}");
        assert!(
            xml.contains("<loc>https://ledger.example/about</loc>"),
            "{xml}"
        );
        assert!(
            !xml.contains(":id"),
            "a dynamic route has no single URL: {xml}"
        );
        assert!(!xml.contains('*'), "a catch-all is not a page: {xml}");
        assert!(
            !xml.contains("/draft"),
            "a noindex page does not belong in a sitemap: {xml}"
        );
        // Google ignores both, so emitting them implies a control nobody has.
        assert!(!xml.contains("priority"), "{xml}");
        assert!(!xml.contains("changefreq"), "{xml}");
    }

    #[test]
    fn no_site_url_means_no_sitemap() {
        let program = parse(r#"Page P (path: "/", title: "P") { Text("x") }"#);
        assert!(sitemap(&config(r#"{"name":"L"}"#), &program).is_none());
    }

    #[test]
    fn robots_points_at_the_sitemap() {
        let txt = robots_txt(&config(SITE), true);
        assert!(txt.contains("User-agent: *"), "{txt}");
        assert!(
            txt.contains("Sitemap: https://ledger.example/sitemap.xml"),
            "{txt}"
        );
    }

    #[test]
    fn a_base_path_reaches_every_absolute_url() {
        let cfg = config(
            r#"{"name":"L","meta":{"site_url":"https://l.example"},"build":{"base_path":"/docs"}}"#,
        );
        assert_eq!(
            absolute_url(&cfg, "/guide").as_deref(),
            Some("https://l.example/docs/guide")
        );
        assert_eq!(
            absolute_url(&cfg, "/").as_deref(),
            Some("https://l.example/docs/")
        );
    }
}
