//! `wf docs`: a project's components, documented as the official reference
//! documents the built-ins.
//!
//! Two builds make it. The docs themselves are a WebFluent site compiled
//! from the kit beside this file — the reference's Blueprint theme, its
//! stylesheet and its fonts — over `docs.json`, what this reads of the
//! project: each component with its `///` comment, its props, events,
//! slots and parts, and who places it; the types, the stores and the
//! pages. And every `example` the project declares is a page of the
//! project's own, built as `wf test` builds a test's page — its theme, its
//! stylesheets, its scripts — under `examples/`, which the docs draw in a
//! frame as tall as what it drew, beside its source. A component with no
//! example is given one made from its props, when every prop it needs has
//! a value to make up.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::base::outfs::At;
use crate::diagnostics::error::{Result, WebFluentError};
use crate::syntax::parser::ast::{
    ComponentDecl, Declaration, ExampleDecl, Program, Statement, StatementKind, TypeRef,
};
use crate::syntax::print;

/// The kit: the docs site's own sources, compiled with `docs.json`.
const KIT: &[(&str, &str)] = &[
    ("src/App.wf", include_str!("kit/kit.wf")),
    ("src/model.wf", include_str!("kit/model.wf")),
    ("src/pages.wf", include_str!("kit/pages.wf")),
    ("src/theme.wf", include_str!("kit/theme.wf")),
    ("src/blueprint.css", include_str!("kit/blueprint.css")),
];

/// The faces the theme names, served from the docs' own `fonts/`.
const FONTS: &[(&str, &[u8])] = &[
    (
        "public/fonts/instrument-sans-latin.woff2",
        include_bytes!("kit/fonts/instrument-sans-latin.woff2"),
    ),
    (
        "public/fonts/jetbrains-mono-latin.woff2",
        include_bytes!("kit/fonts/jetbrains-mono-latin.woff2"),
    ),
];

/// What an example's page says to the frame that holds it: how tall it
/// drew, each time that changes.
const FRAME_SCRIPT: &str = r"(function () {
  function tall() {
    var bottom = 0;
    for (var i = 0; i < document.body.children.length; i++) {
      var r = document.body.children[i].getBoundingClientRect();
      if (r.height > 0) bottom = Math.max(bottom, r.bottom + window.scrollY);
    }
    return Math.ceil(bottom + 24);
  }
  function say() { parent.postMessage({ wfDocsHeight: tall() }, location.origin); }
  new ResizeObserver(say).observe(document.body);
  new MutationObserver(say).observe(document.body, { subtree: true, childList: true });
  addEventListener('load', say);
  say();
})();
";

/// An example's page on the docs' stage: on the project's own page colour,
/// with room around what it draws, and only as tall as that — a page's
/// root fills the window, and the window is the frame, which would grow
/// with every report.
const FRAME_STYLE: &str =
    "html, body, #app { min-height: 0 !important; }\nbody { margin: 0; padding: 24px; }\n";

/// The marker `wf docs` leaves in what it writes, so it may clear the
/// folder the next time and never one it did not write.
const MARK: &str = ".wf-docs";

/// What [`write`] wrote.
#[derive(Debug, Clone)]
pub struct Written {
    /// Where.
    pub out: PathBuf,
    /// The project's components.
    pub components: usize,
    /// Its examples, written and made from props.
    pub examples: usize,
    pub made: usize,
}

/// One example to draw: written in the project, or made from a
/// component's props.
struct Drawn {
    example: ExampleDecl,
    code: String,
    made: bool,
}

/// Document the project in `project_dir` into `out`.
///
/// # Errors
///
/// What reading the project returns ([`crate::build::read_project`]), what
/// [`crate::build::check`] finds when the project does not check, a kit
/// build that fails (a WebFluent bug), and an [`IoError`](WebFluentError::IoError) writing the folder.
pub fn write(project_dir: &Path, out: &Path) -> Result<Written> {
    // An example is held to the language as a page is: one that does not
    // check is not drawn, and what is wrong is said.
    crate::build::check(&crate::build::BuildOptions::new(
        project_dir,
        &crate::base::vfs::FsVfs,
    ))
    .map_err(|f| f.error)?;
    let (program, files) = crate::build::read_project(project_dir)?;
    let tests = crate::tools::testing::collect(project_dir)?;
    let project = crate::project::config::ProjectConfig::load(project_dir)
        .map(|c| c.name)
        .ok()
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| {
            project_dir
                .canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "Project".to_string())
        });

    clear(out)?;
    std::fs::create_dir_all(out).at(out)?;

    // The examples, each a site of one page.
    // `public/` is copied to the root once, not linked into every example.
    let styles = crate::tools::testing::Styles {
        public: None,
        ..tests.styles.clone()
    };
    let site = crate::tools::testing::Site {
        styles: &styles,
        ..tests.site()
    };
    let mut by_component: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let (mut written, mut made) = (0, 0);
    for drawn in examples(&program, &files, project_dir) {
        let component = drawn.example.component.clone();
        let list = by_component.entry(component.clone()).or_default();
        let at = format!("examples/{}/{}", slug(&component), list.len() + 1);
        let dir = out.join(&at);
        crate::tools::testing::example_site(
            &tests.shared,
            &drawn.example,
            &dir,
            &format!("/{at}"),
            &site,
        )?;
        framed(&dir)?;
        list.push(json!({
            "name": drawn.example.name,
            "doc": drawn.example.doc.clone().unwrap_or_default(),
            "code": drawn.code,
            "src": format!("/{at}/"),
            "made": drawn.made,
        }));
        if drawn.made {
            made += 1;
        } else {
            written += 1;
        }
    }

    // The docs, over what was read.
    let docs = model(&program, &files, &project, &by_component);
    let components = docs["components"].as_array().map_or(0, Vec::len);
    let mut sources: Vec<(String, crate::tools::memory::Contents)> = KIT
        .iter()
        .map(|(p, t)| {
            (
                (*p).to_string(),
                crate::tools::memory::Contents::Text((*t).to_string()),
            )
        })
        .collect();
    sources.extend(FONTS.iter().map(|(p, b)| {
        (
            (*p).to_string(),
            crate::tools::memory::Contents::Bytes(b.to_vec()),
        )
    }));
    sources.push((
        "src/docs.json".to_string(),
        crate::tools::memory::Contents::Text(docs.to_string()),
    ));
    let config = json!({
        "name": format!("{project} — Components"),
        "theme": { "name": "Night", "builtin": "full" },
        "build": { "ssg": true, "split": true, "minify": true, "compress": false, "csp": true, "clean_urls": "file" },
        "meta": {
            "title": format!("{project} — Components"),
            "description": format!("The components of {project}."),
            "lang": "en",
            "sitemap": false,
            "frame": [],
            "preload": ["/fonts/instrument-sans-latin.woff2", "/fonts/jetbrains-mono-latin.woff2"]
        }
    });
    let built = crate::tools::memory::compile_files(
        sources,
        Some(&config.to_string()),
        &crate::tools::memory::Options::default(),
    );
    if !built.ok {
        let mut why: Vec<String> = built
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .map(ToString::to_string)
            .collect();
        why.extend(built.error);
        return Err(WebFluentError::codegen(format!(
            "the docs could not be built (a WebFluent bug):\n{}",
            why.join("\n")
        )));
    }
    for (path, text) in &built.files {
        put(out, path, text.as_bytes())?;
    }
    for (path, bytes) in &built.binary {
        put(out, path, bytes)?;
    }
    // `public/` at the root, where an example's site-relative address
    // finds it; the docs' own files win.
    if let Some(public) = &styles_public(&tests) {
        copy_missing(public, out)?;
    }
    std::fs::write(
        out.join(MARK),
        "written by `wf docs`; the next run clears this folder\n",
    )
    .at(out.join(MARK))?;
    Ok(Written {
        out: out.to_path_buf(),
        components,
        examples: written,
        made,
    })
}

fn styles_public(tests: &crate::tools::testing::Tests) -> Option<PathBuf> {
    tests.styles.public.clone()
}

/// Clear what an earlier `wf docs` wrote; a folder it did not write is
/// left as it is (an `index.html` alone was the gallery's, 7.0's).
fn clear(out: &Path) -> Result<()> {
    if out.join(MARK).is_file() {
        std::fs::remove_dir_all(out).at(out)?;
    } else if out.join("index.html").is_file() && std::fs::read_dir(out).at(out)?.count() == 1 {
        std::fs::remove_file(out.join("index.html")).at(out)?;
    }
    Ok(())
}

fn put(out: &Path, path: &str, bytes: &[u8]) -> Result<()> {
    let to = out.join(path);
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).at(parent)?;
    }
    std::fs::write(&to, bytes).at(&to)?;
    Ok(())
}

fn copy_missing(from: &Path, to: &Path) -> Result<()> {
    for entry in std::fs::read_dir(from).at(from)? {
        let path = entry?.path();
        let Some(name) = path.file_name() else {
            continue;
        };
        let target = to.join(name);
        if path.is_dir() {
            std::fs::create_dir_all(&target).at(&target)?;
            copy_missing(&path, &target)?;
        } else if !target.exists() {
            std::fs::copy(&path, &target).at(&path)?;
        }
    }
    Ok(())
}

/// An example's page, told to say how tall it drew and to sit on the
/// stage.
fn framed(dir: &Path) -> Result<()> {
    std::fs::write(dir.join("docs-frame.js"), FRAME_SCRIPT).at(dir.join("docs-frame.js"))?;
    std::fs::write(dir.join("docs-frame.css"), FRAME_STYLE).at(dir.join("docs-frame.css"))?;
    let index = dir.join("index.html");
    let html = std::fs::read_to_string(&index).at(&index)?;
    let html = html.replacen(
        "</head>",
        "<link rel=\"stylesheet\" href=\"docs-frame.css\">\n<script defer src=\"docs-frame.js\"></script>\n</head>",
        1,
    );
    std::fs::write(&index, html).at(&index)?;
    Ok(())
}

/// Every example, in the order the components are declared: those the
/// project writes, and for a component with none, one made from its props.
fn examples(program: &Program, files: &[String], dir: &Path) -> Vec<Drawn> {
    let mut sources: BTreeMap<&str, Option<String>> = BTreeMap::new();
    let mut out = Vec::new();
    let mut shown: BTreeSet<&str> = BTreeSet::new();
    for (i, d) in program.declarations.iter().enumerate() {
        let Declaration::Example(e) = d else { continue };
        let file = files.get(i).map_or("", String::as_str);
        let source = sources
            .entry(file)
            .or_insert_with(|| std::fs::read_to_string(dir.join(file)).ok());
        let code = source
            .as_deref()
            .filter(|_| file.ends_with(".wf"))
            .and_then(|s| s.get(e.body_span.start as usize..e.body_span.end as usize))
            .map_or_else(|| printed_body(e, program), dedent);
        shown.insert(&e.component);
        out.push(Drawn {
            example: e.clone(),
            code,
            made: false,
        });
    }
    for d in &program.declarations {
        let Declaration::Component(c) = d else {
            continue;
        };
        // A part is its owner's, drawn on its page.
        if c.name.contains('.') || shown.contains(c.name.as_str()) {
            continue;
        }
        if let Some(drawn) = made(c) {
            out.push(drawn);
        }
    }
    // In the order the components are declared.
    let order: Vec<&str> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) => Some(c.name.as_str()),
            _ => None,
        })
        .collect();
    out.sort_by_key(|d| order.iter().position(|n| *n == d.example.component));
    out
}

/// The body of an example as the formatter writes it, for a source that
/// cannot be sliced (a `.wfx` file).
fn printed_body(e: &ExampleDecl, program: &Program) -> String {
    let whole = print::declaration(&Declaration::Example(e.clone()), program);
    let mut lines: Vec<&str> = whole.lines().skip_while(|l| l.starts_with("///")).collect();
    if lines.len() >= 2 {
        lines.remove(0);
        lines.pop();
    }
    dedent(&lines.join("\n"))
}

/// The text with the indentation every line shares taken off, and the
/// blank lines at either end.
fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
    let end = lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .map_or(0, |i| i + 1);
    let lines = &lines[start..end.max(start)];
    // A body that starts on the brace's line (`{ state x = 1`) says
    // nothing of its indentation there: the lines after it do.
    let shared = |from: usize| {
        lines
            .iter()
            .skip(from)
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
    };
    let first_inline = start == 0 && text.lines().next().is_some_and(|l| !l.trim().is_empty());
    let indent = if first_inline {
        shared(1).unwrap_or(0)
    } else {
        shared(0).unwrap_or(0)
    };
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if i == 0 && first_inline {
                l.trim()
            } else {
                l.get(indent..).unwrap_or(l.trim_start()).trim_end()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// An example made from a component's props: each one it needs given a
/// value of its type, and those it has a default for left out. None when
/// a prop it needs is of a type no value can be made up for.
fn made(c: &ComponentDecl) -> Option<Drawn> {
    let mut args = Vec::new();
    for p in &c.props {
        if p.default.is_some() || p.optional {
            continue;
        }
        let value = sample(&p.prop_type, &p.name)?;
        if p.positional {
            args.insert(0, value);
        } else {
            args.push(format!("{}: {value}", p.name));
        }
    }
    let call = if args.is_empty() {
        c.name.clone()
    } else {
        format!("{}({})", c.name, args.join(", "))
    };
    let code = if c.slots.iter().any(|s| s.name.is_none())
        || crate::tools::registry_json::uses_children(&c.body)
    {
        format!("{call} {{\n    Text(\"Content\")\n}}")
    } else {
        call
    };
    let source = format!("example {} \"As it comes\" {{\n{code}\n}}\n", c.name);
    let parsed = crate::syntax::parse_source(&source, "<made>").ok()?;
    let Some(Declaration::Example(example)) = parsed.declarations.into_iter().next() else {
        return None;
    };
    Some(Drawn {
        example,
        code,
        made: true,
    })
}

/// A value of `ty` to show a prop with, from its name: `"Title"` for a
/// `title: String`.
fn sample(ty: &TypeRef, name: &str) -> Option<String> {
    Some(match ty {
        TypeRef::String => {
            let mut words = String::new();
            for (i, ch) in name.chars().enumerate() {
                if ch.is_uppercase() && i > 0 {
                    words.push(' ');
                    words.extend(ch.to_lowercase());
                } else if i == 0 {
                    words.extend(ch.to_uppercase());
                } else {
                    words.push(ch);
                }
            }
            format!("\"{}\"", words.replace(['"', '{', '}', '\\'], ""))
        }
        TypeRef::Number => "3".to_string(),
        TypeRef::Bool => "true".to_string(),
        TypeRef::List(_) => "[]".to_string(),
        TypeRef::Optional(_) => "null".to_string(),
        TypeRef::Refined(inner, _) => return sample(inner, name),
        _ => return None,
    })
}

/// What the docs are built over: `docs.json`.
fn model(
    program: &Program,
    files: &[String],
    project: &str,
    examples: &BTreeMap<String, Vec<Value>>,
) -> Value {
    let file = |i: usize| files.get(i).cloned().unwrap_or_default();
    let placed = placed_by(program);
    let mut components = Vec::new();
    let mut types = Vec::new();
    let mut stores = Vec::new();
    let mut pages = Vec::new();
    for (i, d) in program.declarations.iter().enumerate() {
        match d {
            Declaration::Component(c) if !c.name.contains('.') => {
                let doc = c.doc.clone().unwrap_or_default();
                components.push(json!({
                    "name": c.name,
                    "slug": slug(&c.name),
                    "summary": summary(&doc),
                    "doc": doc,
                    "file": file(i),
                    "signature": signature(c),
                    "props": c.props.iter().map(|p| json!({
                        "name": if p.positional { format!("{} (positional)", p.name) } else { format!("{}:", p.name) },
                        "ty": print::type_ref(&p.prop_type),
                        "default": match (&p.default, p.optional) {
                            (Some(v), _) => print::expr(v),
                            (None, true) => "null".to_string(),
                            (None, false) => "required".to_string(),
                        },
                        "doc": p.doc.clone().unwrap_or_default(),
                    })).collect::<Vec<_>>(),
                    "events": c.events.iter().map(|e| json!({
                        "name": e.name,
                        "params": params(e.params.iter().map(|p| (p.name.as_str(), &p.param_type))),
                    })).collect::<Vec<_>>(),
                    "slots": slots_of(c),
                    "parts": c.parts,
                    "examples": examples.get(&c.name).cloned().unwrap_or_default(),
                    "placedBy": placed.get(c.name.as_str()).map(|s| s.iter().collect::<Vec<_>>()).unwrap_or_default(),
                }));
            }
            Declaration::Type(t) => types.push(json!({
                "name": t.name,
                "kind": if t.alias.is_some() { "alias" } else { "record" },
                "doc": t.alias.as_ref().map_or_else(
                    || t.doc.clone().unwrap_or_default(),
                    |a| format!("{}\n\nAnother name for `{}`.", t.doc.clone().unwrap_or_default(), print::type_ref(a)).trim().to_string(),
                ),
                "file": file(i),
                "rows": t.fields.iter().map(|f| json!({
                    "name": f.name,
                    "ty": print::type_ref(&f.ty),
                    "default": f.default.as_ref().map(print::expr).unwrap_or_default(),
                    "doc": f.doc.clone().unwrap_or_default(),
                })).collect::<Vec<_>>(),
            })),
            Declaration::Enum(e) => types.push(json!({
                "name": e.name,
                "kind": "enum",
                "doc": e.doc.clone().unwrap_or_default(),
                "file": file(i),
                "rows": e.cases.iter().map(|c| json!({
                    "name": format!(".{}", c.name),
                    "ty": if c.fields.is_empty() { "—".to_string() } else {
                        c.fields.iter().map(|f| format!("{}: {}", f.name, print::type_ref(&f.ty))).collect::<Vec<_>>().join(", ")
                    },
                    "default": "",
                    "doc": c.status.map(|s| format!("Answered with `{s}`")).unwrap_or_default(),
                })).collect::<Vec<_>>(),
            })),
            Declaration::Store(s) => stores.push(json!({
                "name": s.name,
                "doc": "",
                "file": file(i),
                "members": members(&s.body),
            })),
            Declaration::Page(p) => pages.push(json!({
                "name": p.name,
                "path": p.path,
                "title": p.title.clone().unwrap_or_default(),
                "file": file(i),
            })),
            _ => {}
        }
    }
    pages.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    json!({
        "project": project,
        "version": crate::VERSION,
        "components": components,
        "types": types,
        "stores": stores,
        "pages": pages,
    })
}

/// A store's state, derived values and actions, each with its type or
/// what it takes.
fn members(body: &[Statement]) -> Vec<Value> {
    body.iter()
        .filter_map(|s| match &s.kind {
            StatementKind::State(st) => Some(json!({
                "name": st.name,
                "ty": if st.persist { "persist" } else { "state" },
                "default": print::expr(&st.value),
                "doc": joined(&[st.ty.as_ref().map(|t| format!("`{}`", print::type_ref(t))), st.doc.clone()], " · "),
            })),
            StatementKind::Derived(d) => Some(json!({
                "name": d.name,
                "ty": "derived",
                "default": "",
                "doc": d.doc.clone().unwrap_or_default(),
            })),
            StatementKind::Action(a) => Some(json!({
                "name": format!("{}({})", a.name, params(a.params.iter().map(|p| (p.name.as_str(), &p.param_type)))),
                "ty": "action",
                "default": "",
                "doc": joined(&[a.returns.as_ref().map(|t| format!("Gives `{}`.", print::type_ref(t))), a.doc.clone()], " "),
            })),
            _ => None,
        })
        .collect()
}

/// Its slots, the default one (`children`) among them when its body
/// places one without declaring it.
fn slots_of(c: &ComponentDecl) -> Vec<Value> {
    let mut slots: Vec<Value> = c
        .slots
        .iter()
        .map(|s| {
            json!({
                "name": s.name.clone().unwrap_or_else(|| "children".to_string()),
                "params": params(s.params.iter().map(|p| (p.name.as_str(), &p.param_type))),
            })
        })
        .collect();
    if !c.slots.iter().any(|s| s.name.is_none())
        && crate::tools::registry_json::uses_children(&c.body)
    {
        slots.insert(0, json!({ "name": "children", "params": "" }));
    }
    slots
}

fn joined(parts: &[Option<String>], by: &str) -> String {
    parts.iter().flatten().cloned().collect::<Vec<_>>().join(by)
}

fn params<'a>(params: impl Iterator<Item = (&'a str, &'a TypeRef)>) -> String {
    params
        .map(|(n, t)| format!("{n}: {}", print::type_ref(t)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The component's header, as written: `UserCard(_ name: String, …)`.
fn signature(c: &ComponentDecl) -> String {
    let props: Vec<String> = c
        .props
        .iter()
        .map(|p| {
            let mut s = format!(
                "{}{}: {}",
                if p.positional { "_ " } else { "" },
                p.name,
                print::type_ref(&p.prop_type)
            );
            if let Some(d) = &p.default {
                s.push_str(&format!(" = {}", print::expr(d)));
            }
            s
        })
        .collect();
    if props.is_empty() {
        format!("component {}", c.name)
    } else {
        format!("component {}({})", c.name, props.join(", "))
    }
}

/// A doc comment's first sentence, for a card.
fn summary(doc: &str) -> String {
    let flat = doc.split_whitespace().collect::<Vec<_>>().join(" ");
    match flat.find(". ") {
        Some(at) => flat[..=at].to_string(),
        None => flat,
    }
}

/// Which pages and components place each component.
fn placed_by(program: &Program) -> BTreeMap<&str, BTreeSet<String>> {
    fn visit<'a>(body: &'a [Statement], out: &mut BTreeSet<&'a str>) {
        for s in body {
            if let StatementKind::UIElement(el) = &s.kind
                && let crate::syntax::parser::ast::ComponentRef::UserDefined(name) = &el.component
            {
                out.insert(name.as_str());
            }
            for inner in crate::syntax::parser::walk::child_bodies(s) {
                visit(inner, out);
            }
        }
    }
    let mut placed: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for d in &program.declarations {
        let owner = match d {
            Declaration::Page(p) => p.name.clone(),
            Declaration::Component(c) => c.name.clone(),
            Declaration::App(_) => "app".to_string(),
            _ => continue,
        };
        let mut names = BTreeSet::new();
        visit(crate::syntax::parser::walk::body_of(d), &mut names);
        for n in names {
            if n != owner {
                placed.entry(n).or_default().insert(owner.clone());
            }
        }
    }
    placed
}

/// `UserCard` → `user-card`.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('-');
        }
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_a_slug_of_its_words() {
        assert_eq!(slug("UserCard"), "user-card");
        assert_eq!(slug("A"), "a");
    }

    #[test]
    fn an_example_s_source_loses_the_indentation_it_shares() {
        assert_eq!(
            dedent("\n    UserCard(\"Sam\")\n        Text(\"x\")\n"),
            "UserCard(\"Sam\")\n    Text(\"x\")"
        );
    }

    #[test]
    fn a_body_that_starts_on_the_brace_s_line_is_indented_by_the_rest() {
        assert_eq!(
            dedent(" state here = false\n    UserCard(\"Ada\")\n"),
            "state here = false\nUserCard(\"Ada\")"
        );
    }

    #[test]
    fn a_prop_s_name_makes_its_sample() {
        assert_eq!(
            sample(&TypeRef::String, "fullName").as_deref(),
            Some("\"Full name\"")
        );
        assert_eq!(sample(&TypeRef::Named("User".into()), "user"), None);
    }

    /// The kit is the reference's design: its theme, stylesheet and fonts
    /// are the site's own, byte for byte.
    #[test]
    fn the_kit_wears_the_reference_s_design() {
        let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("site");
        if !site.is_dir() {
            return;
        }
        for (kit, theirs) in [
            (include_str!("kit/theme.wf"), "src/theme.wf"),
            (include_str!("kit/blueprint.css"), "src/blueprint.css"),
        ] {
            let theirs = std::fs::read_to_string(site.join(theirs)).unwrap();
            assert!(
                kit == theirs,
                "src/tools/docs/kit is behind site/{theirs}: copy it again"
            );
        }
        for (path, bytes) in FONTS {
            let name = path.trim_start_matches("public/");
            let theirs = std::fs::read(site.join("public").join(name)).unwrap();
            assert!(theirs == *bytes, "the kit's {name} is not the site's");
        }
    }
}
