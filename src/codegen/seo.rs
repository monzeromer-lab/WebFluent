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

use std::path::Path;

use crate::config::{CleanUrls, Owner, ProjectConfig};
use crate::parser::ast::{
    Arg, Declaration, Expr, PageDecl, Program, Statement, StatementKind, StringPart,
};

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
///
/// `site_url` may be the host alone or the site's whole address, subpath
/// and all — `https://x.github.io/docs` beside a `base_path` of `/docs`.
/// Every absolute URL adds the base path, so one already at the end of
/// `site_url` is read without it: written either way it is one address, not
/// `/docs/docs`, which is what a canonical link, a preview and the sitemap
/// said before. The base path begins with `/`, so only a whole segment
/// matches.
pub fn site_origin(config: &ProjectConfig) -> Option<&str> {
    let url = config.meta.site_url.trim_end_matches('/');
    let base = config.build.base_path.trim_end_matches('/');
    let url = if base.is_empty() {
        url
    } else {
        url.strip_suffix(base).unwrap_or(url)
    };
    if url.is_empty() { None } else { Some(url) }
}

/// The absolute URL of a route.
pub fn absolute_url(config: &ProjectConfig, path: &str) -> Option<String> {
    let origin = site_origin(config)?;
    let base = config.build.base_path.trim_end_matches('/');
    Some(format!("{origin}{base}{}", route_address(config, path)))
}

/// A route as the address a host answers without a redirect: `/contact`, or
/// `/contact/` under `build.clean_urls: "directory"`, where the build writes
/// `contact/index.html` and a host answers `/contact` with a `301` to it.
pub fn route_address(config: &ProjectConfig, path: &str) -> String {
    let route = path.trim_start_matches('/');
    if route.is_empty() {
        return "/".to_string();
    }
    match config.build.clean_urls {
        Some(CleanUrls::Directory) if !route.ends_with('/') => format!("/{route}/"),
        _ => format!("/{route}"),
    }
}

/// `og:locale` for a language tag: Open Graph reads `language_TERRITORY`
/// (`en_US`), not the BCP 47 tag a document's `lang` holds (`en`, `en-GB`).
///
/// A tag that names its region keeps it; one that does not takes the region
/// the language is most read in, and a language this table does not know
/// is paired with its own code (`xx_XX`), which is the usual spelling.
pub fn og_locale(lang: &str) -> String {
    let mut parts = lang.split(['-', '_']).filter(|p| !p.is_empty());
    let language = parts.next().unwrap_or("en").to_ascii_lowercase();
    // A script subtag (`zh-Hant-TW`) sits between the language and region.
    if let Some(region) = parts.find(|p| p.len() == 2 || p.chars().all(|c| c.is_ascii_digit())) {
        return format!("{language}_{}", region.to_ascii_uppercase());
    }
    let region = match language.as_str() {
        "en" => "US",
        "ar" => "AR",
        "zh" => "CN",
        "ja" => "JP",
        "ko" => "KR",
        "he" => "IL",
        "fa" => "IR",
        "ur" => "PK",
        "hi" | "bn" | "ta" | "te" | "mr" | "gu" => "IN",
        "pt" => "BR",
        "sv" => "SE",
        "da" => "DK",
        "nb" | "nn" | "no" => "NO",
        "el" => "GR",
        "cs" => "CZ",
        "uk" => "UA",
        "vi" => "VN",
        "ms" => "MY",
        "sw" => "KE",
        "ca" => "ES",
        "et" => "EE",
        "sl" => "SI",
        "sq" => "AL",
        "sr" => "RS",
        "ga" => "IE",
        "ka" => "GE",
        "hy" => "AM",
        "kk" => "KZ",
        "af" => "ZA",
        "fil" | "tl" => "PH",
        "ps" => "AF",
        _ => return format!("{language}_{}", language.to_ascii_uppercase()),
    };
    format!("{language}_{region}")
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

/// The page's sharing image as the reference it was written with.
fn image_ref<'a>(page: &'a PageDecl, config: &'a ProjectConfig) -> Option<&'a str> {
    page.image
        .as_deref()
        .filter(|i| !i.is_empty())
        .or(Some(config.meta.image.as_str()))
        .filter(|i| !i.is_empty())
}

/// What the page's sharing image shows: the page's own `image_alt:`, or the
/// project's `meta.image_alt` — but the project's only describes the
/// project's image, so a page that names its own picture needs its own.
fn image_alt<'a>(page: &'a PageDecl, config: &'a ProjectConfig) -> Option<&'a str> {
    if let Some(alt) = page.image_alt.as_deref().filter(|a| !a.is_empty()) {
        return Some(alt);
    }
    let own_image = page
        .image
        .as_deref()
        .is_some_and(|i| !i.is_empty() && i != config.meta.image);
    if own_image {
        return None;
    }
    Some(config.meta.image_alt.as_str()).filter(|a| !a.is_empty())
}

/// Read the size of every sharing image that is a file under `public/` —
/// `meta.image` and each page's `image:` — into `meta.image_sizes`, so the
/// card carries `og:image:width` and `og:image:height`. Only the header of
/// each file is read. An image on another origin, or a file that is not
/// there or not a picture, is left unmeasured and its card says nothing of
/// its size, which is what it said before.
pub fn measure_images(project_dir: &Path, config: &mut ProjectConfig, program: &Program) {
    let mut refs: Vec<String> = vec![config.meta.image.clone()];
    refs.extend(program.declarations.iter().filter_map(|d| match d {
        Declaration::Page(p) => p.image.clone(),
        _ => None,
    }));
    for reference in refs {
        if reference.is_empty()
            || reference.contains("://")
            || reference.starts_with("//")
            || config.meta.image_sizes.contains_key(&reference)
        {
            continue;
        }
        let relative = reference
            .split(['?', '#'])
            .next()
            .unwrap_or_default()
            .trim_start_matches('/');
        // A reference written with the base path (`/docs/card.png`) names
        // the same file as one without it.
        let base = config.build.base_path.trim_matches('/');
        let relative = if base.is_empty() {
            relative
        } else {
            relative
                .strip_prefix(base)
                .and_then(|r| r.strip_prefix('/'))
                .unwrap_or(relative)
        };
        if relative.is_empty() || relative.split('/').any(|s| s == "..") {
            continue;
        }
        if let Ok(size) = image::image_dimensions(project_dir.join("public").join(relative)) {
            config.meta.image_sizes.insert(reference, size);
        }
    }
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
    // A `meta` the page writes itself in `head { }` replaces the standard
    // one of the same `property` or `name`: one `og:title`, the page's.
    let own: Vec<String> = page
        .head
        .iter()
        .filter(|t| t.tag == "meta")
        .flat_map(|t| t.attrs.iter())
        .filter_map(|(k, v)| match (k.as_str(), v) {
            ("property" | "name", Expr::StringLiteral(value)) => {
                Some(format!(r#"<meta {k}="{value}""#))
            }
            _ => None,
        })
        .collect();
    let mut push = |line: String| {
        if own.iter().any(|prefix| line.starts_with(prefix.as_str())) {
            return;
        }
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
    if let Some(url) = &canonical
        && !page.noindex
    {
        push(format!(r#"<link rel="canonical" href="{}">"#, attr(url)));
    }

    // Language alternates. Every variant must list itself as well as the others,
    // or the set is ignored.
    if let (Some(i18n), Some(origin)) = (&config.i18n, site_origin(config))
        && i18n.locales.len() > 1
    {
        let base = config.build.base_path.trim_end_matches('/');
        let route = route_address(config, &page.path);
        for locale in &i18n.locales {
            let href = format!("{origin}{base}{route}?lang={locale}");
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
        let locale = og_locale(&config.meta.lang);
        push(format!(
            r#"<meta property="og:locale" content="{}">"#,
            attr(&locale)
        ));
        // The site's other languages, each as Open Graph spells it.
        if let Some(i18n) = &config.i18n {
            let mut seen = vec![locale];
            for other in &i18n.locales {
                let other = og_locale(other);
                if !seen.contains(&other) {
                    push(format!(
                        r#"<meta property="og:locale:alternate" content="{}">"#,
                        attr(&other)
                    ));
                    seen.push(other);
                }
            }
        }
    }

    let reference = image_ref(page, config);
    let image = reference.and_then(|i| absolute_asset(config, i));
    let alt = image_alt(page, config);
    if let Some(img) = &image {
        push(format!(
            r#"<meta property="og:image" content="{}">"#,
            attr(img)
        ));
        // The size lets a preview lay the card out before the picture
        // arrives — and some only draw the large card when they know it.
        if let Some((w, h)) = reference.and_then(|r| config.meta.image_sizes.get(r)) {
            push(format!(r#"<meta property="og:image:width" content="{w}">"#));
            push(format!(
                r#"<meta property="og:image:height" content="{h}">"#
            ));
        }
        if let Some(a) = alt {
            push(format!(
                r#"<meta property="og:image:alt" content="{}">"#,
                attr(a)
            ));
        }
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
        if let Some(a) = alt {
            push(format!(
                r#"<meta name="twitter:image:alt" content="{}">"#,
                attr(a)
            ));
        }
    }

    out.push_str(&structured_data(page, config, program));
    out
}

/// The owner node's keys that come from settings of their own, which
/// `meta.owner_details` may not set.
const OWNER_KEYS: [&str; 7] = [
    "@context", "@type", "@id", "name", "url", "jobTitle", "sameAs",
];

/// A JSON value for a `<script>` block: `<` written as `\u003c`, so no
/// string in it can close the block (`</script>`) or open a comment.
fn json_value(value: &serde_json::Value) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "null".to_string())
        .replace('<', "\\u003c")
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

    // Who the site belongs to: a person for a personal site, an
    // organisation otherwise. The site is published by them, and a page of a
    // personal site is about them.
    let (owner_type, owner_id) = match config.meta.owner {
        Owner::Person => ("Person", format!("{origin}/#person")),
        Owner::Organization => ("Organization", format!("{origin}/#organization")),
    };

    graph.push(format!(
        r#"{{"@type":"WebSite","@id":"{origin}/#website","url":"{origin}/","name":"{}","publisher":{{"@id":"{owner_id}"}}}}"#,
        json_str(&name)
    ));

    let mut owner = format!(
        r#"{{"@type":"{owner_type}","@id":"{owner_id}","name":"{}","url":"{origin}/""#,
        json_str(&name)
    );
    if config.meta.owner == Owner::Person && !config.meta.job_title.is_empty() {
        owner.push_str(&format!(
            r#","jobTitle":"{}""#,
            json_str(&config.meta.job_title)
        ));
    }
    let same_as: Vec<String> = config
        .meta
        .same_as
        .iter()
        .filter(|u| !u.is_empty())
        .map(|u| format!(r#""{}""#, json_str(u)))
        .collect();
    if !same_as.is_empty() {
        owner.push_str(&format!(r#","sameAs":[{}]"#, same_as.join(",")));
    }
    // More of who the owner is, as written. What the node is and what other
    // settings say are theirs (`E111` refuses them here), so they are skipped.
    for (key, value) in &config.meta.owner_details {
        if OWNER_KEYS.contains(&key.as_str()) {
            continue;
        }
        owner.push_str(&format!(r#","{}":{}"#, json_str(key), json_value(value)));
    }
    owner.push('}');
    graph.push(owner);

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
    if config.meta.owner == Owner::Person {
        web_page.push_str(&format!(r#","about":{{"@id":"{owner_id}"}}"#));
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

    // A crumb is a page a reader can open: a level of the path that no
    // route answers (`/notes` above `/notes/:slug`) is left out, so a
    // search result never links to a 404.
    let answers = |path: &str| {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        program.declarations.iter().any(|d| match d {
            Declaration::Page(p) if p.path != "*" => {
                let pattern: Vec<&str> = p.path.split('/').filter(|s| !s.is_empty()).collect();
                pattern.len() == parts.len()
                    && pattern
                        .iter()
                        .zip(&parts)
                        .all(|(a, b)| a.starts_with(':') || a == b)
            }
            _ => false,
        })
    };
    let mut accumulated = String::new();
    for (i, segment) in segments.iter().enumerate() {
        accumulated.push('/');
        accumulated.push_str(segment);
        let last = i + 1 == segments.len();
        if !last && !answers(&accumulated) {
            continue;
        }
        // Prefer the declared page's own title over the URL segment.
        let name = if last {
            page.title.clone()
        } else {
            program.declarations.iter().find_map(|d| match d {
                Declaration::Page(p) if p.path.trim_end_matches('/') == accumulated => {
                    p.title.clone()
                }
                _ => None,
            })
        }
        .unwrap_or_else(|| humanise(segment));
        items.push(format!(
            r#"{{"@type":"ListItem","position":{},"name":"{}","item":"{origin}{base}{}"}}"#,
            items.len() + 1,
            json_str(&name),
            route_address(config, &accumulated)
        ));
    }

    Some(format!(
        r#"{{"@type":"BreadcrumbList","@id":"{origin}{base}{}#breadcrumb","itemListElement":[{}]}}"#,
        route_address(config, &page.path),
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

// ─── Addresses that do not redirect ───────────────────────────────────

/// Under `build.clean_urls: "directory"`, give every link to a page the
/// trailing slash its address has — `Link(to: "/contact")`, `Sidebar.Item(to:
/// …)`, `navigate("/contact")` — so a link, like the canonical, names the
/// address the host answers rather than one it redirects. A link that is
/// not to a page — a file, another origin, a fragment, an address worked out
/// whole at run time — is left as written. Run once, after the checks and
/// before code generation, so the static paint and the live page agree.
pub fn directory_links(config: &ProjectConfig, program: &mut Program) {
    if config.build.clean_urls != Some(CleanUrls::Directory) {
        return;
    }
    let routes: Vec<Vec<String>> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Page(p) if !p.path.contains('*') => Some(
                p.path
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect(),
            ),
            _ => None,
        })
        .collect();
    let fix_expr = |e: &mut Expr| {
        if let Some(fixed) = with_trailing_slash(e, &routes) {
            *e = fixed;
        }
    };
    let mut fix = |stmt: &mut Statement| match &mut stmt.kind {
        StatementKind::UIElement(ui) => {
            for arg in &mut ui.args {
                if let Arg::Named(k, v) = arg
                    && k == "to"
                {
                    fix_expr(v);
                }
            }
        }
        StatementKind::Navigate(target) => fix_expr(target),
        StatementKind::ExprStatement(Expr::FunctionCall(name, args))
            if name == "navigate" && !args.is_empty() =>
        {
            fix_expr(&mut args[0]);
        }
        _ => {}
    };
    for decl in &mut program.declarations {
        let body = match decl {
            Declaration::Page(p) => &mut p.body,
            Declaration::Component(c) => &mut c.body,
            Declaration::Store(s) => &mut s.body,
            Declaration::App(a) => &mut a.body,
            _ => continue,
        };
        crate::parser::ast::walk_statements_mut(body, &mut fix);
    }
}

/// `target` with a `/` at the end of its path, when it is a link to one of
/// `routes` (each a page's path, by segment) that has none; `None` when it
/// is not a page's address or already ends in one.
fn with_trailing_slash(target: &Expr, routes: &[Vec<String>]) -> Option<Expr> {
    const SPLICE: char = '\u{1}';
    let parts: Vec<StringPart> = match target {
        Expr::StringLiteral(s) => vec![StringPart::Literal(s.clone())],
        Expr::InterpolatedString(parts) => parts.clone(),
        _ => return None,
    };
    let text: String = parts
        .iter()
        .map(|p| match p {
            StringPart::Literal(t) => t.clone(),
            StringPart::Expression(_) => SPLICE.to_string(),
        })
        .collect();
    let path = text.split(['?', '#']).next().unwrap_or("");
    if !path.starts_with('/') || path.starts_with("//") || path == "/" || path.ends_with('/') {
        return None;
    }
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    // A file — `/cv.pdf` — is not a page.
    if segments
        .last()
        .is_some_and(|last| !last.contains(SPLICE) && last.contains('.'))
    {
        return None;
    }
    let is_page = routes.iter().any(|route| {
        route.len() == segments.len()
            && route
                .iter()
                .zip(&segments)
                .all(|(r, s)| r.starts_with(':') || s.contains(SPLICE) || r == s)
    });
    if !is_page {
        return None;
    }
    // The slash goes where the path ends: before the first `?` or `#` a
    // literal part holds, or at the very end.
    let mut out = Vec::with_capacity(parts.len() + 1);
    let mut placed = false;
    for part in parts {
        match part {
            StringPart::Literal(t) if !placed && t.contains(['?', '#']) => {
                let at = t.find(['?', '#']).unwrap_or(t.len());
                out.push(StringPart::Literal(format!("{}/{}", &t[..at], &t[at..])));
                placed = true;
            }
            other => out.push(other),
        }
    }
    if !placed {
        match out.last_mut() {
            Some(StringPart::Literal(t)) => t.push('/'),
            _ => out.push(StringPart::Literal("/".to_string())),
        }
    }
    Some(match target {
        Expr::StringLiteral(_) => match out.as_slice() {
            [StringPart::Literal(t)] => Expr::StringLiteral(t.clone()),
            _ => unreachable!("a literal stays one part"),
        },
        _ => Expr::InterpolatedString(out),
    })
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
        // A wildcard is not a page, and a page excluded from search does not
        // belong in a sitemap. A `:param` route lists the routes its
        // `paths:` name, and nothing without them.
        if page.path.contains('*') || page.noindex {
            continue;
        }
        let routes: Vec<String> = if page.path.contains(':') {
            crate::codegen::ssg::static_routes(page, program, &config.env)
                .map(|rs| rs.into_iter().map(|(r, _)| r).collect())
                .unwrap_or_default()
        } else {
            vec![page.path.clone()]
        };
        for route in routes {
            let Some(url) = absolute_url(config, &route) else {
                continue;
            };
            urls.push_str(&format!(
                "  <url>\n    <loc>{}</loc>\n  </url>\n",
                attr(&url)
            ));
        }
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
    if has_sitemap && let Some(origin) = site_origin(config) {
        let base = config.build.base_path.trim_end_matches('/');
        out.push_str(&format!("\nSitemap: {origin}{base}/sitemap.xml\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Program {
        crate::syntax::parse_source(src, "<t>").expect("parse")
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
            r#"page About(path: "/about", title: "About") { Text("x") }"#,
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
            r#"page About(path: "/about", title: "About") { Text("x") }"#,
            r#"{"name":"L"}"#,
        );
        assert!(!out.contains("canonical"), "{out}");
        assert!(!out.contains("og:url"), "{out}");
    }

    #[test]
    fn a_noindex_page_says_so_and_gets_no_canonical() {
        let out = head(
            r#"page Draft(path: "/draft", title: "Draft", noindex: true) { Text("x") }"#,
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
    fn a_pages_own_meta_replaces_the_standard_one_of_the_same_name() {
        let out = head(
            r#"page P(path: "/p", title: "Section") { head { meta(property: "og:title", content: "The post")  meta(name: "twitter:card", content: "player") }  Text("x") }"#,
            SITE,
        );
        assert!(!out.contains(r#"<meta property="og:title""#), "{out}");
        assert!(!out.contains(r#"<meta name="twitter:card""#), "{out}");
        assert!(
            out.contains(r#"<meta name="twitter:title" content="Section">"#),
            "{out}"
        );
    }

    #[test]
    fn an_indexable_page_does_not_state_the_default() {
        let out = head(r#"page P(path: "/", title: "P") { Text("x") }"#, SITE);
        assert!(
            !out.contains("name=\"robots\""),
            "index,follow is the default: {out}"
        );
    }

    #[test]
    fn the_sharing_card_carries_the_page_title_and_description() {
        let out = head(
            r#"page P(path: "/", title: "Invoicing", description: "Nine seconds") { Text("x") }"#,
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
            r#"page P(path: "/", title: "P", image: "/card.png") { Text("x") }"#,
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
            r#"page P(path: "/", title: "P", description: "Page desc") { Text("x") }"#,
            SITE,
        );
        assert!(out.contains(r#"content="Page desc""#), "{out}");
        assert!(!out.contains("Site desc"), "{out}");
    }

    #[test]
    fn structured_data_is_json_ld_and_describes_the_page() {
        let out = head(r#"page P(path: "/", title: "Home") { Text("x") }"#, SITE);
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
            r#"page Post(path: "/post", title: "Post", type: "article") { Text("x") }"#,
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
            r#"page Guide(path: "/docs/getting-started", title: "Getting Started") { Text("x") }"#,
            SITE,
        );
        assert!(nested.contains(r#""@type":"BreadcrumbList""#), "{nested}");
        assert!(nested.contains(r#""name":"Getting Started""#), "{nested}");
        assert!(
            !nested.contains(r#""name":"Docs""#),
            "a level no page answers is no crumb, so a result never links to a 404: {nested}"
        );
        // A level a page answers is a crumb, by that page's title.
        let program = parse(
            r#"page Docs(path: "/docs", title: "The docs") { Text("x") }
page Guide(path: "/docs/getting-started", title: "Getting Started") { Text("x") }"#,
        );
        let Declaration::Page(guide) = &program.declarations[1] else {
            unreachable!("the second declaration is a page")
        };
        let crumbs =
            super::breadcrumbs(guide, &config(SITE), &program, "https://ledger.example").unwrap();
        assert!(crumbs.contains(r#""name":"The docs""#), "{crumbs}");

        let home = head(r#"page P(path: "/", title: "Home") { Text("x") }"#, SITE);
        assert!(
            !home.contains("BreadcrumbList"),
            "a one-item trail says nothing: {home}"
        );
    }

    #[test]
    fn a_multilingual_site_lists_every_variant_including_itself() {
        let out = head(
            r#"page P(path: "/", title: "P") { Text("x") }"#,
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
            r#"page P(path: "/", title: "P") { Text("x") }"#,
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
            r#"page P(path: "/", title: "Say \"hi\" & <b>bye</b>") { Text("x") }"#,
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
            r#"page Home(path: "/", title: "H") { Text("x") }
               page About(path: "/about", title: "A") { Text("x") }
               page User(path: "/user/:id", title: "U") { Text("x") }
               page Missing(path: "*", title: "M") { Text("x") }
               page Draft(path: "/draft", title: "D", noindex: true) { Text("x") }"#,
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
        let program = parse(r#"page P(path: "/", title: "P") { Text("x") }"#);
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
    fn a_site_url_that_ends_with_the_base_path_is_not_given_it_twice() {
        for site_url in [
            "https://l.example/docs",
            "https://l.example/docs/",
            "https://l.example",
        ] {
            let config: ProjectConfig = serde_json::from_str(&format!(
                r#"{{"name":"L","meta":{{"site_url":"{site_url}"}},"build":{{"base_path":"/docs"}}}}"#
            ))
            .unwrap();
            assert_eq!(
                absolute_url(&config, "/guide").as_deref(),
                Some("https://l.example/docs/guide"),
                "{site_url}"
            );
        }
        // Only a whole segment: `/mydocs` is not `/docs`.
        let config: ProjectConfig = serde_json::from_str(
            r#"{"name":"L","meta":{"site_url":"https://l.example/mydocs"},"build":{"base_path":"/docs"}}"#,
        )
        .unwrap();
        assert_eq!(
            absolute_url(&config, "/").as_deref(),
            Some("https://l.example/mydocs/docs/")
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

    // ─── The owner ──────────────────────────────────────

    fn graph(out: &str) -> serde_json::Value {
        let json = out
            .split_once("ld+json\">")
            .and_then(|(_, r)| r.split_once("</script>"))
            .expect("a script body")
            .0;
        serde_json::from_str::<serde_json::Value>(json).expect("valid JSON-LD")["@graph"].clone()
    }

    fn node<'a>(graph: &'a serde_json::Value, ty: &str) -> &'a serde_json::Value {
        graph
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["@type"] == ty)
            .unwrap_or_else(|| panic!("no {ty} node in {graph}"))
    }

    #[test]
    fn a_site_is_published_by_an_organization_by_default() {
        let out = head(r#"page P(path: "/", title: "Home") { Text("x") }"#, SITE);
        let g = graph(&out);
        let org = node(&g, "Organization");
        assert_eq!(org["@id"], "https://ledger.example/#organization");
        assert_eq!(
            node(&g, "WebSite")["publisher"]["@id"],
            "https://ledger.example/#organization"
        );
        assert!(
            node(&g, "WebPage").get("about").is_none(),
            "a company's page is not about the company: {out}"
        );
    }

    #[test]
    fn a_personal_site_is_a_person_the_site_and_its_pages_point_at() {
        let out = head(
            r#"page P(path: "/about", title: "About") { Text("x") }"#,
            r#"{"name":"site","meta":{"site_url":"https://ada.example","site_name":"Ada Lovelace",
                "owner":"person","job_title":"Analyst",
                "same_as":["https://github.com/ada","https://www.linkedin.com/in/ada/"]}}"#,
        );
        let g = graph(&out);
        assert!(!out.contains(r#""Organization""#), "{out}");
        let person = node(&g, "Person");
        assert_eq!(person["@id"], "https://ada.example/#person");
        assert_eq!(person["name"], "Ada Lovelace");
        assert_eq!(person["jobTitle"], "Analyst");
        assert_eq!(person["sameAs"][1], "https://www.linkedin.com/in/ada/");
        assert_eq!(
            node(&g, "WebSite")["publisher"]["@id"],
            "https://ada.example/#person"
        );
        assert_eq!(
            node(&g, "WebPage")["about"]["@id"],
            "https://ada.example/#person"
        );
    }

    #[test]
    fn an_organization_takes_its_profiles_but_no_job_title() {
        let out = head(
            r#"page P(path: "/", title: "Home") { Text("x") }"#,
            r#"{"name":"L","meta":{"site_url":"https://l.example","job_title":"CEO",
                "same_as":["https://github.com/l"]}}"#,
        );
        let g = graph(&out);
        let org = node(&g, "Organization");
        assert_eq!(org["sameAs"][0], "https://github.com/l");
        assert!(org.get("jobTitle").is_none(), "{out}");
    }

    #[test]
    fn owner_details_are_merged_into_the_owner_node() {
        let out = head(
            r#"page P(path: "/", title: "Home") { Text("x") }"#,
            r#"{"name":"Ada Lovelace","meta":{"site_url":"https://ada.example","owner":"person",
                "job_title":"Analyst",
                "owner_details":{"alternateName":"Augusta Ada King",
                    "worksFor":[{"@type":"Organization","name":"Analytical Engines"}],
                    "knowsLanguage":["en","fr"]}}}"#,
        );
        let g = graph(&out);
        let person = node(&g, "Person");
        assert_eq!(person["alternateName"], "Augusta Ada King");
        assert_eq!(person["worksFor"][0]["name"], "Analytical Engines");
        assert_eq!(person["knowsLanguage"][1], "fr");
        assert_eq!(person["jobTitle"], "Analyst");
    }

    #[test]
    fn owner_details_cannot_replace_what_the_node_is() {
        let out = head(
            r#"page P(path: "/", title: "Home") { Text("x") }"#,
            r#"{"name":"Ada","meta":{"site_url":"https://ada.example","owner":"person",
                "owner_details":{"@type":"Organization","name":"Someone else","url":"https://x.example/"}}}"#,
        );
        let g = graph(&out);
        let person = node(&g, "Person");
        assert_eq!(person["name"], "Ada");
        assert_eq!(person["url"], "https://ada.example/");
        assert_eq!(
            out.matches(r#""name":"#).count(),
            3,
            "website, person, page: {out}"
        );
    }

    #[test]
    fn owner_details_cannot_close_the_script_block() {
        let out = head(
            r#"page P(path: "/", title: "Home") { Text("x") }"#,
            r#"{"name":"Ada","meta":{"site_url":"https://ada.example",
                "owner_details":{"description":"</script><script>alert(1)</script>"}}}"#,
        );
        assert!(!out.contains("</script><script>"), "{out}");
        let g = graph(&out);
        assert_eq!(
            node(&g, "Organization")["description"],
            "</script><script>alert(1)</script>"
        );
    }

    // ─── The sharing image ──────────────────────────────

    #[test]
    fn a_measured_image_carries_its_size_and_its_description() {
        let program = parse(r#"page P(path: "/", title: "P") { Text("x") }"#);
        let mut cfg = config(
            r#"{"name":"L","meta":{"site_url":"https://l.example","image":"/card.png",
                "image_alt":"The ledger, open on a desk"}}"#,
        );
        cfg.meta.image_sizes.insert("/card.png".into(), (1200, 630));
        let Declaration::Page(page) = &program.declarations[0] else {
            unreachable!()
        };
        let out = head_tags(page, &cfg, &program);
        for tag in [
            r#"<meta property="og:image:width" content="1200">"#,
            r#"<meta property="og:image:height" content="630">"#,
            r#"<meta property="og:image:alt" content="The ledger, open on a desk">"#,
            r#"<meta name="twitter:image:alt" content="The ledger, open on a desk">"#,
        ] {
            assert!(out.contains(tag), "{tag} missing: {out}");
        }
    }

    #[test]
    fn an_unmeasured_image_says_nothing_of_its_size() {
        let out = head(
            r#"page P(path: "/", title: "P", image: "https://cdn.example/c.png") { Text("x") }"#,
            SITE,
        );
        assert!(out.contains("og:image"), "{out}");
        assert!(!out.contains("og:image:width"), "{out}");
        assert!(
            !out.contains("image:alt"),
            "no description was given: {out}"
        );
    }

    #[test]
    fn a_page_describes_its_own_image_and_the_projects_alt_stays_with_the_projects_image() {
        let cfg = r#"{"name":"L","meta":{"site_url":"https://l.example","image":"/card.png",
            "image_alt":"The project card"}}"#;
        let own = head(
            r#"page P(path: "/", title: "P", image: "/post.png", image_alt: "A chart going up") { Text("x") }"#,
            cfg,
        );
        assert!(
            own.contains(r#"<meta property="og:image:alt" content="A chart going up">"#),
            "{own}"
        );
        let other = head(
            r#"page P(path: "/", title: "P", image: "/post.png") { Text("x") }"#,
            cfg,
        );
        assert!(
            !other.contains("The project card"),
            "the project's alt describes another picture: {other}"
        );
    }

    #[test]
    fn the_build_reads_an_images_size_from_public() {
        let dir = std::env::temp_dir().join(format!("wf-seo-measure-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("public/img")).unwrap();
        image::RgbImage::new(40, 21)
            .save(dir.join("public/img/card.png"))
            .unwrap();
        let program = parse(
            r#"page P(path: "/", title: "P", image: "/img/card.png") { Text("x") }
               page Q(path: "/q", title: "Q", image: "/missing.png") { Text("x") }"#,
        );
        let mut cfg = config(
            r#"{"name":"L","meta":{"image":"https://cdn.example/x.png"},"build":{"base_path":"/docs"}}"#,
        );
        measure_images(&dir, &mut cfg, &program);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(cfg.meta.image_sizes.get("/img/card.png"), Some(&(40, 21)));
        assert_eq!(cfg.meta.image_sizes.len(), 1, "{:?}", cfg.meta.image_sizes);
    }

    // ─── og:locale ──────────────────────────────────────

    #[test]
    fn a_language_tag_becomes_the_locale_open_graph_reads() {
        assert_eq!(og_locale("en"), "en_US");
        assert_eq!(og_locale("en-GB"), "en_GB");
        assert_eq!(og_locale("pt_br"), "pt_BR");
        assert_eq!(og_locale("ar"), "ar_AR");
        assert_eq!(og_locale("fr"), "fr_FR");
        assert_eq!(og_locale("zh-Hant-TW"), "zh_TW");
        assert_eq!(og_locale("es-419"), "es_419");
    }

    #[test]
    fn the_card_names_its_locale_and_the_sites_others() {
        let out = head(
            r#"page P(path: "/", title: "P") { Text("x") }"#,
            r#"{"name":"L","meta":{"site_url":"https://l.example","lang":"en"},
                "i18n":{"default_locale":"en","locales":["en","ar"]}}"#,
        );
        assert!(
            out.contains(r#"<meta property="og:locale" content="en_US">"#),
            "{out}"
        );
        assert!(
            out.contains(r#"<meta property="og:locale:alternate" content="ar_AR">"#),
            "{out}"
        );
        assert!(!out.contains(r#"alternate" content="en_US""#), "{out}");
    }

    // ─── Addresses that do not redirect ─────────────────

    const DIRECTORY: &str = r#"{"name":"L","meta":{"site_url":"https://l.example"},
        "build":{"clean_urls":"directory"}}"#;

    #[test]
    fn a_directory_site_names_every_route_with_its_slash() {
        let out = head(
            r#"page Docs(path: "/docs", title: "Docs") { Text("x") }
               page Guide(path: "/docs/guide", title: "Guide") { Text("x") }"#,
            DIRECTORY,
        );
        assert!(
            out.contains(r#"<link rel="canonical" href="https://l.example/docs/">"#),
            "{out}"
        );
        assert!(
            out.contains(r#"<meta property="og:url" content="https://l.example/docs/">"#),
            "{out}"
        );
        let program = parse(
            r#"page Home(path: "/", title: "H") { Text("x") }
               page Docs(path: "/docs", title: "Docs") { Text("x") }
               page Guide(path: "/docs/guide", title: "Guide") { Text("x") }"#,
        );
        let xml = sitemap(&config(DIRECTORY), &program).unwrap();
        assert!(xml.contains("<loc>https://l.example/</loc>"), "{xml}");
        assert!(xml.contains("<loc>https://l.example/docs/</loc>"), "{xml}");
        assert!(
            xml.contains("<loc>https://l.example/docs/guide/</loc>"),
            "{xml}"
        );
        let Declaration::Page(guide) = &program.declarations[2] else {
            unreachable!()
        };
        let crumbs = breadcrumbs(guide, &config(DIRECTORY), &program, "https://l.example").unwrap();
        assert!(
            crumbs.contains(r#""item":"https://l.example/docs/""#),
            "{crumbs}"
        );
        assert!(
            crumbs.contains(r#""item":"https://l.example/docs/guide/""#),
            "{crumbs}"
        );
    }

    #[test]
    fn a_file_site_and_an_unset_one_keep_the_address_without_the_slash() {
        for cfg in [
            r#"{"name":"L","meta":{"site_url":"https://l.example"},"build":{"clean_urls":"file"}}"#,
            r#"{"name":"L","meta":{"site_url":"https://l.example"}}"#,
        ] {
            assert_eq!(
                absolute_url(&config(cfg), "/contact").as_deref(),
                Some("https://l.example/contact")
            );
        }
    }

    fn to_of(program: &Program, page: usize) -> Vec<String> {
        let Declaration::Page(p) = &program.declarations[page] else {
            unreachable!()
        };
        let mut found = Vec::new();
        fn walk(stmts: &[Statement], found: &mut Vec<String>) {
            for s in stmts {
                match &s.kind {
                    StatementKind::UIElement(ui) => {
                        for a in &ui.args {
                            if let Arg::Named(k, v) = a
                                && k == "to"
                            {
                                found.push(match v {
                                    Expr::StringLiteral(t) => t.clone(),
                                    Expr::InterpolatedString(parts) => parts
                                        .iter()
                                        .map(|p| match p {
                                            StringPart::Literal(t) => t.clone(),
                                            StringPart::Expression(_) => "{}".into(),
                                        })
                                        .collect(),
                                    other => format!("{other:?}"),
                                });
                            }
                        }
                        walk(&ui.children, found);
                        for h in &ui.events {
                            walk(&h.body, found);
                        }
                    }
                    StatementKind::Navigate(Expr::StringLiteral(t)) => found.push(t.clone()),
                    _ => {}
                }
            }
        }
        walk(&p.body, &mut found);
        found
    }

    #[test]
    fn a_directory_site_links_to_a_page_with_its_slash_and_to_a_file_as_written() {
        let mut program = parse(
            r#"page Home(path: "/", title: "H") {
                   Link("Contact", to: "/contact")
                   Link("Post", to: "/p/{slug}?ref=home#top")
                   Link("CV", to: "/cv.pdf")
                   Link("Home", to: "/")
                   Link("Away", to: "https://x.example/contact")
                   Link("Nowhere", to: "/nowhere")
                   Link("Already", to: "/contact/")
                   Button("Go") { on click { navigate("/contact") } }
               }
               page Contact(path: "/contact", title: "C") { Text("x") }
               page Post(path: "/p/:slug", title: "P", slug: String) { Text(slug) }"#,
        );
        directory_links(&config(DIRECTORY), &mut program);
        assert_eq!(
            to_of(&program, 0),
            [
                "/contact/",
                "/p/{}/?ref=home#top",
                "/cv.pdf",
                "/",
                "https://x.example/contact",
                "/nowhere",
                "/contact/",
                "/contact/",
            ]
        );

        // Unset, nothing moves.
        let mut program = parse(
            r#"page Home(path: "/", title: "H") { Link("Contact", to: "/contact") }
               page Contact(path: "/contact", title: "C") { Text("x") }"#,
        );
        directory_links(&config(SITE), &mut program);
        assert_eq!(to_of(&program, 0), ["/contact"]);
    }
}
