use std::path::Path;

use crate::error::Result;
use crate::registry::schema;

/// `wf components [-d DIR] [--json]`: every component a call can name, the
/// built-ins and the project's, with what each takes.
pub fn run_components(dir: &Path, json: bool) -> Result<()> {
    let catalog = schema::of_project_via(&crate::vfs::FsVfs, dir);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&catalog).unwrap_or_default()
        );
        return Ok(());
    }
    let mut group = String::new();
    for c in &catalog.components {
        if c.group != group {
            println!("\n{}", c.group);
            group = c.group.clone();
        }
        let props: Vec<String> = c
            .props
            .iter()
            .map(|p| {
                let lead = if p.positional { "_ " } else { "" };
                let opt = if p.required { "" } else { "?" };
                match p.kind {
                    "enum" if !p.cases.is_empty() => format!(
                        "{lead}{}{opt}: {}",
                        p.name,
                        p.cases
                            .iter()
                            .map(|c| format!(".{}", c.name))
                            .collect::<Vec<_>>()
                            .join("|")
                    ),
                    kind => format!("{lead}{}{opt}: {kind}", p.name),
                }
            })
            .collect();
        let at = match (&c.file, c.line) {
            (Some(f), Some(l)) => format!("   {f}:{l}"),
            _ => String::new(),
        };
        println!("  {}({}){at}", c.name, props.join(", "));
    }
    Ok(())
}
