use crate::config::project::PdfConfig;
use crate::parser::{Program, Declaration, Statement, UIElement, ComponentRef, Expr, StringPart};

// ─── Base14 Font Metrics (Helvetica) ────────────────────────────────
// Character widths for Helvetica at 1000 units per em (standard AFM data)
// Covers ASCII 32-126. For characters outside this range, use the default width.
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

const COURIER_WIDTH: u16 = 600; // Monospace: all glyphs are 600 units

fn char_width(ch: char, font: &str) -> u16 {
    let code = ch as u32;
    if code < 32 || code > 126 {
        return 500; // default for unknown
    }
    let idx = (code - 32) as usize;
    if font.contains("Courier") {
        COURIER_WIDTH
    } else if font.contains("Times") {
        if font.contains("Bold") {
            // Times-Bold is close enough to Times-Roman widths for layout
            TIMES_WIDTHS[idx]
        } else {
            TIMES_WIDTHS[idx]
        }
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

/// Truncate text to fit within max_width, appending "..." if truncated
fn truncate_text(text: &str, font: &str, font_size: f64, max_width: f64) -> String {
    let full_w = text_width(text, font, font_size);
    if full_w <= max_width {
        return text.to_string();
    }
    let ellipsis = "...";
    let ellipsis_w = text_width(ellipsis, font, font_size);
    let target = max_width - ellipsis_w;
    if target <= 0.0 {
        return ellipsis.to_string();
    }
    let mut w = 0.0;
    let mut end = 0;
    for (i, ch) in text.char_indices() {
        let cw = char_width(ch, font) as f64 * font_size / 1000.0;
        if w + cw > target {
            break;
        }
        w += cw;
        end = i + ch.len_utf8();
    }
    format!("{}...", &text[..end])
}

// ─── Page Size Constants (in points, 72 pts = 1 inch) ───────────────

fn page_dimensions(size: &str) -> (f64, f64) {
    match size.to_uppercase().as_str() {
        "A4" => (595.28, 841.89),
        "A3" => (841.89, 1190.55),
        "A5" => (419.53, 595.28),
        "LETTER" => (612.0, 792.0),
        "LEGAL" => (612.0, 1008.0),
        _ => (595.28, 841.89), // default A4
    }
}

// ─── PDF Object Types ────────────────────────────────────────────────

struct PdfObj {
    id: usize,
    data: Vec<u8>,
}

// ─── Content Stream Builder ─────────────────────────────────────────

struct ContentStream {
    ops: Vec<u8>,
}

impl ContentStream {
    fn new() -> Self {
        Self { ops: Vec::new() }
    }

    fn op(&mut self, s: &str) {
        self.ops.extend_from_slice(s.as_bytes());
        self.ops.push(b'\n');
    }

    fn set_font(&mut self, font_tag: &str, size: f64) {
        self.op(&format!("/{} {} Tf", font_tag, fmt_f64(size)));
    }

    fn set_color(&mut self, r: f64, g: f64, b: f64) {
        self.op(&format!("{} {} {} rg", fmt_f64(r), fmt_f64(g), fmt_f64(b)));
    }

    fn set_stroke_color(&mut self, r: f64, g: f64, b: f64) {
        self.op(&format!("{} {} {} RG", fmt_f64(r), fmt_f64(g), fmt_f64(b)));
    }

    fn begin_text(&mut self) {
        self.op("BT");
    }

    fn end_text(&mut self) {
        self.op("ET");
    }

    fn text_position(&mut self, x: f64, y: f64) {
        self.op(&format!("{} {} Td", fmt_f64(x), fmt_f64(y)));
    }

    fn show_text(&mut self, text: &str) {
        let hex = text_to_pdf_hex(text);
        self.op(&format!("<{}> Tj", hex));
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

    fn fill(&mut self) {
        self.op("f");
    }

    fn stroke(&mut self) {
        self.op("S");
    }

    fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.op(&format!("{} {} m {} {} l S", fmt_f64(x1), fmt_f64(y1), fmt_f64(x2), fmt_f64(y2)));
    }

    fn set_line_width(&mut self, w: f64) {
        self.op(&format!("{} w", fmt_f64(w)));
    }

    /// Draw a rounded rectangle using cubic Bezier curves
    fn rounded_rect(&mut self, x: f64, y: f64, w: f64, h: f64, r: f64) {
        // Clamp radius to half the smallest dimension
        let r = r.min(w / 2.0).min(h / 2.0);
        // Magic number for approximating a quarter circle with a cubic Bezier
        let k = 0.5523;
        let kr = k * r;

        // Start at bottom-left + radius
        self.op(&format!("{} {} m", fmt_f64(x + r), fmt_f64(y)));
        // Bottom edge → bottom-right corner
        self.op(&format!("{} {} l", fmt_f64(x + w - r), fmt_f64(y)));
        self.op(&format!("{} {} {} {} {} {} c",
            fmt_f64(x + w - r + kr), fmt_f64(y),
            fmt_f64(x + w), fmt_f64(y + r - kr),
            fmt_f64(x + w), fmt_f64(y + r)));
        // Right edge → top-right corner
        self.op(&format!("{} {} l", fmt_f64(x + w), fmt_f64(y + h - r)));
        self.op(&format!("{} {} {} {} {} {} c",
            fmt_f64(x + w), fmt_f64(y + h - r + kr),
            fmt_f64(x + w - r + kr), fmt_f64(y + h),
            fmt_f64(x + w - r), fmt_f64(y + h)));
        // Top edge → top-left corner
        self.op(&format!("{} {} l", fmt_f64(x + r), fmt_f64(y + h)));
        self.op(&format!("{} {} {} {} {} {} c",
            fmt_f64(x + r - kr), fmt_f64(y + h),
            fmt_f64(x), fmt_f64(y + h - r + kr),
            fmt_f64(x), fmt_f64(y + h - r)));
        // Left edge → bottom-left corner
        self.op(&format!("{} {} l", fmt_f64(x), fmt_f64(y + r)));
        self.op(&format!("{} {} {} {} {} {} c",
            fmt_f64(x), fmt_f64(y + r - kr),
            fmt_f64(x + r - kr), fmt_f64(y),
            fmt_f64(x + r), fmt_f64(y)));
        self.op("h"); // close path
    }

    fn bytes(&self) -> &[u8] {
        &self.ops
    }
}

// ─── PDF Writer ─────────────────────────────────────────────────────

pub struct PdfCodegen {
    objects: Vec<PdfObj>,
    pages: Vec<usize>,       // Page object IDs
    page_width: f64,
    page_height: f64,
    margin_top: f64,
    margin_bottom: f64,
    margin_left: f64,
    margin_right: f64,
    cursor_y: f64,
    current_stream: ContentStream,
    fonts: Vec<(String, String)>, // (tag like "F1", base_font like "Helvetica")
    current_font: String,         // current font tag
    current_font_name: String,    // current base font name
    current_font_size: f64,
    current_color: (f64, f64, f64),
    default_font: String,
    default_font_size: f64,
    // Header/footer content stored for each page
    header_stmts: Vec<Statement>,
    footer_stmts: Vec<Statement>,
    // Image objects for JPEG embedding
    image_objects: Vec<(usize, f64, f64)>, // (obj_id, width, height)
    // Guard against recursion in header/footer rendering
    in_header_footer: bool,
    // Track if the current page has any visible content
    page_has_content: bool,
}

impl PdfCodegen {
    pub fn new(config: &PdfConfig) -> Self {
        let (w, h) = page_dimensions(&config.page_size);
        let mut codegen = Self {
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
        };

        // Register default fonts
        codegen.register_font(&config.default_font);
        codegen.register_font("Helvetica-Bold");
        codegen.register_font("Courier");
        codegen.register_font("Courier-Bold");
        codegen.register_font("Times-Roman");
        codegen.register_font("Times-Bold");

        codegen
    }

    fn register_font(&mut self, base_font: &str) -> String {
        // Check if already registered
        for (tag, name) in &self.fonts {
            if name == base_font {
                return tag.clone();
            }
        }
        let tag = format!("F{}", self.fonts.len() + 1);
        self.fonts.push((tag.clone(), base_font.to_string()));
        tag
    }

    fn font_tag(&self, base_font: &str) -> String {
        for (tag, name) in &self.fonts {
            if name == base_font {
                return tag.clone();
            }
        }
        "F1".to_string()
    }

    fn content_width(&self) -> f64 {
        self.page_width - self.margin_left - self.margin_right
    }

    /// Available vertical space on the current page
    fn available_height(&self) -> f64 {
        self.cursor_y - self.margin_bottom
    }

    fn check_page_break(&mut self, needed: f64) {
        if self.cursor_y - needed < self.margin_bottom && !self.in_header_footer {
            self.finalize_page();
            self.start_new_page();
        }
    }

    fn start_new_page(&mut self) {
        self.cursor_y = self.page_height - self.margin_top;
        self.current_stream = ContentStream::new();
        self.page_has_content = false;

        // Emit header on new page — render in the top margin area
        if !self.header_stmts.is_empty() && !self.in_header_footer {
            self.in_header_footer = true;
            let saved_y = self.cursor_y;
            // Position header with enough room: top of page minus a small top pad
            self.cursor_y = self.page_height - 24.0;
            let stmts = self.header_stmts.clone();
            for stmt in &stmts {
                self.emit_statement(stmt);
            }
            self.cursor_y = saved_y;
            self.in_header_footer = false;
        }
    }

    fn finalize_page(&mut self) {
        // Emit footer before finalizing — render a single line at a safe position
        if !self.footer_stmts.is_empty() && !self.in_header_footer {
            self.in_header_footer = true;
            let stmts = self.footer_stmts.clone();
            let saved_y = self.cursor_y;
            // Place footer text baseline at 28pt from page bottom —
            // this leaves room for the text descenders and avoids clipping
            self.cursor_y = 28.0;
            for stmt in &stmts {
                self.emit_statement(stmt);
            }
            self.cursor_y = saved_y;
            self.in_header_footer = false;
        }

        // Only create a page if there's actual content
        let stream_data = std::mem::replace(&mut self.current_stream, ContentStream::new());
        if stream_data.bytes().is_empty() && !self.page_has_content {
            return; // Skip empty pages
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
        let id = self.objects.len() + 1; // 1-indexed
        self.objects.push(PdfObj {
            id,
            data: data.to_vec(),
        });
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

    fn mark_content(&mut self) {
        self.page_has_content = true;
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    // ─── Main Entry Point ───────────────────────────────────

    pub fn generate(&mut self, program: &Program) -> Vec<u8> {
        self.start_new_page();

        for decl in &program.declarations {
            match decl {
                Declaration::Page(page) => {
                    for stmt in &page.body {
                        self.emit_statement(stmt);
                    }
                }
                Declaration::Component(_) => {
                    // Components are inlined when referenced
                }
                _ => {}
            }
        }

        // Finalize the last page
        self.finalize_page();

        self.serialize()
    }

    // ─── Statement Emitters ─────────────────────────────────

    fn emit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::UIElement(ui) => self.emit_ui_element(ui),
            Statement::If(if_stmt) => {
                // In PDF, we just render the then branch (no reactivity)
                for s in &if_stmt.then_body {
                    self.emit_statement(s);
                }
            }
            Statement::For(for_stmt) => {
                // For loops in PDF: evaluate the list if it's a literal
                if let Expr::ListLiteral(items) = &for_stmt.iterable {
                    for _item in items {
                        for s in &for_stmt.body {
                            self.emit_statement(s);
                        }
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
                // User components: just recurse into children
                for child in &ui.children {
                    self.emit_statement(child);
                }
                return;
            }
        };

        match name.as_str() {
            "Document" => self.emit_document(ui),
            "Section" => self.emit_section(ui),
            "Paragraph" => self.emit_paragraph(ui),
            "Header" => self.emit_header(ui),
            "Footer" => self.emit_footer(ui),
            "PageBreak" => self.emit_page_break(),
            "Text" => self.emit_text(ui),
            "Heading" => self.emit_heading(ui),
            "Table" => self.emit_table(ui),
            "Thead" | "Tbody" => {
                for child in &ui.children {
                    self.emit_statement(child);
                }
            }
            "Trow" => self.emit_table_row(ui),
            "Code" => self.emit_code(ui),
            "Blockquote" => self.emit_blockquote(ui),
            "Image" => self.emit_image(ui),
            "Divider" => self.emit_divider(),
            "Container" | "Row" | "Column" | "Grid" | "Stack" => {
                // Layout containers: just render children sequentially
                for child in &ui.children {
                    self.emit_statement(child);
                }
            }
            "Spacer" => {
                let size = self.get_spacer_size(ui);
                self.cursor_y -= size;
            }
            "List" => self.emit_list(ui),
            "Badge" | "Tag" => self.emit_inline_badge(ui),
            "Alert" => self.emit_alert(ui),
            "Progress" => self.emit_progress(ui),
            "Card" => self.emit_card(ui),
            "Card.Header" | "Card.Body" | "Card.Footer" => {
                for child in &ui.children {
                    self.emit_statement(child);
                }
            }
            "Avatar" => {} // Skip avatars in PDF
            "Icon" => {}   // Skip icons in PDF
            _ => {
                // Unknown elements: try rendering children
                for child in &ui.children {
                    self.emit_statement(child);
                }
            }
        }
    }

    // ─── Document-Level Elements ────────────────────────────

    fn emit_document(&mut self, ui: &UIElement) {
        // Override page size/margins from args
        for arg in &ui.args {
            if let crate::parser::Arg::Named(name, value) = arg {
                match name.as_str() {
                    "page_size" | "pageSize" => {
                        if let Expr::StringLiteral(s) = value {
                            let (w, h) = page_dimensions(s);
                            self.page_width = w;
                            self.page_height = h;
                            self.cursor_y = h - self.margin_top;
                        }
                    }
                    _ => {}
                }
            }
        }

        for child in &ui.children {
            self.emit_statement(child);
        }
    }

    fn emit_section(&mut self, ui: &UIElement) {
        self.cursor_y -= 8.0;
        for child in &ui.children {
            self.emit_statement(child);
        }
        self.cursor_y -= 4.0;
    }

    fn emit_header(&mut self, ui: &UIElement) {
        self.header_stmts = ui.children.clone();
    }

    fn emit_footer(&mut self, ui: &UIElement) {
        self.footer_stmts = ui.children.clone();
    }

    fn emit_page_break(&mut self) {
        self.finalize_page();
        self.start_new_page();
    }

    fn emit_paragraph(&mut self, ui: &UIElement) {
        let (font, size, color, align) = self.extract_text_style(ui);

        for child in &ui.children {
            if let Statement::UIElement(text_ui) = child {
                let text = self.extract_text_content(text_ui);
                if !text.is_empty() {
                    let (child_font, child_size, child_color, child_align) = self.extract_text_style(text_ui);
                    let f = if child_font != self.default_font { &child_font } else { &font };
                    let s = if (child_size - self.default_font_size).abs() > 0.1 { child_size } else { size };
                    let c = if child_color != (0.0, 0.0, 0.0) { child_color } else { color };
                    let a = if !child_align.is_empty() { &child_align } else { &align };
                    self.render_wrapped_text(&text, f, s, c, a);
                }
            }
        }
        // Paragraph spacing
        self.cursor_y -= self.current_font_size * 0.5;
    }

    // ─── Text and Heading ───────────────────────────────────

    fn emit_text(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() {
            return;
        }

        let (font, size, color, align) = self.extract_text_style(ui);
        self.render_wrapped_text(&text, &font, size, color, &align);
    }

    fn emit_heading(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() {
            return;
        }

        // Determine heading level from modifiers
        let mut level = 2; // default
        let mut color = (0.0, 0.0, 0.0);

        for modifier in &ui.modifiers {
            match modifier.as_str() {
                "h1" => level = 1,
                "h2" => level = 2,
                "h3" => level = 3,
                "h4" => level = 4,
                "h5" => level = 5,
                "h6" => level = 6,
                "primary" => color = (0.098, 0.098, 0.647), // #1919A5
                "muted" => color = (0.4, 0.4, 0.4),
                _ => {}
            }
        }

        let size = match level {
            1 => 28.0,
            2 => 22.0,
            3 => 18.0,
            4 => 16.0,
            5 => 14.0,
            _ => 12.0,
        };

        let font = "Helvetica-Bold".to_string();
        let (_, _, style_color, align) = self.extract_text_style(ui);
        if style_color != (0.0, 0.0, 0.0) {
            color = style_color;
        }

        // Spacing before heading
        self.cursor_y -= size * 0.4;
        self.check_page_break(size * 1.5);

        self.render_wrapped_text(&text, &font, size, color, &align);

        // Spacing after heading
        self.cursor_y -= size * 0.3;
    }

    // ─── Table ──────────────────────────────────────────────

    fn emit_table(&mut self, ui: &UIElement) {
        // Collect all rows
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut is_header: Vec<bool> = Vec::new();

        for child in &ui.children {
            if let Statement::UIElement(section) = child {
                let section_name = match &section.component {
                    ComponentRef::BuiltIn(n) => n.clone(),
                    _ => String::new(),
                };
                let header = section_name == "Thead";
                for row_stmt in &section.children {
                    if let Statement::UIElement(row) = row_stmt {
                        let cells: Vec<String> = row.children.iter().filter_map(|c| {
                            if let Statement::UIElement(cell) = c {
                                Some(self.extract_text_content(cell))
                            } else {
                                None
                            }
                        }).collect();
                        if !cells.is_empty() {
                            rows.push(cells);
                            is_header.push(header);
                        }
                    }
                }
            }
        }

        if rows.is_empty() {
            return;
        }

        let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(1);
        let content_w = self.content_width();
        let col_width = content_w / num_cols as f64;
        let cell_padding = 4.0;
        let cell_text_width = col_width - cell_padding * 2.0; // available text width per cell
        let font_size = self.current_font_size;
        let line_height = font_size * 1.5;

        // Calculate row heights based on wrapped text
        let mut row_heights: Vec<f64> = Vec::new();
        for row in &rows {
            let mut max_lines = 1usize;
            for cell_text in row {
                let lines = self.count_wrapped_lines(cell_text, &self.default_font.clone(), font_size, cell_text_width);
                if lines > max_lines {
                    max_lines = lines;
                }
            }
            let row_h = (max_lines as f64 * line_height) + cell_padding * 2.0;
            row_heights.push(row_h.max(font_size * 2.0)); // minimum height
        }

        // Check if first few rows fit
        let initial_height: f64 = row_heights.iter().take(2).sum();
        self.check_page_break(initial_height);

        let table_x = self.margin_left;
        self.mark_content();

        self.current_stream.set_line_width(0.5);

        for (row_idx, row) in rows.iter().enumerate() {
            let row_height = row_heights[row_idx];

            // Page break check for this row
            if self.cursor_y - row_height < self.margin_bottom && !self.in_header_footer {
                self.finalize_page();
                self.start_new_page();
            }

            let row_top = self.cursor_y;
            let cell_y = row_top - row_height;

            // Background for header rows
            if is_header[row_idx] {
                self.current_stream.set_color(0.93, 0.93, 0.93);
                self.current_stream.rect(table_x, cell_y, content_w, row_height);
                self.current_stream.fill();
            }

            // Row border
            self.current_stream.set_stroke_color(0.75, 0.75, 0.75);
            self.current_stream.rect(table_x, cell_y, content_w, row_height);
            self.current_stream.stroke();

            // Cell content
            let font_name = if is_header[row_idx] { "Helvetica-Bold".to_string() } else { self.default_font.clone() };
            let font_tag = self.font_tag(&font_name);

            for (col_idx, cell_text) in row.iter().enumerate() {
                let cell_x = table_x + col_idx as f64 * col_width;

                // Vertical cell border
                if col_idx > 0 {
                    self.current_stream.set_stroke_color(0.75, 0.75, 0.75);
                    self.current_stream.line(cell_x, row_top, cell_x, cell_y);
                }

                // Render wrapped text within the cell
                self.current_stream.set_color(0.0, 0.0, 0.0);
                self.render_cell_text(
                    cell_text,
                    &font_name,
                    &font_tag,
                    font_size,
                    cell_x + cell_padding,
                    row_top - cell_padding - font_size,
                    cell_text_width,
                    line_height,
                );
            }

            self.cursor_y = cell_y;
        }

        self.cursor_y -= 8.0;
    }

    /// Count how many wrapped lines a text needs at given width
    fn count_wrapped_lines(&self, text: &str, font: &str, size: f64, max_width: f64) -> usize {
        if max_width <= 0.0 || text.is_empty() {
            return 1;
        }
        let space_w = text_width(" ", font, size);
        let mut line_count = 0usize;

        for line_text in text.split('\n') {
            line_count += 1;
            let words: Vec<&str> = line_text.split_whitespace().collect();
            if words.is_empty() {
                continue;
            }
            let mut current_w = 0.0;
            for (i, word) in words.iter().enumerate() {
                let word_w = text_width(word, font, size);
                let needed = if i > 0 { space_w + word_w } else { word_w };
                if current_w > 0.0 && current_w + needed > max_width {
                    line_count += 1;
                    current_w = word_w;
                } else {
                    current_w += needed;
                }
            }
        }
        line_count.max(1)
    }

    /// Render text within a table cell with word wrapping
    fn render_cell_text(&mut self, text: &str, font: &str, font_tag: &str, size: f64,
                        x: f64, mut y: f64, max_width: f64, line_height: f64) {
        let space_w = text_width(" ", font, size);

        for line_text in text.split('\n') {
            let words: Vec<&str> = line_text.split_whitespace().collect();
            if words.is_empty() {
                y -= line_height;
                continue;
            }

            let mut current_line = String::new();
            let mut current_w = 0.0;

            for word in &words {
                let word_w = text_width(word, font, size);

                if !current_line.is_empty() && current_w + space_w + word_w > max_width {
                    // Flush line
                    self.current_stream.text_at(x, y, font_tag, size, &current_line);
                    y -= line_height;
                    current_line = word.to_string();
                    current_w = word_w;
                } else {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                        current_w += space_w;
                    }
                    current_line.push_str(word);
                    current_w += word_w;
                }
            }
            if !current_line.is_empty() {
                // If still too wide (single very long word), truncate
                if current_w > max_width {
                    let truncated = truncate_text(&current_line, font, size, max_width);
                    self.current_stream.text_at(x, y, font_tag, size, &truncated);
                } else {
                    self.current_stream.text_at(x, y, font_tag, size, &current_line);
                }
                y -= line_height;
            }
        }
    }

    fn emit_table_row(&mut self, _ui: &UIElement) {
        // Handled by emit_table
    }

    // ─── List ───────────────────────────────────────────────

    fn emit_list(&mut self, ui: &UIElement) {
        let ordered = ui.modifiers.iter().any(|m| m == "ordered");
        let font_size = self.current_font_size;
        let default_font = self.default_font.clone();

        for (idx, child) in ui.children.iter().enumerate() {
            if let Statement::UIElement(item) = child {
                let text = self.extract_text_content(item);
                if text.is_empty() {
                    continue;
                }

                // Build the marker string
                let marker = if ordered {
                    format!("{}. ", idx + 1)
                } else {
                    "- ".to_string()
                };

                // Prepend marker to text and render as one unit
                // This guarantees marker and text are on the same line
                let full_text = format!("{}{}", marker, text);

                // Use a hanging indent: first line starts at margin_left,
                // continuation lines start at margin_left + indent
                let indent = text_width(&marker, &default_font, font_size) + 2.0;

                let saved_left = self.margin_left;

                // Render first line at margin_left (includes marker)
                // Then subsequent wrapped lines at margin_left + indent
                self.render_list_item(&full_text, &default_font, font_size, (0.15, 0.15, 0.15), indent);

                self.margin_left = saved_left;
            }
        }
        self.cursor_y -= 6.0;
    }

    /// Render a list item with hanging indent (first line flush, wraps indented)
    fn render_list_item(&mut self, text: &str, font: &str, size: f64, color: (f64, f64, f64), indent: f64) {
        let font_tag = self.font_tag(font);
        let line_height = size * 1.5;
        let avail_width = self.content_width();
        let space_w = text_width(" ", font, size);

        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return;
        }

        let mut current_line = String::new();
        let mut current_width = 0.0;
        let mut is_first_line = true;

        for word in &words {
            let word_w = text_width(word, font, size);
            let max_w = if is_first_line { avail_width } else { avail_width - indent };

            if !current_line.is_empty() && current_width + space_w + word_w > max_w {
                // Flush line
                self.check_page_break(line_height);
                let x = if is_first_line { self.margin_left } else { self.margin_left + indent };
                self.current_stream.set_color(color.0, color.1, color.2);
                self.current_stream.text_at(x, self.cursor_y, &font_tag, size, &current_line);
                self.cursor_y -= line_height;
                self.mark_content();

                is_first_line = false;
                current_line = word.to_string();
                current_width = word_w;
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += space_w;
                }
                current_line.push_str(word);
                current_width += word_w;
            }
        }

        if !current_line.is_empty() {
            self.check_page_break(line_height);
            let x = if is_first_line { self.margin_left } else { self.margin_left + indent };
            self.current_stream.set_color(color.0, color.1, color.2);
            self.current_stream.text_at(x, self.cursor_y, &font_tag, size, &current_line);
            self.cursor_y -= line_height;
            self.mark_content();
        }
    }

    // ─── Code Block ─────────────────────────────────────────

    fn emit_code(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() {
            return;
        }

        let is_block = ui.modifiers.iter().any(|m| m == "block");

        if is_block {
            let code_font_size = 10.0;
            let code_line_height = code_font_size * 1.4;
            let padding_v = 12.0;
            let padding_h = 8.0;
            let lines: Vec<&str> = text.split('\n').collect();
            let avail_height = self.available_height();

            // Calculate total block height
            let total_block_height = lines.len() as f64 * code_line_height + padding_v * 2.0;

            // If the entire block fits, render it as one
            if total_block_height <= avail_height {
                self.render_code_block(&lines, code_font_size, code_line_height, padding_v, padding_h, total_block_height);
            } else {
                // Split across pages: render as many lines as fit per page
                let mut remaining_lines = &lines[..];
                while !remaining_lines.is_empty() {
                    let avail = self.available_height();
                    let max_lines_on_page = ((avail - padding_v * 2.0) / code_line_height).floor() as usize;
                    let max_lines_on_page = max_lines_on_page.max(1); // at least 1 line

                    let take = remaining_lines.len().min(max_lines_on_page);
                    let chunk = &remaining_lines[..take];
                    remaining_lines = &remaining_lines[take..];

                    let chunk_height = chunk.len() as f64 * code_line_height + padding_v * 2.0;
                    self.render_code_block(chunk, code_font_size, code_line_height, padding_v, padding_h, chunk_height);

                    if !remaining_lines.is_empty() {
                        self.finalize_page();
                        self.start_new_page();
                    }
                }
            }
        } else {
            // Inline code: just render in courier
            let font_tag = self.font_tag("Courier");
            let line_height = self.current_font_size * 1.5;
            self.check_page_break(line_height);
            self.current_stream.set_color(0.2, 0.2, 0.2);
            self.current_stream.text_at(
                self.margin_left, self.cursor_y,
                &font_tag, self.current_font_size, &text,
            );
            self.cursor_y -= line_height;
            self.mark_content();
        }
    }

    /// Render a code block (background + lines) at the current cursor position
    fn render_code_block(&mut self, lines: &[&str], font_size: f64, line_height: f64, padding_v: f64, padding_h: f64, block_height: f64) {
        let font_tag = self.font_tag("Courier");
        let max_text_width = self.content_width() - padding_h * 2.0;

        // Background rectangle
        self.current_stream.set_color(0.95, 0.95, 0.95);
        self.current_stream.rect(
            self.margin_left,
            self.cursor_y - block_height,
            self.content_width(),
            block_height,
        );
        self.current_stream.fill();

        // Border
        self.current_stream.set_stroke_color(0.85, 0.85, 0.85);
        self.current_stream.set_line_width(0.5);
        self.current_stream.rect(
            self.margin_left,
            self.cursor_y - block_height,
            self.content_width(),
            block_height,
        );
        self.current_stream.stroke();

        // Code text
        self.current_stream.set_color(0.2, 0.2, 0.2);
        let mut y = self.cursor_y - padding_v - font_size;
        for line in lines {
            // Truncate long lines instead of overflowing
            let display_line = truncate_text(line, "Courier", font_size, max_text_width);
            self.current_stream.text_at(
                self.margin_left + padding_h, y,
                &font_tag, font_size, &display_line,
            );
            y -= line_height;
        }

        self.cursor_y -= block_height + 8.0;
        self.mark_content();
    }

    // ─── Blockquote ─────────────────────────────────────────

    fn emit_blockquote(&mut self, ui: &UIElement) {
        let bar_width = 3.0;
        let bar_x_offset = 6.0;  // bar position from margin_left
        let text_indent = 18.0;  // text indented past the bar

        // Collect all text from children
        let mut texts = Vec::new();
        for child in &ui.children {
            if let Statement::UIElement(text_ui) = child {
                let t = self.extract_text_content(text_ui);
                if !t.is_empty() {
                    texts.push(t);
                }
            }
        }

        if texts.is_empty() {
            return;
        }

        let full_text = texts.join("\n");
        let font_name = self.default_font.clone();
        let size = self.current_font_size;
        let line_height = size * 1.5;

        // Pre-calculate total height to draw bar first (so it appears behind text)
        let avail = self.content_width() - text_indent;
        let line_count = self.count_wrapped_lines(&full_text, &font_name, size, avail);
        let text_block_height = line_count as f64 * line_height;

        // Ensure we have room for at least a few lines
        self.check_page_break(text_block_height.min(line_height * 2.0));

        let _start_page = self.pages.len();

        // Draw the bar FIRST (behind text) so it can't be occluded
        // Bar extends from current text-top to predicted text-bottom
        let bar_top = self.cursor_y + size * 0.75;     // align with text ascender
        let bar_bottom = self.cursor_y - text_block_height + line_height * 0.3;
        let bar_h = bar_top - bar_bottom;
        if bar_h > 0.0 {
            self.current_stream.set_color(0.72, 0.72, 0.72);
            self.current_stream.rounded_rect(
                self.margin_left + bar_x_offset,
                bar_bottom,
                bar_width,
                bar_h,
                1.5,
            );
            self.current_stream.fill();
        }

        // Render text with indent (on top of the bar)
        let saved_left = self.margin_left;
        self.margin_left += text_indent;
        self.render_wrapped_text(&full_text, &font_name, size, (0.30, 0.30, 0.30), "");
        self.margin_left = saved_left;

        // If text caused a page break, we can't retroactively fix the bar,
        // but at least the text renders correctly
        self.cursor_y -= 8.0;
    }

    // ─── Image ──────────────────────────────────────────────

    fn emit_image(&mut self, ui: &UIElement) {
        let mut _src = String::new();
        let mut width = 200.0f64;
        let mut height = 150.0f64;

        for arg in &ui.args {
            match arg {
                crate::parser::Arg::Named(name, value) => {
                    if let Expr::StringLiteral(s) = value {
                        match name.as_str() {
                            "src" => _src = s.clone(),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // Parse dimensions from style block
        if let Some(style) = &ui.style_block {
            for sp in &style.properties { let (prop, val) = (&sp.name, &sp.value);
                match prop.as_str() {
                    "width" => {
                        if let Expr::NumberLiteral(n) = val {
                            width = *n;
                        }
                    }
                    "height" => {
                        if let Expr::NumberLiteral(n) = val {
                            height = *n;
                        }
                    }
                    _ => {}
                }
            }
        }

        self.check_page_break(height + 8.0);

        // Draw placeholder rectangle
        self.current_stream.set_color(0.93, 0.93, 0.93);
        self.current_stream.rect(self.margin_left, self.cursor_y - height, width, height);
        self.current_stream.fill();

        self.current_stream.set_stroke_color(0.8, 0.8, 0.8);
        self.current_stream.set_line_width(1.0);
        self.current_stream.rect(self.margin_left, self.cursor_y - height, width, height);
        self.current_stream.stroke();

        // "Image" label in center
        let font_tag = self.font_tag(&self.default_font.clone());
        let label = "[Image]";
        let lw = text_width(label, &self.default_font, 10.0);
        self.current_stream.set_color(0.5, 0.5, 0.5);
        self.current_stream.text_at(
            self.margin_left + (width - lw) / 2.0,
            self.cursor_y - height / 2.0,
            &font_tag, 10.0, label,
        );

        self.cursor_y -= height + 8.0;
        self.mark_content();
    }

    // ─── Divider ────────────────────────────────────────────

    fn emit_divider(&mut self) {
        // Space above the line (below previous content)
        let gap = 14.0;
        self.cursor_y -= gap;
        self.check_page_break(gap + 2.0);

        // Draw the line at current cursor_y + half gap
        // This places the line in the vertical center of the total gap
        let line_y = self.cursor_y + gap / 2.0;
        self.current_stream.set_stroke_color(0.80, 0.80, 0.80);
        self.current_stream.set_line_width(0.5);
        self.current_stream.line(
            self.margin_left,
            line_y,
            self.page_width - self.margin_right,
            line_y,
        );

        // Space below the line (above next content)
        self.cursor_y -= gap;
        self.mark_content();
    }

    // ─── Card ───────────────────────────────────────────────

    fn emit_card(&mut self, ui: &UIElement) {
        let card_padding = 14.0;
        let card_margin = 6.0; // external spacing
        let card_radius = 4.0; // rounded corners
        let saved_left = self.margin_left;
        let saved_right = self.margin_right;

        // External margin
        self.cursor_y -= card_margin;

        self.margin_left += card_padding + card_margin;
        self.margin_right += card_padding + card_margin;

        // Record start Y
        let start_y = self.cursor_y;
        let start_page_count = self.pages.len();

        self.cursor_y -= card_padding; // top padding

        for child in &ui.children {
            self.emit_statement(child);
        }

        self.cursor_y -= card_padding; // bottom padding

        let end_y = self.cursor_y;

        self.margin_left = saved_left;
        self.margin_right = saved_right;

        // Only draw card border if content stayed on same page
        if self.pages.len() == start_page_count {
            let card_height = start_y - end_y;
            let card_x = saved_left + card_margin;
            let card_w = self.page_width - saved_left - saved_right - card_margin * 2.0;

            // Light background fill
            self.current_stream.set_color(0.99, 0.99, 0.99);
            self.current_stream.rounded_rect(card_x, end_y, card_w, card_height, card_radius);
            self.current_stream.fill();

            // Border
            self.current_stream.set_stroke_color(0.82, 0.82, 0.82);
            self.current_stream.set_line_width(0.75);
            self.current_stream.rounded_rect(card_x, end_y, card_w, card_height, card_radius);
            self.current_stream.stroke();
        }

        self.cursor_y -= card_margin; // external margin after
    }

    // ─── Badge / Tag ────────────────────────────────────────

    fn emit_inline_badge(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() {
            return;
        }

        let size = 9.0;
        let font_tag = self.font_tag("Helvetica-Bold");
        let tw = text_width(&text, "Helvetica-Bold", size);
        let pad = 6.0;
        let badge_h = size + pad * 2.0;

        self.check_page_break(badge_h);

        // Background pill (rounded)
        let color = self.modifier_color(&ui.modifiers);
        let badge_w = tw + pad * 2.0;
        let badge_radius = badge_h / 2.0; // fully rounded ends
        self.current_stream.set_color(color.0, color.1, color.2);
        self.current_stream.rounded_rect(
            self.margin_left,
            self.cursor_y - badge_h + pad,
            badge_w,
            badge_h,
            badge_radius,
        );
        self.current_stream.fill();

        // Text
        self.current_stream.set_color(1.0, 1.0, 1.0);
        self.current_stream.text_at(
            self.margin_left + pad,
            self.cursor_y - size,
            &font_tag, size, &text,
        );

        self.cursor_y -= badge_h + 4.0;
        self.mark_content();
    }

    // ─── Alert ──────────────────────────────────────────────

    fn emit_alert(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() {
            return;
        }

        let color = self.modifier_color(&ui.modifiers);
        let padding = 12.0;
        let font_name = self.default_font.clone();
        let font_size = self.current_font_size;
        let line_height = font_size * 1.5;
        let text_width_avail = self.content_width() - padding * 2.0 - 4.0; // minus left bar and padding

        // Calculate actual height based on wrapped text
        let line_count = self.count_wrapped_lines(&text, &font_name, font_size, text_width_avail);
        let text_height = line_count as f64 * line_height;
        let box_height = text_height + padding * 2.0;

        self.check_page_break(box_height.min(line_height * 3.0 + padding * 2.0));

        // Background
        self.current_stream.set_color(
            color.0 * 0.15 + 0.85,
            color.1 * 0.15 + 0.85,
            color.2 * 0.15 + 0.85,
        );
        self.current_stream.rect(
            self.margin_left, self.cursor_y - box_height,
            self.content_width(), box_height,
        );
        self.current_stream.fill();

        // Left accent bar
        self.current_stream.set_color(color.0, color.1, color.2);
        self.current_stream.rect(
            self.margin_left, self.cursor_y - box_height,
            3.0, box_height,
        );
        self.current_stream.fill();

        // Render wrapped text inside alert
        let font_tag = self.font_tag(&font_name);
        self.current_stream.set_color(0.15, 0.15, 0.15);

        let text_x = self.margin_left + padding + 4.0;
        let mut text_y = self.cursor_y - padding - font_size;
        let space_w = text_width(" ", &font_name, font_size);

        for raw_line in text.split('\n') {
            let words: Vec<&str> = raw_line.split_whitespace().collect();
            if words.is_empty() {
                text_y -= line_height;
                continue;
            }
            let mut current_line = String::new();
            let mut current_w = 0.0;
            for word in &words {
                let ww = text_width(word, &font_name, font_size);
                if !current_line.is_empty() && current_w + space_w + ww > text_width_avail {
                    self.current_stream.text_at(text_x, text_y, &font_tag, font_size, &current_line);
                    text_y -= line_height;
                    current_line = word.to_string();
                    current_w = ww;
                } else {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                        current_w += space_w;
                    }
                    current_line.push_str(word);
                    current_w += ww;
                }
            }
            if !current_line.is_empty() {
                self.current_stream.text_at(text_x, text_y, &font_tag, font_size, &current_line);
                text_y -= line_height;
            }
        }

        self.cursor_y -= box_height + 8.0;
        self.mark_content();
    }

    // ─── Progress Bar ───────────────────────────────────────

    fn emit_progress(&mut self, ui: &UIElement) {
        let mut value = 0.0f64;
        let mut max = 100.0f64;

        for arg in &ui.args {
            if let crate::parser::Arg::Named(name, val) = arg {
                if let Expr::NumberLiteral(n) = val {
                    match name.as_str() {
                        "value" => value = *n,
                        "max" => max = *n,
                        _ => {}
                    }
                }
            }
        }

        let bar_height = 8.0;
        let bar_radius = bar_height / 2.0; // fully rounded
        let fraction = (value / max).min(1.0).max(0.0);

        self.check_page_break(bar_height + 8.0);

        // Track (rounded)
        self.current_stream.set_color(0.9, 0.9, 0.9);
        self.current_stream.rounded_rect(self.margin_left, self.cursor_y - bar_height, self.content_width(), bar_height, bar_radius);
        self.current_stream.fill();

        // Fill (rounded, clip to track width)
        let fill_width = self.content_width() * fraction;
        if fill_width > 0.5 {
            let color = self.modifier_color(&ui.modifiers);
            self.current_stream.set_color(color.0, color.1, color.2);
            self.current_stream.rounded_rect(self.margin_left, self.cursor_y - bar_height, fill_width, bar_height, bar_radius);
        }
        self.current_stream.fill();

        self.cursor_y -= bar_height + 8.0;
        self.mark_content();
    }

    // ─── Text Rendering (with word wrap) ────────────────────

    fn render_wrapped_text(&mut self, text: &str, font: &str, size: f64, color: (f64, f64, f64), align: &str) {
        let font_tag = self.font_tag(font);
        let line_height = size * 1.5;
        let avail_width = self.content_width();
        let space_w = text_width(" ", font, size);

        let lines: Vec<&str> = text.split('\n').collect();

        for line_text in lines {
            let words: Vec<&str> = line_text.split_whitespace().collect();
            if words.is_empty() {
                self.cursor_y -= line_height * 0.5; // blank line
                continue;
            }

            let mut current_line = String::new();
            let mut current_width = 0.0;

            for word in &words {
                let word_w = text_width(word, font, size);

                if !current_line.is_empty() && current_width + space_w + word_w > avail_width {
                    // Flush line
                    self.check_page_break(line_height);
                    let x = self.text_x_for_align(&current_line, font, size, align);
                    self.current_stream.set_color(color.0, color.1, color.2);
                    self.current_stream.text_at(x, self.cursor_y, &font_tag, size, &current_line);
                    self.cursor_y -= line_height;
                    self.mark_content();
                    current_line = word.to_string();
                    current_width = word_w;
                } else {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                        current_width += space_w;
                    }
                    current_line.push_str(word);
                    current_width += word_w;
                }
            }

            if !current_line.is_empty() {
                self.check_page_break(line_height);
                let x = self.text_x_for_align(&current_line, font, size, align);
                self.current_stream.set_color(color.0, color.1, color.2);
                self.current_stream.text_at(x, self.cursor_y, &font_tag, size, &current_line);
                self.cursor_y -= line_height;
                self.mark_content();
            }
        }
    }

    fn text_x_for_align(&self, text: &str, font: &str, size: f64, align: &str) -> f64 {
        match align {
            "center" => {
                let tw = text_width(text, font, size);
                self.margin_left + (self.content_width() - tw) / 2.0
            }
            "right" => {
                let tw = text_width(text, font, size);
                self.page_width - self.margin_right - tw
            }
            _ => self.margin_left,
        }
    }

    // ─── Helpers ────────────────────────────────────────────

    fn extract_text_content(&self, ui: &UIElement) -> String {
        // First positional arg is usually the text
        for arg in &ui.args {
            if let crate::parser::Arg::Positional(expr) = arg {
                return self.expr_to_string(expr);
            }
        }
        String::new()
    }

    fn expr_to_string(&self, expr: &Expr) -> String {
        match expr {
            Expr::StringLiteral(s) => s.clone(),
            Expr::InterpolatedString(parts) => {
                let mut out = String::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => out.push_str(s),
                        StringPart::Expression(e) => out.push_str(&self.expr_to_string(e)),
                    }
                }
                out
            }
            Expr::NumberLiteral(n) => format!("{}", n),
            Expr::BoolLiteral(b) => format!("{}", b),
            Expr::Identifier(name) => format!("{{{}}}", name), // Keep as placeholder
            _ => String::new(),
        }
    }

    fn extract_text_style(&self, ui: &UIElement) -> (String, f64, (f64, f64, f64), String) {
        let mut font = self.default_font.clone();
        let mut size = self.default_font_size;
        let mut color = (0.0, 0.0, 0.0);
        let mut align = String::new();

        // Modifiers
        for modifier in &ui.modifiers {
            match modifier.as_str() {
                "bold" => font = format!("{}-Bold", font.split('-').next().unwrap_or("Helvetica")),
                "muted" => color = (0.4, 0.4, 0.4),
                "primary" => color = (0.098, 0.098, 0.647),
                "danger" => color = (0.86, 0.21, 0.27),
                "success" => color = (0.16, 0.65, 0.27),
                "warning" => color = (0.90, 0.56, 0.0),
                "info" => color = (0.0, 0.47, 0.84),
                "small" => size = self.default_font_size * 0.85,
                "large" => size = self.default_font_size * 1.25,
                "center" => align = "center".to_string(),
                "right" => align = "right".to_string(),
                _ => {}
            }
        }

        // Style block overrides
        if let Some(style) = &ui.style_block {
            for sp in &style.properties { let (prop, val) = (&sp.name, &sp.value);
                match prop.as_str() {
                    "font-size" => {
                        if let Expr::NumberLiteral(n) = val {
                            size = *n;
                        }
                    }
                    "font-family" | "font" => {
                        if let Expr::StringLiteral(s) = val {
                            font = s.clone();
                        }
                    }
                    "color" => {
                        if let Expr::StringLiteral(s) = val {
                            color = parse_color(s);
                        }
                    }
                    "text-align" => {
                        if let Expr::StringLiteral(s) = val {
                            align = s.clone();
                        }
                    }
                    _ => {}
                }
            }
        }

        (font, size, color, align)
    }

    fn modifier_color(&self, modifiers: &[String]) -> (f64, f64, f64) {
        for m in modifiers {
            match m.as_str() {
                "primary" => return (0.39, 0.39, 0.95),
                "success" => return (0.16, 0.65, 0.27),
                "danger" => return (0.86, 0.21, 0.27),
                "warning" => return (0.90, 0.56, 0.0),
                "info" => return (0.0, 0.47, 0.84),
                _ => {}
            }
        }
        (0.39, 0.39, 0.95) // default primary
    }

    fn get_spacer_size(&self, ui: &UIElement) -> f64 {
        for modifier in &ui.modifiers {
            match modifier.as_str() {
                "small" | "sm" => return 8.0,
                "medium" | "md" => return 16.0,
                "large" | "lg" => return 24.0,
                "xl" => return 32.0,
                _ => {}
            }
        }
        16.0 // default
    }

    // ─── PDF Serialization ──────────────────────────────────

    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut offsets: Vec<usize> = Vec::new();

        // Header
        out.extend_from_slice(b"%PDF-1.7\n");
        // Binary comment (recommended by spec to mark as binary)
        out.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n");

        let font_start = 3;
        let num_fonts = self.fonts.len();
        let resources_id = font_start + num_fonts;

        // Rebuild everything from scratch for clean serialization
        let mut final_objects: Vec<(usize, Vec<u8>)> = Vec::new(); // (id, data)

        // Object 1: Catalog
        let catalog = format!("<< /Type /Catalog /Pages 2 0 R >>");
        final_objects.push((1, catalog.into_bytes()));

        // Font objects (3..3+N-1)
        for (i, (_tag, base_font)) in self.fonts.iter().enumerate() {
            let font_id = font_start + i;
            let font_obj = format!(
                "<< /Type /Font /Subtype /Type1 /BaseFont /{} /Encoding /WinAnsiEncoding >>",
                base_font
            );
            final_objects.push((font_id, font_obj.into_bytes()));
        }

        // Resources object
        let mut font_entries = String::new();
        for (i, (tag, _)) in self.fonts.iter().enumerate() {
            font_entries.push_str(&format!("/{} {} 0 R ", tag, font_start + i));
        }
        let resources = format!("<< /Font << {} >> >>", font_entries);
        final_objects.push((resources_id, resources.into_bytes()));

        // Content streams and Page objects
        let mut new_page_ids: Vec<usize> = Vec::new();
        let mut next_id = resources_id + 1;

        // self.objects are: [content_stream, page, content_stream, page, ...]
        let mut i = 0;
        while i < self.objects.len() {
            if i + 1 < self.objects.len() {
                // Content stream
                let content_id = next_id;
                final_objects.push((content_id, self.objects[i].data.clone()));
                next_id += 1;

                // Page object — rewrite to reference correct content and resources
                let page_id = next_id;
                let page_data = format!(
                    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents {} 0 R /Resources {} 0 R >>",
                    fmt_f64(self.page_width),
                    fmt_f64(self.page_height),
                    content_id,
                    resources_id,
                );
                final_objects.push((page_id, page_data.into_bytes()));
                new_page_ids.push(page_id);
                next_id += 1;
            }
            i += 2;
        }

        // Object 2: Pages (now we know all page IDs)
        let kids: Vec<String> = new_page_ids.iter().map(|id| format!("{} 0 R", id)).collect();
        let pages = format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.join(" "),
            new_page_ids.len(),
        );
        final_objects.push((2, pages.into_bytes()));

        // Sort by ID
        final_objects.sort_by_key(|(id, _)| *id);

        // Write all objects and track offsets
        let total_objects = final_objects.last().map(|(id, _)| *id).unwrap_or(0);
        offsets.resize(total_objects + 1, 0);

        for (id, data) in &final_objects {
            offsets[*id] = out.len();
            let header = format!("{} 0 obj\n", id);
            out.extend_from_slice(header.as_bytes());
            out.extend_from_slice(data);
            out.extend_from_slice(b"\nendobj\n\n");
        }

        // Cross-reference table
        let xref_offset = out.len();
        out.extend_from_slice(b"xref\n");
        out.extend_from_slice(format!("0 {}\n", total_objects + 1).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for id in 1..=total_objects {
            let offset = offsets.get(id).copied().unwrap_or(0);
            out.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
        }

        // Trailer
        out.extend_from_slice(b"trailer\n");
        out.extend_from_slice(
            format!("<< /Size {} /Root 1 0 R >>\n", total_objects + 1).as_bytes(),
        );
        out.extend_from_slice(b"startxref\n");
        out.extend_from_slice(format!("{}\n", xref_offset).as_bytes());
        out.extend_from_slice(b"%%EOF\n");

        out
    }
}

// ─── Utility Functions ──────────────────────────────────────────────

fn fmt_f64(v: f64) -> String {
    if v == v.floor() {
        format!("{:.0}", v)
    } else {
        format!("{:.2}", v)
    }
}

/// Convert a Unicode character to its WinAnsiEncoding byte value
fn char_to_winansi(ch: char) -> u8 {
    match ch {
        '\u{FFFE}' => b'{',
        '\u{FFFF}' => b'}',
        '\u{2014}' => 0x97,       // em-dash —
        '\u{2013}' => 0x96,       // en-dash –
        '\u{2018}' => 0x91,       // left single quote '
        '\u{2019}' => 0x92,       // right single quote '
        '\u{201C}' => 0x93,       // left double quote "
        '\u{201D}' => 0x94,       // right double quote "
        '\u{2022}' => 0x95,       // bullet •
        '\u{2026}' => 0x85,       // ellipsis …
        '\u{2122}' => 0x99,       // trademark ™
        '\u{00A9}' => 0xA9,       // copyright ©
        '\u{00AE}' => 0xAE,       // registered ®
        '\u{00B0}' => 0xB0,       // degree °
        '\u{20AC}' => 0x80,       // euro €
        c if (c as u32) < 256 => c as u8,
        _ => b'?',
    }
}

/// Encode text as a PDF hex string (each byte as two hex digits)
/// This avoids UTF-8 encoding issues with WinAnsiEncoding
fn text_to_pdf_hex(text: &str) -> String {
    text.chars()
        .map(|ch| format!("{:02X}", char_to_winansi(ch)))
        .collect()
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
