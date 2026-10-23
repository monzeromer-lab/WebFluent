//! `wf init`: a new project from a template — a single-page app, a static
//! site, a PDF, or a slide deck — written as a folder of its own. `wf init`
//! makes one in the current directory; the mobile editor, in its projects.

use crate::error::{Result, WebFluentError};
use std::fs;
use std::path::Path;

/// What a new project starts as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    /// An interactive dashboard: pages, components, stores (`spa`).
    Spa,
    /// A static site, every page prerendered (`static`).
    Static,
    /// A document rendered to PDF (`pdf`).
    Pdf,
    /// A slide deck (`slides`).
    Slides,
}

impl Template {
    pub const ALL: [Template; 4] = [
        Template::Spa,
        Template::Static,
        Template::Pdf,
        Template::Slides,
    ];

    /// The template `wf init --template` names.
    pub fn parse(name: &str) -> Option<Template> {
        match name {
            "spa" => Some(Template::Spa),
            "static" => Some(Template::Static),
            "pdf" => Some(Template::Pdf),
            "slides" => Some(Template::Slides),
            _ => None,
        }
    }

    /// Its name for `--template`.
    pub fn name(self) -> &'static str {
        match self {
            Template::Spa => "spa",
            Template::Static => "static",
            Template::Pdf => "pdf",
            Template::Slides => "slides",
        }
    }
}

/// Write a new project named `name` into `dir`, which must not exist yet.
pub fn create(dir: &Path, name: &str, template: Template) -> Result<()> {
    if dir.exists() {
        return Err(WebFluentError::IoError(format!(
            "Directory '{}' already exists",
            dir.display()
        )));
    }
    for (path, text) in template.files() {
        let path = dir.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, text.replace("{{name}}", name))?;
    }
    // The folder static files go in, there from the start so it is found.
    if matches!(template, Template::Spa | Template::Static) {
        fs::create_dir_all(dir.join("public"))?;
    }
    // What a build writes and what a machine holds, kept out of version
    // control from the first commit: the output, the image cache, the sizes
    // the last build weighed, and the `.env` a build reads values from.
    fs::write(
        dir.join(".gitignore"),
        "build/\n.wf-cache/\n.wf-sizes.json\n.env\n",
    )?;
    // The language reference, where a coding agent looks for it — so the
    // one asked to change this project writes the WebFluent of the `wf`
    // that made it, rather than guessing at a language it may never have
    // seen.
    fs::write(dir.join("AGENTS.md"), agents_md())?;
    Ok(())
}

/// The repository's `AGENTS.md`, as this `wf` was built with it.
const AGENTS: &str = include_str!("../AGENTS.md");

/// `AGENTS.md` for a new project: the language reference, headed by the
/// version it describes and where to find a newer one.
pub fn agents_md() -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        "<!-- The WebFluent {version} language reference, written by `wf init`. \
         A newer wf brings a newer one: \
         https://github.com/monzeromer-lab/WebFluent/blob/master/AGENTS.md -->\n\n{AGENTS}"
    )
}

// ═══════════════════════════════════════════════════════════
//  The starters
// ═══════════════════════════════════════════════════════════
//
// Each starter is a folder of real files under `src/init/<template>/`, the
// project's name written `{{name}}`. They used to be Rust string literals
// with every brace doubled, which no editor, formatter or `wf check` could
// read — so the starters drifted from the language and shipped bugs a
// reader met on their first click. As files they are a project: `wf check`,
// `wf build` and `wf test` run on them as they stand (the e2e tests do).

impl Template {
    /// Every file the template writes, by its path in the new project.
    pub fn files(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Template::Spa => SPA,
            Template::Static => STATIC,
            Template::Pdf => PDF,
            Template::Slides => SLIDES,
        }
    }
}

const SPA: &[(&str, &str)] = &[
    ("src/App.wf", include_str!("init/spa/src/App.wf")),
    ("src/components/StatCard.wf", include_str!("init/spa/src/components/StatCard.wf")),
    ("src/components/TaskItem.wf", include_str!("init/spa/src/components/TaskItem.wf")),
    ("src/components/ThemeToggle.wf", include_str!("init/spa/src/components/ThemeToggle.wf")),
    ("src/pages/Home.wf", include_str!("init/spa/src/pages/Home.wf")),
    ("src/pages/Profile.wf", include_str!("init/spa/src/pages/Profile.wf")),
    ("src/pages/Settings.wf", include_str!("init/spa/src/pages/Settings.wf")),
    ("src/pages/Tasks.wf", include_str!("init/spa/src/pages/Tasks.wf")),
    ("src/stores/auth.wf", include_str!("init/spa/src/stores/auth.wf")),
    ("src/stores/tasks.wf", include_str!("init/spa/src/stores/tasks.wf")),
    ("src/styles.css", include_str!("init/spa/src/styles.css")),
    ("src/theme.wf", include_str!("init/spa/src/theme.wf")),
    ("tests/app.wf", include_str!("init/spa/tests/app.wf")),
    ("tests/tasks.wf", include_str!("init/spa/tests/tasks.wf")),
    ("webfluent.app.json", include_str!("init/spa/webfluent.app.json")),
];

const STATIC: &[(&str, &str)] = &[
    ("src/App.wf", include_str!("init/static/src/App.wf")),
    ("src/components/FeatureCard.wf", include_str!("init/static/src/components/FeatureCard.wf")),
    ("src/components/SiteFooter.wf", include_str!("init/static/src/components/SiteFooter.wf")),
    ("src/components/TeamMember.wf", include_str!("init/static/src/components/TeamMember.wf")),
    ("src/pages/About.wf", include_str!("init/static/src/pages/About.wf")),
    ("src/pages/Blog.wf", include_str!("init/static/src/pages/Blog.wf")),
    ("src/pages/Contact.wf", include_str!("init/static/src/pages/Contact.wf")),
    ("src/pages/Home.wf", include_str!("init/static/src/pages/Home.wf")),
    ("src/translations/ar.json", include_str!("init/static/src/translations/ar.json")),
    ("src/translations/en.json", include_str!("init/static/src/translations/en.json")),
    ("tests/site.wf", include_str!("init/static/tests/site.wf")),
    ("webfluent.app.json", include_str!("init/static/webfluent.app.json")),
];

const PDF: &[(&str, &str)] = &[
    ("src/pages/Report.wf", include_str!("init/pdf/src/pages/Report.wf")),
    ("webfluent.app.json", include_str!("init/pdf/webfluent.app.json")),
];

const SLIDES: &[(&str, &str)] = &[
    ("src/pages/Deck.wf", include_str!("init/slides/src/pages/Deck.wf")),
    ("webfluent.app.json", include_str!("init/slides/webfluent.app.json")),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every file under `src/init/<template>/` is one the template writes:
    /// a file added to a starter and left out of its table is a starter that
    /// does not build (its theme named in the config, and not there).
    #[test]
    fn every_starter_file_is_written() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/init");
        for template in Template::ALL {
            let dir = root.join(template.name());
            let mut on_disk = Vec::new();
            let mut stack = vec![dir.clone()];
            while let Some(d) = stack.pop() {
                for entry in fs::read_dir(&d).unwrap() {
                    let path = entry.unwrap().path();
                    if path.is_dir() {
                        stack.push(path);
                    } else {
                        let rel = path.strip_prefix(&dir).unwrap();
                        on_disk.push(rel.to_string_lossy().replace('\\', "/"));
                    }
                }
            }
            on_disk.sort();
            let mut listed: Vec<String> = template.files().iter().map(|(p, _)| p.to_string()).collect();
            listed.sort();
            assert_eq!(on_disk, listed, "the `{}` starter's table and its folder differ", template.name());
        }
    }
}
