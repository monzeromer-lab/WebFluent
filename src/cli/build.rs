use crate::codegen::{
    JsCodegen, dark_css, generate_css_for, generate_html,
};
use crate::config::ProjectConfig;
use crate::config::project::OutputType;
use crate::diagnostics::format::Format;
use crate::error::{Result, WebFluentError};
use crate::parser::{Declaration, Program, Statement};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_build(project_dir: &Path) -> Result<()> {
    run_build_with(project_dir, false)
}

/// Build, and with `stats` print what the output weighs and which runtime
/// modules it carries.
pub fn run_build_with(project_dir: &Path, stats: bool) -> Result<()> {
    run_build_opts(
        project_dir,
        Options {
            stats,
            ..Options::default()
        },
    )
}

/// How a build, or a check, reports.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// Print what the output weighs.
    pub stats: bool,
    /// How the findings are written: for a person, or for a tool.
    pub format: Format,
    /// A warning stops the build as an error does — for CI.
    pub deny_warnings: bool,
    /// A build for `wf serve`: responses are held to their declared types.
    pub dev: bool,
}

thread_local! {
    /// The format of the build running on this thread. `wf serve` builds on
    /// its own threads, always for a person.
    static FORMAT: std::cell::Cell<Format> = const { std::cell::Cell::new(Format::Human) };
    /// What a JSON or SARIF build has found so far: written once, as one
    /// document, when it ends.
    static FOUND: std::cell::RefCell<Vec<crate::error::Diagnostic>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn format() -> Format {
    FORMAT.with(|f| f.get())
}

/// A line of the build's progress: on standard output for a person, and on
/// standard error when standard output carries findings for a tool.
macro_rules! say {
    ($($t:tt)*) => {
        if format() == Format::Human {
            println!($($t)*)
        } else {
            eprintln!($($t)*)
        }
    };
}

/// Build with `options`. With a format for a tool, the findings are written
/// to standard output — once, as one document for JSON and SARIF, whether
/// the build finished or stopped.
pub fn run_build_opts(project_dir: &Path, options: Options) -> Result<()> {
    with_format(options.format, || build(project_dir, options))
}

/// `wf check`: every finding of a build, with nothing written.
pub fn run_check(project_dir: &Path, options: Options) -> Result<()> {
    with_format(options.format, || {
        let mut config = ProjectConfig::load(project_dir)?;
        config.resolve_env(project_dir);
        let analysis = analyse(project_dir, &mut config, None)?;
        let diagnostics = analysis.diagnostics;
        report(project_dir, &diagnostics);
        stop_on(&diagnostics, options)?;
        if format() == Format::Human && diagnostics.is_empty() {
            println!("No problems in {}.", config.name);
        }
        Ok(())
    })
}

fn with_format(format: Format, run: impl FnOnce() -> Result<()>) -> Result<()> {
    FORMAT.with(|f| f.set(format));
    FOUND.with(|f| f.borrow_mut().clear());
    let result = run();
    let found = FOUND.with(|f| std::mem::take(&mut *f.borrow_mut()));
    match format {
        Format::Json => println!("{}", crate::diagnostics::format::json(&found)),
        Format::Sarif => println!("{}", crate::diagnostics::format::sarif(&found)),
        Format::Human | Format::Github => {}
    }
    FORMAT.with(|f| f.set(Format::Human));
    result
}

/// An error stops a build; with `--deny-warnings`, so does a warning.
fn stop_on(diagnostics: &[crate::error::Diagnostic], options: Options) -> Result<()> {
    let (errors, warnings) = crate::diagnostics::counts(diagnostics);
    if errors > 0 {
        return Err(WebFluentError::Diagnostics(diagnostics.to_vec()));
    }
    if options.deny_warnings && warnings > 0 {
        eprintln!("error: --deny-warnings makes each warning stop the build");
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
    pub scripts: Vec<crate::project_js::Script>,
    /// The project's own stylesheets, bundled.
    pub project_css: String,
    /// Every finding: the files that did not parse, then every check.
    pub diagnostics: Vec<crate::error::Diagnostic>,
}

/// Read and check a project, writing nothing but the images `media` names
/// (a build's output directory, its media settings and base path).
pub fn analyse(
    project_dir: &Path,
    config: &mut ProjectConfig,
    media: Option<(&Path, &crate::media::Settings, &str)>,
) -> Result<Analysis> {
    let (program, declaration_files, parse_errors) = read_project_collecting(project_dir, media)?;
    // What the pages will contain, decided before any of them is written:
    // the policy each one ships has to describe that page.
    config.build.inline_styles = crate::codegen::csp::writes_inline_styles(&program);
    // The project's own scripts: every `.js` under `src/`, a plain browser
    // script each, copied as written and linked before the compiled code.
    let scripts = crate::project_js::load(project_dir, &project_dir.join("src"))?;
    config.build.scripts = scripts.iter().map(|s| s.href.clone()).collect();
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
            .and_then(|f| fs::read_to_string(project_dir.join(f)).ok())
    };
    // The author's own stylesheets — every `.css` under `src/` — ship in
    // `styles.css`, and a modifier class one of them defines is a real one.
    let project_css = crate::codegen::project_css::bundle(project_dir, &project_dir.join("src"))?;

    // Every check, over everything that parsed: the scripts and the config,
    // the structure, the registry's vocabulary, the types, the lints, and
    // what this output can draw. A stage's errors no longer stop the next.
    let checked = crate::diagnostics::check::check_project(&crate::diagnostics::check::Project {
        program: &program,
        file_of: &file_of,
        source_of: &source_of,
        dir: Some(project_dir),
        config: Some(config),
        declaration_files: &declaration_files,
        scripts: &scripts,
        stylesheets: &project_css,
        incomplete: !parse_errors.is_empty(),
    });
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

fn build(project_dir: &Path, options: Options) -> Result<()> {
    let stats = options.stats;
    let mut config = ProjectConfig::load(project_dir)?;
    // `env` from a `.env` file and the shell, on top of the config's own.
    config.resolve_env(project_dir);

    say!("Building {}...", config.name);

    // The images the program names are written at every width a page will
    // ask for, before anything checks the program — so a name resolves to
    // the asset it became, with its real size and colour.
    let output_dir = project_dir.join(&config.build.output);
    fs::create_dir_all(&output_dir)?;
    let media = config.build.media.settings();
    let base_path = config.build.base_path.clone();
    let Analysis {
        program,
        lowered,
        scripts,
        project_css,
        diagnostics,
        ..
    } = analyse(
        project_dir,
        &mut config,
        Some((&output_dir, &media, &base_path)),
    )?;
    // Every HTML file this build writes. The policy check speaks for what
    // this build put in the output, not for whatever else is in the
    // directory — a file an older layout left behind is a real problem,
    // but not one this build can answer for.
    let mut written_html: Vec<PathBuf> = Vec::new();
    let config = config;
    report(project_dir, &diagnostics);
    stop_on(&diagnostics, options)?;
    let mut warning_count = diagnostics.len();
    // What each `persist` holds now, for the next build to compare with.
    let shapes = crate::linter::project::persist_shapes(&program);
    if !shapes.is_empty() {
        let cache = project_dir.join(crate::linter::project::PERSIST_CACHE);
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
    crate::linter::project::keep_persisted_values(project_dir, &program, &config.env);
    let program = lowered;

    // A PDF document or a slide deck: the page's markup and the site's
    // stylesheet, laid out on paper by the paged engine — so the document
    // draws what the page draws.
    if matches!(config.build.output_type, OutputType::Pdf | OutputType::Slides) {
        let deck = config.build.output_type == OutputType::Slides;
        let mut tokens = crate::themes::resolve_tokens(&program, &config.theme)?;
        crate::themes::apply_motion(&mut tokens, &config.motion);
        let mut css = generate_css_for(&tokens, config.theme.builtin, &program);
        css.push_str(&project_css);
        css.push_str(&crate::codegen::scoped_css::scoped_rules(&program));
        let fragment = crate::template::render_program_paged(&program, &serde_json::json!({}))?;
        let lang = if config.meta.lang.is_empty() { "en".to_string() } else { config.meta.lang.clone() };
        let title = crate::codegen::markdown::escape(&config.name);
        let dir = if crate::template::is_rtl(&lang) { " dir=\"rtl\"" } else { "" };
        let html = format!(
            "<!DOCTYPE html><html lang=\"{lang}\"{dir}><head><title>{title}</title><style>{css}</style></head><body>{fragment}</body></html>"
        );
        // A picture is read from the project, or from what this build wrote
        // (an `image` the media pipeline sized).
        let project_read = crate::paged::reader(Some(project_dir.to_path_buf()));
        let built = output_dir.clone();
        let read = move |src: &str| {
            project_read(src).or_else(|| {
                let rel = src.split(['?', '#']).next().unwrap_or(src).trim_start_matches('/');
                fs::read(built.join(rel)).ok()
            })
        };
        let rendered = if deck {
            let (options, chrome) = crate::paged::Options::from_slides(&config.build.slides, &read, Some(project_dir));
            crate::paged::render_slides(&html, "", &options, &chrome)
        } else {
            let options = crate::paged::Options::from_pdf(&config.build.pdf, &read, Some(project_dir));
            crate::paged::render(&html, "", &options)
        }
        .map_err(WebFluentError::CodegenError)?;

        let filename = if deck {
            config.build.slides.output_filename.clone()
        } else {
            config.build.pdf.output_filename.clone()
        }
        .unwrap_or_else(|| format!("{}.pdf", config.name));
        fs::write(output_dir.join(&filename), &rendered.bytes)?;
        for note in &rendered.notes {
            say!("  note: {note}");
        }
        say!(
            "  {}: {} bytes, {} {}",
            if deck { "Slides" } else { "PDF" },
            rendered.bytes.len(),
            rendered.pages,
            if deck { "slide(s)" } else { "page(s)" }
        );
        say!("  Output: {}/{}", config.build.output, filename);
        if warning_count == 0 {
            say!("Build complete.");
        } else {
            say!("Build complete with {} warning(s).", warning_count);
        }
        return Ok(());
    }

    // Load translations if i18n is configured
    let translations = if let Some(i18n_config) = &config.i18n {
        load_translations(project_dir, i18n_config)?
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
    js_codegen.set_dev(options.dev);
    js_codegen.set_split_pages(config.build.split);
    js_codegen.set_full_runtime(config.build.runtime == crate::config::RuntimeMode::Full);
    js_codegen.set_env(config.public_env_values());
    if let Some(offline) = &config.offline {
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
        read_back("elements.js", &bundle, false)?;
        fs::write(output_dir.join("elements.js"), shrink(bundle, true))?;
        fs::write(output_dir.join("styles.css"), shrink(css.clone(), false))?;
        fs::write(
            output_dir.join("index.html"),
            crate::codegen::elements::demo_page(&program, &config, base),
        )?;
        let public_dir = project_dir.join("public");
        if public_dir.exists() {
            copy_dir_recursive(&public_dir, &output_dir)?;
        }
        let tags: Vec<String> = config
            .build
            .elements
            .iter()
            .map(|n| format!("<{}>", crate::codegen::elements::tag_name(n)))
            .collect();
        say!("  Elements: {}", tags.join(" "));
        say!("  Output: {}/elements.js", config.build.output);
        if warning_count == 0 {
            say!("Build complete.");
        } else {
            say!("Build complete with {} warning(s).", warning_count);
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
        say!("  SSG: pre-rendered static pages");
    } else {
        // SPA: single index.html
        let html = generate_html(&config, &program);
        fs::write(output_dir.join("index.html"), html)?;
        written_html.push(output_dir.join("index.html"));
    }

    // The modules a program's `external` declarations import, in a file of
    // their own — a module, so `import` works, and separate so the page
    // chunks keep reading the bundle's names from the global scope.
    if let Some(module) = crate::codegen::js::externals_module(&config) {
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
    let minify_js = |name: &str, src: String| -> Result<String> {
        read_back(name, &src, false)?;
        if config.build.minify {
            let small = crate::codegen::minify::minify_js(&src);
            read_back(name, &small, true)?;
            Ok(small)
        } else {
            Ok(src)
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
            say!(
                "  Note: `style-src` allows inline styles, because {} page(s) carry one:",
                loose.len()
            );
            for page in loose.iter().take(5) {
                say!("    {page}");
            }
            if loose.len() > 5 {
                say!("    … and {} more", loose.len() - 5);
            }
            say!(
                "    A `style {{ }}` value that reads state is set on the element; a literal is a class."
            );
        }
        fs::write(output_dir.join("_headers"), headers_file(&config))?;
        // A policy is a promise about what the pages contain. Reading them
        // back is how the promise is kept: a browser would enforce it and
        // show a blank page, and this says so at build time instead.
        let broken: Vec<crate::error::Diagnostic> = check_csp(&output_dir, &written_html, &config)
            .into_iter()
            .map(|v| {
                crate::error::Diagnostic::coded(
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
            report(project_dir, &broken);
            return Err(WebFluentError::Diagnostics(broken));
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
    if public_dir.exists() {
        copy_dir_recursive(&public_dir, &output_dir)?;
    }

    // The token block last, once everything that could name a token is on
    // disk — including whatever `public/` brought with it.
    let tokens_dropped = prune_root_tokens(&output_dir)?;

    // The service worker last of all, since its version is a hash of
    // everything else the build wrote.
    if let Some(offline) = &config.offline {
        write_service_worker(&output_dir, &config, offline, &program, &written_html)?;
    }

    // A `.gz` beside every text file, for a host that serves one when it has
    // it: compressed once here, harder than a server can afford per request.
    if config.build.compress {
        precompress(&output_dir)?;
    }

    let page_count = program
        .declarations
        .iter()
        .filter(|d| matches!(d, Declaration::Page(_)))
        .count();
    let comp_count = program
        .declarations
        .iter()
        .filter(|d| matches!(d, Declaration::Component(_)))
        .count();
    let store_count = program
        .declarations
        .iter()
        .filter(|d| matches!(d, Declaration::Store(_)))
        .count();

    let locale_count = config.i18n.as_ref().map_or(0, |i| i.locales.len());
    if locale_count > 0 {
        say!(
            "  {} pages, {} components, {} stores, {} locales",
            page_count,
            comp_count,
            store_count,
            locale_count
        );
    } else {
        say!(
            "  {} pages, {} components, {} stores",
            page_count,
            comp_count,
            store_count
        );
    }
    say!("  Output: {}/", config.build.output);
    // What this build weighs, against the budget and against the last one.
    let sizes = gzipped_sizes(&output_dir)?;
    let previous = read_sizes(project_dir);
    warning_count += report_budget(&config.build.budget, &sizes);
    write_sizes(project_dir, &sizes)?;

    if stats {
        let (_, runtime_bytes) = js_codegen.runtime_stats();
        print_stats(
            &output_dir,
            js_codegen.runtime_report(),
            runtime_bytes,
            tokens_dropped,
            &sizes,
            previous.as_ref(),
        )?;
    }
    if warning_count == 0 {
        say!("Build complete.");
    } else {
        say!("Build complete with {} warning(s).", warning_count);
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
fn read_sizes(project_dir: &Path) -> Option<BTreeMap<String, usize>> {
    let text = fs::read_to_string(project_dir.join(SIZES_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_sizes(project_dir: &Path, sizes: &BTreeMap<String, usize>) -> Result<()> {
    let text = serde_json::to_string_pretty(sizes).unwrap_or_default();
    fs::write(project_dir.join(SIZES_FILE), text)?;
    Ok(())
}

/// Warn about every output over its budget. Advisory: the build goes on.
fn report_budget(budget: &BTreeMap<String, String>, sizes: &BTreeMap<String, usize>) -> usize {
    let mut over = 0;
    for (name, limit) in budget {
        let Some(max) = parse_size(limit) else {
            eprintln!("  Warning: budget for '{name}' is not a size: '{limit}'");
            over += 1;
            continue;
        };
        match sizes.get(name) {
            Some(bytes) if *bytes > max => {
                eprintln!(
                    "  Warning: {name} is {:.1} kB gzipped, over the {:.1} kB budget by {:.1} kB",
                    *bytes as f64 / 1024.0,
                    max as f64 / 1024.0,
                    (*bytes - max) as f64 / 1024.0
                );
                over += 1;
            }
            Some(_) => {}
            None => {
                eprintln!("  Warning: budget names '{name}', which this build does not write");
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
    say!("\n  What it weighs");
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
        say!(
            "    {:<34} {:>9}  ({} gzipped){}",
            name,
            kb(*bytes),
            kb(gz),
            delta
        );
    }
    if files.len() > 20 {
        say!("    … and {} more", files.len() - 20);
    }
    say!("    {:<34} {:>9}", "total", kb(total));

    let names: Vec<&str> = modules.iter().map(|m| m.name).collect();
    let left_out = crate::runtime::dropped(&names);
    let all = crate::runtime::full().len();
    say!(
        "\n  Runtime: {} of {} modules, {} of {} (before minifying)",
        modules.len(),
        modules.len() + left_out.len(),
        kb(runtime_bytes),
        kb(all)
    );
    let mut by_size: Vec<&crate::runtime::Kept> = modules.iter().collect();
    by_size.sort_by_key(|m| std::cmp::Reverse(m.bytes));
    for m in by_size {
        say!("    {:<12} {:>8}  {}", m.name, kb(m.bytes), m.reason);
    }
    if left_out.is_empty() {
        say!("    left out: nothing");
    } else {
        say!("    left out: {}", left_out.join(" "));
    }
    if tokens_dropped > 0 {
        say!("\n  Tokens: {tokens_dropped} nothing in the output names, left out");
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
    let (program, files, mut errors) = read_project_collecting(project_dir, media)?;
    match errors.len() {
        0 => Ok((program, files)),
        1 => Err(WebFluentError::ParseError(Box::new(errors.remove(0)))),
        _ => Err(WebFluentError::Diagnostics(errors)),
    }
}

/// [`read_project_with`], keeping going past a file that does not parse:
/// the program of every declaration that does, the file of each, and a
/// diagnostic for each mistake in the files that do not — so one typo in
/// one file does not hide every finding in the others. An error is only a
/// file that cannot be read at all.
pub fn read_project_collecting(
    project_dir: &Path,
    media: Option<(&Path, &crate::media::Settings, &str)>,
) -> Result<(Program, Vec<String>, Vec<crate::error::Diagnostic>)> {
    let mut parse_errors: Vec<crate::error::Diagnostic> = Vec::new();
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        return Err(WebFluentError::IoError(
            "src/ directory not found".to_string(),
        ));
    }
    let wf_files = find_wf_files(&src_dir)?;
    if wf_files.is_empty() {
        return Err(WebFluentError::IoError(
            "No .wf files found in src/".to_string(),
        ));
    }
    let mut all_declarations = Vec::new();
    let mut declaration_files: Vec<String> = Vec::new();
    for file_path in &wf_files {
        let source = fs::read_to_string(file_path)?;
        let relative = file_path.strip_prefix(project_dir).unwrap_or(file_path);
        let file_name = relative.to_string_lossy().to_string();
        let program = if file_name.ends_with(".md") {
            match crate::data::markdown_page(&source, &file_name) {
                Ok(program) => program,
                Err(e) if !e.diagnostics().is_empty() => {
                    parse_errors.extend(e.diagnostics());
                    continue;
                }
                Err(e) => return Err(e),
            }
        } else {
            let (program, errors) = crate::syntax::parse_source_recovering(&source, &file_name);
            parse_errors.extend(errors);
            program
        };
        declaration_files.extend(program.declarations.iter().map(|_| file_name.clone()));
        all_declarations.extend(program.declarations);
    }
    // The names the project's scripts make global, one declaration per
    // file, so the checker, the codegen and the editor read the same ones.
    for script in crate::project_js::load(project_dir, &src_dir)? {
        if script.scan.module_at.is_some() || !script.scan.problems.is_empty() {
            continue;
        }
        declaration_files.push(script.path.clone());
        all_declarations.push(Declaration::Script(crate::parser::ast::ScriptDecl {
            path: script.path,
            names: script.scan.names,
        }));
    }
    // And what `meta.scripts` says a library defines: the compiler cannot
    // read a file on another origin, so each name is in scope as `Any`.
    if let Ok(config) = ProjectConfig::load(project_dir) {
        let names: Vec<crate::project_js::scan::Name> = config
            .meta
            .scripts
            .iter()
            .flat_map(|s| s.names())
            .map(|name| crate::project_js::scan::Name {
                name: name.to_string(),
                kind: crate::project_js::scan::NameKind::Value,
                line: 1,
                col: 1,
                doc: None,
            })
            .collect();
        if !names.is_empty() {
            declaration_files.push("webfluent.app.json".to_string());
            all_declarations.push(Declaration::Script(crate::parser::ast::ScriptDecl {
                path: "webfluent.app.json".to_string(),
                names,
            }));
        }
    }
    let mut program = Program {
        declarations: all_declarations,
    };
    // `data x = "file.json"` is a constant once the file is read, and
    // `image hero = "…"` once the picture is.
    if let Err(e) = crate::data::resolve_data_with(&mut program, project_dir, media) {
        if e.diagnostics().is_empty() {
            return Err(e);
        }
        parse_errors.extend(e.diagnostics());
    }
    // `api B from "openapi.json"` is its endpoints once the spec is read,
    // so every reader of a project sees the same service.
    if let Err(e) = crate::openapi::expand(&mut program, &|file| {
        for at in [project_dir.join(file), project_dir.join("src").join(file)] {
            if let Ok(text) = fs::read_to_string(&at) {
                return Some(text);
            }
        }
        None
    }) {
        if e.diagnostics().is_empty() {
            return Err(e);
        }
        parse_errors.extend(e.diagnostics());
    }
    Ok((program, declaration_files, parse_errors))
}

fn find_wf_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.is_dir() {
        return Ok(files);
    }

    // Process App.wf (or App.wfx) first if it exists (so App declaration comes first)
    let app_file = ["App.wf", "App.wfx"]
        .iter()
        .map(|n| dir.join(n))
        .find(|p| p.exists());
    if let Some(app) = &app_file {
        files.push(app.clone());
    }

    // In name order, so a build is the same wherever the project sits:
    // the directory's own order is the file system's, and a page's
    // element numbering follows the order the files are read in.
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::result::Result<_, _>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files.extend(find_wf_files(&path)?);
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
    project_dir: &Path,
    i18n_config: &crate::config::project::I18nConfig,
) -> Result<HashMap<String, HashMap<String, String>>> {
    let dir = project_dir.join(&i18n_config.dir);
    if !dir.exists() {
        say!(
            "  Warning: translations directory '{}' not found",
            i18n_config.dir
        );
    }
    crate::i18n::load(project_dir, i18n_config)
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

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
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
        say!(
            "  Warning: `offline.precache` ({}) names no route this build writes; only the shell is stored",
            offline.precache.join(", ")
        );
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
    say!(
        "  Offline: sw.js, {} file(s) stored, version {}",
        worker.precache.len(),
        &version[..8]
    );
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

/// The build reads back what it wrote: a compiled script the browser would
/// refuse stops the build, as the compiler's fault rather than the author's.
/// It used to ship, and the page died before it drew.
fn read_back(name: &str, source: &str, minified: bool) -> Result<()> {
    let Err(fault) = crate::codegen::jscheck::check_js(source) else {
        return Ok(());
    };
    let stage = if minified { ", after minifying" } else { "" };
    let d = crate::error::Diagnostic::coded(
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
        crate::codegen::jscheck::line_of(source, fault.line)
    ));
    report(Path::new("."), std::slice::from_ref(&d));
    Err(WebFluentError::Diagnostics(vec![d]))
}

/// Every finding, as the build's format says: rendered for a person on
/// standard error with the line that sums them up; kept for the one JSON or
/// SARIF document the build ends with; or as GitHub annotations on standard
/// output, beside the rendering.
fn report(project_dir: &Path, diagnostics: &[crate::error::Diagnostic]) {
    let format = format();
    if matches!(format, Format::Json | Format::Sarif) {
        FOUND.with(|f| f.borrow_mut().extend(diagnostics.iter().cloned()));
        if !diagnostics.is_empty() {
            eprintln!("{}", crate::diagnostics::summary(diagnostics));
        }
        return;
    }
    if diagnostics.is_empty() {
        return;
    }
    if format == Format::Github {
        print!("{}", crate::diagnostics::format::github(diagnostics));
    }
    let color = crate::diagnostics::render::stderr_wants_color();
    eprint!(
        "{}",
        crate::diagnostics::render::human(
            diagnostics,
            &|file| fs::read_to_string(project_dir.join(file)).ok(),
            color,
        )
    );
    let (errors, _) = crate::diagnostics::counts(diagnostics);
    let summary = crate::diagnostics::summary(diagnostics);
    if errors > 0 {
        eprintln!("error: the build stopped: {summary}");
    } else {
        eprintln!("{summary}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_script_the_browser_would_refuse_is_e901_the_compiler_s_fault() {
        assert!(
            read_back(
                "app.js",
                "const a = 1;\nfunction f() { return a; }\n",
                false
            )
            .is_ok()
        );
        let Err(WebFluentError::Diagnostics(found)) =
            read_back("app.js", "const a = 1;\nconst b = (2;\n", true)
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
