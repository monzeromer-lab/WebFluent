//! The editor knows what the project writes: a PDF or a slide deck draws
//! less than a page, so what it offers, what it explains and what it
//! reports follow `build.output_type`.

use tower_lsp::lsp_types::*;
use webfluent::config::OutputType;
use wf_lsp::completion::provide_completions;
use wf_lsp::diagnostics::project_diagnostics;
use wf_lsp::hover::provide_hover;
use wf_lsp::project::Project;

fn project(src: &str, output: OutputType) -> Project {
    let mut p = Project::single(Url::parse("file:///test.wf").unwrap(), src);
    p.output_type = output;
    p
}

/// What completion offers where `|` stands.
fn offered(src: &str, output: OutputType) -> Vec<String> {
    let at = src.find('|').expect("a cursor");
    let text = src.replace('|', "");
    let p = project(&text, output);
    let pos = p.files[0].index.offset_to_position(&text, at);
    provide_completions(&p, 0, pos)
        .into_iter()
        .map(|c| c.label)
        .collect()
}

fn hover(src: &str, needle: &str, output: OutputType) -> String {
    let p = project(src, output);
    let pos = p.files[0]
        .index
        .offset_to_position(src, src.find(needle).unwrap());
    match provide_hover(&p, 0, pos).expect("a hover").contents {
        HoverContents::Markup(m) => m.value,
        _ => panic!("markdown expected"),
    }
}

#[test]
fn a_pdf_project_is_offered_what_a_pdf_draws() {
    let src = "page R(path: \"/\") {\n    Document {\n        |\n    }\n}\n";
    let items = offered(src, OutputType::Pdf);
    for drawn in [
        "Heading",
        "Table",
        "Chart",
        "Header",
        "Watermark",
        "TableOfContents",
    ] {
        assert!(
            items.iter().any(|i| i == drawn),
            "{drawn} offered: {items:?}"
        );
    }
    for refused in [
        "Button",
        "Input",
        "Router",
        "Modal",
        "Video",
        "Slide",
        "Presentation",
    ] {
        assert!(!items.iter().any(|i| i == refused), "{refused} not offered");
    }
    // A handler, a timer, a request: none runs on paper.
    for word in ["on", "resource", "every", "persist"] {
        assert!(!items.iter().any(|i| i == word), "`{word}` not offered");
    }
    assert!(items.iter().any(|i| i == "for"), "`for` still offered");
}

#[test]
fn a_web_project_is_offered_everything_with_paper_last() {
    let src = "page R(path: \"/\") {\n    |\n}\n";
    let p = {
        let text = src.replace('|', "");
        project(&text, OutputType::Spa)
    };
    let text = src.replace('|', "");
    let pos = p.files[0]
        .index
        .offset_to_position(&text, src.find('|').unwrap());
    let items = provide_completions(&p, 0, pos);
    let get = |name: &str| items.iter().find(|i| i.label == name).cloned();
    assert!(get("Button").is_some() && get("Router").is_some());
    let watermark = get("Watermark").expect("a paper element is still offered");
    let button = get("Button").unwrap();
    assert!(
        watermark.sort_text > button.sort_text,
        "paper elements sort last"
    );
    assert!(watermark.detail.unwrap().contains("PDF or a deck"));
    assert!(items.iter().any(|i| i.label == "on"));
}

#[test]
fn a_presentation_holds_slides_and_a_slide_holds_the_rest() {
    let decl = "component Stat(_ n: String) {\n    Slide { Heading(n).h1 }\n}\n";
    let in_deck =
        format!("{decl}page D(path: \"/\") {{\n    Presentation {{\n        |\n    }}\n}}\n");
    let items = offered(&in_deck, OutputType::Slides);
    for slide in [
        "Slide",
        "TitleSlide",
        "SectionSlide",
        "TwoColumn",
        "ImageSlide",
        "Stat",
    ] {
        assert!(
            items.iter().any(|i| i == slide),
            "{slide} offered: {items:?}"
        );
    }
    for other in ["Heading", "Card", "Presentation", "Button"] {
        assert!(
            !items.iter().any(|i| i == other),
            "{other} not offered in a Presentation"
        );
    }

    let in_slide = format!(
        "{decl}page D(path: \"/\") {{\n    Presentation {{\n        Slide {{\n            |\n        }}\n    }}\n}}\n"
    );
    let items = offered(&in_slide, OutputType::Slides);
    for drawn in ["Heading", "Card", "Chart", "QrCode", "Grid"] {
        assert!(
            items.iter().any(|i| i == drawn),
            "{drawn} offered in a slide"
        );
    }
    for other in [
        "Slide",
        "Stat",
        "Header",
        "Footer",
        "PageBreak",
        "Document",
        "Button",
    ] {
        assert!(
            !items.iter().any(|i| i == other),
            "{other} not offered in a slide"
        );
    }
}

#[test]
fn hover_says_what_this_output_does_not_draw() {
    let src = "page R(path: \"/\") {\n    Document { Button(\"Pay\") }\n}\n";
    let text = hover(src, "Button", OutputType::Pdf);
    assert!(text.contains("Not drawn in a PDF"), "{text}");
    assert!(text.contains("E109"), "{text}");
    let web = hover(src, "Button", OutputType::Spa);
    assert!(!web.contains("Not drawn"), "{web}");

    let deck = "page D(path: \"/\") {\n    Presentation { Slide { Footer { Text(\"x\") } } }\n}\n";
    let text = hover(deck, "Footer", OutputType::Slides);
    assert!(text.contains("Not drawn in a slide deck"), "{text}");
    assert!(text.contains("footer_text"), "{text}");
}

#[test]
fn page_and_pages_are_names_inside_a_running_element() {
    let src = "page R(path: \"/\") {\n    Document {\n        Footer { Text(\"Page {page} of {pages}\") }\n    }\n}\n";
    let text = hover(src, "pages}", OutputType::Pdf);
    assert!(text.contains("page number"), "{text}");
}

#[test]
fn the_config_s_output_type_reaches_the_editor() {
    let root = std::env::temp_dir().join(format!("wf-lsp-output-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/pages")).unwrap();
    std::fs::write(
        root.join("webfluent.app.json"),
        r#"{ "name": "t", "build": { "output_type": "pdf" } }"#,
    )
    .unwrap();
    let page = "page R(path: \"/\", title: \"R\", description: \"A report.\") {\n    Document {\n        Heading(\"Report\").h1\n        Button(\"Pay\")\n    }\n}\n";
    std::fs::write(root.join("src/pages/R.wf"), page).unwrap();
    let uri = Url::from_file_path(root.join("src/pages/R.wf")).unwrap();
    let project = Project::load(&uri, &|_| None, &wf_lsp::project::FileCache::default());
    assert_eq!(project.output_type, OutputType::Pdf);
    let ix = project.file_index(&uri).unwrap();
    let found = project_diagnostics(&project).remove(ix);
    assert!(
        found.iter().any(
            |d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "E109")
                && d.range.start.line == 3
        ),
        "E109 on the Button: {found:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
