//! `wf migrate [path] [--check] [--stdout] [--wfx]`: the plan
//! ([`crate::migrate::project`]), printed, and written unless `--check`.

use std::path::{Path, PathBuf};

use crate::error::{Result, WebFluentError};
use crate::migrate::project::{self, Plan};

pub fn run_migrate(path: &Path, check: bool, stdout: bool, wfx: bool) -> Result<()> {
    let plan = project::plan(path, wfx)?;
    let mut changed = 0;
    let mut failed = 0;
    let mut notes = 0;
    for file in &plan.files {
        if let Some(e) = &file.error {
            failed += 1;
            eprintln!("  {}: {e}", file.path.display());
            continue;
        }
        if stdout {
            print!("{}", file.text);
            continue;
        }
        if file.changed {
            changed += 1;
            println!("  {}", file.written_to.display());
        }
        notes += file.notes.len();
        print!("{}", file.report());
        if file.changed && !check {
            project::write(file)?;
        }
    }
    if stdout {
        if failed > 0 {
            return Err(WebFluentError::IoError(format!(
                "{failed} file(s) could not be migrated"
            )));
        }
        return Ok(());
    }
    let verb = if check { "would change" } else { "migrated" };
    println!(
        "{changed} file(s) {verb}, {} unchanged, {notes} note(s), {failed} failed",
        plan.files.len() - changed - failed
    );
    print_four(&plan, check)?;
    if failed > 0 {
        return Err(WebFluentError::IoError(format!(
            "{failed} file(s) could not be migrated"
        )));
    }
    Ok(())
}

/// The WebFluent 3 → 4 step, as `wf migrate` tells it: the `env` names that
/// become public (written unless `check`), what needs a person, and what
/// changed with no edit to make.
fn print_four(plan: &Plan, check: bool) -> Result<()> {
    let (Some(dir), Some(four)) = (&plan.project_dir, &plan.four) else {
        return Ok(()); // a lone file, or not a project
    };
    if !four.public_env.is_empty() {
        println!("\n  `env` names these pages read, which were in the bundle already:");
        for name in &four.public_env {
            println!("    {name}");
        }
        println!("  They are added to `public_env`, so the build keeps working.");
        println!("  Read the list: a value anyone who opens the site may read belongs there,");
        println!("  and anything else belongs behind a request.");
        if !check {
            let config: PathBuf = project::write_public_env(dir, &four.public_env)?;
            println!("  {}", config.display());
        }
    }
    if !four.manual.is_empty() {
        println!("\n  {} thing(s) 4 refuses that 3 allowed:", four.manual.len());
        for line in &four.manual {
            println!("    {line}");
        }
        println!("  An `on*` attribute is script in an attribute — write `on click {{ … }}`.");
        println!("  A `javascript:` or `data:` URL is script in a link — it has no replacement.");
    }
    // The same as `project::WITHOUT_AN_EDIT`, wrapped for a terminal.
    println!("\n  Changes that need no edit, and are worth knowing:");
    println!("    A store is built on first read, not at boot. A `derived` with a side");
    println!("    effect now runs when something reads it; `store X(eager: true)` restores");
    println!("    the old timing.");
    println!("    A `persist` value follows the site's other tabs. `sync: false` keeps one");
    println!("    to its own tab.");
    println!("    `wf init` turns the Content-Security-Policy on; an existing project opts");
    println!("    in with `\"build\": {{ \"csp\": true }}`, and the build then holds its own");
    println!("    output to it.");
    println!("    A hand-written script calling `WF.store(def)` wants `WF.store(name, define,");
    println!("    options)`, and `WF.host(…)` is now `WF.attach(…)`.");
    Ok(())
}
