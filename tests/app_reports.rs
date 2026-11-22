//! What people writing whole applications ran into: data files read as
//! their type, a page's own language and clock in the static paint, Arabic
//! dates, a money literal asked for a method, a number field that refused a
//! decimal, an enum case as an option's value, and the smaller things a
//! reader meets — a breadcrumb's targets, a dialog's heading, a Markdown
//! page's name, and a picture's placeholder under a strict policy.

use std::path::Path;
use std::process::Command;

use crate::common::{page, spa_js, ssg_html};

fn scratch(name: &str, config: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-apps-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), config).unwrap();
    dir
}

fn wf(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> (bool, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wf"));
    command.args(args).current_dir(dir);
    for (k, v) in env {
        command.env(k, v);
    }
    let out = command.output().expect("wf runs");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

/// Every finding for `src`, as `CODE: message`.
fn findings(src: &str) -> Vec<String> {
    let (program, mut out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    let source = src.to_string();
    out.extend(
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| Some(source.clone()),
            dir: None,
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete: false,
        })
        .diagnostics,
    );
    out.into_iter()
        .filter(|d| d.severity == webfluent::diagnostics::Severity::Error)
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

const DISHES: &str = "enum Diet { vegan, meat }\nenum State { idle, failed(reason: String) }\ntype Dish { name: String, diet: Diet, price: Money, state: State }\n";

/// A data file's enum field is its case's name and its money the carrier,
/// as `Dish.decode` reads them: neither is a `String` or a map the type
/// checker then refused (`T01`).
#[test]
fn a_data_file_is_read_as_its_declared_type() {
    let dir = scratch("data", r#"{ "name": "d", "build": { "ssg": true } }"#);
    std::fs::write(
        dir.join("src/menu.json"),
        r#"[{ "name": "Salad", "diet": "vegan", "price": { "amount": 1299, "currency": "EUR" }, "state": ["failed", "sold out"] }]"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        format!(
            "{DISHES}data menu: [Dish] = \"menu.json\"\npage Home(path: \"/\", title: \"Menu\", description: \"d\") {{\n    Heading(\"Menu\").h1\n    for d in menu by d.name {{\n        Text(\"{{d.name}} {{format(d.price)}}\")\n        if d.diet == .vegan {{ Text(\"Vegan\") }}\n        match d.state {{ .failed(r) {{ Text(\"Off: {{r}}\") }} else {{ Text(\"On\") }} }}\n    }}\n}}\n"
        ),
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"], &[]);
    assert!(ok, "{out}");
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    for want in [">Salad €12.99<", ">Vegan<", ">Off: sold out<"] {
        assert!(html.contains(want), "{want}: {html}");
    }
    // A case the enum does not have is still the file's fault, by place.
    std::fs::write(
        dir.join("src/menu.json"),
        r#"[{ "name": "Salad", "diet": "vegn", "price": { "amount": 1, "currency": "EUR" }, "state": "idle" }]"#,
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"], &[]);
    assert!(
        !ok && out.contains("T33") && out.contains("$[0].diet"),
        "{out}"
    );
    assert!(!out.contains("T01"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A template's data is read the same way.
#[test]
fn template_data_is_read_as_the_page_s_parameters_say() {
    let tpl = webfluent::Template::from_str(&format!(
        "{DISHES}page Card(path: \"/\", title: \"Card\", dish: Dish) {{\n    Text(\"{{dish.name}} {{format(dish.price)}}\")\n    if dish.diet == .vegan {{ Text(\"Vegan\") }}\n    match dish.state {{ .failed(r) {{ Text(\"Off: {{r}}\") }} else {{ Text(\"On\") }} }}\n}}\n"
    ))
    .unwrap();
    let html = tpl
        .render_html_fragment(&serde_json::json!({
            "dish": { "name": "Salad", "diet": "vegan", "price": { "amount": 1299, "currency": "EUR" }, "state": ["failed", "sold out"] }
        }))
        .unwrap();
    for want in [">Salad €12.99<", ">Vegan<", ">Off: sold out<"] {
        assert!(html.contains(want), "{want}: {html}");
    }
}

const TRANSLATED: &str = r#"{ "name": "t", "build": { "ssg": true }, "meta": { "lang": "ar" },
  "i18n": { "default_locale": "ar", "locales": ["ar", "en"], "dir": "src/translations" } }"#;

fn translated(name: &str, app: &str) -> std::path::PathBuf {
    let dir = scratch(name, TRANSLATED);
    std::fs::create_dir_all(dir.join("src/translations")).unwrap();
    std::fs::write(
        dir.join("src/translations/ar.json"),
        r#"{ "minutes.one": "دقيقة واحدة", "minutes.other": "{count} دقائق" }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/translations/en.json"),
        r#"{ "minutes.one": "{count} minute", "minutes.other": "{count} minutes" }"#,
    )
    .unwrap();
    std::fs::write(dir.join("src/App.wf"), app).unwrap();
    dir
}

/// `locale` and `dir` are the page's in the static paint — the site's
/// default, or the page's own `lang:` — and `now` the moment of the build,
/// which `SOURCE_DATE_EPOCH` pins. All three used to paint nothing.
#[test]
fn locale_dir_and_now_are_painted_as_the_page_s() {
    let dir = translated(
        "locale",
        "const TITLE = { ar: \"عنوان\", en: \"Title\" }\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    Text(\"[{locale} {dir}]\")\n    Text(TITLE[locale] ?? \"none\")\n    Text(\"N[{now:.date(long)}]\")\n}\npage En(path: \"/en\", title: \"En\", description: \"d\", lang: \"en\", translates: Home) {\n    Heading(\"En\").h1\n    Text(\"[{locale} {dir}]\")\n    Text(TITLE[locale] ?? \"none\")\n}\n",
    );
    // 2026-11-12T00:00:00Z
    let (ok, out) = wf(&dir, &["build"], &[("SOURCE_DATE_EPOCH", "1794441600")]);
    assert!(ok, "{out}");
    let home = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    for want in [">[ar rtl]<", ">عنوان<", ">N[12 نوفمبر 2026]<"] {
        assert!(home.contains(want), "{want}: {home}");
    }
    let en = std::fs::read_to_string(dir.join("build/en/index.html")).unwrap();
    for want in [">[en ltr]<", ">Title<"] {
        assert!(en.contains(want), "{want}: {en}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A plural message whose `count` the build cannot know is left for the
/// live page, never painted as its key.
#[test]
fn a_plural_with_an_unknown_count_never_paints_its_key() {
    let dir = translated(
        "plural",
        "page Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    Text(t(\"minutes\", { count: viewport.width }))\n    Text(\"[{t(\"minutes\", { count: viewport.width })}]\")\n    Text(t(\"minutes\", { count: 1 }))\n}\n",
    );
    let (ok, out) = wf(&dir, &["build"], &[]);
    assert!(ok, "{out}");
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(!html.contains("minutes"), "{html}");
    assert!(html.contains(">دقيقة واحدة<"), "{html}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// An Arabic page's dates are painted in Arabic, as `Intl` writes them on
/// the live page.
#[test]
fn an_arabic_page_paints_its_dates_in_arabic() {
    let dir = translated(
        "dates",
        "const DAY = @2026-11-12\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    Text(format(DAY, .date, \"long\"))\n    Text(\"{DAY:.date(full)}\")\n}\n",
    );
    let (ok, out) = wf(&dir, &["build"], &[]);
    assert!(ok, "{out}");
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(html.contains(">12 نوفمبر 2026<"), "{html}");
    assert!(html.contains(">الخميس، 12 نوفمبر 2026<"), "{html}");
    assert!(!html.contains("November"), "{html}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `€0.01.times(3)` is the literal, then its method: the lexer took the
/// second `.` as more decimals.
#[test]
fn a_money_literal_takes_a_method() {
    let html = ssg_html(&page(
        "    Text(format(€0.01.times(3)))\n    Text(format($2.50.plus($1.25)))",
    ));
    assert!(html.contains(">€0.03<"), "{html}");
    assert!(html.contains(">$3.75<"), "{html}");
}

/// A number field with no `step:` takes any number; the browser's
/// `step="1"` refused `12.5` and blocked the form with nothing shown.
#[test]
fn a_number_field_takes_a_decimal_unless_it_says_its_step() {
    let src = page(
        "    state amount: Number = 12.5\n    state qty: Number = 1\n    Input(bind: amount, label: \"Amount\").number\n    Input(bind: qty, label: \"Qty\", step: 1).number\n    Input(bind: qty, label: \"Plain\").text",
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("<input class=\"wf-input\" type=\"number\" step=\"any\""),
        "{html}"
    );
    assert_eq!(html.matches("step=\"any\"").count(), 1, "{html}");
    assert!(html.contains("step=\"1\""), "{html}");
    let js = spa_js(&src);
    assert_eq!(js.matches("step: \"any\"").count(), 1, "{js}");
}

/// An option's `value:` may be a case of the enum its control binds — the
/// bound state then holds the case.
#[test]
fn an_option_s_value_may_be_a_case_of_the_bound_enum() {
    let src = "enum Category { food, travel }\npage P(path: \"/\", title: \"T\", description: \"D\") {\n    state cat: Category = .food\n    state cats: [Category] = []\n    state all: [Category] = [.food, .travel]\n    Heading(\"T\").h1\n    Select(bind: cat, label: \"Category\") {\n        Select.Option(\"Food\", value: .food)\n        for c in all by c { Select.Option(\"x\", value: c) }\n    }\n    RadioGroup(bind: cat, label: \"R\") { RadioGroup.Option(\"Food\", value: .food) }\n    ToggleGroup(bind: cat, label: \"T\") { ToggleGroup.Item(\"Travel\", value: .travel) }\n    Listbox(bind: cat, label: \"L\") { Listbox.Option(\"Food\", value: .food) }\n    CheckboxGroup(bind: cats, label: \"C\") { CheckboxGroup.Option(\"Food\", value: .food) }\n    Radio(bind: cat, value: .travel, label: \"Travel\")\n}\n";
    assert!(findings(src).is_empty(), "{:#?}", findings(src));
    // The case named with its enum is the same value.
    let qualified = src.replace(
        "value: .food)\n        for",
        "value: Category.food)\n        for",
    );
    assert!(
        findings(&qualified).is_empty(),
        "{:#?}",
        findings(&qualified)
    );
    let clean = src.replace("for c in all by c { Select.Option(\"x\", value: c) }\n", "");
    assert!(findings(&clean).is_empty(), "{:#?}", findings(&clean));
    let html = ssg_html(&clean);
    assert!(html.contains("<option value=\"food\""), "{html}");
    // A case the enum lacks, and a case where the control binds no enum.
    let wrong = clean.replace(
        "Select.Option(\"Food\", value: .food)",
        "Select.Option(\"Food\", value: .fod)",
    );
    assert!(
        findings(&wrong)
            .iter()
            .any(|f| f.starts_with("T02") && f.contains(".fod")),
        "{:#?}",
        findings(&wrong)
    );
    let unbound = page_with(
        "    state s = \"a\"\n    Select(bind: s, label: \"S\") { Select.Option(\"A\", value: .a) }",
    );
    assert!(
        findings(&unbound).iter().any(|f| f.starts_with("E103")),
        "{:#?}",
        findings(&unbound)
    );
}

fn page_with(body: &str) -> String {
    format!(
        "page P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n{body}\n}}\n"
    )
}

/// A breadcrumb's links are targets a pointer can hit; a dialog's title is
/// the `h2` of its own context; `about-en.md` is the page `AboutEn`.
#[test]
fn breadcrumb_targets_dialog_headings_and_markdown_page_names() {
    let css = crate::common::built_in_css();
    assert!(
        css.contains("a.wf-breadcrumb__item { display: inline-flex; align-items: center; justify-content: center; min-height: var(--target-min); min-width: var(--target-min); }"),
        "the breadcrumb's links reach target-min"
    );
    let html = ssg_html(&page(
        "    state open = true\n    Modal(visible: open, title: \"Rename\") { Text(\"x\") }",
    ));
    assert!(
        html.contains("<h2 id=\"") && html.contains(">Rename</h2>"),
        "{html}"
    );
    let js = spa_js(&page(
        "    state open = true\n    Modal(visible: open, title: \"Rename\") { Text(\"x\") }",
    ));
    assert!(
        js.contains("WF.el(\"h2\"") && !js.contains("WF.el(\"h3\""),
        "{js}"
    );

    let program = webfluent::data::markdown_page("# About\n", "src/pages/about-en.md").unwrap();
    let names: Vec<&str> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            webfluent::syntax::parser::ast::Declaration::Page(p) => Some(p.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["AboutEn"]);
}

/// A picture's placeholder is a class the sheet styles, so a site of
/// pictures keeps a policy with no `'unsafe-inline'`.
#[test]
fn an_image_placeholder_keeps_the_strict_policy() {
    let dir = scratch(
        "placeholder",
        r#"{ "name": "p", "build": { "ssg": true, "csp": true } }"#,
    );
    std::fs::create_dir_all(dir.join("public")).unwrap();
    let mut png = Vec::new();
    image::RgbImage::from_pixel(64, 32, image::Rgb([200, 40, 40]))
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    std::fs::write(dir.join("public/red.png"), &png).unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        "image red = \"public/red.png\"\npage Home(path: \"/\", title: \"Home\", description: \"d\") {\n    Heading(\"Home\").h1\n    Image(red, alt: \"Red\", placeholder: .blur)\n    Image(src: \"/red.png\", alt: \"Again\", placeholder: .color)\n}\n",
    )
    .unwrap();
    let (ok, out) = wf(&dir, &["build"], &[]);
    assert!(ok, "{out}");
    assert!(!out.contains("allows inline styles"), "{out}");
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert!(!html.contains(" style=\""), "{html}");
    assert!(!html.contains("'unsafe-inline'"), "{html}");
    let class = html
        .split("class=\"wf-image wf-ph ")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .unwrap_or_else(|| panic!("a placeholder class: {html}"));
    let css = std::fs::read_to_string(dir.join("build/styles.css")).unwrap();
    assert!(css.contains(&format!(".wf-ph.{class}")), "{class}: {css}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A template rendered in a language reads it as `locale` and `dir`, and
/// `format` speaks it, as the page it stands for would.
#[test]
fn a_template_in_a_language_reads_it_and_formats_in_it() {
    let tpl = webfluent::Template::from_str(
        "const DAY = @2026-11-12\npage Card(path: \"/\", title: \"Card\") {\n    Text(\"[{locale} {dir}]\")\n    Text(format(DAY, .date, \"long\"))\n}\n",
    )
    .unwrap();
    let html = tpl
        .page("Card")
        .unwrap()
        .with_lang("ar")
        .render_html_fragment(&serde_json::json!({}))
        .unwrap();
    assert!(html.contains(">[ar rtl]<"), "{html}");
    assert!(html.contains(">12 نوفمبر 2026<"), "{html}");
}
