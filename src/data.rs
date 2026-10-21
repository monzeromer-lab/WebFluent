//! `data posts = "posts.json"`: a file read at build time, whose JSON is
//! the value of a constant. The build resolves every `data` declaration
//! into a `const` before anything else runs, so the checker, the static
//! paint, the template engine and the bundle all see a plain value.

use std::path::Path;

use crate::error::{Result, WebFluentError};
use crate::parser::ast::*;

/// Every `data` declaration of `program` replaced by a `const` holding
/// the file's JSON, read relative to `root`, then `root/src`.
pub fn resolve_data(program: &mut Program, root: &Path) -> Result<()> {
    resolve_data_with(program, root, None)
}

/// [`resolve_data`], with somewhere to write the images to.
///
/// `media` is the output directory and the settings when the build is one
/// that writes files; without it an image is read for its size and colour
/// but nothing is written, which is what the language server and `wf
/// types` want.
pub fn resolve_data_with(
    program: &mut Program,
    root: &Path,
    media: Option<(&Path, &crate::media::Settings, &str)>,
) -> Result<()> {
    resolve_data_via(&crate::vfs::FsVfs, program, root, media)
}

/// [`resolve_data_with`], reading the files through `vfs`.
pub fn resolve_data_via(
    vfs: &dyn crate::vfs::Vfs,
    program: &mut Program,
    root: &Path,
    media: Option<(&Path, &crate::media::Settings, &str)>,
) -> Result<()> {
    for decl in &mut program.declarations {
        let Declaration::Data(d) = decl else {
            continue;
        };
        // `image hero = "media/hero.jpg"`: the picture itself, at every
        // width the page will ask for.
        if d.is_image {
            let candidates = [
                root.join(&d.file),
                root.join("src").join(&d.file),
                root.join("public").join(&d.file),
            ];
            let Some(path) = candidates.iter().find(|p| vfs.is_file(p)) else {
                return Err(WebFluentError::IoError(format!(
                    "`image {}`: no file `{}` under {}, its src/ or its public/",
                    d.name,
                    d.file,
                    root.display()
                )));
            };
            let (out_dir, settings, base_path) = match media {
                Some(m) => m,
                // Nothing to write to: the name still resolves, to the
                // file as it stands.
                None => {
                    *decl = Declaration::Const(ConstDecl {
                        name: d.name.clone(),
                        ty: d.ty.clone(),
                        value: json_expr(&serde_json::json!({ "src": format!("/{}", d.file) })),
                        doc: d.doc.clone(),
                        span: d.span,
                    });
                    continue;
                }
            };
            let asset = crate::media::process_via(
                vfs,
                &d.name,
                path,
                out_dir,
                &root.join(".wf-cache/media"),
                settings,
                base_path,
            )?;
            *decl = Declaration::Const(ConstDecl {
                name: d.name.clone(),
                ty: d.ty.clone(),
                value: json_expr(&asset.as_json()),
                doc: d.doc.clone(),
                span: d.span,
            });
            continue;
        }
        let candidates = [root.join(&d.file), root.join("src").join(&d.file)];
        let Some(path) = candidates.iter().find(|p| vfs.is_file(p)) else {
            return Err(WebFluentError::IoError(format!(
                "`data {}`: no file `{}` under {} or its src/",
                d.name,
                d.file,
                root.display()
            )));
        };
        let text = vfs.read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            WebFluentError::IoError(format!(
                "`data {}`: {} is not JSON: {e}",
                d.name,
                path.display()
            ))
        })?;
        *decl = Declaration::Const(ConstDecl {
            name: d.name.clone(),
            ty: d.ty.clone(),
            value: json_expr(&json),
            doc: d.doc.clone(),
            span: d.span,
        });
    }
    // `Image(src: "/hero.png")`, naming a picture of `public/`: the same
    // picture an `image` declaration makes of it.
    if let Some((out_dir, settings, base_path)) = media
        && settings.pipeline
        && settings.literal_src
    {
        let mut made = std::collections::HashMap::new();
        let mut lower = |ui: &mut UIElement| {
            lower_image_src(vfs, ui, root, out_dir, settings, base_path, &mut made)
        };
        for decl in &mut program.declarations {
            let body = match decl {
                Declaration::Page(p) => &mut p.body,
                Declaration::Component(c) => &mut c.body,
                Declaration::App(a) => &mut a.body,
                Declaration::Store(s) => &mut s.body,
                _ => continue,
            };
            each_element_mut(body, &mut lower);
        }
    }
    Ok(())
}

/// The pictures the pipeline re-encodes, by extension: what it can decode.
pub const PROCESSED: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

/// The file of `public/` a literal `src:` on `Image` names, when it is a
/// picture the pipeline re-encodes: `"/hero.png"` is `public/hero.png`.
pub fn processed_src(ui: &UIElement) -> Option<(usize, String)> {
    if !matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Image")
        || ui.args.iter().any(|a| matches!(a, Arg::Positional(_)))
    {
        return None;
    }
    ui.args.iter().enumerate().find_map(|(i, a)| match a {
        Arg::Named(k, Expr::StringLiteral(url)) if k == "src" => {
            let path = url.split(['?', '#']).next().unwrap_or("");
            let ext = path.rsplit_once('.')?.1.to_ascii_lowercase();
            (path.starts_with('/')
                && !path.starts_with("//")
                && !path.contains("..")
                && PROCESSED.contains(&ext.as_str()))
            .then(|| (i, path.trim_start_matches('/').to_string()))
        }
        _ => None,
    })
}

/// `Image(src: "/hero.png", …)` as `Image(<the picture the build made>, …)`,
/// when `public/` has it and it can be read. Anything else is left as it
/// was: copied, and named by its path.
fn lower_image_src(
    vfs: &dyn crate::vfs::Vfs,
    ui: &mut UIElement,
    root: &Path,
    out_dir: &Path,
    settings: &crate::media::Settings,
    base_path: &str,
    made: &mut std::collections::HashMap<String, Option<serde_json::Value>>,
) {
    let Some((at, file)) = processed_src(ui) else {
        return;
    };
    let asset = made
        .entry(file.clone())
        .or_insert_with(|| {
            let source = root.join("public").join(&file);
            if !vfs.is_file(&source) {
                return None;
            }
            let stem = Path::new(&file)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            crate::media::process_via(
                vfs,
                &stem,
                &source,
                out_dir,
                &root.join(".wf-cache/media"),
                settings,
                base_path,
            )
            .ok()
            .map(|a| a.as_json())
        })
        .clone();
    if let Some(asset) = asset {
        ui.args[at] = Arg::Positional(json_expr(&asset));
    }
}

/// Every element in `stmts`, nested ones too, to change.
fn each_element_mut(stmts: &mut [Statement], f: &mut dyn FnMut(&mut UIElement)) {
    for stmt in stmts {
        if let StatementKind::UIElement(ui) = &mut stmt.kind {
            f(ui);
            each_element_mut(&mut ui.children, f);
            for fill in &mut ui.slot_fills {
                each_element_mut(&mut fill.body, f);
            }
            for h in &mut ui.events {
                each_element_mut(&mut h.body, f);
            }
        }
        for body in stmt.kind.bodies_mut() {
            each_element_mut(body, f);
        }
    }
}

/// A `.md` page: front matter between `---` lines names the page's
/// attributes — `path`, `title`, `description`, `layout`, `image`, `type`,
/// `noindex` — and the rest is the page's Markdown. The page's name is the
/// file's stem, capitalised; its route the `path`, or `/<stem>` (`/` for
/// `index`).
pub fn markdown_page(source: &str, file: &str) -> Result<Program> {
    let stem = std::path::Path::new(file)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Page".to_string());
    let mut front: Vec<(String, String)> = Vec::new();
    let body = if let Some(rest) = source.strip_prefix("---") {
        let rest = rest
            .trim_start_matches(['\r'])
            .strip_prefix('\n')
            .unwrap_or(rest);
        let Some(end) = rest.find("\n---") else {
            return Err(WebFluentError::ParseError(Box::new(
                crate::error::Diagnostic::coded(
                    "E002",
                    "The front matter opened with `---` is never closed",
                    file,
                    1,
                    1,
                ),
            )));
        };
        for line in rest[..end].lines() {
            let Some((k, v)) = line.split_once(':') else {
                continue;
            };
            let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
            front.push((k.trim().to_string(), v));
        }
        rest[end + 4..].trim_start_matches('-').to_string()
    } else {
        source.to_string()
    };
    let get = |key: &str| front.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
    let path = get("path").unwrap_or_else(|| {
        if stem == "index" {
            "/".to_string()
        } else {
            format!("/{}", stem.to_lowercase().replace(' ', "-"))
        }
    });
    let mut name: String = stem.chars().filter(|c| c.is_alphanumeric()).collect();
    if let Some(first) = name.get(..1) {
        name = format!("{}{}", first.to_uppercase(), &name[1..]);
    }
    if name.is_empty() {
        name = "Page".to_string();
    }
    let span = Span::dummy();
    let markdown = Statement::new(
        StatementKind::UIElement(UIElement {
            component: ComponentRef::BuiltIn("Markdown".to_string()),
            args: vec![Arg::Positional(Expr::StringLiteral(
                body.trim().to_string(),
            ))],
            modifiers: Vec::new(),
            children: Vec::new(),
            style_block: None,
            transition_block: None,
            events: Vec::new(),
            slot_fills: Vec::new(),
            span,
            paren_span: None,
            body_span: None,
            style_span: None,
            arg_spans: vec![span],
            modifier_spans: Vec::new(),
        }),
        span,
    );
    let layout = get("layout").map(|l| LayoutRef {
        name: l,
        args: Vec::new(),
        span,
    });
    Ok(Program {
        declarations: vec![Declaration::Page(PageDecl {
            name,
            path,
            title: get("title"),
            title_expr: None,
            description_expr: None,
            guard: None,
            redirect: None,
            description: get("description"),
            image: get("image"),
            image_alt: get("image_alt"),
            page_type: get("type"),
            noindex: get("noindex").is_some_and(|v| v == "true"),
            layout,
            params: Vec::new(),
            head: Vec::new(),
            paths: None,
            body: vec![markdown],
            span,
            header_span: span,
            body_span: span,
        })],
    })
}

/// A JSON value as the expression that writes it.
pub fn json_expr(value: &serde_json::Value) -> Expr {
    match value {
        serde_json::Value::Null => Expr::Null,
        serde_json::Value::Bool(b) => Expr::BoolLiteral(*b),
        serde_json::Value::Number(n) => Expr::NumberLiteral(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Expr::StringLiteral(s.clone()),
        serde_json::Value::Array(items) => Expr::ListLiteral(items.iter().map(json_expr).collect()),
        serde_json::Value::Object(map) => {
            Expr::MapLiteral(map.iter().map(|(k, v)| (k.clone(), json_expr(v))).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_element(source: &str) -> UIElement {
        let (program, errors) = crate::syntax::parse_source_recovering(source, "t.wf");
        assert!(errors.is_empty(), "{errors:?}");
        let Declaration::Page(p) = &program.declarations[0] else {
            panic!("a page");
        };
        let StatementKind::UIElement(ui) = &p.body[0].kind else {
            panic!("an element");
        };
        ui.clone()
    }

    fn src_of(element: &str) -> Option<String> {
        let source = format!(
            "page Home(path: \"/\", title: \"Home\", description: \"Home.\") {{\n    {element}\n}}\n"
        );
        processed_src(&first_element(&source)).map(|(_, file)| file)
    }

    /// Which `src:` the build re-encodes: a literal path of the site to a
    /// picture it can decode, on an `Image` that names no `image` of its
    /// own. Everything else is used as written.
    #[test]
    fn a_literal_src_the_pipeline_can_read_is_processed() {
        assert_eq!(
            src_of(r#"Image(src: "/hero.png", alt: "")"#).as_deref(),
            Some("hero.png")
        );
        assert_eq!(
            src_of(r#"Image(src: "/img/a.JPG?v=2", alt: "")"#).as_deref(),
            Some("img/a.JPG")
        );
        assert_eq!(
            src_of(r#"Image(src: "/a.webp#x", alt: "")"#).as_deref(),
            Some("a.webp")
        );
        for written in [
            r#"Image(src: "/logo.svg", alt: "")"#,
            r#"Image(src: "/favicon.ico", alt: "")"#,
            r#"Image(src: "https://example.com/a.png", alt: "")"#,
            r#"Image(src: "//cdn.example.com/a.png", alt: "")"#,
            r#"Image(src: "a.png", alt: "")"#,
            r#"Image(src: "/../secret.png", alt: "")"#,
            r#"Image(src: "/{name}.png", alt: "")"#,
            r#"Avatar(src: "/me.png", alt: "Me")"#,
            r#"Image(src: "/noextension", alt: "")"#,
        ] {
            assert_eq!(src_of(written), None, "{written}");
        }
    }
}
