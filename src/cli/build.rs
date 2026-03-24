use std::fs;
use std::path::{Path, PathBuf};
use crate::config::ProjectConfig;
use crate::lexer::Lexer;
use crate::parser::{Parser, Program, Declaration};
use crate::codegen::{generate_html, generate_css, JsCodegen};
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

    // Generate output
    let html = generate_html(&config);
    let css = generate_css(&config.theme.name, &config.theme.tokens);
    let mut js_codegen = JsCodegen::new();
    let js = js_codegen.generate(&program);

    // Write output
    let output_dir = project_dir.join(&config.build.output);
    fs::create_dir_all(&output_dir)?;

    fs::write(output_dir.join("index.html"), html)?;
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

    println!("  {} pages, {} components, {} stores", page_count, comp_count, store_count);
    println!("  Output: {}/", config.build.output);
    println!("Build complete.");

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
