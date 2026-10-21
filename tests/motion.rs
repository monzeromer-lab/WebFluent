//! Motion as data and as the page plays it: `motion::catalogue` offers
//! exactly what the engine's stylesheet and the project's declarations
//! play; a project's own animation plays on an element that stands still,
//! as a built-in's does; and the page's root can ask for less motion
//! (`data-wf-motion="reduce"`), as an editor previews it, in a stylesheet
//! the prune keeps. What a real browser does with all of it is
//! `tests/browser/motion.mjs`.

mod common;

use common::{page, spa_generated, ssg_html};
use webfluent::motion::{REDUCED, catalogue, spelling};

const DROP: &str = "animation Drop {\n    from {\n        opacity: 0\n    }\n    to {\n        opacity: 1\n    }\n}\n";

#[test]
fn a_project_animation_plays_on_an_element_that_stands_still() {
    let src = format!(
        "{DROP}{}",
        page(
            "Card(animate: .Drop, id: \"d\") { Text(\"x\") }\n    Card(animate: .Drop, class: \"mine\", id: \"e\") { Text(\"y\") }"
        )
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("class=\"wf-card wf-animate-Drop\" id=\"d\""),
        "{html}"
    );
    assert!(
        html.contains("class=\"wf-card mine wf-animate-Drop\" id=\"e\""),
        "{html}"
    );
    assert!(!html.contains("animate=\""), "no stray attribute: {html}");
    let js = spa_generated(&src);
    assert!(js.contains("classList.add(\"wf-animate-Drop\")"), "{js}");
    assert!(
        js.contains("classList.add(\"mine\", \"wf-animate-Drop\")")
            || js.contains("classList.add(\"mine\",\"wf-animate-Drop\")"),
        "{js}"
    );
    assert!(!js.contains("animate: \"Drop\""), "{js}");
}

#[test]
fn the_class_it_plays_by_is_no_misuse_of_the_engine_s() {
    let dir = std::env::temp_dir().join(format!("wf-motion-v04-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), "{\"name\":\"m\"}").unwrap();
    std::fs::write(
        dir.join("src/pages/Home.wf"),
        format!(
            "{DROP}page Home(path: \"/\", title: \"M\", description: \"Motion.\") {{\n    Heading(\"M\").h1\n    Card(animate: .Drop) {{ Text(\"x\") }}\n    Card(class: \"wf-animate-fadeIn\") {{ Text(\"y\") }}\n}}\n"
        ),
    )
    .unwrap();
    let options = webfluent::build::BuildOptions::new(&dir, &webfluent::vfs::FsVfs);
    let report = match webfluent::build::check(&options) {
        Ok(r) => r,
        Err(f) => *f.report,
    };
    let v04: Vec<String> = report
        .diagnostics
        .iter()
        .filter(|d| d.code == "V04")
        .map(|d| d.message.clone())
        .collect();
    assert_eq!(
        v04,
        ["`class:` names `wf-animate-fadeIn`, one of the engine's own classes"],
        "a built-in's class written by hand still is"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_page_s_root_can_ask_for_less_motion() {
    assert_eq!(REDUCED, "data-wf-motion=\"reduce\"");
    let rule = ":root[data-wf-motion=\"reduce\"] *";
    for css in [
        webfluent::themes::components::component_css(),
        webfluent::themes::structural::structural_css(),
    ] {
        let at = css.find(rule).expect("the attribute's rule");
        let block = &css[at..at + css[at..].find('}').unwrap()];
        for d in [
            "animation-duration: 0.01ms !important",
            "animation-iteration-count: 1 !important",
            "transition-duration: 0.01ms !important",
        ] {
            assert!(block.contains(d), "{d} in {block}");
        }
    }
    // The prune keeps it, whatever the page uses.
    let dir = std::env::temp_dir().join(format!("wf-motion-prune-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        "{\"name\":\"m\",\"build\":{\"ssg\":true}}",
    )
    .unwrap();
    std::fs::write(dir.join("src/pages/Home.wf"), "page Home(path: \"/\", title: \"M\", description: \"Motion.\") {\n    Heading(\"M\").h1\n    state open = false\n    Card.fadeIn { Text(\"x\") }\n    Button(\"Go\") { on click { open = !open } }\n    if open {\n        Card(animate: .slideUp) { Text(\"y\") }\n    }\n}\n").unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let sheet = std::fs::read_to_string(dir.join("build/styles.css")).unwrap();
    assert!(
        sheet.contains(":root[data-wf-motion=\"reduce\"] *"),
        "the built sheet keeps the rule"
    );
    // And the runtime asks the root before the reader's setting.
    let js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    let reads = js
        .find("data-wf-motion")
        .expect("the runtime reads the attribute");
    let setting = js
        .find("prefers-reduced-motion")
        .expect("and the reader's setting");
    assert!(reads < setting, "the attribute first");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_catalogue_is_what_the_build_plays() {
    let program = webfluent::parse_source(
        &format!("{DROP}{}", page("Card(animate: .Drop) { Text(\"x\") }")),
        "t.wf",
    )
    .unwrap();
    let m = catalogue(Some(&program), &Default::default());
    let sheet = webfluent::themes::components::component_css();
    for a in m.animations.iter().filter(|a| !a.project) {
        assert!(sheet.contains(&a.rule), "{}'s rule is the sheet's", a.name);
        if !a.keyframes.is_empty() {
            assert!(
                sheet.contains(&a.keyframes),
                "{}'s keyframes are the sheet's",
                a.name
            );
        }
        // Every built-in is one `animate:` and its flag take.
        assert_eq!(spelling("animate", &a.name, &[]), format!(".{}", a.name));
    }
    let drop = m.animations.iter().find(|a| a.name == "Drop").unwrap();
    assert!(drop.project);
    let css = webfluent::codegen::scoped_css::animations_css(&program);
    assert!(
        css.contains(&drop.keyframes)
            || css
                .replace(' ', "")
                .contains(&drop.keyframes.replace(' ', "")),
        "{css}\n{}",
        drop.keyframes
    );
    // Every ease is a case `easing:` takes, and every token one the baseline has.
    for e in &m.eases {
        if let Some(t) = e
            .css
            .strip_prefix("var(--")
            .and_then(|t| t.strip_suffix(')'))
        {
            assert!(e.value.is_some(), "{t} is a baseline token");
        }
    }
    // The catalogue as JSON is what `wf motion --json` prints.
    let json = serde_json::to_value(&m).unwrap();
    assert_eq!(json["reduced_attribute"], REDUCED);
}
