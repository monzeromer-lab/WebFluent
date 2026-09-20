use crate::codegen::{JsCodegen, PdfCodegen, SlidesCodegen, generate_css_for, generate_html};
use crate::config::ProjectConfig;
use crate::config::project::OutputType;
use crate::error::{Result, WebFluentError};
use crate::parser::{Declaration, Program, Statement};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_build(project_dir: &Path) -> Result<()> {
    let config = ProjectConfig::load(project_dir)?;

    println!("Building {}...", config.name);

    let (program, declaration_files) = read_project(project_dir)?;
    let file_of = |index: usize| {
        declaration_files
            .get(index)
            .cloned()
            .unwrap_or_else(|| "src/".to_string())
    };

    // A reference to nothing — an undeclared component, a route to a page that
    // does not exist, two pages with one name — is a broken site, not a style
    // question, so it stops the build the way a parse error does.
    let semantic = crate::linter::validate_semantics_in(&program, &file_of);
    if !semantic.is_empty() {
        for diagnostic in &semantic {
            eprintln!("{}", diagnostic);
        }
        return Err(WebFluentError::CodegenError(format!(
            "{} semantic error(s)",
            semantic.len()
        )));
    }

    // What the new grammar wrote, checked against what the components
    // declare: a flag, case, event, slot or part that resolves to nothing is
    // a broken site, and stops the build like a parse error.
    let mut findings = crate::sema::check(&program, &file_of);
    // Then the types: what every name is, and the values that do not fit.
    let typed = crate::sema::types::check(&program, &file_of);
    findings.errors.extend(typed.findings.errors);
    findings.warnings.extend(typed.findings.warnings);
    for warning in &findings.warnings {
        eprintln!("Warning: {}", warning);
    }
    if !findings.errors.is_empty() {
        for error in &findings.errors {
            eprintln!("{}", error);
        }
        return Err(WebFluentError::CodegenError(format!(
            "{} error(s)",
            findings.errors.len()
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
    for warning in &a11y_warnings {
        eprintln!("{}", warning);
    }
    // A bare word that resolves to nothing, or a real modifier with no rule
    // behind it, does nothing on screen. The LSP has reported these for a
    // while; a build from the command line said nothing.
    // The author's own stylesheets — every `.css` under `src/` — ship in
    // `styles.css`, and a modifier class one of them defines is a real one.
    let project_css = crate::codegen::project_css::bundle(project_dir, &project_dir.join("src"))?;
    let vocab_warnings = crate::linter::lint_vocabulary_with(&program, &project_css, &file_of);
    for warning in &vocab_warnings {
        eprintln!("{}", warning);
    }
    let warning_count = a11y_warnings.len() + vocab_warnings.len() + findings.warnings.len();

    // PDF output mode
    if config.build.output_type == OutputType::Pdf {
        // Validate: reject interactive elements
        let pdf_errors = crate::linter::validate_for_pdf(&program);
        if !pdf_errors.is_empty() {
            for err in &pdf_errors {
                eprintln!("{}", err);
            }
            return Err(WebFluentError::CodegenError(format!(
                "{} element(s) not allowed in PDF output",
                pdf_errors.len()
            )));
        }

        let mut pdf_codegen = PdfCodegen::new(&config.build.pdf);
        let pdf_bytes = pdf_codegen.generate(&program);

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
        println!("  PDF: {} bytes, {} page(s)", pdf_bytes.len(), page_count);
        println!("  Output: {}/{}", config.build.output, filename);
        if warning_count == 0 {
            println!("Build complete.");
        } else {
            println!("Build complete with {} warning(s).", warning_count);
        }
        return Ok(());
    }

    // Slides output mode (PDF deck)
    if config.build.output_type == OutputType::Slides {
        let slide_errors = crate::linter::validate_for_slides(&program);
        if !slide_errors.is_empty() {
            for err in &slide_errors {
                eprintln!("{}", err);
            }
            return Err(WebFluentError::CodegenError(format!(
                "{} slide validation error(s)",
                slide_errors.len()
            )));
        }

        let mut slides_codegen = SlidesCodegen::new(&config.build.slides);
        let pdf_bytes = slides_codegen.generate(&program);

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
        println!(
            "  Slides: {} bytes, {} slide(s)",
            pdf_bytes.len(),
            slide_count
        );
        println!("  Output: {}/{}", config.build.output, filename);
        if warning_count == 0 {
            println!("Build complete.");
        } else {
            println!("Build complete with {} warning(s).", warning_count);
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
    let tokens = crate::themes::resolve_tokens(&program, &config.theme)?;
    let mut css = generate_css_for(&tokens, config.theme.builtin, &program);
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
    if !config.build.base_path.is_empty() {
        js_codegen.set_base_path(config.build.base_path.clone());
    }
    let js = js_codegen.generate(&program);

    // Write output
    let output_dir = project_dir.join(&config.build.output);
    fs::create_dir_all(&output_dir)?;

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
                // Skip dynamic routes (contain :param)
                if page.path.contains(':') {
                    continue;
                }

                let site = crate::codegen::ssg::SiteContext {
                    config: &config,
                    app_body: app_stmts,
                    translations: &translations,
                    components: &components,
                    program: &program,
                };
                let page_html = crate::codegen::render_page_html(page, &site);

                // Determine output path
                let route = page.path.trim_start_matches('/');
                if route.is_empty() || route == "/" {
                    fs::write(output_dir.join("index.html"), &page_html)?;
                } else if route == "*" {
                    // The catch-all is what a static host serves for a path it
                    // has no file for — GitHub Pages, Netlify and Cloudflare
                    // Pages all look for 404.html at the root. It used to be
                    // written to a directory literally named `*`.
                    fs::write(output_dir.join("404.html"), &page_html)?;
                } else {
                    let dir = output_dir.join(route);
                    fs::create_dir_all(&dir)?;
                    fs::write(dir.join("index.html"), &page_html)?;
                }
            }
        }
        println!("  SSG: pre-rendered static pages");
    } else {
        // SPA: single index.html
        let html = generate_html(&config, &program);
        fs::write(output_dir.join("index.html"), html)?;
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
        fs::write(output_dir.join("_headers"), headers_file(&config))?;
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
        println!(
            "  {} pages, {} components, {} stores, {} locales",
            page_count, comp_count, store_count, locale_count
        );
    } else {
        println!(
            "  {} pages, {} components, {} stores",
            page_count, comp_count, store_count
        );
    }
    println!("  Output: {}/", config.build.output);
    if warning_count == 0 {
        println!("Build complete.");
    } else {
        println!("Build complete with {} warning(s).", warning_count);
    }

    Ok(())
}

/// Every source file under the project's `src/`, parsed and merged into
/// one program, with the file each declaration came from (relative to the
/// project) so a diagnostic over the merged program can still name a file
/// the reader can open.
pub fn read_project(project_dir: &Path) -> Result<(Program, Vec<String>)> {
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
        let program = crate::syntax::parse_source(&source, &file_name)?;
        declaration_files.extend(program.declarations.iter().map(|_| file_name.clone()));
        all_declarations.extend(program.declarations);
    }
    Ok((
        Program {
            declarations: all_declarations,
        },
        declaration_files,
    ))
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
        } else if crate::syntax::is_source_file(&path) {
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
    let mut translations = HashMap::new();
    let trans_dir = project_dir.join(&i18n_config.dir);

    if !trans_dir.exists() {
        println!(
            "  Warning: translations directory '{}' not found",
            i18n_config.dir
        );
        return Ok(translations);
    }

    for locale in &i18n_config.locales {
        let file_path = trans_dir.join(format!("{}.json", locale));
        if !file_path.exists() {
            println!("  Warning: translation file '{}.json' not found", locale);
            continue;
        }

        let content = fs::read_to_string(&file_path)?;
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

/// The `_headers` file a static host reads to set response headers.
///
/// `frame-ancestors` and `X-Content-Type-Options` cannot be set from a meta tag,
/// so a site that only carries the CSP in its HTML is still framable and still
/// subject to MIME sniffing.
fn headers_file(config: &ProjectConfig) -> String {
    format!(
        "/*\n\
         \x20 Content-Security-Policy: {}\n\
         \x20 X-Content-Type-Options: nosniff\n\
         \x20 Referrer-Policy: strict-origin-when-cross-origin\n\
         \x20 X-Frame-Options: DENY\n",
        crate::config::project::csp_policy(&config.meta)
    )
}
