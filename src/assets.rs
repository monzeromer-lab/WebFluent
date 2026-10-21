//! `public/` as data: every file in it — its kind, its size, its
//! dimensions — and every place the project names it, so an assets panel
//! is a listing rather than a guess, the unused are counted, and a picker
//! never offers a path that does not exist.
//!
//! A use is found wherever the project can name a file of `public/`:
//!
//! - a string in a `.wf` or `.md` source — a prop the registry types as a
//!   file (`Image`'s `src`), a page's `image:`, a `const`, a style's
//!   `url(…)`, any literal at all that is a path `public/` has;
//! - an `image hero = "hero.jpg"` the build processes;
//! - a value of `webfluent.app.json` (`meta.favicon`, `meta.preload` …);
//! - a stylesheet's `url(…)`, under `src/` or in `public/` itself, a
//!   project script's string, a page or a manifest of `public/`.
//!
//! A file a browser or a host asks for by its name alone — `favicon.ico`,
//! `robots.txt`, `.well-known/…` — is used by convention, and is not
//! counted as unused.
//!
//! `plan_via` is what the media pipeline will make of one image of it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::config::ProjectConfig;
use crate::config::schema::{self, FieldType};
use crate::error::{Result, WebFluentError};
use crate::media::plan::{Plan, size_text};
use crate::parser::ast::*;
use crate::registry::FileKind;
use crate::vfs::Vfs;

/// What a file is, by its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// A picture: raster, an SVG, or an `.ico`.
    Image,
    Font,
    Video,
    Audio,
    /// What a reader opens: a PDF, text, a page, captions.
    Document,
    /// A script, a stylesheet, or WebAssembly: what the site runs.
    Code,
    /// JSON, XML, a manifest.
    Data,
    Other,
}

impl Kind {
    /// The kind of the file at `path`.
    pub fn of(path: &str) -> Kind {
        match extension(path).as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "bmp" | "tif" | "tiff" | "svg"
            | "ico" => Kind::Image,
            "woff" | "woff2" | "ttf" | "otf" | "eot" => Kind::Font,
            "mp4" | "webm" | "mov" | "ogv" | "m4v" => Kind::Video,
            "mp3" | "wav" | "ogg" | "oga" | "m4a" | "flac" | "aac" | "opus" => Kind::Audio,
            "pdf" | "txt" | "md" | "html" | "htm" | "vtt" | "srt" | "csv" | "doc" | "docx"
            | "xls" | "xlsx" | "ppt" | "pptx" | "epub" | "rtf" | "odt" => Kind::Document,
            "js" | "mjs" | "css" | "wasm" | "map" => Kind::Code,
            "json" | "xml" | "webmanifest" | "yaml" | "yml" | "toml" => Kind::Data,
            _ => Kind::Other,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Kind::Image => "image",
            Kind::Font => "font",
            Kind::Video => "video",
            Kind::Audio => "audio",
            Kind::Document => "document",
            Kind::Code => "code",
            Kind::Data => "data",
            Kind::Other => "other",
        }
    }

    pub const ALL: &[Kind] = &[
        Kind::Image,
        Kind::Font,
        Kind::Video,
        Kind::Audio,
        Kind::Document,
        Kind::Code,
        Kind::Data,
        Kind::Other,
    ];
}

/// Whether a file at `path` is one a prop that takes `kind` can name.
pub fn accepts(kind: FileKind, path: &str) -> bool {
    let ext = extension(path);
    match kind {
        FileKind::Image => Kind::of(path) == Kind::Image && ext != "ico",
        FileKind::Icon => Kind::of(path) == Kind::Image,
        FileKind::Video => Kind::of(path) == Kind::Video,
        FileKind::Audio => Kind::of(path) == Kind::Audio,
        FileKind::Captions => ext == "vtt",
        FileKind::Document => Kind::of(path) == Kind::Document,
    }
}

fn extension(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext.to_ascii_lowercase(),
        _ => String::new(),
    }
}

/// One place the project names a file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Use {
    /// The file it is in, relative to the project, `/`-separated.
    pub file: String,
    pub line: usize,
    pub column: usize,
    /// How it is named: `Image src`, `image hero`, `meta.favicon`, `url()`.
    pub how: String,
    /// Whether the media pipeline processes it there — `image hero = …`,
    /// or `Image(src: "/hero.png")` naming a picture it re-encodes, with
    /// `build.media.pipeline` on; otherwise it is copied as it is.
    pub processed: bool,
}

/// One file of `public/`.
#[derive(Debug, Clone)]
pub struct Asset {
    /// As the site asks for it: `/img/hero.png`.
    pub path: String,
    /// Where it is, relative to the project: `public/img/hero.png`.
    pub file: String,
    pub kind: Kind,
    /// Whether it is what a tab or a home screen shows: an `.ico`, a file
    /// under `icons/`, or one the config names as an icon.
    pub icon: bool,
    /// An SVG: it has a size to lay out at, not pixels.
    pub vector: bool,
    pub bytes: u64,
    /// For a picture whose header says.
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub uses: Vec<Use>,
    /// Why it is fetched without anything naming it, when it is.
    pub convention: Option<&'static str>,
}

impl Asset {
    /// Nothing names it, and nothing asks for it by convention.
    pub fn unused(&self) -> bool {
        self.uses.is_empty() && self.convention.is_none()
    }

    /// Whether the pipeline processes it anywhere.
    pub fn processed(&self) -> bool {
        self.uses.iter().any(|u| u.processed)
    }

    /// The files that name it, each once.
    pub fn used_in(&self) -> BTreeSet<&str> {
        self.uses.iter().map(|u| u.file.as_str()).collect()
    }

    pub fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path,
            "file": self.file,
            "kind": self.kind.word(),
            "icon": self.icon,
            "vector": self.vector,
            "bytes": self.bytes,
            "size": size_text(self.bytes),
            "width": self.width,
            "height": self.height,
            "uses": self.uses.iter().map(|u| serde_json::json!({
                "file": u.file,
                "line": u.line,
                "column": u.column,
                "how": u.how,
                "processed": u.processed,
            })).collect::<Vec<_>>(),
            "used_in": self.used_in().len(),
            "unused": self.unused(),
            "processed": self.processed(),
            "convention": self.convention,
        })
    }
}

/// What `public/` holds.
#[derive(Debug, Clone, Default)]
pub struct Listing {
    /// In path order.
    pub assets: Vec<Asset>,
}

impl Listing {
    /// The file the site asks for as `path`.
    pub fn get(&self, path: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.path == path)
    }

    /// What a picker for `kind` offers: every file of that kind there is,
    /// and nothing else.
    pub fn offered(&self, kind: FileKind) -> impl Iterator<Item = &Asset> {
        self.assets.iter().filter(move |a| accepts(kind, &a.path))
    }

    pub fn unused(&self) -> usize {
        self.assets.iter().filter(|a| a.unused()).count()
    }

    /// How many of each kind, with `icon` and `unused` beside them.
    pub fn counts(&self) -> BTreeMap<&'static str, usize> {
        let mut out = BTreeMap::new();
        for k in Kind::ALL {
            out.insert(
                k.word(),
                self.assets.iter().filter(|a| a.kind == *k).count(),
            );
        }
        out.insert("icon", self.assets.iter().filter(|a| a.icon).count());
        out.insert("unused", self.unused());
        out
    }

    pub fn bytes(&self) -> u64 {
        self.assets.iter().map(|a| a.bytes).sum()
    }

    pub fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "assets": self.assets.iter().map(Asset::as_json).collect::<Vec<_>>(),
            "counts": self.counts(),
            "bytes": self.bytes(),
        })
    }
}

/// Files a browser, a crawler or a host asks for by name, and why.
fn convention(path: &str) -> Option<&'static str> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let top = path.matches('/').count() == 1;
    if path.starts_with("/.well-known/") {
        return Some("asked for at `/.well-known/` by whatever reads it");
    }
    if !top {
        return None;
    }
    match name {
        "favicon.ico" => Some("a browser asks for `/favicon.ico` itself"),
        "robots.txt" => Some("a crawler asks for `/robots.txt`"),
        "sitemap.xml" => Some("a crawler asks for `/sitemap.xml`"),
        "humans.txt" | "ads.txt" | "app-ads.txt" | "security.txt" => {
            Some("read at the site's root by name")
        }
        "manifest.webmanifest" | "site.webmanifest" | "browserconfig.xml" => {
            Some("read by a browser installing the site")
        }
        "CNAME" | "_headers" | "_redirects" | ".nojekyll" | "404.html" | "_routes.json" => {
            Some("read by the host")
        }
        n if n.starts_with("apple-touch-icon") => Some("a phone asks for it by name"),
        _ => None,
    }
}

/// `path` as the site asks for it, when it is a path of the site's own:
/// `/hero.png?v=2` is `/hero.png`; a URL, `//cdn…` or `data:` is not one.
fn site_path(text: &str) -> Option<String> {
    let path = text.trim().split(['?', '#']).next().unwrap_or("");
    if !path.starts_with('/') || path.starts_with("//") || path.len() < 2 {
        return None;
    }
    Some(normalise(path))
}

/// `/a/./b/../c` as `/a/c`.
fn normalise(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for p in path.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    format!("/{}", parts.join("/"))
}

/// `reference` read from a file the site serves at `from` (`/css/site.css`):
/// `../fonts/a.woff2` is `/fonts/a.woff2`.
fn resolve(reference: &str, from: &str) -> Option<String> {
    let r = reference.trim().trim_matches(['"', '\'']);
    if r.is_empty() || r.starts_with('#') || r.starts_with("data:") || r.contains("://") {
        return None;
    }
    if r.starts_with('/') {
        return site_path(r);
    }
    let dir = from.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    site_path(&format!("{dir}/{r}"))
}

/// Every `url(…)` in `text`, as written.
fn css_urls(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find("url(") {
        let start = from + i + 4;
        let Some(end) = text[start..].find(')') else {
            break;
        };
        out.push((start, &text[start..start + end]));
        from = start + end;
    }
    out
}

fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let col = before.len() - before.rfind('\n').map_or(0, |i| i + 1) + 1;
    (line, col)
}

/// Collects the uses of the files `public/` has.
struct Finder<'a> {
    have: &'a BTreeSet<String>,
    uses: BTreeMap<String, Vec<Use>>,
    icons: BTreeSet<String>,
    pipeline: bool,
    /// Whether an `Image`'s literal `src:` is processed: `pipeline`, for a
    /// page rather than a PDF or a deck.
    literal: bool,
    /// Set while the argument being read is one the pipeline processes.
    processing: bool,
}

impl Finder<'_> {
    fn found(&mut self, path: String, file: &str, line: usize, column: usize, how: &str) {
        if self.have.contains(&path) {
            self.uses.entry(path).or_default().push(Use {
                file: file.to_string(),
                line,
                column,
                how: how.to_string(),
                processed: self.processing,
            });
        }
    }

    /// A string of the program: a path, or a style holding `url(…)`.
    fn literal(&mut self, text: &str, file: &str, at: Span, how: &str) {
        let (line, column) = (at.line.max(1) as usize, at.col.max(1) as usize);
        if let Some(p) = site_path(text) {
            self.found(p, file, line, column, how);
            return;
        }
        for (_, url) in css_urls(text) {
            if let Some(p) = resolve(url, "/") {
                self.found(p, file, line, column, "url()");
            }
        }
    }

    fn expr(&mut self, e: &Expr, file: &str, at: Span, how: &str) {
        if let Expr::StringLiteral(s) = e {
            self.literal(s, file, at, how);
        }
        // A path inside the value — a branch of `if`, an item of a list —
        // is named by the same prop.
        for child in e.children() {
            self.expr(child, file, at, how);
        }
    }

    fn style(&mut self, block: &StyleBlock, file: &str) {
        let props = block
            .properties
            .iter()
            .chain(block.media_queries.iter().flat_map(|m| m.properties.iter()))
            .chain(block.pseudo_blocks.iter().flat_map(|p| p.properties.iter()));
        for p in props {
            self.expr(&p.value, file, p.value_span, &format!("style {}", p.name));
        }
    }

    fn statements(&mut self, stmts: &[Statement], file: &str) {
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::UIElement(ui) => {
                    let name = match &ui.component {
                        ComponentRef::BuiltIn(n) | ComponentRef::UserDefined(n) => n.clone(),
                        ComponentRef::SubComponent(a, b) => format!("{a}.{b}"),
                    };
                    // `Image(src: "/hero.png")`: re-encoded, as an `image` is.
                    let processed = self
                        .literal
                        .then(|| crate::data::processed_src(ui).map(|(i, _)| i))
                        .flatten();
                    for (i, arg) in ui.args.iter().enumerate() {
                        let at = ui.arg_spans.get(i).copied().unwrap_or(ui.span);
                        self.processing = processed == Some(i);
                        match arg {
                            Arg::Positional(e) => self.expr(e, file, at, &name),
                            Arg::Named(key, e) => self.expr(e, file, at, &format!("{name} {key}")),
                        }
                        self.processing = false;
                    }
                    if let Some(style) = &ui.style_block {
                        self.style(style, file);
                    }
                    self.statements(&ui.children, file);
                    for fill in &ui.slot_fills {
                        self.statements(&fill.body, file);
                    }
                    for h in &ui.events {
                        self.statements(&h.body, file);
                    }
                }
                StatementKind::Connection(c) if c.kind == ConnectionKind::Worker => {
                    for e in stmt.kind.exprs() {
                        self.expr(e, file, stmt.span, "spawn");
                    }
                }
                kind => {
                    for e in kind.exprs() {
                        self.expr(e, file, stmt.span, "a string");
                    }
                }
            }
            for body in stmt.kind.bodies() {
                self.statements(body, file);
            }
        }
    }

    fn program(&mut self, program: &Program, file: &str, root: &Path, vfs: &dyn Vfs) {
        for decl in &program.declarations {
            match decl {
                Declaration::Page(p) => {
                    if let Some(image) = &p.image {
                        self.literal(image, file, p.span, "page image");
                    }
                    self.statements(&p.body, file);
                }
                Declaration::Component(c) => self.statements(&c.body, file),
                Declaration::Store(s) => self.statements(&s.body, file),
                Declaration::App(a) => self.statements(&a.body, file),
                Declaration::Const(c) => {
                    self.expr(&c.value, file, c.span, &format!("const {}", c.name))
                }
                Declaration::Theme(t) => {
                    for token in &t.tokens {
                        self.expr(&token.value, file, token.span, &format!("--{}", token.name));
                    }
                }
                // `image hero = "hero.jpg"`, found where the build finds it:
                // the project, its `src/`, then its `public/`.
                Declaration::Data(d) if d.is_image => {
                    let earlier = [root.join(&d.file), root.join("src").join(&d.file)];
                    if earlier.iter().any(|p| vfs.is_file(p)) {
                        continue;
                    }
                    let path = normalise(&format!("/{}", d.file));
                    if self.have.contains(&path) {
                        self.uses.entry(path).or_default().push(Use {
                            file: file.to_string(),
                            line: d.span.line.max(1) as usize,
                            column: d.span.col.max(1) as usize,
                            how: format!("image {}", d.name),
                            processed: self.pipeline,
                        });
                    }
                }
                // `data posts = "posts.json"`: its strings are the program's
                // values — a list of pictures, a file to download.
                Declaration::Data(d) => {
                    let found = [root.join(&d.file), root.join("src").join(&d.file)]
                        .into_iter()
                        .find(|p| vfs.is_file(p));
                    if let Some(path) = found
                        && let Ok(text) = vfs.read_to_string(&path)
                    {
                        let how = format!("data {}", d.name);
                        self.quoted(&text, &relative(root, &path), "/", &how, false);
                    }
                }
                _ => {}
            }
        }
    }

    /// The values of `webfluent.app.json` that name a file: a site path,
    /// or a bare name (`favicon.svg`) the head writes from the root.
    fn config(&mut self, text: &str) {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
            return;
        };
        let mut strings = Vec::new();
        fn walk<'v>(v: &'v serde_json::Value, path: &str, out: &mut Vec<(String, &'v str)>) {
            match v {
                serde_json::Value::String(s) => out.push((path.to_string(), s)),
                serde_json::Value::Array(items) => {
                    for i in items {
                        walk(i, path, out);
                    }
                }
                serde_json::Value::Object(m) => {
                    for (k, v) in m {
                        // `{ "src": … }` in a list of scripts is the list's.
                        let p = if path.is_empty() {
                            k.clone()
                        } else if k == "src" && path.ends_with("scripts") {
                            path.to_string()
                        } else {
                            format!("{path}.{k}")
                        };
                        walk(v, &p, out);
                    }
                }
                _ => {}
            }
        }
        walk(&value, "", &mut strings);
        let mut searched_from = BTreeMap::<String, usize>::new();
        for (key, s) in strings {
            let path = site_path(s).or_else(|| {
                (!s.contains("://") && !s.contains(' ') && !extension(s).is_empty())
                    .then(|| normalise(&format!("/{s}")))
            });
            let Some(path) = path else {
                continue;
            };
            if !self.have.contains(&path) {
                continue;
            }
            // The line it is written on: the first place after the last
            // one this same text was found at.
            let quoted = serde_json::to_string(s).unwrap_or_default();
            let start = searched_from.get(&quoted).copied().unwrap_or(0);
            let at = text[start..].find(&quoted).map_or(0, |i| start + i);
            searched_from.insert(quoted.clone(), at + quoted.len());
            let (line, column) = line_col(text, at + 1);
            let field = schema::FIELDS.iter().find(|f| f.path == key);
            if field.is_some_and(|f| matches!(f.ty, FieldType::File(FileKind::Icon)))
                || key == "meta.touch_icon"
            {
                self.icons.insert(path.clone());
            }
            self.found(path, "webfluent.app.json", line, column, &key);
        }
    }

    /// A stylesheet the site serves at `served` (or, under `src/`, one the
    /// build gathers into its own, read from the root).
    fn stylesheet(&mut self, text: &str, file: &str, served: &str) {
        for (at, url) in css_urls(text) {
            if let Some(p) = resolve(url, served) {
                let (line, column) = line_col(text, at);
                self.found(p, file, line, column, "url()");
            }
        }
    }

    /// Strings in a script or a page: `"/data.json"`, `src="logo.svg"`.
    fn quoted(&mut self, text: &str, file: &str, served: &str, how: &str, relative: bool) {
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let q = bytes[i];
            if (q == b'"' || q == b'\'' || q == b'`')
                && let Some(len) = text[i + 1..].find(q as char)
            {
                let s = &text[i + 1..i + 1 + len];
                if !s.contains('\n') && s.len() < 512 {
                    let found = if relative {
                        resolve(s, served)
                    } else {
                        site_path(s)
                    };
                    if let Some(p) = found {
                        let (line, column) = line_col(text, i + 1);
                        self.found(p, file, line, column, how);
                    }
                }
                i += len + 2;
                continue;
            }
            i += 1;
        }
    }
}

/// Every file under `dir`, as the site asks for it.
fn walk_public(vfs: &dyn Vfs, dir: &Path, prefix: &str, out: &mut Vec<(String, PathBuf)>) {
    let Ok(mut entries) = vfs.read_dir(dir) else {
        return;
    };
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let site = format!("{prefix}/{name}");
        if vfs.is_dir(&path) {
            walk_public(vfs, &path, &site, out);
        } else {
            out.push((site, path));
        }
    }
}

/// Every file under `dir` with one of `extensions`, relative to `root`.
fn walk_files(vfs: &dyn Vfs, dir: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(mut entries) = vfs.read_dir(dir) else {
        return;
    };
    entries.sort();
    for path in entries {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string());
        if vfs.is_dir(&path) {
            if !matches!(name.as_deref(), Some("node_modules" | ".wf-cache")) {
                walk_files(vfs, &path, extensions, out);
            }
        } else if path
            .extension()
            .is_some_and(|e| extensions.contains(&e.to_string_lossy().to_lowercase().as_str()))
        {
            out.push(path);
        }
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// A picture's size, from its header: an SVG's `width`/`height`, or its
/// `viewBox`; a raster image's pixels.
fn dimensions(bytes: &[u8], path: &str) -> Option<(u32, u32)> {
    if extension(path) == "svg" {
        let text = std::str::from_utf8(bytes).ok()?;
        let tag_at = text.find("<svg")?;
        let tag = &text[tag_at..tag_at + text[tag_at..].find('>')?];
        let attr = |name: &str| -> Option<f64> {
            let i = tag.find(&format!(" {name}="))? + name.len() + 2;
            let q = tag[i..].chars().next()?;
            let value = &tag[i + 1..i + 1 + tag[i + 1..].find(q)?];
            let number: String = value
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            // A percentage is no size of its own.
            if value[number.len()..].trim().starts_with('%') {
                return None;
            }
            number.parse().ok()
        };
        if let (Some(w), Some(h)) = (attr("width"), attr("height")) {
            return Some((w.round() as u32, h.round() as u32));
        }
        let i = tag.find("viewBox=")? + 8;
        let q = tag[i..].chars().next()?;
        let vb = &tag[i + 1..i + 1 + tag[i + 1..].find(q)?];
        let n: Vec<f64> = vb
            .split([' ', ','])
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        return (n.len() == 4).then(|| (n[2].round() as u32, n[3].round() as u32));
    }
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()
}

/// What `public/` holds, read through `vfs` (an editor's buffers over the
/// disk), and where the project names each file.
pub fn list_via(vfs: &dyn Vfs, root: &Path) -> Result<Listing> {
    let public = root.join("public");
    let mut files = Vec::new();
    walk_public(vfs, &public, "", &mut files);
    let have: BTreeSet<String> = files.iter().map(|(p, _)| p.clone()).collect();
    let config_text = vfs
        .read_to_string(&root.join("webfluent.app.json"))
        .unwrap_or_default();
    let config = ProjectConfig::load_via(vfs, root).ok();
    let pipeline = config.as_ref().is_none_or(|c| c.build.media.pipeline);
    // A PDF or a deck embeds a picture named by its path as it is.
    let literal = pipeline
        && config.as_ref().is_none_or(|c| {
            !matches!(
                c.build.output_type,
                crate::config::OutputType::Pdf | crate::config::OutputType::Slides
            )
        });
    let mut finder = Finder {
        have: &have,
        uses: BTreeMap::new(),
        icons: BTreeSet::new(),
        pipeline,
        literal,
        processing: false,
    };

    // The sources.
    let src = root.join("src");
    for path in crate::build::find_wf_files(vfs, &src).unwrap_or_default() {
        let Ok(text) = vfs.read_to_string(&path) else {
            continue;
        };
        let file = relative(root, &path);
        let program = if file.ends_with(".md") {
            match crate::data::markdown_page(&text, &file) {
                Ok(p) => p,
                Err(_) => continue,
            }
        } else {
            crate::syntax::parse_source_recovering(&text, &file).0
        };
        finder.program(&program, &file, root, vfs);
    }
    finder.config(&config_text);

    // The project's stylesheets and scripts.
    let mut code = Vec::new();
    walk_files(vfs, &src, &["css", "js", "mjs"], &mut code);
    for path in code {
        let Ok(text) = vfs.read_to_string(&path) else {
            continue;
        };
        let file = relative(root, &path);
        if file.ends_with(".css") {
            finder.stylesheet(&text, &file, "/");
        } else {
            finder.quoted(&text, &file, "/", "a string", false);
        }
    }

    // What `public/` says of itself: a stylesheet's fonts, a page's
    // pictures, a manifest's icons, a worker's files.
    for (site, path) in &files {
        let ext = extension(site);
        if !matches!(
            ext.as_str(),
            "css" | "js" | "mjs" | "html" | "htm" | "webmanifest" | "json"
        ) || (ext == "json" && !site.ends_with("manifest.json"))
        {
            continue;
        }
        let Ok(text) = vfs.read_to_string(path) else {
            continue;
        };
        let file = relative(root, path);
        match ext.as_str() {
            "css" => finder.stylesheet(&text, &file, site),
            "js" | "mjs" => finder.quoted(&text, &file, site, "a string", false),
            "webmanifest" | "json" => {
                let before = finder.uses.clone();
                finder.quoted(&text, &file, site, "manifest", true);
                // What a manifest names is an icon.
                for (p, u) in &finder.uses {
                    if u.len() > before.get(p).map_or(0, Vec::len) && Kind::of(p) == Kind::Image {
                        finder.icons.insert(p.clone());
                    }
                }
            }
            _ => finder.quoted(&text, &file, site, "a page", true),
        }
    }

    let Finder {
        mut uses, icons, ..
    } = finder;
    let mut assets = Vec::new();
    for (site, path) in files {
        let kind = Kind::of(&site);
        let bytes = vfs.metadata(&path).map(|m| m.len).unwrap_or(0);
        let (width, height) = if kind == Kind::Image {
            vfs.read(&path)
                .ok()
                .and_then(|b| dimensions(&b, &site))
                .map_or((None, None), |(w, h)| (Some(w), Some(h)))
        } else {
            (None, None)
        };
        let mut uses = uses.remove(&site).unwrap_or_default();
        // A file never uses itself.
        let own = relative(root, &path);
        uses.retain(|u| u.file != own);
        uses.sort();
        uses.dedup();
        assets.push(Asset {
            icon: kind == Kind::Image
                && (extension(&site) == "ico"
                    || site.starts_with("/icons/")
                    || icons.contains(&site)
                    || site
                        .rsplit('/')
                        .next()
                        .is_some_and(|n| n.starts_with("apple-touch-icon"))),
            vector: extension(&site) == "svg",
            convention: convention(&site),
            file: relative(root, &path),
            path: site,
            kind,
            bytes,
            width,
            height,
            uses,
        });
    }
    Ok(Listing { assets })
}

/// [`list_via`], from the disk.
pub fn list(root: &Path) -> Result<Listing> {
    list_via(&crate::vfs::FsVfs, root)
}

/// What the media pipeline will make of the image `public/` serves as
/// `path`, with the project's `build.media` settings: every file, its
/// width, its name and its size — exact where the build has written it
/// already (`.wf-cache/media/`), estimated where it has not.
pub fn plan_via(vfs: &dyn Vfs, root: &Path, path: &str) -> Result<Plan> {
    let site = site_path(path)
        .or_else(|| (!path.contains("://")).then(|| site_path(&format!("/{path}")))?)
        .ok_or_else(|| WebFluentError::IoError(format!("`{path}` is not a path of the site")))?;
    let file = root.join("public").join(site.trim_start_matches('/'));
    if !vfs.is_file(&file) {
        return Err(WebFluentError::IoError(format!(
            "`public{site}` does not exist"
        )));
    }
    if Kind::of(&site) != Kind::Image || matches!(extension(&site).as_str(), "svg" | "ico") {
        return Err(WebFluentError::IoError(format!(
            "`{site}` is not a picture the pipeline re-encodes: it is copied as it is"
        )));
    }
    let settings = ProjectConfig::load_via(vfs, root)
        .map(|c| c.build.media.settings())
        .unwrap_or_default();
    let bytes = vfs
        .read(&file)
        .map_err(|e| WebFluentError::IoError(format!("{}: {e}", file.display())))?;
    crate::media::plan::plan(
        &bytes,
        &file,
        &settings,
        Some(&root.join(".wf-cache/media")),
    )
}

/// [`plan_via`], from the disk.
pub fn plan(root: &Path, path: &str) -> Result<Plan> {
    plan_via(&crate::vfs::FsVfs, root, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_s_kind_is_its_extension() {
        assert_eq!(Kind::of("/hero.PNG"), Kind::Image);
        assert_eq!(Kind::of("/logo.svg"), Kind::Image);
        assert_eq!(Kind::of("/fonts/a.woff2"), Kind::Font);
        assert_eq!(Kind::of("/talk.vtt"), Kind::Document);
        assert_eq!(Kind::of("/.htaccess"), Kind::Other);
        assert_eq!(Kind::of("/CNAME"), Kind::Other);
        assert!(accepts(FileKind::Icon, "/favicon.ico"));
        assert!(!accepts(FileKind::Image, "/favicon.ico"));
        assert!(accepts(FileKind::Captions, "/talk.vtt"));
        assert!(!accepts(FileKind::Captions, "/talk.srt"));
        assert!(!accepts(FileKind::Video, "/hero.png"));
    }

    #[test]
    fn a_path_is_the_site_s_own_or_none() {
        assert_eq!(
            site_path("/a/../hero.png?v=2#x").as_deref(),
            Some("/hero.png")
        );
        assert_eq!(site_path("//cdn.example.com/x.png"), None);
        assert_eq!(site_path("https://x/y.png"), None);
        assert_eq!(site_path("hero.png"), None);
        assert_eq!(site_path("/"), None);
        assert_eq!(
            resolve("../fonts/a.woff2", "/css/site.css").as_deref(),
            Some("/fonts/a.woff2")
        );
        assert_eq!(
            resolve("'a.png'", "/css/site.css").as_deref(),
            Some("/css/a.png")
        );
        assert_eq!(resolve("data:image/png;base64,xx", "/a.css"), None);
        assert_eq!(resolve("#mask", "/a.css"), None);
    }

    #[test]
    fn an_svg_says_its_size_or_its_view_box() {
        let d = |s: &str| dimensions(s.as_bytes(), "/a.svg");
        assert_eq!(d(r#"<svg width="24" height="16px">"#), Some((24, 16)));
        assert_eq!(
            d(r#"<?xml?><svg xmlns="x" viewBox="0 0 120 40.4">"#),
            Some((120, 40))
        );
        assert_eq!(
            d(r#"<svg width="100%" height="100%" viewBox="0,0,8,8">"#),
            Some((8, 8))
        );
        assert_eq!(d("<svg>"), None);
        assert_eq!(d("not svg"), None);
    }

    #[test]
    fn what_is_asked_for_by_name_is_used_by_convention() {
        assert!(convention("/favicon.ico").is_some());
        assert!(convention("/robots.txt").is_some());
        assert!(convention("/.well-known/security.txt").is_some());
        assert!(convention("/apple-touch-icon-180.png").is_some());
        assert!(convention("/img/favicon.ico").is_none());
        assert!(convention("/hero.png").is_none());
    }
}
