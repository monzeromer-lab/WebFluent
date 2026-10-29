//! `wf init <name> [--template spa|static|pdf|slides] [--locales en,ar]`: a new project
//! ([`crate::init`]) in the current directory.

use crate::error::Result;
use crate::init::{Options, Template, create_with, parse_locales};
use std::path::Path;

pub fn run_init(name: &str, template: &str, locales: Option<&str>) -> Result<()> {
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
    let mut options = Options::new(kind);
    if let Some(list) = locales {
        options.locales = parse_locales(list)?;
    }
    create_with(project_dir, name, &options)?;

    println!(
        "Created new WebFluent project: {} (template: {})",
        name, template
    );
    if let Some((default, others)) = options.locales.split_first() {
        let others = if others.is_empty() {
            String::new()
        } else {
            format!(", and {}", others.join(", "))
        };
        println!("  It speaks {default} by default{others}: src/translations/ holds each");
    }
    println!("  AGENTS.md is the language reference, for you and your coding agent");
    println!();
    println!("  cd {}", name);
    println!("  wf build");
    println!("  wf serve");

    Ok(())
}
