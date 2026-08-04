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
{}{}{}    <link rel="stylesheet" href="styles.css">
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
        SKIP_LINK,
    )
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
        crate::config::project::CSP_POLICY
    )
}
