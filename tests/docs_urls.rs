//! Addresses the documentation has given out keep working.
//!
//! Two kinds of address outlive any one version of the site: the pages a
//! published site listed in its sitemap (search engines, bookmarks and other
//! sites hold them), and the `docs:` link every `wf` prints under a finding
//! (a binary keeps printing the link it was built with, for as long as
//! anyone runs it). Each is kept in `tests/fixtures/docs-urls/`, as the
//! release that gave it out had it, and nothing may take one away: a page
//! that moves leaves a redirect behind, and a heading the diagnostics page
//! loses needs somewhere to send its anchor.

use std::path::{Path, PathBuf};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(name: &str) -> Vec<String> {
    std::fs::read_to_string(root().join("tests/fixtures/docs-urls").join(name))
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn site_pages(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            site_pages(&path, out);
        } else if path.extension().is_some_and(|x| x == "wf" || x == "md") {
            out.push(path);
        }
    }
}

/// Every route the site declares: a page's `path:`, or a Markdown page's
/// `path:` in its front matter.
fn routes() -> Vec<String> {
    let mut files = Vec::new();
    site_pages(&root().join("site/src"), &mut files);
    let path = regex::Regex::new(r#"(?m)(?:\bpath: "|^path: )([^"\n]+)"?"#).unwrap();
    let mut out = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        out.extend(path.captures_iter(&text).map(|c| c[1].trim().to_string()));
    }
    out
}

/// Whether `route` answers `url`: segment by segment, a `:param` taking any
/// one segment and a `*` the rest.
fn answers(route: &str, url: &str) -> bool {
    let route: Vec<&str> = route.trim_matches('/').split('/').collect();
    let url: Vec<&str> = url.trim_matches('/').split('/').collect();
    for (i, seg) in route.iter().enumerate() {
        if *seg == "*" {
            return true;
        }
        match url.get(i) {
            Some(u) if seg.starts_with(':') && !u.is_empty() => {}
            Some(u) if u == seg => {}
            _ => return false,
        }
    }
    route.len() == url.len()
}

/// A page on a `:param` route answers only the values its `paths:` name. The
/// components reference's are the slugs of `scripts/docsite/reference.py`'s
/// data; any other parameter route the old site listed is checked by shape
/// alone.
fn reference_slugs() -> Vec<String> {
    let builtins: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("site/src/data/components.json")).unwrap(),
    )
    .unwrap();
    builtins["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["slug"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn every_page_the_6_0_site_listed_still_answers() {
    // The not-found page's `*` answers everything, and answering with it
    // is what losing a page looks like.
    let routes: Vec<String> = routes().into_iter().filter(|r| !r.contains('*')).collect();
    let slugs = reference_slugs();
    let mut lost = Vec::new();
    for url in fixture("site-6.0.txt") {
        let exact = routes
            .iter()
            .any(|r| r.trim_end_matches('/') == url.trim_end_matches('/'));
        let by_param = routes.iter().any(|r| answers(r, &url));
        let ok = if let Some(slug) = url.strip_prefix("/docs/reference/") {
            exact || slugs.iter().any(|s| s == slug)
        } else {
            exact || by_param
        };
        if !ok {
            lost.push(url);
        }
    }
    assert!(
        lost.is_empty(),
        "these addresses the 6.0 site published answer nothing now; leave a redirect page \
         behind (site/src/pages/Redirects.wf): {lost:#?}"
    );
}

/// The `docs:` link of every code, as 6.0 prints it. `WF_WRITE_DOCS_URLS=1`
/// writes the fixture from the compiler under test — only ever for the
/// release that gives the links out, never to make this test pass.
#[test]
fn every_diagnostic_link_6_0_prints_still_lands() {
    let path = root().join("tests/fixtures/docs-urls/diagnostics-6.0.txt");
    if std::env::var_os("WF_WRITE_DOCS_URLS").is_some() {
        let mut lines: Vec<String> = webfluent::diagnostics::codes::CODES
            .iter()
            .map(|c| webfluent::diagnostics::codes::docs_url(c.code))
            .collect();
        lines.sort();
        lines.insert(
            0,
            "# The docs: link under every finding, as wf 6.0 prints it.".into(),
        );
        std::fs::write(&path, format!("{}\n", lines.join("\n"))).unwrap();
    }
    // The old page answers, and sends a code's anchor to the code's page:
    // the forwarding is `site/src/old-links.js`, which reads the code off
    // the anchor (`#t05-…` → `/errors/T05`).
    let routes = routes();
    assert!(
        routes.iter().any(|r| r == "/docs/guide/diagnostics"),
        "the diagnostics chapter's old address has no page to forward from"
    );
    let forwarder = std::fs::read_to_string(root().join("site/src/old-links.js")).unwrap();
    assert!(
        forwarder.contains("/docs/guide/diagnostics") && forwarder.contains("\"/errors/\""),
        "site/src/old-links.js no longer forwards the diagnostics chapter's anchors"
    );
    let mut lost = Vec::new();
    for url in fixture("diagnostics-6.0.txt") {
        let (page_url, anchor) = url.split_once('#').unwrap_or((&url, ""));
        let code = anchor.split('-').next().unwrap_or("").to_uppercase();
        if page_url != "https://webfluent.monzeromer.dev/docs/guide/diagnostics"
            || !root().join(format!("md-docs/errors/{code}.md")).exists()
        {
            lost.push(url);
        }
    }
    assert!(
        lost.is_empty(),
        "these links, printed by every wf 6.0, name a code with no page now: {lost:#?}"
    );
}

/// The link this `wf` prints under a finding goes to a page the site has:
/// `/errors/CODE`, one of the pages `scripts/docsite/errors.py` writes.
#[test]
fn every_diagnostic_link_this_wf_prints_has_a_page() {
    let routes = routes();
    for c in webfluent::diagnostics::codes::CODES {
        let url = webfluent::diagnostics::codes::docs_url(c.code);
        let path = format!("/errors/{}", c.code);
        assert_eq!(url, format!("https://webfluent.monzeromer.dev{path}"));
        assert!(routes.contains(&path), "{path} is no page of the site");
    }
}
