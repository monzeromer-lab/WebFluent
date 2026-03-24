use std::fs;
use std::path::Path;
use crate::config::ProjectConfig;
use crate::error::Result;

pub fn run_init(name: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        return Err(crate::error::WebFluentError::IoError(
            format!("Directory '{}' already exists", name)
        ));
    }

    // Create directory structure
    fs::create_dir_all(project_dir.join("src/pages"))?;
    fs::create_dir_all(project_dir.join("src/components"))?;
    fs::create_dir_all(project_dir.join("src/stores"))?;
    fs::create_dir_all(project_dir.join("public"))?;

    // Write webfluent.app.json
    let config = ProjectConfig::default_config(name);
    let config_json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(project_dir.join("webfluent.app.json"), config_json)?;

    // Write App.wf
    let app_wf = format!(r#"App {{
    Navbar {{
        Navbar.Brand {{
            Text("{}", heading)
        }}
        Navbar.Links {{
            Link(to: "/") {{ Text("Home") }}
        }}
    }}

    Router {{
        Route(path: "/", page: Home)
    }}
}}
"#, name);
    fs::write(project_dir.join("src/App.wf"), app_wf)?;

    // Write Home.wf
    let home_wf = format!(r#"Page Home (path: "/", title: "Home") {{
    Container {{
        Heading("Welcome to {}", h1)
        Text("Your WebFluent app is ready.")

        Spacer()

        Card(elevated) {{
            Text("Edit src/pages/Home.wf to get started.", muted)
        }}
    }}
}}
"#, name);
    fs::write(project_dir.join("src/pages/Home.wf"), home_wf)?;

    println!("Created new WebFluent project: {}", name);
    println!();
    println!("  cd {}", name);
    println!("  webfluent build");
    println!("  webfluent serve");

    Ok(())
}
