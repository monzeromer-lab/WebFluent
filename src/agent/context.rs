//! What an agent is given, named, so nothing it is told is a mystery:
//! the file tree, the open file and its selection, the problems, every
//! component a call can name, the theme's tokens, the project's config
//! with only its public `env`, and the guide — its chapters' titles and
//! sections to cite, or the chapters whole — and a list of what was
//! withheld and why.

use std::path::Path;

use serde::Serialize;

use crate::error::Result;
use crate::vfs::Vfs;

/// The file open in the editor, and what is selected in it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Open {
    pub file: String,
    pub text: String,
    /// The selection as byte offsets, and its text.
    pub selection: Option<(usize, usize)>,
    pub selected: Option<String>,
}

/// One finding, as the Problems list shows it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Problem {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub hint: Option<String>,
}

/// One section of a chapter: what a citation names.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Section {
    /// The anchor the site gives it: `design-tokens-and-the-theme`.
    pub anchor: String,
    pub title: String,
    /// 2 for a `##`, 3 for a `###`.
    pub level: u8,
}

/// One chapter of the guide.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Chapter {
    pub number: u32,
    pub file: String,
    pub title: String,
    /// Where the site has it: `guide/forms`.
    pub route: Option<String>,
    pub blurb: Option<String>,
    pub sections: Vec<Section>,
    /// The chapter itself, when the whole guide is given.
    pub text: Option<String>,
}

impl Chapter {
    /// How the agent cites it: `16 Forms`, or `16 Forms › validate` for a
    /// section.
    pub fn cite(&self, anchor: Option<&str>) -> String {
        let head = format!("{} {}", self.number, self.title);
        match anchor.and_then(|a| self.sections.iter().find(|s| s.anchor == a)) {
            Some(s) => format!("{head} › {}", s.title),
            None => head,
        }
    }
}

/// How much of the guide to give.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    /// Each chapter's title, blurb and sections: enough to cite, and to
    /// ask for a chapter by name.
    Index,
    /// The chapters whole.
    Full,
}

/// The anchor the site gives a heading: lower case, each run of anything
/// but a letter or a digit one `-`.
pub fn anchor(title: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in title.chars() {
        if c.is_alphanumeric() {
            if dash && !out.is_empty() {
                out.push('-');
            }
            dash = false;
            out.extend(c.to_lowercase());
        } else {
            dash = true;
        }
    }
    out
}

/// The guide's chapters, read from its files (`md-docs/NN-slug.md`).
pub fn chapters(files: &[(&str, &str)], depth: Depth) -> Vec<Chapter> {
    let mut out = Vec::new();
    for (file, text) in files {
        let Some((digits, _)) = file.split_once('-') else {
            continue;
        };
        let Ok(number) = digits.parse::<u32>() else {
            continue;
        };
        let mut title = String::new();
        let mut route = None;
        let mut blurb = None;
        let mut sections = Vec::new();
        let mut fenced = false;
        let mut meta = false;
        for line in text.lines() {
            if line.starts_with("```") {
                fenced = !fenced;
                continue;
            }
            if fenced {
                continue;
            }
            if line.starts_with("<!--") {
                meta = true;
                continue;
            }
            if meta {
                if line.starts_with("-->") {
                    meta = false;
                } else if let Some(r) = line.strip_prefix("route:") {
                    route = Some(r.trim().to_string());
                } else if let Some(b) = line.strip_prefix("blurb:") {
                    blurb = Some(b.trim().to_string());
                }
                continue;
            }
            if let Some(t) = line.strip_prefix("# ") {
                if title.is_empty() {
                    title = t
                        .split_once(". ")
                        .map(|(_, t)| t)
                        .unwrap_or(t)
                        .trim()
                        .to_string();
                }
            } else if let Some(t) = line.strip_prefix("## ")
                // `## Next` is the link to the next chapter, not a section.
                && t.trim() != "Next"
            {
                sections.push(Section {
                    anchor: anchor(t),
                    title: t.trim().replace('`', ""),
                    level: 2,
                });
            } else if let Some(t) = line.strip_prefix("### ") {
                sections.push(Section {
                    anchor: anchor(t),
                    title: t.trim().replace('`', ""),
                    level: 3,
                });
            }
        }
        out.push(Chapter {
            number,
            file: file.to_string(),
            title,
            route,
            blurb,
            sections,
            text: (depth == Depth::Full).then(|| text.to_string()),
        });
    }
    out
}

mod bundled {
    include!(concat!(env!("OUT_DIR"), "/guide_files.rs"));
}

/// The guide this `wf` was built with: every chapter of `md-docs/`.
pub fn guide() -> &'static [(&'static str, &'static str)] {
    bundled::FILES
}

/// Everything an agent is given.
#[derive(Debug, Clone, Serialize)]
pub struct Context {
    /// The project's name.
    pub project: String,
    /// Every file of the project, by path inside it — but the build's
    /// output, `.git/`, dependencies and `.env` files.
    pub files: Vec<String>,
    pub open: Option<Open>,
    pub problems: Vec<Problem>,
    /// Every component a call can name, built-in and the project's, with
    /// what each takes (`wf components`).
    pub components: crate::registry::schema::Catalog,
    /// The theme's tokens and their values.
    pub tokens: Vec<(String, String)>,
    /// `webfluent.app.json`, its private `env` values withheld.
    pub config: serde_json::Value,
    pub guide: Vec<Chapter>,
    /// What was not given, and why.
    pub withheld: Vec<String>,
}

fn skipped(rel: &str) -> bool {
    let first = rel.split('/').next().unwrap_or("");
    matches!(
        first,
        ".git" | "build" | ".wf-cache" | "node_modules" | "dist" | "target"
    ) || rel
        .rsplit('/')
        .next()
        .is_some_and(|n| n == ".env" || n.starts_with(".env."))
}

fn tree(vfs: &dyn Vfs, dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = vfs.read_dir(&d) else {
            continue;
        };
        for p in entries {
            let rel = p
                .strip_prefix(dir)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            if skipped(&rel) {
                continue;
            }
            if vfs.is_dir(&p) {
                stack.push(p);
            } else {
                out.push(rel);
            }
        }
    }
    out.sort();
    out
}

/// What an agent is given for the project at `dir`, as the editor holds
/// it (`vfs`: an unsaved buffer is the one read), with `open` the file in
/// the editor (a path inside the project) and its selection, and the guide
/// to the given depth.
pub fn context_via(
    vfs: &dyn Vfs,
    dir: &Path,
    open: Option<(&str, Option<(usize, usize)>)>,
    guide_files: &[(&str, &str)],
    depth: Depth,
) -> Result<Context> {
    let mut withheld = Vec::new();
    let config_text = vfs
        .read_to_string(&dir.join("webfluent.app.json"))
        .unwrap_or_else(|_| "{}".into());
    let config = crate::config::ProjectConfig::load_via(vfs, dir).ok();
    // The config, each private `env` value withheld.
    let mut shown: serde_json::Value =
        serde_json::from_str(&config_text).unwrap_or(serde_json::json!({}));
    if let (Some(cfg), Some(env)) = (
        &config,
        shown.get_mut("env").and_then(|e| e.as_object_mut()),
    ) {
        let mut private: Vec<String> = Vec::new();
        for (name, value) in env.iter_mut() {
            if !cfg.env_is_public(name) {
                *value = serde_json::Value::String(super::changeset::WITHHELD.into());
                private.push(name.clone());
            }
        }
        if !private.is_empty() {
            private.sort();
            withheld.push(format!(
                "The values of `env` that `public_env` does not list ({}): build-time configuration that never reaches a browser.",
                private.join(", ")
            ));
        }
    }
    withheld.push(
        "Anything outside the project, `.git/`, the build's output, and any `.env` file.".into(),
    );
    withheld.push("The GitHub token and any other credential the editor holds.".into());
    let name = config.as_ref().map(|c| c.name.clone()).unwrap_or_default();

    let open = open.and_then(|(file, selection)| {
        if skipped(file) {
            return None;
        }
        let text = vfs.read_to_string(&dir.join(file)).ok()?;
        let text = if file == "webfluent.app.json" {
            serde_json::to_string_pretty(&shown).unwrap_or_default()
        } else {
            text
        };
        let selected =
            selection.and_then(|(a, b)| text.get(a.min(b)..a.max(b)).map(str::to_string));
        Some(Open {
            file: file.to_string(),
            text,
            selection,
            selected,
        })
    });

    let options = crate::build::BuildOptions::new(dir, vfs);
    let report = match crate::build::check(&options) {
        Ok(r) => r,
        Err(f) => *f.report,
    };
    let problems = report
        .diagnostics
        .iter()
        .map(|d| Problem {
            code: d.code.to_string(),
            severity: format!("{:?}", d.severity).to_lowercase(),
            message: d.message.clone(),
            file: d.file.clone(),
            line: d.line,
            column: d.column,
            hint: d.hint.clone(),
        })
        .collect();
    let tokens = crate::themes::view::view_via(vfs, dir)
        .map(|v| v.tokens.into_iter().map(|t| (t.name, t.value)).collect())
        .unwrap_or_default();
    Ok(Context {
        project: name,
        files: tree(vfs, dir),
        open,
        problems,
        components: crate::registry::schema::of_project_via(vfs, dir),
        tokens,
        config: shown,
        guide: chapters(guide_files, depth),
        withheld,
    })
}

impl Context {
    /// The context as one Markdown document, for a provider that takes text.
    pub fn to_markdown(&self) -> String {
        let mut out = format!("# The project `{}`\n\n## Files\n\n", self.project);
        for f in &self.files {
            out.push_str(&format!("- `{f}`\n"));
        }
        if let Some(o) = &self.open {
            out.push_str(&format!(
                "\n## Open: `{}`\n\n```\n{}\n```\n",
                o.file,
                o.text.trim_end()
            ));
            if let Some(s) = &o.selected {
                out.push_str(&format!("\nSelected:\n\n```\n{s}\n```\n"));
            }
        }
        out.push_str(&format!("\n## Problems ({})\n\n", self.problems.len()));
        for p in &self.problems {
            out.push_str(&format!(
                "- {} {}:{}:{} {}\n",
                p.code, p.file, p.line, p.column, p.message
            ));
        }
        out.push_str("\n## Components\n\n");
        for c in &self.components.components {
            let props: Vec<String> = c
                .props
                .iter()
                .map(|p| {
                    if p.cases.is_empty() {
                        format!("{}: {}", p.name, p.ty)
                    } else {
                        let cases: Vec<String> =
                            p.cases.iter().map(|x| format!(".{}", x.name)).collect();
                        format!("{}: {}", p.name, cases.join(" | "))
                    }
                })
                .collect();
            let mut line = format!("- `{}`({})", c.name, props.join(", "));
            if !c.events.is_empty() {
                let events: Vec<&str> = c.events.iter().map(|e| e.name.as_str()).collect();
                line.push_str(&format!(" · on {}", events.join(", ")));
            }
            if !c.slots.is_empty() {
                let slots: Vec<&str> = c.slots.iter().map(|s| s.name.as_str()).collect();
                line.push_str(&format!(" · slots {}", slots.join(", ")));
            }
            if let Some(s) = &c.summary {
                line.push_str(&format!(" — {s}"));
            }
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str("\n## Theme tokens\n\n");
        for (n, v) in &self.tokens {
            out.push_str(&format!("- `--{n}`: `{v}`\n"));
        }
        out.push_str(&format!(
            "\n## webfluent.app.json\n\n```json\n{}\n```\n",
            serde_json::to_string_pretty(&self.config).unwrap_or_default()
        ));
        out.push_str("\n## The guide\n\n");
        for ch in &self.guide {
            out.push_str(&format!("### {} {}\n\n", ch.number, ch.title));
            if let Some(b) = &ch.blurb {
                out.push_str(&format!("{b}\n\n"));
            }
            match &ch.text {
                Some(t) => {
                    out.push_str(t);
                    out.push('\n');
                }
                None => {
                    for s in ch.sections.iter().filter(|s| s.level == 2) {
                        out.push_str(&format!("- {}\n", s.title));
                    }
                    out.push('\n');
                }
            }
        }
        out.push_str("## Withheld\n\n");
        for w in &self.withheld {
            out.push_str(&format!("- {w}\n"));
        }
        out
    }
}
