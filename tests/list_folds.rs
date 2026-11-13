//! `reduce` is worked out at build time as the browser works it out, and a
//! component or store member only a `test` reaches is not unused.

use std::path::{Path, PathBuf};
use std::process::Command;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-folds-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "t" }"#).unwrap();
    for (path, text) in files {
        let at = dir.join(path);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    dir
}

fn wf(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

/// A `derived` made with `reduce` rendered as nothing, while the same sum
/// with `map(…).sum()` rendered: the build-time evaluator had no `reduce`.
#[test]
fn a_template_folds_a_list_with_reduce() {
    let tpl = webfluent::Template::from_str(
        "page P(path: \"/\") {\n\
         \x20   state items = [{ qty: 2 }, { qty: 3 }, { qty: 4 }]\n\
         \x20   derived total = items.reduce((n, i) => n + i.qty, 0)\n\
         \x20   derived product = items.map(i => i.qty).reduce((a, b) => a * b)\n\
         \x20   derived order = [\"a\", \"b\", \"c\"].reduceRight((s, x) => s + x)\n\
         \x20   derived indexed = items.reduce((s, i, ix) => s + ix, 10)\n\
         \x20   Text(\"total {total}, product {product}, order {order}, indexed {indexed}\")\n\
         }\n",
    )
    .unwrap();
    let html = tpl.render_html_fragment(&serde_json::json!({})).unwrap();
    assert!(
        html.contains("total 9, product 24, order cba, indexed 13"),
        "{html}"
    );
}

/// `wf test` renders a store's `derived` made with `reduce`; and the
/// component and the store member only that test reaches are not U03 / U04,
/// whose fix would have removed them and broken the test.
#[test]
fn wf_test_renders_reduce_and_what_a_test_reaches_is_used() {
    let dir = project(
        "test",
        &[
            (
                "src/App.wf",
                "page Home(path: \"/\", title: \"H\", description: \"D.\") { Heading(\"H\").h1 }\n",
            ),
            (
                "src/stores/cart.wf",
                "store Cart {\n\
                 \x20   state items: [Map] = [{ qty: 2 }, { qty: 3 }]\n\
                 \x20   derived total = items.reduce((n, i) => n + i.qty, 0)\n\
                 }\n",
            ),
            (
                "src/components/Total.wf",
                "component Total(_ n: Number) { Text(\"Sum {n}\") }\n\
                 component Inline(_ n: Number) { Text(\"Inline {n}\") }\n\
                 test \"a test beside the component\" {\n\
                 \x20   Inline(2)\n\
                 \x20   expect \"Inline 2\"\n\
                 }\n",
            ),
            (
                "tests/cart.wf",
                "test \"the cart totals\" {\n\
                 \x20   use Cart\n\
                 \x20   Total(Cart.total)\n\
                 \x20   expect \"Sum 5\"\n\
                 }\n",
            ),
        ],
    );
    let (ok, text) = wf(&dir, &["test"]);
    assert!(ok, "{text}");
    assert!(text.contains("2 passed"), "{text}");

    let (_, text) = wf(&dir, &["check"]);
    assert!(!text.contains("U03"), "{text}");
    assert!(!text.contains("U04"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A component nothing places — not even a test — is still U03.
#[test]
fn a_component_no_test_places_is_still_unused() {
    let dir = project(
        "unused",
        &[
            (
                "src/App.wf",
                "page Home(path: \"/\", title: \"H\", description: \"D.\") { Heading(\"H\").h1 }\n\
                 component Lonely { Text(\"x\") }\n",
            ),
            (
                "tests/a.wf",
                "test \"text\" {\n    Text(\"hello\")\n    expect \"hello\"\n}\n",
            ),
        ],
    );
    let (_, text) = wf(&dir, &["check"]);
    assert!(text.contains("U03"), "{text}");
    assert!(text.contains("Lonely"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}
