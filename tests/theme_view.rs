//! A theme as an editor sees it, over every project in the repository that
//! declares one (and `WF_THEME_PROJECTS` — Halyard): every token's value
//! is the one the build resolves; every token the theme declares says so,
//! on its line; every `$token` a style names is a use of some token; each
//! failing A13 the build reports is a failing pairing the view shows; a
//! theme edited and not saved is the one read; and setting a token to its
//! own value writes nothing, while setting it to another changes that
//! token's bytes alone.

use std::path::{Path, PathBuf};

use webfluent::themes::view::{reset_token, set_token, view_via};
use webfluent::vfs::{FsVfs, OverlayVfs};

fn projects() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = vec![root.join("site")];
    for base in [
        "examples/tutorials",
        "examples/decks",
        "examples/documents",
        "examples/videos",
        "tests/fixtures",
    ] {
        if let Ok(entries) = std::fs::read_dir(root.join(base)) {
            let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            dirs.sort();
            out.extend(
                dirs.into_iter()
                    .filter(|d| d.join("webfluent.app.json").is_file()),
            );
        }
    }
    if let Ok(more) = std::env::var("WF_THEME_PROJECTS") {
        out.extend(more.split(':').filter(|d| !d.is_empty()).map(PathBuf::from));
    }
    out
}

#[test]
fn every_theme_is_read_as_the_build_resolves_it() {
    let mut themes = 0;
    let mut wrong = Vec::new();
    for dir in projects() {
        let Ok(view) = view_via(&FsVfs, &dir) else {
            continue;
        };
        let Some(theme) = view.theme.clone() else {
            continue;
        };
        themes += 1;
        let (program, files) = webfluent::build::read_project(&dir).unwrap();
        let config = webfluent::config::ProjectConfig::load(&dir).unwrap();
        let resolved = webfluent::themes::resolve::resolve_tokens(&program, &config.theme).unwrap();
        for t in &view.tokens {
            if let Some(v) = resolved.get(&t.name)
                && v != &t.value
            {
                wrong.push(format!(
                    "{}: {} is {} in the view, {v} in the build",
                    dir.display(),
                    t.name,
                    t.value
                ));
            }
        }
        let decl = program
            .declarations
            .iter()
            .find_map(|d| match d {
                webfluent::parser::ast::Declaration::Theme(x) if x.name == theme => Some(x),
                _ => None,
            })
            .unwrap();
        for k in &decl.tokens {
            match view.tokens.iter().find(|t| t.name == k.name) {
                Some(t) if t.declared && t.line == Some(k.span.line) => {}
                other => wrong.push(format!(
                    "{}: {} declared on line {}, the view says {:?}",
                    dir.display(),
                    k.name,
                    k.span.line,
                    other.map(|t| (t.declared, t.line))
                )),
            }
        }
        // Every `$token` a style block names is a use of some token.
        let named: usize = view.tokens.iter().map(|t| t.uses.len()).sum();
        let mut written = 0;
        for (i, d) in program.declarations.iter().enumerate() {
            let Some(body) = webfluent::print::body_of(d) else {
                continue;
            };
            let file = &files[i];
            let mut count = |stmts: &[webfluent::parser::ast::Statement]| {
                fn walk(stmts: &[webfluent::parser::ast::Statement], n: &mut usize) {
                    use webfluent::parser::ast::*;
                    for s in stmts {
                        if let StatementKind::UIElement(ui) = &s.kind {
                            if let Some(b) = &ui.style_block {
                                for p in b
                                    .properties
                                    .iter()
                                    .chain(b.media_queries.iter().flat_map(|m| m.properties.iter()))
                                    .chain(b.pseudo_blocks.iter().flat_map(|q| q.properties.iter()))
                                {
                                    let text = webfluent::print::style_value(&p.value);
                                    if text.contains('$') {
                                        *n += 1;
                                    }
                                }
                            }
                            walk(&ui.children, n);
                            for f in &ui.slot_fills {
                                walk(&f.body, n);
                            }
                        }
                        for b in s.kind.bodies() {
                            walk(b, n);
                        }
                    }
                }
                walk(stmts, &mut written);
            };
            count(body);
            let _ = file;
        }
        if written > 0 && named == 0 {
            wrong.push(format!(
                "{}: {written} styles name a token, the view finds no use",
                dir.display()
            ));
        }
        // What fails A13 in the build is a failing pairing here.
        let failing: Vec<&str> = view
            .pairs
            .iter()
            .filter(|p| p.drawn && !p.passes())
            .map(|p| p.description)
            .collect();
        let warned = webfluent::linter::contrast::lint_contrast(&program, &resolved);
        for w in &warned {
            if !failing.iter().any(|d| w.message.starts_with(d)) {
                wrong.push(format!(
                    "{}: the build warns {:?}, the view has no failing pair for it",
                    dir.display(),
                    w.message
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    assert!(themes >= 10, "{themes} themes");
}

#[test]
fn an_unsaved_theme_is_the_one_read_and_a_token_is_written_in_its_place() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut edited = 0;
    for dir in projects() {
        let Ok(view) = view_via(&FsVfs, &dir) else {
            continue;
        };
        let (Some(theme), Some(file)) = (view.theme.clone(), view.file.clone()) else {
            continue;
        };
        let path = dir.join(&file);
        let source = std::fs::read_to_string(&path).unwrap();
        let declared: Vec<&webfluent::themes::view::Token> =
            view.tokens.iter().filter(|t| t.declared).collect();
        for t in &declared {
            // To its own value: not a byte moves.
            let same = set_token(&source, &file, &theme, &t.name, &t.value).unwrap();
            assert_eq!(same, source, "{}: {} set to itself", path.display(), t.name);
        }
        let Some(t) = declared
            .iter()
            .find(|t| t.group == "color" || t.group == "spacing")
        else {
            continue;
        };
        let value = if t.group == "color" {
            "#123456"
        } else {
            "13px"
        };
        let out = set_token(&source, &file, &theme, &t.name, value).unwrap();
        let line = t.line.unwrap() as usize;
        let changed: Vec<usize> = source
            .lines()
            .zip(out.lines())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i + 1)
            .collect();
        assert_eq!(
            changed,
            [line],
            "{}: only {}'s line",
            path.display(),
            t.name
        );
        // Unsaved, the editor's view has the new value; the disk is as it was.
        let mut vfs = OverlayVfs::over_disk();
        vfs.set(&path, out.clone());
        let after = view_via(&vfs, &dir).unwrap();
        let now = after.tokens.iter().find(|x| x.name == t.name).unwrap();
        assert_eq!(now.value, value, "{}", path.display());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
        // Reset: the baseline's again, or gone when it is the theme's own.
        let reset = reset_token(&source, &file, &theme, &t.name).unwrap();
        assert!(
            !reset.contains(&format!("{}:", t.name))
                || source.matches(&format!("{}:", t.name)).count() > 1
        );
        edited += 1;
    }
    assert!(edited >= 8, "{edited} themes edited");
    let _ = root;
}
