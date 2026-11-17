//! `wf api`: a service's contract, written as files from its OpenAPI spec.

use std::path::Path;

use crate::diagnostics::error::{Result, WebFluentError};
use crate::tools::api::{self, Change};

fn said(change: &Change) -> String {
    match change {
        Change::Added(p) => format!("  wrote    {p}"),
        Change::Changed(p) => format!("  rewrote  {p}"),
        Change::Removed(p) => format!("  removed  {p}"),
    }
}

fn warn(warnings: &[String]) {
    for w in warnings {
        eprintln!("warning: {w}");
    }
}

/// The services a command names: the one asked for, or every one kept.
fn chosen(
    dir: &Path,
    name: Option<&str>,
) -> Result<Vec<(String, crate::project::config::project::ApiSource)>> {
    let all = api::services(dir)?;
    match name {
        Some(n) => match all.get(n) {
            Some(s) => Ok(vec![(n.to_string(), s.clone())]),
            None => Err(WebFluentError::config(format!(
                "`{n}` is not kept in `apis`{}: add it with `wf api add openapi.json --name {n}`",
                if all.is_empty() {
                    String::new()
                } else {
                    format!(
                        " (it keeps {})",
                        all.keys()
                            .map(|k| format!("`{k}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            ))),
        },
        None if all.is_empty() => Err(WebFluentError::config(
            "No service is kept in `apis`: add one with `wf api add openapi.json --name Backend`"
                .to_string(),
        )),
        None => Ok(all.into_iter().collect()),
    }
}

/// `wf api add SPEC --name NAME [--out DIR]`.
pub(crate) fn run_add(dir: &Path, spec: &str, name: &str, out: Option<&str>) -> Result<()> {
    let added = api::add(dir, spec, name, out)?;
    warn(&added.generated.warnings);
    println!("{name} is read from {spec}.");
    if let Some(service) = &added.service {
        println!("  wrote    {service}  (yours: sync never changes it)");
    }
    for c in &added.changes {
        println!("{}", said(c));
    }
    if added.gitattributes {
        println!("  marked   .gitattributes  (a review collapses the generated files)");
    }
    println!("Run `wf api check` in CI: it fails when the files are behind the spec.");
    Ok(())
}

/// `wf api sync [NAME] [--skip OP]…`.
pub(crate) fn run_sync(dir: &Path, name: Option<&str>, skip: &[String]) -> Result<()> {
    if !skip.is_empty() && name.is_none() && api::services(dir)?.len() > 1 {
        return Err(WebFluentError::config(
            "`--skip` names the service it is for: `wf api sync Backend --skip sendXml`"
                .to_string(),
        ));
    }
    for (service, source) in chosen(dir, name)? {
        let source = if skip.is_empty() {
            source
        } else {
            api::record_skip(dir, &service, &source, skip)?
        };
        let (generated, changes) = api::sync(dir, &service, &source)?;
        warn(&generated.warnings);
        if changes.is_empty() {
            println!("{service}: up to date with {}.", source.spec);
        } else {
            println!("{service}: from {}", source.spec);
            for c in &changes {
                println!("{}", said(c));
            }
        }
    }
    Ok(())
}

/// `wf api check [NAME]`: fails when `sync` would change anything.
pub(crate) fn run_check(dir: &Path, name: Option<&str>) -> Result<()> {
    let mut behind = false;
    for (service, source) in chosen(dir, name)? {
        let (_, changes) = api::changes(&crate::base::vfs::FsVfs, dir, &service, &source)?;
        if changes.is_empty() {
            println!("{service}: up to date with {}.", source.spec);
        } else {
            behind = true;
            println!(
                "{service}: behind {} — `wf api sync {service}` would change:",
                source.spec
            );
            for c in &changes {
                println!(
                    "{}",
                    match c {
                        Change::Added(p) => format!("  add      {p}"),
                        Change::Changed(p) => format!("  rewrite  {p}"),
                        Change::Removed(p) => format!("  remove   {p}"),
                    }
                );
            }
        }
    }
    if behind {
        std::process::exit(1);
    }
    Ok(())
}

/// `wf api eject NAME [--out DIR]`.
pub(crate) fn run_eject(dir: &Path, name: &str, out: Option<&str>) -> Result<()> {
    let ejected = api::eject(dir, name, out)?;
    warn(&ejected.added.generated.warnings);
    println!(
        "{name} is kept as files now; `api {name} from` left {}.",
        ejected.from_file
    );
    println!(
        "  wrote    {}  (yours: sync never changes it)",
        ejected.service
    );
    for c in &ejected.added.changes {
        println!("{}", said(c));
    }
    Ok(())
}

/// `wf api list [--json]`.
pub(crate) fn run_list(dir: &Path, json: bool) -> Result<()> {
    let listed = api::list(dir)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&listed).unwrap_or_default()
        );
        return Ok(());
    }
    if listed.is_empty() {
        println!(
            "No service is read from a spec. Add one with `wf api add openapi.json --name Backend`."
        );
        return Ok(());
    }
    for s in listed {
        match &s.out {
            Some(out) => println!("{}  from {}  in {out}", s.name, s.spec),
            None => println!(
                "{}  from {}  (api {} from \"{}\")",
                s.name, s.spec, s.name, s.spec
            ),
        }
        for g in &s.groups {
            println!("  {}.{}", s.name, g.name);
            for e in &g.endpoints {
                println!("    {e}");
            }
        }
    }
    Ok(())
}
