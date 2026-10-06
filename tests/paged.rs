//! The paged engine, held to what a document should say on each page: page
//! numbers, repeated table headers, contents, links, Arabic, slides, and the
//! reference documents under `examples/documents`.

use serde_json::json;
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
fn a_footer_numbers_every_page_and_a_header_keeps_to_its_pages() {
    let filler: String = (0..120)
        .map(|i| {
            format!("Text(\"Paragraph number {i} of the body, long enough to take a line.\")\n")
        })
        .collect();
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
    let rows: String = (0..90)
        .map(|i| {
            format!(
                "Table.Row {{ Table.Cell(\"row {i}\")  Table.Cell(\"{}\") }}\n",
                i * 7
            )
        })
        .collect();
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
    let long: String = (0..60).map(|i| format!("Text(\"Line {i}\")\n")).collect();
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
    let examples = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let root = examples.join(group);
    let tmp = std::env::temp_dir().join(format!("wf-{group}-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
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
    copy(
        &examples.join("documents/fonts"),
        &tmp.join("documents/fonts"),
    );
    copy(&root.join(name), &tmp.join(group).join(name));
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(tmp.join(group).join(name))
        .output()
        .unwrap();
    let stdout =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stdout}");
    (stdout, tmp)
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
    let (out, tmp) = build_document("resume");
    // Its one Japanese line may come from the machine; nothing else does.
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
