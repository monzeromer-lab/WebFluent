//! Slides codegen — emits PDF slide decks (one Slide = one page).
//!
//! Layout model: each `Slide`/`TitleSlide`/`SectionSlide`/`TwoColumn`/`ImageSlide`
//! becomes exactly one PDF page, no flow pagination. Content that overflows the
//! bottom margin is clipped and a stderr warning is emitted; the build does not fail.
//!
//! PDF byte plumbing (font metrics, content streams, serialization) is duplicated
//! from `pdf.rs` rather than extracted to a shared module — see implementation
//! plan for the rationale.

use crate::config::project::SlidesConfig;
use crate::parser::{Program, Declaration, Statement, UIElement, ComponentRef, Expr, StringPart, Arg};

// ─── Base14 Font Metrics ────────────────────────────────────────────

const HELVETICA_WIDTHS: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278,
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556,
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556,
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556,
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];

const HELVETICA_BOLD_WIDTHS: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278,
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611,
    975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556,
    333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611,
    611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
];

const TIMES_WIDTHS: [u16; 95] = [
    250, 333, 408, 500, 500, 833, 778, 180, 333, 333, 500, 564, 250, 333, 250, 278,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 278, 278, 564, 564, 564, 444,
    921, 722, 667, 667, 722, 611, 556, 722, 722, 333, 389, 722, 611, 889, 722, 722,
    556, 722, 667, 556, 611, 722, 722, 944, 722, 722, 611, 333, 278, 333, 469, 500,
    333, 444, 500, 444, 500, 444, 333, 500, 500, 278, 278, 500, 278, 778, 500, 500,
    500, 500, 333, 389, 278, 500, 500, 722, 500, 500, 444, 480, 200, 480, 541,
];

const COURIER_WIDTH: u16 = 600;

fn char_width(ch: char, font: &str) -> u16 {
    let code = ch as u32;
    if code < 32 || code > 126 { return 500; }
    let idx = (code - 32) as usize;
    if font.contains("Courier") { COURIER_WIDTH }
    else if font.contains("Times") { TIMES_WIDTHS[idx] }
    else if font.contains("Bold") { HELVETICA_BOLD_WIDTHS[idx] }
    else { HELVETICA_WIDTHS[idx] }
}

fn text_width(text: &str, font: &str, font_size: f64) -> f64 {
    let units: u32 = text.chars().map(|c| char_width(c, font) as u32).sum();
    units as f64 * font_size / 1000.0
}

fn slide_dimensions(size: &str, w_override: Option<f64>, h_override: Option<f64>) -> (f64, f64) {
    let (mut w, mut h) = match size.to_uppercase().as_str() {
        "16:9" => (960.0, 540.0),
        "4:3"  => (720.0, 540.0),
        "A4-LANDSCAPE" | "A4LANDSCAPE" => (841.89, 595.28),
        s => parse_explicit_dims(s).unwrap_or((960.0, 540.0)),
    };
    if let Some(ow) = w_override { w = ow; }
    if let Some(oh) = h_override { h = oh; }
    (w, h)
}

fn parse_explicit_dims(s: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = s.split(|c| c == 'x' || c == 'X').collect();
    if parts.len() != 2 { return None; }
    let w = parts[0].trim().parse::<f64>().ok()?;
    let h = parts[1].trim().parse::<f64>().ok()?;
    Some((w, h))
}

// ─── PDF Object & Content Stream ────────────────────────────────────

struct PdfObj {
    #[allow(dead_code)]
    id: usize,
    data: Vec<u8>,
}

struct ContentStream { ops: Vec<u8> }

impl ContentStream {
    fn new() -> Self { Self { ops: Vec::new() } }
    fn op(&mut self, s: &str) {
        self.ops.extend_from_slice(s.as_bytes());
        self.ops.push(b'\n');
    }
    fn set_font(&mut self, tag: &str, size: f64) {
        self.op(&format!("/{} {} Tf", tag, fmt_f64(size)));
    }
    fn set_color(&mut self, r: f64, g: f64, b: f64) {
        self.op(&format!("{} {} {} rg", fmt_f64(r), fmt_f64(g), fmt_f64(b)));
    }
    fn begin_text(&mut self) { self.op("BT"); }
    fn end_text(&mut self)   { self.op("ET"); }
    fn text_position(&mut self, x: f64, y: f64) {
        self.op(&format!("{} {} Td", fmt_f64(x), fmt_f64(y)));
    }
    fn show_text(&mut self, text: &str) {
        self.op(&format!("<{}> Tj", text_to_pdf_hex(text)));
    }
    fn text_at(&mut self, x: f64, y: f64, font_tag: &str, size: f64, text: &str) {
        self.begin_text();
        self.set_font(font_tag, size);
        self.text_position(x, y);
        self.show_text(text);
        self.end_text();
    }
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.op(&format!("{} {} {} {} re", fmt_f64(x), fmt_f64(y), fmt_f64(w), fmt_f64(h)));
    }
    fn fill(&mut self) { self.op("f"); }
    fn bytes(&self) -> &[u8] { &self.ops }
}

// ─── Codegen ────────────────────────────────────────────────────────

/// Slide deck PDF generator. Each `Slide` (or layout variant) is exactly one page;
/// content that exceeds the slide is clipped with a stderr warning.
pub struct SlidesCodegen {
    objects: Vec<PdfObj>,
    page_width: f64,
    page_height: f64,
    margin: f64,
    margin_left: f64,
    margin_right: f64,
    cursor_y: f64,
    current_stream: ContentStream,
    fonts: Vec<(String, String)>,
    default_font: String,
    default_font_size: f64,
    show_slide_numbers: bool,
    footer_text: Option<String>,
    current_slide: usize,
    total_slides: usize,
    slide_overflowed: bool,
    overflow_warnings: Vec<usize>,
}

impl SlidesCodegen {
    pub fn new(config: &SlidesConfig) -> Self {
        let (w, h) = slide_dimensions(&config.size, config.width, config.height);
        let mut cg = Self {
            objects: Vec::new(),
            page_width: w,
            page_height: h,
            margin: config.margin,
            margin_left: config.margin,
            margin_right: config.margin,
            cursor_y: h - config.margin,
            current_stream: ContentStream::new(),
            fonts: Vec::new(),
            default_font: config.default_font.clone(),
            default_font_size: config.default_font_size,
            show_slide_numbers: config.show_slide_numbers,
            footer_text: config.footer_text.clone(),
            current_slide: 0,
            total_slides: 0,
            slide_overflowed: false,
            overflow_warnings: Vec::new(),
        };
        cg.register_font(&config.default_font);
        cg.register_font("Helvetica");
        cg.register_font("Helvetica-Bold");
        cg.register_font("Times-Roman");
        cg.register_font("Times-Bold");
        cg.register_font("Courier");
        cg
    }

    pub fn slide_count(&self) -> usize { self.current_slide }

    fn register_font(&mut self, base_font: &str) -> String {
        for (tag, name) in &self.fonts {
            if name == base_font { return tag.clone(); }
        }
        let tag = format!("F{}", self.fonts.len() + 1);
        self.fonts.push((tag.clone(), base_font.to_string()));
        tag
    }

    fn font_tag(&self, base_font: &str) -> String {
        for (tag, name) in &self.fonts {
            if name == base_font { return tag.clone(); }
        }
        "F1".to_string()
    }

    fn content_width(&self) -> f64 {
        self.page_width - self.margin_left - self.margin_right
    }

    fn add_object(&mut self, data: &[u8]) -> usize {
        let id = self.objects.len() + 1;
        self.objects.push(PdfObj { id, data: data.to_vec() });
        id
    }

    fn add_stream_object(&mut self, stream_data: &[u8]) -> usize {
        let header = format!("<< /Length {} >>", stream_data.len());
        let mut data = Vec::new();
        data.extend_from_slice(header.as_bytes());
        data.extend_from_slice(b"\nstream\n");
        data.extend_from_slice(stream_data);
        data.extend_from_slice(b"\nendstream");
        self.add_object(&data)
    }

    // ─── Main Entry ──────────────────────────────────────────

    pub fn generate(&mut self, program: &Program) -> Vec<u8> {
        // Pass 1: count slides for "n / total" chrome.
        self.total_slides = self.count_slides(program);

        // Pass 2: emit pages.
        for decl in &program.declarations {
            if let Declaration::Page(page) = decl {
                for stmt in &page.body { self.emit_top_statement(stmt); }
            }
        }

        for idx in &self.overflow_warnings {
            eprintln!("warning[slides]: slide {} content overflows; truncated", idx);
        }

        self.serialize()
    }

    fn count_slides(&self, program: &Program) -> usize {
        let mut n = 0;
        for decl in &program.declarations {
            if let Declaration::Page(page) = decl {
                for stmt in &page.body { n += count_slides_in_stmt(stmt); }
            }
        }
        n
    }

    // ─── Slide dispatch ──────────────────────────────────────

    fn emit_top_statement(&mut self, stmt: &Statement) {
        if let Statement::UIElement(ui) = stmt {
            if let ComponentRef::BuiltIn(name) = &ui.component {
                if name == "Presentation" {
                    for child in &ui.children { self.emit_slide(child); }
                }
            }
        }
    }

    fn emit_slide(&mut self, stmt: &Statement) {
        let ui = match stmt {
            Statement::UIElement(u) => u,
            _ => return,
        };
        let name = match &ui.component {
            ComponentRef::BuiltIn(n) => n.as_str(),
            _ => return,
        };
        self.start_slide();
        match name {
            "Slide"        => self.emit_freeform_slide(ui),
            "TitleSlide"   => self.emit_title_slide(ui),
            "SectionSlide" => self.emit_section_slide(ui),
            "TwoColumn"    => self.emit_two_column(ui),
            "ImageSlide"   => self.emit_image_slide(ui),
            _ => {}
        }
        self.finalize_slide();
    }

    fn start_slide(&mut self) {
        self.current_slide += 1;
        self.cursor_y = self.page_height - self.margin;
        self.margin_left = self.margin;
        self.margin_right = self.margin;
        self.current_stream = ContentStream::new();
        self.slide_overflowed = false;
    }

    fn finalize_slide(&mut self) {
        self.emit_chrome();
        if self.slide_overflowed {
            self.overflow_warnings.push(self.current_slide);
        }
        let stream = std::mem::replace(&mut self.current_stream, ContentStream::new());
        let content_id = self.add_stream_object(stream.bytes());
        let page_data = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents {} 0 R >>",
            fmt_f64(self.page_width), fmt_f64(self.page_height), content_id
        );
        self.add_object(page_data.as_bytes());
    }

    fn emit_chrome(&mut self) {
        let chrome_size = 11.0;
        let baseline = self.margin * 0.4;
        let ft = self.font_tag("Helvetica");
        self.current_stream.set_color(0.55, 0.55, 0.55);
        if let Some(footer) = self.footer_text.clone() {
            self.current_stream.text_at(self.margin, baseline, &ft, chrome_size, &footer);
        }
        if self.show_slide_numbers {
            let label = format!("{} / {}", self.current_slide, self.total_slides);
            let tw = text_width(&label, "Helvetica", chrome_size);
            let x = self.page_width - self.margin - tw;
            self.current_stream.text_at(x, baseline, &ft, chrome_size, &label);
        }
    }

    // ─── Layout: freeform Slide ─────────────────────────────

    fn emit_freeform_slide(&mut self, ui: &UIElement) {
        for c in &ui.children { self.emit_statement(c); }
    }

    // ─── Layout: TitleSlide ─────────────────────────────────

    fn emit_title_slide(&mut self, ui: &UIElement) {
        let title = self.positional_string(ui, 0);
        let subtitle = self.positional_string(ui, 1);

        let title_size = 56.0;
        let sub_size = 28.0;
        let gap = 24.0;
        let block_h = if subtitle.is_empty() {
            title_size
        } else {
            title_size + gap + sub_size
        };
        let block_top = (self.page_height + block_h) / 2.0;

        // Title
        let title_baseline = block_top - title_size * 0.85;
        let tw = text_width(&title, "Helvetica-Bold", title_size);
        let tx = ((self.page_width - tw) / 2.0).max(0.0);
        let ft = self.font_tag("Helvetica-Bold");
        self.current_stream.set_color(0.05, 0.05, 0.10);
        self.current_stream.text_at(tx, title_baseline, &ft, title_size, &title);

        if !subtitle.is_empty() {
            let sub_baseline = title_baseline - title_size * 0.15 - gap - sub_size * 0.85;
            let sw = text_width(&subtitle, "Helvetica", sub_size);
            let sx = ((self.page_width - sw) / 2.0).max(0.0);
            let ft = self.font_tag("Helvetica");
            self.current_stream.set_color(0.40, 0.40, 0.45);
            self.current_stream.text_at(sx, sub_baseline, &ft, sub_size, &subtitle);
        }
    }

    // ─── Layout: SectionSlide ───────────────────────────────

    fn emit_section_slide(&mut self, ui: &UIElement) {
        let label = self.positional_string(ui, 0);
        let color = modifier_color(&ui.modifiers, (0.39, 0.39, 0.95));

        // Full-bleed background.
        self.current_stream.set_color(color.0, color.1, color.2);
        self.current_stream.rect(0.0, 0.0, self.page_width, self.page_height);
        self.current_stream.fill();

        // Centered white label.
        let size = 48.0;
        let lw = text_width(&label, "Helvetica-Bold", size);
        let x = ((self.page_width - lw) / 2.0).max(0.0);
        let y = self.page_height / 2.0 - size * 0.30;
        let ft = self.font_tag("Helvetica-Bold");
        self.current_stream.set_color(1.0, 1.0, 1.0);
        self.current_stream.text_at(x, y, &ft, size, &label);
    }

    // ─── Layout: TwoColumn ──────────────────────────────────

    fn emit_two_column(&mut self, ui: &UIElement) {
        let cols: Vec<&UIElement> = ui.children.iter().filter_map(|c| {
            if let Statement::UIElement(u) = c { Some(u) } else { None }
        }).take(2).collect();
        if cols.len() < 2 { return; }

        let gutter = 24.0;
        let half = (self.page_width - 2.0 * self.margin - gutter) / 2.0;
        let saved_left = self.margin_left;
        let saved_right = self.margin_right;
        let saved_y = self.cursor_y;

        // Left column: from margin to (margin + half).
        self.margin_left = self.margin;
        self.margin_right = self.page_width - (self.margin + half);
        for c in &cols[0].children { self.emit_statement(c); }
        let left_end_y = self.cursor_y;

        // Right column: from (margin + half + gutter) to (page_width - margin).
        self.cursor_y = saved_y;
        self.margin_left = self.margin + half + gutter;
        self.margin_right = self.margin;
        for c in &cols[1].children { self.emit_statement(c); }
        let right_end_y = self.cursor_y;

        self.margin_left = saved_left;
        self.margin_right = saved_right;
        self.cursor_y = left_end_y.min(right_end_y);
    }

    // ─── Layout: ImageSlide ─────────────────────────────────

    fn emit_image_slide(&mut self, ui: &UIElement) {
        let mut caption = String::new();
        for arg in &ui.args {
            if let Arg::Named(name, value) = arg {
                if name == "caption" {
                    caption = self.expr_to_string(value);
                }
            }
        }

        // Reserve space for caption (if any).
        let caption_size = 16.0;
        let caption_gap = 16.0;
        let caption_h = if caption.is_empty() { 0.0 } else { caption_size + caption_gap };

        // Image area: fill 80% of available content (height-wise), centered.
        let avail_w = self.page_width - 2.0 * self.margin;
        let avail_h = self.page_height - 2.0 * self.margin - caption_h;
        let img_w = avail_w * 0.85;
        let img_h = avail_h * 0.90;
        let img_x = (self.page_width - img_w) / 2.0;
        let img_y = self.margin + caption_h + (avail_h - img_h) / 2.0;

        // Placeholder rectangle (image embedding is a v2 feature, mirrors pdf.rs).
        self.current_stream.set_color(0.93, 0.93, 0.94);
        self.current_stream.rect(img_x, img_y, img_w, img_h);
        self.current_stream.fill();

        // "[Image]" label in the center of the placeholder.
        let lbl = "[Image]";
        let lbl_size = 14.0;
        let lw = text_width(lbl, "Helvetica", lbl_size);
        let ft = self.font_tag("Helvetica");
        self.current_stream.set_color(0.55, 0.55, 0.55);
        self.current_stream.text_at(
            img_x + (img_w - lw) / 2.0,
            img_y + img_h / 2.0 - lbl_size * 0.3,
            &ft, lbl_size, lbl,
        );

        if !caption.is_empty() {
            let cw = text_width(&caption, "Helvetica", caption_size);
            let cx = ((self.page_width - cw) / 2.0).max(0.0);
            let cy = self.margin + caption_size * 0.4;
            let ft = self.font_tag("Helvetica");
            self.current_stream.set_color(0.40, 0.40, 0.45);
            self.current_stream.text_at(cx, cy, &ft, caption_size, &caption);
        }
    }

    // ─── Statement dispatch (inside slide body) ─────────────

    fn emit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::UIElement(ui) => self.emit_ui_element(ui),
            Statement::If(if_stmt) => {
                for s in &if_stmt.then_body { self.emit_statement(s); }
            }
            Statement::For(for_stmt) => {
                if let Expr::ListLiteral(items) = &for_stmt.iterable {
                    for _ in items {
                        for s in &for_stmt.body { self.emit_statement(s); }
                    }
                }
            }
            _ => {}
        }
    }

    fn emit_ui_element(&mut self, ui: &UIElement) {
        let name = match &ui.component {
            ComponentRef::BuiltIn(n) => n.clone(),
            ComponentRef::SubComponent(p, s) => format!("{}.{}", p, s),
            ComponentRef::UserDefined(_) => {
                for c in &ui.children { self.emit_statement(c); }
                return;
            }
        };
        match name.as_str() {
            "Text"     => self.emit_text(ui),
            "Heading"  => self.emit_heading(ui),
            "Spacer"   => { self.cursor_y -= self.spacer_size(ui); }
            "Divider"  => self.emit_divider(),
            "List"     => self.emit_list(ui),
            "Container" | "Column" | "Stack" | "Grid" | "Section" => {
                for c in &ui.children { self.emit_statement(c); }
            }
            _ => {
                // Unknown component inside a slide — recurse into children for graceful degradation.
                for c in &ui.children { self.emit_statement(c); }
            }
        }
    }

    // ─── Body components (clip-on-overflow) ─────────────────

    fn emit_text(&mut self, ui: &UIElement) {
        let text = self.text_content(ui);
        if text.is_empty() { return; }
        let (font, size, color, align) = self.text_style(ui);
        self.render_wrapped_text(&text, &font, size, color, &align);
    }

    fn emit_heading(&mut self, ui: &UIElement) {
        let text = self.text_content(ui);
        if text.is_empty() { return; }
        let mut level = 2;
        let mut color = (0.05, 0.05, 0.10);
        for m in &ui.modifiers {
            match m.as_str() {
                "h1" => level = 1, "h2" => level = 2, "h3" => level = 3,
                "h4" => level = 4, "h5" => level = 5, "h6" => level = 6,
                "muted" => color = (0.4, 0.4, 0.4),
                _ => {}
            }
        }
        // Slide-scaled heading sizes (~2x PDF sizes).
        let size = match level { 1=>48.0, 2=>36.0, 3=>28.0, 4=>22.0, 5=>18.0, _=>16.0 };
        let (_, _, sc, align) = self.text_style(ui);
        if sc != (0.0, 0.0, 0.0) { color = sc; }
        self.cursor_y -= size * 0.3;
        self.render_wrapped_text(&text, "Helvetica-Bold", size, color, &align);
        self.cursor_y -= size * 0.2;
    }

    fn emit_divider(&mut self) {
        let gap = 16.0;
        if self.cursor_y - gap < self.margin { self.slide_overflowed = true; return; }
        let y = self.cursor_y - gap / 2.0;
        let stroke_w = 1.0;
        // Use a thin filled rect as a divider (no stroke ops in our minimal stream).
        self.current_stream.set_color(0.80, 0.80, 0.80);
        self.current_stream.rect(self.margin_left, y - stroke_w / 2.0,
                                  self.content_width(), stroke_w);
        self.current_stream.fill();
        self.cursor_y -= gap;
    }

    fn emit_list(&mut self, ui: &UIElement) {
        let ordered = ui.modifiers.iter().any(|m| m == "ordered");
        let fs = self.default_font_size;
        let lh = fs * 1.5;
        let indent = 22.0;
        for (idx, child) in ui.children.iter().enumerate() {
            let item = match child {
                Statement::UIElement(u) => u,
                _ => continue,
            };
            let text = self.text_content(item);
            if text.is_empty() { continue; }
            if self.cursor_y - lh < self.margin { self.slide_overflowed = true; return; }
            let marker = if ordered { format!("{}.", idx + 1) } else { "•".to_string() };
            let ft = self.font_tag(&self.default_font.clone());
            self.current_stream.set_color(0.35, 0.35, 0.35);
            self.current_stream.text_at(self.margin_left, self.cursor_y, &ft, fs, &marker);
            let saved_l = self.margin_left;
            self.margin_left += indent;
            self.render_wrapped_text(&text, &self.default_font.clone(), fs, (0.10, 0.10, 0.10), "");
            self.margin_left = saved_l;
        }
        self.cursor_y -= 6.0;
    }

    // ─── Word-wrapped text, clip on overflow ────────────────

    fn render_wrapped_text(&mut self, text: &str, font: &str, size: f64, color: (f64, f64, f64), align: &str) {
        let ft = self.font_tag(font);
        let lh = size * 1.4;
        let aw = self.content_width();
        let sw = text_width(" ", font, size);

        for line in text.split('\n') {
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.is_empty() {
                if self.cursor_y - lh * 0.5 < self.margin { self.slide_overflowed = true; return; }
                self.cursor_y -= lh * 0.5;
                continue;
            }
            let mut cl = String::new();
            let mut cw = 0.0;
            for w in &words {
                let ww = text_width(w, font, size);
                if !cl.is_empty() && cw + sw + ww > aw {
                    if self.cursor_y - lh < self.margin { self.slide_overflowed = true; return; }
                    let x = self.text_x_for_align(&cl, font, size, align);
                    self.current_stream.set_color(color.0, color.1, color.2);
                    self.current_stream.text_at(x, self.cursor_y, &ft, size, &cl);
                    self.cursor_y -= lh;
                    cl = w.to_string(); cw = ww;
                } else {
                    if !cl.is_empty() { cl.push(' '); cw += sw; }
                    cl.push_str(w); cw += ww;
                }
            }
            if !cl.is_empty() {
                if self.cursor_y - lh < self.margin { self.slide_overflowed = true; return; }
                let x = self.text_x_for_align(&cl, font, size, align);
                self.current_stream.set_color(color.0, color.1, color.2);
                self.current_stream.text_at(x, self.cursor_y, &ft, size, &cl);
                self.cursor_y -= lh;
            }
        }
    }

    fn text_x_for_align(&self, text: &str, font: &str, size: f64, align: &str) -> f64 {
        match align {
            "center" => self.margin_left + (self.content_width() - text_width(text, font, size)) / 2.0,
            "right"  => self.page_width - self.margin_right - text_width(text, font, size),
            _        => self.margin_left,
        }
    }

    // ─── Helpers ────────────────────────────────────────────

    fn text_content(&self, ui: &UIElement) -> String {
        for arg in &ui.args {
            if let Arg::Positional(expr) = arg {
                return self.expr_to_string(expr);
            }
        }
        String::new()
    }

    fn positional_string(&self, ui: &UIElement, index: usize) -> String {
        let mut i = 0;
        for arg in &ui.args {
            if let Arg::Positional(expr) = arg {
                if i == index { return self.expr_to_string(expr); }
                i += 1;
            }
        }
        String::new()
    }

    fn expr_to_string(&self, expr: &Expr) -> String {
        match expr {
            Expr::StringLiteral(s) => s.clone(),
            Expr::InterpolatedString(parts) => {
                let mut out = String::new();
                for p in parts {
                    match p {
                        StringPart::Literal(s) => out.push_str(s),
                        StringPart::Expression(e) => out.push_str(&self.expr_to_string(e)),
                    }
                }
                out
            }
            Expr::NumberLiteral(n) => format!("{}", n),
            Expr::BoolLiteral(b) => format!("{}", b),
            Expr::Identifier(name) => format!("{{{}}}", name),
            _ => String::new(),
        }
    }

    fn text_style(&self, ui: &UIElement) -> (String, f64, (f64, f64, f64), String) {
        let mut font = self.default_font.clone();
        let mut size = self.default_font_size;
        let mut color = (0.0, 0.0, 0.0);
        let mut align = String::new();
        for m in &ui.modifiers {
            match m.as_str() {
                "bold"    => font = format!("{}-Bold", font.split('-').next().unwrap_or("Helvetica")),
                "muted"   => color = (0.4, 0.4, 0.4),
                "primary" => color = (0.098, 0.098, 0.647),
                "danger"  => color = (0.86, 0.21, 0.27),
                "success" => color = (0.16, 0.65, 0.27),
                "warning" => color = (0.90, 0.56, 0.0),
                "info"    => color = (0.0, 0.47, 0.84),
                "small"   => size = self.default_font_size * 0.85,
                "large"   => size = self.default_font_size * 1.25,
                "center"  => align = "center".to_string(),
                "right"   => align = "right".to_string(),
                _ => {}
            }
        }
        if let Some(style) = &ui.style_block {
            for sp in &style.properties {
                match sp.name.as_str() {
                    "font-size" => { if let Expr::NumberLiteral(n) = sp.value { size = n; } }
                    "font-family" | "font" => { if let Expr::StringLiteral(s) = &sp.value { font = s.clone(); } }
                    "color" => { if let Expr::StringLiteral(s) = &sp.value { color = parse_color(s); } }
                    "text-align" => { if let Expr::StringLiteral(s) = &sp.value { align = s.clone(); } }
                    _ => {}
                }
            }
        }
        (font, size, color, align)
    }

    fn spacer_size(&self, ui: &UIElement) -> f64 {
        for m in &ui.modifiers {
            match m.as_str() {
                "small" | "sm" => return 12.0,
                "medium" | "md" => return 24.0,
                "large" | "lg" => return 40.0,
                "xl" => return 56.0,
                _ => {}
            }
        }
        24.0
    }

    // ─── Serialization ──────────────────────────────────────

    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut offsets: Vec<usize> = Vec::new();

        out.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

        let font_start = 3;
        let num_fonts = self.fonts.len();
        let resources_id = font_start + num_fonts;

        let mut final_objects: Vec<(usize, Vec<u8>)> = Vec::new();

        // 1: Catalog
        final_objects.push((1, b"<< /Type /Catalog /Pages 2 0 R >>".to_vec()));

        // Fonts
        for (i, (_, bf)) in self.fonts.iter().enumerate() {
            final_objects.push((font_start + i,
                format!("<< /Type /Font /Subtype /Type1 /BaseFont /{} /Encoding /WinAnsiEncoding >>", bf).into_bytes()));
        }

        // Resources
        let mut fe = String::new();
        for (i, (tag, _)) in self.fonts.iter().enumerate() {
            fe.push_str(&format!("/{} {} 0 R ", tag, font_start + i));
        }
        final_objects.push((resources_id, format!("<< /Font << {} >> >>", fe).into_bytes()));

        // Content streams + Pages (objects come in pairs: stream, then page).
        let mut new_page_ids: Vec<usize> = Vec::new();
        let mut next_id = resources_id + 1;
        let mut i = 0;
        while i + 1 < self.objects.len() {
            let cid = next_id;
            final_objects.push((cid, self.objects[i].data.clone()));
            next_id += 1;
            let pid = next_id;
            final_objects.push((pid, format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents {} 0 R /Resources {} 0 R >>",
                fmt_f64(self.page_width), fmt_f64(self.page_height), cid, resources_id
            ).into_bytes()));
            new_page_ids.push(pid);
            next_id += 1;
            i += 2;
        }

        // 2: Pages
        let kids: Vec<String> = new_page_ids.iter().map(|id| format!("{} 0 R", id)).collect();
        final_objects.push((2, format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), new_page_ids.len()).into_bytes()));

        final_objects.sort_by_key(|(id, _)| *id);

        let total_objects = final_objects.last().map(|(id, _)| *id).unwrap_or(0);
        offsets.resize(total_objects + 1, 0);

        for (id, data) in &final_objects {
            offsets[*id] = out.len();
            out.extend_from_slice(format!("{} 0 obj\n", id).as_bytes());
            out.extend_from_slice(data);
            out.extend_from_slice(b"\nendobj\n\n");
        }

        let xref_offset = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n", total_objects + 1).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for id in 1..=total_objects {
            out.extend_from_slice(format!("{:010} 00000 n \n", offsets.get(id).copied().unwrap_or(0)).as_bytes());
        }
        out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", total_objects + 1, xref_offset).as_bytes());
        out
    }
}

// ─── Free functions ─────────────────────────────────────────────────

fn count_slides_in_stmt(stmt: &Statement) -> usize {
    if let Statement::UIElement(ui) = stmt {
        if let ComponentRef::BuiltIn(name) = &ui.component {
            if name == "Presentation" {
                let mut n = 0;
                for child in &ui.children {
                    if let Statement::UIElement(c) = child {
                        if let ComponentRef::BuiltIn(cn) = &c.component {
                            if matches!(cn.as_str(),
                                "Slide" | "TitleSlide" | "SectionSlide" | "TwoColumn" | "ImageSlide"
                            ) { n += 1; }
                        }
                    }
                }
                return n;
            }
        }
    }
    0
}

fn modifier_color(mods: &[String], default: (f64, f64, f64)) -> (f64, f64, f64) {
    for m in mods {
        match m.as_str() {
            "primary" => return (0.39, 0.39, 0.95),
            "success" => return (0.16, 0.65, 0.27),
            "danger"  => return (0.86, 0.21, 0.27),
            "warning" => return (0.90, 0.56, 0.0),
            "info"    => return (0.0, 0.47, 0.84),
            _ => {}
        }
    }
    default
}

fn fmt_f64(v: f64) -> String {
    if v == v.floor() { format!("{:.0}", v) } else { format!("{:.2}", v) }
}

fn char_to_winansi(ch: char) -> u8 {
    match ch {
        '\u{2014}' => 0x97, '\u{2013}' => 0x96,
        '\u{2018}' => 0x91, '\u{2019}' => 0x92,
        '\u{201C}' => 0x93, '\u{201D}' => 0x94,
        '\u{2022}' => 0x95, '\u{2026}' => 0x85,
        '\u{2122}' => 0x99, '\u{00A9}' => 0xA9,
        '\u{00AE}' => 0xAE, '\u{00B0}' => 0xB0,
        '\u{20AC}' => 0x80,
        c if (c as u32) < 256 => c as u8,
        _ => b'?',
    }
}

fn text_to_pdf_hex(text: &str) -> String {
    text.chars().map(|ch| format!("{:02X}", char_to_winansi(ch))).collect()
}

fn parse_color(s: &str) -> (f64, f64, f64) {
    let s = s.trim_start_matches('#');
    if s.len() >= 6 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0) as f64 / 255.0;
        (r, g, b)
    } else {
        (0.0, 0.0, 0.0)
    }
}
