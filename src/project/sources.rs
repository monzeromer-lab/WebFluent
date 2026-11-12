//! A project's sources, read: every `.wf`, `.wfx` and `.md` file under
//! `src/`, parsed and merged into one program, with its scripts, its `data`
//! and `image` declarations resolved and any `api … from` specification
//! expanded.

use crate::base::vfs::{FsVfs, Vfs};
use crate::diagnostics::error::{Diagnostic, Result, WebFluentError};
use crate::project::config::ProjectConfig;
use crate::syntax::parser::{Declaration, Program};
use std::path::{Path, PathBuf};

/// Every source file under the project's `src/`, parsed and merged into
/// one program, with the file each declaration came from (relative to the
/// project) so a diagnostic over the merged program can still name a file
/// the reader can open.
pub fn read_project(project_dir: &Path) -> Result<(Program, Vec<String>)> {
    read_project_with(project_dir, None)
}

/// [`read_project`], with somewhere to write the images the program names.
///
/// A reader that writes no files — `wf types`, the language server — passes
/// nothing, and an `image` resolves to the file as it stands.
pub fn read_project_with(
    project_dir: &Path,
    media: Option<(&Path, &crate::project::media::Settings, &str)>,
) -> Result<(Program, Vec<String>)> {
    read_project_via(&FsVfs, project_dir, media)
}

/// [`read_project_with`], reading every file through `vfs`.
pub fn read_project_via(
    vfs: &dyn Vfs,
    project_dir: &Path,
    media: Option<(&Path, &crate::project::media::Settings, &str)>,
) -> Result<(Program, Vec<String>)> {
    let (program, files, mut errors) = read_project_collecting_via(vfs, project_dir, media)?;
    match errors.len() {
        0 => Ok((program, files)),
        1 => Err(WebFluentError::ParseError(Box::new(errors.remove(0)))),
        _ => Err(WebFluentError::Diagnostics(errors)),
    }
}

/// [`read_project_with`], keeping going past a file that does not parse:
/// the program of every declaration that does, the file of each, and a
/// diagnostic for each mistake in the files that do not — so one typo in
/// one file does not hide every finding in the others. An error is only a
/// file that cannot be read at all.
pub fn read_project_collecting(
    project_dir: &Path,
    media: Option<(&Path, &crate::project::media::Settings, &str)>,
) -> Result<(Program, Vec<String>, Vec<Diagnostic>)> {
    read_project_collecting_via(&FsVfs, project_dir, media)
}

/// [`read_project_collecting`], reading every file through `vfs`.
pub fn read_project_collecting_via(
    vfs: &dyn Vfs,
    project_dir: &Path,
    media: Option<(&Path, &crate::project::media::Settings, &str)>,
) -> Result<(Program, Vec<String>, Vec<Diagnostic>)> {
    let mut parse_errors: Vec<Diagnostic> = Vec::new();
    let src_dir = project_dir.join("src");
    if !vfs.exists(&src_dir) {
        return Err(WebFluentError::IoError(
            "src/ directory not found".to_string(),
        ));
    }
    let wf_files = find_wf_files(vfs, &src_dir)?;
    if wf_files.is_empty() {
        return Err(WebFluentError::IoError(
            "No .wf files found in src/".to_string(),
        ));
    }
    let mut all_declarations = Vec::new();
    let mut declaration_files: Vec<String> = Vec::new();
    for file_path in &wf_files {
        let source = vfs.read_to_string(file_path)?;
        let relative = file_path.strip_prefix(project_dir).unwrap_or(file_path);
        let file_name = relative.to_string_lossy().to_string();
        let program = if file_name.ends_with(".md") {
            match crate::project::data::markdown_page(&source, &file_name) {
                Ok(program) => program,
                Err(e) if !e.diagnostics().is_empty() => {
                    parse_errors.extend(e.diagnostics());
                    continue;
                }
                Err(e) => return Err(e),
            }
        } else {
            let (program, errors) = crate::syntax::parse_source_recovering(&source, &file_name);
            parse_errors.extend(errors);
            program
        };
        declaration_files.extend(program.declarations.iter().map(|_| file_name.clone()));
        all_declarations.extend(program.declarations);
    }
    // The names the project's scripts make global, one declaration per
    // file, so the checker, the codegen and the editor read the same ones.
    for script in crate::project::project_js::load_via(vfs, project_dir, &src_dir)? {
        if script.scan.module_at.is_some() || !script.scan.problems.is_empty() {
            continue;
        }
        declaration_files.push(script.path.clone());
        all_declarations.push(Declaration::Script(
            crate::syntax::parser::ast::ScriptDecl {
                path: script.path,
                names: script.scan.names,
            },
        ));
    }
    // And what `meta.scripts` says a library defines: the compiler cannot
    // read a file on another origin, so each name is in scope as `Any`.
    if let Ok(config) = ProjectConfig::load_via(vfs, project_dir) {
        let names: Vec<crate::syntax::js_scan::Name> = config
            .meta
            .scripts
            .iter()
            .flat_map(|s| s.names())
            .map(|name| crate::syntax::js_scan::Name {
                name: name.to_string(),
                kind: crate::syntax::js_scan::NameKind::Value,
                line: 1,
                col: 1,
                doc: None,
            })
            .collect();
        if !names.is_empty() {
            declaration_files.push("webfluent.app.json".to_string());
            all_declarations.push(Declaration::Script(
                crate::syntax::parser::ast::ScriptDecl {
                    path: "webfluent.app.json".to_string(),
                    names,
                },
            ));
        }
    }
    let mut program = Program {
        declarations: all_declarations,
    };
    // `data x = "file.json"` is a constant once the file is read, and
    // `image hero = "…"` once the picture is.
    if let Err(e) = crate::project::data::resolve_data_via(vfs, &mut program, project_dir, media) {
        if e.diagnostics().is_empty() {
            return Err(e);
        }
        parse_errors.extend(e.diagnostics());
    }
    // `api B from "openapi.json"` is its endpoints once the spec is read,
    // so every reader of a project sees the same service.
    if let Err(e) = crate::project::openapi::expand(&mut program, &|file| {
        for at in [project_dir.join(file), project_dir.join("src").join(file)] {
            if let Ok(text) = vfs.read_to_string(&at) {
                return Some(text);
            }
        }
        None
    }) {
        if e.diagnostics().is_empty() {
            return Err(e);
        }
        parse_errors.extend(e.diagnostics());
    }
    Ok((program, declaration_files, parse_errors))
}

pub(crate) fn find_wf_files(vfs: &dyn Vfs, dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !vfs.is_dir(dir) {
        return Ok(files);
    }

    // Process App.wf (or App.wfx) first if it exists (so App declaration comes first)
    let app_file = ["App.wf", "App.wfx"]
        .iter()
        .map(|n| dir.join(n))
        .find(|p| vfs.exists(p));
    if let Some(app) = &app_file {
        files.push(app.clone());
    }

    // In name order, so a build is the same wherever the project sits:
    // the directory's own order is the file system's, and a page's
    // element numbering follows the order the files are read in.
    let mut entries: Vec<PathBuf> = vfs.read_dir(dir)?;
    entries.sort();
    for path in entries {
        if vfs.is_dir(&path) {
            files.extend(find_wf_files(vfs, &path)?);
        } else if crate::syntax::is_source_file(&path)
            || path.extension().is_some_and(|e| e == "md")
        {
            // Skip App.wf since we already added it
            if app_file.as_ref() == Some(&path) {
                continue;
            }
            files.push(path);
        }
    }

    Ok(files)
}
