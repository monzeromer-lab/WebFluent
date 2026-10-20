use std::path::Path;

use serde_json::Value;

use crate::config::schema;
use crate::error::{Result, WebFluentError};

const CONFIG: &str = "webfluent.app.json";

/// `wf config schema [--json]`: every setting, its type, its default and
/// what it does.
pub fn run_schema(json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&schema::to_json()).unwrap_or_default()
        );
        return Ok(());
    }
    let defaults = schema::defaults();
    let mut section = "";
    for f in schema::FIELDS {
        let here = schema::section_of(f.path);
        if here != section {
            println!("\n{here}");
            section = here;
        }
        let default = schema::at(&defaults, f.path)
            .filter(|v| !v.is_null())
            .map(|v| format!(" = {v}"))
            .unwrap_or_default();
        println!(
            "  {} ({}){default}\n      {}",
            f.path,
            f.ty.word(),
            f.summary
        );
    }
    Ok(())
}

fn read(dir: &Path) -> Result<String> {
    let path = dir.join(CONFIG);
    std::fs::read_to_string(&path)
        .map_err(|e| WebFluentError::IoError(format!("{}: {e}", path.display())))
}

/// `wf config get PATH`: what the project sets, or the default, said so.
pub fn run_get(dir: &Path, dotted: &str) -> Result<()> {
    let text = read(dir)?;
    let (keys, field) = schema::path_of(dotted).ok_or_else(|| {
        WebFluentError::ConfigError(format!(
            "`{dotted}` is not a setting; `wf config schema` lists them"
        ))
    })?;
    let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
    let set = crate::config::json_edit::get(&text, &keys)
        .map_err(|e| WebFluentError::ConfigError(e.to_string()))?;
    match set {
        Some(v) => println!("{v}"),
        None => {
            let d = schema::at(&schema::defaults(), field.path)
                .cloned()
                .unwrap_or(Value::Null);
            println!("{d} (the default; {CONFIG} does not set it)");
        }
    }
    Ok(())
}

/// `wf config set PATH VALUE` and `wf config unset PATH`: the file edited
/// in place, its shape kept, the value held to the setting's type.
pub fn run_set(dir: &Path, dotted: &str, value: Option<&str>) -> Result<()> {
    let text = read(dir)?;
    // A value is JSON when it reads as JSON — `true`, `3`, `["en", "ar"]` —
    // and a string otherwise, so `wf config set meta.lang ar` needs no quotes.
    let value = value
        .map(|v| serde_json::from_str::<Value>(v).unwrap_or_else(|_| Value::String(v.to_string())));
    let out = schema::edit(&text, dotted, value.as_ref())
        .map_err(|e| WebFluentError::ConfigError(e.to_string()))?;
    if out == text {
        println!("{CONFIG} already says so.");
        return Ok(());
    }
    let path = dir.join(CONFIG);
    std::fs::write(&path, &out)
        .map_err(|e| WebFluentError::IoError(format!("{}: {e}", path.display())))?;
    match value {
        Some(v) => println!("{dotted} = {v}"),
        None => println!("{dotted} removed: the default applies"),
    }
    Ok(())
}
