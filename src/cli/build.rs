use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::config::ProjectConfig;
use crate::lexer::Lexer;
use crate::parser::{Parser, Program, Declaration, Statement};
use crate::codegen::{generate_html, generate_css, JsCodegen, PdfCodegen};
use crate::config::project::OutputType;
use crate::error::{WebFluentError, Result};

pub fn run_build(project_dir: &Path) -> Result<()> {
    let config = ProjectConfig::load(project_dir)?;

    println!("Building {}...", config.name);

    // Discover all .wf files
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        return Err(WebFluentError::IoError("src/ directory not found".to_string()));
    }

    let wf_files = find_wf_files(&src_dir)?;
    if wf_files.is_empty() {
        return Err(WebFluentError::IoError("No .wf files found in src/".to_string()));
    }

    // Lex and parse all files into a single program
    let mut all_declarations = Vec::new();

    for file_path in &wf_files {
        let source = fs::read_to_string(file_path)?;
        let relative = file_path.strip_prefix(project_dir).unwrap_or(file_path);
        let file_name = relative.to_string_lossy().to_string();

        let mut lexer = Lexer::new(&source, &file_name);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens, &file_name);
        let program = parser.parse()?;

        all_declarations.extend(program.declarations);
    }

    let program = Program {
        declarations: all_declarations,
    };

    // Run accessibility linter
    let a11y_warnings = crate::linter::lint_accessibility(&program);
    for warning in &a11y_warnings {
        eprintln!("{}", warning);
    }

    // PDF output mode
    if config.build.output_type == OutputType::Pdf {
        // Validate: reject interactive elements
        let pdf_errors = crate::linter::validate_for_pdf(&program);
        if !pdf_errors.is_empty() {
            for err in &pdf_errors {
                eprintln!("{}", err);
            }
            return Err(WebFluentError::CodegenError(
                format!("{} element(s) not allowed in PDF output", pdf_errors.len())
            ));
        }

        let mut pdf_codegen = PdfCodegen::new(&config.build.pdf);
        let pdf_bytes = pdf_codegen.generate(&program);

        let output_dir = project_dir.join(&config.build.output);
        fs::create_dir_all(&output_dir)?;

        let filename = config.build.pdf.output_filename
            .clone()
            .unwrap_or_else(|| format!("{}.pdf", config.name));
        fs::write(output_dir.join(&filename), &pdf_bytes)?;

        let page_count = pdf_codegen.page_count();
        println!("  PDF: {} bytes, {} page(s)", pdf_bytes.len(), page_count);
        println!("  Output: {}/{}", config.build.output, filename);
        if a11y_warnings.is_empty() {
            println!("Build complete.");
        } else {
            println!("Build complete with {} accessibility warning(s).", a11y_warnings.len());
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
    let css = generate_css(&config.theme.name, &config.theme.tokens);
    let mut js_codegen = JsCodegen::new();
    if let Some(i18n_config) = &config.i18n {
        js_codegen.set_i18n(i18n_config.default_locale.clone(), translations.clone());
    }
    if config.build.ssg {
        js_codegen.set_ssg(true);
    }
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
            if let Declaration::App(a) = d { Some(a.body.clone()) } else { None }
        });
        let app_stmts = app_body.as_deref();

        for decl in &program.declarations {
            if let Declaration::Page(page) = decl {
                // Skip dynamic routes (contain :param)
                if page.path.contains(':') {
                    continue;
                }

                let page_html = crate::codegen::render_page_html(
                    page, &config, app_stmts, &translations,
                );

                // Determine output path
                let route = page.path.trim_start_matches('/');
                if route.is_empty() || route == "/" {
                    fs::write(output_dir.join("index.html"), &page_html)?;
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
        let html = generate_html(&config);
        fs::write(output_dir.join("index.html"), html)?;
    }

    fs::write(output_dir.join("styles.css"), css)?;
    fs::write(output_dir.join("app.js"), js)?;

    // Copy public/ assets
    let public_dir = project_dir.join("public");
    if public_dir.exists() {
        copy_dir_recursive(&public_dir, &output_dir.join("public"))?;
    }

    let page_count = program.declarations.iter().filter(|d| matches!(d, Declaration::Page(_))).count();
    let comp_count = program.declarations.iter().filter(|d| matches!(d, Declaration::Component(_))).count();
    let store_count = program.declarations.iter().filter(|d| matches!(d, Declaration::Store(_))).count();

    let locale_count = config.i18n.as_ref().map_or(0, |i| i.locales.len());
    if locale_count > 0 {
        println!("  {} pages, {} components, {} stores, {} locales", page_count, comp_count, store_count, locale_count);
    } else {
        println!("  {} pages, {} components, {} stores", page_count, comp_count, store_count);
    }
    println!("  Output: {}/", config.build.output);
    if a11y_warnings.is_empty() {
        println!("Build complete.");
    } else {
        println!("Build complete with {} accessibility warning(s).", a11y_warnings.len());
    }

    Ok(())
}

fn find_wf_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.is_dir() {
        return Ok(files);
    }

    // Process App.wf first if it exists (so App declaration comes first)
    let app_file = dir.join("App.wf");
    if app_file.exists() {
        files.push(app_file);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(find_wf_files(&path)?);
        } else if path.extension().map_or(false, |ext| ext == "wf") {
            // Skip App.wf since we already added it
            if path.file_name().map_or(false, |n| n == "App.wf") && path.parent() == Some(dir) {
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
        println!("  Warning: translations directory '{}' not found", i18n_config.dir);
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
