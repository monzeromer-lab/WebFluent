//! `wf audit` — what this project trusts ([`crate::audit`]), printed.

use crate::audit::Report;
use crate::error::Result;
use std::path::Path;

pub fn run_audit(project_dir: &Path, json: bool) -> Result<()> {
    let report = crate::audit::audit_project(project_dir)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        print(&report);
    }
    Ok(())
}

fn print(report: &Report) {
    println!("\n  What this project trusts\n");

    println!("  Markup a page did not write");
    if report.unsafe_html.is_empty() {
        println!("    none — every string in this project goes in as text");
    }
    for finding in &report.unsafe_html {
        println!(
            "    {}  {}",
            finding.at,
            if finding.sanitized {
                "through `sanitize`"
            } else {
                "NOT sanitised"
            }
        );
    }

    println!("\n  Other origins");
    if report.origins.is_empty() {
        println!("    none — everything is served from this site");
    }
    for origin in &report.origins {
        println!("    {origin}");
    }

    println!("\n  Written to the reader's machine");
    if report.persisted.is_empty() {
        println!("    nothing");
    }
    for kept in &report.persisted {
        println!(
            "    {}.{}  in {}Storage",
            kept.owner, kept.name, kept.storage
        );
    }

    println!("\n  `env`");
    if report.env.is_empty() {
        println!("    none");
    }
    for name in &report.env {
        println!(
            "    {}  {}",
            name.name,
            if name.public {
                "public — it is in the bundle"
            } else {
                "not public — the pages may not read it"
            }
        );
    }

    println!("\n  Content-Security-Policy");
    println!("    {}", report.csp);

    println!("\n  What the compiler depends on");
    for dep in &report.dependencies {
        println!("    {dep}");
    }
    println!();
}
