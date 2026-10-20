//! `webfluent.app.json` as data: every field with its type, the values it
//! takes, its default and one sentence — so a settings window, an editor's
//! completion or a tool is generated from the compiler rather than written
//! out again by hand, and a field the compiler gains appears in them with
//! no other change.
//!
//! The defaults are not written here: they are what the configuration's
//! own types give a file that sets nothing ([`defaults`]), so they cannot
//! disagree with what a build does. A test holds the table to the types:
//! every field the configuration reads has a row, and every row is a field
//! it reads.

use serde_json::Value;

/// What a field holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Bool,
    String,
    /// A number, with a fraction or without.
    Number,
    /// A whole number, at least `min` and at most `max`.
    Integer {
        min: i64,
        max: i64,
    },
    /// One of these words.
    Enum(&'static [&'static str]),
    /// A list of strings.
    Strings,
    /// A list of whole numbers.
    Integers,
    /// Names to strings: tokens, sizes, hashes. With `values`, each value
    /// is one of those words.
    StringMap {
        values: Option<&'static [&'static str]>,
    },
    /// Names to any JSON value: what `env` holds, and an owner's details.
    Map,
    /// A list of scripts, each a URL or `{ "src": … }`.
    Scripts,
}

impl FieldType {
    /// The word a schema says for it.
    pub fn word(self) -> &'static str {
        match self {
            FieldType::Bool => "boolean",
            FieldType::String => "string",
            FieldType::Number => "number",
            FieldType::Integer { .. } => "integer",
            FieldType::Enum(_) => "enum",
            FieldType::Strings => "strings",
            FieldType::Integers => "integers",
            FieldType::StringMap { .. } => "string map",
            FieldType::Map => "map",
            FieldType::Scripts => "scripts",
        }
    }

    /// Whether `value` is of this type.
    pub fn admits(self, value: &Value) -> bool {
        match (self, value) {
            (_, Value::Null) => true,
            (FieldType::Bool, Value::Bool(_)) => true,
            (FieldType::String, Value::String(_)) => true,
            (FieldType::Number, Value::Number(_)) => true,
            (FieldType::Integer { min, max }, Value::Number(n)) => {
                n.as_i64().is_some_and(|n| n >= min && n <= max)
            }
            (FieldType::Enum(cases), Value::String(s)) => cases.contains(&s.as_str()),
            (FieldType::Strings, Value::Array(items)) => items.iter().all(Value::is_string),
            (FieldType::Integers, Value::Array(items)) => {
                items.iter().all(|i| i.as_u64().is_some())
            }
            (FieldType::StringMap { values }, Value::Object(m)) => {
                m.values().all(|v| match (v, values) {
                    (Value::String(s), Some(cases)) => cases.contains(&s.as_str()),
                    (Value::String(_), None) => true,
                    _ => false,
                })
            }
            (FieldType::Map, Value::Object(_)) => true,
            (FieldType::Scripts, Value::Array(items)) => items
                .iter()
                .all(|i| i.is_string() || i.get("src").is_some_and(Value::is_string)),
            _ => false,
        }
    }
}

/// One field of `webfluent.app.json`.
#[derive(Debug, Clone, Copy)]
pub struct Field {
    /// Where it is: `build.base_path`.
    pub path: &'static str,
    pub ty: FieldType,
    /// What it does, in one sentence.
    pub summary: &'static str,
}

const fn f(path: &'static str, ty: FieldType, summary: &'static str) -> Field {
    Field { path, ty, summary }
}

use FieldType::*;

const OUTPUT_TYPES: &[&str] = &["spa", "static", "pdf", "slides", "elements"];
const BUILTINS: &[&str] = &["full", "structural"];
const CLEAN_URLS: &[&str] = &["file", "directory"];
const RUNTIMES: &[&str] = &["auto", "full"];
const OWNERS: &[&str] = &["organization", "person"];
const LINT_LEVELS: &[&str] = &["off", "warn", "error"];
const FORMATS: &[&str] = &["webp", "png", "jpeg"];

/// Every field, in the order a person reads them.
pub const FIELDS: &[Field] = &[
    // ── The project ──
    f(
        "name",
        String,
        "The project's name, used where a title or a site name is not given.",
    ),
    f(
        "version",
        String,
        "The project's version, as its author numbers it.",
    ),
    f("author", String, "Who wrote it."),
    // ── Theme ──
    f(
        "theme.name",
        String,
        "The `theme` declaration the build uses; unset, the one the project declares, or the baseline.",
    ),
    f(
        "theme.builtin",
        Enum(BUILTINS),
        "`full` ships the engine's design with its layout; `structural` only the layout, for a design of your own.",
    ),
    f(
        "theme.dark",
        String,
        "The `theme` declaration that stands in when the reader prefers dark, or `setTheme(\"dark\")` asks.",
    ),
    f(
        "theme.tokens",
        StringMap { values: None },
        "Token values applied over the theme — a colour a deploy injects, a size the studio sets.",
    ),
    // ── Build ──
    f(
        "build.output",
        String,
        "The directory the site is written to.",
    ),
    f(
        "build.output_type",
        Enum(OUTPUT_TYPES),
        "What the project builds: a single page, a static site, a PDF, a slide deck, or custom elements.",
    ),
    f(
        "build.minify",
        Bool,
        "Minify the JavaScript and CSS; off, the output is readable and larger.",
    ),
    f(
        "build.sourcemap",
        Bool,
        "Write source maps beside the JavaScript, so the browser's tools show the `.wf` lines.",
    ),
    f(
        "build.ssg",
        Bool,
        "Pre-render every page as HTML, so it is read before any script runs.",
    ),
    f(
        "build.base_path",
        String,
        "The path the site is served under — `/Halyard` for a project site — which every link and asset begins with.",
    ),
    f(
        "build.clean_urls",
        Enum(CLEAN_URLS),
        "How a route's address is written so it never redirects: `file` keeps `/contact`, `directory` names it `/contact/`.",
    ),
    f(
        "build.csp",
        Bool,
        "Ship a strict content security policy: a third-party script is then blocked unless the policy names it.",
    ),
    f(
        "build.split",
        Bool,
        "Write each page's code as its own file, loaded when the page is shown; off, every page ships in `app.js`.",
    ),
    f(
        "build.compress",
        Bool,
        "Write a gzipped copy of every text file beside it, for hosts that serve one.",
    ),
    f(
        "build.budget",
        StringMap { values: None },
        "A size an output must stay under, gzipped — `{ \"app.js\": \"40 kB\" }` — warned of when a build is over it.",
    ),
    f(
        "build.runtime",
        Enum(RUNTIMES),
        "`auto` ships only the runtime the program reaches; `full` all of it, for a script that calls it in ways the build cannot see.",
    ),
    f(
        "build.elements",
        Strings,
        "The components a custom-elements build publishes, each a tag named in kebab case.",
    ),
    // ── Media ──
    f(
        "build.media.pipeline",
        Bool,
        "Resize and re-encode the images the program names; off, an image is copied as it is.",
    ),
    f(
        "build.media.formats",
        Strings,
        "The formats an image is written in, best first.",
    ),
    f(
        "build.media.widths",
        Integers,
        "The widths an image is written at, for a page that does not say.",
    ),
    f(
        "build.media.quality",
        Integer { min: 1, max: 100 },
        "How much an image keeps when it is re-encoded, 1 to 100.",
    ),
    // ── PDF ──
    f(
        "build.pdf.page_size",
        String,
        "The page: `A4`, `Letter`, or a width and height in points.",
    ),
    f(
        "build.pdf.margins.top",
        Number,
        "The top margin, in points.",
    ),
    f(
        "build.pdf.margins.bottom",
        Number,
        "The bottom margin, in points.",
    ),
    f(
        "build.pdf.margins.left",
        Number,
        "The left margin, in points.",
    ),
    f(
        "build.pdf.margins.right",
        Number,
        "The right margin, in points.",
    ),
    f(
        "build.pdf.default_font",
        String,
        "The font text is set in when nothing says otherwise.",
    ),
    f(
        "build.pdf.default_font_size",
        Number,
        "The size text is set at when nothing says otherwise, in points.",
    ),
    f(
        "build.pdf.output_filename",
        String,
        "The name the PDF is written under; unset, the project's name.",
    ),
    f(
        "build.pdf.fonts",
        Strings,
        "Font files or folders to embed, beside the project's `fonts/`.",
    ),
    f(
        "build.pdf.system_fonts",
        Bool,
        "Let the machine's fonts stand in for a family or a character the project's fonts lack.",
    ),
    // ── Slides ──
    f(
        "build.slides.size",
        String,
        "The slide: `16:9`, `4:3`, `A4-landscape`, or a width and height in points.",
    ),
    f(
        "build.slides.width",
        Number,
        "A width in points, over `size`.",
    ),
    f(
        "build.slides.height",
        Number,
        "A height in points, over `size`.",
    ),
    f(
        "build.slides.default_font",
        String,
        "The font a slide is set in when nothing says otherwise.",
    ),
    f(
        "build.slides.default_font_size",
        Number,
        "The size a slide's text is set at when nothing says otherwise, in points.",
    ),
    f(
        "build.slides.margin",
        Number,
        "The margin around every slide, in points.",
    ),
    f(
        "build.slides.show_slide_numbers",
        Bool,
        "Number the slides, bottom right.",
    ),
    f(
        "build.slides.footer_text",
        String,
        "A line shown bottom left on every slide.",
    ),
    f(
        "build.slides.background_color",
        String,
        "The deck's background colour; a slide's own style overrides it.",
    ),
    f(
        "build.slides.chrome_color",
        String,
        "The colour of the numbers and the footer; unset, chosen from each slide's background.",
    ),
    f(
        "build.slides.output_filename",
        String,
        "The name the deck is written under; unset, the project's name.",
    ),
    f(
        "build.slides.fonts",
        Strings,
        "Font files or folders to embed, beside the project's `fonts/`.",
    ),
    f(
        "build.slides.system_fonts",
        Bool,
        "Let the machine's fonts stand in for a family or a character the project's fonts lack.",
    ),
    // ── Dev ──
    f(
        "dev.port",
        Integer { min: 1, max: 65535 },
        "The port `wf serve` listens on.",
    ),
    f(
        "dev.hot_reload",
        Bool,
        "Reload the page in the browser when a build finishes.",
    ),
    // ── Meta ──
    f(
        "meta.title",
        String,
        "The site's title, for a page that does not give its own.",
    ),
    f(
        "meta.description",
        String,
        "The site's description, for a page that does not give its own.",
    ),
    f(
        "meta.lang",
        String,
        "The language the pages are written in, as a tag: `en`, `ar`, `pt-BR`.",
    ),
    f(
        "meta.favicon",
        String,
        "The icon a tab shows: a file in `public/` or a URL.",
    ),
    f(
        "meta.touch_icon",
        String,
        "The icon a phone puts on its home screen: a 180×180 PNG in `public/`.",
    ),
    f(
        "meta.site_url",
        String,
        "The site's own origin — `https://example.com` — without which no canonical link or sitemap is written.",
    ),
    f(
        "meta.site_name",
        String,
        "The person or organisation behind the site, for search results and sharing; unset, the project's name.",
    ),
    f(
        "meta.owner",
        Enum(OWNERS),
        "Whether the site belongs to an organisation or a person, for the structured data it publishes.",
    ),
    f(
        "meta.same_as",
        Strings,
        "The owner's other profiles, which a search engine joins to the site.",
    ),
    f(
        "meta.job_title",
        String,
        "What a person who owns the site does.",
    ),
    f(
        "meta.owner_details",
        Map,
        "More of what the owner is, as schema.org properties: an email, a logo, where they work.",
    ),
    f(
        "meta.image",
        String,
        "The image a link to a page shows when shared, for a page that does not name its own.",
    ),
    f(
        "meta.image_alt",
        String,
        "What the sharing image shows, for a reader who cannot see it.",
    ),
    f(
        "meta.sitemap",
        Bool,
        "Write `sitemap.xml` and `robots.txt` for a static build.",
    ),
    f(
        "meta.fonts",
        Strings,
        "Web-font stylesheets to load, by URL, before the site's own styles.",
    ),
    f(
        "meta.preload",
        Strings,
        "Files to fetch early, before the page asks for them: a font, a hero image.",
    ),
    f(
        "meta.stylesheets",
        Strings,
        "Stylesheets of the project's own or another site's, linked on every page.",
    ),
    f(
        "meta.scripts",
        Scripts,
        "Scripts every page loads — a library, an analytics tag — and the globals they define.",
    ),
    f(
        "meta.integrity",
        StringMap { values: None },
        "The integrity hash of each asset from another site, so the browser checks it has not changed.",
    ),
    f(
        "meta.connect",
        Strings,
        "The origins a page may fetch from or connect to, beyond its own, under the security policy.",
    ),
    f(
        "meta.img",
        Strings,
        "The origins a page may show images from, beyond its own, under the security policy.",
    ),
    f(
        "meta.frame",
        Strings,
        "The origins a page may show in a frame, under the security policy.",
    ),
    f(
        "meta.wasm",
        Bool,
        "Let a page run WebAssembly, under the security policy.",
    ),
    f(
        "meta.style_elements",
        Bool,
        "Let a library write `<style>` elements, under the security policy.",
    ),
    // ── Motion ──
    f(
        "motion.duration",
        String,
        "How long an animation runs when the element does not say: `180ms`.",
    ),
    f(
        "motion.easing",
        String,
        "How an animation is paced when the element does not say: a token or a CSS timing function.",
    ),
    // ── Environment ──
    f(
        "env",
        Map,
        "Values the program reads as `env.NAME`, set per build.",
    ),
    f(
        "public_env",
        Strings,
        "The `env` names a page may read and so ships to the browser, beyond those beginning `PUBLIC_`.",
    ),
    // ── Offline ──
    f(
        "offline.precache",
        Strings,
        "The routes a first visit stores for when the network is gone, as globs: `/`, `/docs/*`.",
    ),
    f(
        "offline.fallback",
        String,
        "The page shown offline for a route that is not stored.",
    ),
    f(
        "offline.cache",
        StringMap {
            values: Some(crate::config::project::OFFLINE_STRATEGIES),
        },
        "How a path the build did not write is fetched, by glob: `network-first`, `cache-first`, `stale-while-revalidate` or `network-only`.",
    ),
    f(
        "offline.sync",
        Bool,
        "Queue a write made while offline, and send it when the connection returns.",
    ),
    // ── Languages ──
    f(
        "i18n.default_locale",
        String,
        "The language a message is shown in when a locale has no translation of it.",
    ),
    f(
        "i18n.locales",
        Strings,
        "The languages the project is translated into, each a file under `dir`.",
    ),
    f(
        "i18n.dir",
        String,
        "The folder the translation files are in.",
    ),
    // ── Lints ──
    f(
        "lints",
        StringMap {
            values: Some(LINT_LEVELS),
        },
        "What a finding counts as, by its code or family: `off`, `warn` or `error`.",
    ),
];

/// The part of the file a field is in: its first key, or `project` for
/// the project's own name, version and author — and for `env`,
/// `public_env` and `lints`, each a part of its own.
pub fn section_of(path: &str) -> &str {
    match path.split_once('.') {
        Some((head, _)) => head,
        None if matches!(path, "name" | "version" | "author") => "project",
        None => path,
    }
}

/// The field at `path`, if there is one.
pub fn field(path: &str) -> Option<&'static Field> {
    FIELDS.iter().find(|f| f.path == path)
}

/// What a file that sets nothing reads as — with `i18n` and `offline`,
/// which are absent until set, as they read once they are.
pub fn defaults() -> Value {
    let empty: crate::config::project::ProjectConfig =
        serde_json::from_str(r#"{ "name": "", "i18n": {}, "offline": {} }"#)
            .expect("an empty config reads");
    serde_json::to_value(&empty).expect("a config writes")
}

/// The value at a dotted `path` of `value`.
pub fn at<'v>(value: &'v Value, path: &str) -> Option<&'v Value> {
    path.split('.').try_fold(value, |v, key| v.get(key))
}

/// The table as JSON: each field's path, type, values, default and
/// summary — `wf config schema --json`, and what an app or an editor reads.
pub fn to_json() -> Value {
    let defaults = defaults();
    Value::Array(
        FIELDS
            .iter()
            .map(|f| {
                let mut row = serde_json::json!({
                    "path": f.path,
                    "section": section_of(f.path),
                    "type": f.ty.word(),
                    "default": at(&defaults, f.path).cloned().unwrap_or(Value::Null),
                    "summary": f.summary,
                });
                match f.ty {
                    Enum(cases) => row["values"] = serde_json::json!(cases),
                    StringMap {
                        values: Some(cases),
                    } => row["values"] = serde_json::json!(cases),
                    Integer { min, max } => {
                        row["min"] = serde_json::json!(min);
                        row["max"] = serde_json::json!(max);
                    }
                    Strings if f.path == "build.media.formats" => {
                        row["values"] = serde_json::json!(FORMATS)
                    }
                    _ => {}
                }
                row
            })
            .collect(),
    )
}

/// A dotted path as the keys it names, read through the table: under a
/// map field, the rest is one key — `build.budget.app.js` is `build`,
/// `budget`, `app.js`. With the field it is, or is under.
pub fn path_of(dotted: &str) -> Option<(Vec<std::string::String>, &'static Field)> {
    let parts: Vec<&str> = dotted.split('.').collect();
    for n in (1..=parts.len()).rev() {
        let head = parts[..n].join(".");
        let Some(field) = field(&head) else {
            continue;
        };
        if n == parts.len() {
            return Some((parts.iter().map(|p| p.to_string()).collect(), field));
        }
        if matches!(field.ty, StringMap { .. } | Map) {
            let mut keys: Vec<std::string::String> =
                parts[..n].iter().map(|p| p.to_string()).collect();
            keys.push(parts[n..].join("."));
            return Some((keys, field));
        }
        return None;
    }
    None
}

/// `webfluent.app.json`'s text with the setting at `dotted` set to `value`,
/// or removed when it is `None` — its shape kept ([`super::json_edit`]),
/// the value held to the field's type, and the result read back as a
/// config before it is handed back, so what is written is what builds.
pub fn edit(
    text: &str,
    dotted: &str,
    value: Option<&Value>,
) -> Result<std::string::String, super::json_edit::EditError> {
    use super::json_edit::{EditError, remove, set};
    let (keys, field) = path_of(dotted).ok_or_else(|| {
        EditError(format!(
            "`{dotted}` is not a setting; `wf config schema` lists them"
        ))
    })?;
    let keys: Vec<&str> = keys.iter().map(std::string::String::as_str).collect();
    let out = match value {
        Some(v) => {
            let fits = if keys.len() > field.path.split('.').count() {
                // An entry of a map: one of its values.
                match field.ty {
                    StringMap {
                        values: Some(cases),
                    } => v.as_str().is_some_and(|s| cases.contains(&s)),
                    StringMap { values: None } => v.is_string(),
                    _ => true,
                }
            } else {
                field.ty.admits(v)
            };
            if !fits {
                let takes = match field.ty {
                    Enum(cases) => format!("one of {}", cases.join(", ")),
                    StringMap {
                        values: Some(cases),
                    } => format!("values among {}", cases.join(", ")),
                    Integer { min, max } => format!("a whole number from {min} to {max}"),
                    other => format!("a {}", other.word()),
                };
                return Err(EditError(format!(
                    "`{dotted}` takes {takes}, and {v} is not"
                )));
            }
            set(text, &keys, v)?
        }
        None => remove(text, &keys)?,
    };
    serde_json::from_str::<crate::config::project::ProjectConfig>(&out)
        .map_err(|e| EditError(format!("the file would not read as a config: {e}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every leaf the configuration's types read — a map counts as one —
    /// and has no row is listed.
    fn missing(value: &Value, prefix: &str, out: &mut Vec<std::string::String>) {
        let Value::Object(map) = value else {
            return;
        };
        for (key, v) in map {
            let path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            if field(&path).is_some() {
                continue;
            }
            match v {
                Value::Object(_) => missing(v, &path, out),
                _ => out.push(path),
            }
        }
    }

    #[test]
    fn every_field_the_config_reads_has_a_row() {
        let mut out = Vec::new();
        missing(&defaults(), "", &mut out);
        assert!(
            out.is_empty(),
            "fields with no row in config::schema::FIELDS — give each its type and one sentence: {out:?}"
        );
    }

    /// A value of `ty` that is not its default, so a round trip shows it.
    fn sample(ty: FieldType) -> Value {
        match ty {
            Bool => Value::Bool(true),
            FieldType::String => Value::String("sample".into()),
            Number => serde_json::json!(12.5),
            Integer { min, max } => serde_json::json!((min + max) / 2),
            Enum(cases) => Value::String(cases[cases.len() - 1].into()),
            Strings => serde_json::json!(["a", "b"]),
            Integers => serde_json::json!([320, 640]),
            StringMap {
                values: Some(cases),
            } => serde_json::json!({ "x": cases[0] }),
            StringMap { values: None } => serde_json::json!({ "x": "y" }),
            Map => serde_json::json!({ "x": 1 }),
            Scripts => serde_json::json!(["/a.js"]),
        }
    }

    #[test]
    fn every_row_is_a_field_the_config_reads() {
        for f in FIELDS {
            let value = sample(f.ty);
            let mut doc = serde_json::json!({ "name": "p" });
            let mut at_ = &mut doc;
            let keys: Vec<&str> = f.path.split('.').collect();
            for key in &keys[..keys.len() - 1] {
                at_ = at_
                    .as_object_mut()
                    .unwrap()
                    .entry(key.to_string())
                    .or_insert(serde_json::json!({}));
            }
            at_[keys[keys.len() - 1]] = value.clone();
            let read: crate::config::project::ProjectConfig = serde_json::from_value(doc)
                .unwrap_or_else(|e| panic!("`{}` set to {value} is refused: {e}", f.path));
            let back = serde_json::to_value(&read).unwrap();
            // An enum's value is written as it was read; a script is
            // written back as itself.
            let got = at(&back, f.path).cloned().unwrap_or(Value::Null);
            assert_eq!(
                got, value,
                "`{}` is not a field the config reads: it did not survive a round trip",
                f.path
            );
            assert!(f.ty.admits(&value), "{}", f.path);
            assert!(
                !f.summary.is_empty() && f.summary.ends_with('.'),
                "`{}`: one sentence",
                f.path
            );
        }
    }

    #[test]
    fn an_edit_is_held_to_the_field_and_keeps_the_file() {
        let text = "{\n  \"name\": \"p\",\n  \"build\": {\n    \"ssg\": true\n  }\n}\n";
        let out = edit(
            text,
            "build.base_path",
            Some(&serde_json::json!("/Halyard")),
        )
        .unwrap();
        assert_eq!(
            out,
            "{\n  \"name\": \"p\",\n  \"build\": {\n    \"ssg\": true,\n    \"base_path\": \"/Halyard\"\n  }\n}\n"
        );
        let out = edit(
            &out,
            "build.budget.app.js",
            Some(&serde_json::json!("40 kB")),
        )
        .unwrap();
        assert!(
            out.contains("\"budget\": {\n      \"app.js\": \"40 kB\"\n    }"),
            "{out}"
        );
        assert_eq!(
            edit(&out, "build.budget.app.js", None)
                .unwrap()
                .matches("app.js")
                .count(),
            0
        );
        assert_eq!(
            edit(&out, "build.base_path", None).unwrap(),
            edit(
                text,
                "build.budget.app.js",
                Some(&serde_json::json!("40 kB"))
            )
            .unwrap()
        );
        let err = edit(
            text,
            "build.output_type",
            Some(&serde_json::json!("website")),
        )
        .unwrap_err();
        assert_eq!(
            err.0,
            "`build.output_type` takes one of spa, static, pdf, slides, elements, and \"website\" is not"
        );
        let err = edit(text, "dev.port", Some(&serde_json::json!(99999))).unwrap_err();
        assert!(err.0.contains("a whole number from 1 to 65535"), "{err}");
        let err = edit(text, "lints.U01", Some(&serde_json::json!("loud"))).unwrap_err();
        assert!(err.0.contains("values among off, warn, error"), "{err}");
        let err = edit(text, "build.colour", Some(&serde_json::json!(1))).unwrap_err();
        assert!(err.0.contains("is not a setting"), "{err}");
    }

    #[test]
    fn a_default_is_what_the_build_does() {
        let d = defaults();
        assert_eq!(
            at(&d, "build.output"),
            Some(&Value::String("./build".into()))
        );
        assert_eq!(at(&d, "build.minify"), Some(&Value::Bool(true)));
        assert_eq!(at(&d, "dev.port"), Some(&serde_json::json!(3000)));
        assert_eq!(
            at(&d, "i18n.default_locale"),
            Some(&Value::String("en".into()))
        );
        let row = to_json();
        let base = row
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["path"] == "build.output_type")
            .unwrap();
        assert_eq!(base["default"], "spa");
        assert_eq!(base["values"], serde_json::json!(OUTPUT_TYPES));
    }
}
