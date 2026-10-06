//! WebFluent as a template engine from Rust: the public API a server or a
//! batch job uses, held to what it promises — data of any `Serialize`
//! type, templates spread over files, one page of several, a document with
//! a title and a language, PDF settings, a template shared across threads,
//! and data that cannot smuggle script into the page.

use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use webfluent::{PdfConfig, Template};

const INVOICE: &str = r#"
page Invoice(path: "/", title: "Invoice #{number}") {
    Heading("Invoice #{number}").h1
    Text("For {customer.name}")
    for line in lines {
        Text("{line.label}: {format(line.amount, .currency)}")
    }
}
"#;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-template-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn data_is_any_serializable_value() {
    #[derive(Serialize)]
    struct Customer {
        name: String,
    }
    #[derive(Serialize)]
    struct Line {
        label: &'static str,
        amount: f64,
    }
    #[derive(Serialize)]
    struct Invoice {
        number: u32,
        customer: Customer,
        lines: Vec<Line>,
    }
    let tpl = Template::from_str(INVOICE).unwrap();
    let invoice = Invoice {
        number: 7,
        customer: Customer {
            name: "Acme".into(),
        },
        lines: vec![Line {
            label: "Widget",
            amount: 9.99,
        }],
    };
    let html = tpl.render_html_fragment(&invoice).unwrap();
    assert!(
        html.contains("Invoice #7") && html.contains("For Acme"),
        "{html}"
    );
    assert!(html.contains("Widget: $9.99"), "{html}");
    // A `serde_json::Value` still works as it always did.
    let html = tpl
        .render_html_fragment(&json!({ "number": 8, "customer": { "name": "B" }, "lines": [] }))
        .unwrap();
    assert!(html.contains("Invoice #8"), "{html}");
}

#[test]
fn a_template_is_shared_across_threads() {
    fn shareable<T: Send + Sync + Clone + 'static>() {}
    shareable::<Template>();
    let tpl = std::sync::Arc::new(Template::from_str(INVOICE).unwrap());
    let handles: Vec<_> = (0..4)
        .map(|n| {
            let tpl = tpl.clone();
            std::thread::spawn(move || {
                tpl.render_html_fragment(
                    &json!({ "number": n, "customer": { "name": "x" }, "lines": [] }),
                )
                .unwrap()
            })
        })
        .collect();
    for (n, h) in handles.into_iter().enumerate() {
        assert!(h.join().unwrap().contains(&format!("Invoice #{n}")));
    }
}

#[test]
fn components_are_shared_across_sources_and_a_finding_names_its_file() {
    let tpl = Template::from_sources(&[
        (
            "price.wf",
            "component Price(_ amount: Number) { Text(format(amount, .currency)).bold }",
        ),
        (
            "quote.wf",
            r#"page Quote(path: "/") { Heading("Quote").h1  Price(total) }"#,
        ),
    ])
    .unwrap();
    assert!(
        tpl.render_html_fragment(&json!({ "total": 12.5 }))
            .unwrap()
            .contains("$12.50")
    );

    let err = Template::from_sources(&[
        (
            "price.wf",
            "component Price(_ amount: Number) { Text(\"x\") }",
        ),
        ("quote.wf", r#"page Quote(path: "/") { Prise(1) }"#),
    ])
    .err()
    .expect("an unknown component is refused")
    .to_string();
    assert!(err.contains("Prise") && err.contains("quote.wf"), "{err}");
}

#[test]
fn one_page_of_several_is_rendered_by_name() {
    let tpl = Template::from_sources(&[(
        "docs.wf",
        r#"page Invoice(path: "/i", title: "Invoice") { Text("the invoice") }
           page Receipt(path: "/r", title: "Receipt") { Text("the receipt") }"#,
    )])
    .unwrap();
    assert_eq!(tpl.pages(), ["Invoice", "Receipt"]);
    let receipt = tpl
        .page("Receipt")
        .unwrap()
        .render_html_fragment(&json!({}))
        .unwrap();
    assert!(
        receipt.contains("the receipt") && !receipt.contains("the invoice"),
        "{receipt}"
    );
    let err = tpl.page("Refund").err().unwrap().to_string();
    assert!(err.contains("`Invoice`, `Receipt`"), "{err}");
}

#[test]
fn a_document_has_its_page_title_and_language() {
    let tpl = Template::from_str(INVOICE).unwrap();
    let data = json!({ "number": "<9>", "customer": { "name": "x" }, "lines": [] });
    let html = tpl.render_html(&data).unwrap();
    assert!(
        html.contains("<title>Invoice #&lt;9&gt;</title>"),
        "the title, from the data, escaped"
    );
    assert!(html.contains("<html lang=\"en\">"), "{}", &html[..120]);
    let html = tpl.with_lang("ar").render_html(&data).unwrap();
    assert!(
        html.contains("<html lang=\"ar\" dir=\"rtl\">"),
        "{}",
        &html[..120]
    );
}

#[test]
fn pdf_settings_are_the_callers() {
    let tpl = Template::from_str(r#"page P(path: "/") { Text("Hi") }"#).unwrap();
    let a4 = tpl.render_pdf(&json!({})).unwrap();
    let letter = tpl
        .clone()
        .with_pdf(PdfConfig {
            page_size: "Letter".into(),
            ..PdfConfig::default()
        })
        .render_pdf(&json!({}))
        .unwrap();
    let text = |b: &[u8]| String::from_utf8_lossy(b).to_string();
    assert!(text(&a4).contains("595.28"), "A4 is 595.28pt wide");
    assert!(text(&letter).contains("612"), "Letter is 612pt wide");
}

#[test]
fn a_url_from_the_data_cannot_run_script() {
    let tpl = Template::from_str(
        r#"page P(path: "/") {
            Link("Pay", to: url)
            Image(src: url, alt: "x")
            Element("x-card", href: url) { Text("e") }
            Sidebar { Sidebar.Item(to: url) { Text("s") } }
            Link("Home", to: home)
        }"#,
    )
    .unwrap();
    let html = tpl
        .render_html_fragment(
            &json!({ "url": " JavaScript:alert(1)", "home": "https://example.com/" }),
        )
        .unwrap();
    assert!(!html.to_lowercase().contains("javascript"), "{html}");
    assert!(
        html.contains("href=\"https://example.com/\""),
        "a safe URL is kept: {html}"
    );
}

#[test]
fn a_directory_is_one_template_with_its_data_files() {
    let dir = scratch("dir");
    std::fs::create_dir_all(dir.join("components")).unwrap();
    std::fs::write(
        dir.join("components/badge.wfx"),
        "component Tagline(_ text: String)\n    Text(text).bold\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("plans.json"),
        r#"[{ "name": "Pro" }, { "name": "Team" }]"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("pricing.wf"),
        "data plans = \"plans.json\"\npage Pricing(path: \"/\") {\n    Tagline(\"Plans\")\n    for p in plans { Text(p.name) }\n}\n",
    )
    .unwrap();
    let tpl = Template::from_dir(&dir).unwrap();
    let html = tpl.render_html_fragment(&json!({})).unwrap();
    assert!(
        html.contains("Plans") && html.contains("Pro") && html.contains("Team"),
        "{html}"
    );
    // And a single file by any path type.
    let one = Template::from_file(dir.join("pricing.wf"));
    assert!(one.is_err(), "pricing.wf alone does not know `Tagline`");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The text a PDF draws, every page, a line at a time.
fn pdf_text(tpl: &Template, data: &serde_json::Value) -> String {
    let report = tpl.render_pdf_report(data).unwrap();
    assert!(report.bytes.starts_with(b"%PDF-"), "a PDF");
    report.text.concat().join("\n")
}

#[test]
fn a_pdf_draws_what_the_components_draw() {
    let tpl = Template::from_sources(&[
        ("parts.wf", "component LineItem(_ label: String, amount: Number) {\n    Row { Text(label)  Text(format(amount, .currency)) }\n}\ncomponent Boxed { Card { children } }"),
        ("doc.wf", "page Doc(path: \"/\") {\n    for l in lines { LineItem(l.label, amount: l.amount) }\n    Boxed { Text(note.toUpperCase()) }\n}"),
    ])
    .unwrap();
    let text = pdf_text(
        &tpl,
        &json!({ "lines": [{ "label": "Design", "amount": 1200 }], "note": "thanks" }),
    );
    assert!(
        text.contains("Design") && text.contains("$1,200.00"),
        "{text}"
    );
    assert!(
        text.contains("THANKS"),
        "a block placed through `children`, a method evaluated: {text}"
    );
}

#[test]
fn constants_and_data_files_are_in_scope() {
    let tpl = Template::from_str(
        "const LIMIT = 3\nconst NAMES = [\"a\", \"b\"]\npage P(path: \"/\") { Text(\"limit {LIMIT}\")  for n in NAMES { Text(n) } }",
    )
    .unwrap();
    let html = tpl.render_html_fragment(&json!({})).unwrap();
    assert!(
        html.contains("limit 3") && html.contains(">a<") && html.contains(">b<"),
        "{html}"
    );
    assert!(pdf_text(&tpl, &json!({})).contains("limit 3"));
}

#[test]
fn a_page_s_state_and_derived_values_are_their_values_in_a_render() {
    let tpl = Template::from_str(
        r#"page P(path: "/") {
    derived total = items.map(i => i.price).sum()
    derived label = "{count} items, {total} in all"
    state count = items.length
    state note = "draft"
    Text(label)
    Text(note)
}"#,
    )
    .unwrap();
    let html = tpl
        .render_html_fragment(
            &json!({ "items": [{ "price": 2 }, { "price": 3 }], "note": "final" }),
        )
        .unwrap();
    // A derived value may read one declared below it, and a state.
    assert!(html.contains("2 items, 5 in all"), "{html}");
    // Data handed to the render wins over the state's first value.
    assert!(html.contains("final") && !html.contains("draft"), "{html}");
}

#[test]
fn a_responsive_layout_carries_the_class_its_rules_are_written_under() {
    // The page's CSS held `@media (min-width: 768px) { .wf-r… { … } }`, but the
    // element was written without `wf-r…`, so a rendered grid stayed one column.
    let tpl = Template::from_str(
        r#"page P(path: "/", title: "Grid") { Grid(columns: { base: 1, md: 3 }) { Text("a") Text("b") Text("c") } }"#,
    )
    .unwrap();
    let html = tpl.render_html(&json!({})).unwrap();
    let class = html
        .split("@media (min-width: 768px) { .wf-r")
        .nth(1)
        .and_then(|rest| rest.split(|c: char| !c.is_ascii_alphanumeric()).next())
        .map(|hash| format!("wf-r{hash}"))
        .expect("a responsive rule in the page's CSS");
    let grid = html
        .split("class=\"wf-grid")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("the grid element");
    assert!(
        grid.contains(&class),
        "the grid carries {class}: wf-grid{grid}"
    );
}
