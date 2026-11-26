//! Paged output: PDF documents and slide decks.
//!
//! A page is the static HTML and CSS the template engine renders, laid out
//! and printed: the HTML is read into a tree, styled by the page's sheets,
//! turned into boxes, laid out (taffy for block, flex and grid; our own
//! inline layout for text), cut into pages, painted, and written by krilla.

pub mod boxes;
pub mod cascade;
pub mod css;
pub mod fonts;
pub mod html;
pub mod icons;
pub mod layout;
pub mod page;
pub mod paginate;
pub mod paint;
pub mod style;
pub mod style_value;
pub mod tags;
pub mod text;
pub mod toc;

use std::path::PathBuf;

use krilla::geom::{Point, Rect, Transform};
use krilla::page::PageSettings;
use krilla::tagging::{Artifact, ArtifactType, ContentTag};
use krilla::{Document, SerializeSettings};

use boxes::{AssetReader, Builder, On, Tree};
use cascade::{Styler, UA_SHEET};
use css::{Media, Sheet};
use fonts::FontDb;
use html::Dom;
use layout::{Frag, Layout};
use paint::Painter;
use style::{Rgba, Style, Units};

/// What a paged output adds to the browser's defaults: how its own
/// elements look before a page's sheets say otherwise.
pub const PAGED_SHEET: &str = r"
.wf-document, .wf-page-break { display: block }
.wf-watermark { font-size: 72pt; font-weight: bold; color: rgba(0, 0, 0, 0.08); white-space: nowrap; display: block }
";

/// The paper.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Paper {
    pub width: f32,
    pub height: f32,
    /// Top, right, bottom, left.
    pub margin: [f32; 4],
}

impl Paper {
    pub fn content_width(&self) -> f32 {
        (self.width - self.margin[1] - self.margin[3]).max(10.0)
    }
    pub fn content_height(&self) -> f32 {
        (self.height - self.margin[0] - self.margin[2]).max(10.0)
    }
}

/// A paper size by name or by its measurements: `A4`, `Letter`,
/// `210mm 297mm`, `800x600`.
pub fn paper_size(name: &str) -> Option<(f32, f32)> {
    let n = name.trim();
    let named = match n.to_ascii_lowercase().as_str() {
        "a3" => Some((841.89, 1190.55)),
        "a4" => Some((595.28, 841.89)),
        "a5" => Some((419.53, 595.28)),
        "a6" => Some((297.64, 419.53)),
        "b5" => Some((498.9, 708.66)),
        "letter" => Some((612.0, 792.0)),
        "legal" => Some((612.0, 1008.0)),
        "tabloid" | "ledger" => Some((792.0, 1224.0)),
        "executive" => Some((522.0, 756.0)),
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    let units = Units {
        em: 12.0,
        rem: 12.0,
        vw: 0.0,
        vh: 0.0,
    };
    let parts: Vec<&str> = n
        .split(|c: char| c == 'x' || c == '×' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .collect();
    if let [w, h] = parts.as_slice() {
        let w = style::length_pt(w, &units)?;
        let h = style::length_pt(h, &units)?;
        return Some((w, h));
    }
    None
}

pub struct Options<'a> {
    pub paper: Paper,
    /// The document's base font size, in points.
    pub font_size: f32,
    /// The document's base family, when the page's sheets name none.
    pub font_family: String,
    pub assets: &'a AssetReader<'a>,
    /// Where the fonts in [`font_dirs`](Self::font_dirs) are read from.
    pub vfs: &'a dyn crate::base::vfs::Vfs,
    /// Directories whose fonts the document may use.
    pub font_dirs: Vec<PathBuf>,
    /// Whether the machine's fonts may stand in for a family or a character
    /// nothing else has.
    pub system_fonts: bool,
    /// Whether to write a tagged PDF: a structure of headings, paragraphs,
    /// lists, tables, figures and links over what is drawn, and everything
    /// else (backgrounds, borders, running headers and footers) marked as
    /// an artifact.
    pub tagged: bool,
}

impl std::fmt::Debug for Options<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Options")
            .field("paper", &self.paper)
            .field("font_size", &self.font_size)
            .field("font_family", &self.font_family)
            .field("font_dirs", &self.font_dirs)
            .field("system_fonts", &self.system_fonts)
            .field("tagged", &self.tagged)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Output {
    pub bytes: Vec<u8>,
    pub pages: usize,
    /// The text each page draws, a line at a time in reading order.
    pub text: Vec<Vec<String>>,
    /// What the reader of the build should know: characters no font had, a
    /// family taken from this machine.
    pub notes: Vec<String>,
}

/// The author's sheet: the engine's paged rules, then `defaults` (rules the
/// configuration implies, which the page's own may override), then every
/// `<style>` of the page, then `extra` — each `@media` query asked of
/// `media`, the page area.
fn sheets(dom: &Dom, defaults: &str, extra: &str, media: Media) -> Sheet {
    let mut sheet = Sheet::parse_for(PAGED_SHEET, media);
    if !defaults.is_empty() {
        sheet.extend(Sheet::parse_for(defaults, media));
    }
    for n in dom.descendants(0) {
        if dom.tag(n) == Some("style") {
            sheet.extend(Sheet::parse_for(&dom.text_content(n), media));
        }
    }
    if !extra.is_empty() {
        sheet.extend(Sheet::parse_for(extra, media));
    }
    sheet
}

/// What the reader of the build should know about the sheets: a selector
/// naming a pseudo-class the engine does not know, an `@page` margin box.
fn sheet_notes(sheet: &Sheet) -> Vec<String> {
    let mut out: Vec<String> = sheet
        .unknown
        .iter()
        .map(|p| format!("the PDF engine does not know `{p}`; the rules that name it are left out"))
        .collect();
    out.extend(sheet.unsupported.iter().map(|u| {
        format!("{u} is not drawn; a document's `Header` and `Footer` go in the margins")
    }));
    out
}

/// Whether the document reads right to left: its root's or body's `dir`.
fn document_rtl(dom: &Dom) -> bool {
    ["html", "body"].iter().any(|t| {
        dom.find_tag(t)
            .and_then(|n| dom.element(n))
            .and_then(|e| e.attr("dir"))
            .is_some_and(|d| d.eq_ignore_ascii_case("rtl"))
    })
}

/// The root style: the base size and family, with the page's `html` and
/// `body` rules on top.
fn root_style(opts: &Options<'_>) -> Style {
    let family = match opts.font_family.as_str() {
        "" => "sans-serif".to_string(),
        f => fonts::alias(f).map_or_else(|| f.to_string(), std::string::ToString::to_string),
    };
    Style::root(opts.font_size, &family)
}

/// The paper a document asks for on its `Document` element, over `base`:
/// the configured paper with the sheets' `@page` rules applied (see
/// [`page`] for the order).
fn document_paper(dom: &Dom, doc: Option<usize>, base: Paper) -> Paper {
    let Some(el) = doc.and_then(|d| dom.element(d)) else {
        return base;
    };
    let mut paper = base;
    if let Some((w, h)) = el
        .attr("data-size")
        .or_else(|| el.attr("data-page-size"))
        .and_then(paper_size)
    {
        paper.width = w;
        paper.height = h;
    }
    if el.attr("data-landscape").is_some_and(|v| v != "false") && paper.height > paper.width {
        std::mem::swap(&mut paper.width, &mut paper.height);
    }
    if let Some(m) = el.attr("data-margin") {
        let units = Units {
            em: 12.0,
            rem: 12.0,
            vw: paper.width,
            vh: paper.height,
        };
        let v: Vec<f32> = m
            .split_whitespace()
            .filter_map(|p| style::length_pt(p, &units))
            .collect();
        if let Some(f) = style::four(&v) {
            paper.margin = f;
        }
    }
    paper
}

fn metadata(dom: &Dom, doc: Option<usize>) -> krilla::metadata::Metadata {
    let el = doc.and_then(|d| dom.element(d));
    let get = |name: &str| {
        el.and_then(|e| e.attr(name))
            .map(std::string::ToString::to_string)
            .filter(|s| !s.is_empty())
    };
    let mut m = krilla::metadata::Metadata::new().creator("WebFluent".to_string());
    let title = get("data-title").or_else(|| {
        dom.find_tag("title")
            .map(|t| dom.text_content(t).trim().to_string())
            .filter(|t| !t.is_empty())
    });
    if let Some(t) = title {
        m = m.title(t);
    }
    if let Some(a) = get("data-author") {
        m = m.authors(a.split(',').map(|s| s.trim().to_string()).collect());
    }
    if let Some(s) = get("data-subject") {
        m = m.description(s);
    }
    if let Some(k) = get("data-keywords") {
        m = m.keywords(k.split(',').map(|s| s.trim().to_string()).collect());
    }
    let lang = get("data-lang").or_else(|| {
        dom.find_tag("html")
            .and_then(|h| dom.element(h))
            .and_then(|e| e.attr("lang"))
            .map(std::string::ToString::to_string)
    });
    if let Some(l) = lang {
        m = m.language(l);
    }
    m
}

/// The `position: fixed` boxes, each laid out on its own against the page
/// box — as paged media has it, a fixed box is drawn on every page, placed
/// by its insets from the paper's edges (a percentage of the paper's width
/// or height), and as wide as its content unless both `left` and `right`
/// (or a `width`) say otherwise. One with no inset on a side sits at the
/// first page's margin on that side.
fn fixed_frags(tree: &Tree, db: &FontDb, geometry: &page::Geometry) -> Vec<Frag> {
    let (pw, ph) = (geometry.width, geometry.height);
    let m = geometry.first;
    let layout = Layout::new(tree, db);
    let mut out = Vec::new();
    for (b, lb) in tree.boxes.iter().enumerate() {
        let s = &lb.style;
        if s.position != style::Position::Fixed
            || s.display == style::Display::None
            || b == tree.flow
        {
            continue;
        }
        let inset = |i: usize| s.inset[i].resolve(if i.is_multiple_of(2) { ph } else { pw });
        let (top, right, bottom, left) = (inset(0), inset(1), inset(2), inset(3));
        let margin = |i: usize| s.margin[i].or_zero(pw);
        let mut frag = match (left, right) {
            (Some(l), Some(r)) if s.width == style::Length::Auto => {
                let w = (pw - l - r - margin(1) - margin(3)).max(1.0);
                match (top, bottom) {
                    (Some(t), Some(bo)) if s.height == style::Length::Auto => {
                        let h = (ph - t - bo - margin(0) - margin(2)).max(1.0);
                        layout.layout_page(b, w, h)
                    }
                    _ => layout.layout(b, w),
                }
            }
            _ if s.width != style::Length::Auto => {
                layout.layout(b, s.width.resolve(pw).unwrap_or(pw))
            }
            _ => layout.layout_shrink(b, (pw - left.unwrap_or(0.0) - right.unwrap_or(0.0)).max(1.0)),
        };
        let x = match (left, right) {
            (Some(l), _) => l + margin(3),
            (None, Some(r)) => pw - r - margin(1) - frag.w,
            (None, None) => m[3] + margin(3),
        };
        let y = match (top, bottom) {
            (Some(t), _) => t + margin(0),
            (None, Some(bo)) => ph - bo - margin(2) - frag.h,
            (None, None) => m[0] + margin(0),
        };
        frag.shift(x - frag.x, y - frag.y);
        frag.measure_extent();
        out.push(frag);
    }
    out
}

/// Where every box's frag sits, by box.
fn positions(f: &Frag, out: &mut std::collections::HashMap<usize, (f32, f32)>) {
    out.entry(f.boxed).or_insert((f.x, f.y));
    for c in f.children.iter().chain(f.atoms.iter()) {
        positions(c, out);
    }
}

/// What a PDF or a deck says in a build without the `pdf` feature.
#[cfg(not(wf_pdf))]
const NO_PDF: &str = "this build of the compiler has no PDF engine (a WebAssembly build without the `pdf` feature); build a PDF or a slide deck with `wf build`";

/// Render a page's HTML (with its `<style>` elements) and any further CSS to
/// a PDF.
///
/// # Errors
///
/// A message when the paper has no size or the PDF cannot be written, and
/// in a build without the `pdf` feature always.
#[cfg_attr(
    not(wf_pdf),
    allow(
        unreachable_code,
        unused_variables,
        reason = "without the PDF engine every render refuses before reading its arguments"
    )
)]
pub fn render(html: &str, css: &str, opts: &Options<'_>) -> Result<Output, String> {
    // The running elements, each built for each page it is drawn on.
    struct RunningFrag {
        page: usize,
        kind: u8,
        tree: Tree,
        frag: Frag,
    }

    #[cfg(not(wf_pdf))]
    {
        let _ = (html, css, opts);
        return Err(NO_PDF.to_string());
    }
    let dom = Dom::parse(html);
    let doc = dom
        .descendants(0)
        .into_iter()
        .find(|&n| dom.element(n).is_some_and(|e| e.has_class("wf-document")));
    // The page box first — the configuration, then the sheets' `@page`,
    // then the document's own size — and then the sheets again, their
    // `@media` queries asked of the page area it leaves.
    let configured = Media::page(opts.paper.content_width(), opts.paper.content_height());
    let probe = sheets(&dom, "", css, configured);
    let (geometry, page_notes) = page::resolve(
        opts.paper,
        &probe.pages,
        |p| document_paper(&dom, doc, p),
        document_rtl(&dom),
        opts.font_size,
    );
    let paper = geometry.paper();
    let media = Media::page(paper.content_width(), paper.content_height());
    let author = if media == configured {
        probe
    } else {
        sheets(&dom, "", css, media)
    };
    let mut db = FontDb::new(opts.system_fonts);
    for dir in &opts.font_dirs {
        db.add_dir_via(opts.vfs, dir);
    }
    // The sheets' own fonts.
    let mut font_notes = Vec::new();
    for face in &author.font_faces {
        let mut found = false;
        for src in &face.sources {
            if let Some(bytes) = (opts.assets)(src) {
                let known_as = std::path::PathBuf::from(format!(
                    "@font-face/{}",
                    face.family.replace(' ', "_")
                ));
                if db.add_bytes(bytes, &known_as, Some(&face.family)) > 0 {
                    found = true;
                    break;
                }
            }
        }
        if !found && !face.sources.is_empty() {
            font_notes.push(format!(
                "the font `{}` could not be read from {}",
                face.family,
                face.sources.join(", ")
            ));
        }
    }

    let ua = Sheet::parse(UA_SHEET);
    let base = root_style(opts);
    // `rem` is the root element's font size: work it out, then style for real.
    let first = Styler::new(
        &ua,
        &author,
        Units {
            em: opts.font_size,
            rem: opts.font_size,
            vw: paper.width,
            vh: paper.height,
        },
    );
    let rem = dom
        .find_tag("html")
        .map_or(opts.font_size, |h| first.compute(&dom, h, &base).font_size);
    let styler = Styler::new(
        &ua,
        &author,
        Units {
            em: rem,
            rem,
            vw: paper.width,
            vh: paper.height,
        },
    );
    let cw = paper.content_width();
    let bands = geometry.bands();

    // A table of contents: entries with a placeholder for each page, laid
    // out once to learn where each heading lands, then filled in.
    let mut dom = dom;
    let toc = toc::insert(&mut dom);
    let lay = |dom: &Dom, db: &mut FontDb| {
        let tree = Builder::new(dom, &styler, db, opts.assets).build(&base);
        let layout = Layout::new(&tree, db);
        let mut flow = layout.layout(tree.flow, cw);
        let pages = paginate::paginate_bands(&mut flow, &tree, bands);
        (tree, flow, pages)
    };
    let (mut tree, mut flow, mut pages) = lay(&dom, &mut db);
    if let Some(entries) = toc {
        for _ in 0..2 {
            let mut box_pos = std::collections::HashMap::new();
            positions(&flow, &mut box_pos);
            let changed = toc::fill(&mut dom, &entries, |node| {
                let b = tree.boxes.iter().position(|b| b.node == Some(node))?;
                let (_, y) = box_pos.get(&b)?;
                Some(bands.index_of(*y) + 1)
            });
            if !changed {
                break;
            }
            (tree, flow, pages) = lay(&dom, &mut db);
        }
    }
    let dom = dom;
    flow.measure_extent();
    let mut extra = std::mem::take(&mut pages.extra);
    for e in &mut extra {
        e.measure_extent();
    }
    let count = pages.count;
    let fixed = fixed_frags(&tree, &db, &geometry);

    let mut running: Vec<RunningFrag> = Vec::new();
    let lists: [(u8, &Vec<(On, usize)>); 4] = [
        (0, &tree.running.backgrounds),
        (1, &tree.running.watermarks),
        (2, &tree.running.headers),
        (3, &tree.running.footers),
    ];
    for page in 1..=count {
        for (kind, list) in lists {
            for (on, node) in list {
                if !on.applies(page) {
                    continue;
                }
                let rt = Builder::new(&dom, &styler, &mut db, opts.assets)
                    .build_running(*node, page, count);
                let mut frag = {
                    let layout = Layout::new(&rt, &db);
                    match kind {
                        0 => layout.layout_page(rt.flow, paper.width, paper.height),
                        1 => layout.layout_natural(rt.flow),
                        _ => layout.layout(rt.flow, cw),
                    }
                };
                // Where it goes on the page, in that page's margins.
                let m = geometry.margins(page - 1);
                let (x, y) = match kind {
                    0 => (0.0, 0.0),
                    1 => ((paper.width - frag.w) / 2.0, (paper.height - frag.h) / 2.0),
                    2 => (m[3], ((m[0] - frag.h) / 2.0).max(0.0)),
                    _ => (
                        m[3],
                        paper.height - m[2] + ((m[2] - frag.h) / 2.0).max(0.0),
                    ),
                };
                frag.shift(x - frag.x, y - frag.y);
                frag.measure_extent();
                running.push(RunningFrag {
                    page,
                    kind,
                    tree: rt,
                    frag,
                });
            }
        }
    }

    // The canvas: the root's or the body's background fills every page —
    // and the body's box does not paint it again over a page's background.
    let canvas = {
        let html_bg = dom
            .find_tag("html")
            .map(|h| styler.compute(&dom, h, &base).background_color);
        let body_bg = tree.boxes[tree.flow].style.background_color;
        html_bg.filter(style::Rgba::is_visible).unwrap_or(body_bg)
    };
    if canvas.is_visible() {
        let mut body = (*tree.boxes[tree.flow].style).clone();
        body.background_color = Rgba::TRANSPARENT;
        body.background_images.clear();
        tree.boxes[tree.flow].style = std::rc::Rc::new(body);
    }

    let settings = SerializeSettings {
        enable_tagging: opts.tagged,
        ..SerializeSettings::default()
    };
    let mut document = Document::new_with(settings);
    document.set_metadata(metadata(&dom, doc));
    let mut box_pos = std::collections::HashMap::new();
    positions(&flow, &mut box_pos);
    let mut painter = Painter::new(&tree, &db);
    painter.tagging = opts.tagged;
    // The link annotations, as tagged content of the element they are on.
    let mut link_leaves: Vec<paint::Leaf> = Vec::new();
    // Where a place on the strip lands: its page, and its point there.
    let place = |x: f32, y: f32| {
        let p = bands.index_of(y).min(count - 1);
        let m = geometry.margins(p);
        (p, Point::from_xy(x + m[3], y - bands.top(p) + m[0]))
    };
    let mut page_text: Vec<Vec<String>> = Vec::new();
    for page_ix in 0..count {
        painter.text.clear();
        let settings =
            PageSettings::from_wh(paper.width, paper.height).ok_or("a page with no size")?;
        let mut page = document.start_page_with(settings);
        let mut links = Vec::new();
        let m = geometry.margins(page_ix);
        let (top, band_h) = (bands.top(page_ix), bands.height(page_ix));
        {
            let mut s = page.surface();
            // Everything the page draws beside its own content — the
            // canvas, backgrounds, watermarks, headers, footers, fixed
            // boxes, a repeated table header — is an artifact.
            let artifact = |s: &mut krilla::surface::Surface<'_>, kind| {
                if opts.tagged {
                    s.start_tagged(ContentTag::Artifact(Artifact::new(kind, None)));
                }
            };
            let end = |s: &mut krilla::surface::Surface<'_>| {
                if opts.tagged {
                    s.end_tagged();
                }
            };
            if canvas.is_visible()
                && let Some(p) = paint::rounded(0.0, 0.0, paper.width, paper.height, [0.0; 4])
            {
                artifact(&mut s, ArtifactType::Layout);
                s.set_stroke(None);
                s.set_fill(Some(krilla::paint::Fill {
                    paint: krilla::color::rgb::Color::new(
                        (canvas.r * 255.0) as u8,
                        (canvas.g * 255.0) as u8,
                        (canvas.b * 255.0) as u8,
                    )
                    .into(),
                    opacity: krilla::num::NormalizedF32::ONE,
                    rule: krilla::paint::FillRule::default(),
                }));
                s.draw_path(&p);
                end(&mut s);
            }
            painter.artifact = true;
            // Backgrounds and watermarks, behind the content.
            for r in running
                .iter()
                .filter(|r| r.page == page_ix + 1 && r.kind <= 1)
            {
                painter.tree = &r.tree;
                painter.band = (f32::MIN, f32::MAX);
                painter.dx = 0.0;
                painter.dy = 0.0;
                if r.kind == 1 {
                    artifact(&mut s, ArtifactType::Watermark);
                    let style = &r.tree.boxes[r.tree.flow].style;
                    let angle = if style.rotate.abs() > 0.01 {
                        0.0
                    } else {
                        -45.0
                    };
                    s.push_transform(&Transform::from_rotate_at(
                        angle,
                        paper.width / 2.0,
                        paper.height / 2.0,
                    ));
                    painter.frag(&mut s, &r.frag);
                    s.pop();
                } else {
                    artifact(&mut s, ArtifactType::Layout);
                    painter.frag(&mut s, &r.frag);
                }
                end(&mut s);
                links.append(&mut painter.links);
            }
            // The content: this page's band of the strip.
            painter.tree = &tree;
            painter.band = (top, top + band_h);
            painter.dx = m[3];
            painter.dy = m[0] - top;
            if let Some(clip) = paint::rounded(0.0, m[0], paper.width, band_h, [0.0; 4]) {
                s.push_clip_path(&clip, &krilla::paint::FillRule::NonZero);
                s.push_transform(&Transform::from_translate(m[3], m[0] - top));
                // A repeated table header first: it is what the page reads
                // first.
                for e in extra
                    .iter()
                    .filter(|e| e.y >= top - 0.5 && e.y < top + band_h)
                {
                    artifact(&mut s, ArtifactType::PaginationOther);
                    painter.frag(&mut s, e);
                    end(&mut s);
                }
                painter.artifact = false;
                painter.frag(&mut s, &flow);
                painter.artifact = true;
                s.pop();
                s.pop();
            }
            links.append(&mut painter.links);
            // The fixed boxes, the same on every page, over the content.
            painter.band = (f32::MIN, f32::MAX);
            painter.dx = 0.0;
            painter.dy = 0.0;
            for f in &fixed {
                artifact(&mut s, ArtifactType::PaginationOther);
                painter.frag(&mut s, f);
                end(&mut s);
            }
            links.append(&mut painter.links);
            // Headers and footers, in the margins.
            for r in running
                .iter()
                .filter(|r| r.page == page_ix + 1 && r.kind >= 2)
            {
                painter.tree = &r.tree;
                painter.band = (f32::MIN, f32::MAX);
                painter.dx = 0.0;
                painter.dy = 0.0;
                artifact(
                    &mut s,
                    if r.kind == 2 {
                        ArtifactType::Header
                    } else {
                        ArtifactType::Footer
                    },
                );
                painter.frag(&mut s, &r.frag);
                end(&mut s);
                links.append(&mut painter.links);
            }
            painter.artifact = false;
            s.finish();
        }
        page_text.push(std::mem::take(&mut painter.text));
        for l in links {
            let (x, y, w, h) = l.rect;
            let Some(rect) = Rect::from_xywh(x, y, w.max(1.0), h.max(1.0)) else {
                continue;
            };
            let target = if let Some(id) = l.href.strip_prefix('#') {
                // A link within the document: the page and place of the
                // element with that id.
                let found = dom
                    .descendants(0)
                    .into_iter()
                    .find(|&n| dom.element(n).and_then(|e| e.attr("id")) == Some(id))
                    .and_then(|n| tree.boxes.iter().position(|b| b.node == Some(n)))
                    .and_then(|b| box_pos.get(&b).copied());
                let Some((bx, by)) = found else { continue };
                let (p, at) = place(bx, by);
                krilla::annotation::Target::Destination(krilla::destination::Destination::Xyz(
                    krilla::destination::XyzDestination::new(p, at),
                ))
            } else {
                krilla::annotation::Target::Action(krilla::action::Action::Link(
                    krilla::action::LinkAction::new(l.href.clone()),
                ))
            };
            let annotation = krilla::annotation::Annotation::new_link(
                krilla::annotation::LinkAnnotation::new(rect, target),
                Some(l.href),
            );
            // A link in the content is tagged with the element it is on; one
            // in a header or a footer is not part of the structure.
            match l.node.filter(|_| opts.tagged) {
                Some(node) => {
                    let id = page.add_tagged_annotation(annotation);
                    link_leaves.push(paint::Leaf {
                        node,
                        seq: l.seq,
                        label: false,
                        id,
                    });
                }
                _ => page.add_annotation(annotation),
            }
        }
        page.finish();
    }
    if opts.tagged {
        let mut leaves = std::mem::take(&mut painter.leaves);
        leaves.extend(link_leaves);
        document.set_tag_tree(tags::tree(&dom, leaves));
    }

    // The outline: the headings, nested by level.
    let mut outline = krilla::outline::Outline::new();
    let mut stack: Vec<(u8, krilla::outline::OutlineNode)> = Vec::new();
    let finish = |stack: &mut Vec<(u8, krilla::outline::OutlineNode)>,
                  outline: &mut krilla::outline::Outline,
                  level: u8| {
        while let Some((_, node)) = stack.pop_if(|(l, _)| *l >= level) {
            match stack.last_mut() {
                Some((_, parent)) => parent.push_child(node),
                None => outline.push_child(node),
            }
        }
    };
    for &(b, level) in &tree.headings {
        let Some(&(x, y)) = box_pos.get(&b) else {
            continue;
        };
        let Some(node) = tree.boxes[b].node else {
            continue;
        };
        let text = dom
            .text_content(node)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if text.is_empty() {
            continue;
        }
        finish(&mut stack, &mut outline, level);
        let (p, at) = place(x, y);
        let dest = krilla::destination::XyzDestination::new(p, at);
        stack.push((level, krilla::outline::OutlineNode::new(text, dest)));
    }
    finish(&mut stack, &mut outline, 0);
    document.set_outline(outline);

    let bytes = document
        .finish()
        .map_err(|e| format!("the PDF could not be written: {e:?}"))?;
    let mut notes = font_notes;
    notes.extend(page_notes);
    notes.extend(sheet_notes(&author));
    notes.extend(tree.notes.iter().cloned());
    for (ch, _) in &tree.missing {
        notes.push(format!(
            "no font has `{ch}` (U+{:04X}); put one that does under fonts/",
            *ch as u32
        ));
    }
    for family in &db.system_used {
        notes.push(format!("the font `{family}` came from this machine; put it under fonts/ so the document is the same everywhere"));
    }
    Ok(Output {
        bytes,
        pages: count,
        text: page_text,
        notes,
    })
}

// ─── Configuration ──────────────────────────────────────────────────

/// Reads a page's assets from a project: an address is looked up beside
/// the project, in its `public/` and in its `src/`.
pub fn reader(root: Option<PathBuf>) -> impl Fn(&str) -> Option<Vec<u8>> {
    reader_via(&crate::base::vfs::FsVfs, root)
}

/// [`reader`], reading through `vfs`.
pub fn reader_via(
    vfs: &dyn crate::base::vfs::Vfs,
    root: Option<PathBuf>,
) -> impl Fn(&str) -> Option<Vec<u8>> + '_ {
    move |src: &str| {
        if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("data:") {
            // The compiler never fetches anything.
            return None;
        }
        let root = root.as_ref()?;
        let rel = src
            .split(['?', '#'])
            .next()
            .unwrap_or(src)
            .trim_start_matches('/');
        let rel = rel.trim_start_matches("./");
        // An address names a file under one of the places below, and only
        // there: it may come from a template's data, and `../../x` would
        // embed whatever the process can read.
        if rel.contains(['\\', '\0'])
            || std::path::Path::new(rel).components().any(|c| {
                !matches!(
                    c,
                    std::path::Component::Normal(_) | std::path::Component::CurDir
                )
            })
        {
            return None;
        }
        let mut bases = vec![root.clone(), root.join("public"), root.join("src")];
        if let Some(parent) = root.parent() {
            bases.push(parent.join("public"));
        }
        bases
            .into_iter()
            .map(|base| {
                let file = base.join(rel);
                (base, file)
            })
            .find(|(base, file)| vfs.is_file(file) && stays_under(vfs, base, file))
            .and_then(|(_, file)| vfs.read(&file).ok())
    }
}

/// Whether `file` is under `base` once every link is resolved — on a file
/// system that has links; one that has none keeps a path where it says.
fn stays_under(
    vfs: &dyn crate::base::vfs::Vfs,
    base: &std::path::Path,
    file: &std::path::Path,
) -> bool {
    match (vfs.canonical(base), vfs.canonical(file)) {
        (Some(base), Some(file)) => file.starts_with(base),
        // Not on a disk (a buffer, a file system with no links).
        (_, None) => true,
        (None, Some(_)) => false,
    }
}

#[cfg(test)]
mod reader_tests {
    use super::*;

    #[test]
    fn an_address_reads_a_file_under_the_project_and_nothing_else() {
        let base = std::env::temp_dir().join(format!("wf-reader-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("site/templates");
        std::fs::create_dir_all(root.join("public")).unwrap();
        std::fs::create_dir_all(base.join("site/public")).unwrap();
        std::fs::create_dir_all(base.join("private")).unwrap();
        std::fs::write(base.join("private/scan.png"), "SECRET").unwrap();
        std::fs::write(root.join("public/logo.png"), "logo").unwrap();
        std::fs::write(base.join("site/public/shared.png"), "shared").unwrap();
        let read = reader(Some(root.clone()));
        assert_eq!(read("/logo.png").as_deref(), Some(&b"logo"[..]));
        assert_eq!(read("logo.png?v=2").as_deref(), Some(&b"logo"[..]));
        // The project's parent's `public/`, as a template beside its site has.
        assert_eq!(read("/shared.png").as_deref(), Some(&b"shared"[..]));
        for bad in [
            "../../private/scan.png",
            "/../../private/scan.png",
            "public/../../../private/scan.png",
            "..\\..\\private\\scan.png",
            "logo.png\0",
        ] {
            assert_eq!(read(bad), None, "{bad}");
        }
        let abs = base.join("private/scan.png");
        assert_eq!(read(&abs.to_string_lossy()), None);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&abs, root.join("public/linked.png")).unwrap();
            std::os::unix::fs::symlink(base.join("private"), base.join("site/public/dir")).unwrap();
            assert_eq!(read("/linked.png"), None);
            assert_eq!(read("/dir/scan.png"), None);
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}

/// The directories a project's fonts are kept in.
pub fn font_dirs(root: Option<&std::path::Path>, extra: &[String]) -> Vec<PathBuf> {
    font_dirs_via(&crate::base::vfs::FsVfs, root, extra)
}

/// [`font_dirs`], looked for through `vfs`.
pub fn font_dirs_via(
    vfs: &dyn crate::base::vfs::Vfs,
    root: Option<&std::path::Path>,
    extra: &[String],
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(root) = root {
        for d in ["fonts", "src/fonts", "public/fonts", "assets/fonts"] {
            let p = root.join(d);
            if vfs.is_dir(&p) {
                out.push(p);
            }
        }
        if let Some(parent) = root.parent() {
            let p = parent.join("fonts");
            if vfs.is_dir(&p) && !out.contains(&p) {
                out.push(p);
            }
        }
        for e in extra {
            out.push(root.join(e));
        }
    }
    out
}

impl<'a> Options<'a> {
    pub fn from_pdf(
        config: &crate::project::config::project::PdfConfig,
        assets: &'a AssetReader<'a>,
        root: Option<&std::path::Path>,
    ) -> Options<'a> {
        Self::from_pdf_via(&crate::base::vfs::FsVfs, config, assets, root)
    }

    /// [`from_pdf`](Self::from_pdf), with the project's fonts read through
    /// `vfs`.
    pub fn from_pdf_via(
        vfs: &'a dyn crate::base::vfs::Vfs,
        config: &crate::project::config::project::PdfConfig,
        assets: &'a AssetReader<'a>,
        root: Option<&std::path::Path>,
    ) -> Options<'a> {
        let (width, height) = paper_size(&config.page_size).unwrap_or((595.28, 841.89));
        let m = &config.margins;
        Options {
            paper: Paper {
                width,
                height,
                margin: [m.top as f32, m.right as f32, m.bottom as f32, m.left as f32],
            },
            font_size: config.default_font_size as f32,
            font_family: config.default_font.clone(),
            assets,
            vfs,
            font_dirs: font_dirs_via(vfs, root, &config.fonts),
            system_fonts: config.system_fonts,
            // Every PDF is tagged: the configuration has no setting for it
            // yet (`build.pdf.tagged` would be one).
            tagged: true,
        }
    }

    pub fn from_slides(
        config: &crate::project::config::project::SlidesConfig,
        assets: &'a AssetReader<'a>,
        root: Option<&std::path::Path>,
    ) -> (Options<'a>, SlideChrome) {
        Self::from_slides_via(&crate::base::vfs::FsVfs, config, assets, root)
    }

    /// [`from_slides`](Self::from_slides), with the project's fonts read
    /// through `vfs`.
    pub fn from_slides_via(
        vfs: &'a dyn crate::base::vfs::Vfs,
        config: &crate::project::config::project::SlidesConfig,
        assets: &'a AssetReader<'a>,
        root: Option<&std::path::Path>,
    ) -> (Options<'a>, SlideChrome) {
        let (mut width, mut height) = match config.size.as_str() {
            "16:9" => (960.0, 540.0),
            "4:3" => (720.0, 540.0),
            "A4-landscape" => (841.89, 595.28),
            other => paper_size(other).unwrap_or((960.0, 540.0)),
        };
        if let Some(w) = config.width {
            width = w as f32;
        }
        if let Some(h) = config.height {
            height = h as f32;
        }
        let options = Options {
            paper: Paper {
                width,
                height,
                margin: [0.0; 4],
            },
            font_size: config.default_font_size as f32,
            font_family: config.default_font.clone(),
            assets,
            vfs,
            font_dirs: font_dirs_via(vfs, root, &config.fonts),
            system_fonts: config.system_fonts,
            tagged: false,
        };
        let chrome = SlideChrome {
            margin: config.margin as f32,
            numbers: config.show_slide_numbers,
            footer: config.footer_text.clone(),
            background: config.background_color.as_deref().and_then(style::color),
            color: config.chrome_color.as_deref().and_then(style::color),
        };
        (options, chrome)
    }
}

/// What a deck draws around every slide.
#[derive(Debug, Clone, Default)]
pub struct SlideChrome {
    pub margin: f32,
    pub numbers: bool,
    pub footer: Option<String>,
    pub background: Option<Rgba>,
    /// The chrome's colour; `None` picks a grey that reads on the slide.
    pub color: Option<Rgba>,
}

fn luminance(c: Rgba) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

/// Render a deck: every slide of the page's `Presentation` on a page of its
/// own, the slide's size, its content clipped at the edge.
///
/// # Errors
///
/// A message when a slide has no size or the PDF cannot be written, and in
/// a build without the `pdf` feature always.
#[cfg_attr(
    not(wf_pdf),
    allow(
        unreachable_code,
        unused_variables,
        reason = "without the PDF engine every render refuses before reading its arguments"
    )
)]
pub fn render_slides(
    html: &str,
    css: &str,
    opts: &Options<'_>,
    chrome: &SlideChrome,
) -> Result<Output, String> {
    #[cfg(not(wf_pdf))]
    {
        let _ = (html, css, opts, chrome);
        return Err(NO_PDF.to_string());
    }
    let dom = Dom::parse(html);
    let m = chrome.margin;
    // A slide is the page: its margin is its padding, unless the deck's own
    // rules say otherwise.
    let slide_css = format!(
        ".wf-slide, .wf-title-slide, .wf-section-slide, .wf-two-column, .wf-image-slide {{ padding: {m}pt; overflow: hidden; }}"
    );
    let author = sheets(
        &dom,
        &slide_css,
        css,
        Media::page(opts.paper.width, opts.paper.height),
    );
    let mut db = FontDb::new(opts.system_fonts);
    for dir in &opts.font_dirs {
        db.add_dir_via(opts.vfs, dir);
    }
    let ua = Sheet::parse(UA_SHEET);
    let base = root_style(opts);
    let paper = opts.paper;
    let styler = Styler::new(
        &ua,
        &author,
        Units {
            em: opts.font_size,
            rem: opts.font_size,
            vw: paper.width,
            vh: paper.height,
        },
    );
    let tree = Builder::new(&dom, &styler, &mut db, opts.assets).build(&base);
    // The slides: the boxes whose element is a child of the presentation.
    let presentation = dom.descendants(0).into_iter().find(|&n| {
        dom.element(n)
            .is_some_and(|e| e.has_class("wf-presentation"))
    });
    let slide_nodes: Vec<usize> = presentation
        .map(|p| dom.element_children(p).collect())
        .unwrap_or_default();
    let slides: Vec<usize> = slide_nodes
        .iter()
        .filter_map(|n| tree.boxes.iter().position(|b| b.node == Some(*n)))
        .collect();
    let mut notes = Vec::new();
    let mut frags = Vec::new();
    {
        let layout = Layout::new(&tree, &db);
        for (i, &b) in slides.iter().enumerate() {
            let natural = layout.layout(b, paper.width);
            if natural.h > paper.height + 0.5 {
                notes.push(format!(
                    "slide {} is taller than the slide; what overflows is clipped",
                    i + 1
                ));
            }
            let mut f = layout.layout_page(b, paper.width, paper.height);
            f.measure_extent();
            frags.push(f);
        }
    }
    let total = frags.len();
    // The chrome: a number and a footer, each a small paragraph of its own.
    let mut chrome_trees: Vec<(usize, Tree, Frag)> = Vec::new();
    for (i, f) in frags.iter().enumerate() {
        let bg = tree.boxes[f.boxed].style.background_color;
        let bg = if bg.is_visible() {
            bg
        } else {
            chrome.background.unwrap_or(Rgba::rgb(1.0, 1.0, 1.0))
        };
        let ink = chrome.color.unwrap_or(if luminance(bg) < 0.5 {
            Rgba::rgb(0.75, 0.75, 0.75)
        } else {
            Rgba::rgb(0.45, 0.45, 0.45)
        });
        let hex = format!(
            "#{:02x}{:02x}{:02x}",
            (ink.r * 255.0) as u8,
            (ink.g * 255.0) as u8,
            (ink.b * 255.0) as u8
        );
        let mut pieces = Vec::new();
        if let Some(text) = &chrome.footer {
            pieces.push((0u8, text.clone()));
        }
        if chrome.numbers {
            pieces.push((1u8, format!("{} / {}", i + 1, total)));
        }
        for (side, text) in pieces {
            let snippet = format!(
                "<html><body><p style=\"font-size: 11pt; color: {hex}; margin: 0\">{}</p></body></html>",
                text.replace('&', "&amp;").replace('<', "&lt;")
            );
            let sdom = Dom::parse(&snippet);
            let sstyler = Styler::new(
                &ua,
                &Sheet::default(),
                Units {
                    em: 11.0,
                    rem: 11.0,
                    vw: paper.width,
                    vh: paper.height,
                },
            );
            let st = Builder::new(&sdom, &sstyler, &mut db, opts.assets).build(&base);
            let mut sf = Layout::new(&st, &db).layout_natural(st.flow);
            let x = if side == 0 { m } else { paper.width - m - sf.w };
            let y = paper.height - m / 2.0 - sf.h / 2.0;
            sf.shift(x - sf.x, y - sf.y);
            sf.measure_extent();
            chrome_trees.push((i, st, sf));
        }
    }
    let mut document = Document::new_with(SerializeSettings::default());
    document.set_metadata(metadata(&dom, None));
    let mut painter = Painter::new(&tree, &db);
    let mut slide_text: Vec<Vec<String>> = Vec::new();
    for (i, f) in frags.iter().enumerate() {
        painter.text.clear();
        let settings =
            PageSettings::from_wh(paper.width, paper.height).ok_or("a slide with no size")?;
        let mut page = document.start_page_with(settings);
        let mut links = Vec::new();
        {
            let mut s = page.surface();
            if let Some(bg) = chrome.background
                && let Some(p) = paint::rounded(0.0, 0.0, paper.width, paper.height, [0.0; 4])
            {
                s.set_fill(Some(krilla::paint::Fill {
                    paint: krilla::color::rgb::Color::new(
                        (bg.r * 255.0) as u8,
                        (bg.g * 255.0) as u8,
                        (bg.b * 255.0) as u8,
                    )
                    .into(),
                    opacity: krilla::num::NormalizedF32::ONE,
                    rule: krilla::paint::FillRule::default(),
                }));
                s.draw_path(&p);
            }
            painter.tree = &tree;
            painter.band = (f32::MIN, f32::MAX);
            painter.dx = 0.0;
            painter.dy = 0.0;
            painter.frag(&mut s, f);
            links.append(&mut painter.links);
            for (_, st, sf) in chrome_trees.iter().filter(|(n, _, _)| *n == i) {
                painter.tree = st;
                painter.frag(&mut s, sf);
            }
            s.finish();
        }
        slide_text.push(std::mem::take(&mut painter.text));
        for l in links {
            let (x, y, w, h) = l.rect;
            if l.href.starts_with('#') {
                continue;
            }
            let Some(rect) = Rect::from_xywh(x, y, w.max(1.0), h.max(1.0)) else {
                continue;
            };
            let target = krilla::annotation::Target::Action(krilla::action::Action::Link(
                krilla::action::LinkAction::new(l.href.clone()),
            ));
            page.add_annotation(krilla::annotation::Annotation::new_link(
                krilla::annotation::LinkAnnotation::new(rect, target),
                Some(l.href),
            ));
        }
        page.finish();
    }
    if total == 0 {
        // A deck with no slides is still a document: one empty slide.
        let settings =
            PageSettings::from_wh(paper.width, paper.height).ok_or("a slide with no size")?;
        let page = document.start_page_with(settings);
        page.finish();
    }
    let bytes = document
        .finish()
        .map_err(|e| format!("the PDF could not be written: {e:?}"))?;
    notes.extend(tree.notes.iter().cloned());
    for (ch, _) in &tree.missing {
        notes.push(format!(
            "no font has `{ch}` (U+{:04X}); put one that does under fonts/",
            *ch as u32
        ));
    }
    for family in &db.system_used {
        notes.push(format!("the font `{family}` came from this machine; put it under fonts/ so the document is the same everywhere"));
    }
    Ok(Output {
        bytes,
        pages: total.max(1),
        text: slide_text,
        notes,
    })
}
