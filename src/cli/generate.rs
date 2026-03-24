use std::fs;
use std::path::Path;
use crate::error::Result;

pub fn run_generate(kind: &str, name: &str, project_dir: &Path) -> Result<()> {
    match kind {
        "page" => generate_page(name, project_dir),
        "component" => generate_component(name, project_dir),
        "store" => generate_store(name, project_dir),
        _ => {
            eprintln!("Unknown generator '{}'. Use: page, component, or store", kind);
            Ok(())
        }
    }
}

fn generate_page(name: &str, project_dir: &Path) -> Result<()> {
    let dir = project_dir.join("src/pages");
    fs::create_dir_all(&dir)?;

    let path_slug = name.to_lowercase();
    let content = format!(r#"Page {} (path: "/{}", title: "{}") {{
    Container {{
        Heading("{}", h1)
        Text("This is the {} page.")
    }}
}}
"#, name, path_slug, name, name, name);

    let file_path = dir.join(format!("{}.wf", name));
    if file_path.exists() {
        eprintln!("Page {} already exists", name);
        return Ok(());
    }

    fs::write(&file_path, content)?;
    println!("Created page: src/pages/{}.wf", name);
    Ok(())
}

fn generate_component(name: &str, project_dir: &Path) -> Result<()> {
    let dir = project_dir.join("src/components");
    fs::create_dir_all(&dir)?;

    let content = format!(r#"Component {} () {{
    Card {{
        Text("{} component")
    }}
}}
"#, name, name);

    let file_path = dir.join(format!("{}.wf", name));
    if file_path.exists() {
        eprintln!("Component {} already exists", name);
        return Ok(());
    }

    fs::write(&file_path, content)?;
    println!("Created component: src/components/{}.wf", name);
    Ok(())
}

fn generate_store(name: &str, project_dir: &Path) -> Result<()> {
    let dir = project_dir.join("src/stores");
    fs::create_dir_all(&dir)?;

    let content = format!(r#"Store {} {{
    state items = []

    action add(item: String) {{
        items.push(item)
    }}
}}
"#, name);

    let file_name = name.to_lowercase();
    let file_path = dir.join(format!("{}.wf", file_name));
    if file_path.exists() {
        eprintln!("Store {} already exists", name);
        return Ok(());
    }

    fs::write(&file_path, content)?;
    println!("Created store: src/stores/{}.wf", file_name);
    Ok(())
}
