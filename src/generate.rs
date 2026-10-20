//! What `wf generate` writes: a new page, component or store — or an empty
//! file — in the layout the project's sources are written in.

use crate::error::Result;
use crate::vfs::{FsVfs, Vfs};
use std::path::{Path, PathBuf};

/// What a new file declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Page,
    Component,
    Store,
    /// Nothing: an empty file.
    Empty,
}

impl Kind {
    /// The directory under `src/` `wf generate` writes one to; empty for a
    /// file that declares nothing.
    pub fn folder(self) -> &'static str {
        match self {
            Kind::Page => "pages",
            Kind::Component => "components",
            Kind::Store => "stores",
            Kind::Empty => "",
        }
    }

    /// What `wf generate` calls one: `Page Home already exists`.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Page => "Page",
            Kind::Component => "Component",
            Kind::Store => "Store",
            Kind::Empty => "File",
        }
    }

    /// The file stem `wf generate <kind> <name>` writes: a store's is lower
    /// case (`stores/deploys.wf` declares `Deploys`), the rest are the name.
    pub fn file_stem(self, name: &str) -> String {
        match self {
            Kind::Store => name.to_lowercase(),
            _ => name.to_string(),
        }
    }
}

/// The source of a new `kind` declaring `name`, in the braced layout.
pub fn source(kind: Kind, name: &str) -> String {
    match kind {
        Kind::Page => {
            let path_slug = name.to_lowercase();
            format!(
                r#"page {}(path: "/{}", title: "{}", description: "What the {} page is for, in a sentence.") {{
    Container {{
        Heading("{}").h1
        Text("This is the {} page.")
    }}
}}
"#,
                name, path_slug, name, name, name, name
            )
        }
        Kind::Component => format!(
            r#"component {} {{
    Card {{
        Text("{} component")
    }}
}}
"#,
            name, name
        ),
        Kind::Store => format!(
            r#"store {} {{
    state items = []

    action add(item: String) {{
        items.push(item)
    }}
}}
"#,
            name
        ),
        Kind::Empty => String::new(),
    }
}

/// The name a file of stem `file_stem` declares: `status-pill` →
/// `StatusPill`, `deploys` → `Deploys`.
///
/// Letters and digits are kept, and every other character separates two
/// words. A stem with no letter, or one that starts with a digit, makes no
/// name a declaration can have: the caller refuses it.
pub fn declaration_name(file_stem: &str) -> String {
    file_stem
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            let first = chars.next().map(|c| c.to_uppercase().collect::<String>());
            first.unwrap_or_default() + chars.as_str()
        })
        .collect()
}

/// Whether the sources under `src_dir` are written in the indented layout:
/// there are `.wfx` files and no `.wf`.
pub fn is_indented(src_dir: &Path) -> bool {
    is_indented_via(&FsVfs, src_dir)
}

/// [`is_indented`], looking through `vfs`.
pub fn is_indented_via(vfs: &dyn Vfs, src_dir: &Path) -> bool {
    let mut has_wfx = false;
    let mut has_wf = false;
    fn walk(vfs: &dyn Vfs, dir: &Path, has_wfx: &mut bool, has_wf: &mut bool) {
        let Ok(entries) = vfs.read_dir(dir) else {
            return;
        };
        for path in entries {
            if vfs.is_dir(&path) {
                walk(vfs, &path, has_wfx, has_wf);
            } else if path.extension().is_some_and(|e| e == "wfx") {
                *has_wfx = true;
            } else if path.extension().is_some_and(|e| e == "wf") {
                *has_wf = true;
            }
        }
    }
    walk(vfs, src_dir, &mut has_wfx, &mut has_wf);
    has_wfx && !has_wf
}

/// What writing a new file came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Written {
    /// The file written.
    Created(PathBuf),
    /// Nothing was written: a file of that stem is there already, in one
    /// layout or the other.
    Exists(PathBuf),
}

/// Write `content`, a braced source, as `<dir>/<stem>.wf` — or as a
/// `.wfx` in the indented layout when `indented`.
pub fn write_source(dir: &Path, stem: &str, content: &str, indented: bool) -> Result<Written> {
    let (path, text) = if indented {
        let path = dir.join(format!("{stem}.wfx"));
        // An empty file is empty in either layout.
        let text = if content.is_empty() {
            String::new()
        } else {
            crate::layout::to_offside(content, &path.to_string_lossy())?
        };
        (path, text)
    } else {
        (dir.join(format!("{stem}.wf")), content.to_string())
    };
    if path.exists() || path.with_extension("wf").exists() || path.with_extension("wfx").exists() {
        return Ok(Written::Exists(path));
    }
    std::fs::write(&path, text)?;
    Ok(Written::Created(path))
}

/// Write a new `kind` as `<dir>/<file_stem>.wf` (or `.wfx`), declaring
/// [`declaration_name`]`(file_stem)`, in the layout of the project `dir` is
/// in — the one around the nearest `webfluent.app.json`, or `dir`'s parent
/// when there is none, as `wf generate` reads `src/` from `src/pages`.
///
/// `dir` can be anywhere under the project: the kind's usual folder is
/// where `wf generate` puts it, not a rule. It is created if need be.
pub fn create(kind: Kind, dir: &Path, file_stem: &str) -> Result<Written> {
    let src = dir
        .ancestors()
        .find(|at| at.join("webfluent.app.json").is_file())
        .map(|root| root.join("src"))
        .unwrap_or_else(|| dir.parent().unwrap_or(dir).to_path_buf());
    std::fs::create_dir_all(dir)?;
    let content = source(kind, &declaration_name(file_stem));
    write_source(dir, file_stem, &content, is_indented(&src))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_stem_names_its_declaration() {
        assert_eq!(declaration_name("status-pill"), "StatusPill");
        assert_eq!(declaration_name("deploys"), "Deploys");
        assert_eq!(declaration_name("Home"), "Home");
        assert_eq!(declaration_name("user_profile card"), "UserProfileCard");
        assert_eq!(declaration_name("pageTwo"), "PageTwo");
        assert_eq!(declaration_name("--"), "");
    }

    #[test]
    fn every_kind_parses() {
        for kind in [Kind::Page, Kind::Component, Kind::Store] {
            let text = source(kind, "StatusPill");
            let program = crate::syntax::parse_source(&text, "x.wf")
                .unwrap_or_else(|e| panic!("{kind:?}: {e}\n{text}"));
            assert_eq!(program.declarations.len(), 1, "{kind:?}");
        }
        assert_eq!(source(Kind::Empty, "X"), "");
        assert!(source(Kind::Page, "Home").contains("path: \"/home\""));
    }

    #[test]
    fn a_new_file_takes_the_layout_of_its_project() {
        let root = std::env::temp_dir().join(format!("wf-generate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src/pages")).unwrap();
        std::fs::write(root.join("webfluent.app.json"), "{}").unwrap();
        std::fs::write(
            root.join("src/App.wfx"),
            "page Home(path: \"/\")\n    Text(\"hi\")\n",
        )
        .unwrap();

        // Deeper than `wf generate` writes, and indented all the same.
        let dir = root.join("src/components/cards");
        let Written::Created(path) = create(Kind::Component, &dir, "status-pill").unwrap() else {
            panic!("created");
        };
        assert_eq!(path, dir.join("status-pill.wfx"));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("component StatusPill\n"), "{text}");
        assert!(!text.contains('{'), "{text}");
        crate::syntax::parse_source(&text, "status-pill.wfx").unwrap();

        // A file of that stem in either layout is left alone.
        let pages = root.join("src/pages");
        std::fs::write(pages.join("About.wf"), "").unwrap();
        assert_eq!(
            write_source(&pages, "About", &source(Kind::Page, "About"), true).unwrap(),
            Written::Exists(pages.join("About.wfx"))
        );
        assert_eq!(std::fs::read_to_string(pages.join("About.wf")).unwrap(), "");

        let Written::Created(empty) = create(Kind::Empty, &root.join("src"), "notes").unwrap()
        else {
            panic!("created");
        };
        assert_eq!(std::fs::read_to_string(empty).unwrap(), "");
        let _ = std::fs::remove_dir_all(&root);
    }
}
