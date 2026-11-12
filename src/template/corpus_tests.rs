//! The template engine's expressions, held to a snapshot: every line of
//! `tests/eval-corpus/expressions.txt`, evaluated over `data.json`, with the
//! value and the text a page would show. A change to what an expression
//! means shows up here as a diff to read, not as a page that renders
//! differently. `UPDATE_EVAL_CORPUS=1` rewrites the snapshot.

use super::*;

const PRELUDE: &str = "const dbl = (x) => x * 2\nconst LIMIT = 3\n";

fn corpus_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/eval-corpus")
}

/// What one line comes to: the value as JSON and the text it shows.
fn evaluate(line: &str, data: &Value) -> String {
    let source = format!("{PRELUDE}const __e = {line}\n");
    let Ok(program) = crate::syntax::parser::v2::parse_v2(&source, "corpus.wf") else {
        return "does not parse".to_string();
    };
    let Some(expr) = program.declarations.iter().find_map(|d| match d {
        Declaration::Const(c) if c.name == "__e" => Some(c.value.clone()),
        _ => None,
    }) else {
        return "no expression".to_string();
    };
    let ctx = RenderContext::for_program(&program, data);
    let value = ctx.eval_expr(&expr);
    format!(
        "{}  |{}|",
        serde_json::to_string(&value).unwrap_or_default(),
        value_to_string(&value)
    )
}

#[test]
fn every_expression_means_what_the_snapshot_says() {
    let dir = corpus_dir();
    let data: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("data.json")).unwrap()).unwrap();
    let mut out = String::new();
    for line in std::fs::read_to_string(dir.join("expressions.txt"))
        .unwrap()
        .lines()
    {
        if line.trim().is_empty() || line.starts_with('#') {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        out.push_str(&format!("{line}\n    => {}\n", evaluate(line, &data)));
    }
    let snap = dir.join("template.snap");
    if std::env::var_os("UPDATE_EVAL_CORPUS").is_some() {
        std::fs::write(&snap, &out).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&snap).unwrap_or_default();
    if want != out {
        let changed: Vec<String> = want
            .lines()
            .zip(out.lines())
            .filter(|(a, b)| a != b)
            .map(|(a, b)| format!("  was {a}\n  now {b}"))
            .collect();
        panic!(
            "what an expression means changed; if that is the point, rerun with UPDATE_EVAL_CORPUS=1:\n{}",
            changed.join("\n")
        );
    }
}

/// A component handed a long list, drawing a row per item: each value in
/// a row must not cost the length of the list again. Run with
/// `--ignored --nocapture` to see the time.
#[test]
#[ignore = "a timing, not a check"]
fn a_long_list_through_a_component_renders_in_linear_time() {
    let source = r#"
component Board(rows: [Map]) {
    for r in rows {
        Row { Text(r.title) Text("{r.score} points") Badge(if r.done { "done" } else { "open" }) }
    }
}
page P(path: "/") { Board(rows: rows) }
"#;
    let tpl = Template::from_str(source).unwrap();
    for n in [500usize, 1000, 2000] {
        let rows: Vec<Value> = (0..n)
            .map(|i| serde_json::json!({ "title": format!("Row {i}"), "score": i, "done": i % 2 == 0 }))
            .collect();
        let data = serde_json::json!({ "rows": rows });
        let started = std::time::Instant::now();
        let html = tpl.render_html_fragment(&data).unwrap();
        println!("{n} rows: {:?} ({} bytes)", started.elapsed(), html.len());
        if n == 500 {
            println!("{}", &html[..html.len().min(400)]);
        }
    }
}

/// The same list, with a component placed once per row: placing one must
/// not cost the length of the list either.
#[test]
#[ignore = "a timing, not a check"]
fn a_component_per_row_renders_in_linear_time() {
    let source = r#"
component Line(_ r: Map) { Row { Text(r.title) Text("{r.score} points") } }
component Board(rows: [Map]) { for r in rows { Line(r) } }
page P(path: "/") { Board(rows: rows) }
"#;
    let tpl = Template::from_str(source).unwrap();
    for n in [500usize, 1000, 2000] {
        let rows: Vec<Value> = (0..n)
            .map(|i| serde_json::json!({ "title": format!("Row {i}"), "score": i }))
            .collect();
        let data = serde_json::json!({ "rows": rows });
        let started = std::time::Instant::now();
        let html = tpl.render_html_fragment(&data).unwrap();
        println!(
            "{n} rows, a component each: {:?} ({} bytes)",
            started.elapsed(),
            html.len()
        );
    }
}
