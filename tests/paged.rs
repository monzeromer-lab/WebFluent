//! The paged engine, held to what a document should say on each page: page
//! numbers, repeated table headers, contents, links, Arabic, slides, and the
//! reference documents under `examples/documents`.

use serde_json::json;
use std::fmt::Write as _;
use webfluent::{PdfConfig, SlidesConfig, Template};

fn report(src: &str) -> webfluent::PdfReport {
    Template::from_str(src)
        .unwrap()
        .render_pdf_report(&json!({}))
        .unwrap()
}

fn page_text(r: &webfluent::PdfReport, page: usize) -> String {
    r.text[page].join("\n")
}

#[test]
fn a_document_shorter_than_a_page_is_one_page() {
    // The baseline gives `body` `min-height: 100vh`, and the engine took
    // 100vh as the paper's height while laying out inside the margins, so
    // every short document grew an empty second page.
    let r = report(
        r#"page P(path: "/", title: "One page") { Document { Heading("Invoice").h1  Text("Total: $40") } }"#,
    );
    assert_eq!(r.pages, 1, "{:?}", r.text);
    let r = report(r#"page P(path: "/", title: "Bare") { Heading("Invoice").h1 }"#);
    assert_eq!(r.pages, 1, "{:?}", r.text);
}

#[test]
fn an_ordered_list_numbers_its_items_and_a_nested_one_starts_again() {
    // The sheet numbers `List.ordered` with a CSS counter, which the engine
    // did not keep, so every item was a bare "." beside its text.
    let r = report(
        r#"page P(path: "/", title: "Steps") {
    List.ordered {
        List.Item { Text("Sign") }
        List.Item {
            Stack {
                Text("Pay")
                List.ordered { List.Item { Text("Deposit") }  List.Item { Text("Balance") } }
            }
        }
        List.Item { Text("Launch") }
    }
}"#,
    );
    let text = page_text(&r, 0);
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    for (number, word) in [
        ("1.", "Sign"),
        ("2.", "Pay"),
        ("1.", "Deposit"),
        ("2.", "Balance"),
        ("3.", "Launch"),
    ] {
        let at = lines
            .iter()
            .position(|l| l.contains(word))
            .unwrap_or_else(|| panic!("{word}: {text}"));
        assert!(
            lines[at].starts_with(number) || at > 0 && lines[at - 1] == number,
            "{word} is numbered {number}: {text}"
        );
    }
    assert!(!lines.contains(&"."), "{text}");
}

#[test]
fn a_columns_span_reaches_the_paint_a_pdf_is_laid_out_from() {
    // The paint wrote every `Column` as a bare `wf-col`, so a PDF drew
    // `span: 8` and `span: 4` as two equal halves.
    let html = Template::from_str(
        r#"page P(path: "/", title: "Spans") { Row(gap: .md) { Column(span: 8) { Text("a") } Column(span: 4) { Text("b") } } }"#,
    )
    .unwrap()
    .render_html_fragment(&json!({}))
    .unwrap();
    assert!(html.contains(r#"class="wf-col wf-col--8""#), "{html}");
    assert!(html.contains(r#"class="wf-col wf-col--4""#), "{html}");
}

#[test]
fn a_footer_numbers_every_page_and_a_header_keeps_to_its_pages() {
    let filler: String = (0..120).fold(String::new(), |mut acc, i| {
        let _ = writeln!(
            acc,
            "Text(\"Paragraph number {i} of the body, long enough to take a line.\")"
        );
        acc
    });
    let r = report(&format!(
        "page P(path: \"/\") {{ Document {{ Header(on: .rest) {{ Text(\"Running head\") }} Footer {{ Text(\"Page {{page}} of {{pages}}\") }} {filler} }} }}"
    ));
    assert!(r.pages >= 3, "{} pages", r.pages);
    for p in 0..r.pages {
        assert!(
            page_text(&r, p).contains(&format!("Page {} of {}", p + 1, r.pages)),
            "page {}: {:?}",
            p + 1,
            r.text[p]
        );
    }
    assert!(
        !page_text(&r, 0).contains("Running head"),
        "not on the first page"
    );
    assert!(page_text(&r, 1).contains("Running head"));
    assert!(r.bytes.starts_with(b"%PDF-"));
}

#[test]
fn a_long_table_repeats_its_header_on_every_page() {
    let rows: String = (0..90).fold(String::new(), |mut acc, i| {
        let _ = writeln!(
            acc,
            "Table.Row {{ Table.Cell(\"row {i}\")  Table.Cell(\"{}\") }}",
            i * 7
        );
        acc
    });
    let r = report(&format!(
        "page P(path: \"/\") {{ Document {{ Table {{ Table.Head {{ Table.Row {{ Table.Cell(\"Item\")  Table.Cell(\"Amount\") }} }} Table.Body {{ {rows} }} }} }} }}"
    ));
    assert!(r.pages >= 2, "{} pages", r.pages);
    for p in 0..r.pages {
        let t = page_text(&r, p);
        // The engine's table sets its header in capitals, as on the web.
        assert!(
            t.to_lowercase().starts_with("item"),
            "page {} starts with the header: {:?}",
            p + 1,
            &r.text[p][..3]
        );
    }
    // Every row is somewhere, once.
    let all = r.text.concat().join("\n");
    for i in 0..90 {
        assert_eq!(
            all.matches(&format!("row {i}\n")).count()
                + usize::from(all.ends_with(&format!("row {i}"))),
            1,
            "row {i}"
        );
    }
}

#[test]
fn a_page_break_and_the_contents_point_at_the_right_pages() {
    let r = report(
        r#"page P(path: "/") {
            Document {
                TableOfContents(levels: 2)
                PageBreak
                Heading("Introduction").h2
                Text("Opening words.")
                PageBreak
                Heading("Terms").h2
                Text("Payment within 30 days.")
            }
        }"#,
    );
    assert_eq!(r.pages, 3, "{:?}", r.text);
    // Each entry: its title, then its page.
    let toc = page_text(&r, 0);
    assert!(toc.contains("Introduction\n2"), "{toc}");
    assert!(toc.contains("Terms\n3"), "{toc}");
    assert!(page_text(&r, 2).contains("Terms"));
}

#[test]
fn links_are_links_and_metadata_is_written() {
    let r = report(
        r##"page P(path: "/") {
            Document(title: "Quarterly", author: "Ada") {
                Link("Our site", to: "https://example.com/about")
                Link("Terms", to: "#terms")
                Heading("Terms", id: "terms").h2
            }
        }"##,
    );
    let bytes = String::from_utf8_lossy(&r.bytes);
    assert!(
        bytes.contains("https://example.com/about"),
        "the link's address is in the file"
    );
    assert!(bytes.contains("/Annot"), "as an annotation");
    assert!(page_text(&r, 0).contains("Our site"));
}

#[test]
fn arabic_is_set_from_the_projects_fonts_and_a_missing_glyph_is_said() {
    let dir = std::env::temp_dir().join(format!("wf-paged-ar-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    let fonts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/documents/fonts");
    std::fs::copy(
        fonts.join("NotoSansArabic.ttf"),
        dir.join("fonts/NotoSansArabic.ttf"),
    )
    .unwrap();
    std::fs::write(
        dir.join("doc.wf"),
        r#"page P(path: "/") { Document(lang: "ar") { Text("مرحبا بالعالم") { style { font-family: Noto Sans Arabic } } } }"#,
    )
    .unwrap();
    let no_system = PdfConfig {
        system_fonts: false,
        ..PdfConfig::default()
    };
    let tpl = Template::from_dir(&dir)
        .unwrap()
        .with_pdf(no_system.clone());
    let r = tpl.render_pdf_report(&json!({})).unwrap();
    assert!(r.notes.is_empty(), "{:?}", r.notes);
    assert!(page_text(&r, 0).contains("مرحبا بالعالم"), "{:?}", r.text);
    // Without the font, and without the machine's: said, not drawn as `?`.
    let bare = Template::from_str(r#"page P(path: "/") { Document { Text("مرحبا") } }"#)
        .unwrap()
        .with_pdf(no_system);
    let r = bare.render_pdf_report(&json!({})).unwrap();
    assert!(
        r.notes.iter().any(|n| n.contains("no font has")),
        "{:?}",
        r.notes
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_deck_is_a_page_per_slide_with_its_chrome() {
    let tpl = Template::from_str(
        r#"page D(path: "/") {
            Presentation {
                TitleSlide("Q4 review", subtitle: "October")
                Slide { Heading("Numbers").h2  Text("Up 18%.") }
                TwoColumn { Container { Text("Left") } Container { Text("Right") } }
            }
        }"#,
    )
    .unwrap()
    .with_slides(SlidesConfig {
        show_slide_numbers: true,
        footer_text: Some("Acme".into()),
        ..SlidesConfig::default()
    });
    let r = tpl.render_slides_report(&json!({})).unwrap();
    assert_eq!(r.pages, 3);
    assert!(r.notes.is_empty(), "{:?}", r.notes);
    assert!(page_text(&r, 0).contains("Q4 review") && page_text(&r, 0).contains("1 / 3"));
    assert!(page_text(&r, 2).contains("Acme") && page_text(&r, 2).contains("3 / 3"));
    // A slide that cannot hold its content says so.
    let long: String = (0..60).fold(String::new(), |mut acc, i| {
        let _ = writeln!(acc, "Text(\"Line {i}\")");
        acc
    });
    let r = Template::from_str(&format!(
        "page D(path: \"/\") {{ Presentation {{ Slide {{ {long} }} }} }}"
    ))
    .unwrap()
    .render_slides_report(&json!({}))
    .unwrap();
    assert!(
        r.notes.iter().any(|n| n.contains("slide 1 is taller")),
        "{:?}",
        r.notes
    );
}

/// Build a reference document in a copy of its project, as a reader would.
fn build_document(name: &str) -> (String, std::path::PathBuf) {
    build_example("documents", name)
}

/// Build an example under `examples/<group>/<name>` in a copy laid out as the
/// repository is, so its `../../documents/fonts` resolves.
fn build_example(group: &str, name: &str) -> (String, std::path::PathBuf) {
    fn copy(from: &std::path::Path, to: &std::path::Path) {
        std::fs::create_dir_all(to).unwrap();
        for e in std::fs::read_dir(from).unwrap().flatten() {
            let p = e.path();
            if p.file_name().is_some_and(|n| n == "build") {
                continue;
            }
            let target = to.join(e.file_name());
            if p.is_dir() {
                copy(&p, &target);
            } else {
                std::fs::copy(&p, &target).unwrap();
            }
        }
    }

    let examples = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let root = examples.join(group);
    let tmp = std::env::temp_dir().join(format!("wf-{group}-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    copy(
        &examples.join("documents/fonts"),
        &tmp.join("documents/fonts"),
    );
    copy(&root.join(name), &tmp.join(group).join(name));
    run_build(&tmp, group, name)
}

/// Build the copy of an example at `tmp/<group>/<name>`.
fn run_build(tmp: &std::path::Path, group: &str, name: &str) -> (String, std::path::PathBuf) {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(tmp.join(group).join(name))
        .output()
        .unwrap();
    let stdout = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{stdout}");
    (stdout, tmp.to_path_buf())
}

fn pages_in(stdout: &str) -> usize {
    let line = stdout
        .lines()
        .find(|l| l.contains("PDF:"))
        .expect("a PDF line");
    line.split(", ")
        .nth(1)
        .and_then(|p| p.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap()
}

fn slides_in(stdout: &str) -> usize {
    let line = stdout
        .lines()
        .find(|l| l.contains("Slides:"))
        .expect("a Slides line");
    line.split(", ")
        .nth(1)
        .and_then(|p| p.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap()
}

#[test]
fn the_webfluent_deck_builds_whole_with_its_own_fonts() {
    let (out, tmp) = build_example("decks", "webfluent");
    assert!(
        !out.contains("note:"),
        "nothing clipped, nothing missing: {out}"
    );
    assert!(!out.contains("warning"), "{out}");
    assert_eq!(slides_in(&out), 16, "{out}");
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn the_arabic_webfluent_deck_builds_whole_right_to_left() {
    let (out, tmp) = build_example("decks", "webfluent-ar");
    assert!(
        !out.contains("note:"),
        "nothing clipped, nothing missing: {out}"
    );
    assert!(!out.contains("warning"), "{out}");
    assert_eq!(slides_in(&out), 16, "{out}");
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn the_halyard_review_deck_makes_a_slide_per_incident() {
    let (out, tmp) = build_example("decks", "halyard-review");
    assert!(!out.contains("note:"), "{out}");
    // Twelve written slides, and one for each of the four incidents.
    assert_eq!(slides_in(&out), 16, "{out}");
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn the_arabic_deck_builds_right_to_left() {
    let (out, tmp) = build_example("decks", "arabic-workshop");
    assert!(!out.contains("note:"), "{out}");
    assert!(!out.contains("warning"), "{out}");
    assert_eq!(slides_in(&out), 8, "{out}");
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn a_presentation_takes_slides_from_components_and_loops() {
    let src = r#"
const POINTS = ["One", "Two", "Three"]
component Point(_ label: String, n: Number) {
    Slide { Heading(label).h1  Text("Slide {n}") }
}
page D(path: "/") {
    derived count = POINTS.length
    Presentation {
        TitleSlide("Deck", subtitle: "{count} points")
        for p, i in POINTS { Point(p, n: i + 1) }
        if count > 2 { Slide { Text("Many") } }
    }
}
"#;
    let r = Template::from_str(src)
        .unwrap()
        .render_slides_report(&json!({}))
        .unwrap();
    assert_eq!(r.pages, 5);
    let text: Vec<String> = r.text.iter().map(|p| p.join(" ")).collect();
    assert!(
        text[0].contains("3 points"),
        "a derived value is its value on paper: {text:?}"
    );
    assert!(
        text[2].contains("Two") && text[2].contains("Slide 2"),
        "{text:?}"
    );
    assert!(text[4].contains("Many"), "{text:?}");
}

#[test]
fn the_halyard_report_builds_with_its_own_fonts_and_nothing_missing() {
    let (out, tmp) = build_document("halyard-report");
    assert!(
        !out.contains("note:"),
        "every glyph from the project's fonts: {out}"
    );
    assert!((7..=12).contains(&pages_in(&out)), "{out}");
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn the_invoice_builds_with_its_own_fonts_and_nothing_missing() {
    let (out, tmp) = build_document("invoice");
    assert!(!out.contains("note:"), "{out}");
    assert!((2..=5).contains(&pages_in(&out)), "{out}");
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn the_arabic_resume_builds() {
    // Its one Japanese line has no font under fonts/. Built without the
    // machine's fonts, so every machine reports the same: that line's
    // characters are missing, and nothing else is.
    let (_, tmp) = build_document("resume");
    let config = tmp.join("documents/resume/webfluent.app.json");
    let text = std::fs::read_to_string(&config).unwrap();
    std::fs::write(
        &config,
        text.replace("\"system_fonts\": true", "\"system_fonts\": false"),
    )
    .unwrap();
    let (out, _) = run_build(&tmp, "documents", "resume");
    assert!(!out.contains("came from this machine"), "{out}");
    assert!(
        out.lines()
            .filter(|l| l.contains("note:"))
            .all(|l| l.contains("CJK") || l.contains("no font has `")),
        "{out}"
    );
    assert!((1..=3).contains(&pages_in(&out)), "{out}");
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn a_background_picture_is_painted() {
    let dir = std::env::temp_dir().join(format!("wf-paged-bg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("public")).unwrap();
    let png = one_colour_png(8, 6, [0x0F, 0x76, 0x6E]);
    std::fs::write(dir.join("public/cover.png"), &png).unwrap();
    // A picture that cannot be read is left out, with a note: it never
    // stops the document.
    let broken = png[..png.len() / 2].to_vec();
    std::fs::write(dir.join("public/broken.png"), &broken).unwrap();
    std::fs::write(
        dir.join("doc.wf"),
        r#"page P(path: "/") { Document { Background(on: .first) { Stack { style { height: 100%; background: url(/cover.png) center / cover } } } Heading("Cover").h1  Image(src: "/broken.png", alt: "") } }"#,
    )
    .unwrap();
    let r = Template::from_dir(&dir)
        .unwrap()
        .render_pdf_report(&json!({}))
        .unwrap();
    let bytes = String::from_utf8_lossy(&r.bytes);
    assert!(
        bytes.contains("/Subtype/Image") || bytes.contains("/Subtype /Image"),
        "the picture is in the file"
    );
    assert!(
        r.notes.iter().any(|n| n.contains("broken.png")),
        "{:?}",
        r.notes
    );
    let _ = std::fs::remove_dir_all(&dir);
}

fn one_colour_png(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    fn crc(data: &[u8]) -> u32 {
        let mut table = [0u32; 256];
        for (n, slot) in table.iter_mut().enumerate() {
            let mut c = n as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xEDB8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
            *slot = c;
        }
        let mut c = 0xFFFF_FFFFu32;
        for b in data {
            c = table[((c ^ u32::from(*b)) & 0xFF) as usize] ^ (c >> 8);
        }
        c ^ 0xFFFF_FFFF
    }
    fn chunk(kind: &[u8], data: &[u8]) -> Vec<u8> {
        let mut out = (data.len() as u32).to_be_bytes().to_vec();
        let body: Vec<u8> = kind.iter().chain(data).copied().collect();
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc(&body).to_be_bytes());
        out
    }
    let mut row = vec![0u8];
    for _ in 0..width {
        row.extend_from_slice(&rgb);
    }
    let raw: Vec<u8> = row.repeat(height as usize);
    // Stored deflate blocks, which every decoder reads.
    let mut deflated = vec![0x78u8, 0x01];
    for (i, block) in raw.chunks(65_535).enumerate() {
        let last = u8::from((i + 1) * 65_535 >= raw.len());
        deflated.push(last);
        deflated.extend_from_slice(&(block.len() as u16).to_le_bytes());
        deflated.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        deflated.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &raw {
        a = (a + u32::from(*byte)) % 65521;
        b = (b + a) % 65521;
    }
    deflated.extend_from_slice(&((b << 16) | a).to_be_bytes());

    let mut header = width.to_be_bytes().to_vec();
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend_from_slice(&chunk(b"IHDR", &header));
    png.extend_from_slice(&chunk(b"IDAT", &deflated));
    png.extend_from_slice(&chunk(b"IEND", b""));
    png
}

// ─── The page box, media, fixed boxes, selectors, tagging ──────────

/// A PDF of `html` (its `<style>` elements and all) with the default
/// configuration — A4, one-inch margins — and `tagged` as asked.
fn pdf_tagged(html: &str, tagged: bool) -> webfluent::paged::Output {
    let config = PdfConfig::default();
    let reader = |_: &str| -> Option<Vec<u8>> { None };
    let mut options = webfluent::paged::Options::from_pdf(&config, &reader, None);
    options.tagged = tagged;
    webfluent::paged::render(html, "", &options).unwrap()
}

fn pdf(html: &str) -> webfluent::paged::Output {
    pdf_tagged(html, true)
}

/// The first page's `/MediaBox`, as written.
fn media_box(out: &webfluent::paged::Output) -> String {
    let bytes = String::from_utf8_lossy(&out.bytes);
    let at = bytes.find("/MediaBox[").expect("a media box");
    let rest = &bytes[at + "/MediaBox[".len()..];
    rest[..rest.find(']').unwrap()].to_string()
}

fn paragraphs(n: usize) -> String {
    (0..n).fold(String::new(), |mut acc, i| {
        let _ = write!(acc, "<p>Paragraph {i} of the document.</p>");
        acc
    })
}

#[test]
fn page_rules_size_the_paper_and_the_document_element_wins_over_them() {
    let a5 =
        pdf("<html><head><style>@page { size: A5 }</style></head><body><p>x</p></body></html>");
    assert_eq!(media_box(&a5), "0 0 419.53 595.28");
    let turned = pdf(
        "<html><head><style>@page { size: A5 landscape }</style></head><body><p>x</p></body></html>",
    );
    assert_eq!(media_box(&turned), "0 0 595.28 419.53");
    // `Document(size:)` is written on the element, over every sheet.
    let doc = pdf(
        "<html><head><style>@page { size: A5 }</style></head><body><div class=\"wf-document\" data-size=\"Letter\"><p>x</p></div></body></html>",
    );
    assert_eq!(media_box(&doc), "0 0 612 792");
    // With nothing said, the configuration's paper.
    assert_eq!(media_box(&pdf("<p>x</p>")), "0 0 595.28 841.89");
}

#[test]
fn page_margins_differ_on_the_first_page_and_by_side() {
    let body = paragraphs(120);
    let plain = pdf(&format!("<html><body>{body}</body></html>"));
    let first = pdf(&format!(
        "<html><head><style>@page :first {{ margin-top: 400pt }}</style></head><body>{body}</body></html>"
    ));
    // The first page holds fewer lines under its deep top margin; the
    // second as many as before.
    assert!(
        first.text[0].len() + 10 < plain.text[0].len(),
        "{} vs {}",
        first.text[0].len(),
        plain.text[0].len()
    );
    assert_eq!(first.text[1].len(), plain.text[1].len());
    // Every paragraph is somewhere, once.
    let all = first.text.concat().join("\n");
    for i in 0..120 {
        assert_eq!(all.matches(&format!("Paragraph {i} ")).count(), 1, "{i}");
    }
    // `:left` pages (2, 4…) with a deeper bottom hold less than `:right`
    // ones.
    let sides = pdf(&format!(
        "<html><head><style>@page :left {{ margin-bottom: 300pt }}</style></head><body>{body}</body></html>"
    ));
    let lens: Vec<usize> = sides.text.iter().map(Vec::len).collect();
    // Page 1 is a right-hand page, so the left ones are 2 and 4.
    assert!(
        lens[1] + 5 < lens[2] && lens[3] + 5 < lens[2] && lens[0] == lens[2],
        "{lens:?}"
    );
    // A margin box is noted, not drawn.
    let boxed = pdf(
        "<html><head><style>@page { @top-center { content: \"x\" } margin: 1in }</style></head><body><p>x</p></body></html>",
    );
    assert!(
        boxed.notes.iter().any(|n| n.contains("@top-center")),
        "{:?}",
        boxed.notes
    );
}

#[test]
fn width_media_queries_are_asked_of_the_page_area() {
    let css = ".n, .w, .p, .s { display: none }
        @media (max-width: 500px) { .n { display: block } }
        @media (min-width: 501px) { .w { display: block } }
        @media print and (orientation: portrait) { .p { display: block } }
        @media screen { .s { display: block } }";
    let html = |page: &str| {
        format!(
            "<html><head><style>{page}{css}</style></head><body><p class=\"n\">narrow</p><p class=\"w\">wide</p><p class=\"p\">portrait</p><p class=\"s\">screen</p></body></html>"
        )
    };
    // A4 inside one-inch margins is 451pt, 602 CSS pixels, wide.
    let a4 = pdf(&html("")).text.concat();
    assert_eq!(a4, vec!["wide", "portrait"]);
    // A6 leaves 153pt, 204 pixels: narrow.
    let a6 = pdf(&html("@page { size: A6 }")).text.concat();
    assert_eq!(a6, vec!["narrow", "portrait"]);
    // Turned, the page is no longer portrait.
    let wide = pdf(&html("@page { size: A4 landscape }")).text.concat();
    assert_eq!(wide, vec!["wide"]);
}

#[test]
fn a_fixed_box_is_drawn_on_every_page_and_takes_no_room() {
    let body = paragraphs(120);
    let out = pdf(&format!(
        "<html><body><div style=\"position: fixed; top: 10pt; right: 10pt\">CONFIDENTIAL</div>{body}</body></html>"
    ));
    assert!(out.pages >= 2, "{} pages", out.pages);
    for (i, page) in out.text.iter().enumerate() {
        assert_eq!(
            page.iter().filter(|l| l.contains("CONFIDENTIAL")).count(),
            1,
            "page {}: {page:?}",
            i + 1
        );
    }
    // The flow starts where it would without it.
    let plain = pdf(&format!("<html><body>{body}</body></html>"));
    assert_eq!(out.pages, plain.pages);
}

/// The computed style of the element with `id`, through its ancestors.
fn style_of(html: &str, css: &str, id: &str) -> webfluent::paged::style::Style {
    use webfluent::paged::cascade::{Styler, UA_SHEET};
    use webfluent::paged::css::Sheet;
    use webfluent::paged::html::Dom;
    use webfluent::paged::style::{Style, Units};
    let dom = Dom::parse(html);
    let styler = Styler::new(
        &Sheet::parse(UA_SHEET),
        &Sheet::parse(css),
        Units {
            em: 12.0,
            rem: 12.0,
            vw: 600.0,
            vh: 800.0,
        },
    );
    let node = dom
        .descendants(0)
        .into_iter()
        .find(|&n| dom.element(n).and_then(|e| e.attr("id")) == Some(id))
        .unwrap_or_else(|| panic!("no #{id}"));
    let mut chain = vec![node];
    while let Some(p) = dom.nodes[*chain.last().unwrap()].parent {
        chain.push(p);
    }
    let mut style = Style::root(12.0, "sans-serif");
    for &n in chain.iter().rev() {
        if dom.element(n).is_some() {
            style = styler.compute(&dom, n, &style);
        }
    }
    style
}

#[test]
fn logical_insets_and_sides_follow_the_direction() {
    use webfluent::paged::style::Length;
    let css = "#a { position: absolute; inset-inline-start: 5pt; inset-inline-end: 7pt; margin-inline-start: 3pt; padding-inline-end: 2pt }";
    let ltr = style_of("<div id=\"a\">x</div>", css, "a");
    assert_eq!(ltr.inset[3], Length::Pt(5.0));
    assert_eq!(ltr.inset[1], Length::Pt(7.0));
    assert_eq!(ltr.margin[3], Length::Pt(3.0));
    assert_eq!(ltr.padding[1], Length::Pt(2.0));
    let rtl = style_of("<div dir=\"rtl\"><div id=\"a\">x</div></div>", css, "a");
    assert_eq!(rtl.inset[1], Length::Pt(5.0));
    assert_eq!(rtl.inset[3], Length::Pt(7.0));
    assert_eq!(rtl.margin[1], Length::Pt(3.0));
    assert_eq!(rtl.padding[3], Length::Pt(2.0));
    // The direction a rule sets on the element itself counts, wherever in
    // the rule it is written.
    let own = style_of(
        "<div id=\"a\">x</div>",
        "#a { inset-inline-start: 5pt; direction: rtl }",
        "a",
    );
    assert_eq!(own.inset[1], Length::Pt(5.0));
}

#[test]
fn text_shadow_is_read_and_drawn_under_the_text_once() {
    let s = style_of(
        "<p id=\"a\">x</p>",
        "#a { text-shadow: 1px 2px 3px #ff0000, 0 0 2pt }",
        "a",
    );
    assert_eq!(s.text_shadows.len(), 2);
    assert!((s.text_shadows[0].x - 0.75).abs() < 0.01);
    assert!((s.text_shadows[0].blur - 2.25).abs() < 0.01);
    assert!(s.text_shadows[0].color.is_some());
    // No colour named: the text's own.
    assert!(s.text_shadows[1].color.is_none());
    // Inherited.
    let child = style_of(
        "<div style=\"text-shadow: 1px 1px red\"><p id=\"a\">x</p></div>",
        "",
        "a",
    );
    assert_eq!(child.text_shadows.len(), 1);
    let plain = pdf("<p>Shadowed words</p>");
    let shadowed = pdf("<p style=\"text-shadow: 1pt 1pt 2pt #888\">Shadowed words</p>");
    assert_eq!(shadowed.text, plain.text, "the words are there once");
    assert!(
        shadowed.bytes.len() > plain.bytes.len() + 200,
        "the shadow is drawn: {} vs {}",
        shadowed.bytes.len(),
        plain.bytes.len()
    );
}

#[test]
fn structural_and_relational_pseudo_classes_match() {
    let html = "<ul><li id=\"a\">a</li><li id=\"b\">b</li><li id=\"c\">c</li></ul>
        <div id=\"card\"><img src=\"x.png\"><p id=\"only\">t</p></div>
        <div id=\"plain\"><p>t</p></div>
        <h2 id=\"h\">h</h2><p id=\"after\">p</p>
        <div dir=\"rtl\" lang=\"ar-EG\"><span id=\"ar\">ع</span></div>";
    let css = "li:nth-last-child(1) { color: #ff0000 }
        li:nth-last-of-type(2) { color: #00ff00 }
        p:only-of-type { font-weight: 700 }
        div:has(> img) { color: #0000ff }
        h2:has(+ p) { color: #ff00ff }
        span:dir(rtl):lang(ar) { color: #00ffff }";
    let red = |id: &str| style_of(html, css, id).color;
    assert!(red("c").r > 0.9 && red("c").g < 0.1);
    assert!(red("b").g > 0.9);
    assert!(red("a").r < 0.1 && red("a").g < 0.1);
    assert_eq!(style_of(html, css, "only").font_weight, 700);
    assert!(red("card").b > 0.9);
    assert!(red("plain").b < 0.1);
    assert!(red("h").r > 0.9 && red("h").b > 0.9);
    assert!(red("ar").g > 0.9 && red("ar").b > 0.9);
    // A pseudo-class the engine does not know leaves its rule out, and
    // says so once; one only a screen has leaves it out quietly.
    let out = pdf(
        "<html><head><style>p:frobnicate { color: red } p:frobnicate > b { color: red } a:hover { color: red }</style></head><body><p>x</p></body></html>",
    );
    let unknown: Vec<&String> = out
        .notes
        .iter()
        .filter(|n| n.contains("frobnicate"))
        .collect();
    assert_eq!(unknown.len(), 1, "{:?}", out.notes);
    assert!(
        !out.notes.iter().any(|n| n.contains("hover")),
        "{:?}",
        out.notes
    );
}

#[test]
fn a_pdf_is_tagged_with_its_structure_and_language() {
    let html = "<html lang=\"en\"><body>
        <h1>Report</h1>
        <p>Read <a href=\"https://example.com\">the site</a> first.</p>
        <ul><li>One</li><li>Two</li></ul>
        <table><thead><tr><th>Item</th></tr></thead><tbody><tr><td>Tea</td></tr></tbody></table>
        <svg aria-label=\"A square\" width=\"20\" height=\"20\" viewBox=\"0 0 20 20\"><rect width=\"20\" height=\"20\"/></svg>
        <svg aria-hidden=\"true\" width=\"20\" height=\"20\" viewBox=\"0 0 20 20\"><rect width=\"20\" height=\"20\"/></svg>
        </body></html>";
    let out = pdf(html);
    let bytes = String::from_utf8_lossy(&out.bytes);
    for tag in [
        "/StructTreeRoot",
        "/Marked true",
        "/Lang(en)",
        "/S/H1",
        "/S/P",
        "/S/Link",
        "/S/L",
        "/S/LI",
        "/S/LBody",
        "/S/Table",
        "/S/TH",
        "/S/TD",
        "/S/Figure",
        "/Alt(A square)",
    ] {
        assert!(bytes.contains(tag), "{tag} is in the file");
    }
    // The decorative picture is no figure.
    assert_eq!(bytes.matches("/S/Figure").count(), 1);
    // Untagged, the structure is left out.
    let plain = pdf_tagged(html, false);
    assert!(!String::from_utf8_lossy(&plain.bytes).contains("/StructTreeRoot"));
}

#[test]
fn the_reference_documents_are_tagged() {
    let (out, tmp) = build_document("invoice");
    let pdf = std::fs::read_dir(tmp.join("documents/invoice/build"))
        .unwrap()
        .flatten()
        .find(|e| e.path().extension().is_some_and(|x| x == "pdf"))
        .unwrap()
        .path();
    let bytes = std::fs::read(&pdf).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/StructTreeRoot"), "{out}");
    assert!(text.contains("/S/Table") && text.contains("/S/H1"));
    let _ = std::fs::remove_dir_all(&tmp);
}
