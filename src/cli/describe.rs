use std::path::Path;

use crate::error::Result;

/// `wf registry --json`: every built-in, as a tool reads it.
pub fn run_registry(json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&crate::registry::json::registry_json()).unwrap()
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

/// `wf types --json [path]`: what a project declares, as a tool reads it.
pub fn run_types(project_dir: &Path, json: bool) -> Result<()> {
    let (program, files) = super::build::read_project(project_dir)?;
    let file_of = |index: usize| {
        files
            .get(index)
            .cloned()
            .unwrap_or_else(|| "src/".to_string())
    };
    let value = crate::registry::json::declarations_json(&program, &file_of);
    if json {
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
        return Ok(());
    }
    for (kind, key) in [
        ("enum", "enums"),
        ("type", "types"),
        ("component", "components"),
        ("store", "stores"),
        ("page", "pages"),
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
