use std::collections::HashSet;
use crate::config::project::PdfConfig;
use crate::parser::{Program, Declaration, Statement, UIElement, ComponentRef, Expr, StringPart};
use crate::codegen::style::{
    Background, Color as StyleColor, LinearGradient, StyleProps,
    gradient_endpoints,
};

// ─── Base14 Font Metrics (Helvetica) ────────────────────────────────
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
    if code < 32 || code > 126 {
        return 500;
    }
    let idx = (code - 32) as usize;
    if font.contains("Courier") {
        COURIER_WIDTH
    } else if font.contains("Times") {
        TIMES_WIDTHS[idx]
    } else if font.contains("Bold") {
        HELVETICA_BOLD_WIDTHS[idx]
    } else {
        HELVETICA_WIDTHS[idx]
    }
}

fn text_width(text: &str, font: &str, font_size: f64) -> f64 {
    let units: u32 = text.chars().map(|c| char_width(c, font) as u32).sum();
    units as f64 * font_size / 1000.0
}

fn truncate_text(text: &str, font: &str, font_size: f64, max_width: f64) -> String {
    let full_w = text_width(text, font, font_size);
    if full_w <= max_width {
        return text.to_string();
    }
    let ellipsis_w = text_width("...", font, font_size);
    let target = max_width - ellipsis_w;
    if target <= 0.0 {
        return "...".to_string();
    }
    let mut w = 0.0;
    let mut end = 0;
    for (i, ch) in text.char_indices() {
        let cw = char_width(ch, font) as f64 * font_size / 1000.0;
        if w + cw > target { break; }
        w += cw;
        end = i + ch.len_utf8();
    }
    format!("{}...", &text[..end])
}

fn page_dimensions(size: &str) -> (f64, f64) {
    match size.to_uppercase().as_str() {
        "A4" => (595.28, 841.89),
        "A3" => (841.89, 1190.55),
        "A5" => (419.53, 595.28),
        "LETTER" => (612.0, 792.0),
        "LEGAL" => (612.0, 1008.0),
        _ => (595.28, 841.89),
    }
}

// ─── PDF Object ─────────────────────────────────────────────────────

struct PdfObj {
    id: usize,
    data: Vec<u8>,
}

// ─── Content Stream Builder ─────────────────────────────────────────

struct ContentStream {
    ops: Vec<u8>,
}

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
    fn set_stroke_color(&mut self, r: f64, g: f64, b: f64) {
        self.op(&format!("{} {} {} RG", fmt_f64(r), fmt_f64(g), fmt_f64(b)));
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
    fn fill(&mut self)   { self.op("f"); }
    fn stroke(&mut self) { self.op("S"); }
    fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.op(&format!("{} {} m {} {} l S", fmt_f64(x1), fmt_f64(y1), fmt_f64(x2), fmt_f64(y2)));
    }
    fn set_line_width(&mut self, w: f64) {
        self.op(&format!("{} w", fmt_f64(w)));
    }
    /// Rounded rect path only (no fill/stroke). Caller must invoke `fill()` or `stroke()`.
    fn rounded_rect_path(&mut self, x: f64, y: f64, w: f64, h: f64, r: f64) {
        let r = r.min(w / 2.0).min(h / 2.0);
        if r < 0.5 { self.rect(x, y, w, h); return; }
        let k = 0.5523;
        let kr = k * r;
        self.op(&format!("{} {} m", fmt_f64(x + r), fmt_f64(y)));
        self.op(&format!("{} {} l", fmt_f64(x + w - r), fmt_f64(y)));
        self.op(&format!("{} {} {} {} {} {} c", fmt_f64(x+w-r+kr), fmt_f64(y), fmt_f64(x+w), fmt_f64(y+r-kr), fmt_f64(x+w), fmt_f64(y+r)));
        self.op(&format!("{} {} l", fmt_f64(x + w), fmt_f64(y + h - r)));
        self.op(&format!("{} {} {} {} {} {} c", fmt_f64(x+w), fmt_f64(y+h-r+kr), fmt_f64(x+w-r+kr), fmt_f64(y+h), fmt_f64(x+w-r), fmt_f64(y+h)));
        self.op(&format!("{} {} l", fmt_f64(x + r), fmt_f64(y + h)));
        self.op(&format!("{} {} {} {} {} {} c", fmt_f64(x+r-kr), fmt_f64(y+h), fmt_f64(x), fmt_f64(y+h-r+kr), fmt_f64(x), fmt_f64(y+h-r)));
        self.op(&format!("{} {} l", fmt_f64(x), fmt_f64(y + r)));
        self.op(&format!("{} {} {} {} {} {} c", fmt_f64(x), fmt_f64(y+r-kr), fmt_f64(x+r-kr), fmt_f64(y), fmt_f64(x+r), fmt_f64(y)));
        self.op("h");
    }
    fn rounded_rect(&mut self, x: f64, y: f64, w: f64, h: f64, r: f64) {
        let r = r.min(w / 2.0).min(h / 2.0);
        if r < 0.5 { self.rect(x, y, w, h); return; }
        let k = 0.5523;
        let kr = k * r;
        self.op(&format!("{} {} m", fmt_f64(x + r), fmt_f64(y)));
        self.op(&format!("{} {} l", fmt_f64(x + w - r), fmt_f64(y)));
        self.op(&format!("{} {} {} {} {} {} c", fmt_f64(x+w-r+kr), fmt_f64(y), fmt_f64(x+w), fmt_f64(y+r-kr), fmt_f64(x+w), fmt_f64(y+r)));
        self.op(&format!("{} {} l", fmt_f64(x + w), fmt_f64(y + h - r)));
        self.op(&format!("{} {} {} {} {} {} c", fmt_f64(x+w), fmt_f64(y+h-r+kr), fmt_f64(x+w-r+kr), fmt_f64(y+h), fmt_f64(x+w-r), fmt_f64(y+h)));
        self.op(&format!("{} {} l", fmt_f64(x + r), fmt_f64(y + h)));
        self.op(&format!("{} {} {} {} {} {} c", fmt_f64(x+r-kr), fmt_f64(y+h), fmt_f64(x), fmt_f64(y+h-r+kr), fmt_f64(x), fmt_f64(y+h-r)));
        self.op(&format!("{} {} l", fmt_f64(x), fmt_f64(y + r)));
        self.op(&format!("{} {} {} {} {} {} c", fmt_f64(x), fmt_f64(y+r-kr), fmt_f64(x+r-kr), fmt_f64(y), fmt_f64(x+r), fmt_f64(y)));
        self.op("h");
    }
    fn bytes(&self) -> &[u8] { &self.ops }
}

// ─── PDF Writer ─────────────────────────────────────────────────────
//
// COORDINATE CONTRACT:
//   cursor_y = the Y coordinate where the next text BASELINE will be drawn.
//   After drawing a line of text at cursor_y, decrement cursor_y by line_height.
//   Shapes (rects, lines) position themselves relative to cursor_y.
//   Y=0 is page bottom. Y increases upward.

struct GradientInstance {
    tag: String,
    start: StyleColor,
    end: StyleColor,
    x0: f64, y0: f64, x1: f64, y1: f64,
}

/// PDF code generator — renders the AST to a PDF document using Base14 fonts.
///
/// Supports text, headings, tables, lists, code blocks, cards, badges, alerts,
/// progress bars, images, and page headers/footers with `{page}`/`{pages}` variables.
pub struct PdfCodegen {
    objects: Vec<PdfObj>,
    pages: Vec<usize>,
    page_width: f64,
    page_height: f64,
    margin_top: f64,
    margin_bottom: f64,
    margin_left: f64,
    margin_right: f64,
    cursor_y: f64,
    current_stream: ContentStream,
    fonts: Vec<(String, String)>,
    current_font: String,
    current_font_name: String,
    current_font_size: f64,
    current_color: (f64, f64, f64),
    default_font: String,
    default_font_size: f64,
    header_stmts: Vec<Statement>,
    footer_stmts: Vec<Statement>,
    image_objects: Vec<(usize, f64, f64)>,
    in_header_footer: bool,
    page_has_content: bool,
    current_page_number: usize,
    shadings: Vec<GradientInstance>,
    warned_styles: HashSet<String>,
}

impl PdfCodegen {
    pub fn new(config: &PdfConfig) -> Self {
        let (w, h) = page_dimensions(&config.page_size);
        let mut cg = Self {
            objects: Vec::new(),
            pages: Vec::new(),
            page_width: w,
            page_height: h,
            margin_top: config.margins.top,
            margin_bottom: config.margins.bottom,
            margin_left: config.margins.left,
            margin_right: config.margins.right,
            cursor_y: h - config.margins.top,
            current_stream: ContentStream::new(),
            fonts: Vec::new(),
            current_font: "F1".to_string(),
            current_font_name: config.default_font.clone(),
            current_font_size: config.default_font_size,
            current_color: (0.0, 0.0, 0.0),
            default_font: config.default_font.clone(),
            default_font_size: config.default_font_size,
            header_stmts: Vec::new(),
            footer_stmts: Vec::new(),
            image_objects: Vec::new(),
            in_header_footer: false,
            page_has_content: false,
            current_page_number: 0,
            shadings: Vec::new(),
            warned_styles: HashSet::new(),
        };
        cg.register_font(&config.default_font);
        cg.register_font("Helvetica-Bold");
        cg.register_font("Courier");
        cg.register_font("Courier-Bold");
        cg.register_font("Times-Roman");
        cg.register_font("Times-Bold");
        cg
    }

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

    fn available_height(&self) -> f64 {
        self.cursor_y - self.margin_bottom
    }

    fn check_page_break(&mut self, needed: f64) {
        if !self.in_header_footer && self.cursor_y - needed < self.margin_bottom {
            self.finalize_page();
            self.start_new_page();
        }
    }

    fn start_new_page(&mut self) {
        self.current_page_number += 1;
        self.cursor_y = self.page_height - self.margin_top;
        self.current_stream = ContentStream::new();
        self.page_has_content = false;

        // Emit header in top margin area
        if !self.header_stmts.is_empty() && !self.in_header_footer {
            self.in_header_footer = true;
            let saved = self.cursor_y;
            // Header baseline: inside top margin, 30pt from page top
            self.cursor_y = self.page_height - 30.0;
            let stmts = self.header_stmts.clone();
            for s in &stmts { self.emit_statement(s); }
            self.cursor_y = saved;
            self.in_header_footer = false;
        }
    }

    fn finalize_page(&mut self) {
        // Emit footer in bottom margin area
        if !self.footer_stmts.is_empty() && !self.in_header_footer {
            self.in_header_footer = true;
            let saved = self.cursor_y;
            // Footer needs space. We give it the full bottom margin area.
            // Start at margin_bottom - 10 (just above the content area bottom edge)
            // and work downward. For a Divider + Row, that's ~40pt.
            // So we start the footer cursor at margin_bottom - 6, which is ~66pt from page bottom.
            self.cursor_y = self.margin_bottom - 6.0;
            let stmts = self.footer_stmts.clone();
            for s in &stmts { self.emit_statement(s); }
            self.cursor_y = saved;
            self.in_header_footer = false;
        }

        let stream_data = std::mem::replace(&mut self.current_stream, ContentStream::new());
        if stream_data.bytes().is_empty() && !self.page_has_content {
            return;
        }
        let content_id = self.add_stream_object(stream_data.bytes());
        let page_data = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents {} 0 R >>",
            fmt_f64(self.page_width), fmt_f64(self.page_height), content_id
        );
        let page_obj_id = self.add_object(page_data.as_bytes());
        self.pages.push(page_obj_id);
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

    fn mark_content(&mut self) { self.page_has_content = true; }

    fn substitute_page_vars(&self, text: &str) -> String {
        if !self.in_header_footer { return text.to_string(); }
        text.replace("{page}", &self.current_page_number.to_string())
            .replace("{pages}", "###")
    }

    fn fixup_total_pages(&mut self) {
        let total = self.current_page_number;
        let total_str = format!("{:<3}", total);
        let placeholder_hex = "232323";
        let replacement_hex: String = total_str.chars()
            .map(|ch| format!("{:02X}", ch as u8)).collect();
        for obj in &mut self.objects {
            // Only process stream objects (they contain hex text)
            let data_str = String::from_utf8_lossy(&obj.data).to_string();
            if data_str.contains(placeholder_hex) {
                obj.data = data_str.replace(placeholder_hex, &replacement_hex).into_bytes();
            }
        }
    }

    pub fn page_count(&self) -> usize { self.pages.len() }

    // ─── Main Entry ──────────────────────────────────────────

    pub fn generate(&mut self, program: &Program) -> Vec<u8> {
        self.start_new_page();
        for decl in &program.declarations {
            if let Declaration::Page(page) = decl {
                for stmt in &page.body { self.emit_statement(stmt); }
            }
        }
        self.finalize_page();
        self.fixup_total_pages();
        self.serialize()
    }

    // ─── Statement Dispatch ─────────────────────────────────

    fn emit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::UIElement(ui) => self.emit_ui_element(ui),
            Statement::If(if_stmt) => {
                for s in &if_stmt.then_body { self.emit_statement(s); }
            }
            Statement::For(for_stmt) => {
                if let Expr::ListLiteral(items) = &for_stmt.iterable {
                    for _item in items {
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
            "Document"  => self.emit_document(ui),
            "Section"   => self.emit_section(ui),
            "Paragraph" => self.emit_paragraph(ui),
            "Header"    => { self.header_stmts = ui.children.clone(); }
            "Footer"    => { self.footer_stmts = ui.children.clone(); }
            "PageBreak" => { self.finalize_page(); self.start_new_page(); }
            "Text"      => self.emit_text(ui),
            "Heading"   => self.emit_heading(ui),
            "Table"     => self.emit_table(ui),
            "Thead" | "Tbody" => { for c in &ui.children { self.emit_statement(c); } }
            "Trow"      => {}
            "Code"      => self.emit_code(ui),
            "Blockquote"=> self.emit_blockquote(ui),
            "Image"     => self.emit_image(ui),
            "Divider"   => self.emit_divider(),
            "Row"       => self.emit_row(ui),
            "Container" | "Column" | "Grid" | "Stack" => {
                self.emit_container(ui, &name);
            }
            "Spacer"    => { self.cursor_y -= self.get_spacer_size(ui); }
            "List" | "TypeList" => self.emit_list(ui),
            "Badge" | "Tag" => self.emit_badge(ui),
            "Alert"     => self.emit_alert(ui),
            "Progress"  => self.emit_progress(ui),
            "Card"      => self.emit_card(ui),
            "Card.Header" | "Card.Body" | "Card.Footer" => {
                for c in &ui.children { self.emit_statement(c); }
            }
            "Avatar" | "Icon" => {}
            _ => { for c in &ui.children { self.emit_statement(c); } }
        }
    }

    // ─── Row (justify:between) ──────────────────────────────

    fn emit_row(&mut self, ui: &UIElement) {
        let mut justify_between = false;
        for arg in &ui.args {
            if let crate::parser::Arg::Named(name, value) = arg {
                if name == "justify" {
                    if let Expr::Identifier(v) = value {
                        if v == "between" { justify_between = true; }
                    }
                }
            }
        }

        if justify_between && ui.children.len() >= 2 {
            let mut items: Vec<(String, String, f64, (f64, f64, f64))> = Vec::new();
            for c in &ui.children {
                if let Statement::UIElement(cu) = c {
                    let t = self.extract_text_content(cu);
                    let (f, s, col, _) = self.extract_text_style(cu);
                    items.push((t, f, s, col));
                }
            }
            if items.len() >= 2 {
                let lh = items.iter().map(|(_, _, s, _)| *s * 1.5).fold(0.0f64, f64::max);
                self.check_page_break(lh);

                // Left item
                let (ref t, ref f, sz, col) = items[0];
                if !t.is_empty() {
                    let ft = self.font_tag(f);
                    self.current_stream.set_color(col.0, col.1, col.2);
                    self.current_stream.text_at(self.margin_left, self.cursor_y, &ft, sz, t);
                }
                // Right item
                let last = items.len() - 1;
                let (ref t, ref f, sz, col) = items[last];
                if !t.is_empty() {
                    let ft = self.font_tag(f);
                    let tw = text_width(t, f, sz);
                    let x = self.page_width - self.margin_right - tw;
                    self.current_stream.set_color(col.0, col.1, col.2);
                    self.current_stream.text_at(x, self.cursor_y, &ft, sz, t);
                }
                self.cursor_y -= lh;
                self.mark_content();
                return;
            }
        }
        for c in &ui.children { self.emit_statement(c); }
    }

    // ─── Document / Section ─────────────────────────────────

    fn emit_document(&mut self, ui: &UIElement) {
        for arg in &ui.args {
            if let crate::parser::Arg::Named(name, value) = arg {
                if name == "page_size" || name == "pageSize" {
                    if let Expr::StringLiteral(s) = value {
                        let (w, h) = page_dimensions(s);
                        self.page_width = w;
                        self.page_height = h;
                        self.cursor_y = h - self.margin_top;
                    }
                }
            }
        }
        for c in &ui.children { self.emit_statement(c); }
    }

    fn emit_section(&mut self, ui: &UIElement) {
        self.cursor_y -= 8.0;
        for c in &ui.children { self.emit_statement(c); }
        self.cursor_y -= 4.0;
    }

    fn emit_paragraph(&mut self, ui: &UIElement) {
        let (font, size, color, align) = self.extract_text_style(ui);
        for c in &ui.children {
            if let Statement::UIElement(tu) = c {
                let text = self.extract_text_content(tu);
                if !text.is_empty() {
                    let (cf, cs, cc, ca) = self.extract_text_style(tu);
                    let f = if cf != self.default_font { &cf } else { &font };
                    let s = if (cs - self.default_font_size).abs() > 0.1 { cs } else { size };
                    let c = if cc != (0.0, 0.0, 0.0) { cc } else { color };
                    let a = if !ca.is_empty() { &ca } else { &align };
                    self.render_wrapped_text(&text, f, s, c, a);
                }
            }
        }
        self.cursor_y -= self.current_font_size * 0.5;
    }

    // ─── Text ───────────────────────────────────────────────

    fn emit_text(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() { return; }
        let (font, size, color, align) = self.extract_text_style(ui);
        self.render_wrapped_text(&text, &font, size, color, &align);
    }

    // ─── Heading ────────────────────────────────────────────

    fn emit_heading(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() { return; }

        let mut level = 2;
        let mut color = (0.0, 0.0, 0.0);
        for m in &ui.modifiers {
            match m.as_str() {
                "h1" => level = 1, "h2" => level = 2, "h3" => level = 3,
                "h4" => level = 4, "h5" => level = 5, "h6" => level = 6,
                "primary" => color = (0.098, 0.098, 0.647),
                "muted" => color = (0.4, 0.4, 0.4),
                _ => {}
            }
        }
        let size = match level { 1=>28.0, 2=>22.0, 3=>18.0, 4=>16.0, 5=>14.0, _=>12.0 };
        let font = "Helvetica-Bold".to_string();
        let (_, _, sc, align) = self.extract_text_style(ui);
        if sc != (0.0, 0.0, 0.0) { color = sc; }

        self.cursor_y -= size * 0.4;
        self.check_page_break(size * 1.5);
        self.render_wrapped_text(&text, &font, size, color, &align);
        self.cursor_y -= size * 0.3;
    }

    // ─── Divider ────────────────────────────────────────────
    //
    // Contract: consume space, draw line in the middle of the gap.
    // cursor_y before: points to where next text baseline would go.
    // We move cursor_y down by total_gap. The line is drawn at
    // old_cursor_y - half_gap (the vertical midpoint of the gap).

    fn emit_divider(&mut self) {
        let total_gap = 20.0;
        let half = total_gap / 2.0;

        self.check_page_break(total_gap);

        // Line at the midpoint: cursor_y - half
        let line_y = self.cursor_y - half;
        self.current_stream.set_stroke_color(0.80, 0.80, 0.80);
        self.current_stream.set_line_width(0.5);
        self.current_stream.line(self.margin_left, line_y, self.page_width - self.margin_right, line_y);

        self.cursor_y -= total_gap;
        self.mark_content();
    }

    // ─── Table ──────────────────────────────────────────────

    fn emit_table(&mut self, ui: &UIElement) {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut is_header: Vec<bool> = Vec::new();

        for child in &ui.children {
            if let Statement::UIElement(section) = child {
                let sn = match &section.component {
                    ComponentRef::BuiltIn(n) => n.clone(), _ => String::new(),
                };
                let hdr = sn == "Thead";
                for rs in &section.children {
                    if let Statement::UIElement(row) = rs {
                        let cells: Vec<String> = row.children.iter().filter_map(|c| {
                            if let Statement::UIElement(cell) = c { Some(self.extract_text_content(cell)) } else { None }
                        }).collect();
                        if !cells.is_empty() { rows.push(cells); is_header.push(hdr); }
                    }
                }
            }
        }
        if rows.is_empty() { return; }

        let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(1);
        let cw = self.content_width();
        let col_w = cw / num_cols as f64;
        let cell_pad = 4.0;
        let cell_tw = col_w - cell_pad * 2.0;
        let fs = self.current_font_size;
        let lh = fs * 1.5;

        let mut row_heights: Vec<f64> = Vec::new();
        for row in &rows {
            let mut mx = 1usize;
            for ct in row {
                let l = self.count_wrapped_lines(ct, &self.default_font.clone(), fs, cell_tw);
                if l > mx { mx = l; }
            }
            row_heights.push(((mx as f64 * lh) + cell_pad * 2.0).max(fs * 2.0));
        }

        let init_h: f64 = row_heights.iter().take(2).sum();
        self.check_page_break(init_h);
        let table_x = self.margin_left;
        self.mark_content();
        self.current_stream.set_line_width(0.5);

        for (ri, row) in rows.iter().enumerate() {
            let rh = row_heights[ri];
            if !self.in_header_footer && self.cursor_y - rh < self.margin_bottom {
                self.finalize_page();
                self.start_new_page();
            }
            let top = self.cursor_y;
            let bot = top - rh;

            if is_header[ri] {
                self.current_stream.set_color(0.93, 0.93, 0.93);
                self.current_stream.rect(table_x, bot, cw, rh);
                self.current_stream.fill();
            }
            self.current_stream.set_stroke_color(0.75, 0.75, 0.75);
            self.current_stream.rect(table_x, bot, cw, rh);
            self.current_stream.stroke();

            let fn_ = if is_header[ri] { "Helvetica-Bold".to_string() } else { self.default_font.clone() };
            let ft = self.font_tag(&fn_);
            for (ci, ct) in row.iter().enumerate() {
                let cx = table_x + ci as f64 * col_w;
                if ci > 0 {
                    self.current_stream.set_stroke_color(0.75, 0.75, 0.75);
                    self.current_stream.line(cx, top, cx, bot);
                }
                self.current_stream.set_color(0.0, 0.0, 0.0);
                self.render_cell_text(ct, &fn_, &ft, fs, cx + cell_pad, top - cell_pad - fs, cell_tw, lh);
            }
            self.cursor_y = bot;
        }
        self.cursor_y -= 8.0;
    }

    fn count_wrapped_lines(&self, text: &str, font: &str, size: f64, max_w: f64) -> usize {
        if max_w <= 0.0 || text.is_empty() { return 1; }
        let sw = text_width(" ", font, size);
        let mut count = 0usize;
        for line in text.split('\n') {
            count += 1;
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.is_empty() { continue; }
            let mut cw = 0.0;
            for (i, w) in words.iter().enumerate() {
                let ww = text_width(w, font, size);
                let need = if i > 0 { sw + ww } else { ww };
                if cw > 0.0 && cw + need > max_w { count += 1; cw = ww; }
                else { cw += need; }
            }
        }
        count.max(1)
    }

    fn render_cell_text(&mut self, text: &str, font: &str, font_tag: &str, size: f64,
                        x: f64, mut y: f64, max_w: f64, lh: f64) {
        let sw = text_width(" ", font, size);
        for line in text.split('\n') {
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.is_empty() { y -= lh; continue; }
            let mut cl = String::new();
            let mut cw = 0.0;
            for w in &words {
                let ww = text_width(w, font, size);
                if !cl.is_empty() && cw + sw + ww > max_w {
                    self.current_stream.text_at(x, y, font_tag, size, &cl);
                    y -= lh; cl = w.to_string(); cw = ww;
                } else {
                    if !cl.is_empty() { cl.push(' '); cw += sw; }
                    cl.push_str(w); cw += ww;
                }
            }
            if !cl.is_empty() {
                if cw > max_w { cl = truncate_text(&cl, font, size, max_w); }
                self.current_stream.text_at(x, y, font_tag, size, &cl);
                y -= lh;
            }
        }
    }

    // ─── List ───────────────────────────────────────────────
    //
    // Draws marker via text_at at margin_left, then renders body
    // text with increased margin_left so body is indented past marker.
    // Both share the same cursor_y so they're on the same line.

    fn emit_list(&mut self, ui: &UIElement) {
        let ordered = ui.modifiers.iter().any(|m| m == "ordered");
        let fs = self.current_font_size;
        let lh = fs * 1.5;
        let df = self.default_font.clone();
        let ft = self.font_tag(&df);
        let indent = 16.0;

        for (idx, child) in ui.children.iter().enumerate() {
            // List children might be bare UIElements or wrapped in other statements
            let item_opt = match child {
                Statement::UIElement(u) => Some(u),
                _ => None,
            };
            if let Some(item) = item_opt {
                let text = self.extract_text_content(item);
                if text.is_empty() { continue; }

                self.check_page_break(lh);

                // Draw marker at margin_left, same baseline as text
                let marker = if ordered { format!("{}.", idx + 1) } else { "-".to_string() };
                let marker_font = self.font_tag(&df);
                self.current_stream.set_color(0.35, 0.35, 0.35);
                self.current_stream.text_at(self.margin_left, self.cursor_y, &marker_font, fs, &marker);

                // Draw body text indented past the marker
                let saved_ml = self.margin_left;
                self.margin_left += indent;
                self.render_wrapped_text(&text, &df, fs, (0.1, 0.1, 0.1), "");
                self.margin_left = saved_ml;

                self.mark_content();
            }
        }
        self.cursor_y -= 4.0;
    }

    // ─── Code Block ─────────────────────────────────────────

    fn emit_code(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() { return; }
        let is_block = ui.modifiers.iter().any(|m| m == "block");

        if is_block {
            let cfs = 10.0;
            let clh = cfs * 1.4;
            let pv = 12.0;
            let ph = 8.0;
            let lines: Vec<&str> = text.split('\n').collect();
            let total_h = lines.len() as f64 * clh + pv * 2.0;

            if total_h <= self.available_height() {
                self.render_code_block(&lines, cfs, clh, pv, ph, total_h);
            } else {
                let mut rem = &lines[..];
                while !rem.is_empty() {
                    let avail = self.available_height();
                    let max_l = ((avail - pv * 2.0) / clh).floor().max(1.0) as usize;
                    let take = rem.len().min(max_l);
                    let chunk = &rem[..take];
                    rem = &rem[take..];
                    let ch = chunk.len() as f64 * clh + pv * 2.0;
                    self.render_code_block(chunk, cfs, clh, pv, ph, ch);
                    if !rem.is_empty() { self.finalize_page(); self.start_new_page(); }
                }
            }
        } else {
            let ft = self.font_tag("Courier");
            let lh = self.current_font_size * 1.5;
            self.check_page_break(lh);
            self.current_stream.set_color(0.2, 0.2, 0.2);
            self.current_stream.text_at(self.margin_left, self.cursor_y, &ft, self.current_font_size, &text);
            self.cursor_y -= lh;
            self.mark_content();
        }
    }

    fn render_code_block(&mut self, lines: &[&str], fs: f64, lh: f64, pv: f64, ph: f64, bh: f64) {
        let ft = self.font_tag("Courier");
        let mtw = self.content_width() - ph * 2.0;

        // Background (drawn FIRST, behind text)
        self.current_stream.set_color(0.95, 0.95, 0.95);
        self.current_stream.rect(self.margin_left, self.cursor_y - bh, self.content_width(), bh);
        self.current_stream.fill();
        self.current_stream.set_stroke_color(0.85, 0.85, 0.85);
        self.current_stream.set_line_width(0.5);
        self.current_stream.rect(self.margin_left, self.cursor_y - bh, self.content_width(), bh);
        self.current_stream.stroke();

        // Text (drawn AFTER background)
        self.current_stream.set_color(0.2, 0.2, 0.2);
        let mut y = self.cursor_y - pv - fs;
        for line in lines {
            let dl = truncate_text(line, "Courier", fs, mtw);
            self.current_stream.text_at(self.margin_left + ph, y, &ft, fs, &dl);
            y -= lh;
        }
        self.cursor_y -= bh + 8.0;
        self.mark_content();
    }

    // ─── Blockquote ─────────────────────────────────────────
    //
    // Strategy: render text FIRST to know exact height, THEN draw the bar.
    // The bar is drawn on top of nothing (it's to the left of the text),
    // so z-order doesn't matter.

    fn emit_blockquote(&mut self, ui: &UIElement) {
        let bar_width = 3.0;
        let bar_x = 8.0;       // offset from margin_left
        let text_indent = 20.0; // text starts here from margin_left

        let mut texts = Vec::new();
        for c in &ui.children {
            if let Statement::UIElement(tu) = c {
                let t = self.extract_text_content(tu);
                if !t.is_empty() { texts.push(t); }
            }
        }
        if texts.is_empty() { return; }

        let full = texts.join("\n");
        let font = self.default_font.clone();
        let size = self.current_font_size;
        let lh = size * 1.5;

        let avail = self.content_width() - text_indent;
        let lc = self.count_wrapped_lines(&full, &font, size, avail);
        let th = lc as f64 * lh;
        self.check_page_break(th.min(lh * 2.0));

        // Record start Y (before text renders)
        let y_before = self.cursor_y;

        // Render text indented
        let saved = self.margin_left;
        self.margin_left += text_indent;
        self.render_wrapped_text(&full, &font, size, (0.30, 0.30, 0.30), "");
        self.margin_left = saved;

        // Record end Y (after text)
        let y_after = self.cursor_y;

        // Now draw the bar AFTER text, using exact positions.
        // Bar top = y_before + ascender height (~75% of font size)
        // Bar bottom = y_after + a small upward offset so bar doesn't extend too far below
        let bar_top = y_before + size * 0.75;
        let bar_bot = y_after + lh * 0.35;
        let bar_h = bar_top - bar_bot;
        if bar_h > 0.0 {
            self.current_stream.set_color(0.70, 0.70, 0.70);
            self.current_stream.rounded_rect(
                saved + bar_x, bar_bot, bar_width, bar_h, 1.5,
            );
            self.current_stream.fill();
        }

        self.cursor_y -= 8.0;
    }

    // ─── Image ──────────────────────────────────────────────

    fn emit_image(&mut self, ui: &UIElement) {
        let mut width = 200.0f64;
        let mut height = 150.0f64;
        for arg in &ui.args {
            if let crate::parser::Arg::Named(name, value) = arg {
                if let Expr::StringLiteral(_s) = value {
                    match name.as_str() { "src" => {} _ => {} }
                }
            }
        }
        if let Some(style) = &ui.style_block {
            for sp in &style.properties {
                match sp.name.as_str() {
                    "width"  => { if let Expr::NumberLiteral(n) = sp.value { width = n; } }
                    "height" => { if let Expr::NumberLiteral(n) = sp.value { height = n; } }
                    _ => {}
                }
            }
        }
        self.check_page_break(height + 8.0);
        self.current_stream.set_color(0.93, 0.93, 0.93);
        self.current_stream.rect(self.margin_left, self.cursor_y - height, width, height);
        self.current_stream.fill();
        self.current_stream.set_stroke_color(0.8, 0.8, 0.8);
        self.current_stream.set_line_width(1.0);
        self.current_stream.rect(self.margin_left, self.cursor_y - height, width, height);
        self.current_stream.stroke();
        let ft = self.font_tag(&self.default_font.clone());
        let lbl = "[Image]";
        let lw = text_width(lbl, &self.default_font, 10.0);
        self.current_stream.set_color(0.5, 0.5, 0.5);
        self.current_stream.text_at(self.margin_left + (width - lw) / 2.0, self.cursor_y - height / 2.0, &ft, 10.0, lbl);
        self.cursor_y -= height + 8.0;
        self.mark_content();
    }

    // ─── Card ───────────────────────────────────────────────
    //
    // Draw border AFTER content (stroke only, no fill that covers text).

    fn emit_card(&mut self, ui: &UIElement) {
        let pad = 14.0;
        let margin = 6.0;
        let radius = 4.0;
        let saved_l = self.margin_left;
        let saved_r = self.margin_right;

        self.cursor_y -= margin;
        self.margin_left += pad + margin;
        self.margin_right += pad + margin;

        let start_y = self.cursor_y;
        let start_pages = self.pages.len();
        self.cursor_y -= pad;

        for c in &ui.children { self.emit_statement(c); }

        self.cursor_y -= pad;
        let end_y = self.cursor_y;

        self.margin_left = saved_l;
        self.margin_right = saved_r;

        // Only draw border (no fill!) so we don't cover content
        if self.pages.len() == start_pages {
            let h = start_y - end_y;
            let x = saved_l + margin;
            let w = self.page_width - saved_l - saved_r - margin * 2.0;
            self.current_stream.set_stroke_color(0.82, 0.82, 0.82);
            self.current_stream.set_line_width(0.75);
            self.current_stream.rounded_rect(x, end_y, w, h, radius);
            self.current_stream.stroke();
        }
        self.cursor_y -= margin;
    }

    // ─── Badge ──────────────────────────────────────────────

    fn emit_badge(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() { return; }

        let sz = 9.0;
        let ft = self.font_tag("Helvetica-Bold");
        let tw = text_width(&text, "Helvetica-Bold", sz);
        let pad = 6.0;
        let bh = sz + pad * 2.0;
        let bw = tw + pad * 2.0;
        let br = bh / 2.0;

        self.check_page_break(bh + 4.0);

        // Pill background
        let color = self.modifier_color(&ui.modifiers);
        let pill_y = self.cursor_y - sz - pad; // bottom of pill
        self.current_stream.set_color(color.0, color.1, color.2);
        self.current_stream.rounded_rect(self.margin_left, pill_y, bw, bh, br);
        self.current_stream.fill();

        // Text centered in pill
        let text_y = pill_y + pad;
        self.current_stream.set_color(1.0, 1.0, 1.0);
        self.current_stream.text_at(self.margin_left + pad, text_y, &ft, sz, &text);

        self.cursor_y -= bh + 6.0;
        self.mark_content();
    }

    // ─── Alert ──────────────────────────────────────────────

    fn emit_alert(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() { return; }

        let color = self.modifier_color(&ui.modifiers);
        let pad = 12.0;
        let font = self.default_font.clone();
        let fs = self.current_font_size;
        let lh = fs * 1.5;
        let tw_avail = self.content_width() - pad * 2.0 - 4.0;
        let lc = self.count_wrapped_lines(&text, &font, fs, tw_avail);
        let box_h = lc as f64 * lh + pad * 2.0;

        self.check_page_break(box_h.min(lh * 3.0 + pad * 2.0));

        let box_top = self.cursor_y;
        let box_bot = box_top - box_h;

        // Background (drawn first)
        self.current_stream.set_color(color.0 * 0.15 + 0.85, color.1 * 0.15 + 0.85, color.2 * 0.15 + 0.85);
        self.current_stream.rect(self.margin_left, box_bot, self.content_width(), box_h);
        self.current_stream.fill();

        // Left accent bar
        self.current_stream.set_color(color.0, color.1, color.2);
        self.current_stream.rect(self.margin_left, box_bot, 3.0, box_h);
        self.current_stream.fill();

        // Text (drawn after background)
        let ft = self.font_tag(&font);
        let text_x = self.margin_left + pad + 4.0;
        let mut ty = box_top - pad - fs;
        let sw = text_width(" ", &font, fs);
        self.current_stream.set_color(0.15, 0.15, 0.15);

        for raw_line in text.split('\n') {
            let words: Vec<&str> = raw_line.split_whitespace().collect();
            if words.is_empty() { ty -= lh; continue; }
            let mut cl = String::new();
            let mut cw = 0.0;
            for w in &words {
                let ww = text_width(w, &font, fs);
                if !cl.is_empty() && cw + sw + ww > tw_avail {
                    self.current_stream.text_at(text_x, ty, &ft, fs, &cl);
                    ty -= lh; cl = w.to_string(); cw = ww;
                } else {
                    if !cl.is_empty() { cl.push(' '); cw += sw; }
                    cl.push_str(w); cw += ww;
                }
            }
            if !cl.is_empty() {
                self.current_stream.text_at(text_x, ty, &ft, fs, &cl);
                ty -= lh;
            }
        }

        self.cursor_y = box_bot - 8.0;
        self.mark_content();
    }

    // ─── Progress ───────────────────────────────────────────

    fn emit_progress(&mut self, ui: &UIElement) {
        let mut val = 0.0f64;
        let mut max = 100.0f64;
        for arg in &ui.args {
            if let crate::parser::Arg::Named(n, v) = arg {
                if let Expr::NumberLiteral(num) = v {
                    match n.as_str() { "value" => val = *num, "max" => max = *num, _ => {} }
                }
            }
        }
        let bh = 8.0;
        let br = bh / 2.0;
        let frac = (val / max).min(1.0).max(0.0);

        self.check_page_break(bh + 8.0);

        // Track (behind fill)
        let bar_y = self.cursor_y - bh;
        self.current_stream.set_color(0.9, 0.9, 0.9);
        self.current_stream.rounded_rect(self.margin_left, bar_y, self.content_width(), bh, br);
        self.current_stream.fill();

        // Fill
        let fw = self.content_width() * frac;
        if fw > 0.5 {
            let color = self.modifier_color(&ui.modifiers);
            self.current_stream.set_color(color.0, color.1, color.2);
            self.current_stream.rounded_rect(self.margin_left, bar_y, fw, bh, br);
            self.current_stream.fill();
        }
        self.cursor_y -= bh + 8.0;
        self.mark_content();
    }

    // ─── Text Rendering (with word wrap) ────────────────────
    //
    // Renders text at cursor_y (baseline), then decrements cursor_y
    // by line_height for each line rendered.

    fn render_wrapped_text(&mut self, text: &str, font: &str, size: f64, color: (f64, f64, f64), align: &str) {
        let ft = self.font_tag(font);
        let lh = size * 1.5;
        let aw = self.content_width();
        let sw = text_width(" ", font, size);

        for line in text.split('\n') {
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.is_empty() { self.cursor_y -= lh * 0.5; continue; }

            let mut cl = String::new();
            let mut cw = 0.0;

            for w in &words {
                let ww = text_width(w, font, size);
                if !cl.is_empty() && cw + sw + ww > aw {
                    self.check_page_break(lh);
                    let x = self.text_x_for_align(&cl, font, size, align);
                    self.current_stream.set_color(color.0, color.1, color.2);
                    self.current_stream.text_at(x, self.cursor_y, &ft, size, &cl);
                    self.cursor_y -= lh;
                    self.mark_content();
                    cl = w.to_string(); cw = ww;
                } else {
                    if !cl.is_empty() { cl.push(' '); cw += sw; }
                    cl.push_str(w); cw += ww;
                }
            }
            if !cl.is_empty() {
                self.check_page_break(lh);
                let x = self.text_x_for_align(&cl, font, size, align);
                self.current_stream.set_color(color.0, color.1, color.2);
                self.current_stream.text_at(x, self.cursor_y, &ft, size, &cl);
                self.cursor_y -= lh;
                self.mark_content();
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

    fn extract_text_content(&self, ui: &UIElement) -> String {
        for arg in &ui.args {
            if let crate::parser::Arg::Positional(expr) = arg {
                let text = self.expr_to_string(expr);
                return self.substitute_page_vars(&text);
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

    fn extract_text_style(&self, ui: &UIElement) -> (String, f64, (f64, f64, f64), String) {
        let mut font = self.default_font.clone();
        let mut size = self.default_font_size;
        let mut color = (0.0, 0.0, 0.0);
        let mut align = String::new();

        for m in &ui.modifiers {
            match m.as_str() {
                "bold" => font = format!("{}-Bold", font.split('-').next().unwrap_or("Helvetica")),
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

    fn modifier_color(&self, mods: &[String]) -> (f64, f64, f64) {
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
        (0.39, 0.39, 0.95)
    }

    fn get_spacer_size(&self, ui: &UIElement) -> f64 {
        for m in &ui.modifiers {
            match m.as_str() {
                "small" | "sm" => return 8.0,
                "medium" | "md" => return 16.0,
                "large" | "lg" => return 24.0,
                "xl" => return 32.0,
                _ => {}
            }
        }
        16.0
    }

    // ─── Container with full styling ────────────────────────
    //
    // Mirror of slides::emit_container: shadow → background → border → children,
    // ordered via splice into the content stream so children paint on top.

    fn emit_container(&mut self, ui: &UIElement, component: &str) {
        let style = ui.style_block.as_ref().map(StyleProps::from_block);
        if let Some(s) = &style {
            self.warn_unsupported(component, &s.unknown);
        }

        let s = match style {
            Some(s) if container_has_box_styling_pdf(&s) => s,
            _ => {
                for c in &ui.children { self.emit_statement(c); }
                return;
            }
        };

        let parent_w = self.content_width();
        let outer_x = self.margin_left;
        let mut outer_w = s.width.map(|d| d.resolve(parent_w)).unwrap_or(parent_w);
        if outer_w > parent_w { outer_w = parent_w; }
        let explicit_h = s.height.map(|d| d.resolve(self.page_height));

        let saved_left = self.margin_left;
        let saved_right = self.margin_right;
        let saved_y = self.cursor_y;

        let pad = s.padding;
        self.margin_left  = outer_x + pad.left;
        self.margin_right = self.page_width - (outer_x + outer_w) + pad.right;
        self.cursor_y     -= pad.top;

        let splice_at = self.current_stream.ops.len();
        for c in &ui.children { self.emit_statement(c); }
        self.cursor_y -= pad.bottom;

        self.margin_left = saved_left;
        self.margin_right = saved_right;

        let measured_h = saved_y - self.cursor_y;
        let outer_h = explicit_h.unwrap_or(measured_h);
        let outer_y = saved_y - outer_h;
        if explicit_h.is_some() {
            self.cursor_y = outer_y;
        }

        let bg_ops = self.build_box_decoration_pdf(&s, outer_x, outer_y, outer_w, outer_h);
        if !bg_ops.is_empty() {
            self.current_stream.ops.splice(splice_at..splice_at, bg_ops);
            self.mark_content();
        }
    }

    fn register_gradient_pdf(&mut self, g: &LinearGradient, x: f64, y: f64, w: f64, h: f64) -> String {
        let tag = format!("Sh{}", self.shadings.len() + 1);
        let ((x0, y0), (x1, y1)) = gradient_endpoints(g.angle_deg, x, y, w, h);
        self.shadings.push(GradientInstance {
            tag: tag.clone(),
            start: g.start, end: g.end,
            x0, y0, x1, y1,
        });
        tag
    }

    fn build_box_decoration_pdf(&mut self, s: &StyleProps, x: f64, y: f64, w: f64, h: f64) -> Vec<u8> {
        let mut tmp = ContentStream::new();
        let radius = s.border_radius.unwrap_or(0.0);

        if let Some(sh) = &s.box_shadow {
            let sx = x + sh.offset_x;
            let sy = y - sh.offset_y;
            tmp.set_color(sh.color.0, sh.color.1, sh.color.2);
            if radius > 0.5 {
                tmp.rounded_rect_path(sx, sy, w, h, radius);
            } else {
                tmp.rect(sx, sy, w, h);
            }
            tmp.fill();
        }

        match &s.background {
            Some(Background::Color(c)) => {
                tmp.set_color(c.0, c.1, c.2);
                if radius > 0.5 {
                    tmp.rounded_rect_path(x, y, w, h, radius);
                } else {
                    tmp.rect(x, y, w, h);
                }
                tmp.fill();
            }
            Some(Background::LinearGradient(g)) => {
                let tag = self.register_gradient_pdf(g, x, y, w, h);
                tmp.op("q");
                if radius > 0.5 {
                    tmp.rounded_rect_path(x, y, w, h, radius);
                } else {
                    tmp.rect(x, y, w, h);
                }
                tmp.op("W n");
                tmp.op(&format!("/{} sh", tag));
                tmp.op("Q");
            }
            None => {}
        }

        if let Some(bw) = s.border_width {
            let bc = s.border_color.unwrap_or((0.5, 0.5, 0.5));
            tmp.set_stroke_color(bc.0, bc.1, bc.2);
            tmp.set_line_width(bw);
            if radius > 0.5 {
                tmp.rounded_rect_path(x, y, w, h, radius);
            } else {
                tmp.rect(x, y, w, h);
            }
            tmp.stroke();
        }

        tmp.bytes().to_vec()
    }

    fn warn_unsupported(&mut self, component: &str, unknown: &[String]) {
        for prop in unknown {
            let key = format!("{}::{}", component, prop);
            if self.warned_styles.insert(key) {
                eprintln!("warning[pdf]: unsupported style property '{}' on {}", prop, component);
            }
        }
    }

    // ─── PDF Serialization ──────────────────────────────────

    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut offsets: Vec<usize> = Vec::new();

        out.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

        let font_start = 3;
        let num_fonts = self.fonts.len();
        let shading_func_start = font_start + num_fonts;
        let shading_dict_start = shading_func_start + self.shadings.len();
        let resources_id = shading_dict_start + self.shadings.len();

        let mut final_objects: Vec<(usize, Vec<u8>)> = Vec::new();

        // 1: Catalog
        final_objects.push((1, b"<< /Type /Catalog /Pages 2 0 R >>".to_vec()));

        // Fonts
        for (i, (_, bf)) in self.fonts.iter().enumerate() {
            final_objects.push((font_start + i,
                format!("<< /Type /Font /Subtype /Type1 /BaseFont /{} /Encoding /WinAnsiEncoding >>", bf).into_bytes()));
        }

        // Gradient functions + shading dicts (Type 2 axial).
        for (i, g) in self.shadings.iter().enumerate() {
            let func_obj = format!(
                "<< /FunctionType 2 /Domain [0 1] /N 1 /C0 [{} {} {}] /C1 [{} {} {}] >>",
                fmt_f64(g.start.0), fmt_f64(g.start.1), fmt_f64(g.start.2),
                fmt_f64(g.end.0),   fmt_f64(g.end.1),   fmt_f64(g.end.2),
            );
            final_objects.push((shading_func_start + i, func_obj.into_bytes()));
        }
        for (i, g) in self.shadings.iter().enumerate() {
            let func_id = shading_func_start + i;
            let shading_obj = format!(
                "<< /ShadingType 2 /ColorSpace /DeviceRGB /Coords [{} {} {} {}] /Function {} 0 R /Extend [true true] >>",
                fmt_f64(g.x0), fmt_f64(g.y0), fmt_f64(g.x1), fmt_f64(g.y1), func_id,
            );
            final_objects.push((shading_dict_start + i, shading_obj.into_bytes()));
        }

        // Resources
        let mut fe = String::new();
        for (i, (tag, _)) in self.fonts.iter().enumerate() {
            fe.push_str(&format!("/{} {} 0 R ", tag, font_start + i));
        }
        let mut shading_entries = String::new();
        for (i, g) in self.shadings.iter().enumerate() {
            shading_entries.push_str(&format!("/{} {} 0 R ", g.tag, shading_dict_start + i));
        }
        let resources_dict = if shading_entries.is_empty() {
            format!("<< /Font << {} >> >>", fe)
        } else {
            format!("<< /Font << {} >> /Shading << {} >> >>", fe, shading_entries)
        };
        final_objects.push((resources_id, resources_dict.into_bytes()));

        // Content streams + Pages
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

// ─── Utility Functions ──────────────────────────────────────────────

fn container_has_box_styling_pdf(s: &StyleProps) -> bool {
    s.background.is_some()
        || !s.padding.is_zero()
        || s.border_width.is_some()
        || s.border_radius.is_some()
        || s.box_shadow.is_some()
        || s.width.is_some()
        || s.height.is_some()
}

fn fmt_f64(v: f64) -> String {
    if v == v.floor() { format!("{:.0}", v) } else { format!("{:.2}", v) }
}

fn char_to_winansi(ch: char) -> u8 {
    match ch {
        '\u{FFFE}' => b'{', '\u{FFFF}' => b'}',
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
