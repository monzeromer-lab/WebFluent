//! `wf init`: a new project from a template.
//!
//! `wf init`: a new project from a template — a single-page app, a static
//! site, a PDF, or a slide deck — written as a folder of its own. `wf init`
//! makes one in the current directory; the mobile editor, in its projects.

use crate::base::outfs::At;
use crate::diagnostics::error::{Result, WebFluentError};
use std::fs;
use std::path::Path;

/// What a new project starts as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
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

/// How a new project starts: its template, and the languages it speaks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub template: Template,
    /// The locales, the default first; empty for the template as it is.
    /// Given, the project's visible strings go through `t("…")`, its
    /// `i18n` names these, and `src/translations/` holds a file for each.
    pub locales: Vec<String>,
}

impl Options {
    /// The template as it is, in the languages it comes in.
    pub fn new(template: Template) -> Options {
        Options {
            template,
            locales: Vec::new(),
        }
    }
}

/// Write a new project named `name` into `dir`, which must not exist yet.
///
/// # Errors
///
/// [`WebFluentError::IoError`] when `dir` exists or a file cannot be written.
pub fn create(dir: &Path, name: &str, template: Template) -> Result<()> {
    create_with(dir, name, &Options::new(template))
}

/// Write a new project named `name` into `dir`, which must not exist yet,
/// as `options` say — translated into `options.locales` when it names any.
///
/// # Errors
///
/// [`WebFluentError::ConfigError`] when `options.locales` names a locale
/// that is not one, names one twice, or is given to a PDF or slides
/// template; [`WebFluentError::IoError`] when `dir` exists or a file
/// cannot be written.
pub fn create_with(dir: &Path, name: &str, options: &Options) -> Result<()> {
    let template = options.template;
    let locales = if options.locales.is_empty() {
        Vec::new()
    } else {
        if !template.translates() {
            return Err(WebFluentError::config(format!(
                "The `{}` template does not take locales: a {} is made in one \
                 language. Start it in that language, or use `spa` or `static` \
                 for a project in several.",
                template.name(),
                if template == Template::Pdf {
                    "PDF"
                } else {
                    "slide deck"
                }
            )));
        }
        check_locales(&options.locales)?
    };
    if dir.exists() {
        return Err(WebFluentError::io(format!(
            "Directory '{}' already exists",
            dir.display()
        )));
    }
    for (path, text) in project_files(template, &locales)? {
        let path = dir.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).at(parent)?;
        }
        fs::write(&path, text.replace("{{name}}", name)).at(&path)?;
    }
    // The folder static files go in, there from the start so it is found.
    if matches!(template, Template::Spa | Template::Static) {
        fs::create_dir_all(dir.join("public")).at(dir.join("public"))?;
    }
    // What a build writes and what a machine holds, kept out of version
    // control from the first commit: the output, the image cache, the sizes
    // the last build weighed, and the `.env` a build reads values from.
    fs::write(
        dir.join(".gitignore"),
        "build/\n.wf-cache/\n.wf-sizes.json\n.env\n",
    )?;
    // The core of the language, where a coding agent looks for it — so the
    // one asked to change this project writes the WebFluent of the `wf`
    // that made it, rather than guessing at a language it may never have
    // seen. Short on purpose: many agents read `AGENTS.md` into every turn.
    fs::write(dir.join("AGENTS.md"), agents_md()).at(dir.join("AGENTS.md"))?;
    // The skill, in the folders agents look in for one: `.agents/skills/`
    // (Codex, Cursor, Gemini CLI and the agents that follow the standard)
    // and `.claude/skills/` (Claude Code). It holds the whole reference, a
    // file a section, so an agent reads the section it needs.
    for root in SKILL_ROOTS {
        for (path, text) in skill_files() {
            let path = dir.join(root).join(&path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).at(parent)?;
            }
            fs::write(&path, text).at(&path)?;
        }
    }
    Ok(())
}

/// Where a new project keeps its skill, relative to the project.
pub const SKILL_ROOTS: [&str; 2] = [
    ".agents/skills/writing-webfluent",
    ".claude/skills/writing-webfluent",
];

/// The `writing-webfluent` skill (`skills/writing-webfluent/` in the
/// repository), as this `wf` was built with it.
const SKILL: &[(&str, &str)] = &[
    (
        "SKILL.md",
        include_str!("../../skills/writing-webfluent/SKILL.md"),
    ),
    (
        "references/core.md",
        include_str!("../../skills/writing-webfluent/references/core.md"),
    ),
    (
        "references/code-quality.md",
        include_str!("../../skills/writing-webfluent/references/code-quality.md"),
    ),
    (
        "references/diagnostics.md",
        include_str!("../../skills/writing-webfluent/references/diagnostics.md"),
    ),
    (
        "references/documents.md",
        include_str!("../../skills/writing-webfluent/references/documents.md"),
    ),
    (
        "references/recipes.md",
        include_str!("../../skills/writing-webfluent/references/recipes.md"),
    ),
    (
        "references/styling.md",
        include_str!("../../skills/writing-webfluent/references/styling.md"),
    ),
    (
        "references/tauri-apps.md",
        include_str!("../../skills/writing-webfluent/references/tauri-apps.md"),
    ),
    (
        "references/testing.md",
        include_str!("../../skills/writing-webfluent/references/testing.md"),
    ),
];

/// Every file of the skill a new project gets, by its path in the skill's
/// folder: the hand-written files, and the reference split by section
/// ([`reference_files`]).
pub fn skill_files() -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = SKILL
        .iter()
        .map(|(p, t)| ((*p).to_string(), (*t).to_string()))
        .collect();
    files.extend(reference_files(AGENTS));
    files
}

/// The language reference (`AGENTS.md`), a file a section, under the
/// skill's `references/`: `language.md` lists them with their sizes, and
/// `language/<section>.md` holds each — a `##` section, or for the long
/// `## Core Syntax`, a `###` one. An agent reads one section rather than
/// the whole reference, which would sit in its context for every turn after.
/// The repository's `skills/writing-webfluent/references/` is written by
/// this function (`just bump`), and its tests hold the two equal.
pub fn reference_files(agents: &str) -> Vec<(String, String)> {
    // (level, title, text) of each section, fenced code not counted.
    let mut sections: Vec<(u8, String, String)> = Vec::new();
    let mut fenced = false;
    let mut in_core = false;
    for line in agents.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        let heading = if fenced {
            None
        } else if let Some(t) = line.strip_prefix("## ") {
            // The two long chapters are a file a subsection, so each stays one an
            // agent can afford to read.
            in_core = matches!(t.trim(), "Core Syntax" | "Built-in Components");
            Some((2u8, t.trim().to_string()))
        } else if in_core {
            line.strip_prefix("### ")
                .map(|t| (3u8, t.trim().to_string()))
        } else {
            None
        };
        match heading {
            Some((level, title)) => sections.push((level, title, line.to_string())),
            // The title and the lines before the first section go nowhere.
            None => {
                if let Some(last) = sections.last_mut() {
                    last.2.push_str(line);
                }
            }
        }
    }
    let mut taken = std::collections::BTreeSet::new();
    let mut files = Vec::new();
    let mut rows = String::new();
    for (level, title, text) in &sections {
        let mut slug = slug(title);
        while !taken.insert(slug.clone()) {
            slug.push_str("-2");
        }
        let indent = if *level == 3 { "  " } else { "" };
        rows.push_str(&format!(
            "{indent}- [{title}](language/{slug}.md) — {} kB\n",
            text.len().div_ceil(1000)
        ));
        files.push((format!("references/language/{slug}.md"), text.clone()));
    }
    let version = env!("CARGO_PKG_VERSION");
    let index = format!(
        "# The WebFluent {version} reference, a file a section\n\n\
         Read [core.md](core.md) first: it covers most sites and apps. For \
         anything else, read only the section you need — every file you read \
         stays in your context for the rest of the session, and the whole \
         reference is about {} kB. Or ask the compiler: `wf components Name`, \
         `wf explain CODE`, `wf scope FILE:LINE`, `wf config schema`.\n\n{rows}",
        agents.len().div_ceil(1000)
    );
    let mut out = vec![("references/language.md".to_string(), index)];
    out.extend(files);
    out
}

/// A section's file name: its title in lower case, a word joined to the
/// next by `-`, punctuation dropped (`App (Router + Layout)` is
/// `app-router-layout`).
fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

/// The locales of `wf init --locales en,ar`, checked: the default first.
///
/// # Errors
///
/// What [`check_locales`] returns for the list.
pub fn parse_locales(list: &str) -> Result<Vec<String>> {
    let locales: Vec<String> = list
        .split(',')
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    check_locales(&locales)
}

/// `locales` as a project's `i18n` names them: each a language tag —
/// `en`, `ar`, `pt-BR`, `zh-Hant` — written the usual way (the language in
/// lower case, a region in upper, a script in title case), none twice, and
/// at least one. The first is the default.
///
/// # Errors
///
/// [`WebFluentError::ConfigError`] when there is none, one is not a
/// language tag, or one is named twice.
pub fn check_locales(locales: &[String]) -> Result<Vec<String>> {
    if locales.is_empty() {
        return Err(WebFluentError::config(
            "No locales: name at least one, the default first (`en,ar`).",
        ));
    }
    let mut out: Vec<String> = Vec::new();
    for given in locales {
        let Some(tag) = locale_tag(given) else {
            let hint = if given.contains('_') {
                format!(
                    " (a tag is written with `-`: `{}`)",
                    given.replace('_', "-")
                )
            } else {
                String::new()
            };
            return Err(WebFluentError::config(format!(
                "`{given}` is not a locale: a language tag is two or three \
                 letters, then a region or script if it needs one, as in `en`, \
                 `ar`, `pt-BR`{hint}."
            )));
        };
        if out.contains(&tag) {
            return Err(WebFluentError::config(format!(
                "`{tag}` is named twice: each locale once, the default first."
            )));
        }
        out.push(tag);
    }
    Ok(out)
}

/// `given` as a language tag in its usual case, or nothing when it is not
/// one: a language of two or three letters, then subtags of letters or
/// digits — a four-letter script, a two-letter or three-digit region, a
/// variant of five to eight.
fn locale_tag(given: &str) -> Option<String> {
    let mut parts = given.split('-');
    let language = parts.next()?;
    if !(2..=3).contains(&language.len()) || !language.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let mut tag = language.to_ascii_lowercase();
    for part in parts {
        let alnum = part.chars().all(|c| c.is_ascii_alphanumeric());
        let alpha = part.chars().all(|c| c.is_ascii_alphabetic());
        let digits = part.chars().all(|c| c.is_ascii_digit());
        let written = match part.len() {
            2 if alpha => part.to_ascii_uppercase(),
            3 if digits => part.to_string(),
            4 if alpha => {
                let lower = part.to_ascii_lowercase();
                format!("{}{}", lower[..1].to_ascii_uppercase(), &lower[1..])
            }
            5..=8 if alnum => part.to_ascii_lowercase(),
            _ => return None,
        };
        tag.push('-');
        tag.push_str(&written);
    }
    Some(tag)
}

/// Every file of a new project, by its path, before `{{name}}` is filled:
/// the template's own, or — with locales — its translated form.
fn project_files(template: Template, locales: &[String]) -> Result<Vec<(String, String)>> {
    let mut files: Vec<(String, String)> = template
        .files()
        .iter()
        .map(|(p, t)| (p.to_string(), t.to_string()))
        .collect();
    if locales.is_empty() {
        return Ok(files);
    }
    for (path, text) in template.translated() {
        match files.iter_mut().find(|(p, _)| p == path) {
            Some(file) => file.1 = text.to_string(),
            None => files.push((path.to_string(), text.to_string())),
        }
    }
    // The tables the starter comes with, by locale: its English, and the
    // languages it has been translated into.
    let mut tables: Vec<(String, String)> = Vec::new();
    files.retain(|(path, text)| {
        match path
            .strip_prefix("src/translations/")
            .and_then(|f| f.strip_suffix(".json"))
        {
            Some(locale) => {
                tables.push((locale.to_string(), text.clone()));
                false
            }
            None => true,
        }
    });
    let english = tables
        .iter()
        .find(|(l, _)| l == "en")
        .map(|(_, t)| t.clone())
        .unwrap_or_default();
    // The test that switches to Arabic, where Arabic is a language to
    // switch to.
    let arabic = locales[1..].iter().any(|l| l == "ar");
    files.retain(|(path, _)| path != "tests/arabic.wf" || arabic);
    for (path, text) in &mut files {
        if path.ends_with(".wf") {
            *text = one_line_per_locale(text, locales);
        } else if path == "webfluent.app.json" {
            *text = translated_config(text, locales)?;
        }
    }
    // The default locale holds the text the starter is written in, so its
    // pages and its tests read the same; another holds the starter's
    // translation into it, or that text again, to be translated.
    for (i, locale) in locales.iter().enumerate() {
        let table = tables
            .iter()
            .find(|(l, _)| i > 0 && l == locale)
            .map_or_else(|| english.clone(), |(_, t)| t.clone());
        files.push((format!("src/translations/{locale}.json"), table));
    }
    Ok(files)
}

/// The config with `i18n` naming `locales`, the first the default, and the
/// page's language that default — every other byte as it was.
fn translated_config(text: &str, locales: &[String]) -> Result<String> {
    use crate::project::config::json_edit;
    let i18n = serde_json::json!({
        "default_locale": locales[0],
        "locales": locales,
        "dir": "src/translations",
    });
    let edit = |e: json_edit::EditError| WebFluentError::config(e.to_string());
    let text = json_edit::set(text, &["i18n"], &i18n).map_err(edit)?;
    json_edit::set(&text, &["meta", "lang"], &serde_json::json!(locales[0])).map_err(edit)
}

/// `text` with its language switch made of `locales`: a line that switches
/// to English — `setLocale("en")`, or a `Select.Option` whose value is
/// `"en"` — written once for each locale, in place of the lines for the
/// languages the starter came with.
fn one_line_per_locale(text: &str, locales: &[String]) -> String {
    let is_option = |l: &str| l.contains("Select.Option(") && l.contains("value: \"");
    let mut out = String::with_capacity(text.len());
    let mut lines = text.split_inclusive('\n').peekable();
    while let Some(line) = lines.next() {
        let button = line.contains("setLocale(\"en\")");
        let option = is_option(line) && line.contains("value: \"en\"");
        if !button && !option {
            out.push_str(line);
            continue;
        }
        for locale in locales {
            out.push_str(
                &line
                    .replace("setLocale(\"en\")", &format!("setLocale(\"{locale}\")"))
                    .replace("value: \"en\"", &format!("value: \"{locale}\""))
                    .replace("(\"EN\")", &format!("(\"{}\")", locale.to_uppercase()))
                    .replace("(\"English\"", &format!("(\"{}\"", locale_name(locale))),
            );
        }
        while let Some(next) = lines.peek() {
            let same = if button {
                next.contains("setLocale(\"")
            } else {
                is_option(next)
            };
            if !same {
                break;
            }
            lines.next();
        }
    }
    out
}

/// A locale by the name its speakers give it, for a language switch: the
/// common ones, a region after it (`Português (BR)`), else the tag itself.
pub fn locale_name(locale: &str) -> String {
    let (language, rest) = match locale.split_once('-') {
        Some((l, r)) => (l, Some(r)),
        None => (locale, None),
    };
    let name = match language {
        "ar" => "العربية",
        "bn" => "বাংলা",
        "de" => "Deutsch",
        "en" => "English",
        "es" => "Español",
        "fa" => "فارسی",
        "fr" => "Français",
        "he" => "עברית",
        "hi" => "हिन्दी",
        "id" => "Bahasa Indonesia",
        "it" => "Italiano",
        "ja" => "日本語",
        "ko" => "한국어",
        "nl" => "Nederlands",
        "pl" => "Polski",
        "pt" => "Português",
        "ru" => "Русский",
        "sw" => "Kiswahili",
        "tr" => "Türkçe",
        "uk" => "Українська",
        "ur" => "اردو",
        "vi" => "Tiếng Việt",
        "zh" => "中文",
        _ => return locale.to_string(),
    };
    match rest {
        Some(rest) => format!("{name} ({rest})"),
        None => name.to_string(),
    }
}

/// The repository's `AGENTS.md`, as this `wf` was built with it.
const AGENTS: &str = include_str!("../../AGENTS.md");

/// `AGENTS.md` for a new project: the core of the language
/// (`references/core.md` of the skill), headed by the version it describes,
/// its links pointing into the skill's folder, where the rest of the
/// reference is, a file a section. Many agents read `AGENTS.md` into every
/// turn, so it holds what most work needs and no more.
pub fn agents_md() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let core = include_str!("../../skills/writing-webfluent/references/core.md");
    let core = core
        .replace(
            "](language",
            "](.agents/skills/writing-webfluent/references/language",
        )
        .replace(
            "](testing.md",
            "](.agents/skills/writing-webfluent/references/testing.md",
        );
    format!(
        "<!-- The core of WebFluent {version}, written by `wf init`. The whole \
         reference, a file a section: .agents/skills/writing-webfluent/references/language.md. \
         A newer wf brings a newer one. -->\n\n{core}"
    )
}

// ═══════════════════════════════════════════════════════════
//  The starters
// ═══════════════════════════════════════════════════════════
//
// Each starter is a folder of real files under `src/tools/init/<template>/`, the
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

    /// Whether `wf init --locales` can start it translated: a site can; a
    /// PDF or a deck is made in one language.
    pub fn translates(self) -> bool {
        matches!(self, Template::Spa | Template::Static)
    }

    /// What the template writes instead, or as well, when it is translated,
    /// under `src/tools/init/<template>-i18n/`: its sources with every visible
    /// string through `t("…")`, and the tables they read.
    pub fn translated(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Template::Spa => SPA_I18N,
            _ => &[],
        }
    }
}

const SPA: &[(&str, &str)] = &[
    ("src/App.wf", include_str!("init/spa/src/App.wf")),
    (
        "src/components/StatCard.wf",
        include_str!("init/spa/src/components/StatCard.wf"),
    ),
    (
        "src/components/TaskItem.wf",
        include_str!("init/spa/src/components/TaskItem.wf"),
    ),
    (
        "src/components/DarkModeSwitch.wf",
        include_str!("init/spa/src/components/DarkModeSwitch.wf"),
    ),
    (
        "src/pages/Home.wf",
        include_str!("init/spa/src/pages/Home.wf"),
    ),
    (
        "src/pages/Profile.wf",
        include_str!("init/spa/src/pages/Profile.wf"),
    ),
    (
        "src/pages/Settings.wf",
        include_str!("init/spa/src/pages/Settings.wf"),
    ),
    (
        "src/pages/Tasks.wf",
        include_str!("init/spa/src/pages/Tasks.wf"),
    ),
    (
        "src/stores/auth.wf",
        include_str!("init/spa/src/stores/auth.wf"),
    ),
    (
        "src/stores/tasks.wf",
        include_str!("init/spa/src/stores/tasks.wf"),
    ),
    ("src/styles.css", include_str!("init/spa/src/styles.css")),
    ("src/theme.wf", include_str!("init/spa/src/theme.wf")),
    ("tests/app.wf", include_str!("init/spa/tests/app.wf")),
    ("tests/tasks.wf", include_str!("init/spa/tests/tasks.wf")),
    (
        "webfluent.app.json",
        include_str!("init/spa/webfluent.app.json"),
    ),
];

const STATIC: &[(&str, &str)] = &[
    ("src/App.wf", include_str!("init/static/src/App.wf")),
    (
        "src/components/FeatureCard.wf",
        include_str!("init/static/src/components/FeatureCard.wf"),
    ),
    (
        "src/components/SiteFooter.wf",
        include_str!("init/static/src/components/SiteFooter.wf"),
    ),
    (
        "src/components/TeamMember.wf",
        include_str!("init/static/src/components/TeamMember.wf"),
    ),
    (
        "src/pages/About.wf",
        include_str!("init/static/src/pages/About.wf"),
    ),
    (
        "src/pages/Blog.wf",
        include_str!("init/static/src/pages/Blog.wf"),
    ),
    (
        "src/pages/Contact.wf",
        include_str!("init/static/src/pages/Contact.wf"),
    ),
    (
        "src/pages/Home.wf",
        include_str!("init/static/src/pages/Home.wf"),
    ),
    (
        "src/translations/ar.json",
        include_str!("init/static/src/translations/ar.json"),
    ),
    (
        "src/translations/en.json",
        include_str!("init/static/src/translations/en.json"),
    ),
    (
        "tests/arabic.wf",
        include_str!("init/static/tests/arabic.wf"),
    ),
    ("tests/site.wf", include_str!("init/static/tests/site.wf")),
    (
        "webfluent.app.json",
        include_str!("init/static/webfluent.app.json"),
    ),
];

const SPA_I18N: &[(&str, &str)] = &[
    ("src/App.wf", include_str!("init/spa-i18n/src/App.wf")),
    (
        "src/components/TaskItem.wf",
        include_str!("init/spa-i18n/src/components/TaskItem.wf"),
    ),
    (
        "src/components/DarkModeSwitch.wf",
        include_str!("init/spa-i18n/src/components/DarkModeSwitch.wf"),
    ),
    (
        "src/pages/Home.wf",
        include_str!("init/spa-i18n/src/pages/Home.wf"),
    ),
    (
        "src/pages/Profile.wf",
        include_str!("init/spa-i18n/src/pages/Profile.wf"),
    ),
    (
        "src/pages/Settings.wf",
        include_str!("init/spa-i18n/src/pages/Settings.wf"),
    ),
    (
        "src/pages/Tasks.wf",
        include_str!("init/spa-i18n/src/pages/Tasks.wf"),
    ),
    (
        "src/translations/ar.json",
        include_str!("init/spa-i18n/src/translations/ar.json"),
    ),
    (
        "src/translations/en.json",
        include_str!("init/spa-i18n/src/translations/en.json"),
    ),
    (
        "tests/arabic.wf",
        include_str!("init/spa-i18n/tests/arabic.wf"),
    ),
];

const PDF: &[(&str, &str)] = &[
    (
        "src/pages/Report.wf",
        include_str!("init/pdf/src/pages/Report.wf"),
    ),
    (
        "webfluent.app.json",
        include_str!("init/pdf/webfluent.app.json"),
    ),
];

const SLIDES: &[(&str, &str)] = &[
    (
        "src/pages/Deck.wf",
        include_str!("init/slides/src/pages/Deck.wf"),
    ),
    (
        "webfluent.app.json",
        include_str!("init/slides/webfluent.app.json"),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every file under `src/tools/init/<template>/` is one the template writes:
    /// a file added to a starter and left out of its table is a starter that
    /// does not build (its theme named in the config, and not there).
    #[test]
    fn every_starter_file_is_written() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tools/init");
        let mut starters: Vec<(String, &[(&str, &str)])> = Vec::new();
        for template in Template::ALL {
            starters.push((template.name().to_string(), template.files()));
            if !template.translated().is_empty() {
                starters.push((format!("{}-i18n", template.name()), template.translated()));
            }
        }
        for (starter, files) in starters {
            let dir = root.join(&starter);
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
            let mut listed: Vec<String> = files.iter().map(|(p, _)| p.to_string()).collect();
            listed.sort();
            assert_eq!(
                on_disk, listed,
                "the `{starter}` starter's table and its folder differ"
            );
        }
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("wf-init-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn locales(list: &[&str]) -> Vec<String> {
        list.iter().map(std::string::ToString::to_string).collect()
    }

    fn read(dir: &Path, path: &str) -> String {
        fs::read_to_string(dir.join(path)).unwrap()
    }

    #[test]
    fn a_locale_is_written_the_usual_way() {
        assert_eq!(
            parse_locales("en, AR ,pt-br,zh-hant,es-419").unwrap(),
            locales(&["en", "ar", "pt-BR", "zh-Hant", "es-419"])
        );
        assert_eq!(check_locales(&locales(&["fr"])).unwrap(), locales(&["fr"]));
    }

    #[test]
    fn a_locale_list_with_no_locale_twice_or_not_a_locale_is_refused() {
        let message = |list: &str| parse_locales(list).unwrap_err().to_string();
        assert!(message("").contains("No locales"));
        assert!(message(" , ").contains("No locales"));
        assert!(message("en,EN").contains("`en` is named twice"));
        assert!(message("pt-BR,pt-br").contains("`pt-BR` is named twice"));
        assert!(message("pt_BR").contains("written with `-`: `pt-BR`"));
        for bad in [
            "e",
            "english",
            "en-",
            "en-B",
            "1n",
            "en-Latn-x",
            "ar/eg",
            "en.json",
        ] {
            assert!(
                message(bad).contains("is not a locale"),
                "{bad}: {}",
                message(bad)
            );
        }
    }

    #[test]
    fn a_pdf_or_a_deck_takes_no_locales() {
        for template in [Template::Pdf, Template::Slides] {
            let dir = scratch(template.name());
            let options = Options {
                template,
                locales: locales(&["en", "ar"]),
            };
            let error = create_with(&dir, "x", &options).unwrap_err().to_string();
            assert!(error.contains("does not take locales"), "{error}");
            assert!(!dir.exists(), "nothing is written");
        }
    }

    #[test]
    fn without_locales_a_template_is_written_as_it_is() {
        for template in Template::ALL {
            let dir = scratch(&format!("plain-{}", template.name()));
            create(&dir, "plain", template).unwrap();
            for (path, text) in template.files() {
                assert_eq!(
                    read(&dir, path),
                    text.replace("{{name}}", "plain"),
                    "{path}"
                );
            }
            fs::remove_dir_all(&dir).unwrap();
        }
    }

    #[test]
    fn a_translated_project_names_its_locales_and_holds_a_table_for_each() {
        let dir = scratch("spa-fr-ar");
        let options = Options {
            template: Template::Spa,
            locales: locales(&["fr", "en", "ar"]),
        };
        create_with(&dir, "Atelier", &options).unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&read(&dir, "webfluent.app.json")).unwrap();
        assert_eq!(
            config["i18n"],
            serde_json::json!({
                "default_locale": "fr",
                "locales": ["fr", "en", "ar"],
                "dir": "src/translations",
            })
        );
        assert_eq!(config["meta"]["lang"], "fr");
        assert_eq!(config["theme"]["name"], "Day", "the rest is as it was");

        // The default holds the text the starter is written in; English
        // its English; Arabic the starter's translation into it.
        let english = include_str!("init/spa-i18n/src/translations/en.json");
        let arabic = include_str!("init/spa-i18n/src/translations/ar.json");
        assert_eq!(read(&dir, "src/translations/fr.json"), english);
        assert_eq!(read(&dir, "src/translations/en.json"), english);
        assert_eq!(read(&dir, "src/translations/ar.json"), arabic);
        assert!(dir.join("tests/arabic.wf").exists());

        let app = read(&dir, "src/App.wf");
        assert!(app.contains("Text(t(\"nav.dashboard\"))"), "{app}");
        let settings = read(&dir, "src/pages/Settings.wf");
        let options: Vec<&str> = settings
            .lines()
            .filter(|l| l.contains("Select.Option("))
            .map(str::trim)
            .collect();
        assert_eq!(
            options,
            [
                "Select.Option(\"Français\", value: \"fr\")",
                "Select.Option(\"English\", value: \"en\")",
                "Select.Option(\"العربية\", value: \"ar\")",
            ]
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_static_site_switches_between_its_own_locales() {
        let dir = scratch("static-ar-de");
        let options = Options {
            template: Template::Static,
            locales: locales(&["ar", "de"]),
        };
        create_with(&dir, "Site", &options).unwrap();
        let switch: Vec<&str> = include_str!("init/static/src/App.wf")
            .lines()
            .filter(|l| l.contains("setLocale"))
            .collect();
        assert_eq!(switch.len(), 2, "the starter switches between en and ar");
        let app = read(&dir, "src/App.wf");
        let switch: Vec<&str> = app
            .lines()
            .filter(|l| l.contains("setLocale"))
            .map(str::trim)
            .collect();
        assert_eq!(
            switch,
            [
                "Button(\"AR\").sm { on click { setLocale(\"ar\") } }",
                "Button(\"DE\").sm { on click { setLocale(\"de\") } }",
            ]
        );
        // Arabic is the default, so it holds the starter's English, and
        // there is no Arabic to switch to.
        let english = include_str!("init/static/src/translations/en.json");
        assert_eq!(read(&dir, "src/translations/ar.json"), english);
        assert_eq!(read(&dir, "src/translations/de.json"), english);
        assert!(!dir.join("src/translations/en.json").exists());
        assert!(!dir.join("tests/arabic.wf").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_project_is_not_written_over_one_that_is_there() {
        let dir = scratch("there");
        fs::create_dir_all(&dir).unwrap();
        let options = Options {
            template: Template::Static,
            locales: locales(&["en"]),
        };
        let error = create_with(&dir, "x", &options).unwrap_err().to_string();
        assert!(error.contains("already exists"), "{error}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_locale_is_named_in_its_own_language() {
        assert_eq!(locale_name("ar"), "العربية");
        assert_eq!(locale_name("pt-BR"), "Português (BR)");
        assert_eq!(locale_name("tlh"), "tlh");
    }
}
