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
pub fn create(dir: &Path, name: &str, template: Template) -> Result<()> {
    create_with(dir, name, &Options::new(template))
}

/// Write a new project named `name` into `dir`, which must not exist yet,
/// as `options` say — translated into `options.locales` when it names any.
pub fn create_with(dir: &Path, name: &str, options: &Options) -> Result<()> {
    let template = options.template;
    let locales = if options.locales.is_empty() {
        Vec::new()
    } else {
        if !template.translates() {
            return Err(WebFluentError::ConfigError(format!(
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
        return Err(WebFluentError::IoError(format!(
            "Directory '{}' already exists",
            dir.display()
        )));
    }
    for (path, text) in project_files(template, &locales)? {
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

/// The locales of `wf init --locales en,ar`, checked: the default first.
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
pub fn check_locales(locales: &[String]) -> Result<Vec<String>> {
    if locales.is_empty() {
        return Err(WebFluentError::ConfigError(
            "No locales: name at least one, the default first (`en,ar`).".to_string(),
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
            return Err(WebFluentError::ConfigError(format!(
                "`{given}` is not a locale: a language tag is two or three \
                 letters, then a region or script if it needs one, as in `en`, \
                 `ar`, `pt-BR`{hint}."
            )));
        };
        if out.contains(&tag) {
            return Err(WebFluentError::ConfigError(format!(
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
                lower[..1].to_ascii_uppercase() + &lower[1..]
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
    for (path, text) in files.iter_mut() {
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
            .map(|(_, t)| t.clone())
            .unwrap_or_else(|| english.clone());
        files.push((format!("src/translations/{locale}.json"), table));
    }
    Ok(files)
}

/// The config with `i18n` naming `locales`, the first the default, and the
/// page's language that default — every other byte as it was.
fn translated_config(text: &str, locales: &[String]) -> Result<String> {
    use crate::config::json_edit;
    let i18n = serde_json::json!({
        "default_locale": locales[0],
        "locales": locales,
        "dir": "src/translations",
    });
    let edit = |e: json_edit::EditError| WebFluentError::ConfigError(e.to_string());
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

    /// Whether `wf init --locales` can start it translated: a site can; a
    /// PDF or a deck is made in one language.
    pub fn translates(self) -> bool {
        matches!(self, Template::Spa | Template::Static)
    }

    /// What the template writes instead, or as well, when it is translated,
    /// under `src/init/<template>-i18n/`: its sources with every visible
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
        "src/components/ThemeToggle.wf",
        include_str!("init/spa/src/components/ThemeToggle.wf"),
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
        "src/components/ThemeToggle.wf",
        include_str!("init/spa-i18n/src/components/ThemeToggle.wf"),
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

    /// Every file under `src/init/<template>/` is one the template writes:
    /// a file added to a starter and left out of its table is a starter that
    /// does not build (its theme named in the config, and not there).
    #[test]
    fn every_starter_file_is_written() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/init");
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
        list.iter().map(|l| l.to_string()).collect()
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
