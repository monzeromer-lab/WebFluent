//! The author's own stylesheets: every `.css` file under `src/`.
//!
//! A `style { }` block says what one element looks like, and the design
//! tokens say what the site's palette and type are. What neither can say —
//! a rule with a selector, a keyframe, a `@font-face`, a class a dozen
//! elements share by name — belongs in a stylesheet, and an author should
//! not have to leave the project to write one. Any `.css` file under `src/`
//! is part of the build: read in path order, bundled into `styles.css` after
//! the engine's own rules (so a rule here wins on equal specificity) and
//! before the rules `style { }` blocks compile to (which carry tripled
//! specificity, as an inline style would beat a sheet), and minified with
//! the rest. `var(--token)` reaches every design token the theme declares.
//!
//! An element picks a rule up by naming its class: `Card(class: "feature")`.
//! `build.split` keeps these sheets shared: a class is global by nature.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

/// Every `.css` file under `dir`, sorted by path so the bundle is stable.
pub fn find_stylesheets(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk(dir, &mut files);
    files.sort();
    files
}

fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "css") {
            files.push(path);
        }
    }
}

/// The author's stylesheets under `src_dir`, bundled in path order, each
/// under a marker that names its file. Empty when there are none.
pub fn bundle(project_dir: &Path, src_dir: &Path) -> Result<String> {
    let mut out = String::new();
    for path in find_stylesheets(src_dir) {
        let css = fs::read_to_string(&path)?;
        let name = path
            .strip_prefix(project_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        out.push_str(&format!("\n/* ─── {name} ─── */\n"));
        out.push_str(css.trim_end());
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stylesheets_under_src_are_bundled_in_path_order() {
        let dir = std::env::temp_dir().join(format!("wf-project-css-{}", std::process::id()));
        let src = dir.join("src");
        fs::create_dir_all(src.join("components")).unwrap();
        fs::write(src.join("theme.css"), ".b { color: red }\n").unwrap();
        fs::write(src.join("components/card.css"), ".a { padding: 1rem }\n").unwrap();
        fs::write(src.join("App.wf"), "").unwrap();
        let css = bundle(&dir, &src).unwrap();
        let a = css.find(".a {").unwrap();
        let b = css.find(".b {").unwrap();
        assert!(a < b, "components/card.css sorts before theme.css: {css}");
        assert!(css.contains("/* ─── src/components/card.css ─── */"));
        assert!(css.contains("/* ─── src/theme.css ─── */"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_project_without_stylesheets_bundles_nothing() {
        let dir = std::env::temp_dir().join(format!("wf-no-css-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(bundle(&dir, &dir.join("src")).unwrap(), "");
        fs::remove_dir_all(&dir).unwrap();
    }
}
