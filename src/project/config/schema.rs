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
    /// A file of `public/` by its site-relative path, or a URL: text, and
    /// what an editor offers a picker of that kind for.
    File(crate::registry::FileKind),
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
            FieldType::File(_) => "file",
        }
    }

    /// Whether `value` is of this type.
    pub fn admits(self, value: &Value) -> bool {
        match (self, value) {
            (_, Value::Null) => true,
            (FieldType::Bool, Value::Bool(_)) => true,
            (FieldType::String | FieldType::File(_), Value::String(_)) => true,
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

use crate::registry::FileKind;
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
        "Which `theme` declaration is the site's. A project with exactly one need not name it; with none, the baseline applies.",
    ),
    f(
        "version",
        String,
        "The project's version, as you number it. Nothing in the build reads it.",
    ),
    f("author", String, "Who wrote it. Nothing in the build reads it."),
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
        "The `theme` declaration that stands in for dark mode: when the reader prefers dark, or `setTheme(\"dark\")` asks (Styling).",
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
        "Where the build is written, relative to the project and inside it: a path that climbs out (`../site`, `/var/www`) or a link to somewhere else is refused unless the command line says `--allow-output-outside` (CLI).",
    ),
    f(
        "build.output_type",
        Enum(OUTPUT_TYPES),
        "What the project builds: `spa` is a website; `static` is a website whose every page is pre-rendered, as `ssg: true` builds it; `pdf` a document; `slides` a deck; `elements` custom elements (PDF and slides, custom elements).",
    ),
    f(
        "build.minify",
        Bool,
        "Strips comments and whitespace from the scripts and stylesheets. Off, the output is readable and larger.",
    ),
    f(
        "build.sourcemap",
        Bool,
        "Writes a source map (version 3) beside the bundle and each page's chunk — `app.js.map`, `pages/Home.js.map` — so a stack trace from the running site reads back to the `.wf` line each statement came from. The scripts are otherwise unchanged.",
    ),
    f(
        "build.ssg",
        Bool,
        "Pre-renders every page to its own HTML file, so it is read before any script runs (Static sites and SPAs).",
    ),
    f(
        "build.base_path",
        String,
        "The path the site is served under — `/Halyard` for a project site — which every link and asset begins with.",
    ),
    f(
        "build.clean_urls",
        Enum(CLEAN_URLS),
        "How a route's address avoids a redirect on a host that adds a trailing slash. `\"file\"` also writes `contact.html` beside `contact/index.html`; `\"directory\"` names every route `/contact/` in canonical links, the sitemap and links to pages (Deploy to GitHub Pages).",
    ),
    f(
        "build.csp",
        Bool,
        "Emits a Content-Security-Policy and a `_headers` file, and checks the output against the policy (Security).",
    ),
    f(
        "build.split",
        Bool,
        "Write each page's code as its own file, loaded when the page is shown; off, every page ships in `app.js`.",
    ),
    f(
        "build.compress",
        Bool,
        "Writes a gzipped `.gz` copy beside every text output over a kilobyte, for hosts that serve one.",
    ),
    f(
        "build.budget",
        StringMap { values: None },
        "A size an output must stay under, gzipped — `{ \"app.js\": \"40 kB\" }` — warned of when a build is over it.",
    ),
    f(
        "build.runtime",
        Enum(RUNTIMES),
        "`auto` ships only the runtime modules the program reaches; `full` ships every one, for a hand-written script that calls `WF` in ways the build cannot see (Call WF from a script).",
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
        "The formats every `image` is written in, best first, as well as its original.",
    ),
    f(
        "build.media.widths",
        Integers,
        "The widths an image is written at, for a page that does not say.",
    ),
    f(
        "build.media.quality",
        Integer { min: 1, max: 100 },
        "The encoding quality. WebP and JPEG are lossy at it. A file no smaller than the original is not written.",
    ),
    // ── PDF ──
    f(
        "build.pdf.page_size",
        String,
        "`A3`, `A4`, `A5`, `A6`, `B5`, `Letter`, `Legal`, `Tabloid` (or `Ledger`), `Executive`, or two lengths (`\"210mm 297mm\"`, `\"800x600\"`). A `Document(size:)` overrides it.",
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
        "The document's family when its stylesheets name none: a standard name (`Helvetica`, `Times`, `Courier` — Liberation, built in) or a family under `fonts/`.",
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
        "The space around each slide's content, in points. What runs past it is clipped.",
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
        "The colour of the numbers and the footer. Unset, it flips between dark and light on each slide's background.",
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
        "Rebuilds on a save, reloads open tabs, and draws a failed build's error over the page.",
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
        "The document's language: `en`, `ar`, `pt-BR`. A page written in another language says so with its own `lang:`.",
    ),
    f(
        "meta.favicon",
        File(FileKind::Icon),
        "The icon a browser tab shows: a file in `public/` — an SVG scales to every size — or a URL.",
    ),
    f(
        "meta.touch_icon",
        File(FileKind::Image),
        "The icon a phone puts on its home screen: a 180×180 PNG in `public/`. iOS reads this, not an SVG `favicon`.",
    ),
    f(
        "meta.site_url",
        String,
        "The site's absolute address, such as `https://example.com`. Without it, canonical links, the sitemap's URLs and absolute sharing URLs are left out rather than guessed.",
    ),
    f(
        "meta.site_name",
        String,
        "The person or organisation behind the site, for search results and sharing; unset, the project's name.",
    ),
    f(
        "meta.owner",
        Enum(OWNERS),
        "Whether the site belongs to an organisation or a person: the JSON-LD node the site is published by (Search and link previews).",
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
        "More about the owner, as schema.org properties merged into its node as written: `alternateName`, `email`, `worksFor`, `alumniOf`, `knowsAbout`, `logo`… The node's `@type`, `@id`, `name`, `url`, `jobTitle` and `sameAs` come from the settings above; naming one here is `E111`.",
    ),
    f(
        "meta.image",
        File(FileKind::Image),
        "The sharing image for pages that set none. For an image in `public/`, its size is read into `og:image:width` and `og:image:height`.",
    ),
    f(
        "meta.image_alt",
        String,
        "What that image shows, as `og:image:alt` and `twitter:image:alt`. A page's `image_alt:` wins.",
    ),
    f(
        "meta.sitemap",
        Bool,
        "Write `sitemap.xml` and `robots.txt` for a static build.",
    ),
    f(
        "meta.fonts",
        Strings,
        "Web-font stylesheets to link, each with a `preconnect`. Their origins join the policy.",
    ),
    f(
        "meta.preload",
        Strings,
        "Files on this site to fetch with the page, ahead of the stylesheets: a font a project `.css` file's `@font-face` names, the first screen's picture. `as` comes from the extension (a font is also `crossorigin`). A file on another origin is `E111` (Search and link previews).",
    ),
    f(
        "meta.stylesheets",
        Strings,
        "Stylesheets of the project's own or another site's, linked on every page.",
    ),
    f(
        "meta.scripts",
        Scripts,
        "Libraries to load before the project's own scripts (below). Their origins join the policy (JavaScript interop).",
    ),
    f(
        "meta.integrity",
        StringMap { values: None },
        "The integrity hash of each asset from another site, so the browser checks it has not changed.",
    ),
    f(
        "meta.connect",
        Strings,
        "Origins a page may send requests to besides its own — an API elsewhere, an analytics endpoint — as the policy's `connect-src`: `\"https://api.example.com\"`, `\"https://*.google-analytics.com\"`, or an app's own scheme alone, `\"ipc:\"`.",
    ),
    f(
        "meta.img",
        Strings,
        "Origins a page may load images from besides its own and `data:` — a CDN, a tag's tracking pixel — as the policy's `img-src`: `\"https://images.example.com\"`, `\"https://www.googletagmanager.com\"`.",
    ),
    f(
        "meta.frame",
        Strings,
        "Origins a page may embed in a frame besides its own — a video player, a map — as the policy's `frame-src`: `\"https://www.youtube-nocookie.com\"`, or an app's own scheme alone, `\"wfpreview:\"`.",
    ),
    f(
        "meta.wasm",
        Bool,
        "The pages compile WebAssembly: `script-src` gains `'wasm-unsafe-eval'`, which allows that and not `eval`.",
    ),
    f(
        "meta.style_elements",
        Bool,
        "A script on the page writes `<style>` elements (an editor, a chart library): `style-src` gains `'unsafe-inline'`, for styles only.",
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
        "Values read as `env.NAME`, fixed at build time. A `.env` file and the shell's variables add to them.",
    ),
    f(
        "public_env",
        Strings,
        "The `env` names a page may read and so ships to the browser, beyond those beginning `PUBLIC_`.",
    ),
    // ── Services ──
    f(
        "apis",
        Map,
        "Each service `wf api sync` writes from an OpenAPI spec: `{ \"spec\": \"openapi.json\", \"out\": \"src/api/backend\", \"skip\": [] }`.",
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
            values: Some(crate::project::config::project::OFFLINE_STRATEGIES),
        },
        "How a path the build did not write is fetched, by glob: `network-first`, `cache-first`, `stale-while-revalidate` or `network-only`.",
    ),
    f(
        "offline.sync",
        Bool,
        "Keeps a write safe to send twice — a `PUT` or a `DELETE` — that failed for want of a network, and sends it, in order, when the connection returns. A `POST` or a `PATCH` is kept only when its endpoint is marked `idempotent`; any other fails as `.offline(queued: false)`.",
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
    f(
        "i18n.literals",
        Bool,
        "Translate by the words themselves: each string a reader sees is its own key.",
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
    let empty: crate::project::config::project::ProjectConfig =
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
                    File(kind) => row["file"] = serde_json::json!(kind.word()),
                    Integer { min, max } => {
                        row["min"] = serde_json::json!(min);
                        row["max"] = serde_json::json!(max);
                    }
                    Strings if f.path == "build.media.formats" => {
                        row["values"] = serde_json::json!(FORMATS);
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
            return Some((
                parts.iter().map(std::string::ToString::to_string).collect(),
                field,
            ));
        }
        if matches!(field.ty, StringMap { .. } | Map) {
            let mut keys: Vec<std::string::String> = parts[..n]
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
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
///
/// # Errors
///
/// An [`EditError`](super::json_edit::EditError) when `dotted` is not a
/// setting, `value` is not of its type, the text is not JSON, or the result
/// would not read as a config.
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
    serde_json::from_str::<crate::project::config::project::ProjectConfig>(&out)
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
            File(_) => Value::String("/sample.png".into()),
            Number => serde_json::json!(12.5),
            Integer { min, max } => serde_json::json!(i64::midpoint(min, max)),
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
            // `apis` holds services, each a spec and a folder.
            let value = if f.path == "apis" {
                serde_json::json!({ "Backend": { "spec": "openapi.json", "out": "src/api/backend", "skip": [] } })
            } else {
                sample(f.ty)
            };
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
            let read: crate::project::config::project::ProjectConfig = serde_json::from_value(doc)
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
