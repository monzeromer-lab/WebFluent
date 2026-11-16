//! Data that comes in from outside the program, held to the type that
//! receives it where it comes in: a `data` file when the project is built,
//! a template's data before it renders, a test's seeds before it runs. Each
//! names the place it goes wrong (`T33`).

use serde_json::json;
use std::path::Path;
use std::process::Command;
use webfluent::Template;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-boundaries-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "scratch", "build": { "output": "build" } }"#,
    )
    .unwrap();
    dir
}

fn wf(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("wf runs");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

const POSTS: &str = "type Post { slug: String, title: String, date: Date }\ndata posts: [Post] = \"posts.json\"\napp { Router }\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Posts\").h1\n    for p in posts by p.slug { Text(\"{p.title} {p.date}\") }\n}\n";

#[test]
fn a_data_file_is_held_to_its_type_and_the_finding_names_the_place() {
    let dir = scratch("data");
    std::fs::write(dir.join("src/App.wf"), POSTS).unwrap();
    let mut posts: Vec<serde_json::Value> = (1..=5)
        .map(|n| json!({ "slug": format!("p{n}"), "title": format!("Post {n}"), "date": format!("2026-03-0{n}") }))
        .collect();
    std::fs::write(
        dir.join("posts.json"),
        serde_json::to_string(&posts).unwrap(),
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["check"]);
    assert!(ok, "a file of its type checks: {out}");
    posts[3]["date"] = json!("March 5");
    std::fs::write(
        dir.join("posts.json"),
        serde_json::to_string(&posts).unwrap(),
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["check"]);
    assert!(!ok, "{out}");
    assert!(
        out.contains("error[T33]") && out.contains("$[3].date is \"March 5\", not a Date"),
        "{out}"
    );
    assert!(out.contains("src/App.wf:2:1"), "at the declaration: {out}");
    // Reported once: nothing that reads `posts` is reported with it.
    assert_eq!(out.matches("error[").count(), 1, "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

const INVOICE: &str = "type Customer { name: String }\ntype Line { name: String, price: Number }\n\npage Invoice(path: \"/\", title: \"Invoice {number}\", number: String, customer: Customer, items: [Line], paid: Bool?) {\n    Heading(\"Invoice #{number}\").h1\n    Text(\"For {customer.name}\")\n    for item in items { Text(\"{item.name}: {item.price}\") }\n    if paid == true { Badge(\"PAID\").success }\n}\n";

#[test]
fn a_template_that_says_what_it_takes_is_given_that_or_refuses_to_render() {
    let tpl = Template::from_str(INVOICE).expect("checks");
    let good = json!({ "number": "7", "customer": { "name": "Acme" }, "items": [{ "name": "Widget", "price": 3 }] });
    let html = tpl.render_html_fragment(&good).expect("renders");
    assert!(
        html.contains("Invoice #7") && html.contains("Widget: 3"),
        "{html}"
    );
    let bad = json!({ "number": "7", "customer": { "name": "Acme" }, "items": [{ "name": "Widget", "price": 3 }, { "name": "Gadget" }] });
    let err = tpl
        .render_html_fragment(&bad)
        .expect_err("a line with no price")
        .to_string();
    assert!(
        err.contains("T33") && err.contains("$.items[1].price is missing"),
        "{err}"
    );
    let err = tpl
        .render_pdf(&json!({ "number": 7, "customer": { "name": "Acme" }, "items": [] }))
        .expect_err("a number for a String")
        .to_string();
    assert!(err.contains("$.number is 7, not a String"), "{err}");
}

#[test]
fn a_name_a_template_with_parameters_does_not_declare_is_t13() {
    let src = INVOICE.replace("Text(\"For {customer.name}\")", "Text(note)");
    let err = Template::from_str(&src)
        .expect_err("`note` is no parameter")
        .to_string();
    assert!(err.contains("T13") && err.contains("`note`"), "{err}");
    // A template that declares nothing still reads whatever it is given.
    let open = Template::from_str("page P(path: \"/\") { Text(note) }").expect("checks");
    let html = open.render_html_fragment(&json!({ "note": "hi" })).unwrap();
    assert!(html.contains("hi"), "{html}");
}

#[test]
fn a_tests_seed_is_held_to_the_state_it_seeds() {
    let dir = scratch("seeds");
    std::fs::write(
        dir.join("src/App.wf"),
        "store Cart { state items: [String] = []  state total = 0 }\napp { Router }\npage Home(path: \"/\", title: \"Home\", description: \"d\") { Heading(\"H\").h1 }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/seeds.wf"),
        "test \"counts\"(data: { count: \"two\", Cart: { items: [1], totl: 3 }, label: \"free\" }) {\n    state count = 0\n    use Cart\n    Text(\"{count} {Cart.items.length} {label}\")\n    expect \"two\"\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["test"]);
    assert!(!ok, "{out}");
    assert!(
        out.contains("seeds `count` with what it cannot hold: count is \"two\", not a Number"),
        "{out}"
    );
    assert!(out.contains("Cart.items[0] is 1, not a String"), "{out}");
    assert!(
        out.contains("error[T05]") && out.contains("`Cart` has no state `totl`"),
        "{out}"
    );
    // `label` seeds no state: it is data the body reads by name.
    assert!(!out.contains("`label`"), "{out}");
    std::fs::write(
        dir.join("tests/seeds.wf"),
        "test \"counts\"(data: { count: 2, Cart: { items: [\"a\"] } }) {\n    state count = 0\n    use Cart\n    Text(\"{count} {Cart.items.length}\")\n    expect \"2 1\"\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["test"]);
    assert!(ok, "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}
