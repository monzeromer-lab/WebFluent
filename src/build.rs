//! `wf build`, as a library call.
//!
//! [`build`] reads a project through a [`Vfs`] — the disk, or the disk with
//! an editor's unsaved buffers over it — checks it, and writes the site to
//! its output directory. `wf build` is this function and a loop that prints
//! what it reports, so the two cannot drift: the report holds every line the
//! command prints, in order and on its stream, and the findings among them
//! with their code, file, line and column, so a caller that shows them its
//! own way has no text to parse.

use crate::codegen::{
    JsCodegen, PdfCodegen, SlidesCodegen, dark_css, generate_css_for, generate_html,
};
use crate::config::ProjectConfig;
use crate::config::project::OutputType;
use crate::error::{A11yWarning, Diagnostic, Result, VocabWarning, WebFluentError};
use crate::parser::{Declaration, Program, Statement};
use crate::vfs::{FsVfs, Vfs};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// What to build, and where to read it from.
pub struct BuildOptions<'a> {
    /// The directory holding `webfluent.app.json`.
    pub project_dir: &'a Path,
    /// Where the project is read from. What the build writes — the output,
    /// the image cache, `.wf-sizes.json` — goes to the disk.
    pub vfs: &'a dyn Vfs,
    /// Where to write the site instead of the config's `build.output`,
    /// relative to the project or absolute.
    pub output_dir: Option<PathBuf>,
    /// Report what the output weighs, as `wf build --stats` does.
    pub stats: bool,
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
            stats: false,
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
/// newline: a diagnostic with a hint is one line of two.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildLine {
    pub stream: Stream,
    pub text: String,
}

/// A finding the build reported: an error that stopped it, or a warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildDiagnostic {
    /// The code it is filed under in the diagnostics chapter — `T06`,
    /// `A01`, `V01` — when it has one.
    pub code: Option<String>,
    /// What is wrong, in a sentence, without the code.
    pub message: String,
    /// The file it is about, relative to the project (`src/pages/Home.wf`);
    /// `webfluent.app.json` for one about the config; empty for one about
    /// no file.
    pub file: String,
    /// 1-based; 0 for a finding with no place in its file.
    pub line: usize,
    /// 1-based; 0 for a finding with no place in its file.
    pub column: usize,
    pub hint: Option<String>,
    /// Exactly what `wf build` prints for it.
    pub text: String,
}

impl BuildDiagnostic {
    /// A finding of the compiler's, which `wf build` prints as `text`.
    fn of(finding: &Diagnostic, text: String) -> Self {
        let (code, message) = split_code(&finding.message);
        Self {
            code,
            message,
            file: finding.file.clone(),
            line: finding.line,
            column: finding.column,
            hint: finding.hint.clone(),
            text,
        }
    }

    fn of_lint(warning: &A11yWarning) -> Self {
        Self {
            code: Some(warning.rule_id.clone()),
            message: warning.message.clone(),
            file: warning.file.clone(),
            line: warning.line,
            column: warning.column,
            hint: Some(warning.hint.clone()),
            text: warning.to_string(),
        }
    }

    fn of_vocabulary(warning: &VocabWarning) -> Self {
        Self {
            code: Some(warning.rule_id.clone()),
            message: warning.message.clone(),
            file: warning.file.clone(),
            line: warning.line,
            column: warning.column,
            hint: warning.hint.clone(),
            text: warning.to_string(),
        }
    }

    /// A finding about a file as a whole, or about no file.
    fn about(file: &str, message: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            code: None,
            message: message.into(),
            file: file.to_string(),
            line: 0,
            column: 0,
            hint: None,
            text: text.into(),
        }
    }
}

/// `"[T06] `S` has no member `x`"` → `T06` and the sentence after it.
fn split_code(message: &str) -> (Option<String>, String) {
    if let Some(rest) = message.strip_prefix('[')
        && let Some((code, sentence)) = rest.split_once("] ")
        && code.len() == 3
        && code.starts_with(|c: char| c.is_ascii_uppercase())
        && code[1..].bytes().all(|b| b.is_ascii_digit())
    {
        return (Some(code.to_string()), sentence.to_string());
    }
    (None, message.to_string())
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
    /// Every error the build found. A failed build has at least one.
    pub errors: Vec<BuildDiagnostic>,
    /// Every warning the build printed, whether or not `Build complete with
    /// N warning(s)` counts it.
    pub warnings: Vec<BuildDiagnostic>,
}

/// A build that stopped: what it reported until it did, and why.
#[derive(Debug)]
pub struct BuildFailure {
    pub report: Box<BuildReport>,
    /// What `wf build` prints last, before it exits with 1.
    pub error: WebFluentError,
}

impl std::fmt::Display for BuildFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for BuildFailure {}

/// Build the project, as `wf build` does.
pub fn build(options: &BuildOptions) -> std::result::Result<BuildReport, BuildFailure> {
    let started = Instant::now();
    let mut out = Out {
        report: BuildReport::default(),
        on_line: options.on_line,
    };
    let outcome = run(options, &mut out);
    let mut report = out.report;
    report.duration = started.elapsed();
    let Err(error) = outcome else {
        return Ok(report);
    };
    // A parse error is a finding like the others, with a place. An error
    // no finding stands for yet — a missing file, a config that does not
    // parse — is kept as one with no place, so a failed build always has
    // an error to show.
    match &error {
        WebFluentError::LexerError(finding) | WebFluentError::ParseError(finding) => {
            report
                .errors
                .push(BuildDiagnostic::of(finding, error.to_string()));
        }
        WebFluentError::CodegenError(message)
        | WebFluentError::ConfigError(message)
        | WebFluentError::IoError(message)
        | WebFluentError::EditError(message)
            if report.errors.is_empty() =>
        {
            report.errors.push(BuildDiagnostic::about(
                "",
                message.clone(),
                error.to_string(),
            ));
        }
        _ => {}
    }
    Err(BuildFailure {
        report: Box::new(report),
        error,
    })
}

/// What the build has said so far.
struct Out<'a> {
    report: BuildReport,
    on_line: Option<&'a dyn Fn(&BuildLine)>,
}

impl Out<'_> {
    fn line(&mut self, stream: Stream, text: String) {
        let line = BuildLine { stream, text };
        if let Some(on_line) = self.on_line {
            on_line(&line);
        }
        self.report.lines.push(line);
    }

    fn stdout(&mut self, text: impl Into<String>) {
        self.line(Stream::Stdout, text.into());
    }

    fn stderr(&mut self, text: impl Into<String>) {
        self.line(Stream::Stderr, text.into());
    }

    /// Print an error to stderr, and keep it.
    fn error(&mut self, finding: BuildDiagnostic) {
        self.stderr(finding.text.clone());
        self.report.errors.push(finding);
    }

    /// Print a warning to stderr, and keep it.
    fn warning(&mut self, finding: BuildDiagnostic) {
        self.stderr(finding.text.clone());
        self.report.warnings.push(finding);
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

fn run(options: &BuildOptions, out: &mut Out) -> Result<()> {
    let project_dir = options.project_dir;
    let vfs = options.vfs;
    let stats = options.stats;
    let mut config = ProjectConfig::load_via(vfs, project_dir)?;
    // `env` from a `.env` file and the shell, on top of the config's own.
    config.resolve_env_via(vfs, project_dir, std::env::vars());
    if let Some(output) = &options.output_dir {
        config.build.output = output.to_string_lossy().to_string();
    }

    out.stdout(format!("Building {}...", config.name));
    for unknown in ProjectConfig::unknown_keys_via(vfs, project_dir) {
        let text = format!("Warning: webfluent.app.json: {unknown}");
        out.warning(BuildDiagnostic::about("webfluent.app.json", unknown, text));
    }

    // The images the program names are written at every width a page will
    // ask for, before anything checks the program — so a name resolves to
    // the asset it became, with its real size and colour.
    let output_dir = project_dir.join(&config.build.output);
    out.report.output_dir = output_dir.clone();
    fs::create_dir_all(&output_dir)?;
    let media = config.build.media.settings();
    let (program, declaration_files) = read_project_via(
        vfs,
        project_dir,
        Some((&output_dir, &media, &config.build.base_path)),
    )?;
    out.report.routes = routes(&program, &declaration_files);
    (out.report.pages, out.report.components, out.report.stores) = counts(&program);
    // Every HTML file this build writes. The policy check speaks for what
    // this build put in the output, not for whatever else is in the
    // directory — a file an older layout left behind is a real problem,
    // but not one this build can answer for.
    let mut written_html: Vec<PathBuf> = Vec::new();

    // What the pages will contain, decided before any of them is written:
    // the policy each one ships has to describe that page.
    config.build.inline_styles = crate::codegen::csp::writes_inline_styles(&program);
    config.build.script_origins = crate::codegen::js::external_modules(&program)
        .iter()
        .filter_map(|(url, _)| {
            let at = url.find("://")?;
            let host = url[at + 3..].split('/').next()?;
            (!host.is_empty()).then(|| format!("{}//{host}", &url[..at + 1]))
        })
        .collect();
    let config = config;
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

    // A stylesheet or a font from somewhere else is code that origin can
    // change after it was read. A hash is how the browser checks it has
    // not; the build cannot work one out without fetching the file, so it
    // says which assets are missing theirs.
    for url in config
        .meta
        .fonts
        .iter()
        .chain(config.meta.stylesheets.iter())
    {
        if !url.contains("://") || config.meta.integrity.contains_key(url) {
            continue;
        }
        // A Google Fonts stylesheet is written per user-agent, so its
        // bytes differ between readers and a hash cannot match.
        if url.contains("fonts.googleapis.com") {
            continue;
        }
        let message = format!("`{url}` is loaded from another origin with no integrity hash");
        let hint =
            "Add one to `meta.integrity`, so a change at that origin cannot reach your readers";
        out.stdout(format!("  Warning: {message}"));
        out.stdout(format!("    {hint}"));
        out.report.warnings.push(BuildDiagnostic {
            hint: Some(hint.to_string()),
            ..BuildDiagnostic::about(
                "webfluent.app.json",
                message.clone(),
                format!("  Warning: {message}\n    {hint}"),
            )
        });
    }

    // A value in `env` the page reads is a value in the bundle. A name
    // that has not said it is public stops the build at the line that
    // reads it, rather than shipping as a key anyone can read.
    let leaked = crate::linter::lint_env_via(vfs, project_dir, &config, &declaration_files);
    if !leaked.is_empty() {
        for diagnostic in &leaked {
            out.error(BuildDiagnostic::of(diagnostic, diagnostic.to_string()));
        }
        return Err(crate::error::WebFluentError::CodegenError(format!(
            "{} `env` name(s) a page may not read",
            leaked.len()
        )));
    }

    // A reference to nothing — an undeclared component, a route to a page that
    // does not exist, two pages with one name — is a broken site, not a style
    // question, so it stops the build the way a parse error does.
    let semantic = crate::linter::validate_semantics_in(&program, &file_of);
    if !semantic.is_empty() {
        for diagnostic in &semantic {
            out.error(BuildDiagnostic::of(diagnostic, diagnostic.to_string()));
        }
        return Err(WebFluentError::CodegenError(format!(
            "{} semantic error(s)\n{}",
            semantic.len(),
            semantic
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )));
    }

    // What the new grammar wrote, checked against what the components
    // declare: a flag, case, event, slot or part that resolves to nothing is
    // a broken site, and stops the build like a parse error.
    let mut findings = crate::sema::check(&program, &file_of);
    // Then the types: what every name is, and the values that do not fit.
    let typed = crate::sema::types::check_in(&program, &file_of, &source_of);
    findings.errors.extend(typed.findings.errors);
    findings.warnings.extend(typed.findings.warnings);
    // A name nothing declares is a ReferenceError the first time the page
    // reads it; the build says so where it is written.
    findings.errors.extend(typed.unresolved);
    for warning in &findings.warnings {
        out.warning(BuildDiagnostic::of(warning, warning.as_warning()));
    }
    if !findings.errors.is_empty() {
        for error in &findings.errors {
            out.error(BuildDiagnostic::of(error, error.to_string()));
        }
        return Err(WebFluentError::CodegenError(format!(
            "{} error(s)\n{}",
            findings.errors.len(),
            findings
                .errors
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )));
    }
    // Then lowered onto the vocabulary the code generators — and the
    // linters, which read the words the stylesheet knows — read.
    let program = crate::sema::lower(program);

    // Run accessibility linter
    let mut a11y_warnings = crate::linter::lint_accessibility_in(&program, &file_of);
    // Contrast is checked against the tokens this build will actually ship, so
    // the ratio reported is the one a reader will experience.
    if let Ok(tokens) = crate::themes::resolve_tokens(&program, &config.theme) {
        a11y_warnings.extend(crate::linter::lint_contrast_in(&program, &tokens, &file_of));
    }
    // What is declared and never read.
    a11y_warnings.extend(crate::linter::lint_unused_in(&program, &file_of));
    // A component this build publishes as a custom element is placed — by
    // whoever loads it. Nothing in the project places it, and that is the
    // point.
    if !config.build.elements.is_empty() {
        a11y_warnings.retain(|w| {
            w.rule_id != "U03"
                || !config
                    .build
                    .elements
                    .iter()
                    .any(|name| w.message.contains(&format!("`{name}`")))
        });
    }
    for warning in &a11y_warnings {
        out.warning(BuildDiagnostic::of_lint(warning));
    }
    // A bare word that resolves to nothing, or a real modifier with no rule
    // behind it, does nothing on screen. The LSP has reported these for a
    // while; a build from the command line said nothing.
    // The author's own stylesheets — every `.css` under `src/` — ship in
    // `styles.css`, and a modifier class one of them defines is a real one.
    let project_css =
        crate::codegen::project_css::bundle_via(vfs, project_dir, &project_dir.join("src"))?;
    let vocab_warnings = crate::linter::lint_vocabulary_with(&program, &project_css, &file_of);
    for warning in &vocab_warnings {
        out.warning(BuildDiagnostic::of_vocabulary(warning));
    }
    let mut warning_count = a11y_warnings.len() + vocab_warnings.len() + findings.warnings.len();

    // PDF output mode
    if config.build.output_type == OutputType::Pdf {
        // Validate: reject interactive elements
        let pdf_errors = crate::linter::validate_for_pdf(&program);
        if !pdf_errors.is_empty() {
            for err in &pdf_errors {
                let text = err.to_string();
                let message = text.trim_start_matches("error[pdf]: ").to_string();
                out.error(BuildDiagnostic::about("", message, text));
            }
            return Err(WebFluentError::CodegenError(format!(
                "{} element(s) not allowed in PDF output",
                pdf_errors.len()
            )));
        }

        let mut pdf_codegen = PdfCodegen::new(&config.build.pdf);
        // Where the pictures are, so a report shows the chart rather than a
        // box that says there was one.
        pdf_codegen.set_asset_root(project_dir.to_path_buf());
        pdf_codegen.hold_warnings();
        let pdf_bytes = pdf_codegen.generate(&program);
        for warning in pdf_codegen.take_warnings() {
            let message = warning.trim_start_matches("warning[pdf]: ").to_string();
            out.warning(BuildDiagnostic::about("", message, warning));
        }

        let output_dir = project_dir.join(&config.build.output);
        fs::create_dir_all(&output_dir)?;

        let filename = config
            .build
            .pdf
            .output_filename
            .clone()
            .unwrap_or_else(|| format!("{}.pdf", config.name));
        fs::write(output_dir.join(&filename), &pdf_bytes)?;

        let page_count = pdf_codegen.page_count();
        out.stdout(format!(
            "  PDF: {} bytes, {} page(s)",
            pdf_bytes.len(),
            page_count
        ));
        out.stdout(format!("  Output: {}/{}", config.build.output, filename));
        if warning_count == 0 {
            out.stdout("Build complete.");
        } else {
            out.stdout(format!("Build complete with {} warning(s).", warning_count));
        }
        return Ok(());
    }

    // Slides output mode (PDF deck)
    if config.build.output_type == OutputType::Slides {
        let slide_errors = crate::linter::validate_for_slides(&program);
        if !slide_errors.is_empty() {
            for err in &slide_errors {
                let text = err.to_string();
                let message = text.trim_start_matches("error[slides]: ").to_string();
                out.error(BuildDiagnostic::about("", message, text));
            }
            return Err(WebFluentError::CodegenError(format!(
                "{} slide validation error(s)",
                slide_errors.len()
            )));
        }

        let mut slides_codegen = SlidesCodegen::new(&config.build.slides);
        slides_codegen.set_asset_root(project_dir.to_path_buf());
        slides_codegen.hold_warnings();
        let pdf_bytes = slides_codegen.generate(&program);
        for warning in slides_codegen.take_warnings() {
            let message = warning.trim_start_matches("warning[slides]: ").to_string();
            out.warning(BuildDiagnostic::about("", message, warning));
        }

        let output_dir = project_dir.join(&config.build.output);
        fs::create_dir_all(&output_dir)?;

        let filename = config
            .build
            .slides
            .output_filename
            .clone()
            .unwrap_or_else(|| format!("{}.pdf", config.name));
        fs::write(output_dir.join(&filename), &pdf_bytes)?;

        let slide_count = slides_codegen.slide_count();
        out.stdout(format!(
            "  Slides: {} bytes, {} slide(s)",
            pdf_bytes.len(),
            slide_count
        ));
        out.stdout(format!("  Output: {}/{}", config.build.output, filename));
        if warning_count == 0 {
            out.stdout("Build complete.");
        } else {
            out.stdout(format!("Build complete with {} warning(s).", warning_count));
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
    let mut tokens = crate::themes::resolve_tokens(&program, &config.theme)?;
    crate::themes::apply_motion(&mut tokens, &config.motion);
    let mut css = generate_css_for(&tokens, config.theme.builtin, &program);
    if let Some(dark) = crate::themes::resolve_dark_tokens(&program, &config.theme)? {
        css.push_str(&dark_css(&dark));
    }
    css.push_str(&project_css);
    // What an inline style cannot say — pseudo-states, media queries — is
    // compiled into the sheet under content-named classes. With `build.split`
    // the rules only one page reaches go to that page's own sheet.
    let page_sheets = if config.build.split {
        let split = crate::codegen::scoped_css::split_rules(&program);
        css.push_str(&split.shared);
        split.pages
    } else {
        css.push_str(&crate::codegen::scoped_css::scoped_rules(&program));
        Default::default()
    };
    let mut js_codegen = JsCodegen::new();
    if let Some(i18n_config) = &config.i18n {
        js_codegen.set_i18n(i18n_config.default_locale.clone(), translations.clone());
    }
    if config.build.ssg {
        js_codegen.set_ssg(true);
    }
    js_codegen.set_split_pages(config.build.split);
    js_codegen.set_full_runtime(config.build.runtime == crate::config::RuntimeMode::Full);
    js_codegen.set_env(config.public_env_values());
    if let Some(offline) = &config.offline {
        let problems = offline_problems(offline, &config, &program);
        if !problems.is_empty() {
            for problem in &problems {
                out.report.errors.push(BuildDiagnostic::about(
                    "webfluent.app.json",
                    problem.clone(),
                    problem.clone(),
                ));
            }
            return Err(WebFluentError::ConfigError(problems.join("\n")));
        }
        js_codegen.set_offline(offline.sync);
    }
    if !config.build.base_path.is_empty() {
        js_codegen.set_base_path(config.build.base_path.clone());
    }
    let js = js_codegen.generate(&program);

    // The project as custom elements: one file any framework can load, and
    // a page listing the tags it published. There are no routes, no shell
    // and no pages — a component is the whole of what is published.
    if config.build.output_type == OutputType::Elements {
        let problems = crate::codegen::elements::check(&program, &config);
        if !problems.is_empty() {
            for problem in &problems {
                out.error(BuildDiagnostic::about(
                    "webfluent.app.json",
                    problem.clone(),
                    format!("Error: {problem}"),
                ));
            }
            return Err(WebFluentError::CodegenError(format!(
                "{} problem(s) with `build.elements`",
                problems.len()
            )));
        }
        let bundle = format!(
            "{js}{}",
            crate::codegen::elements::definitions(&program, &config)
        );
        let base = config.build.base_path.trim_end_matches('/');
        let shrink = |text: String, js: bool| {
            if !config.build.minify {
                return text;
            }
            if js {
                crate::codegen::minify::minify_js(&text)
            } else {
                crate::codegen::minify::minify_css(&text)
            }
        };
        fs::write(output_dir.join("elements.js"), shrink(bundle, true))?;
        fs::write(output_dir.join("styles.css"), shrink(css.clone(), false))?;
        fs::write(
            output_dir.join("index.html"),
            crate::codegen::elements::demo_page(&program, &config, base),
        )?;
        let public_dir = project_dir.join("public");
        if vfs.exists(&public_dir) {
            copy_dir_recursive(vfs, &public_dir, &output_dir)?;
        }
        let tags: Vec<String> = config
            .build
            .elements
            .iter()
            .map(|n| format!("<{}>", crate::codegen::elements::tag_name(n)))
            .collect();
        out.stdout(format!("  Elements: {}", tags.join(" ")));
        out.stdout(format!("  Output: {}/elements.js", config.build.output));
        if warning_count == 0 {
            out.stdout("Build complete.");
        } else {
            out.stdout(format!("Build complete with {} warning(s).", warning_count));
        }
        return Ok(());
    }

    // Write output

    if config.build.ssg {
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
        let components: std::collections::HashMap<String, crate::parser::ast::ComponentDecl> =
            program
                .declarations
                .iter()
                .filter_map(|d| match d {
                    Declaration::Component(c) => Some((c.name.clone(), c.clone())),
                    _ => None,
                })
                .collect();

        for decl in &program.declarations {
            if let Declaration::Page(page) = decl {
                let site = crate::codegen::ssg::SiteContext {
                    config: &config,
                    app_body: app_stmts,
                    translations: &translations,
                    components: &components,
                    program: &program,
                };
                // A `:param` route is rendered once per value its `paths:`
                // names, and not at all without them.
                if page.path.contains(':') {
                    for (route, params) in
                        crate::codegen::ssg::static_routes(page, &program, &config.env)?
                    {
                        let page_html = crate::codegen::ssg::render_page_html_with_params(
                            page, &site, &route, &params,
                        );
                        let dir = output_dir.join(route.trim_start_matches('/'));
                        fs::create_dir_all(&dir)?;
                        fs::write(dir.join("index.html"), &page_html)?;
                        written_html.push(dir.join("index.html"));
                    }
                    continue;
                }
                let page_html = crate::codegen::render_page_html(page, &site);

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
                    let dir = output_dir.join(route);
                    fs::create_dir_all(&dir)?;
                    fs::write(dir.join("index.html"), &page_html)?;
                    written_html.push(dir.join("index.html"));
                }
            }
        }
        out.stdout("  SSG: pre-rendered static pages");
    } else {
        // SPA: single index.html
        let html = generate_html(&config, &program);
        fs::write(output_dir.join("index.html"), html)?;
        written_html.push(output_dir.join("index.html"));
    }

    // The modules a program's `external` declarations import, in a file of
    // their own — a module, so `import` works, and separate so the page
    // chunks keep reading the bundle's names from the global scope.
    if let Some(module) = crate::codegen::js::externals_module(&program) {
        fs::write(output_dir.join("externals.js"), module)?;
    }

    // `build.minify` — on by default, and read for the first time here: the
    // bundle and the sheet lose their comments and whitespace, and nothing
    // else, so a stack trace still reads as the compiler wrote it.
    let minify_js = |src: String| {
        if config.build.minify {
            crate::codegen::minify::minify_js(&src)
        } else {
            src
        }
    };
    let minify_css = |src: String| {
        if config.build.minify {
            crate::codegen::minify::minify_css(&src)
        } else {
            src
        }
    };
    fs::write(output_dir.join("styles.css"), minify_css(css))?;
    fs::write(output_dir.join("app.js"), minify_js(js))?;

    // Each page in its own chunk, fetched when its route shows, and its own
    // sheet beside it when it has rules no other page reaches.
    let chunks = js_codegen.take_chunks();
    if !chunks.is_empty() || !page_sheets.is_empty() {
        let pages_dir = output_dir.join("pages");
        fs::create_dir_all(&pages_dir)?;
        for (name, source) in chunks {
            fs::write(pages_dir.join(format!("{name}.js")), minify_js(source))?;
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
            out.stdout(format!(
                "  Note: `style-src` allows inline styles, because {} page(s) carry one:",
                loose.len()
            ));
            for page in loose.iter().take(5) {
                out.stdout(format!("    {page}"));
            }
            if loose.len() > 5 {
                out.stdout(format!("    … and {} more", loose.len() - 5));
            }
            out.stdout(
                "    A `style { }` value that reads state is set on the element; a literal is a class.",
            );
        }
        fs::write(output_dir.join("_headers"), headers_file(&config))?;
        // A policy is a promise about what the pages contain. Reading them
        // back is how the promise is kept: a browser would enforce it and
        // show a blank page, and this says so at build time instead.
        let broken = check_csp(&output_dir, &written_html, &config);
        if !broken.is_empty() {
            let mut message = format!(
                "the Content-Security-Policy this build ships forbids {} thing(s) in its own output:\n",
                broken.len()
            );
            for violation in &broken {
                message.push_str(&format!("  {violation}\n"));
                out.report.errors.push(BuildDiagnostic {
                    hint: Some(violation.hint.clone()),
                    ..BuildDiagnostic::about("", violation.to_string(), format!("  {violation}"))
                });
            }
            return Err(crate::error::WebFluentError::CodegenError(message));
        }
    }

    // A crawler looks for both of these at the site root. The engine already
    // knows every route, so a sitemap is something it can write rather than
    // something the author has to keep in step by hand.
    if config.meta.sitemap {
        let map = crate::codegen::seo::sitemap(&config, &program);
        if let Some(xml) = &map {
            fs::write(output_dir.join("sitemap.xml"), xml)?;
        }
        fs::write(
            output_dir.join("robots.txt"),
            crate::codegen::seo::robots_txt(&config, map.is_some()),
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
    if config.build.compress {
        precompress(&output_dir)?;
    }

    let (page_count, comp_count, store_count) = counts(&program);

    let locale_count = config.i18n.as_ref().map_or(0, |i| i.locales.len());
    if locale_count > 0 {
        out.stdout(format!(
            "  {} pages, {} components, {} stores, {} locales",
            page_count, comp_count, store_count, locale_count
        ));
    } else {
        out.stdout(format!(
            "  {} pages, {} components, {} stores",
            page_count, comp_count, store_count
        ));
    }
    out.stdout(format!("  Output: {}/", config.build.output));
    // What this build weighs, against the budget and against the last one.
    let sizes = gzipped_sizes(&output_dir)?;
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
    if warning_count == 0 {
        out.stdout("Build complete.");
    } else {
        out.stdout(format!("Build complete with {} warning(s).", warning_count));
    }

    Ok(())
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
            let path = entry?.path();
            if path.is_dir() {
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
fn gzipped_sizes(output_dir: &Path) -> Result<BTreeMap<String, usize>> {
    let mut sizes = BTreeMap::new();
    let mut walk = vec![output_dir.to_path_buf()];
    while let Some(dir) = walk.pop() {
        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
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
            let bytes = fs::read(&path)?;
            sizes.insert(name, crate::codegen::gzip::gzip(&bytes).len());
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
    out: &mut Out,
    budget: &BTreeMap<String, String>,
    sizes: &BTreeMap<String, usize>,
) -> usize {
    let mut over = 0;
    let mut warn = |message: String| {
        let text = format!("  Warning: {message}");
        out.warning(BuildDiagnostic::about("webfluent.app.json", message, text));
        over += 1;
    };
    for (name, limit) in budget {
        let Some(max) = parse_size(limit) else {
            warn(format!("budget for '{name}' is not a size: '{limit}'"));
            continue;
        };
        match sizes.get(name) {
            Some(bytes) if *bytes > max => {
                warn(format!(
                    "{name} is {:.1} kB gzipped, over the {:.1} kB budget by {:.1} kB",
                    *bytes as f64 / 1024.0,
                    max as f64 / 1024.0,
                    (*bytes - max) as f64 / 1024.0
                ));
            }
            Some(_) => {}
            None => {
                warn(format!(
                    "budget names '{name}', which this build does not write"
                ));
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
    out: &mut Out,
    output_dir: &Path,
    modules: &[crate::runtime::Kept],
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
            let path = entry?.path();
            if path.is_dir() {
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
            let bytes = fs::metadata(&path)?.len() as usize;
            let gz = fs::metadata(path.with_extension(format!("{ext}.gz")))
                .map(|m| m.len() as usize)
                .unwrap_or(0);
            files.push((name.to_string(), bytes, gz));
        }
    }
    files.sort_by_key(|f| std::cmp::Reverse(f.1));
    let total: usize = files.iter().map(|f| f.1).sum();
    out.stdout("\n  What it weighs");
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
        out.stdout(format!(
            "    {:<34} {:>9}  ({} gzipped){}",
            name,
            kb(*bytes),
            kb(gz),
            delta
        ));
    }
    if files.len() > 20 {
        out.stdout(format!("    … and {} more", files.len() - 20));
    }
    out.stdout(format!("    {:<34} {:>9}", "total", kb(total)));

    let names: Vec<&str> = modules.iter().map(|m| m.name).collect();
    let left_out = crate::runtime::dropped(&names);
    let all = crate::runtime::full().len();
    out.stdout(format!(
        "\n  Runtime: {} of {} modules, {} of {} (before minifying)",
        modules.len(),
        modules.len() + left_out.len(),
        kb(runtime_bytes),
        kb(all)
    ));
    let mut by_size: Vec<&crate::runtime::Kept> = modules.iter().collect();
    by_size.sort_by_key(|m| std::cmp::Reverse(m.bytes));
    for m in by_size {
        out.stdout(format!(
            "    {:<12} {:>8}  {}",
            m.name,
            kb(m.bytes),
            m.reason
        ));
    }
    if left_out.is_empty() {
        out.stdout("    left out: nothing");
    } else {
        out.stdout(format!("    left out: {}", left_out.join(" ")));
    }
    if tokens_dropped > 0 {
        out.stdout(format!(
            "\n  Tokens: {tokens_dropped} nothing in the output names, left out"
        ));
    }
    Ok(())
}

/// Every source file under the project's `src/`, parsed and merged into
/// one program, with the file each declaration came from (relative to the
/// project) so a diagnostic over the merged program can still name a file
/// the reader can open.
pub fn read_project(project_dir: &Path) -> Result<(Program, Vec<String>)> {
    read_project_with(project_dir, None)
}

/// [`read_project`], with somewhere to write the images the program names.
///
/// A reader that writes no files — `wf types`, the language server — passes
/// nothing, and an `image` resolves to the file as it stands.
pub fn read_project_with(
    project_dir: &Path,
    media: Option<(&Path, &crate::media::Settings, &str)>,
) -> Result<(Program, Vec<String>)> {
    read_project_via(&FsVfs, project_dir, media)
}

/// [`read_project_with`], reading every file through `vfs`.
pub fn read_project_via(
    vfs: &dyn Vfs,
    project_dir: &Path,
    media: Option<(&Path, &crate::media::Settings, &str)>,
) -> Result<(Program, Vec<String>)> {
    let src_dir = project_dir.join("src");
    if !vfs.exists(&src_dir) {
        return Err(WebFluentError::IoError(
            "src/ directory not found".to_string(),
        ));
    }
    let wf_files = find_wf_files(vfs, &src_dir)?;
    if wf_files.is_empty() {
        return Err(WebFluentError::IoError(
            "No .wf files found in src/".to_string(),
        ));
    }
    let mut all_declarations = Vec::new();
    let mut declaration_files: Vec<String> = Vec::new();
    for file_path in &wf_files {
        let source = vfs.read_to_string(file_path)?;
        let relative = file_path.strip_prefix(project_dir).unwrap_or(file_path);
        let file_name = relative.to_string_lossy().to_string();
        let program = if file_name.ends_with(".md") {
            crate::data::markdown_page(&source, &file_name)?
        } else {
            crate::syntax::parse_source(&source, &file_name)?
        };
        declaration_files.extend(program.declarations.iter().map(|_| file_name.clone()));
        all_declarations.extend(program.declarations);
    }
    let mut program = Program {
        declarations: all_declarations,
    };
    // `data x = "file.json"` is a constant once the file is read, and
    // `image hero = "…"` once the picture is.
    crate::data::resolve_data_via(vfs, &mut program, project_dir, media)?;
    // `api B from "openapi.json"` is its endpoints once the spec is read,
    // so every reader of a project sees the same service.
    crate::openapi::expand(&mut program, &|file| {
        for at in [project_dir.join(file), project_dir.join("src").join(file)] {
            if let Ok(text) = vfs.read_to_string(&at) {
                return Some(text);
            }
        }
        None
    })?;
    Ok((program, declaration_files))
}

fn find_wf_files(vfs: &dyn Vfs, dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !vfs.is_dir(dir) {
        return Ok(files);
    }

    // Process App.wf (or App.wfx) first if it exists (so App declaration comes first)
    let app_file = ["App.wf", "App.wfx"]
        .iter()
        .map(|n| dir.join(n))
        .find(|p| vfs.exists(p));
    if let Some(app) = &app_file {
        files.push(app.clone());
    }

    // In name order, so a build is the same wherever the project sits:
    // the directory's own order is the file system's, and a page's
    // element numbering follows the order the files are read in.
    let mut entries: Vec<PathBuf> = vfs.read_dir(dir)?;
    entries.sort();
    for path in entries {
        if vfs.is_dir(&path) {
            files.extend(find_wf_files(vfs, &path)?);
        } else if crate::syntax::is_source_file(&path)
            || path.extension().is_some_and(|e| e == "md")
        {
            // Skip App.wf since we already added it
            if app_file.as_ref() == Some(&path) {
                continue;
            }
            files.push(path);
        }
    }

    Ok(files)
}

fn load_translations(
    vfs: &dyn Vfs,
    out: &mut Out,
    project_dir: &Path,
    i18n_config: &crate::config::project::I18nConfig,
) -> Result<HashMap<String, HashMap<String, String>>> {
    let mut translations = HashMap::new();
    let trans_dir = project_dir.join(&i18n_config.dir);
    // Printed to stdout, as it always was, and kept as a warning.
    let mut warn = |file: String, message: String| {
        out.stdout(format!("  Warning: {message}"));
        out.report.warnings.push(BuildDiagnostic::about(
            &file,
            message.clone(),
            format!("  Warning: {message}"),
        ));
    };

    if !vfs.exists(&trans_dir) {
        warn(
            i18n_config.dir.clone(),
            format!("translations directory '{}' not found", i18n_config.dir),
        );
        return Ok(translations);
    }

    for locale in &i18n_config.locales {
        let file_path = trans_dir.join(format!("{}.json", locale));
        if !vfs.exists(&file_path) {
            warn(
                i18n_config.dir.clone(),
                format!("translation file '{}.json' not found", locale),
            );
            continue;
        }

        let content = vfs.read_to_string(&file_path)?;
        let messages: HashMap<String, String> = serde_json::from_str(&content).map_err(|e| {
            WebFluentError::ConfigError(format!("Failed to parse {}.json: {}", locale, e))
        })?;

        translations.insert(locale.clone(), messages);
    }

    Ok(translations)
}

/// Write `<file>.gz` beside every text file under `dir` that is worth it —
/// a file a few hundred bytes long fits in one packet either way, and one
/// gzip does not shrink is left alone. A stale `.gz` from an earlier build
/// whose source no longer exists is removed, so a host never serves it.
fn precompress(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            precompress(&path)?;
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if ext == "gz" {
            if !path.with_extension("").exists() {
                fs::remove_file(&path)?;
            }
            continue;
        }
        if !crate::codegen::gzip::is_text_extension(ext) {
            continue;
        }
        let data = fs::read(&path)?;
        let gz_path = PathBuf::from(format!("{}.gz", path.display()));
        if data.len() < 1024 {
            let _ = fs::remove_file(&gz_path);
            continue;
        }
        let gz = crate::codegen::gzip::gzip(&data);
        if gz.len() < data.len() {
            fs::write(&gz_path, gz)?;
        } else {
            let _ = fs::remove_file(&gz_path);
        }
    }
    Ok(())
}

/// Copy the directory `src` of `vfs` into `dst` on the disk.
fn copy_dir_recursive(vfs: &dyn Vfs, src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
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
) -> Vec<crate::codegen::csp::Violation> {
    let policy = crate::config::project::csp_meta_policy(config);
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
        found.extend(crate::codegen::csp::check(&page, &html, &policy));
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
         \x20 X-Frame-Options: DENY\n",
        crate::config::project::csp_policy(config)
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

/// What `offline` in the config asks for that it cannot mean, one problem
/// a line.
fn offline_problems(
    offline: &crate::config::OfflineConfig,
    config: &ProjectConfig,
    program: &crate::parser::ast::Program,
) -> Vec<String> {
    let mut problems = Vec::new();
    if !matches!(config.build.output_type, OutputType::Spa) {
        problems.push("`offline` is for a web build; this one writes a document".to_string());
    }
    for (path, strategy) in &offline.cache {
        if !crate::config::OFFLINE_STRATEGIES.contains(&strategy.as_str()) {
            problems.push(format!(
                "`offline.cache` asks for `{strategy}` on `{path}`; the policies are {}",
                crate::config::OFFLINE_STRATEGIES
                    .iter()
                    .map(|s| format!("`{s}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    if let Some(fallback) = &offline.fallback {
        let known = program
            .declarations
            .iter()
            .any(|d| matches!(d, crate::parser::ast::Declaration::Page(p) if &p.path == fallback));
        if !known {
            problems.push(format!(
                "`offline.fallback` is `{fallback}`, which no page's `path` is; \
                 declare `page Offline(path: \"{fallback}\")` or name a page that exists"
            ));
        }
    }
    problems
}

/// The page a route a static build wrote belongs to: its own `path`, or the
/// `:param` pattern it was rendered from.
fn page_for_route<'a>(
    program: &'a crate::parser::ast::Program,
    route: &str,
) -> Option<&'a crate::parser::ast::PageDecl> {
    let segments: Vec<&str> = route.split('/').filter(|s| !s.is_empty()).collect();
    program.declarations.iter().find_map(|d| {
        let crate::parser::ast::Declaration::Page(page) = d else {
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
    out: &mut Out,
    output_dir: &Path,
    config: &ProjectConfig,
    offline: &crate::config::OfflineConfig,
    program: &crate::parser::ast::Program,
    written_html: &[PathBuf],
) -> Result<()> {
    use crate::codegen::offline::{Worker, route_matches, service_worker, version_of};
    let base = config.build.base_path.trim_end_matches('/').to_string();
    let site = |rel: &str| format!("{base}/{}", rel.trim_start_matches('/'));
    let exists = |rel: &str| output_dir.join(rel).is_file();

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
    let chunks_of = |page: &crate::parser::ast::PageDecl, list: &mut Vec<String>| {
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
    if config.build.ssg {
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
            let crate::parser::ast::Declaration::Page(page) = d else {
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
        let message = format!(
            "`offline.precache` ({}) names no route this build writes; only the shell is stored",
            offline.precache.join(", ")
        );
        out.stdout(format!("  Warning: {message}"));
        out.report.warnings.push(BuildDiagnostic::about(
            "webfluent.app.json",
            message.clone(),
            format!("  Warning: {message}"),
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
    out.stdout(format!(
        "  Offline: sw.js, {} file(s) stored, version {}",
        worker.precache.len(),
        &version[..8]
    ));
    Ok(())
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
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
