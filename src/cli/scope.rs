use std::path::Path;

use crate::error::{Result, WebFluentError};

/// `wf scope FILE:LINE[:COL] [-d DIR] [--data JSON] [--json]`: what a path
/// picker lists at that place — the declaration's own state and derived
/// values, its props, the bindings around the place, the members of the
/// stores it uses, and a template's data — each with its type and where it
/// comes from.
pub fn run_scope(dir: &Path, place: &str, data: Option<&Path>, json: bool) -> Result<()> {
    let bad = || WebFluentError::EditError(format!("`{place}` is not FILE:LINE or FILE:LINE:COL"));
    let mut parts = place.rsplitn(3, ':').collect::<Vec<_>>();
    parts.reverse();
    let (file, line, col) = match parts.as_slice() {
        [f, l, c] if l.parse::<usize>().is_ok() && c.parse::<usize>().is_ok() => (
            f.to_string(),
            l.parse::<usize>().unwrap(),
            c.parse::<usize>().unwrap(),
        ),
        [f, l, c] if c.parse::<usize>().is_ok() => {
            (format!("{f}:{l}"), c.parse::<usize>().unwrap(), 1)
        }
        [f, l] if l.parse::<usize>().is_ok() => (f.to_string(), l.parse::<usize>().unwrap(), 1),
        _ => return Err(bad()),
    };
    let source = std::fs::read_to_string(dir.join(&file))?;
    let mut offset = 0;
    for (i, text) in source.split_inclusive('\n').enumerate() {
        if i + 1 == line {
            offset += text
                .char_indices()
                .nth(col.saturating_sub(1))
                .map(|(b, _)| b)
                .unwrap_or(0);
            break;
        }
        offset += text.len();
    }
    let mut entries = crate::scope::paths::at_via(&crate::vfs::FsVfs, dir, &file, offset)?;
    if let Some(data) = data {
        let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data)?)
            .map_err(|e| WebFluentError::EditError(format!("{}: {e}", data.display())))?;
        entries.extend(crate::scope::paths::data(&value));
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&entries).unwrap_or_default()
        );
        return Ok(());
    }
    fn print(entries: &[crate::scope::paths::Entry], indent: usize) {
        for e in entries {
            println!(
                "{:indent$}{:<24} {:<20} {}",
                "",
                e.path,
                e.ty,
                e.from,
                indent = indent
            );
            print(&e.members, indent + 2);
        }
    }
    print(&entries, 0);
    Ok(())
}
