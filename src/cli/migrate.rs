use std::path::{Path, PathBuf};

use crate::config::ProjectConfig;

use crate::error::{Result, WebFluentError};
use crate::migrate::{migrate_project, report, source_files};

/// `wf migrate [path] [--check] [--stdout] [--wfx]`.
pub fn run_migrate(path: &Path, check: bool, stdout: bool, wfx: bool) -> Result<()> {
    let files = if path.is_file() {
        vec![(path.to_path_buf(), std::fs::read_to_string(path)?)]
    } else {
        let src = if path.join("src").is_dir() {
            path.join("src")
        } else {
            path.to_path_buf()
        };
        source_files(&src)?
    };
    if files.is_empty() {
        return Err(WebFluentError::IoError(format!(
            "no .wf files under {}",
            path.display()
        )));
    }

    let outcomes = migrate_project(&files);
    let mut changed = 0;
    let mut failed = 0;
    let mut notes = 0;
    for (file, outcome) in &outcomes {
        match outcome {
            Ok(migrated) => {
                // `--wfx`: the migrated text in the indented layout, under
                // the `.wfx` name; the `.wf` file goes.
                let (text, written_to) = if wfx {
                    let name = file.to_string_lossy();
                    match crate::layout::to_offside(&migrated.text, &name) {
                        Ok(text) => (text, super::fmt::offside_path(file)),
                        Err(e) => {
                            failed += 1;
                            eprintln!("  {}: {e}", file.display());
                            continue;
                        }
                    }
                } else {
                    (migrated.text.clone(), file.clone())
                };
                if stdout {
                    print!("{text}");
                    continue;
                }
                if migrated.changed || wfx {
                    changed += 1;
                    println!("  {}", written_to.display());
                }
                notes += migrated.notes.len();
                print!("{}", report(file, migrated));
                if (migrated.changed || wfx) && !check {
                    std::fs::write(&written_to, &text)?;
                    if written_to != *file {
                        std::fs::remove_file(file)?;
                    }
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("  {}: {e}", file.display());
            }
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
        outcomes.len() - changed - failed
    );
    // The spelling is only half of it: 4 also changed what the compiler
    // allows, and a project already written in 3's grammar needs that
    // half on its own.
    if failed == 0 && path.is_dir() {
        migrate_to_4(path, &files, check)?;
    }
    if failed > 0 {
        return Err(WebFluentError::IoError(format!(
            "{failed} file(s) could not be migrated"
        )));
    }
    Ok(())
}

/// The WebFluent 3 → 4 step, run over a whole project.
///
/// The file-level migration above is 2 → 3, a change of spelling. What 4
/// changed is what the compiler *allows*, and most of it is a decision
/// only the author can make: whether a value in `env` may be read by
/// anyone who opens the site, what an `onclick:` attribute was for. So
/// this does the one mechanical part, names every place that needs a
/// person with its file and line, and states the changes that need no
/// edit at all.
fn migrate_to_4(project_dir: &Path, files: &[(PathBuf, String)], check: bool) -> Result<()> {
    let Ok(mut config) = ProjectConfig::load(project_dir) else {
        return Ok(()); // a lone file, or not a project
    };

    // ── The mechanical part: `env` names a page reads ──
    //
    // In 3 every name was inlined into the bundle. In 4 only a public one
    // may be, so the migration preserves what the project did — the names
    // it actually reads become public — and says so, because "public"
    // means anyone who opens the site can read the value.
    let mut read: Vec<String> = Vec::new();
    for (file, _) in files {
        let relative = file
            .strip_prefix(project_dir)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string();
        for diagnostic in crate::linter::lint_env(project_dir, &config, &[relative]) {
            if let Some(name) = between(&diagnostic.message, "`env.", "`")
                && !read.contains(&name)
            {
                read.push(name);
            }
        }
    }
    if !read.is_empty() {
        println!("\n  `env` names these pages read, which were in the bundle already:");
        for name in &read {
            println!("    {name}");
        }
        println!("  They are added to `public_env`, so the build keeps working.");
        println!("  Read the list: a value anyone who opens the site may read belongs there,");
        println!("  and anything else belongs behind a request.");
        if !check {
            config.public_env.extend(read.iter().cloned());
            config.public_env.sort();
            config.public_env.dedup();
            write_config(project_dir, &config.public_env)?;
        }
    }

    // ── What needs a person ──
    let mut manual: Vec<String> = Vec::new();
    for (file, text) in files {
        let name = file
            .strip_prefix(project_dir)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string();
        let Ok(program) = crate::syntax::parse_source(text, &name) else {
            continue;
        };
        for error in crate::sema::check(&program, &|_| name.clone()).errors {
            let message = error.to_string();
            if message.contains("would put script in an attribute")
                || message.contains("which a browser runs")
            {
                manual.push(format!("    {message}"));
            }
        }
    }
    if !manual.is_empty() {
        println!("\n  {} thing(s) 4 refuses that 3 allowed:", manual.len());
        for line in &manual {
            println!("{line}");
        }
        println!("  An `on*` attribute is script in an attribute — write `on click {{ … }}`.");
        println!("  A `javascript:` or `data:` URL is script in a link — it has no replacement.");
    }

    // ── What changed under a program that still compiles ──
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

/// The text between two markers, when both are there.
fn between(text: &str, open: &str, close: &str) -> Option<String> {
    let from = text.find(open)? + open.len();
    let to = text[from..].find(close)? + from;
    Some(text[from..to].to_string())
}

/// Write `public_env` into the project's config, leaving everything else
/// in it — including the order it was written in — alone.
fn write_config(project_dir: &Path, public: &[String]) -> Result<()> {
    let path = project_dir.join("webfluent.app.json");
    let text = std::fs::read_to_string(&path)?;
    let mut value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| WebFluentError::ConfigError(format!("{}: {e}", path.display())))?;
    if let Some(map) = value.as_object_mut() {
        map.insert(
            "public_env".to_string(),
            serde_json::Value::Array(
                public
                    .iter()
                    .map(|n| serde_json::Value::String(n.clone()))
                    .collect(),
            ),
        );
    }
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&value).unwrap_or(text) + "\n",
    )?;
    println!("  {}", path.display());
    Ok(())
}
