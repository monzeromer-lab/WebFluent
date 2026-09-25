//! `output_type: "elements"` — the project as custom elements.
//!
//! A component in this language is a function that builds nodes. A custom
//! element is the platform's name for the same thing, and every framework
//! can place one: React, Vue, Svelte, Angular, Rails, WordPress, a plain
//! HTML page. So the answer to "how do I use this from React" is not an
//! adapter per framework — it is the one interface all of them already
//! speak.
//!
//! What a build writes: `elements.js`, holding the runtime, the project's
//! components and stores, and a `customElements.define` per name in
//! `build.elements`. Each element maps its observed attributes to the
//! component's props, re-renders when one changes, and cleans up when it
//! leaves the document.

use crate::config::ProjectConfig;
use crate::parser::ast::{ComponentDecl, Declaration, Program, TypeRef};

/// `PriceTag` as a tag name is `price-tag`.
///
/// A custom element's name must hold a hyphen, so a one-word component is
/// given a prefix rather than being refused: `Rating` is `<wf-rating>`.
pub fn tag_name(component: &str) -> String {
    let mut out = String::with_capacity(component.len() + 4);
    for (i, c) in component.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('-');
        }
        out.push(c.to_ascii_lowercase());
    }
    if out.contains('-') {
        out
    } else {
        format!("wf-{out}")
    }
}

/// `publishableKey` as an attribute is `publishable-key`.
fn attribute_name(prop: &str) -> String {
    tag_name(prop).trim_start_matches("wf-").to_string()
}

/// The components a build publishes, and what each one is missing.
///
/// A name in `build.elements` that the project does not declare is the one
/// thing this cannot paper over.
pub fn check(program: &Program, config: &ProjectConfig) -> Vec<String> {
    let declared: Vec<&str> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) => Some(c.name.as_str()),
            _ => None,
        })
        .collect();
    let mut out = Vec::new();
    if config.build.elements.is_empty() {
        out.push(
            "`output_type: \"elements\"` publishes nothing: name the components in `build.elements`"
                .to_string(),
        );
    }
    for name in &config.build.elements {
        if !declared.contains(&name.as_str()) {
            out.push(format!(
                "`build.elements` names `{name}`, which the project does not declare as a component"
            ));
        }
    }
    out
}

/// The `customElements.define` calls for the components a build publishes,
/// appended to a bundle that already holds the runtime and the components.
pub fn definitions(program: &Program, config: &ProjectConfig) -> String {
    let components: Vec<&ComponentDecl> = config
        .build
        .elements
        .iter()
        .filter_map(|name| {
            program.declarations.iter().find_map(|d| match d {
                Declaration::Component(c) if &c.name == name => Some(c),
                _ => None,
            })
        })
        .collect();
    if components.is_empty() {
        return String::new();
    }

    let mut out = String::from(
        "\n// ─── Custom elements ─────────────────────────────────\n\
         //\n\
         // Each component this build publishes, as a tag any framework can\n\
         // place. An attribute is a prop: it is read when the element is\n\
         // connected and again whenever it changes, and the element is\n\
         // rebuilt from what the attributes now say. What the component\n\
         // created — effects, timers, listeners — goes when the element\n\
         // leaves the document, because the runtime's own scope owns it.\n",
    );
    for component in &components {
        let tag = tag_name(&component.name);
        let props: Vec<(String, &TypeRef, bool)> = component
            .props
            .iter()
            .map(|p| (attribute_name(&p.name), &p.prop_type, p.positional))
            .collect();
        let observed: Vec<String> = props
            .iter()
            .map(|(attr, _, _)| format!("\"{attr}\""))
            .collect();
        // Reading each attribute back into the prop it came from, in the
        // type the component declared.
        let reads: Vec<String> = component
            .props
            .iter()
            .map(|p| {
                let attr = attribute_name(&p.name);
                let raw = format!("this.getAttribute(\"{attr}\")");
                let value = match &p.prop_type {
                    TypeRef::Number => format!("({raw} === null ? undefined : Number({raw}))"),
                    TypeRef::Bool => {
                        // An attribute that is present is true, as HTML
                        // reads `disabled` — and `="false"` is false, so a
                        // framework that writes the string still works.
                        format!("(this.hasAttribute(\"{attr}\") ? {raw} !== \"false\" : undefined)")
                    }
                    TypeRef::Map | TypeRef::List(_) => {
                        format!("WF.jsonAttr({raw})")
                    }
                    _ => format!("({raw} === null ? undefined : {raw})"),
                };
                format!("      {}: {},", p.name, value)
            })
            .collect();
        // An event the component declares becomes a DOM event the host
        // page listens for with `addEventListener`, which is what every
        // framework's `onReady` binds to underneath.
        let events: Vec<String> = component
            .events
            .iter()
            .map(|e| {
                format!(
                    "      {}: (...args) => this.dispatchEvent(new CustomEvent(\"{}\", {{ detail: args.length > 1 ? args : args[0], bubbles: true }})),",
                    e.name, e.name
                )
            })
            .collect();
        out.push_str(&format!(
            r#"
class {class}Element extends HTMLElement {{
  static get observedAttributes() {{ return [{observed}]; }}
  connectedCallback() {{ this._render(); }}
  attributeChangedCallback() {{ if (this.isConnected) this._render(); }}
  disconnectedCallback() {{ this._dispose && this._dispose(); this._dispose = null; }}
  _render() {{
    this._dispose && this._dispose();
    const props = {{
{reads}
      on: {{
{events}
      }},
    }};
    const [node, dispose] = WF.scoped(() => Component_{class}(props));
    this._dispose = dispose;
    this.replaceChildren(node);
  }}
}}
if (!customElements.get("{tag}")) customElements.define("{tag}", {class}Element);
"#,
            class = component.name,
            tag = tag,
            observed = observed.join(", "),
            reads = reads.join("\n"),
            events = events.join("\n"),
        ));
    }
    out
}

/// The page a build writes beside `elements.js`: every tag it published,
/// with the attributes it takes, so the file is usable without reading the
/// source.
pub fn demo_page(program: &Program, config: &ProjectConfig, base: &str) -> String {
    let mut rows = String::new();
    for name in &config.build.elements {
        let Some(component) = program.declarations.iter().find_map(|d| match d {
            Declaration::Component(c) if &c.name == name => Some(c),
            _ => None,
        }) else {
            continue;
        };
        let tag = tag_name(name);
        let attrs: Vec<String> = component
            .props
            .iter()
            .map(|p| format!("{}=\"…\"", attribute_name(&p.name)))
            .collect();
        let events: Vec<String> = component.events.iter().map(|e| e.name.clone()).collect();
        rows.push_str(&format!(
            "    <section>\n      <h2><code>&lt;{tag}&gt;</code></h2>\n\
             \x20     <pre><code>&lt;{tag} {}&gt;&lt;/{tag}&gt;</code></pre>\n{}    </section>\n",
            attrs.join(" "),
            if events.is_empty() {
                String::new()
            } else {
                format!(
                    "      <p>Events: {}</p>\n",
                    events
                        .iter()
                        .map(|e| format!("<code>{e}</code>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        ));
    }
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name} — custom elements</title>
    <link rel="stylesheet" href="{base}/styles.css">
    <script src="{base}/elements.js" defer></script>
</head>
<body>
    <h1>{name}</h1>
    <p>Load <code>elements.js</code> and <code>styles.css</code>, then place any
    of these tags — from React, Vue, Svelte, Rails, WordPress or a plain page.</p>
{rows}</body>
</html>
"#,
        name = config.name,
        base = base,
        rows = rows,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_component_name_becomes_a_tag_a_browser_accepts() {
        assert_eq!(tag_name("PriceTag"), "price-tag");
        assert_eq!(tag_name("UserCard"), "user-card");
        // A custom element's name must hold a hyphen, so a one-word
        // component is prefixed rather than refused.
        assert_eq!(tag_name("Rating"), "wf-rating");
        assert_eq!(tag_name("HTMLish"), "h-t-m-lish");
    }

    #[test]
    fn a_prop_becomes_the_attribute_a_framework_would_write() {
        assert_eq!(attribute_name("publishableKey"), "publishable-key");
        assert_eq!(attribute_name("title"), "title");
    }
}
