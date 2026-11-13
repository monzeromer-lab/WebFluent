use std::path::Path;

use crate::diagnostics::error::{Result, WebFluentError};

/// `wf scope FILE:LINE[:COL] [-d DIR] [--data JSON] [--json]`: what a path
/// picker lists at that place — the declaration's own state and derived
/// values, its props, the bindings around the place, the members of the
/// stores it uses, and a template's data — each with its type and where it
/// comes from.
pub(crate) fn run_scope(dir: &Path, place: &str, data: Option<&Path>, json: bool) -> Result<()> {
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

    let bad = || WebFluentError::edit(format!("`{place}` is not FILE:LINE or FILE:LINE:COL"));
    let mut parts = place.rsplitn(3, ':').collect::<Vec<_>>();
    parts.reverse();
    let number = |s: &str| s.parse::<usize>().ok();
    let (file, line, col) = match parts.as_slice() {
        [f, l, c] => match (number(l), number(c)) {
            (Some(l), Some(c)) => (f.to_string(), l, c),
            // A colon in the file's own name: `a:b.wf:12`.
            (None, Some(c)) => (format!("{f}:{l}"), c, 1),
            (_, None) => return Err(bad()),
        },
        [f, l] => (f.to_string(), number(l).ok_or_else(bad)?, 1),
        _ => return Err(bad()),
    };
    let source = std::fs::read_to_string(dir.join(&file))?;
    let mut offset = 0;
    for (i, text) in source.split_inclusive('\n').enumerate() {
        if i + 1 == line {
            offset += text
                .char_indices()
                .nth(col.saturating_sub(1))
                .map_or(0, |(b, _)| b);
            break;
        }
        offset += text.len();
    }
    let mut entries = crate::scope::paths::at_via(&crate::vfs::FsVfs, dir, &file, offset)?;
    if let Some(data) = data {
        let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data)?)
            .map_err(|e| WebFluentError::edit(format!("{}: {e}", data.display())))?;
        entries.extend(crate::scope::paths::data(&value));
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&entries).unwrap_or_default()
        );
        return Ok(());
    }
    print(&entries, 0);
    Ok(())
}
