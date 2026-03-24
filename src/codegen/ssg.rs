use std::collections::HashMap;
use crate::parser::ast::*;
use crate::config::ProjectConfig;

/// Renders a page to static HTML for SSG.
pub fn render_page_html(
    page: &PageDecl,
    config: &ProjectConfig,
    app_body: Option<&[Statement]>,
    translations: &HashMap<String, HashMap<String, String>>,
) -> String {
    let title = page.title.as_deref().unwrap_or(&config.name);
    let lang = if config.meta.lang.is_empty() { "en" } else { &config.meta.lang };

    let default_locale = config.i18n.as_ref()
        .map(|i| i.default_locale.as_str())
        .unwrap_or("en");

    let default_messages = translations.get(default_locale)
        .cloned()
        .unwrap_or_default();

    let mut ctx = SsgContext {
        default_messages,
        indent: 2,
    };

    // Render app shell (navbar, etc.) if available
    let mut body_html = String::new();
    if let Some(app_stmts) = app_body {
        for stmt in app_stmts {
            // Render pre-router elements (like Navbar)
            if let Statement::UIElement(ui) = stmt {
                if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router") {
                    // Router content — render the specific page
                    body_html.push_str(&render_statements(&page.body, &mut ctx));
                } else {
                    body_html.push_str(&render_ui_element(ui, &mut ctx));
                }
            }
        }
    } else {
        body_html = render_statements(&page.body, &mut ctx);
    }

    let description_meta = if config.meta.description.is_empty() {
        String::new()
    } else {
        format!("    <meta name=\"description\" content=\"{}\">\n", config.meta.description)
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
{}    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div id="app">
{}    </div>
    <script src="/app.js"></script>
</body>
</html>"#,
        lang, title, description_meta, body_html
    )
}

struct SsgContext {
    default_messages: HashMap<String, String>,
    indent: usize,
}

impl SsgContext {
    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }
}

fn render_statements(stmts: &[Statement], ctx: &mut SsgContext) -> String {
    let mut html = String::new();
    for stmt in stmts {
        match stmt {
            Statement::UIElement(ui) => html.push_str(&render_ui_element(ui, ctx)),
            Statement::If(_) => {
                // Dynamic — emit placeholder comment
                html.push_str(&format!("{}<!--wf-if-->\n", ctx.indent_str()));
            }
            Statement::For(_) => {
                html.push_str(&format!("{}<!--wf-for-->\n", ctx.indent_str()));
            }
            Statement::Show(show) => {
                // Render content but hidden
                let inner = render_statements(&show.body, ctx);
                html.push_str(&format!(
                    "{}<div style=\"display:none\">\n{}{}</div>\n",
                    ctx.indent_str(), inner, ctx.indent_str()
                ));
            }
            Statement::Fetch(fetch) => {
                // Render loading block if present
                if let Some(loading) = &fetch.loading_block {
                    html.push_str(&render_statements(loading, ctx));
                } else {
                    html.push_str(&format!("{}<!--wf-fetch-->\n", ctx.indent_str()));
                }
            }
            // Skip state, derived, effect, action, use, events, navigate, log, animate
            _ => {}
        }
    }
    html
}

fn render_ui_element(ui: &UIElement, ctx: &mut SsgContext) -> String {
    match &ui.component {
        ComponentRef::BuiltIn(name) => render_builtin(name, ui, ctx),
        ComponentRef::SubComponent(parent, sub) => {
            let class = format!("wf-{}__{}",
                parent.to_lowercase(),
                camel_to_kebab(sub)
            );
            let tag = match sub.as_str() {
                "Item" => "li",
                _ => "div",
            };
            render_tag(tag, &class, ui, ctx)
        }
        ComponentRef::UserDefined(_name) => {
            // Can't pre-render user components without expanding them
            // Emit a placeholder div
            let indent = ctx.indent_str();
            format!("{}<!--wf-component-->\n", indent)
        }
    }
}

fn render_builtin(name: &str, ui: &UIElement, ctx: &mut SsgContext) -> String {
    let (tag, base_class) = builtin_to_html_tag(name);

    // Build class string
    let mut classes = vec![base_class.to_string()];
    for m in &ui.modifiers {
        let mc = modifier_to_css_class(base_class, m);
        if !mc.is_empty() {
            classes.push(mc);
        }
    }
    let class_str = classes.iter().filter(|c| !c.is_empty()).cloned().collect::<Vec<_>>().join(" ");

    // Special handling for certain components
    match name {
        "Spacer" => {
            return format!("{}<div class=\"{}\"></div>\n", ctx.indent_str(), class_str);
        }
        "Divider" => {
            return format!("{}<hr class=\"{}\">\n", ctx.indent_str(), class_str);
        }
        "Spinner" => {
            return format!("{}<div class=\"{}\"></div>\n", ctx.indent_str(), class_str);
        }
        "Children" | "_StyleBlock" | "Router" | "Route" => {
            return String::new();
        }
        "Toast" => return String::new(), // Imperative, no SSG output
        _ => {}
    }

    // Extract attributes and text content
    let mut attrs = Vec::new();
    let mut text_content: Option<String> = None;

    if !class_str.is_empty() {
        attrs.push(format!("class=\"{}\"", class_str));
    }

    for arg in &ui.args {
        match arg {
            Arg::Named(key, val) => {
                match key.as_str() {
                    "src" | "alt" | "href" | "placeholder" | "type" | "min" | "max" |
                    "step" | "accept" | "role" | "value" => {
                        if let Some(s) = expr_to_static_string(val) {
                            attrs.push(format!("{}=\"{}\"", key, html_escape(&s)));
                        }
                    }
                    "to" => {
                        if let Some(s) = expr_to_static_string(val) {
                            attrs.push(format!("href=\"{}\"", html_escape(&s)));
                        }
                    }
                    "required" => attrs.push("required".to_string()),
                    "disabled" => attrs.push("disabled".to_string()),
                    "controls" => attrs.push("controls".to_string()),
                    "title" => {
                        if let Some(s) = expr_to_static_string(val) {
                            attrs.push(format!("title=\"{}\"", html_escape(&s)));
                        }
                    }
                    "label" => {
                        // For checkbox/radio/switch/slider, the label is visible text
                        if let Some(s) = expr_to_static_string(val) {
                            text_content = Some(s);
                        }
                    }
                    "columns" => {
                        if let Expr::NumberLiteral(n) = val {
                            attrs.push(format!("style=\"grid-template-columns: repeat({}, 1fr)\"", *n as i32));
                        }
                    }
                    "visible" | "bind" | "checked" | "icon" | "span" |
                    "gap" | "align" | "justify" => {} // Skip runtime-only attrs
                    _ => {}
                }
            }
            Arg::Positional(expr) => {
                if text_content.is_none() {
                    text_content = resolve_text(expr, &ctx.default_messages);
                }
            }
        }
    }

    // Handle input type from modifiers
    for m in &ui.modifiers {
        match m.as_str() {
            "text" | "email" | "password" | "number" | "search" | "tel" | "url" |
            "date" | "time" | "color" => {
                let t = if m == "datetime" { "datetime-local" } else { m.as_str() };
                attrs.push(format!("type=\"{}\"", t));
            }
            "submit" | "reset" => attrs.push(format!("type=\"{}\"", m)),
            _ => {}
        }
    }

    // Heading tag override based on modifier
    let actual_tag = if name == "Heading" {
        heading_tag(&ui.modifiers)
    } else {
        tag
    };

    let indent = ctx.indent_str();
    let attrs_str = if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    };

    // Self-closing tags
    if matches!(actual_tag, "input" | "img" | "hr" | "br") {
        return format!("{}<{}{}>\n", indent, actual_tag, attrs_str);
    }

    // Has children?
    let has_children = !ui.children.is_empty();
    let has_text = text_content.is_some();

    if !has_children && !has_text {
        return format!("{}<{}{}></{}>​\n", indent, actual_tag, attrs_str, actual_tag);
    }

    let mut result = format!("{}<{}{}>\n", indent, actual_tag, attrs_str);

    if let Some(text) = &text_content {
        // Inline text
        if !has_children {
            return format!("{}<{}{}>{}</{}>​\n", indent, actual_tag, attrs_str, html_escape(text), actual_tag);
        }
        result.push_str(&format!("{}    {}\n", indent, html_escape(text)));
    }

    ctx.indent += 1;
    result.push_str(&render_statements(&ui.children, ctx));
    ctx.indent -= 1;

    result.push_str(&format!("{}</{}>​\n", indent, actual_tag));
    result
}

fn render_tag(tag: &str, class: &str, ui: &UIElement, ctx: &mut SsgContext) -> String {
    let indent = ctx.indent_str();
    let mut result = format!("{}<{} class=\"{}\">\n", indent, tag, class);
    ctx.indent += 1;
    result.push_str(&render_statements(&ui.children, ctx));
    ctx.indent -= 1;
    result.push_str(&format!("{}</{}>​\n", indent, tag));
    result
}

/// Try to resolve an expression to a static string.
fn expr_to_static_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::StringLiteral(s) => Some(s.clone()),
        Expr::NumberLiteral(n) => Some(format!("{}", n)),
        Expr::BoolLiteral(b) => Some(format!("{}", b)),
        _ => None, // Dynamic — can't resolve
    }
}

/// Resolve text content, including i18n t() calls.
fn resolve_text(expr: &Expr, messages: &HashMap<String, String>) -> Option<String> {
    match expr {
        Expr::StringLiteral(s) => Some(s.clone()),
        Expr::NumberLiteral(n) => {
            if *n == (*n as i64) as f64 {
                Some(format!("{}", *n as i64))
            } else {
                Some(format!("{}", n))
            }
        }
        Expr::BoolLiteral(b) => Some(format!("{}", b)),
        Expr::FunctionCall(name, args) if name == "t" => {
            // i18n: resolve from default locale
            if let Some(Expr::StringLiteral(key)) = args.first() {
                messages.get(key).cloned().or_else(|| Some(key.clone()))
            } else {
                None
            }
        }
        _ => None, // Dynamic expression — leave empty for client
    }
}

fn heading_tag(modifiers: &[String]) -> &'static str {
    for m in modifiers {
        match m.as_str() {
            "h1" => return "h1",
            "h2" => return "h2",
            "h3" => return "h3",
            "h4" => return "h4",
            "h5" => return "h5",
            "h6" => return "h6",
            _ => {}
        }
    }
    "h2"
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

fn builtin_to_html_tag(name: &str) -> (&'static str, &'static str) {
    match name {
        "Container" => ("div", "wf-container"),
        "Row" => ("div", "wf-row"),
        "Column" => ("div", "wf-col"),
        "Grid" => ("div", "wf-grid"),
        "Stack" => ("div", "wf-stack"),
        "Spacer" => ("div", "wf-spacer"),
        "Divider" => ("hr", "wf-divider"),
        "Navbar" => ("nav", "wf-navbar"),
        "Sidebar" => ("aside", "wf-sidebar"),
        "Breadcrumb" => ("nav", "wf-breadcrumb"),
        "Link" => ("a", "wf-link"),
        "Menu" => ("div", "wf-menu"),
        "Tabs" => ("div", "wf-tabs"),
        "TabPage" => ("div", "wf-tab-page"),
        "Card" => ("div", "wf-card"),
        "Table" => ("table", "wf-table"),
        "Thead" => ("thead", ""),
        "Tbody" => ("tbody", ""),
        "Trow" => ("tr", ""),
        "Tcell" => ("td", ""),
        "List" => ("ul", "wf-list"),
        "Badge" => ("span", "wf-badge"),
        "Avatar" => ("div", "wf-avatar"),
        "Tooltip" => ("div", "wf-tooltip"),
        "Tag" => ("span", "wf-tag"),
        "Input" => ("input", "wf-input"),
        "Select" => ("select", "wf-select"),
        "Option" => ("option", ""),
        "Checkbox" => ("label", "wf-checkbox"),
        "Radio" => ("label", "wf-radio"),
        "Switch" => ("label", "wf-switch"),
        "Slider" => ("input", "wf-slider"),
        "DatePicker" => ("input", "wf-datepicker"),
        "FileUpload" => ("input", "wf-file-upload"),
        "Form" => ("form", "wf-form"),
        "Alert" => ("div", "wf-alert"),
        "Toast" => ("div", "wf-toast"),
        "Modal" => ("div", "wf-modal"),
        "Dialog" => ("div", "wf-dialog"),
        "Spinner" => ("div", "wf-spinner"),
        "Progress" => ("progress", "wf-progress"),
        "Skeleton" => ("div", "wf-skeleton"),
        "Button" => ("button", "wf-btn"),
        "IconButton" => ("button", "wf-icon-btn"),
        "ButtonGroup" => ("div", "wf-btn-group"),
        "Dropdown" => ("div", "wf-dropdown"),
        "Image" => ("img", "wf-image"),
        "Video" => ("video", "wf-video"),
        "Icon" => ("i", "wf-icon"),
        "Carousel" => ("div", "wf-carousel"),
        "Text" => ("p", "wf-text"),
        "Heading" => ("h2", "wf-heading"),
        "Code" => ("code", "wf-code"),
        "Blockquote" => ("blockquote", "wf-blockquote"),
        _ => ("div", ""),
    }
}

fn modifier_to_css_class(base_class: &str, modifier: &str) -> String {
    match modifier {
        "small" => format!("{}--small", base_class),
        "medium" => String::new(),
        "large" => format!("{}--large", base_class),
        "primary" => format!("{}--primary", base_class),
        "secondary" => format!("{}--secondary", base_class),
        "success" => format!("{}--success", base_class),
        "danger" => format!("{}--danger", base_class),
        "warning" => format!("{}--warning", base_class),
        "info" => format!("{}--info", base_class),
        "rounded" => format!("{}--rounded", base_class),
        "pill" => format!("{}--pill", base_class),
        "flat" => format!("{}--flat", base_class),
        "elevated" => format!("{}--elevated", base_class),
        "outlined" => format!("{}--outlined", base_class),
        "full" => format!("{}--full", base_class),
        "bold" => "wf-text--bold".to_string(),
        "italic" => "wf-text--italic".to_string(),
        "center" => "wf-text--center".to_string(),
        "heading" => "wf-text--heading".to_string(),
        "subtitle" => "wf-text--subtitle".to_string(),
        "muted" => "wf-text--muted".to_string(),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => format!("wf-heading--{}", modifier),
        "dismissible" => format!("{}--dismissible", base_class),
        "fluid" => format!("{}--fluid", base_class),
        "fadeIn" | "fadeOut" | "slideUp" | "slideDown" | "slideLeft" | "slideRight" |
        "scaleIn" | "scaleOut" | "bounce" | "shake" | "pulse" | "spin" => {
            format!("wf-animate-{}", modifier)
        }
        "fast" => "wf-animate--fast".to_string(),
        "slow" => "wf-animate--slow".to_string(),
        _ => String::new(),
    }
}

fn camel_to_kebab(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}
