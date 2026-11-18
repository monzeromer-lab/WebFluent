//! `wf docs [-d DIR] [-o OUT]`: the project's components, documented as
//! the reference documents the built-ins ([`crate::docs`]).

use std::path::Path;

use crate::diagnostics::error::Result;

pub(crate) fn run_docs(project_dir: &Path, out: &Path) -> Result<()> {
    let out = if out.is_absolute() {
        out.to_path_buf()
    } else {
        project_dir.join(out)
    };
    let written = crate::docs::write(project_dir, &out)?;
    println!(
        "  docs: {} component(s), {} example(s){} → {}",
        written.components,
        written.examples,
        if written.made > 0 {
            format!(" and {} made from props", written.made)
        } else {
            String::new()
        },
        written.out.display()
    );
    println!("  Serve the folder to read it: its pages are linked from the site's root.");
    Ok(())
}
