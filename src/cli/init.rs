//! `wf init <name> [--template spa|static|pdf|slides]`: a new project
//! ([`crate::init`]) in the current directory.

use crate::error::Result;
use crate::init::{Template, create};
use std::path::Path;

pub fn run_init(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        return Err(crate::error::WebFluentError::IoError(format!(
            "Directory '{}' already exists",
            name
        )));
    }

    let Some(kind) = Template::parse(template) else {
        eprintln!(
            "Unknown template '{}'. Use 'spa', 'static', 'pdf', or 'slides'.",
            template
        );
        std::process::exit(1);
    };
    create(project_dir, name, kind)?;

    println!(
        "Created new WebFluent project: {} (template: {})",
        name, template
    );
    println!();
    println!("  cd {}", name);
    println!("  wf build");
    println!("  wf serve");

    Ok(())
}
