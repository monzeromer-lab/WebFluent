//! Style values over every project in the repository (and, with
//! `WF_PRINT_CORPUS`, more): every declaration of every style block — on
//! an element, in a nested rule, in a media query — is read into its shape
//! and written back, and what is written reads as the same parts. What a
//! shape cannot hold is kept as written; and of the properties whose shape
//! is not free, nearly all are read into it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use webfluent::parser::ast::*;
use webfluent::style_value::{Shape, Value, parse, print, property};

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
                "build" | "node_modules" | "target" | ".wf-cache" | "corpus"
            ) {
                sources(&p, out);
            }
        } else if name.ends_with(".wf") {
            out.push(p);
        }
    }
}

fn each_style(stmts: &[Statement], f: &mut dyn FnMut(&StyleProperty)) {
    for s in stmts {
        if let StatementKind::UIElement(ui) = &s.kind {
            if let Some(block) = &ui.style_block {
                for p in block
                    .properties
                    .iter()
                    .chain(block.media_queries.iter().flat_map(|m| m.properties.iter()))
                    .chain(block.pseudo_blocks.iter().flat_map(|b| b.properties.iter()))
                {
                    f(p);
                }
            }
            each_style(&ui.children, f);
            for fill in &ui.slot_fills {
                each_style(&fill.body, f);
            }
        }
        for body in s.kind.bodies() {
            each_style(body, f);
        }
    }
}

#[test]
fn every_style_value_reads_into_its_shape_and_back() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in ["examples", "site", "tests"] {
        sources(&root.join(dir), &mut files);
    }
    if let Ok(more) = std::env::var("WF_PRINT_CORPUS") {
        for dir in more.split(':').filter(|d| !d.is_empty()) {
            sources(Path::new(dir), &mut files);
        }
    }
    // What the paged engine computes from a declaration: the same for what
    // was written and what is printed, where it draws the property.
    let styler = webfluent::paged::cascade::Styler::new(
        &webfluent::paged::css::Sheet::parse(webfluent::paged::cascade::UA_SHEET),
        &webfluent::paged::css::Sheet::parse(""),
        webfluent::paged::style::Units {
            em: 12.0,
            rem: 12.0,
            vw: 600.0,
            vh: 800.0,
        },
    );
    let base = webfluent::paged::style::Style::root(12.0, "Sans");
    let computed = |name: &str, value: &str| {
        format!(
            "{:?}",
            styler.apply_all(
                &base,
                &[webfluent::paged::css::Declaration {
                    name: name.to_string(),
                    value: value.to_string(),
                    important: false
                }],
            )
        )
    };
    let mut compared = 0;
    let mut total = 0;
    let mut read: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut free_examples: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut wrong = Vec::new();
    for f in &files {
        let Ok(source) = std::fs::read_to_string(f) else {
            continue;
        };
        let Ok(program) = webfluent::syntax::parse_source(&source, "x.wf") else {
            continue;
        };
        for d in &program.declarations {
            let Some(body) = webfluent::print::body_of(d) else {
                continue;
            };
            each_style(body, &mut |p| {
                total += 1;
                let text = webfluent::print::style_value(&p.value);
                let v = parse(&p.name, &text);
                let printed = print(&p.name, &v);
                let again = parse(&p.name, &printed);
                if again != v {
                    wrong.push(format!("{}: {}: {text} → {printed}", f.display(), p.name));
                }
                let name = webfluent::codegen::style_tokens::canonical_style_prop(&p.name);
                if !matches!(v, Value::Free { .. })
                    && webfluent::paged::cascade::drawn_on_paper(&name)
                    && !text.contains('$')
                    && !text.contains('{')
                {
                    compared += 1;
                    if computed(&name, text.trim()) != computed(&name, &printed) {
                        wrong.push(format!(
                            "{}: {name}: {text} and {printed} are drawn differently",
                            f.display()
                        ));
                    }
                }
                if matches!(v, Value::Free { .. }) && text.trim() != printed {
                    wrong.push(format!(
                        "{}: {}: free, but {text:?} was written as {printed:?}",
                        f.display(),
                        p.name
                    ));
                }
                let shape = property(&p.name).shape;
                if shape != Shape::Free {
                    let e = read.entry(format!("{shape:?}")).or_default();
                    e.0 += 1;
                    if !matches!(v, Value::Free { .. }) {
                        e.1 += 1;
                    } else {
                        let ex = free_examples.entry(format!("{shape:?}")).or_default();
                        if ex.len() < 6 {
                            ex.push(format!("{}: {text}", p.name));
                        }
                    }
                }
            });
        }
    }
    let mut report = Vec::new();
    for (shape, (all, ok)) in &read {
        report.push(format!(
            "{shape}: {ok} of {all}  e.g. {:?}",
            free_examples.get(shape).cloned().unwrap_or_default()
        ));
    }
    eprintln!(
        "{total} declarations, {compared} drawn alike\n{}",
        report.join("\n")
    );
    assert!(
        wrong.is_empty(),
        "{} wrong:\n{}",
        wrong.len(),
        wrong
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(total > 1_500, "{total} declarations");
    assert!(compared > 1_000, "{compared} compared on paper");
    let (all, ok) = read.values().fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    assert!(ok * 100 >= all * 90, "{ok} of {all} read into their shape");
}
