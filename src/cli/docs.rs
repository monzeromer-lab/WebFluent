//! `wf docs [-d DIR] [-o OUT]`: a component gallery — every built-in with
//! its props, cases, flags, events, slots and parts, and what the project
//! declares: components with their props, events, slots and parts, enums
//! with their cases, records with their fields, stores with their members,
//! pages with their routes — written as one self-contained HTML page,
//! read from the same registry the compiler and the editor read.

use std::path::Path;

use serde_json::Value;

use crate::error::Result;

pub fn run_docs(project_dir: &Path, out: &Path) -> Result<()> {
    let registry = crate::registry::json::registry_json();
    let project = match super::build::read_project(project_dir) {
        Ok((program, files)) => {
            let file_of = |index: usize| files.get(index).cloned().unwrap_or_default();
            Some((
                crate::registry::json::declarations_json(&program, &file_of),
                project_dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "project".to_string()),
            ))
        }
        Err(_) => None,
    };
    let html = render(&registry, project.as_ref());
    std::fs::create_dir_all(out)?;
    let file = out.join("index.html");
    std::fs::write(&file, html)?;
    println!(
        "  docs: {} built-in components{} → {}",
        registry["components"].as_array().map_or(0, |c| c.len()),
        project
            .as_ref()
            .map(|(p, _)| {
                format!(
                    ", {} of the project's",
                    p["components"].as_array().map_or(0, |c| c.len())
                )
            })
            .unwrap_or_default(),
        file.display()
    );
    Ok(())
}

fn esc(text: &str) -> String {
    crate::codegen::markdown::escape(text)
}

fn s(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// The page.
pub fn render(registry: &Value, project: Option<&(Value, String)>) -> String {
    let mut body = String::new();
    let components = registry["components"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    // Groups in the registry's order.
    let mut groups: Vec<String> = Vec::new();
    for c in &components {
        let g = s(&c["group"]);
        if c["owner"].is_null() && !groups.contains(&g) {
            groups.push(g);
        }
    }
    let mut nav = String::new();
    for g in &groups {
        nav.push_str(&format!("<a href=\"#g-{}\">{}</a>", slug(g), esc(g)));
    }
    if project.is_some() {
        nav.push_str("<a href=\"#project\">This project</a>");
    }
    body.push_str(&format!("<nav>{nav}</nav>\n"));

    body.push_str("<section id=\"builtins\"><h1>Built-in components</h1>\n");
    body.push_str(&format!(
        "<p class=\"muted\">WebFluent {}. Every prop, case, flag, event, slot and part below is what the compiler accepts and the editor completes.</p>\n",
        esc(&s(&registry["version"]))
    ));
    for g in &groups {
        body.push_str(&format!("<h2 id=\"g-{}\">{}</h2>\n", slug(g), esc(g)));
        for c in components
            .iter()
            .filter(|c| s(&c["group"]) == *g && c["owner"].is_null())
        {
            body.push_str(&component_card(c, &components));
        }
    }
    body.push_str("<h2 id=\"universal\">On every element</h2>\n");
    body.push_str(
        "<div class=\"card\"><table><tr><th>Prop</th><th>Type</th><th>What it does</th></tr>",
    );
    for p in registry["universal"]["props"]
        .as_array()
        .into_iter()
        .flatten()
    {
        body.push_str(&prop_row(p));
    }
    body.push_str("</table><p class=\"muted\">Events: ");
    body.push_str(
        &registry["universal"]["events"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|e| format!("<code>on {}</code>", esc(&s(&e["name"]))))
            .collect::<Vec<_>>()
            .join(", "),
    );
    body.push_str("</p></div>\n</section>\n");

    if let Some((decls, name)) = project {
        body.push_str(&format!("<section id=\"project\"><h1>{}</h1>\n", esc(name)));
        let section =
            |body: &mut String, title: &str, key: &str, render: &dyn Fn(&Value) -> String| {
                let items = decls[key].as_array().cloned().unwrap_or_default();
                if items.is_empty() {
                    return;
                }
                body.push_str(&format!("<h2>{title}</h2>\n"));
                for item in &items {
                    body.push_str(&render(item));
                }
            };
        section(&mut body, "Components", "components", &|c| {
            let mut card = format!(
                "<div class=\"card\"><h3>{}{}</h3>{}",
                esc(&s(&c["name"])),
                c["positional"]
                    .as_str()
                    .map(|p| format!(" <span class=\"muted\">(_ {p})</span>"))
                    .unwrap_or_default(),
                doc(&c["doc"])
            );
            let props = c["props"].as_array().cloned().unwrap_or_default();
            if !props.is_empty() {
                card.push_str("<table><tr><th>Prop</th><th>Type</th><th>Default</th></tr>");
                for p in &props {
                    card.push_str(&format!(
                        "<tr><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                        esc(&s(&p["name"])),
                        esc(&s(&p["type"])),
                        esc(&s(&p["default"]))
                    ));
                }
                card.push_str("</table>");
            }
            let list = |label: &str, key: &str, f: &dyn Fn(&Value) -> String| {
                let items = c[key].as_array().cloned().unwrap_or_default();
                if items.is_empty() {
                    return String::new();
                }
                format!(
                    "<p><b>{label}:</b> {}</p>",
                    items.iter().map(f).collect::<Vec<_>>().join(", ")
                )
            };
            card.push_str(&list("Events", "events", &|e| {
                format!("<code>on {}</code>", esc(&s(&e["name"])))
            }));
            card.push_str(&list("Slots", "slots", &|sl| {
                format!("<code>{}</code>", esc(&s(sl)))
            }));
            card.push_str(&list("Parts", "parts", &|p| {
                format!("<code>{}.{}</code>", esc(&s(&c["name"])), esc(&s(p)))
            }));
            card.push_str(&format!(
                "<p class=\"muted\">{}:{}</p></div>\n",
                esc(&s(&c["file"])),
                s(&c["line"])
            ));
            card
        });
        section(&mut body, "Enums", "enums", &|e| {
            format!(
                "<div class=\"card\"><h3>{}</h3>{}<p>{}</p></div>\n",
                esc(&s(&e["name"])),
                doc(&e["doc"]),
                e["cases"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|c| format!("<code>.{}</code>", esc(&s(c))))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        });
        section(&mut body, "Types", "types", &|t| {
            let mut card = format!(
                "<div class=\"card\"><h3>{}{}</h3>{}<table><tr><th>Field</th><th>Type</th><th>Default</th></tr>",
                esc(&s(&t["name"])),
                t["extends"]
                    .as_str()
                    .map(|b| format!(" <span class=\"muted\">= {b}</span>"))
                    .unwrap_or_default(),
                doc(&t["doc"])
            );
            for f in t["fields"].as_array().into_iter().flatten() {
                card.push_str(&format!(
                    "<tr><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                    esc(&s(&f["name"])),
                    esc(&s(&f["type"])),
                    esc(&s(&f["default"]))
                ));
            }
            card.push_str("</table></div>\n");
            card
        });
        section(&mut body, "Stores", "stores", &|st| {
            format!(
                "<div class=\"card\"><h3>{}</h3><p>{}</p></div>\n",
                esc(&s(&st["name"])),
                st["members"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|m| format!(
                        "<code>{}</code> <span class=\"muted\">{}</span>",
                        esc(&s(&m["name"])),
                        esc(&s(&m["kind"]))
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        });
        section(&mut body, "Pages", "pages", &|p| {
            format!(
                "<div class=\"card\"><h3>{} <span class=\"muted\">{}</span></h3><p>{}</p></div>\n",
                esc(&s(&p["name"])),
                esc(&s(&p["path"])),
                esc(&s(&p["title"]))
            )
        });
        body.push_str("</section>\n");
    }

    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>WebFluent components</title>\n<style>{CSS}</style>\n</head>\n<body>\n{body}</body>\n</html>\n"
    )
}

fn component_card(c: &Value, all: &[Value]) -> String {
    let name = s(&c["name"]);
    let mut card = format!(
        "<div class=\"card\" id=\"c-{}\"><h3>{}{}</h3><p>{}</p>",
        slug(&name),
        esc(&name),
        c["positional"]
            .as_object()
            .map(|p| format!(
                " <span class=\"muted\">({}: {})</span>",
                esc(&s(&p["name"])),
                esc(&s(&p["type"]))
            ))
            .unwrap_or_default(),
        esc(&s(&c["summary"]))
    );
    let props = c["props"].as_array().cloned().unwrap_or_default();
    if !props.is_empty() {
        card.push_str("<table><tr><th>Prop</th><th>Type</th><th>What it does</th></tr>");
        for p in &props {
            card.push_str(&prop_row(p));
        }
        card.push_str("</table>");
    }
    let flags: Vec<String> = c["flags"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|f| format!("<code>.{}</code>", esc(&s(&f["name"]))))
        .collect();
    if !flags.is_empty() {
        card.push_str(&format!("<p><b>Flags:</b> {}</p>", flags.join(" ")));
    }
    let events: Vec<String> = c["events"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|e| format!("<code>on {}</code>", esc(&s(e))))
        .collect();
    if !events.is_empty() {
        card.push_str(&format!("<p><b>Events:</b> {}</p>", events.join(", ")));
    }
    let parts: Vec<&Value> = all
        .iter()
        .filter(|p| p["owner"].as_str() == Some(name.as_str()))
        .collect();
    if !parts.is_empty() {
        card.push_str("<p><b>Parts:</b></p>");
        for p in parts {
            card.push_str(&format!(
                "<div class=\"part\"><code>{}.{}</code> — {}",
                esc(&name),
                esc(&s(&p["name"])),
                esc(&s(&p["summary"]))
            ));
            let pprops = p["props"].as_array().cloned().unwrap_or_default();
            if !pprops.is_empty() {
                card.push_str("<table>");
                for pp in &pprops {
                    card.push_str(&prop_row(pp));
                }
                card.push_str("</table>");
            }
            card.push_str("</div>");
        }
    }
    if s(&c["children"]) == "elements" {
        card.push_str("<p class=\"muted\">Takes a block of children.</p>");
    }
    card.push_str("</div>\n");
    card
}

fn prop_row(p: &Value) -> String {
    let cases = p["cases"]
        .as_array()
        .map(|cs| {
            cs.iter()
                .map(|c| format!("<code>.{}</code>", esc(&s(&c["name"]))))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    format!(
        "<tr><td><code>{}</code></td><td>{}{}</td><td>{}</td></tr>",
        esc(&s(&p["name"])),
        esc(&s(&p["type"])),
        if cases.is_empty() {
            String::new()
        } else {
            format!(" {cases}")
        },
        esc(&s(&p["summary"]))
    )
}

fn doc(v: &Value) -> String {
    v.as_str()
        .map(|d| format!("<p>{}</p>", esc(d)))
        .unwrap_or_default()
}

fn slug(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

const CSS: &str = r#"
:root { --ink: #1a1a1a; --muted: #6b7280; --line: #e5e7eb; --bg: #fff; --card: #f9fafb; --accent: #2563eb; }
@media (prefers-color-scheme: dark) { :root { --ink: #e5e7eb; --muted: #9ca3af; --line: #374151; --bg: #111827; --card: #1f2937; --accent: #60a5fa; } }
* { box-sizing: border-box; }
body { margin: 0; font: 15px/1.55 system-ui, sans-serif; color: var(--ink); background: var(--bg); }
nav { position: sticky; top: 0; display: flex; flex-wrap: wrap; gap: 4px 14px; padding: 12px 24px; background: var(--bg); border-bottom: 1px solid var(--line); }
nav a { color: var(--accent); text-decoration: none; }
section { max-width: 960px; margin: 0 auto; padding: 24px; }
h1 { font-size: 28px; margin: 24px 0 8px; } h2 { font-size: 20px; margin: 32px 0 12px; padding-top: 8px; border-top: 1px solid var(--line); } h3 { margin: 0 0 6px; font-size: 17px; }
.card { background: var(--card); border: 1px solid var(--line); border-radius: 8px; padding: 14px 16px; margin: 12px 0; }
.card p { margin: 6px 0; } .muted { color: var(--muted); }
table { border-collapse: collapse; width: 100%; margin: 8px 0; font-size: 14px; }
th { text-align: left; color: var(--muted); font-weight: 500; } td, th { padding: 4px 8px 4px 0; border-top: 1px solid var(--line); vertical-align: top; }
code { font: 13px ui-monospace, monospace; background: rgba(127,127,127,.12); padding: 1px 5px; border-radius: 4px; }
.part { margin: 6px 0 6px 12px; padding-left: 12px; border-left: 2px solid var(--line); }
"#;
