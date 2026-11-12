//! `wf build` and `wf check`, as library calls.
//!
//! [`build()`](crate::build::build) reads a project through [a
//! `Vfs`](crate::base::vfs::Vfs) — the disk, or the disk with an editor's unsaved
//! buffers over it — checks it, and writes the site to its output directory.
//! `wf build` is this function and a loop that prints what it reports, so the
//! two cannot drift: the report holds every line the command prints, in order
//! and on its stream, and every finding as the one [`Diagnostic`] the rest of
//! the compiler uses, so a caller that shows them its own way has no text to
//! parse. [`check()`](crate::build::check) is `wf check`: every finding of a
//! build, with nothing written.

internal! {
    pub mod assets;
}
internal! {
    pub mod cache;
}
internal! {
    pub mod offline;
}

pub(crate) use crate::project::sources::find_wf_files;
pub use crate::project::sources::{
    read_project, read_project_collecting, read_project_collecting_via, read_project_via,
    read_project_with,
};
pub use cache::BuildCache;

use crate::base::outfs as fs;
use crate::base::vfs::{Confined, FsVfs, Vfs};
use crate::diagnostics::Severity;
use crate::diagnostics::error::{Diagnostic, Result, WebFluentError};
use crate::diagnostics::format::Format;
use crate::project::config::ProjectConfig;
use crate::project::config::project::OutputType;
use crate::syntax::parser::{Declaration, Program, Statement};
use crate::web::codegen::{JsCodegen, dark_css, generate_css_for, generate_html};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What to build, where to read it from, and how to report.
pub struct BuildOptions<'a> {
    /// The directory holding `webfluent.app.json`.
    pub project_dir: &'a Path,
    /// Where the project is read from. What the build writes — the output,
    /// the image cache, `.wf-sizes.json` — goes to the disk.
    pub vfs: &'a dyn Vfs,
    /// Where to write the site instead of the config's `build.output`,
    /// relative to the project or absolute.
    pub output_dir: Option<PathBuf>,
    /// Let the config's `build.output` name a directory outside the
    /// project (`wf build --allow-output-outside`). Without it, a project
    /// can only ever write inside itself: a config is part of what is
    /// cloned, and a build follows it.
    pub allow_output_outside: bool,
    /// Report what the output weighs, as `wf build --stats` does.
    pub stats: bool,
    /// How the findings are written: for a person, or for a tool.
    pub format: Format,
    /// A warning stops the build as an error does — for CI.
    pub deny_warnings: bool,
    /// A build for `wf serve`: responses are held to their declared types.
    pub dev: bool,
    /// Whether the findings rendered for a person are in colour.
    pub color: bool,
    /// What earlier builds made, to reuse what this one would make the
    /// same; `None` builds everything.
    pub cache: Option<&'a BuildCache>,
    /// For an editor's preview: every element's root carries its node id
    /// (`data-wf-node`), in the pre-rendered pages and the bundle alike, and
    /// the report hands back the map from each id to its source.
    pub studio: bool,
    /// For an editor's preview: write source maps (`build.sourcemap`)
    /// whatever the config says, so the console can read a stack trace
    /// back to the `.wf` lines.
    pub sourcemap: bool,
    /// For an editor's preview when a whole build takes longer than it can
    /// wait: render only the page answering this route (`/posts/hello`);
    /// every other page the cache holds as it stands is reused, and one it
    /// would have to render is written as this session last painted it
    /// (rendered only if it never was). The site may be behind
    /// the source until a build without it; the report counts the pages
    /// that are ([`BuildReport::pages_deferred`]). Without a cache it
    /// builds everything.
    pub only_route: Option<&'a str>,
    /// Called with each line as the build prints it, before it is kept in
    /// the report: `wf build` prints them as they come, so a build that
    /// spends a while on its images shows where it is.
    pub on_line: Option<&'a dyn Fn(&BuildLine)>,
}

impl<'a> BuildOptions<'a> {
    /// A build of `project_dir` read through `vfs`, as `wf build` runs it.
    pub fn new(project_dir: &'a Path, vfs: &'a dyn Vfs) -> Self {
        Self {
            project_dir,
            vfs,
            output_dir: None,
            allow_output_outside: false,
            stats: false,
            format: Format::Human,
            deny_warnings: false,
            dev: false,
            color: false,
            cache: None,
            studio: false,
            sourcemap: false,
            only_route: None,
            on_line: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Stdout,
    Stderr,
}

/// What one `println!` or `eprintln!` of `wf build` wrote, without its
/// newline: a finding rendered for a person is one line of several.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildLine {
    pub stream: Stream,
    pub text: String,
}

/// A page, and the route it answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteInfo {
    /// The page's `path:`, as written: `/posts/:slug`.
    pub path: String,
    /// The page's name.
    pub page: String,
    /// The file it is declared in, relative to the project.
    pub file: String,
    /// The route's parameters, in order: `slug` for `/posts/:slug`.
    pub params: Vec<String>,
}

/// What a build did and said.
#[derive(Clone, Debug, Default)]
pub struct BuildReport {
    /// For a studio build: every element's node id and where it is written.
    pub nodes: Option<crate::syntax::node_id::NodeMap>,
    /// Every line `wf build` prints, in order. The error a failed build
    /// ends with is not among them: it is [`BuildFailure::error`].
    pub lines: Vec<BuildLine>,
    pub pages: usize,
    pub components: usize,
    pub stores: usize,
    /// Every page's route, in the order the pages are declared.
    pub routes: Vec<RouteInfo>,
    /// Where the site was written; empty when the build stopped before it
    /// read the config.
    pub output_dir: PathBuf,
    pub duration: Duration,
    /// Pre-rendered pages this build took from its cache, and those it
    /// rendered.
    pub pages_reused: usize,
    pub pages_rendered: usize,
    /// Pre-rendered pages written as an earlier build painted them, because
    /// the build rendered only [`BuildOptions::only_route`]: the site is
    /// the source's once a build renders them again.
    pub pages_deferred: usize,
    /// Every finding the build reported, in the order it reported them.
    /// A build that could not run at all — a config it could not load, a
    /// file it could not read — has none, and its error says why.
    pub diagnostics: Vec<Diagnostic>,
}

impl BuildReport {
    /// The findings that stop a build.
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
    }

    /// The findings a build goes on past.
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
    }
}

/// A build that stopped: what it reported until it did, and why.
#[derive(Debug)]
pub struct BuildFailure {
    pub report: Box<BuildReport>,
    /// What `wf build` ends with, before it exits with the error's code.
    pub error: WebFluentError,
}

impl std::fmt::Display for BuildFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for BuildFailure {}

/// Build the project, as `wf build` does.
///
/// # Errors
///
/// A [`BuildFailure`] when the build stops, holding what it reported until
/// then: its error is [`WebFluentError::Diagnostics`] when a finding stops it
/// (an error, or a warning under `--deny-warnings`),
/// [`WebFluentError::ConfigError`] when the config is missing, is not one, or its
/// output leads out of the project, and [`WebFluentError::IoError`] when a
/// file cannot be read or written.
pub fn build(options: &BuildOptions) -> std::result::Result<BuildReport, BuildFailure> {
    finish(options, |out| run(options, out))
}

/// Every finding of a build, with nothing written, as `wf check` does.
///
/// # Errors
///
/// A [`BuildFailure`] when the check stops, holding what it reported until
/// then: its error is [`WebFluentError::Diagnostics`] when a finding stops it
/// (an error, or a warning under `--deny-warnings`),
/// [`WebFluentError::ConfigError`] when the config is missing, is not one, or its
/// output leads out of the project, and [`WebFluentError::IoError`] when a
/// file cannot be read.
pub fn check(options: &BuildOptions) -> std::result::Result<BuildReport, BuildFailure> {
    finish(options, |out| run_check(options, out))
}

/// Run `body`, then end as the format says: a JSON or SARIF build writes its
/// findings once, as one document, whether it finished or stopped.
fn finish(
    options: &BuildOptions,
    body: impl FnOnce(&Out) -> Result<()>,
) -> std::result::Result<BuildReport, BuildFailure> {
    let started = crate::base::platform::Stopwatch::start();
    let out = Out {
        report: RefCell::new(BuildReport::default()),
        found: RefCell::new(Vec::new()),
        on_line: options.on_line,
        format: options.format,
        color: options.color,
        vfs: options.vfs,
        project_dir: options.project_dir,
    };
    let outcome = body(&out);
    let found = out.found.take();
    match options.format {
        Format::Json => out.stdout(crate::diagnostics::format::json(&found)),
        Format::Sarif => out.stdout(crate::diagnostics::format::sarif(&found)),
        Format::Human | Format::Github => {}
    }
    let mut report = out.report.into_inner();
    report.duration = started.elapsed();
    match outcome {
        Ok(()) => Ok(report),
        Err(error) => Err(BuildFailure {
            report: Box::new(report),
            error,
        }),
    }
}

/// What the build has said so far, and how it says it.
struct Out<'a> {
    report: RefCell<BuildReport>,
    /// What a JSON or SARIF build has found so far: written once, as one
    /// document, when it ends.
    found: RefCell<Vec<Diagnostic>>,
    on_line: Option<&'a dyn Fn(&BuildLine)>,
    format: Format,
    color: bool,
    vfs: &'a dyn Vfs,
    project_dir: &'a Path,
}

impl Out<'_> {
    fn line(&self, stream: Stream, text: String) {
        let line = BuildLine { stream, text };
        if let Some(on_line) = self.on_line {
            on_line(&line);
        }
        self.report.borrow_mut().lines.push(line);
    }

    fn stdout(&self, text: impl Into<String>) {
        self.line(Stream::Stdout, text.into());
    }

    fn stderr(&self, text: impl Into<String>) {
        self.line(Stream::Stderr, text.into());
    }

    /// What `print!` or `eprint!` would write: text that ends its own line.
    fn print(&self, stream: Stream, text: &str) {
        if !text.is_empty() {
            self.line(stream, text.strip_suffix('\n').unwrap_or(text).to_string());
        }
    }

    /// A line of the build's progress: on standard output for a person, and
    /// on standard error when standard output carries findings for a tool.
    fn say(&self, text: impl Into<String>) {
        if self.format == Format::Human {
            self.stdout(text);
        } else {
            self.stderr(text);
        }
    }

    /// Every finding, as the build's format says: rendered for a person on
    /// standard error with the line that sums them up; kept for the one JSON
    /// or SARIF document the build ends with; or as GitHub annotations on
    /// standard output, beside the rendering. The report keeps each either
    /// way.
    fn findings(&self, diagnostics: &[Diagnostic]) {
        self.report
            .borrow_mut()
            .diagnostics
            .extend(diagnostics.iter().cloned());
        if matches!(self.format, Format::Json | Format::Sarif) {
            self.found.borrow_mut().extend(diagnostics.iter().cloned());
            if !diagnostics.is_empty() {
                self.stderr(crate::diagnostics::summary(diagnostics));
            }
            return;
        }
        if diagnostics.is_empty() {
            return;
        }
        if self.format == Format::Github {
            self.print(
                Stream::Stdout,
                &crate::diagnostics::format::github(diagnostics),
            );
        }
        self.print(
            Stream::Stderr,
            &crate::diagnostics::render::human(
                diagnostics,
                &|file| self.vfs.read_to_string(&self.project_dir.join(file)).ok(),
                self.color,
            ),
        );
        let (errors, _) = crate::diagnostics::counts(diagnostics);
        let summary = crate::diagnostics::summary(diagnostics);
        if errors > 0 {
            self.stderr(format!("error: the build stopped: {summary}"));
        } else {
            self.stderr(summary);
        }
    }

    /// The pages, components, stores and routes a program declares.
    fn program(&self, program: &Program, declaration_files: &[String]) {
        let mut report = self.report.borrow_mut();
        (report.pages, report.components, report.stores) = counts(program);
        report.routes = routes(program, declaration_files);
    }
}

/// How many pages, components and stores a program declares.
fn counts(program: &Program) -> (usize, usize, usize) {
    let count =
        |of: fn(&Declaration) -> bool| program.declarations.iter().filter(|d| of(d)).count();
    (
        count(|d| matches!(d, Declaration::Page(_))),
        count(|d| matches!(d, Declaration::Component(_))),
        count(|d| matches!(d, Declaration::Store(_))),
    )
}

/// Every page's route, with the file it is declared in.
fn routes(program: &Program, declaration_files: &[String]) -> Vec<RouteInfo> {
    program
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(index, d)| match d {
            Declaration::Page(page) => Some(RouteInfo {
                path: page.path.clone(),
                page: page.name.clone(),
                file: declaration_files.get(index).cloned().unwrap_or_default(),
                params: page
                    .path
                    .split('/')
                    .filter_map(|segment| segment.strip_prefix(':'))
                    .map(str::to_string)
                    .collect(),
            }),
            _ => None,
        })
        .collect()
}

/// The paths the config names that the build reads as they are — the
/// translations — each inside the project, or the finding that one is not.
///
/// The font directories of a document and a deck may be elsewhere (a
/// folder of fonts several projects share, `../fonts`): only font files are
/// read there, and only the glyphs a document uses are embedded.
fn check_config_paths(config: &ProjectConfig) -> Result<()> {
    let mut named: Vec<(&str, &str)> = Vec::new();
    if let Some(i18n) = &config.i18n {
        named.push(("`i18n.dir`", &i18n.dir));
    }
    for (what, path) in named {
        if !crate::base::vfs::stays_inside(path) {
            return Err(WebFluentError::config(format!(
                "{what} names `{path}`, which is outside the project: a build reads only the project's own files, by a path inside it (no absolute path, no `..`)"
            )));
        }
    }
    Ok(())
}

/// The config, with `env` from a `.env` file and the shell on top of its own.
fn load_config(options: &BuildOptions) -> Result<ProjectConfig> {
    let mut config = ProjectConfig::load_via(options.vfs, options.project_dir)?;
    check_config_paths(&config)?;
    config.resolve_env_via(
        options.vfs,
        options.project_dir,
        crate::base::platform::env_vars(),
    );
    if let Some(output) = &options.output_dir {
        config.build.output = output.to_string_lossy().to_string();
    }
    Ok(config)
}

/// `wf check`: every finding of a build, with nothing written.
fn run_check(options: &BuildOptions, out: &Out) -> Result<()> {
    let confined = Confined::new(options.vfs, options.project_dir);
    let mut config = load_config(options)?;
    output_dir_of(options, &config)?;
    let analysis = analyse_via(&confined, options.project_dir, &mut config, None)?;
    say_skipped(out, &confined);
    out.program(&analysis.program, &analysis.declaration_files);
    let diagnostics = analysis.diagnostics;
    out.findings(&diagnostics);
    stop_on(out, &diagnostics, options)?;
    if out.format == Format::Human && diagnostics.is_empty() {
        out.stdout(format!("No problems in {}.", config.name));
    }
    Ok(())
}

/// An error stops a build; with `--deny-warnings`, so does a warning.
fn stop_on(out: &Out, diagnostics: &[Diagnostic], options: &BuildOptions) -> Result<()> {
    let (errors, warnings) = crate::diagnostics::counts(diagnostics);
    if errors > 0 {
        return Err(WebFluentError::Diagnostics(diagnostics.to_vec()));
    }
    if options.deny_warnings && warnings > 0 {
        out.stderr("error: --deny-warnings makes each warning stop the build");
        return Err(WebFluentError::Diagnostics(diagnostics.to_vec()));
    }
    Ok(())
}

/// What the checks found in a project, and the program they read.
pub struct Analysis {
    /// Every declaration that parsed, as written.
    pub program: Program,
    /// The same, lowered for the backends.
    pub lowered: Program,
    /// The file of each declaration.
    pub declaration_files: Vec<String>,
    pub scripts: Vec<crate::project::project_js::Script>,
    /// The project's own stylesheets, bundled.
    pub project_css: String,
    /// Every finding: the files that did not parse, then every check.
    pub diagnostics: Vec<Diagnostic>,
}

/// Read and check a project, writing nothing but the images `media` names
/// (a build's output directory, its media settings and base path).
///
/// # Errors
///
/// [`WebFluentError::IoError`] when `src/` is missing, holds no source file,
/// or has a file that cannot be read. What the checks find is in the
/// [`Analysis`], not an error.
pub fn analyse(
    project_dir: &Path,
    config: &mut ProjectConfig,
    media: Option<(&Path, &crate::project::media::Settings, &str)>,
) -> Result<Analysis> {
    analyse_via(&FsVfs, project_dir, config, media)
}

/// [`analyse`], reading the project through `vfs`.
///
/// # Errors
///
/// [`WebFluentError::IoError`] when `src/` is missing, holds no source file,
/// or has a file that cannot be read. What the checks find is in the
/// [`Analysis`], not an error.
pub fn analyse_via(
    vfs: &dyn Vfs,
    project_dir: &Path,
    config: &mut ProjectConfig,
    media: Option<(&Path, &crate::project::media::Settings, &str)>,
) -> Result<Analysis> {
    let (program, declaration_files, parse_errors) =
        read_project_collecting_via(vfs, project_dir, media)?;
    // What the pages will contain, decided before any of them is written:
    // the policy each one ships has to describe that page.
    config.build.inline_styles = crate::web::codegen::csp::writes_inline_styles(&program);
    config.build.frames_itself = crate::web::codegen::csp::frames_itself(&program);
    // The project's own scripts: every `.js` under `src/`, a plain browser
    // script each, copied as written and linked before the compiled code.
    let scripts = crate::project::project_js::load_via(vfs, project_dir, &project_dir.join("src"))?;
    config.build.scripts = scripts.iter().map(|s| s.href.clone()).collect();
    // The size of each sharing image under `public/`, for the card.
    crate::web::codegen::seo::measure_images(project_dir, config, &program);
    let file_of = |index: usize| {
        declaration_files
            .get(index)
            .cloned()
            .unwrap_or_else(|| "src/".to_string())
    };
    // The text of each file, read again for the type checker to place its
    // errors at the expression they name.
    let source_of = |index: usize| {
        declaration_files
            .get(index)
            .and_then(|f| vfs.read_to_string(&project_dir.join(f)).ok())
    };
    // The author's own stylesheets — every `.css` under `src/` — ship in
    // `styles.css`, and a modifier class one of them defines is a real one.
    let project_css =
        crate::web::codegen::project_css::bundle_via(vfs, project_dir, &project_dir.join("src"))?;

    // Every check, over everything that parsed: the scripts and the config,
    // the structure, the registry's vocabulary, the types, the lints, and
    // what this output can draw. A stage's errors no longer stop the next.
    let checked = crate::web::check::check_project_via(
        vfs,
        &crate::web::check::Project {
            program: &program,
            file_of: &file_of,
            source_of: &source_of,
            dir: Some(project_dir),
            config: Some(config),
            declaration_files: &declaration_files,
            scripts: &scripts,
            stylesheets: &project_css,
            incomplete: !parse_errors.is_empty(),
        },
    );
    let mut diagnostics = parse_errors;
    diagnostics.extend(checked.diagnostics);
    crate::diagnostics::dedupe(&mut diagnostics);
    Ok(Analysis {
        program,
        lowered: checked.lowered,
        declaration_files,
        scripts,
        project_css,
        diagnostics,
    })
}

fn run(options: &BuildOptions, out: &Out) -> Result<()> {
    let project_dir = options.project_dir;
    // The project is read through a view that keeps it inside itself: a
    // link out of it, or a path that climbs out, is not there.
    let confined = Confined::new(options.vfs, project_dir);
    let vfs: &dyn Vfs = &confined;
    let stats = options.stats;
    let mut config = load_config(options)?;

    out.say(format!("Building {}...", config.name));
    // A value whose name does not say it is public, in the bundle because
    // `public_env` lists it: said on every build, so nobody is surprised.
    for name in config.unprefixed_public_env() {
        out.say(format!(
            "  Warning: env.{name} goes into the bundle: `public_env` lists it, so anyone who opens the site can read it"
        ));
    }

    // The images the program names are written at every width a page will
    // ask for, before anything checks the program — so a name resolves to
    // the asset it became, with its real size and colour.
    let output_dir = output_dir_of(options, &config)?;
    out.report.borrow_mut().output_dir = output_dir.clone();
    fs::create_dir_all(&output_dir)?;
    // What this build reads of the image cache, from here on: the rest is
    // pruned once it has finished.
    let media_cache = project_dir.join(".wf-cache/media");
    crate::project::media::take_used(&media_cache);
    let mut media = config.build.media.settings();
    // A PDF or a deck embeds the picture it names, not a `<picture>`.
    media.literal_src = !matches!(
        config.build.output_type,
        crate::project::config::OutputType::Pdf | crate::project::config::OutputType::Slides
    );
    let base_path = config.build.base_path.clone();
    let Analysis {
        program,
        lowered,
        declaration_files,
        scripts,
        project_css,
        diagnostics,
    } = analyse_via(
        vfs,
        project_dir,
        &mut config,
        Some((&output_dir, &media, &base_path)),
    )?;
    out.program(&program, &declaration_files);
    // Every HTML file this build writes. The policy check speaks for what
    // this build put in the output, not for whatever else is in the
    // directory — a file an older layout left behind is a real problem,
    // but not one this build can answer for.
    let mut written_html: Vec<PathBuf> = Vec::new();
    let config = config;
    out.findings(&diagnostics);
    stop_on(out, &diagnostics, options)?;
    let mut warning_count = diagnostics.len();
    // What each `persist` holds now, for the next build to compare with.
    let shapes = crate::check::linter::project::persist_shapes(&program);
    if !shapes.is_empty() {
        let cache = project_dir.join(crate::check::linter::project::PERSIST_CACHE);
        if let Some(parent) = cache.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(
            cache,
            serde_json::to_string_pretty(&shapes).unwrap_or_default(),
        );
    }
    // And what each starts as, which `wf verify --returning-visitor` puts in
    // storage — the values of the build before, when it changed — to load
    // the pages as a reader who last visited then.
    crate::check::linter::project::keep_persisted_values(project_dir, &program, &config.env);
    let mut program = lowered;
    // A project translated by its own words: each string a reader sees is
    // the `t()` it stands for, from here to the code it is written as.
    if config.i18n.as_ref().is_some_and(|i| i.literals) {
        crate::project::i18n::literals::translate(&mut program);
    }
    // Under `clean_urls: "directory"`, every link to a page names the
    // address with the slash, as the canonical does.
    crate::web::codegen::seo::directory_links(&config, &mut program);

    // A PDF document or a slide deck: the page's markup and the site's
    // stylesheet, laid out on paper by the paged engine — so the document
    // draws what the page draws.
    if matches!(
        config.build.output_type,
        OutputType::Pdf | OutputType::Slides
    ) {
        let deck = config.build.output_type == OutputType::Slides;
        let mut tokens = crate::project::themes::resolve_tokens(&program, &config.theme)?;
        crate::project::themes::apply_motion(&mut tokens, &config.motion);
        let mut css = generate_css_for(&tokens, config.theme.builtin, &program);
        css.push_str(&project_css);
        css.push_str(&crate::project::scoped_css::scoped_rules(&program));
        let fragment = crate::template::render_program_paged(&program, &serde_json::json!({}))?;
        // The document's language: the first page's own `lang:`, as
        // `wf render` reads it, else the site's.
        let lang = program
            .declarations
            .iter()
            .find_map(|d| match d {
                Declaration::Page(p) => p.lang.clone(),
                _ => None,
            })
            .unwrap_or_else(|| {
                if config.meta.lang.is_empty() {
                    "en".to_string()
                } else {
                    config.meta.lang.clone()
                }
            });
        let title = crate::web::codegen::markdown::escape(&config.name);
        let dir = if crate::template::is_rtl(&lang) {
            " dir=\"rtl\""
        } else {
            ""
        };
        let html = format!(
            "<!DOCTYPE html><html lang=\"{lang}\"{dir}><head><title>{title}</title><style>{css}</style></head><body>{fragment}</body></html>"
        );
        // A picture is read from the project, or from what this build wrote
        // (an `image` the media pipeline sized).
        let project_read = crate::paged::reader_via(options.vfs, Some(project_dir.to_path_buf()));
        let built = output_dir.clone();
        let read = move |src: &str| {
            project_read(src).or_else(|| {
                let rel = src
                    .split(['?', '#'])
                    .next()
                    .unwrap_or(src)
                    .trim_start_matches('/');
                // Only what this build wrote, under its output.
                let inside = !rel.contains(['\\', '\0'])
                    && Path::new(rel)
                        .components()
                        .all(|c| matches!(c, std::path::Component::Normal(_)));
                inside.then(|| fs::read(built.join(rel)).ok()).flatten()
            })
        };
        let rendered = if deck {
            let (options, chrome) = crate::paged::Options::from_slides_via(
                options.vfs,
                &config.build.slides,
                &read,
                Some(project_dir),
            );
            crate::paged::render_slides(&html, "", &options, &chrome)
        } else {
            let options = crate::paged::Options::from_pdf_via(
                options.vfs,
                &config.build.pdf,
                &read,
                Some(project_dir),
            );
            crate::paged::render(&html, "", &options)
        }
        .map_err(WebFluentError::codegen)?;

        let filename = if deck {
            config.build.slides.output_filename.clone()
        } else {
            config.build.pdf.output_filename.clone()
        }
        .unwrap_or_else(|| format!("{}.pdf", config.name));
        fs::write(output_dir.join(&filename), &rendered.bytes)?;
        for note in &rendered.notes {
            out.say(format!("  note: {note}"));
        }
        out.say(format!(
            "  {}: {} bytes, {} {}",
            if deck { "Slides" } else { "PDF" },
            rendered.bytes.len(),
            rendered.pages,
            if deck { "slide(s)" } else { "page(s)" }
        ));
        out.say(format!("  Output: {}/{}", config.build.output, filename));
        if warning_count == 0 {
            out.say("Build complete.");
        } else {
            out.say(format!("Build complete with {warning_count} warning(s)."));
        }
        return Ok(());
    }

    // Load translations if i18n is configured
    let translations = if let Some(i18n_config) = &config.i18n {
        load_translations(vfs, out, project_dir, i18n_config)?
    } else {
        HashMap::new()
    };

    // Generate output
    // `theme.builtin` decides how much of the engine's own design ships. The
    // build used to call the always-full entry point, so a project that asked
    // for `structural` still received the baseline it was trying to avoid.
    let mut tokens = crate::project::themes::resolve_tokens(&program, &config.theme)?;
    crate::project::themes::apply_motion(&mut tokens, &config.motion);
    let mut css = generate_css_for(&tokens, config.theme.builtin, &program);
    if let Some(dark) = crate::project::themes::resolve_dark_tokens(&program, &config.theme)? {
        css.push_str(&dark_css(&dark));
    }
    css.push_str(&project_css);
    // What an inline style cannot say — pseudo-states, media queries — is
    // compiled into the sheet under content-named classes. With `build.split`
    // the rules only one page reaches go to that page's own sheet.
    let page_sheets = if config.build.split {
        let split = crate::project::scoped_css::split_rules(&program);
        css.push_str(&split.shared);
        split.pages
    } else {
        css.push_str(&crate::project::scoped_css::scoped_rules(&program));
        Default::default()
    };
    let mut js_codegen = JsCodegen::new().with_options(crate::web::codegen::JsOptions {
        ssg: config.build.pre_renders(),
        dev: options.dev,
        split_pages: config.build.split,
        full_runtime: config.build.runtime == crate::project::config::RuntimeMode::Full,
        offline: config.offline.as_ref().map(|offline| offline.sync),
    });
    if let Some(i18n_config) = &config.i18n {
        js_codegen.set_i18n(i18n_config.default_locale.clone(), translations.clone());
    }
    js_codegen.set_env(config.public_env_values());
    js_codegen.set_later_scripts(
        config
            .meta
            .scripts
            .iter()
            .filter(|s| s.loads_after())
            .map(|s| {
                let src = s.src().to_string();
                let integrity = config.meta.integrity.get(&src).cloned();
                (src, integrity)
            })
            .collect(),
    );
    if !config.build.base_path.is_empty() {
        js_codegen.set_base_path(config.build.base_path.clone());
    }
    js_codegen.set_app_schemes(crate::project::config::project::app_schemes(&config));
    js_codegen.set_site_lang(config.page_lang());
    // `build.sourcemap`: each declaration's file, as the map's sources name
    // it (the project's own path, `src/pages/home.wf`).
    let sourcemap = config.build.sourcemap || options.sourcemap;
    let map_sources: Vec<String> = if sourcemap {
        let mut sources: Vec<String> = Vec::new();
        let mut keys = HashMap::new();
        for (i, decl) in program.declarations.iter().enumerate() {
            let (Some(key), Some(file)) = (
                crate::web::codegen::js::JsCodegen::map_key(decl),
                declaration_files.get(i),
            ) else {
                continue;
            };
            let file = file.replace('\\', "/");
            let index = match sources.iter().position(|s| *s == file) {
                Some(n) => n,
                None => {
                    sources.push(file);
                    sources.len() - 1
                }
            };
            keys.insert(key, index);
        }
        js_codegen.set_sourcemap(keys);
        sources
    } else {
        Vec::new()
    };
    // Their text, through the same `Vfs` (an editor's unsaved buffers too).
    let map_contents: Vec<Option<String>> = map_sources
        .iter()
        .map(|f| vfs.read_to_string(&project_dir.join(f)).ok())
        .collect();
    // An editor's preview: each element stamped with the id its source has.
    let nodes = options.studio.then(|| {
        let files = declaration_files.clone();
        crate::syntax::node_id::build_node_map_with(&program, &|i| files.get(i).cloned())
    });
    if let Some(map) = &nodes {
        js_codegen.set_studio(map.clone());
        out.report.borrow_mut().nodes = Some(map.clone());
    }
    let js = js_codegen.generate(&program);

    // The project as custom elements: one file any framework can load, and
    // a page listing the tags it published. There are no routes, no shell
    // and no pages — a component is the whole of what is published.
    if config.build.output_type == OutputType::Elements {
        let bundle = format!(
            "{js}{}",
            crate::web::codegen::elements::definitions(&program, &config)
        );
        let base = config.build.base_path.trim_end_matches('/');
        let shrink = |text: String, js: bool| {
            if !config.build.minify {
                return text;
            }
            if js {
                crate::web::codegen::minify::minify_js(&text)
            } else {
                crate::web::codegen::minify::minify_css(&text)
            }
        };
        read_back(out, "elements.js", &bundle, false)?;
        let small = shrink(bundle, true);
        // `build.sourcemap`: the markers out, into `elements.js.map`.
        let small = if sourcemap {
            let (code, map) = crate::web::codegen::sourcemap::extract(
                &small,
                "elements.js",
                &map_sources,
                &map_contents,
            );
            fs::write(output_dir.join("elements.js.map"), map)?;
            format!("{code}\n//# sourceMappingURL=elements.js.map\n")
        } else {
            small
        };
        fs::write(output_dir.join("elements.js"), small)?;
        fs::write(output_dir.join("styles.css"), shrink(css.clone(), false))?;
        fs::write(
            output_dir.join("index.html"),
            crate::web::codegen::elements::demo_page(&program, &config, base),
        )?;
        let public_dir = project_dir.join("public");
        if vfs.exists(&public_dir) {
            copy_dir_recursive(vfs, &public_dir, &output_dir)?;
        }
        let tags: Vec<String> = config
            .build
            .elements
            .iter()
            .map(|n| format!("<{}>", crate::web::codegen::elements::tag_name(n)))
            .collect();
        out.say(format!("  Elements: {}", tags.join(" ")));
        out.say(format!("  Output: {}/elements.js", config.build.output));
        warning_count += say_skipped(out, &confined);
        if warning_count == 0 {
            out.say("Build complete.");
        } else {
            out.say(format!("Build complete with {warning_count} warning(s)."));
        }
        return Ok(());
    }

    // Write output

    if config.build.pre_renders() {
        // SSG: generate per-page HTML files
        let app_body: Option<Vec<Statement>> = program.declarations.iter().find_map(|d| {
            if let Declaration::App(a) = d {
                Some(a.body.clone())
            } else {
                None
            }
        });
        let app_stmts = app_body.as_deref();
        // Expand user components into the static paint, so an exported site is a
        // genuine static site rather than a page of placeholder comments.
        let components: std::collections::HashMap<
            String,
            crate::syntax::parser::ast::ComponentDecl,
        > = program
            .declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Component(c) => Some((c.name.clone(), c.clone())),
                _ => None,
            })
            .collect();
        // The program's part of every page's build-time scope — `env`, the
        // stores' values, the constants — is the same for all of them:
        // worked out once rather than twice a page, it was most of what an
        // SSG build of a project with many pages cost.
        let program_scope = crate::project::static_eval::Scope::for_program(&program, &config.env);
        // A page whose inputs are what they were is taken from the cache.
        let keys = options.cache.map(|_| {
            let sheets: Vec<&String> = page_sheets.keys().collect();
            cache::PageKeys::new(&program, &config, &translations, &sheets, options.studio)
        });
        let render = |page: &crate::syntax::parser::ast::PageDecl,
                      route: &str,
                      params: &dyn std::fmt::Debug,
                      make: &dyn Fn() -> String|
         -> String {
            let (Some(cache), Some(keys)) = (options.cache, &keys) else {
                out.report.borrow_mut().pages_rendered += 1;
                return make();
            };
            let key = keys.page(page, route, params);
            // A page another route needs rendered waits, as it was painted.
            if options.only_route.is_some_and(|only| only != route)
                && !cache.holds(key)
                && let Some(html) = cache.last_paint(route)
            {
                out.report.borrow_mut().pages_deferred += 1;
                return html;
            }
            let (html, reused) = cache.page(route, key, make);
            let mut report = out.report.borrow_mut();
            if reused {
                report.pages_reused += 1;
            } else {
                report.pages_rendered += 1;
            }
            html
        };

        for decl in &program.declarations {
            if let Declaration::Page(page) = decl {
                let site = crate::web::codegen::ssg::SiteContext {
                    config: &config,
                    app_body: app_stmts,
                    translations: &translations,
                    components: &components,
                    program: &program,
                    program_scope: Some(&program_scope),
                    page_sheets: Some(&page_sheets),
                    node_map: nodes.as_ref(),
                };
                // A `:param` route is rendered once per value its `paths:`
                // names, and not at all without them.
                if page.path.contains(':') {
                    for (route, params) in
                        crate::web::codegen::ssg::static_routes(page, &program, &config.env)?
                    {
                        let sorted: std::collections::BTreeMap<_, _> = params.iter().collect();
                        let page_html = render(page, &route, &sorted, &|| {
                            crate::web::codegen::ssg::render_page_html_with_params(
                                page, &site, &route, &params,
                            )
                        });
                        let dir = page_dir(&output_dir, &page.name, &route)?;
                        fs::create_dir_all(&dir)?;
                        fs::write(dir.join("index.html"), &page_html)?;
                        written_html.push(dir.join("index.html"));
                        write_beside(&config, &dir, &page_html)?;
                    }
                    continue;
                }
                let page_html = render(page, &page.path, &(), &|| {
                    crate::web::codegen::render_page_html(page, &site)
                });

                // Determine output path
                let route = page.path.trim_start_matches('/');
                if route.is_empty() || route == "/" {
                    fs::write(output_dir.join("index.html"), &page_html)?;
                    written_html.push(output_dir.join("index.html"));
                } else if route == "*" {
                    // The catch-all is what a static host serves for a path it
                    // has no file for — GitHub Pages, Netlify and Cloudflare
                    // Pages all look for 404.html at the root. It used to be
                    // written to a directory literally named `*`.
                    fs::write(output_dir.join("404.html"), &page_html)?;
                    written_html.push(output_dir.join("404.html"));
                } else {
                    let dir = page_dir(&output_dir, &page.name, route)?;
                    fs::create_dir_all(&dir)?;
                    fs::write(dir.join("index.html"), &page_html)?;
                    written_html.push(dir.join("index.html"));
                    write_beside(&config, &dir, &page_html)?;
                }
            }
        }
        if let Some(cache) = options.cache {
            cache.finish();
        }
        out.say("  SSG: pre-rendered static pages");
    } else {
        // SPA: single index.html
        let html = generate_html(&config, &program);
        fs::write(output_dir.join("index.html"), html)?;
        written_html.push(output_dir.join("index.html"));
    }

    // The modules a program's `external` declarations import, in a file of
    // their own — a module, so `import` works, and separate so the page
    // chunks keep reading the bundle's names from the global scope.
    if let Some(module) = crate::web::codegen::js::externals_module(&config) {
        fs::write(output_dir.join("externals.js"), module)?;
    }

    // The project's own scripts, byte for byte: what the browser runs is
    // what is in `src/`, so a stack trace points at the author's own line.
    for script in &scripts {
        let to = output_dir.join(&script.href);
        if let Some(dir) = to.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(to, &script.source)?;
    }

    // `build.minify` — on by default, and read for the first time here: the
    // bundle and the sheet lose their comments and whitespace, and nothing
    // else, so a stack trace still reads as the compiler wrote it.
    // With `build.sourcemap`, its markers come out last, into `<name>.map`
    // beside it, which its last line names.
    let minify_js = |name: &str, src: String| -> Result<String> {
        read_back(out, name, &src, false)?;
        let done = if config.build.minify {
            let small = crate::web::codegen::minify::minify_js(&src);
            read_back(out, name, &small, true)?;
            small
        } else {
            src
        };
        if !sourcemap {
            return Ok(done);
        }
        let base = name.rsplit('/').next().unwrap_or(name);
        let (code, map) =
            crate::web::codegen::sourcemap::extract(&done, base, &map_sources, &map_contents);
        fs::write(output_dir.join(format!("{name}.map")), map)?;
        Ok(format!("{code}\n//# sourceMappingURL={base}.map\n"))
    };
    let minify_css = |src: String| {
        if config.build.minify {
            crate::web::codegen::minify::minify_css(&src)
        } else {
            src
        }
    };
    fs::write(output_dir.join("styles.css"), minify_css(css))?;
    fs::write(output_dir.join("app.js"), minify_js("app.js", js)?)?;

    // Each page in its own chunk, fetched when its route shows, and its own
    // sheet beside it when it has rules no other page reaches.
    let chunks = js_codegen.take_chunks();
    if !chunks.is_empty() || !page_sheets.is_empty() {
        let pages_dir = output_dir.join("pages");
        fs::create_dir_all(&pages_dir)?;
        for (name, source) in chunks {
            let file = format!("pages/{name}.js");
            fs::write(
                pages_dir.join(format!("{name}.js")),
                minify_js(&file, source)?,
            )?;
        }
        for (name, source) in page_sheets {
            fs::write(pages_dir.join(format!("{name}.css")), minify_css(source))?;
        }
    }

    // A meta tag cannot express `frame-ancestors`, and nothing in a static
    // bundle can set a response header, so the policy is also written where a
    // host can pick it up. Netlify, Cloudflare Pages and Vercel all read this
    // format; hosts that do not simply ignore the file.
    if config.build.csp {
        // A style value that reads state is written on the element, and
        // there is nowhere else for it to go. The policy has to say so, or
        // it is a promise the pages break: it is widened, and the build
        // names the pages that widened it, so the stricter policy is one
        // edit away rather than a mystery.
        let loose = inline_styled_pages(&output_dir, &written_html);
        if !loose.is_empty() {
            out.say(format!(
                "  Note: `style-src` allows inline styles, because {} page(s) carry one:",
                loose.len()
            ));
            for page in loose.iter().take(5) {
                out.say(format!("    {page}"));
            }
            if loose.len() > 5 {
                out.say(format!("    … and {} more", loose.len() - 5));
            }
            out.say("    A `style { }` value that reads state is set on the element; a literal is a class.");
        }
        fs::write(output_dir.join("_headers"), headers_file(&config))?;
        // A policy is a promise about what the pages contain. Reading them
        // back is how the promise is kept: a browser would enforce it and
        // show a blank page, and this says so at build time instead.
        let violations = check_csp(&output_dir, &written_html, &config);
        // A page the policy refuses is not left where a deploy that
        // ignores the exit code would publish it.
        for v in &violations {
            let _ = fs::remove_file(output_dir.join(&v.page));
        }
        let broken: Vec<Diagnostic> = violations
            .into_iter()
            .map(|v| {
                Diagnostic::coded(
                    "E902",
                    format!(
                        "{}, which the `{}` of the policy this build ships forbids",
                        v.what, v.directive
                    ),
                    format!("{}/{}", config.build.output.trim_end_matches('/'), v.page),
                    1,
                    1,
                )
                .with_hint(v.hint)
            })
            .collect();
        if !broken.is_empty() {
            out.findings(&broken);
            return Err(WebFluentError::Diagnostics(broken));
        }
    }

    // A crawler looks for both of these at the site root. The engine already
    // knows every route, so a sitemap is something it can write rather than
    // something the author has to keep in step by hand.
    if config.meta.sitemap {
        let map = crate::web::codegen::seo::sitemap(&config, &program);
        if let Some(xml) = &map {
            fs::write(output_dir.join("sitemap.xml"), xml)?;
        }
        fs::write(
            output_dir.join("robots.txt"),
            crate::web::codegen::seo::robots_txt(&config, map.is_some()),
        )?;
    }

    // Copy public/ assets to the root of the output directory
    let public_dir = project_dir.join("public");
    if vfs.exists(&public_dir) {
        copy_dir_recursive(vfs, &public_dir, &output_dir)?;
    }

    // The token block last, once everything that could name a token is on
    // disk — including whatever `public/` brought with it.
    let tokens_dropped = prune_root_tokens(&output_dir)?;

    // The service worker last of all, since its version is a hash of
    // everything else the build wrote.
    if let Some(offline) = &config.offline {
        write_service_worker(out, &output_dir, &config, offline, &program, &written_html)?;
    }

    // A `.gz` beside every text file, for a host that serves one when it has
    // it: compressed once here, harder than a server can afford per request.
    // An editor's preview is served by the editor, which never asks for one,
    // and has a reader waiting on it.
    if config.build.compress && !options.studio {
        let manifest = project_dir.join(PRECOMPRESSED);
        let mut all = read_precompressed(&manifest);
        let key = output_dir.to_string_lossy().to_string();
        let before = all.remove(&key).unwrap_or_default();
        let mut wrote = std::collections::BTreeSet::new();
        precompress(&output_dir, &output_dir, &before, &mut wrote)?;
        all.insert(key, wrote);
        if let Some(parent) = manifest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(
            &manifest,
            serde_json::to_string_pretty(&all).unwrap_or_default(),
        );
    }

    let (page_count, comp_count, store_count) = counts(&program);

    let locale_count = config.i18n.as_ref().map_or(0, |i| i.locales.len());
    if locale_count > 0 {
        out.say(format!(
            "  {page_count} pages, {comp_count} components, {store_count} stores, {locale_count} locales"
        ));
    } else {
        out.say(format!(
            "  {page_count} pages, {comp_count} components, {store_count} stores"
        ));
    }
    out.say(format!("  Output: {}/", config.build.output));
    // What this build weighs, against the budget and against the last one.
    // An editor's preview weighs only what the budget names: nothing reads
    // the rest of the sizes without `--stats`.
    let sizes = gzipped_sizes(
        &output_dir,
        (options.studio && !stats).then_some(&config.build.budget),
    )?;
    let previous = read_sizes(vfs, project_dir);
    warning_count += report_budget(out, &config.build.budget, &sizes);
    write_sizes(project_dir, &sizes)?;

    if stats {
        let (_, runtime_bytes) = js_codegen.runtime_stats();
        print_stats(
            out,
            &output_dir,
            js_codegen.runtime_report(),
            runtime_bytes,
            tokens_dropped,
            &sizes,
            previous.as_ref(),
        )?;
    }
    warning_count += say_skipped(out, &confined);
    crate::project::media::prune_cache(
        &media_cache,
        &crate::project::media::take_used(&media_cache),
    );
    if warning_count == 0 {
        out.say("Build complete.");
    } else {
        out.say(format!("Build complete with {warning_count} warning(s)."));
    }

    Ok(())
}

/// Say each link a walk of the project left out, and how many.
fn say_skipped(out: &Out, confined: &Confined) -> usize {
    let skipped = confined.skipped();
    for (path, why) in &skipped {
        out.say(format!("  Warning: {} was left out: {why}", path.display()));
    }
    skipped.len()
}

/// Where the build writes: the config's `build.output`, inside the
/// project. A directory the caller named (`BuildOptions::output_dir`) is
/// the caller's to choose, and so is one outside the project with
/// `--allow-output-outside`; a config's is not, since the config comes with
/// the project and a build writes, copies and deletes there.
///
/// # Errors
///
/// What [`project_output`] returns for the config's `build.output`.
pub fn output_dir_of(options: &BuildOptions, config: &ProjectConfig) -> Result<PathBuf> {
    let output = project_output(
        options.project_dir,
        &config.build.output,
        options.output_dir.is_some() || options.allow_output_outside,
    )?;
    Ok(output)
}

/// `output` (the config's `build.output`) under `project_dir`, refused when
/// it leads out of the project — by an absolute path, a `..`, or a link on
/// the way — unless `allow_outside`. A link already inside the output is
/// refused too: the build would write through it.
///
/// # Errors
///
/// [`WebFluentError::ConfigError`] when `output` leads out of the project
/// or the output holds a link, unless `allow_outside`.
pub fn project_output(project_dir: &Path, output: &str, allow_outside: bool) -> Result<PathBuf> {
    let joined = project_dir.join(output);
    if allow_outside {
        return Ok(joined);
    }
    let refuse = |why: &str| {
        WebFluentError::config(format!(
            "`build.output` is `{output}`, which {why}. A build writes only inside its project; \
             to write elsewhere, run it with `--allow-output-outside`"
        ))
    };
    let path = Path::new(output);
    if path.is_absolute() || output.starts_with('\\') {
        return Err(refuse("is an absolute path"));
    }
    if output.contains('\0') {
        return Err(refuse("holds a NUL character"));
    }
    if path.components().any(|c| {
        !matches!(
            c,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    }) || output.split(['/', '\\']).any(|part| part == "..")
    {
        return Err(refuse("climbs out of the project with `..`"));
    }
    if crate::base::vfs::escapes(project_dir, &joined) {
        return Err(refuse("leads out of the project through a link"));
    }
    if let Some(link) = link_inside(&joined) {
        return Err(WebFluentError::config(format!(
            "the output holds a link, {}, which a build would write through. \
             Remove it, or run the build with `--allow-output-outside`",
            link.display()
        )));
    }
    Ok(joined)
}

/// The first symbolic link under `dir`, walked without following one.
fn link_inside(dir: &Path) -> Option<PathBuf> {
    if crate::base::outfs::capturing() {
        return None;
    }
    let mut walk = vec![dir.to_path_buf()];
    while let Some(at) = walk.pop() {
        let Ok(entries) = std::fs::read_dir(&at) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_symlink() {
                return Some(entry.path());
            }
            if kind.is_dir() {
                walk.push(entry.path());
            }
        }
    }
    None
}

/// The directory a page's route is written to, under `output_dir`. A route
/// is the page's `path:` with its `paths:` values in place; each of its
/// parts must name a directory below the output, never `..` or the root.
fn page_dir(output_dir: &Path, page: &str, route: &str) -> Result<PathBuf> {
    let rel = route.trim_start_matches('/');
    let fine = !rel.contains(['\\', '\0'])
        && Path::new(rel)
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)));
    if !fine {
        return Err(WebFluentError::codegen(format!(
            "page {page}: the route `{route}` does not name a directory inside the output"
        )));
    }
    Ok(output_dir.join(rel))
}

/// Where a build records the `.gz` files it wrote, by output directory, so
/// the next one removes only those: a `.gz` it did not write — copied from
/// `public/`, or anybody's in a directory it was pointed at — is not its to
/// delete.
const PRECOMPRESSED: &str = ".wf-cache/precompressed.json";

fn read_precompressed(manifest: &Path) -> BTreeMap<String, std::collections::BTreeSet<String>> {
    fs::read_to_string(manifest)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// What the build weighs: every text file with its gzipped size, and the
/// runtime modules it kept and left out.
///
/// It reports; it never fails a build. A size budget that breaks the build is
/// a decision for a project to make, not for the compiler to impose.
/// Drop from `styles.css`'s `:root` the design tokens nothing in the output
/// names, and say how many went.
///
/// A token is kept when anything the build wrote — a stylesheet, a script, a
/// page, a file copied from `public/` — says `var(--name)`, and when another
/// kept token's value does. It runs over the finished output rather than the
/// generated sheet so a hand-written stylesheet or script in `public/` counts
/// as a reader, and so the `$token` a `style { }` block resolved is seen where
/// it landed.
fn prune_root_tokens(output_dir: &Path) -> Result<usize> {
    let sheet = output_dir.join("styles.css");
    let Ok(css) = fs::read_to_string(&sheet) else {
        return Ok(0);
    };
    let Some(start) = css.find(":root{").or_else(|| css.find(":root {")) else {
        return Ok(0);
    };
    let open = start + css[start..].find('{').unwrap_or(0) + 1;
    let Some(close) = css[open..].find('}').map(|n| open + n) else {
        return Ok(0);
    };

    // Everything the build wrote, minus the block itself.
    let mut read = String::new();
    let mut walk = vec![output_dir.to_path_buf()];
    while let Some(dir) = walk.pop() {
        for entry in fs::read_dir(&dir)? {
            let path = entry;
            if fs::is_dir(&path) {
                walk.push(path);
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext, "css" | "js" | "html" | "svg") {
                read.push_str(&fs::read_to_string(&path).unwrap_or_default());
                read.push('\n');
            }
        }
    }
    let body = &css[open..close];
    let declarations: Vec<&str> = body.split(';').filter(|d| !d.trim().is_empty()).collect();

    let named = |text: &str, token: &str| text.contains(&format!("var(--{token}"));
    // Without the block, so a token is not kept by its own declaration.
    let elsewhere = read.replacen(body, "", 1);
    let mut keep: Vec<bool> = declarations
        .iter()
        .map(|d| match d.split(':').next().map(str::trim) {
            Some(name) => named(&elsewhere, name.trim_start_matches("--")),
            None => true,
        })
        .collect();
    // A kept token's value may name another.
    loop {
        let mut more = false;
        let held: String = declarations
            .iter()
            .zip(&keep)
            .filter(|(_, k)| **k)
            .map(|(d, _)| *d)
            .collect::<Vec<_>>()
            .join(";");
        for (i, d) in declarations.iter().enumerate() {
            if keep[i] {
                continue;
            }
            if let Some(name) = d.split(':').next().map(str::trim)
                && named(&held, name.trim_start_matches("--"))
            {
                keep[i] = true;
                more = true;
            }
        }
        if !more {
            break;
        }
    }
    let dropped = keep.iter().filter(|k| !**k).count();
    if dropped == 0 {
        return Ok(0);
    }
    let kept: Vec<&str> = declarations
        .iter()
        .zip(&keep)
        .filter(|(_, k)| **k)
        .map(|(d, _)| *d)
        .collect();
    let mut out = String::with_capacity(css.len());
    out.push_str(&css[..open]);
    out.push_str(&kept.join(";"));
    if !kept.is_empty() {
        out.push(';');
    }
    out.push_str(&css[close..]);
    fs::write(&sheet, out)?;
    Ok(dropped)
}

/// Where a build records what it weighed, so the next one can show the diff.
///
/// Beside the project, not inside the output: the output is published, and
/// what the last build weighed is nobody's business but the author's.
const SIZES_FILE: &str = ".wf-sizes.json";

/// Every text output with its gzipped size, keyed by its path in the output.
///
/// Gzipped, because that is what a visitor downloads.
/// Under `build.clean_urls: "file"`, `contact/index.html` is written again
/// as `contact.html`, which a static host serves for `/contact` itself —
/// GitHub Pages answers `/contact` with a `301` to `/contact/` otherwise, so
/// the canonical address would be one that redirects. The page addresses its
/// assets from the site root under this setting, so the copy is exact.
fn write_beside(config: &ProjectConfig, dir: &Path, page_html: &str) -> Result<()> {
    if config.build.clean_urls != Some(crate::project::config::CleanUrls::File) {
        return Ok(());
    }
    let (Some(parent), Some(name)) = (dir.parent(), dir.file_name()) else {
        return Ok(());
    };
    let mut file = name.to_os_string();
    file.push(".html");
    fs::write(parent.join(file), page_html)?;
    Ok(())
}

fn gzipped_sizes(
    output_dir: &Path,
    only: Option<&BTreeMap<String, String>>,
) -> Result<BTreeMap<String, usize>> {
    let mut sizes = BTreeMap::new();
    let mut walk = vec![output_dir.to_path_buf()];
    while let Some(dir) = walk.pop() {
        for entry in fs::read_dir(&dir)? {
            let path = entry;
            if fs::is_dir(&path) {
                walk.push(path);
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "html" | "js" | "css" | "json" | "svg" | "xml") {
                continue;
            }
            if path.file_name().is_some_and(|n| n == SIZES_FILE) {
                continue;
            }
            let name = path
                .strip_prefix(output_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if only.is_some_and(|names| !names.contains_key(&name)) {
                continue;
            }
            let bytes = fs::read(&path)?;
            sizes.insert(name, crate::web::codegen::gzip::gzip(&bytes).len());
        }
    }
    Ok(sizes)
}

/// The sizes the last build recorded, for the diff `--stats` prints.
fn read_sizes(vfs: &dyn Vfs, project_dir: &Path) -> Option<BTreeMap<String, usize>> {
    let text = vfs.read_to_string(&project_dir.join(SIZES_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_sizes(project_dir: &Path, sizes: &BTreeMap<String, usize>) -> Result<()> {
    let text = serde_json::to_string_pretty(sizes).unwrap_or_default();
    fs::write(project_dir.join(SIZES_FILE), text)?;
    Ok(())
}

/// Warn about every output over its budget. Advisory: the build goes on.
fn report_budget(
    out: &Out,
    budget: &BTreeMap<String, String>,
    sizes: &BTreeMap<String, usize>,
) -> usize {
    let mut over = 0;
    for (name, limit) in budget {
        let Some(max) = parse_size(limit) else {
            out.stderr(format!(
                "  Warning: budget for '{name}' is not a size: '{limit}'"
            ));
            over += 1;
            continue;
        };
        match sizes.get(name) {
            Some(bytes) if *bytes > max => {
                out.stderr(format!(
                    "  Warning: {name} is {:.1} kB gzipped, over the {:.1} kB budget by {:.1} kB",
                    *bytes as f64 / 1024.0,
                    max as f64 / 1024.0,
                    (*bytes - max) as f64 / 1024.0
                ));
                over += 1;
            }
            Some(_) => {}
            None => {
                out.stderr(format!(
                    "  Warning: budget names '{name}', which this build does not write"
                ));
                over += 1;
            }
        }
    }
    over
}

/// `"40 kB"`, `"40kb"`, `"40960"` — bytes either way.
fn parse_size(text: &str) -> Option<usize> {
    let text = text.trim().to_ascii_lowercase();
    let (number, scale) = if let Some(n) = text.strip_suffix("mb") {
        (n, 1024 * 1024)
    } else if let Some(n) = text.strip_suffix("kb") {
        (n, 1024)
    } else if let Some(n) = text.strip_suffix('b') {
        (n, 1)
    } else {
        (text.as_str(), 1)
    };
    number
        .trim()
        .parse::<f64>()
        .ok()
        .map(|n| (n * scale as f64) as usize)
}

fn print_stats(
    out: &Out,
    output_dir: &Path,
    modules: &[crate::web::runtime::Kept],
    runtime_bytes: usize,
    tokens_dropped: usize,
    sizes: &BTreeMap<String, usize>,
    previous: Option<&BTreeMap<String, usize>>,
) -> Result<()> {
    fn kb(n: usize) -> String {
        format!("{:.1} kB", n as f64 / 1024.0)
    }
    let mut files: Vec<(String, usize, usize)> = Vec::new();
    let mut walk = vec![output_dir.to_path_buf()];
    while let Some(dir) = walk.pop() {
        for entry in fs::read_dir(&dir)? {
            let path = entry;
            if fs::is_dir(&path) {
                walk.push(path);
                continue;
            }
            let name = path
                .strip_prefix(output_dir)
                .unwrap_or(&path)
                .to_string_lossy();
            if name.ends_with(".gz") || name == SIZES_FILE {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "html" | "js" | "css" | "json" | "svg" | "xml") {
                continue;
            }
            let bytes = fs::len(&path)? as usize;
            let gz = fs::len(path.with_extension(format!("{ext}.gz"))).map_or(0, |n| n as usize);
            files.push((name.to_string(), bytes, gz));
        }
    }
    files.sort_by_key(|f| std::cmp::Reverse(f.1));
    let total: usize = files.iter().map(|f| f.1).sum();
    out.say("\n  What it weighs");
    for (name, bytes, _) in files.iter().take(20) {
        let gz = sizes.get(name).copied().unwrap_or(0);
        // Against the last build in this output directory, when there was one.
        let delta = match previous.and_then(|p| p.get(name)) {
            Some(was) if *was != gz => {
                let diff = gz as i64 - *was as i64;
                let sign = if diff > 0 { "+" } else { "−" };
                let size = diff.unsigned_abs() as usize;
                if size < 1024 {
                    format!("  {sign}{size} B")
                } else {
                    format!("  {sign}{:.1} kB", size as f64 / 1024.0)
                }
            }
            _ => String::new(),
        };
        out.say(format!(
            "    {:<34} {:>9}  ({} gzipped){}",
            name,
            kb(*bytes),
            kb(gz),
            delta
        ));
    }
    if files.len() > 20 {
        out.say(format!("    … and {} more", files.len() - 20));
    }
    out.say(format!("    {:<34} {:>9}", "total", kb(total)));

    let names: Vec<&str> = modules.iter().map(|m| m.name).collect();
    let left_out = crate::web::runtime::dropped(&names);
    let all = crate::web::runtime::full().len();
    out.say(format!(
        "\n  Runtime: {} of {} modules, {} of {} (before minifying)",
        modules.len(),
        modules.len() + left_out.len(),
        kb(runtime_bytes),
        kb(all)
    ));
    let mut by_size: Vec<&crate::web::runtime::Kept> = modules.iter().collect();
    by_size.sort_by_key(|m| std::cmp::Reverse(m.bytes));
    for m in by_size {
        out.say(format!(
            "    {:<12} {:>8}  {}",
            m.name,
            kb(m.bytes),
            m.reason
        ));
    }
    if left_out.is_empty() {
        out.say("    left out: nothing");
    } else {
        out.say(format!("    left out: {}", left_out.join(" ")));
    }
    if tokens_dropped > 0 {
        out.say(format!(
            "\n  Tokens: {tokens_dropped} nothing in the output names, left out"
        ));
    }
    Ok(())
}

fn load_translations(
    vfs: &dyn Vfs,
    out: &Out,
    project_dir: &Path,
    i18n_config: &crate::project::config::project::I18nConfig,
) -> Result<HashMap<String, HashMap<String, String>>> {
    let dir = project_dir.join(&i18n_config.dir);
    if !vfs.exists(&dir) {
        out.say(format!(
            "  Warning: translations directory '{}' not found",
            i18n_config.dir
        ));
    }
    crate::project::i18n::load_via(vfs, project_dir, i18n_config)
}

/// Write `<file>.gz` beside every text file under `dir` that is worth it —
/// a file a few hundred bytes long fits in one packet either way, and one
/// gzip does not shrink is left alone. A stale `.gz` an earlier build wrote
/// (`before`, by its path under `root`) whose source no longer exists is
/// removed, so a host never serves it; any other `.gz` is left as it is.
/// What this one writes goes in `wrote`.
fn precompress(
    root: &Path,
    dir: &Path,
    before: &std::collections::BTreeSet<String>,
    wrote: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    let ours = |path: &Path| {
        path.strip_prefix(root)
            .ok()
            .map(|rel| rel.to_string_lossy().replace('\\', "/"))
            .is_some_and(|rel| before.contains(&rel))
    };
    for entry in fs::read_dir(dir)? {
        let path = entry;
        if fs::is_dir(&path) {
            precompress(root, &path, before, wrote)?;
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if ext == "gz" {
            if !fs::exists(path.with_extension("")) && ours(&path) {
                fs::remove_file(&path)?;
            }
            continue;
        }
        if !crate::web::codegen::gzip::is_text_extension(ext) {
            continue;
        }
        let data = fs::read(&path)?;
        let gz_path = PathBuf::from(format!("{}.gz", path.display()));
        let gz = (data.len() >= 1024)
            .then(|| crate::web::codegen::gzip::gzip(&data))
            .filter(|gz| gz.len() < data.len());
        match gz {
            Some(gz) => {
                fs::write(&gz_path, gz)?;
                if let Ok(rel) = gz_path.strip_prefix(root) {
                    wrote.insert(rel.to_string_lossy().replace('\\', "/"));
                }
            }
            None if ours(&gz_path) => {
                let _ = fs::remove_file(&gz_path);
            }
            None => {}
        }
    }
    Ok(())
}

/// Copy the directory `src` of `vfs` into `dst` on the disk.
fn copy_dir_recursive(vfs: &dyn Vfs, src: &Path, dst: &Path) -> Result<()> {
    if !fs::exists(dst) {
        fs::create_dir_all(dst)?;
    }

    for src_path in vfs.read_dir(src)? {
        let Some(name) = src_path.file_name() else {
            continue;
        };
        let dst_path = dst.join(name);

        if vfs.is_dir(&src_path) {
            copy_dir_recursive(vfs, &src_path, &dst_path)?;
        } else {
            vfs.copy_out(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Everything in the built pages that the policy beside them forbids.
///
/// It reads the output, not the source, so a hand-written file copied from
/// `public/` is held to the policy as well as anything the compiler wrote.
fn check_csp(
    output_dir: &Path,
    written: &[PathBuf],
    config: &ProjectConfig,
) -> Vec<crate::web::codegen::csp::Violation> {
    let policy = crate::project::config::project::csp_meta_policy(config);
    let mut found = Vec::new();
    for path in written {
        let Ok(html) = fs::read_to_string(path) else {
            continue;
        };
        let page = path
            .strip_prefix(output_dir)
            .unwrap_or(path)
            .display()
            .to_string();
        found.extend(crate::web::codegen::csp::check(&page, &html, &policy));
    }
    found.sort_by(|a, b| a.page.cmp(&b.page).then(a.what.cmp(&b.what)));
    found.dedup();
    found
}

/// The `_headers` file a static host reads to set response headers.
///
/// `frame-ancestors` and `X-Content-Type-Options` cannot be set from a meta tag,
/// so a site that only carries the CSP in its HTML is still framable and still
/// subject to MIME sniffing.
fn headers_file(config: &ProjectConfig) -> String {
    let mut out = format!(
        "/*\n\
         \x20 Content-Security-Policy: {}\n\
         \x20 X-Content-Type-Options: nosniff\n\
         \x20 Referrer-Policy: strict-origin-when-cross-origin\n\
         \x20 X-Frame-Options: {}\n",
        crate::project::config::project::csp_policy(config),
        crate::project::config::project::frame_options(config)
    );
    // A browser checks the worker for a new version on every visit; a host
    // that let it be cached would keep a deploy from reaching anyone.
    if config.offline.is_some() {
        let base = config.build.base_path.trim_end_matches('/');
        out.push_str(&format!("\n{base}/sw.js\n  Cache-Control: no-cache\n"));
    }
    out
}

/// The pages this build wrote that carry a `style=` attribute, as the
/// reader would ask for them.
fn inline_styled_pages(output_dir: &Path, written: &[PathBuf]) -> Vec<String> {
    let mut out: Vec<String> = written
        .iter()
        .filter(|p| fs::read_to_string(p).is_ok_and(|html| html.contains(" style=\"")))
        .map(|p| {
            p.strip_prefix(output_dir)
                .unwrap_or(p)
                .display()
                .to_string()
        })
        .collect();
    out.sort();
    out
}

/// The page a route a static build wrote belongs to: its own `path`, or the
/// `:param` pattern it was rendered from.
fn page_for_route<'a>(
    program: &'a crate::syntax::parser::ast::Program,
    route: &str,
) -> Option<&'a crate::syntax::parser::ast::PageDecl> {
    let segments: Vec<&str> = route.split('/').filter(|s| !s.is_empty()).collect();
    program.declarations.iter().find_map(|d| {
        let crate::syntax::parser::ast::Declaration::Page(page) = d else {
            return None;
        };
        let pattern: Vec<&str> = page.path.split('/').filter(|s| !s.is_empty()).collect();
        let fits = pattern.len() == segments.len()
            && pattern
                .iter()
                .zip(&segments)
                .all(|(p, s)| p.starts_with(':') || p == s);
        fits.then_some(page)
    })
}

/// Write `sw.js`: what a first visit stores — the shell, each route
/// `offline.precache` names with its own chunk and sheet, the fallback — and
/// the version that says when it has all changed.
fn write_service_worker(
    out: &Out,
    output_dir: &Path,
    config: &ProjectConfig,
    offline: &crate::project::config::OfflineConfig,
    program: &crate::syntax::parser::ast::Program,
    written_html: &[PathBuf],
) -> Result<()> {
    use crate::build::offline::{Worker, route_matches, service_worker, version_of};
    let base = config.build.base_path.trim_end_matches('/').to_string();
    let site = |rel: &str| format!("{base}/{}", rel.trim_start_matches('/'));
    let exists = |rel: &str| fs::is_file(output_dir.join(rel));

    let mut precache: Vec<String> = Vec::new();
    let add = |rel: &str, list: &mut Vec<String>| {
        let url = site(rel);
        if !list.contains(&url) {
            list.push(url);
        }
    };
    for shell in ["app.js", "styles.css", "externals.js"] {
        if exists(shell) {
            add(shell, &mut precache);
        }
    }
    let chunks_of = |page: &crate::syntax::parser::ast::PageDecl, list: &mut Vec<String>| {
        for ext in ["js", "css"] {
            let rel = format!("pages/{}.{ext}", page.name);
            if exists(&rel) {
                let url = site(&rel);
                if !list.contains(&url) {
                    list.push(url);
                }
            }
        }
    };

    let mut routes: Vec<(String, String)> = Vec::new();
    let mut shell = None;
    let mut fallback = None;
    let mut matched = 0usize;
    if config.build.pre_renders() {
        for html in written_html {
            let Ok(rel) = html.strip_prefix(output_dir) else {
                continue;
            };
            let rel = rel.to_string_lossy().replace('\\', "/");
            if rel == "404.html" {
                continue;
            }
            let route = match rel.strip_suffix("index.html") {
                Some("") => "/".to_string(),
                Some(dir) => format!("/{}", dir.trim_end_matches('/')),
                None => continue,
            };
            let wanted = route_matches(&offline.precache, &route);
            let is_fallback = offline.fallback.as_deref() == Some(route.as_str());
            if !wanted && !is_fallback {
                continue;
            }
            matched += usize::from(wanted);
            add(&rel, &mut precache);
            if let Some(page) = page_for_route(program, &route) {
                chunks_of(page, &mut precache);
            }
            routes.push((route.clone(), site(&rel)));
            if is_fallback {
                fallback = Some(site(&route));
            }
        }
    } else {
        // One shell renders every route; a route's chunk is what it needs.
        add("index.html", &mut precache);
        shell = Some(site("index.html"));
        for d in &program.declarations {
            let crate::syntax::parser::ast::Declaration::Page(page) = d else {
                continue;
            };
            if page.path.contains(':') || page.path == "*" {
                continue;
            }
            let wanted = route_matches(&offline.precache, &page.path);
            let is_fallback = offline.fallback.as_deref() == Some(page.path.as_str());
            if wanted || is_fallback {
                matched += usize::from(wanted);
                chunks_of(page, &mut precache);
                // The shell answers a stored route; the worker has to know
                // which those are, or it sends them to the fallback too.
                routes.push((page.path.clone(), site("index.html")));
            }
            if is_fallback {
                fallback = Some(site(&page.path));
            }
        }
    }
    if matched == 0 {
        out.say(format!(
            "  Warning: `offline.precache` ({}) names no route this build writes; only the shell is stored",
            offline.precache.join(", ")
        ));
    }

    // Everything on disk but the worker and the compressed copies.
    let mut files = Vec::new();
    collect_files(output_dir, output_dir, &mut files)?;
    let version = version_of(&files);

    let worker = Worker {
        version: &version,
        base: &base,
        precache,
        routes,
        shell,
        fallback,
        policies: offline
            .cache
            .iter()
            .map(|(g, s)| (g.clone(), s.clone()))
            .collect(),
        sync: offline.sync,
    };
    fs::write(output_dir.join("sw.js"), service_worker(&worker))?;
    out.say(format!(
        "  Offline: sw.js, {} file(s) stored, version {}",
        worker.precache.len(),
        &version[..8]
    ));
    Ok(())
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry;
        if fs::is_dir(&path) {
            collect_files(root, &path, out)?;
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if rel == "sw.js" || rel.ends_with(".gz") {
            continue;
        }
        out.push((rel, fs::read(&path)?));
    }
    Ok(())
}

/// The build reads back what it wrote: a compiled script the browser would
/// refuse stops the build, as the compiler's fault rather than the author's.
/// It used to ship, and the page died before it drew.
fn read_back(out: &Out, name: &str, source: &str, minified: bool) -> Result<()> {
    let Err(fault) = crate::web::codegen::jscheck::check_js(source) else {
        return Ok(());
    };
    let stage = if minified { ", after minifying" } else { "" };
    let d = Diagnostic::coded(
        "E901",
        format!(
            "the compiler wrote JavaScript a browser would refuse, in {name}{stage}: {}",
            fault.message
        ),
        name,
        fault.line,
        1,
    )
    .with_hint(format!(
        "It wrote `{}`. This is a bug in WebFluent, not in your program. Please report it with the source that produced it: https://github.com/monzeromer-lab/WebFluent/issues",
        crate::web::codegen::jscheck::line_of(source, fault.line)
    ));
    out.findings(std::slice::from_ref(&d));
    Err(WebFluentError::Diagnostics(vec![d]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_script_the_browser_would_refuse_is_e901_the_compiler_s_fault() {
        let out = Out {
            report: RefCell::new(BuildReport::default()),
            found: RefCell::new(Vec::new()),
            on_line: None,
            format: Format::Human,
            color: false,
            vfs: &FsVfs,
            project_dir: Path::new("."),
        };
        assert!(
            read_back(
                &out,
                "app.js",
                "const a = 1;\nfunction f() { return a; }\n",
                false
            )
            .is_ok()
        );
        let Err(WebFluentError::Diagnostics(found)) =
            read_back(&out, "app.js", "const a = 1;\nconst b = (2;\n", true)
        else {
            panic!("an unclosed bracket was read back as a script");
        };
        assert_eq!(found[0].code, "E901");
        assert!(
            found[0].message.contains("after minifying"),
            "{}",
            found[0].message
        );
        assert!(
            found[0]
                .hint
                .as_deref()
                .is_some_and(|h| h.contains("bug in WebFluent")),
            "{:?}",
            found[0].hint
        );
    }
}
