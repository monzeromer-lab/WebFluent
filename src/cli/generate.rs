use crate::diagnostics::error::Result;
use crate::generate::{Kind, Written, is_indented, source, write_source};
use std::fs;
use std::path::Path;

pub(crate) fn run_generate(kind: &str, name: &str, project_dir: &Path) -> Result<()> {
    if kind == "block" {
        return run_block(name, project_dir);
    }
    let kind = match kind {
        "page" => Kind::Page,
        "component" => Kind::Component,
        "store" => Kind::Store,
        _ => {
            eprintln!("Unknown generator '{kind}'. Use: page, component, store or block");
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
            path.file_name()
                .expect("a file it wrote has a name")
                .to_string_lossy()
        ),
        Written::Exists(_) => eprintln!("{} {stem} already exists", kind.label()),
    }
    Ok(())
}

/// `wf generate block <name>`: a whole page the project then owns, written
/// to `src/pages/` in the project's layout.
fn run_block(name: &str, project_dir: &Path) -> Result<()> {
    let Some((page, text)) = crate::generate::block(name) else {
        let names: Vec<&str> = crate::generate::BLOCKS.iter().map(|(n, _, _)| *n).collect();
        eprintln!(
            "No block is called '{name}'. The blocks: {}",
            names.join(", ")
        );
        std::process::exit(2);
    };
    let src = project_dir.join("src");
    let dir = src.join("pages");
    fs::create_dir_all(&dir)?;
    match write_source(&dir, &page, text, is_indented(&src))? {
        Written::Created(path) => println!(
            "Created page {page}: src/pages/{}",
            path.file_name()
                .expect("a file it wrote has a name")
                .to_string_lossy()
        ),
        Written::Exists(_) => eprintln!("Page {page} already exists"),
    }
    Ok(())
}
