//! What a build made, kept by what it was made from (`BuildCache`), so the
//! next build redoes only what an edit changed.
//!
//! A pre-rendered page is the build's costliest output, and an edit to one
//! page changes one page. Each is kept under a key that digests everything
//! its render reads:
//!
//! - the page itself, as the generators read it;
//! - every component it reaches, however it reaches it, transitively;
//! - and what every page reads: the app's shell and the components it
//!   reaches, every store, constant, data file, type, theme and service
//!   (the build-time scope holds them all), every page's header (titles and
//!   routes feed breadcrumbs and structured data), the config, the
//!   translations, and the compiler that rendered it.
//!
//! A declaration is digested through its `Debug` form, so a field added to
//! the syntax tree is in the key without anyone remembering to put it
//! there; and what a page reaches is found the same way — every name the
//! digest saw written as a string — which can only over-count. A cache hit
//! is therefore the page a full build would render, byte for byte; a
//! property test holds the two to that.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{self, Debug, Write as _};
use std::hash::{DefaultHasher, Hasher};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::config::ProjectConfig;
use crate::parser::ast::PageDecl;
use crate::parser::{Declaration, Program};

/// The compiler, as built: its version and a digest of its sources (the
/// crate's build script), so a cache never outlives the code that made it.
const COMPILER: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "+",
    env!("WF_SOURCE_FINGERPRINT")
);

/// The cache a build reads from and adds to. Hand the same one to every
/// build of a project — an editor keeps one for the session — and give it a
/// directory to keep what it holds across runs.
#[derive(Debug, Default)]
pub struct BuildCache {
    dir: Option<PathBuf>,
    pages: Mutex<HashMap<Key, String>>,
    used: Mutex<HashSet<Key>>,
    /// The key each route was last painted under: what a build that
    /// renders only the previewed page writes for every other.
    routes: Mutex<HashMap<String, Key>>,
}

impl BuildCache {
    /// A cache that lasts as long as the value does.
    pub fn in_memory() -> Self {
        Self::default()
    }

    /// A cache kept in `dir` as well, so a build after a restart reuses
    /// what the last run made. `wf serve` keeps one in `.wf-cache/build/`.
    pub fn on_disk(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: Some(dir.into()),
            ..Self::default()
        }
    }

    /// The page kept under `key`, or `render`'s, kept for next time. The
    /// flag is whether it was kept.
    pub(crate) fn page(
        &self,
        route: &str,
        key: Key,
        render: impl FnOnce() -> String,
    ) -> (String, bool) {
        self.used.lock().expect("the cache's lock").insert(key);
        self.routes
            .lock()
            .expect("the cache's lock")
            .insert(route.to_string(), key);
        if let Some(html) = self.pages.lock().expect("the cache's lock").get(&key) {
            return (html.clone(), true);
        }
        if let Some(file) = self.file(key)
            && let Ok(html) = std::fs::read_to_string(&file)
        {
            self.pages
                .lock()
                .expect("the cache's lock")
                .insert(key, html.clone());
            return (html, true);
        }
        let html = render();
        if let Some(file) = self.file(key)
            && let Some(dir) = file.parent()
            && std::fs::create_dir_all(dir).is_ok()
        {
            // Written whole or not at all: a build that stops halfway
            // leaves no half a page for the next to reuse.
            let partial = file.with_extension("partial");
            if std::fs::write(&partial, &html).is_ok() {
                let _ = std::fs::rename(&partial, &file);
            }
        }
        self.pages
            .lock()
            .expect("the cache's lock")
            .insert(key, html.clone());
        (html, false)
    }

    /// The page `route` was last painted as, in this session, however its
    /// inputs have changed since; kept for the next build as well.
    pub(crate) fn last_paint(&self, route: &str) -> Option<String> {
        let key = *self.routes.lock().expect("the cache's lock").get(route)?;
        let html = self
            .pages
            .lock()
            .expect("the cache's lock")
            .get(&key)
            .cloned()?;
        self.used.lock().expect("the cache's lock").insert(key);
        Some(html)
    }

    /// After a build: let go of what it did not use, in memory and on disk,
    /// so the cache holds the site as it stands and no more.
    pub(crate) fn finish(&self) {
        let used = std::mem::take(&mut *self.used.lock().expect("the cache's lock"));
        self.pages
            .lock()
            .expect("the cache's lock")
            .retain(|key, _| used.contains(key));
        self.routes
            .lock()
            .expect("the cache's lock")
            .retain(|_, key| used.contains(key));
        let Some(dir) = &self.dir else {
            return;
        };
        let keep: HashSet<String> = used.iter().map(|k| format!("{}.html", k.hex())).collect();
        if let Ok(entries) = std::fs::read_dir(dir.join("pages")) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !keep.contains(&name) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    fn file(&self, key: Key) -> Option<PathBuf> {
        Some(
            self.dir
                .as_ref()?
                .join("pages")
                .join(format!("{}.html", key.hex())),
        )
    }
}

/// A digest of everything something was made from: 128 bits, two
/// independent hashes of the same bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Key(u128);

impl Key {
    fn hex(self) -> String {
        format!("{:032x}", self.0)
    }
}

/// A `fmt::Write` that digests what is written to it, and keeps every
/// string it saw written in quotes — the names a `Debug` form mentions.
struct Digest {
    fnv: u64,
    sip: DefaultHasher,
    quoted: HashSet<String>,
    in_string: bool,
    escaped: bool,
    current: String,
}

impl Digest {
    fn new() -> Self {
        Self {
            fnv: 0xcbf2_9ce4_8422_2325,
            sip: DefaultHasher::new(),
            quoted: HashSet::new(),
            in_string: false,
            escaped: false,
            current: String::new(),
        }
    }

    fn key(&self) -> Key {
        Key((u128::from(self.fnv) << 64) | u128::from(self.sip.finish()))
    }

    /// Something whole, kept apart from what follows it.
    fn part(&mut self, value: &dyn Debug) {
        let _ = write!(self, "{value:?}");
        let _ = self.write_str("\u{0}");
    }
}

impl fmt::Write for Digest {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            self.fnv ^= u64::from(b);
            self.fnv = self.fnv.wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.sip.write(s.as_bytes());
        for c in s.chars() {
            if !self.in_string {
                if c == '"' {
                    self.in_string = true;
                    self.current.clear();
                }
            } else if self.escaped {
                self.current.push(c);
                self.escaped = false;
            } else if c == '\\' {
                self.escaped = true;
            } else if c == '"' {
                self.in_string = false;
                self.quoted.insert(std::mem::take(&mut self.current));
            } else {
                self.current.push(c);
            }
        }
        Ok(())
    }
}

/// A declaration's digest, and the names it mentions.
fn digest(value: &dyn Debug) -> (Key, HashSet<String>) {
    let mut d = Digest::new();
    d.part(value);
    (d.key(), d.quoted)
}

/// The config as a page reads it, in an order that does not change between
/// runs: its settings through JSON (whose objects keep their keys sorted;
/// the config holds hash maps, whose `Debug` order is a run's own), then
/// what the build works out into it and serde skips.
fn canonical_config(config: &ProjectConfig) -> String {
    // Where the site is written is not what a page says: an editor builds
    // each preview into a folder of its own, and its pages are the same.
    let mut config = config.clone();
    config.build.output = String::new();
    let config = &config;
    let settings = serde_json::to_value(config)
        .map(|v| v.to_string())
        .unwrap_or_default();
    let mut sizes: Vec<_> = config.meta.image_sizes.iter().collect();
    sizes.sort();
    format!(
        "{settings}\u{0}{:?}\u{0}{:?}\u{0}{sizes:?}",
        config.build.inline_styles, config.build.scripts
    )
}

/// A page's header — everything but its body, and no places in its file —
/// which every page reads: titles and routes feed breadcrumbs.
fn header(page: &PageDecl) -> PageDecl {
    PageDecl {
        body: Vec::new(),
        span: Default::default(),
        header_span: Default::default(),
        body_span: Default::default(),
        ..page.clone()
    }
}

/// The keys of a build's pages: worked out once per build from the program
/// its generators read.
pub(crate) struct PageKeys {
    /// What every page reads.
    shared: Key,
    /// Each component's digest and the names it mentions.
    components: HashMap<String, (Key, HashSet<String>)>,
}

impl PageKeys {
    pub(crate) fn new(
        program: &Program,
        config: &ProjectConfig,
        translations: &HashMap<String, HashMap<String, String>>,
        page_sheets: &[&String],
        studio: bool,
    ) -> Self {
        let mut components = HashMap::new();
        for decl in &program.declarations {
            if let Declaration::Component(c) = decl {
                components.insert(c.name.clone(), digest(c));
            }
        }
        let mut shared = Digest::new();
        shared.part(&COMPILER);
        // A preview's pages carry node ids a site's do not.
        shared.part(&studio);
        shared.part(&canonical_config(config));
        let mut locales: Vec<_> = translations.iter().collect();
        locales.sort_by(|a, b| a.0.cmp(b.0));
        for (locale, messages) in locales {
            let mut messages: Vec<_> = messages.iter().collect();
            messages.sort();
            shared.part(&(locale, messages));
        }
        shared.part(&page_sheets);
        let mut app_names = HashSet::new();
        for decl in &program.declarations {
            match decl {
                Declaration::Page(page) => shared.part(&header(page)),
                Declaration::Component(_) => {}
                Declaration::App(app) => {
                    let (key, names) = digest(app);
                    shared.part(&key);
                    app_names = names;
                }
                // Stores, constants, data, types, themes, services, scripts,
                // animations and tests: the build-time scope reads them, or
                // the shell does.
                other => shared.part(other),
            }
        }
        let mut keys = Self {
            shared: shared.key(),
            components,
        };
        // The shell is on every page, and so is what it reaches.
        let mut with_shell = Digest::new();
        with_shell.part(&keys.shared);
        for name in keys.reached(&app_names) {
            with_shell.part(&(&name, keys.components[&name].0));
        }
        keys.shared = with_shell.key();
        keys
    }

    /// Every component `names` reaches, transitively, in name order.
    fn reached(&self, names: &HashSet<String>) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut queue: Vec<&String> = names.iter().collect();
        while let Some(name) = queue.pop() {
            if let Some((_, mentions)) = self.components.get(name)
                && seen.insert(name.clone())
            {
                queue.extend(mentions.iter());
            }
        }
        seen
    }

    /// The key of `page` rendered at `route` with `params`.
    pub(crate) fn page(&self, page: &PageDecl, route: &str, params: &dyn Debug) -> Key {
        let (own, mentions) = digest(page);
        let mut d = Digest::new();
        d.part(&self.shared);
        d.part(&own);
        d.part(&route);
        d.part(params);
        for name in self.reached(&mentions) {
            d.part(&(&name, self.components[&name].0));
        }
        d.key()
    }
}
