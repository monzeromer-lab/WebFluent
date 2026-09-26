use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Write `content`, a braced source, as `<dir>/<stem>.wf` — or as a `.wfx`
/// in the indented layout when the project writes its sources that way
/// (it has `.wfx` files and no `.wf`).
fn write_source(dir: &Path, stem: &str, content: &str, what: &str) -> Result<Option<PathBuf>> {
    let (path, text) = if project_is_indented(dir) {
        let path = dir.join(format!("{stem}.wfx"));
        let text = crate::layout::to_offside(content, &path.to_string_lossy())?;
        (path, text)
    } else {
        (dir.join(format!("{stem}.wf")), content.to_string())
    };
    if path.exists() || path.with_extension("wf").exists() || path.with_extension("wfx").exists() {
        eprintln!("{what} {stem} already exists");
        return Ok(None);
    }
    fs::write(&path, text)?;
    Ok(Some(path))
}

/// Whether the `src/` around `dir` is written in the indented layout.
fn project_is_indented(dir: &Path) -> bool {
    let src = dir.parent().unwrap_or(dir);
    let mut has_wfx = false;
    let mut has_wf = false;
    fn walk(dir: &Path, has_wfx: &mut bool, has_wf: &mut bool) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, has_wfx, has_wf);
            } else if path.extension().is_some_and(|e| e == "wfx") {
                *has_wfx = true;
            } else if path.extension().is_some_and(|e| e == "wf") {
                *has_wf = true;
            }
        }
    }
    walk(src, &mut has_wfx, &mut has_wf);
    has_wfx && !has_wf
}

pub fn run_generate(kind: &str, name: &str, project_dir: &Path) -> Result<()> {
    match kind {
        "page" => generate_page(name, project_dir),
        "component" => generate_component(name, project_dir),
        "store" => generate_store(name, project_dir),
        _ => {
            eprintln!(
                "Unknown generator '{}'. Use: page, component, or store",
                kind
            );
            Ok(())
        }
    }
}

fn generate_page(name: &str, project_dir: &Path) -> Result<()> {
    let dir = project_dir.join("src/pages");
    fs::create_dir_all(&dir)?;

    let path_slug = name.to_lowercase();
    let content = format!(
        r#"page {}(path: "/{}", title: "{}", description: "What the {} page is for, in a sentence.") {{
    Container {{
        Heading("{}").h1
        Text("This is the {} page.")
    }}
}}
"#,
        name, path_slug, name, name, name, name
    );

    if let Some(path) = write_source(&dir, name, &content, "Page")? {
        println!(
            "Created page: src/pages/{}",
            path.file_name().unwrap().to_string_lossy()
        );
    }
    Ok(())
}

fn generate_component(name: &str, project_dir: &Path) -> Result<()> {
    let dir = project_dir.join("src/components");
    fs::create_dir_all(&dir)?;

    let content = format!(
        r#"component {} {{
    Card {{
        Text("{} component")
    }}
}}
"#,
        name, name
    );

    if let Some(path) = write_source(&dir, name, &content, "Component")? {
        println!(
            "Created component: src/components/{}",
            path.file_name().unwrap().to_string_lossy()
        );
    }
    Ok(())
}

fn generate_store(name: &str, project_dir: &Path) -> Result<()> {
    let dir = project_dir.join("src/stores");
    fs::create_dir_all(&dir)?;

    let content = format!(
        r#"store {} {{
    state items = []

    action add(item: String) {{
        items.push(item)
    }}
}}
"#,
        name
    );

    let file_name = name.to_lowercase();
    if let Some(path) = write_source(&dir, &file_name, &content, "Store")? {
        println!(
            "Created store: src/stores/{}",
            path.file_name().unwrap().to_string_lossy()
        );
    }
    Ok(())
}
