use crate::config::ProjectConfig;

pub fn generate_html(config: &ProjectConfig) -> String {
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

    let description_meta = if config.meta.description.is_empty() {
        String::new()
    } else {
        format!(r#"    <meta name="description" content="{}">"#, config.meta.description)
    };

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
{}{}    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div id="app"></div>
    <script src="app.js"></script>
</body>
</html>"#,
        lang,
        title,
        if description_meta.is_empty() { String::new() } else { format!("{}\n", description_meta) },
        if favicon_link.is_empty() { String::new() } else { format!("{}\n", favicon_link) },
    )
}
