//! `wf api`: a service's contract written as files from its OpenAPI spec,
//! where it can be read, reviewed and diffed — never edited.
//!
//! `add` records a service in `webfluent.app.json` and writes its
//! `service.wf` once; `sync` rewrites `generated/` whole from the spec;
//! `check` says whether `sync` would change anything (for CI); `eject` turns
//! `api Backend from "…"` into files; `list` says what each service has.
//!
//! Every generated file opens with the marker Go, protobuf and GitHub
//! recognise, and one without it is never overwritten: it was not written
//! by `sync`. The output is the printer's, so it is deterministic and a
//! change to the spec is the smallest diff.

use std::path::Path;

use crate::base::vfs::FsVfs;
use crate::diagnostics::error::{Result, WebFluentError};
use crate::project::config::project::ApiSource;
use crate::project::openapi::{self, Imported};
use crate::syntax::parser::Declaration;

const CONFIG: &str = "webfluent.app.json";

use crate::project::openapi::files::relative;

fn config_error(message: String) -> WebFluentError {
    WebFluentError::config(message)
}

fn io(path: &Path, e: &std::io::Error) -> WebFluentError {
    WebFluentError::io(format!("{}: {e}", path.display()))
}

/// The services the project keeps, as its config lists them.
///
/// # Errors
///
/// When the config cannot be read.
pub fn services(project: &Path) -> Result<std::collections::BTreeMap<String, ApiSource>> {
    let config = crate::project::config::ProjectConfig::load(project)?;
    Ok(config.apis)
}

#[allow(
    unused_imports,
    reason = "read only through the feature: tests and tools"
)]
pub use crate::project::openapi::files::{
    Change, File, Generated, changes, generate, is_generated, marker, stale_services,
};

/// Rewrite `generated/` from the spec: every file it should hold, and none
/// it should not.
///
/// # Errors
///
/// What [`changes`] returns, and a file that cannot be written.
pub fn sync(project: &Path, name: &str, source: &ApiSource) -> Result<(Generated, Vec<Change>)> {
    let (generated, changes) = changes(&FsVfs, project, name, source)?;
    for change in &changes {
        let at = project.join(change.path());
        match change {
            Change::Added(p) | Change::Changed(p) => {
                let Some(file) = generated.files.iter().find(|f| f.path == *p) else {
                    continue;
                };
                let text = &file.text;
                if let Some(parent) = at.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| io(parent, &e))?;
                }
                std::fs::write(&at, text).map_err(|e| io(&at, &e))?;
            }
            Change::Removed(_) => std::fs::remove_file(&at).map_err(|e| io(&at, &e))?,
        }
    }
    Ok((generated, changes))
}

/// `service.wf` as `add` writes it: where the service is and how it is
/// reached, from the spec's first server and its security schemes.
pub fn service_text(name: &str, imported: &Imported) -> String {
    let base = imported
        .facts
        .server
        .clone()
        .unwrap_or_else(|| "/".to_string());
    let mut out =
        String::from("// Written once by `wf api add`. Yours: `wf api sync` never changes it.\n\n");
    if let Some(title) = &imported.facts.title {
        out.push_str(&format!("/// {title}\n"));
    }
    out.push_str(&format!(
        "api {name}(base: {}) {{\n    timeout: 30.seconds\n",
        crate::syntax::print::string(&base)
    ));
    if imported.facts.bearer {
        out.push_str(
            "    // The spec's security scheme is a bearer token: send one with every\n    // call, from a store that keeps it.\n    // middleware [bearer(Session.token)]\n",
        );
    }
    for scheme in &imported.facts.schemes {
        out.push_str(&format!(
            "    // A security scheme the spec names: {scheme}.\n"
        ));
    }
    out.push_str(
        "\n    // What a write makes stale, and how a read pages, for the generated\n    // endpoints: `invalidations { group.create invalidates [group.list] }`,\n    // `pagination { group.list pages .cursor(param: after, next: p => p.next, items: p => p.items) }`.\n}\n",
    );
    out
}

/// What `add` did.
#[derive(Debug)]
pub struct Added {
    pub service: Option<String>,
    pub gitattributes: bool,
    pub generated: Generated,
    pub changes: Vec<Change>,
}

/// Record a service, write its `service.wf` (unless there is one), mark its
/// generated folder in `.gitattributes`, and sync it.
///
/// # Errors
///
/// When the service is already kept, when the spec cannot be read, and what
/// [`sync`] returns.
pub fn add(project: &Path, spec: &str, name: &str, out: Option<&str>) -> Result<Added> {
    if !openapi::is_name(name) || !name.starts_with(|c: char| c.is_ascii_uppercase()) {
        return Err(config_error(format!(
            "`{name}` is not a service's name: one word with a capital, `Backend`"
        )));
    }
    let spec = spec.trim_start_matches("./").replace('\\', "/");
    let out = out.map_or_else(
        || format!("src/api/{}", name.to_ascii_lowercase()),
        |o| o.trim_end_matches('/').to_string(),
    );
    if services(project)?.contains_key(name) {
        return Err(config_error(format!(
            "`{name}` is already kept in `apis`: run `wf api sync {name}`"
        )));
    }
    let source = ApiSource {
        spec: spec.clone(),
        out: out.clone(),
        skip: Vec::new(),
    };
    // Read first: a spec that cannot be read records nothing.
    let generated = generate(&FsVfs, project, name, &source)?;

    let config_path = project.join(CONFIG);
    let text = std::fs::read_to_string(&config_path).map_err(|e| io(&config_path, &e))?;
    let value = serde_json::json!({ "spec": spec, "out": out });
    let edited = crate::project::config::schema::edit(&text, &format!("apis.{name}"), Some(&value))
        .map_err(|e| config_error(e.to_string()))?;
    std::fs::write(&config_path, edited).map_err(|e| io(&config_path, &e))?;

    let service_path = project.join(&out).join("service.wf");
    let service = if service_path.exists() {
        None
    } else {
        std::fs::create_dir_all(project.join(&out)).map_err(|e| io(&project.join(&out), &e))?;
        std::fs::write(&service_path, service_text(name, &generated.imported))
            .map_err(|e| io(&service_path, &e))?;
        Some(relative(project, &service_path))
    };

    let attributes = project.join(".gitattributes");
    let line = format!("{out}/generated/** linguist-generated=true");
    let existing = std::fs::read_to_string(&attributes).unwrap_or_default();
    let gitattributes = !existing.lines().any(|l| l.trim() == line);
    if gitattributes {
        let mut text = existing;
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&line);
        text.push('\n');
        std::fs::write(&attributes, text).map_err(|e| io(&attributes, &e))?;
    }

    let (generated, changes) = sync(project, name, &source)?;
    Ok(Added {
        service,
        gitattributes,
        generated,
        changes,
    })
}

/// Record that `sync` leaves `names` out of `service`, keeping the
/// config's shape.
///
/// # Errors
///
/// When the config cannot be read or written.
pub fn record_skip(
    project: &Path,
    service: &str,
    source: &ApiSource,
    names: &[String],
) -> Result<ApiSource> {
    let mut updated = source.clone();
    for n in names {
        if !updated.skip.contains(n) {
            updated.skip.push(n.clone());
        }
    }
    if updated.skip != source.skip {
        let config_path = project.join(CONFIG);
        let text = std::fs::read_to_string(&config_path).map_err(|e| io(&config_path, &e))?;
        let value = serde_json::to_value(&updated).map_err(|e| config_error(e.to_string()))?;
        let edited =
            crate::project::config::schema::edit(&text, &format!("apis.{service}"), Some(&value))
                .map_err(|e| config_error(e.to_string()))?;
        std::fs::write(&config_path, edited).map_err(|e| io(&config_path, &e))?;
    }
    Ok(updated)
}

/// What `eject` did.
#[derive(Debug)]
pub struct Ejected {
    /// The file the `api … from` was taken out of.
    pub from_file: String,
    pub service: String,
    pub added: Added,
}

/// Turn `api Backend from "openapi.json" (…) { … }` into a kept service:
/// the declaration leaves its file, `service.wf` holds it without `from`,
/// and `generated/` holds what the spec said.
///
/// # Errors
///
/// When no file declares `api NAME from`, and what [`add`] returns.
pub fn eject(project: &Path, name: &str, out: Option<&str>) -> Result<Ejected> {
    let files = crate::project::sources::find_wf_files(&FsVfs, &project.join("src"))?;
    for file in files {
        let source = std::fs::read_to_string(&file).map_err(|e| io(&file, &e))?;
        let label = relative(project, &file);
        let Ok(program) = crate::syntax::parse_source(&source, &label) else {
            continue;
        };
        let Some(api) = program.declarations.iter().find_map(|d| match d {
            Declaration::Api(a) if a.name == name && a.from.is_some() => Some(a),
            _ => None,
        }) else {
            continue;
        };
        let spec = api.from.clone().unwrap_or_default();
        // `from` names a file beside the config or under `src/`.
        let spec = if project.join(&spec).is_file() {
            spec
        } else {
            format!("src/{spec}")
        };
        let mut kept = api.clone();
        kept.from = None;
        let decl = crate::syntax::print::declaration(&Declaration::Api(kept), &program);
        let out_dir = out.map_or_else(
            || format!("src/api/{}", name.to_ascii_lowercase()),
            str::to_string,
        );
        let service_path = project.join(&out_dir).join("service.wf");
        if service_path.exists() {
            return Err(config_error(format!(
                "`{}` is already there",
                relative(project, &service_path)
            )));
        }
        // The declaration leaves its file, with its `///` and the blank line
        // after it.
        let start = line_start(&source, api.span.start as usize, api.doc.is_some());
        let end = (api.span.end as usize).min(source.len());
        let mut rest = format!(
            "{}{}",
            &source[..start],
            source[end..].trim_start_matches('\n')
        );
        if rest.trim().is_empty() {
            rest.clear();
        }
        std::fs::create_dir_all(project.join(&out_dir))
            .map_err(|e| io(&project.join(&out_dir), &e))?;
        std::fs::write(
            &service_path,
            format!(
                "// Written by `wf api eject`. Yours: `wf api sync` never changes it.\n\n{decl}\n"
            ),
        )
        .map_err(|e| io(&service_path, &e))?;
        if rest.is_empty() {
            std::fs::remove_file(&file).map_err(|e| io(&file, &e))?;
        } else {
            std::fs::write(&file, rest).map_err(|e| io(&file, &e))?;
        }
        let added = add(project, &spec, name, Some(&out_dir))?;
        return Ok(Ejected {
            from_file: label,
            service: relative(project, &service_path),
            added,
        });
    }
    Err(config_error(format!(
        "No file under `src/` declares `api {name} from \"…\"`"
    )))
}

/// Where the line holding `at` starts — and, with `doc`, where the `///`
/// lines above it start.
fn line_start(source: &str, at: usize, doc: bool) -> usize {
    let mut start = source[..at.min(source.len())]
        .rfind('\n')
        .map_or(0, |i| i + 1);
    if doc {
        while start > 0 {
            let prev = source[..start - 1].rfind('\n').map_or(0, |i| i + 1);
            if source[prev..start].trim_start().starts_with("///") {
                start = prev;
            } else {
                break;
            }
        }
    }
    start
}

/// One service as `list` shows it.
#[derive(Debug, serde::Serialize)]
pub struct Listing {
    pub name: String,
    pub spec: String,
    /// Where its files are, or `null` for one read with `from`.
    pub out: Option<String>,
    pub groups: Vec<ListedGroup>,
}

#[derive(Debug, serde::Serialize)]
pub struct ListedGroup {
    pub name: String,
    pub endpoints: Vec<String>,
}

/// Every service the project reads from a spec: those `apis` keeps, and
/// each `api … from`.
///
/// # Errors
///
/// When the config or a spec cannot be read.
pub fn list(project: &Path) -> Result<Vec<Listing>> {
    let mut out = Vec::new();
    for (name, source) in services(project)? {
        let generated = generate(&FsVfs, project, &name, &source)?;
        out.push(listing(
            &name,
            &source.spec,
            Some(source.out.clone()),
            &generated.imported,
        ));
    }
    let files =
        crate::project::sources::find_wf_files(&FsVfs, &project.join("src")).unwrap_or_default();
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let Ok(program) = crate::syntax::parse_source(&text, &relative(project, &file)) else {
            continue;
        };
        for d in &program.declarations {
            if let Declaration::Api(a) = d
                && let Some(spec) = &a.from
            {
                let path = if project.join(spec).is_file() {
                    project.join(spec)
                } else {
                    project.join("src").join(spec)
                };
                let text = std::fs::read_to_string(&path).map_err(|e| io(&path, &e))?;
                let imported = openapi::read(&text, spec, &[])?;
                out.push(listing(&a.name, spec, None, &imported));
            }
        }
    }
    Ok(out)
}

fn listing(name: &str, spec: &str, out: Option<String>, imported: &Imported) -> Listing {
    Listing {
        name: name.to_string(),
        spec: spec.to_string(),
        out,
        groups: imported
            .groups
            .iter()
            .map(|g| ListedGroup {
                name: g.name.clone(),
                endpoints: g
                    .endpoints
                    .iter()
                    .map(crate::syntax::print::endpoint)
                    .collect(),
            })
            .collect(),
    }
}
