use std::fs;
use std::io::{self, Read};
use std::path::Path;
use crate::error::{WebFluentError, Result};
use crate::template::Template;

pub fn run_render(
    template_path: &Path,
    data_path: Option<&Path>,
    format: &str,
    output_path: Option<&Path>,
    theme: &str,
) -> Result<()> {
    // Read template
    let source = fs::read_to_string(template_path).map_err(|e| {
        WebFluentError::IoError(format!("Failed to read template '{}': {}", template_path.display(), e))
    })?;

    // Read JSON data
    let json_str = if let Some(dp) = data_path {
        fs::read_to_string(dp).map_err(|e| {
            WebFluentError::IoError(format!("Failed to read data '{}': {}", dp.display(), e))
        })?
    } else {
        // Read from stdin
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).map_err(|e| {
            WebFluentError::IoError(format!("Failed to read stdin: {}", e))
        })?;
        buf
    };

    let data: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
        WebFluentError::ConfigError(format!("Invalid JSON data: {}", e))
    })?;

    let tpl = Template::from_str(&source)?.with_theme(theme);

    match format {
        "html" => {
            let html = tpl.render_html(&data)?;
            write_output(output_path, html.as_bytes())?;
        }
        "html-fragment" | "fragment" => {
            let frag = tpl.render_html_fragment(&data)?;
            write_output(output_path, frag.as_bytes())?;
        }
        "pdf" => {
            let pdf = tpl.render_pdf(&data)?;
            write_output(output_path, &pdf)?;
        }
        _ => {
            return Err(WebFluentError::ConfigError(
                format!("Unknown format '{}'. Use 'html', 'html-fragment', or 'pdf'.", format)
            ));
        }
    }

    if let Some(out) = output_path {
        eprintln!("  Rendered → {}", out.display());
    }

    Ok(())
}

fn write_output(path: Option<&Path>, data: &[u8]) -> Result<()> {
    if let Some(p) = path {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(p, data)?;
    } else {
        use std::io::Write;
        io::stdout().write_all(data).map_err(|e| {
            WebFluentError::IoError(format!("Failed to write stdout: {}", e))
        })?;
    }
    Ok(())
}
