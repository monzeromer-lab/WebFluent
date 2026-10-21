//! The smallest edit, over every project in the repository (and, with
//! `WF_PRINT_CORPUS`, more): for every element of every file, changing
//! nothing writes nothing; changing an argument, a flag, a style value,
//! taking a child out or putting one in changes only the bytes of that
//! piece, and the file reads back as the change asked for. Every `if`,
//! `for` and `show` has its head changed the same way, and every page's
//! body a statement added and one taken out.

use std::path::{Path, PathBuf};

use webfluent::edit::{rewrite_body, rewrite_element, rewrite_statement};
use webfluent::parser::ast::*;

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if p.is_dir() {
            if !matches!(
                name.as_str(),
                "build" | "node_modules" | "target" | ".wf-cache" | ".git" | "corpus"
            ) {
                sources(&p, out);
            }
        } else if name.ends_with(".wf") || name.ends_with(".wfx") {
            out.push(p);
        }
    }
}

/// The bytes `after` changed of `before`: the part between what the two
/// share at the front and at the back.
fn changed(before: &str, after: &str) -> (usize, usize) {
    let (a, b) = (before.as_bytes(), after.as_bytes());
    let front = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let back = a[front..]
        .iter()
        .rev()
        .zip(b[front..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    (front, a.len() - back)
}

fn text(s: &str) -> Expr {
    Expr::StringLiteral(s.to_string())
}

fn text_element(s: &str) -> Statement {
    Statement::new(
        StatementKind::UIElement(UIElement {
            component: ComponentRef::BuiltIn("Text".into()),
            args: vec![Arg::Positional(text(s))],
            modifiers: Vec::new(),
            children: Vec::new(),
            style_block: None,
            transition_block: None,
            events: Vec::new(),
            slot_fills: Vec::new(),
            span: Span::dummy(),
            paren_span: None,
            body_span: None,
            style_span: None,
            arg_spans: Vec::new(),
            modifier_spans: Vec::new(),
        }),
        Span::dummy(),
    )
}

struct Tally {
    edits: usize,
    failures: Vec<String>,
}

fn check(
    t: &mut Tally,
    file: &str,
    what: &str,
    line: u32,
    before: &str,
    result: webfluent::error::Result<String>,
    within: (usize, usize),
) {
    t.edits += 1;
    match result {
        Err(e) => t.failures.push(format!("{file}:{line} {what}: {e}")),
        Ok(after) => {
            if after == before {
                return;
            }
            // Everything before the piece and everything after it is
            // where it was, byte for byte.
            let (w0, w1) = (within.0.min(before.len()), within.1.min(before.len()));
            let kept = after.starts_with(&before[..w0])
                && after.len() >= before.len() - w1
                && after.ends_with(&before[w1..]);
            let (s, e) = changed(before, &after);
            if !kept {
                t.failures.push(format!(
                    "{file}:{line} {what}: changed {s}..{e}, outside {}..{}\n  was: {}\n  now: {}",
                    within.0,
                    within.1,
                    before[s.min(before.len())..e.min(before.len())]
                        .lines()
                        .next()
                        .unwrap_or(""),
                    after[s.min(after.len())..].lines().next().unwrap_or("")
                ));
            }
        }
    }
}

fn line_range(source: &str, start: usize, end: usize) -> (usize, usize) {
    let s = source[..start].rfind('\n').unwrap_or(0);
    let e = source[end..]
        .find('\n')
        .map_or(source.len(), |n| end + n + 1);
    (s, e)
}

fn prove_file(path: &Path, root: &Path) -> Tally {
    let mut t = Tally {
        edits: 0,
        failures: Vec::new(),
    };
    let file = path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string();
    let source = std::fs::read_to_string(path).unwrap();
    let name = path.to_string_lossy().to_string();
    let braced = name.ends_with(".wf");
    let Ok(program) = webfluent::syntax::parse_source(
        &if braced {
            source.clone()
        } else {
            webfluent::layout::to_braces(&source, &name).unwrap_or_default()
        },
        "x.wf",
    ) else {
        return t;
    };
    let whole = (0, source.len() + 1);

    // Every element.
    let mut nodes: Vec<(String, UIElement)> = Vec::new();
    webfluent::codegen::node_id::visit_nodes(&program, &mut |ui, id, _| {
        nodes.push((id.to_string(), ui.clone()))
    });
    // A debug build edits some forty elements of a file, spread through
    // it: every edit reads the whole file again, and a debug build is slow.
    if cfg!(debug_assertions) && nodes.len() > 40 {
        let step = nodes.len().div_ceil(40);
        nodes = nodes.into_iter().step_by(step).collect();
    }
    for (id, ui) in &nodes {
        let line = ui.span.line;
        // The places are the braced spelling's; a `.wfx` file is held only
        // to reading back, and to changing nothing when nothing changes.
        let span = |s: &Span| {
            if braced {
                (s.start as usize, s.end as usize)
            } else {
                whole
            }
        };
        let el = span(&ui.span);
        let identity = rewrite_element(&source, &name, id, |_| {});
        t.edits += 1;
        match identity {
            Ok(out) if out == source => {}
            Ok(_) => t
                .failures
                .push(format!("{file}:{line} nothing changed, but bytes moved")),
            Err(e) => t
                .failures
                .push(format!("{file}:{line} nothing changed: {e}")),
        }
        if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Children" || n.starts_with("__"))
        {
            continue;
        }
        // An argument.
        if let Some(i) = ui.args.iter().position(|a| matches!(a, Arg::Named(..))) {
            let within = ui.arg_spans.get(i).map(span).unwrap_or(el);
            let r = rewrite_element(&source, &name, id, |e| {
                if let Arg::Named(_, v) = &mut e.args[i] {
                    *v = text("changed");
                }
            });
            check(&mut t, &file, "an argument", line, &source, r, within);
        }
        // A flag.
        let header_end = ui
            .body_span
            .map(|b| b.start as usize)
            .unwrap_or(ui.span.end as usize);
        let r = rewrite_element(&source, &name, id, |e| e.modifiers.push("zz".into()));
        check(
            &mut t,
            &file,
            "a flag",
            line,
            &source,
            r,
            if braced { (el.0, header_end) } else { whole },
        );
        // A style value.
        if let Some(style) = &ui.style_block
            && let Some(p) = style.properties.first()
        {
            let r = rewrite_element(&source, &name, id, |e| {
                e.style_block.as_mut().unwrap().properties[0].value = text("1px");
            });
            check(
                &mut t,
                &file,
                "a style value",
                line,
                &source,
                r,
                span(&p.value_span),
            );
            let r = rewrite_element(&source, &name, id, |e| {
                e.style_block
                    .as_mut()
                    .unwrap()
                    .properties
                    .push(StyleProperty {
                        name: "outline".into(),
                        value: text("none"),
                        span: Span::dummy(),
                        value_span: Span::dummy(),
                    });
            });
            check(
                &mut t,
                &file,
                "a style declaration added",
                line,
                &source,
                r,
                span(&ui.style_span.unwrap()),
            );
        }
        // A child out, and one in.
        if let Some(last) = ui.children.last() {
            let (s, e) = (last.span.start as usize, last.span.end as usize);
            let within = if braced {
                line_range(&source, s, e)
            } else {
                whole
            };
            let r = rewrite_element(&source, &name, id, |x| {
                x.children.pop();
            });
            check(&mut t, &file, "a child taken out", line, &source, r, within);
            let r = rewrite_element(&source, &name, id, |x| {
                x.children.push(text_element("added"))
            });
            check(
                &mut t,
                &file,
                "a child added",
                line,
                &source,
                r,
                if braced { (e, el.1) } else { whole },
            );
            let r = rewrite_element(&source, &name, id, |x| {
                x.children.insert(0, text_element("first"))
            });
            check(&mut t, &file, "a child added first", line, &source, r, el);
        } else if ui.body_span.is_some() || ui.events.is_empty() {
            let r = rewrite_element(&source, &name, id, |x| {
                x.children.push(text_element("added"))
            });
            check(&mut t, &file, "a first child", line, &source, r, el);
        }
    }

    // Every `if`, `for` and `show`: its head.
    if braced {
        let mut heads: Vec<(usize, usize, u32, usize)> = Vec::new();
        webfluent::print::each_statement(&program, &mut |stmts, i, _| {
            let s = &stmts[i];
            let (start, end) = (s.span.start as usize, s.span.end as usize);
            if let StatementKind::If(_) | StatementKind::For(_) | StatementKind::Show(_) = &s.kind {
                let open = webfluent::reprint::blocks_in(&source, start, end)
                    .first()
                    .map(|b| b.0)
                    .unwrap_or(end);
                heads.push((start, end, s.span.line, open));
            }
        });
        for (start, end, line, open) in heads {
            let r = rewrite_statement(&source, &name, start, end, |s| match &mut s.kind {
                StatementKind::If(i) => i.condition = Expr::Identifier("zz".into()),
                StatementKind::For(f) => f.iterable = Expr::Identifier("zz".into()),
                StatementKind::Show(x) => x.condition = Expr::Identifier("zz".into()),
                _ => {}
            });
            check(&mut t, &file, "a head", line, &source, r, (start, open));
        }
        // Every declaration: changed in its header, its members, its
        // `///`; taken out; and one added after the last.
        let identity = webfluent::edit::rewrite_program(&source, &name, |_| {});
        t.edits += 1;
        match identity {
            Ok(out) if out == source => {}
            Ok(_) => t
                .failures
                .push(format!("{file} the file, nothing changed, but bytes moved")),
            Err(e) => t
                .failures
                .push(format!("{file} the file, nothing changed: {e}")),
        }
        let end_of_last = program
            .declarations
            .iter()
            .filter_map(|d| webfluent::print::declaration_span(&source, d))
            .map(|s| s.1)
            .max()
            .unwrap_or(0);
        let r = webfluent::edit::rewrite_program(&source, &name, |p| {
            p.declarations.push(Declaration::Const(ConstDecl {
                name: "ADDED".into(),
                ty: None,
                value: text("x"),
                doc: Some("Added.".into()),
                span: Span::dummy(),
            }))
        });
        check(
            &mut t,
            &file,
            "a declaration added",
            0,
            &source,
            r,
            (end_of_last, source.len()),
        );
        for d in &program.declarations {
            let Some(dname) = webfluent::edit::declaration_name(d).map(str::to_string) else {
                continue;
            };
            if program
                .declarations
                .iter()
                .filter(|x| webfluent::edit::declaration_name(x) == Some(dname.as_str()))
                .count()
                > 1
            {
                continue;
            }
            let Some((ds, de)) = webfluent::print::declaration_span(&source, d) else {
                continue;
            };
            let line = source[..ds].matches('\n').count() as u32 + 1;
            match d {
                Declaration::Component(c) if !c.name.contains('.') => {
                    let header = (c.header_span.start as usize, c.header_span.end as usize);
                    let r = webfluent::edit::rewrite_declaration(&source, &name, &dname, |d| {
                        if let Declaration::Component(c) = d {
                            c.props.push(PropDecl {
                                name: "zz".into(),
                                prop_type: TypeRef::String,
                                optional: false,
                                default: Some(text("x")),
                                positional: false,
                                doc: None,
                                span: Span::dummy(),
                            });
                        }
                    });
                    check(&mut t, &file, "a prop added", line, &source, r, header);
                    if let Some(first) = c.props.first().filter(|p| p.doc.is_none()) {
                        let r = webfluent::edit::rewrite_declaration(&source, &name, &dname, |d| {
                            if let Declaration::Component(c) = d {
                                c.props[0].default = Some(text("changed"));
                            }
                        });
                        check(
                            &mut t,
                            &file,
                            "a prop's default",
                            line,
                            &source,
                            r,
                            (first.span.start as usize, first.span.end as usize),
                        );
                    }
                    let r = webfluent::edit::rewrite_declaration(&source, &name, &dname, |d| {
                        if let Declaration::Component(c) = d {
                            c.doc = Some("What it is.\n\nAnd more.".into());
                        }
                    });
                    check(
                        &mut t,
                        &file,
                        "a component's doc",
                        line,
                        &source,
                        r,
                        (ds, c.span.start as usize),
                    );
                    let r = webfluent::edit::rewrite_declaration(&source, &name, &dname, |d| {
                        if let Declaration::Component(c) = d {
                            c.events.push(EventDecl {
                                name: "picked".into(),
                                params: vec![ParamDecl {
                                    name: "id".into(),
                                    param_type: TypeRef::String,
                                }],
                                doc: Some("When one is picked.".into()),
                                span: Span::dummy(),
                            });
                        }
                    });
                    check(
                        &mut t,
                        &file,
                        "an event added",
                        line,
                        &source,
                        r,
                        (c.body_span.start as usize, c.body_span.end as usize),
                    );
                }
                Declaration::Page(p) => {
                    let r = webfluent::edit::rewrite_declaration(&source, &name, &dname, |d| {
                        if let Declaration::Page(p) = d {
                            p.title = Some("Changed".into());
                            p.title_expr = None;
                        }
                    });
                    check(
                        &mut t,
                        &file,
                        "a page's title",
                        line,
                        &source,
                        r,
                        (p.header_span.start as usize, p.header_span.end as usize),
                    );
                }
                Declaration::Type(ty) => {
                    let r = webfluent::edit::rewrite_declaration(&source, &name, &dname, |d| {
                        if let Declaration::Type(t) = d {
                            t.fields.push(FieldDecl {
                                name: "added".into(),
                                ty: TypeRef::Optional(Box::new(TypeRef::String)),
                                default: None,
                                doc: None,
                                span: Span::dummy(),
                            });
                        }
                    });
                    check(
                        &mut t,
                        &file,
                        "a field added",
                        line,
                        &source,
                        r,
                        (ty.span.start as usize, ty.span.end as usize),
                    );
                }
                Declaration::Theme(th) if !th.tokens.is_empty() => {
                    let tok = &th.tokens[0];
                    let r = webfluent::edit::rewrite_declaration(&source, &name, &dname, |d| {
                        if let Declaration::Theme(t) = d {
                            t.tokens[0].value = text("1px");
                        }
                    });
                    check(
                        &mut t,
                        &file,
                        "a token's value",
                        line,
                        &source,
                        r,
                        (tok.span.start as usize, tok.span.end as usize),
                    );
                }
                _ => {}
            }
            if matches!(d, Declaration::Component(c) if c.name.contains('.')) {
                continue;
            }
            let r = webfluent::edit::rewrite_program(&source, &name, |p| {
                p.declarations
                    .retain(|x| webfluent::edit::declaration_name(x) != Some(dname.as_str()))
            });
            // Its own lines, and a blank line beside them.
            let from = source[..ds].trim_end_matches(['\n', ' ']).len();
            let to = de + source[de..].len() - source[de..].trim_start_matches(['\n', ' ']).len();
            check(
                &mut t,
                &file,
                "a declaration taken out",
                line,
                &source,
                r,
                (from, to),
            );
        }
        // Every page's body.
        for d in &program.declarations {
            let (Declaration::Page(PageDecl { name: n, body, .. })
            | Declaration::Component(ComponentDecl { name: n, body, .. })) = d
            else {
                continue;
            };
            let r = rewrite_body(&source, &name, n, |b| b.push(text_element("added")));
            check(
                &mut t,
                &file,
                "a statement added to a body",
                0,
                &source,
                r,
                whole,
            );
            if let Some(first) = body.first() {
                let (s, e) = (first.span.start as usize, first.span.end as usize);
                // A destructuring `let` is several statements over one span.
                if body.get(1).is_some_and(|b| b.span == first.span) {
                    continue;
                }
                let r = rewrite_body(&source, &name, n, |b| {
                    b.remove(0);
                });
                check(
                    &mut t,
                    &file,
                    "a statement taken out of a body",
                    first.span.line,
                    &source,
                    r,
                    line_range(&source, s, e),
                );
            }
        }
    }
    t
}

#[test]
fn every_edit_changes_only_its_own_bytes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in [
        "examples",
        "site",
        "tests/fixtures",
        "editors/zed/grammars/webfluent/examples",
        "editors/tree-sitter-webfluentx/test/guide",
    ] {
        sources(&root.join(dir), &mut files);
    }
    if let Ok(more) = std::env::var("WF_PRINT_CORPUS") {
        for dir in more.split(':').filter(|d| !d.is_empty()) {
            sources(Path::new(dir), &mut files);
        }
    }
    // A debug build edits every fourth file; CI runs the whole of it in
    // release.
    if cfg!(debug_assertions) {
        files = files.into_iter().step_by(4).collect();
    }
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunks: Vec<Vec<PathBuf>> = (0..threads)
        .map(|k| files.iter().skip(k).step_by(threads).cloned().collect())
        .collect();
    let tallies: Vec<Tally> = std::thread::scope(|s| {
        let handles: Vec<_> = chunks
            .iter()
            .map(|chunk| {
                s.spawn(move || {
                    let mut all = Tally {
                        edits: 0,
                        failures: Vec::new(),
                    };
                    for f in chunk {
                        let t = prove_file(f, root);
                        all.edits += t.edits;
                        all.failures.extend(t.failures);
                    }
                    all
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let edits: usize = tallies.iter().map(|t| t.edits).sum();
    let failures: Vec<String> = tallies.into_iter().flat_map(|t| t.failures).collect();
    eprintln!(
        "{} files, {edits} edits, {} wrong",
        files.len(),
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "{} edits were wrong:\n{}",
        failures.len(),
        failures
            .iter()
            .take(50)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        edits
            > if cfg!(debug_assertions) {
                3_000
            } else {
                40_000
            },
        "{edits} edits"
    );
}
