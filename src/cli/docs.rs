//! `wf docs [-d DIR] [-o OUT]`: the component gallery ([`crate::gallery`]),
//! written as `index.html`.

use std::path::Path;

use crate::error::Result;

pub fn run_docs(project_dir: &Path, out: &Path) -> Result<()> {
    let (html, builtins, declared) = crate::gallery::for_project(project_dir);
    std::fs::create_dir_all(out)?;
    let file = out.join("index.html");
    std::fs::write(&file, html)?;
    println!(
        "  docs: {builtins} built-in components{} → {}",
        declared
            .map(|n| format!(", {n} of the project's"))
            .unwrap_or_default(),
        file.display()
    );
    Ok(())
}
