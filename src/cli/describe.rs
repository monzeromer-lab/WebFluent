use std::path::Path;

use crate::diagnostics::error::Result;

/// `wf registry --json`: every built-in, as a tool reads it.
#[expect(
    clippy::unnecessary_wraps,
    reason = "a command returns a Result as every other does, so `run` dispatches them alike"
)]
pub(crate) fn run_registry(json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&crate::registry::json::registry_json())
                .expect("a JSON value always serialises")
        );
        return Ok(());
    }
    for sig in crate::registry::components() {
        let flags: Vec<String> = sig
            .all_props()
            .filter(|p| p.shorthand)
            .flat_map(|p| match p.ty {
                crate::registry::PropType::Bool => vec![format!(".{}", p.name)],
                crate::registry::PropType::Enum(cases) => cases
                    .iter()
                    .filter(|c| matches!(sig.flag(c.name), crate::registry::Flag::Case(..)))
                    .map(|c| format!(".{}", c.name))
                    .collect(),
                _ => Vec::new(),
            })
            .collect();
        let props: Vec<&str> = sig.props.iter().map(|p| p.name).collect();
        println!(
            "{:<14} {:<12} {}{}",
            sig.name,
            sig.group,
            sig.positional
                .as_ref()
                .map(|p| format!("({}) ", p.name))
                .unwrap_or_default(),
            sig.summary
        );
        if !props.is_empty() {
            println!("               props: {}", props.join(", "));
        }
        if !flags.is_empty() {
            println!("               flags: {}", flags.join(" "));
        }
        if !sig.parts.is_empty() {
            println!("               parts: {}", sig.parts.join(", "));
        }
    }
    Ok(())
}

/// `wf types --holes [--json] [path]`: every place a value's type is
/// unknown, with why, and the count of each kind.
pub(crate) fn run_holes(project_dir: &Path, json: bool) -> Result<()> {
    use crate::check::sema::types::{HoleKind, holes_of};
    let holes = holes_of(project_dir)?;
    let count = |kind: HoleKind| holes.iter().filter(|h| h.kind == kind).count();
    if json {
        let kinds: serde_json::Map<String, serde_json::Value> = HoleKind::ALL
            .iter()
            .map(|k| (k.name().to_string(), serde_json::json!(count(*k))))
            .collect();
        let list: Vec<serde_json::Value> = holes
            .iter()
            .map(|h| {
                serde_json::json!({
                    "file": h.file,
                    "line": h.line,
                    "column": h.column,
                    "kind": h.kind.name(),
                    "what": h.what,
                })
            })
            .collect();
        let value = serde_json::json!({ "total": holes.len(), "kinds": kinds, "holes": list });
        println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("a JSON value always serialises")
        );
        return Ok(());
    }
    for h in &holes {
        println!(
            "  {:<44} {:<18} {}",
            format!("{}:{}:{}", h.file, h.line, h.column),
            h.kind.name(),
            h.what
        );
    }
    let files: std::collections::BTreeSet<&str> = holes.iter().map(|h| h.file.as_str()).collect();
    println!(
        "\n  {} place(s) where a type is unknown, in {} file(s)",
        holes.len(),
        files.len()
    );
    for kind in HoleKind::ALL {
        let n = count(kind);
        if n > 0 {
            println!("    {:<18} {n:>5}  {}", kind.name(), kind.describe());
        }
    }
    Ok(())
}

/// `wf types --json [path]`: what a project declares, as a tool reads it.
pub(crate) fn run_types(project_dir: &Path, json: bool) -> Result<()> {
    let (program, files) = super::build::read_project(project_dir)?;
    let file_of = |index: usize| {
        files
            .get(index)
            .cloned()
            .unwrap_or_else(|| "src/".to_string())
    };
    let value = crate::registry::json::declarations_json(&program, &file_of);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("a JSON value always serialises")
        );
        return Ok(());
    }
    for (kind, key) in [
        ("enum", "enums"),
        ("type", "types"),
        ("component", "components"),
        ("store", "stores"),
        ("page", "pages"),
        ("script", "scripts"),
    ] {
        for item in value[key].as_array().into_iter().flatten() {
            let name = item["name"].as_str().unwrap_or("");
            let detail = match kind {
                "enum" => item["cases"]
                    .as_array()
                    .map(|c| {
                        c.iter()
                            .filter_map(|x| x.as_str())
                            .map(|x| format!(".{x}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default(),
                "type" => item["fields"]
                    .as_array()
                    .map(|f| {
                        f.iter()
                            .map(|x| {
                                format!(
                                    "{}: {}",
                                    x["name"].as_str().unwrap_or(""),
                                    x["type"].as_str().unwrap_or("")
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default(),
                "component" => item["props"]
                    .as_array()
                    .map(|f| {
                        f.iter()
                            .map(|x| {
                                format!(
                                    "{}{}: {}",
                                    if x["positional"] == true { "_ " } else { "" },
                                    x["name"].as_str().unwrap_or(""),
                                    x["type"].as_str().unwrap_or("")
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default(),
                "store" => item["members"]
                    .as_array()
                    .map(|m| {
                        m.iter()
                            .map(|x| x["name"].as_str().unwrap_or("").to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default(),
                "script" => {
                    let params = item["params"]
                        .as_array()
                        .map(|ps| {
                            ps.iter()
                                .map(|p| {
                                    format!(
                                        "{}{}{}: {}",
                                        if p["rest"] == true { "..." } else { "" },
                                        p["name"].as_str().unwrap_or(""),
                                        if p["optional"] == true { "?" } else { "" },
                                        p["type"].as_str().unwrap_or("")
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    match item["kind"].as_str() {
                        Some("value") => String::new(),
                        _ => format!(
                            "({params}){}",
                            item["returns"]
                                .as_str()
                                .map(|r| format!(" -> {r}"))
                                .unwrap_or_default()
                        ),
                    }
                }
                _ => item["path"].as_str().unwrap_or("").to_string(),
            };
            println!(
                "{kind:<10} {name:<20} {detail}    ({}:{})",
                item["file"].as_str().unwrap_or(""),
                item["line"]
            );
        }
    }
    Ok(())
}
