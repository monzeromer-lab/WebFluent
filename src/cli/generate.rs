use crate::error::Result;
use crate::generate::{Kind, Written, is_indented, source, write_source};
use std::fs;
use std::path::Path;

pub fn run_generate(kind: &str, name: &str, project_dir: &Path) -> Result<()> {
    let kind = match kind {
        "page" => Kind::Page,
        "component" => Kind::Component,
        "store" => Kind::Store,
        _ => {
            eprintln!(
                "Unknown generator '{}'. Use: page, component, or store",
                kind
            );
            return Ok(());
        }
    };
    let src = project_dir.join("src");
    let dir = src.join(kind.folder());
    fs::create_dir_all(&dir)?;

    let stem = kind.file_stem(name);
    match write_source(&dir, &stem, &source(kind, name), is_indented(&src))? {
        Written::Created(path) => println!(
            "Created {}: src/{}/{}",
            kind.label().to_lowercase(),
            kind.folder(),
            path.file_name().unwrap().to_string_lossy()
        ),
        Written::Exists(_) => eprintln!("{} {stem} already exists", kind.label()),
    }
    Ok(())
}
