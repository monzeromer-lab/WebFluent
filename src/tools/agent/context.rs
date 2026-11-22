//! What an agent is given, named, so nothing it is told is a mystery:
//! the file tree, the open file and its selection, the problems, every
//! component a call can name, the theme's tokens, the project's config
//! with only its public `env`, and the language reference — the skill's
//! files' titles and sections to cite, or the files whole — and a list of
//! what was withheld and why.

use std::path::Path;

use serde::Serialize;

use crate::base::vfs::Vfs;
use crate::diagnostics::error::Result;

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

/// One file of the reference: `references/core.md`, or a section of the
/// language's, `references/language/data-fetching.md`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Chapter {
    /// Its path in the skill: `references/language/pages.md`.
    pub file: String,
    pub title: String,
    /// Its first paragraph, when it opens with one.
    pub blurb: Option<String>,
    pub sections: Vec<Section>,
    /// The file itself, when the whole reference is given.
    pub text: Option<String>,
}

impl Chapter {
    /// How the agent cites it: `Pages`, or `Pages › Head tags` for a
    /// section.
    pub fn cite(&self, anchor: Option<&str>) -> String {
        match anchor.and_then(|a| self.sections.iter().find(|s| s.anchor == a)) {
            Some(s) => format!("{} › {}", self.title, s.title),
            None => self.title.clone(),
        }
    }
}

/// How much of the reference to give.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    /// Each file's title, blurb and sections: enough to cite, and to
    /// read the one file that is wanted.
    Index,
    /// The files whole: the whole reference.
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

/// The reference's files as chapters: each file of the skill's
/// `references/` (`core.md`, `styling.md`, …, `language/pages.md`), titled
/// by its first heading, with the headings under it as its sections and its
/// first paragraph as its blurb. The files keep the order they are given in.
pub fn chapters(files: &[(String, String)], depth: Depth) -> Vec<Chapter> {
    let mut out = Vec::new();
    for (file, text) in files {
        let mut title = String::new();
        let mut top = 0usize;
        let mut blurb: Vec<&str> = Vec::new();
        let mut blurb_done = false;
        let mut sections = Vec::new();
        let mut fenced = false;
        for line in text.lines() {
            if line.trim_start().starts_with("```") {
                fenced = !fenced;
                blurb_done |= !title.is_empty();
                continue;
            }
            if fenced {
                continue;
            }
            let depth = line.chars().take_while(|c| *c == '#').count();
            let heading =
                (depth > 0 && line[depth..].starts_with(' ')).then(|| line[depth..].trim());
            match heading {
                Some(t) if title.is_empty() => {
                    title = t.replace('`', "");
                    top = depth;
                }
                Some(t) => {
                    blurb_done = true;
                    // A section one or two levels under the file's title.
                    let level = depth.saturating_sub(top) + 1;
                    if (2..=3).contains(&level) {
                        sections.push(Section {
                            anchor: anchor(t),
                            title: t.replace('`', ""),
                            level: u8::try_from(level).unwrap_or(3),
                        });
                    }
                }
                None if title.is_empty() || blurb_done => {}
                None if line.trim().is_empty() => blurb_done = !blurb.is_empty(),
                None if line.trim_start().starts_with(['|', '-', '<']) => blurb_done = true,
                None => blurb.push(line.trim()),
            }
        }
        if title.is_empty() {
            continue;
        }
        out.push(Chapter {
            file: file.clone(),
            title,
            blurb: (!blurb.is_empty()).then(|| blurb.join(" ")),
            sections,
            text: (depth == Depth::Full).then(|| text.clone()),
        });
    }
    out
}

/// The language reference this `wf` was built with: the repository's
/// `AGENTS.md`, which `wf init` writes into every project.
pub fn reference() -> &'static str {
    include_str!("../../../AGENTS.md")
}

/// The reference's core: each section marked `<!-- core -->`, from its
/// heading to the next heading of its level or higher. What an agent is
/// given whole; the rest it asks for by section.
pub fn reference_core() -> String {
    let lines: Vec<&str> = reference().lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() == "<!-- core -->" && i + 1 < lines.len() {
            let level = lines[i + 1].chars().take_while(|c| *c == '#').count();
            let mut j = i + 1;
            out.push(lines[j].to_string());
            let mut fenced = false;
            j += 1;
            while j < lines.len() {
                let l = lines[j];
                if l.starts_with("```") {
                    fenced = !fenced;
                }
                let depth = l.chars().take_while(|c| *c == '#').count();
                if !fenced
                    && ((depth > 0 && depth <= level && l[depth..].starts_with(' '))
                        || l.trim() == "<!-- core -->")
                {
                    break;
                }
                out.push(l.to_string());
                j += 1;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out.join("\n").trim().to_string()
}

/// A section of the reference by its heading's title (`Data Fetching`),
/// from its heading to the next of its level or higher, and the titles
/// of all of them when none is named so.
///
/// # Errors
///
/// The title of every section, when none is called `title`.
pub fn reference_section(title: &str) -> std::result::Result<String, Vec<String>> {
    let lines: Vec<&str> = reference().lines().collect();
    let mut fenced = false;
    let mut titles = Vec::new();
    let mut found: Option<(usize, usize)> = None;
    for (i, l) in lines.iter().enumerate() {
        if l.starts_with("```") {
            fenced = !fenced;
        }
        if fenced {
            continue;
        }
        let depth = l.chars().take_while(|c| *c == '#').count();
        if (2..=3).contains(&depth) && l[depth..].starts_with(' ') {
            let t = l[depth..].trim().replace('`', "");
            if found.is_none()
                && (t.eq_ignore_ascii_case(title.trim()) || anchor(&t) == anchor(title))
            {
                found = Some((i, depth));
            }
            titles.push(t);
        }
    }
    let Some((start, level)) = found else {
        return Err(titles);
    };
    let mut out = vec![lines[start]];
    let mut fenced = false;
    for l in &lines[start + 1..] {
        if l.starts_with("```") {
            fenced = !fenced;
        }
        let depth = l.chars().take_while(|c| *c == '#').count();
        if !fenced && depth > 0 && depth <= level && l[depth..].starts_with(' ') {
            break;
        }
        out.push(l);
    }
    Ok(out
        .join("\n")
        .trim_end()
        .replace("<!-- core -->", "")
        .trim_end()
        .to_string())
}

/// The guide this `wf` was built with: the language reference as the
/// `writing-webfluent` skill gives it to a project (`wf init` writes the
/// same files), each file of its `references/` by its path in the skill —
/// `references/core.md` first, then the topical files, then
/// `references/language/<section>.md` for each section of `AGENTS.md`. The
/// skill's index of those sections, `references/language.md`, is not a
/// chapter: the chapters are its entries.
pub fn guide() -> &'static [(String, String)] {
    static GUIDE: std::sync::OnceLock<Vec<(String, String)>> = std::sync::OnceLock::new();
    GUIDE.get_or_init(|| {
        crate::tools::init::skill_files()
            .into_iter()
            .filter(|(path, _)| path.starts_with("references/") && path != "references/language.md")
            .collect()
    })
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
    pub components: crate::tools::registry_schema::Catalog,
    /// The theme's tokens and their values.
    pub tokens: Vec<(String, String)>,
    /// `webfluent.app.json`, its private `env` values withheld.
    pub config: serde_json::Value,
    /// The language reference: each file of the skill's `references/`.
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
            if skipped(&rel) || super::leaves(dir, &p) {
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
/// the editor (a path inside the project) and its selection, and the reference
/// to the given depth.
///
/// # Errors
///
/// What [`ProjectConfig::load_if_present_via`](crate::project::config::ProjectConfig::load_if_present_via)
/// returns: the project's `webfluent.app.json` is there but cannot be read
/// or is not a config.
pub fn context_via(
    vfs: &dyn Vfs,
    dir: &Path,
    open: Option<(&str, Option<(usize, usize)>)>,
    guide_files: &[(String, String)],
    depth: Depth,
) -> Result<Context> {
    let mut withheld = Vec::new();
    let config_text = vfs
        .read_to_string(&dir.join("webfluent.app.json"))
        .unwrap_or_else(|_| "{}".into());
    let config = crate::project::config::ProjectConfig::load_if_present_via(vfs, dir)?;
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
        if skipped(file) || super::escapes(dir, &dir.join(file)) {
            return None;
        }
        let text = vfs.read_to_string(&dir.join(file)).ok()?;
        let text = if file == "webfluent.app.json" {
            serde_json::to_string_pretty(&in_order(&shown)).unwrap_or_default()
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
    // A finding in a file that leads out of the project may quote what is
    // out there: it is not given.
    let problems = report
        .diagnostics
        .iter()
        .filter(|d| !super::escapes(dir, &dir.join(&d.file)))
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
    let tokens = crate::tools::theme_view::view_via(vfs, dir)
        .map(|v| v.tokens.into_iter().map(|t| (t.name, t.value)).collect())
        .unwrap_or_default();
    Ok(Context {
        project: name,
        files: tree(vfs, dir),
        open,
        problems,
        components: crate::tools::registry_schema::of_project_via(vfs, dir),
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
            serde_json::to_string_pretty(&in_order(&self.config)).unwrap_or_default()
        ));
        out.push_str("\n## The reference\n\n");
        if self.guide.iter().any(|ch| ch.text.is_none()) {
            out.push_str(
                "Each file of the `writing-webfluent` skill's reference, by its path in the \
                 skill (a project `wf init` made has it in `.claude/skills/writing-webfluent/` \
                 and `.agents/skills/writing-webfluent/`): read the one a task needs, or run \
                 `wf context --guide full` for all of them.\n\n",
            );
        }
        for ch in &self.guide {
            out.push_str(&format!("### {} (`{}`)\n\n", ch.title, ch.file));
            if let Some(t) = &ch.text {
                out.push_str(t.trim_end());
                out.push_str("\n\n");
            } else {
                if let Some(b) = &ch.blurb {
                    out.push_str(&format!("{b}\n\n"));
                }
                let sections: Vec<&str> = ch
                    .sections
                    .iter()
                    .filter(|s| s.level == 2)
                    .map(|s| s.title.as_str())
                    .collect();
                if !sections.is_empty() {
                    out.push_str(&format!("Sections: {}\n\n", sections.join(" · ")));
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

/// A value with its keys sorted, at every depth. What an agent is given has
/// to read the same on every run, and how `serde_json` orders a map depends on
/// a feature that any crate in the build can turn on.
fn in_order(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut sorted = serde_json::Map::new();
            for k in keys {
                sorted.insert(k.clone(), in_order(&map[k]));
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(in_order).collect())
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod in_order_tests {
    #[test]
    fn a_config_reads_with_its_keys_in_order_at_every_depth() {
        let v = serde_json::json!({ "z": 1, "a": { "y": [{ "c": 1, "b": 2 }], "x": 2 } });
        let text = serde_json::to_string(&super::in_order(&v)).unwrap();
        assert_eq!(text, r#"{"a":{"x":2,"y":[{"b":2,"c":1}]},"z":1}"#);
    }
}
