//! A project's own scripts — the `.js` files under `src/` — seen from the
//! script's side: who uses what it declares.
//!
//! A function a script declares is called from `.wf` files, which no
//! JavaScript tool reads, so to the script's own editor it looks unused.
//! Here each top-level name a script makes global (the build's reading of
//! it, [`webfluent::project_js::scan`]) is followed to every place that
//! names it: the pages, components and stores that call it — resolved as
//! go-to-definition resolves them, so a local of the same name is not one —
//! and the project's other scripts, and the script itself. From that come
//! find-references, document highlights, a hover and a code lens on each
//! declaration, and a faded hint on a name nothing uses.

use std::collections::HashMap;

use lsp_types::{
    CodeLens, Command, Diagnostic, DiagnosticSeverity, DiagnosticTag, Hover, HoverContents,
    Location, MarkupContent, MarkupKind, NumberOrString, Position, PrepareRenameResponse, Range,
    TextEdit, WorkspaceEdit,
};

use webfluent::project_js::scan::{Name, NameKind};
use webfluent::syntax::parser::ast::Declaration;

use crate::project::Project;

/// Whether the file at `ix` is one of the project's scripts: JavaScript,
/// not the config's list of library globals.
pub fn is_script(project: &Project, ix: usize) -> bool {
    let file = &project.files[ix];
    file.script
        && file
            .path
            .extension()
            .is_some_and(|e| e == "js" || e == "mjs" || e == "cjs")
}

/// What the script at `ix` declares, as the build read it.
pub fn names(project: &Project, ix: usize) -> Vec<&Name> {
    let Ok(program) = &project.files[ix].parsed else {
        return Vec::new();
    };
    program
        .declarations
        .iter()
        .flat_map(|d| match d {
            Declaration::Script(s) => s.names.iter().collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .collect()
}

/// Where a script declares `name`: its file and the range of the name.
pub fn declaration(project: &Project, name: &str) -> Option<Location> {
    let (ix, n) = project.script_name(name)?;
    if !is_script(project, ix) {
        return None;
    }
    let file = &project.files[ix];
    Some(Location {
        uri: file.uri.clone(),
        range: file
            .index
            .word_range_at_line_col(&file.source, n.line, n.col),
    })
}

/// Every place `name` — a script's — is named, its declaration first: the
/// `.wf` files that use it and every script that does.
pub fn uses(project: &Project, name: &str) -> Vec<Location> {
    let Some(target) = declaration(project, name) else {
        return Vec::new();
    };
    let mut out: Vec<Location> = crate::rename::uses(project, &target, name)
        .into_iter()
        .map(|(uri, range)| Location { uri, range })
        .collect();
    out.sort_by_key(|l| {
        (
            l != &target,
            l.uri.to_string(),
            l.range.start.line,
            l.range.start.character,
        )
    });
    out.dedup();
    out
}

/// The uses of `name` in every script of the project: what the WebFluent
/// tokens cannot see.
pub fn script_uses(project: &Project, name: &str) -> Vec<Location> {
    let mut out = Vec::new();
    for (ix, file) in project.files.iter().enumerate() {
        if !is_script(project, ix) {
            continue;
        }
        for (line, col) in webfluent::project_js::scan::uses_of(&file.source, name) {
            out.push(Location {
                uri: file.uri.clone(),
                range: file.index.word_range_at_line_col(&file.source, line, col),
            });
        }
    }
    out
}

/// The script name at `position` in the script at `ix`, when it is one a
/// script of the project declares.
fn name_at(project: &Project, ix: usize, position: Position) -> Option<String> {
    let file = &project.files[ix];
    let offset = file.index.position_to_offset(&file.source, position)?;
    let (word, _) = crate::line_index::word_at(&file.source, offset)?;
    declaration(project, word)?;
    // Only where it is written as a name, not in a string or a comment.
    let pos = file.index.offset_to_position(&file.source, offset);
    let written = script_uses(project, word).into_iter().any(|l| {
        l.uri == file.uri
            && l.range.start.line == pos.line
            && l.range.start <= pos
            && pos <= l.range.end
    });
    written.then(|| word.to_string())
}

/// Find-references in a script.
pub fn references(
    project: &Project,
    ix: usize,
    position: Position,
    include_declaration: bool,
) -> Vec<Location> {
    let Some(name) = name_at(project, ix, position) else {
        return Vec::new();
    };
    let declared = declaration(project, &name);
    uses(project, &name)
        .into_iter()
        .filter(|l| include_declaration || Some(l) != declared.as_ref())
        .collect()
}

/// Renaming a script's name from the script: where the cursor is, when it
/// is one.
pub fn prepare_rename(
    project: &Project,
    ix: usize,
    position: Position,
) -> Option<PrepareRenameResponse> {
    let name = name_at(project, ix, position)?;
    let file = &project.files[ix];
    let offset = file.index.position_to_offset(&file.source, position)?;
    let (_, at) = crate::line_index::word_at(&file.source, offset)?;
    Some(PrepareRenameResponse::RangeWithPlaceholder {
        range: Range {
            start: file.index.offset_to_position(&file.source, at.start),
            end: file.index.offset_to_position(&file.source, at.end),
        },
        placeholder: name,
    })
}

/// The edits that rename a script's name everywhere it is named: the
/// script, the other scripts, and every `.wf` file that calls it.
///
/// # Errors
///
/// A message for the editor when `new_name` is not a name, or there is no
/// script's name at `position`.
pub fn rename(
    project: &Project,
    ix: usize,
    position: Position,
    new_name: &str,
) -> Result<WorkspaceEdit, String> {
    if !crate::rename::is_identifier(new_name) {
        return Err(format!("`{new_name}` is not a name"));
    }
    let Some(name) = name_at(project, ix, position) else {
        return Err("Nothing to rename here".to_string());
    };
    let mut changes: HashMap<lsp_types::Url, Vec<TextEdit>> = HashMap::new();
    for l in uses(project, &name) {
        changes.entry(l.uri).or_default().push(TextEdit {
            range: l.range,
            new_text: new_name.to_string(),
        });
    }
    Ok(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

/// The places in this script that name what is under the cursor.
pub fn highlights(project: &Project, ix: usize, position: Position) -> Vec<Range> {
    let Some(name) = name_at(project, ix, position) else {
        return Vec::new();
    };
    let uri = &project.files[ix].uri;
    uses(project, &name)
        .into_iter()
        .filter(|l| &l.uri == uri)
        .map(|l| l.range)
        .collect()
}

/// Where `name` is used, outside its own declaration: in `.wf` files, and
/// in scripts.
struct Count {
    webfluent: Vec<Location>,
    scripts: Vec<Location>,
}

fn count(project: &Project, name: &str) -> Count {
    let declared = declaration(project, name);
    let mut c = Count {
        webfluent: Vec::new(),
        scripts: Vec::new(),
    };
    for l in uses(project, name) {
        if Some(&l) == declared.as_ref() {
            continue;
        }
        let in_script = project
            .file_index(&l.uri)
            .is_some_and(|i| is_script(project, i));
        if in_script {
            c.scripts.push(l);
        } else {
            c.webfluent.push(l);
        }
    }
    c
}

/// What a lens and a hover say of a name's uses.
fn said(c: &Count) -> String {
    let times = |n: usize| {
        if n == 1 {
            "once".to_string()
        } else {
            format!("{n} times")
        }
    };
    match (c.webfluent.len(), c.scripts.len()) {
        (0, 0) => "Not used".to_string(),
        (w, 0) => format!("Used {} in WebFluent", times(w)),
        (0, s) => format!("Used {} in scripts", times(s)),
        (w, s) => format!("Used {} in WebFluent, {} in scripts", times(w), times(s)),
    }
}

/// The hover on a script's name: who uses it.
pub fn hover(project: &Project, ix: usize, position: Position) -> Option<Hover> {
    let name = name_at(project, ix, position)?;
    let c = count(project, &name);
    let root = project.root.clone().unwrap_or_default();
    let place = |l: &Location| {
        let path = l
            .uri
            .to_file_path()
            .ok()
            .and_then(|p| {
                p.strip_prefix(&root)
                    .ok()
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
            })
            .unwrap_or_else(|| l.uri.path().to_string());
        format!("- `{path}:{}`", l.range.start.line + 1)
    };
    let mut text = format!("**{name}**: {}", said(&c));
    let listed: Vec<String> = c
        .webfluent
        .iter()
        .chain(&c.scripts)
        .take(12)
        .map(place)
        .collect();
    if !listed.is_empty() {
        text.push_str("\n\n");
        text.push_str(&listed.join("\n"));
    }
    let more = c.webfluent.len() + c.scripts.len();
    if more > 12 {
        text.push_str(&format!("\n- … and {} more", more - 12));
    }
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: text,
        }),
        range: None,
    })
}

/// A lens above each name the script declares, saying where it is used;
/// pressing it lists the uses (`webfluent.showReferences`, which an
/// editor's extension maps to its own references view).
pub fn lenses(project: &Project, ix: usize) -> Vec<CodeLens> {
    let file = &project.files[ix];
    names(project, ix)
        .into_iter()
        .map(|n| {
            let range = file
                .index
                .word_range_at_line_col(&file.source, n.line, n.col);
            let c = count(project, &n.name);
            let locations: Vec<Location> = c.webfluent.iter().chain(&c.scripts).cloned().collect();
            CodeLens {
                range,
                command: Some(Command {
                    title: said(&c),
                    command: "webfluent.showReferences".to_string(),
                    arguments: Some(vec![
                        serde_json::json!(file.uri),
                        serde_json::json!(range.start),
                        serde_json::json!(locations),
                    ]),
                }),
                data: None,
            }
        })
        .collect()
}

/// A name the script declares that nothing names — no page, component or
/// store, no other script, and not the script itself — faded, as an editor
/// fades code nothing reaches.
pub fn unused(project: &Project, ix: usize) -> Vec<Diagnostic> {
    let file = &project.files[ix];
    names(project, ix)
        .into_iter()
        .filter(|n| {
            let c = count(project, &n.name);
            c.webfluent.is_empty() && c.scripts.is_empty()
        })
        .map(|n| {
            let what = match n.kind {
                NameKind::Function { .. } => "function",
                NameKind::Class { .. } => "class",
                _ => "name",
            };
            Diagnostic {
                range: file
                    .index
                    .word_range_at_line_col(&file.source, n.line, n.col),
                severity: Some(DiagnosticSeverity::HINT),
                code: Some(NumberOrString::String("unused".to_string())),
                source: Some("webfluent".to_string()),
                message: format!(
                    "The {what} `{}` is not used by any page, component, store or script of the project",
                    n.name
                ),
                tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                ..Diagnostic::default()
            }
        })
        .collect()
}

/// What `webfluent/scriptUses` answers for a script: each name it
/// declares, where, and every place that uses it.
#[derive(Debug, PartialEq)]
pub struct ScriptName {
    pub name: String,
    pub range: Range,
    pub uses: Vec<Location>,
    pub title: String,
}

pub fn script_names(project: &Project, ix: usize) -> Vec<ScriptName> {
    let file = &project.files[ix];
    names(project, ix)
        .into_iter()
        .map(|n| {
            let c = count(project, &n.name);
            ScriptName {
                name: n.name.clone(),
                range: file
                    .index
                    .word_range_at_line_col(&file.source, n.line, n.col),
                title: said(&c),
                uses: c.webfluent.into_iter().chain(c.scripts).collect(),
            }
        })
        .collect()
}

impl ScriptName {
    /// As the request answers it.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "range": self.range,
            "uses": self.uses,
            "title": self.title,
        })
    }
}
