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
            let Some(path) = candidates.iter().find(|p| p.is_file()) else {
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
            let asset = crate::media::process(
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
        let Some(path) = candidates.iter().find(|p| p.is_file()) else {
            return Err(WebFluentError::IoError(format!(
                "`data {}`: no file `{}` under {} or its src/",
                d.name,
                d.file,
                root.display()
            )));
        };
        let text = std::fs::read_to_string(path)?;
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
    Ok(())
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
            return Err(WebFluentError::ParseError(crate::error::Diagnostic::new(
                "The front matter opened with `---` is never closed",
                file,
                1,
                1,
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
            guard: None,
            redirect: None,
            description: get("description"),
            image: get("image"),
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
