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

const COURIER_WIDTH: u16 = 600; // Monospace: all glyphs are 600 units

fn char_width(ch: char, font: &str) -> u16 {
    let code = ch as u32;
    if code < 32 || code > 126 {
        return 500; // default for unknown
    }
    let idx = (code - 32) as usize;
    if font.contains("Courier") {
        COURIER_WIDTH
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

    fn check_page_break(&mut self, needed: f64) {
        if self.cursor_y - needed < self.margin_bottom && !self.in_header_footer {
            self.finalize_page();
            self.start_new_page();
        }
    }

    fn start_new_page(&mut self) {
        self.cursor_y = self.page_height - self.margin_top;
        self.current_stream = ContentStream::new();

        // Emit header on new page
        if !self.header_stmts.is_empty() && !self.in_header_footer {
            self.in_header_footer = true;
            let saved_y = self.cursor_y;
            self.cursor_y = self.page_height - self.margin_top / 2.0;
            let stmts = self.header_stmts.clone();
            for stmt in &stmts {
                self.emit_statement(stmt);
            }
            self.cursor_y = saved_y;
            self.in_header_footer = false;
        }
    }

    fn finalize_page(&mut self) {
        // Emit footer before finalizing
        if !self.footer_stmts.is_empty() && !self.in_header_footer {
            self.in_header_footer = true;
            let stmts = self.footer_stmts.clone();
            let saved_y = self.cursor_y;
            self.cursor_y = self.margin_bottom / 2.0;
            for stmt in &stmts {
                self.emit_statement(stmt);
            }
            self.cursor_y = saved_y;
            self.in_header_footer = false;
        }

        let stream_data = std::mem::replace(&mut self.current_stream, ContentStream::new());
        let content_id = self.add_stream_object(stream_data.bytes());

        // Build font resource references
        let font_refs: Vec<String> = self.fonts.iter().map(|(tag, _)| {
            // Font object IDs will be assigned during serialize
            format!("/{} << >>", tag) // placeholder
        }).collect();
        let _ = font_refs; // We'll build resources properly in serialize

        let page_id = self.objects.len() + 1; // will be assigned next
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
                    for item in items {
                        // Simple: treat each item as text
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
        // Add some spacing before sections
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
        // Parse style overrides
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
        let col_width = self.content_width() / num_cols as f64;
        let row_height = self.current_font_size * 2.0;
        let total_height = row_height * rows.len() as f64;

        self.check_page_break(total_height.min(row_height * 3.0));

        let table_x = self.margin_left;
        let table_y = self.cursor_y;

        // Draw table grid and content
        self.current_stream.set_line_width(0.5);
        self.current_stream.set_stroke_color(0.6, 0.6, 0.6);

        for (row_idx, row) in rows.iter().enumerate() {
            let y = table_y - (row_idx as f64 * row_height);

            if y - row_height < self.margin_bottom {
                // Need page break mid-table
                self.finalize_page();
                self.start_new_page();
                // Continue from top
                // (simplified: just continue at current cursor_y)
            }

            let cell_y = y - row_height;

            // Background for header rows
            if is_header[row_idx] {
                self.current_stream.set_color(0.93, 0.93, 0.93);
                self.current_stream.rect(table_x, cell_y, self.content_width(), row_height);
                self.current_stream.fill();
            }

            // Row border
            self.current_stream.set_stroke_color(0.6, 0.6, 0.6);
            self.current_stream.rect(table_x, cell_y, self.content_width(), row_height);
            self.current_stream.stroke();

            // Cell content
            let font_name = if is_header[row_idx] { "Helvetica-Bold" } else { &self.default_font };
            let font_tag = self.font_tag(font_name);

            for (col_idx, cell_text) in row.iter().enumerate() {
                let cell_x = table_x + col_idx as f64 * col_width;

                // Vertical cell border
                if col_idx > 0 {
                    self.current_stream.line(cell_x, y, cell_x, cell_y);
                }

                // Text in cell
                let text_x = cell_x + 4.0;
                let text_y = cell_y + (row_height - self.current_font_size) / 2.0;

                self.current_stream.set_color(0.0, 0.0, 0.0);
                self.current_stream.text_at(text_x, text_y, &font_tag, self.current_font_size, cell_text);
            }
        }

        self.cursor_y = table_y - (rows.len() as f64 * row_height) - 8.0;
    }

    fn emit_table_row(&mut self, _ui: &UIElement) {
        // Handled by emit_table
    }

    // ─── List ───────────────────────────────────────────────

    fn emit_list(&mut self, ui: &UIElement) {
        let font_tag = self.font_tag(&self.default_font.clone());

        for (idx, child) in ui.children.iter().enumerate() {
            if let Statement::UIElement(item) = child {
                let text = self.extract_text_content(item);
                if text.is_empty() {
                    continue;
                }

                let line_height = self.current_font_size * 1.5;
                self.check_page_break(line_height);

                // Check for "ordered" modifier
                let ordered = ui.modifiers.iter().any(|m| m == "ordered");
                let bullet = if ordered {
                    format!("{}.", idx + 1)
                } else {
                    "\u{2022}".to_string() // bullet character
                };

                let bullet_x = self.margin_left;
                let text_x = self.margin_left + 20.0;

                self.current_stream.set_color(0.0, 0.0, 0.0);
                self.current_stream.text_at(bullet_x, self.cursor_y, &font_tag, self.current_font_size, &bullet);

                // Render text with indent
                let saved_left = self.margin_left;
                self.margin_left += 20.0;
                self.render_wrapped_text(&text, &self.default_font.clone(), self.current_font_size, (0.0, 0.0, 0.0), "");
                self.margin_left = saved_left;
            }
        }
        self.cursor_y -= 4.0;
    }

    // ─── Code Block ─────────────────────────────────────────

    fn emit_code(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() {
            return;
        }

        let is_block = ui.modifiers.iter().any(|m| m == "block");

        if is_block {
            let lines: Vec<&str> = text.split('\n').collect();
            let line_height = 12.0 * 1.4;
            let block_height = lines.len() as f64 * line_height + 16.0;

            self.check_page_break(block_height);

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
            let font_tag = self.font_tag("Courier");
            self.current_stream.set_color(0.2, 0.2, 0.2);
            let mut y = self.cursor_y - 12.0;
            for line in &lines {
                self.current_stream.text_at(
                    self.margin_left + 8.0, y,
                    &font_tag, 10.0, line,
                );
                y -= line_height;
            }

            self.cursor_y -= block_height + 8.0;
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
        }
    }

    // ─── Blockquote ─────────────────────────────────────────

    fn emit_blockquote(&mut self, ui: &UIElement) {
        let indent = 24.0;
        let bar_width = 3.0;

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
        let font_name = "Times-Roman";
        let font_tag = self.font_tag(font_name);
        let size = self.current_font_size;
        let line_height = size * 1.6;

        // Estimate block height for the left bar
        let words: Vec<&str> = full_text.split_whitespace().collect();
        let avail = self.content_width() - indent;
        let mut line_count = 1usize;
        let mut line_w = 0.0;
        for word in &words {
            let ww = text_width(word, font_name, size) + text_width(" ", font_name, size);
            if line_w + ww > avail && line_w > 0.0 {
                line_count += 1;
                line_w = ww;
            } else {
                line_w += ww;
            }
        }
        let block_height = line_count as f64 * line_height + 8.0;

        self.check_page_break(block_height);

        // Left bar
        self.current_stream.set_color(0.7, 0.7, 0.7);
        self.current_stream.rect(
            self.margin_left + 4.0,
            self.cursor_y - block_height,
            bar_width,
            block_height,
        );
        self.current_stream.fill();

        // Text
        let saved_left = self.margin_left;
        self.margin_left += indent;
        self.current_stream.set_color(0.3, 0.3, 0.3);
        self.render_wrapped_text(&full_text, font_name, size, (0.3, 0.3, 0.3), "");
        self.margin_left = saved_left;

        self.cursor_y -= 4.0;
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

        // Draw placeholder rectangle (since we can't easily embed images without file I/O at this stage)
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
    }

    // ─── Divider ────────────────────────────────────────────

    fn emit_divider(&mut self) {
        self.cursor_y -= 8.0;
        self.check_page_break(2.0);
        self.current_stream.set_stroke_color(0.75, 0.75, 0.75);
        self.current_stream.set_line_width(0.5);
        self.current_stream.line(
            self.margin_left,
            self.cursor_y,
            self.page_width - self.margin_right,
            self.cursor_y,
        );
        self.cursor_y -= 8.0;
    }

    // ─── Card ───────────────────────────────────────────────

    fn emit_card(&mut self, ui: &UIElement) {
        // Estimate card content height (rough)
        let card_padding = 12.0;
        let saved_left = self.margin_left;
        let saved_right = self.margin_right;

        self.margin_left += card_padding;
        self.margin_right += card_padding;

        // Light border
        let start_y = self.cursor_y;

        for child in &ui.children {
            self.emit_statement(child);
        }

        let end_y = self.cursor_y;
        let card_height = start_y - end_y + card_padding * 2.0;

        // Draw card border retroactively (simplified: draw at current position)
        self.current_stream.set_stroke_color(0.85, 0.85, 0.85);
        self.current_stream.set_line_width(1.0);
        self.current_stream.rect(
            saved_left,
            end_y - card_padding,
            self.page_width - saved_left - saved_right,
            card_height,
        );
        self.current_stream.stroke();

        self.margin_left = saved_left;
        self.margin_right = saved_right;
        self.cursor_y -= card_padding;
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

        // Background pill
        let color = self.modifier_color(&ui.modifiers);
        self.current_stream.set_color(color.0, color.1, color.2);
        self.current_stream.rect(
            self.margin_left,
            self.cursor_y - badge_h + pad,
            tw + pad * 2.0,
            badge_h,
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
    }

    // ─── Alert ──────────────────────────────────────────────

    fn emit_alert(&mut self, ui: &UIElement) {
        let text = self.extract_text_content(ui);
        if text.is_empty() {
            return;
        }

        let color = self.modifier_color(&ui.modifiers);
        let line_height = self.current_font_size * 1.5;
        let padding = 12.0;
        let box_height = line_height + padding * 2.0;

        self.check_page_break(box_height);

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

        // Text
        let font_tag = self.font_tag(&self.default_font.clone());
        self.current_stream.set_color(0.15, 0.15, 0.15);
        self.current_stream.text_at(
            self.margin_left + padding + 4.0,
            self.cursor_y - padding - self.current_font_size,
            &font_tag, self.current_font_size, &text,
        );

        self.cursor_y -= box_height + 8.0;
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
        let fraction = (value / max).min(1.0).max(0.0);

        self.check_page_break(bar_height + 8.0);

        // Track
        self.current_stream.set_color(0.9, 0.9, 0.9);
        self.current_stream.rect(self.margin_left, self.cursor_y - bar_height, self.content_width(), bar_height);
        self.current_stream.fill();

        // Fill
        let color = self.modifier_color(&ui.modifiers);
        self.current_stream.set_color(color.0, color.1, color.2);
        self.current_stream.rect(self.margin_left, self.cursor_y - bar_height, self.content_width() * fraction, bar_height);
        self.current_stream.fill();

        self.cursor_y -= bar_height + 8.0;
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

        // We need to build objects in a specific order:
        // 1: Catalog
        // 2: Pages
        // 3+: Font objects
        // Then: content streams and page objects (already in self.objects)
        // Then: Resources

        // Assign final IDs:
        // obj 1 = Catalog
        // obj 2 = Pages
        // obj 3..3+N = Font objects
        // obj 3+N+1 = Resources
        // obj 3+N+2.. = existing objects (content streams, pages)

        let font_start = 3;
        let num_fonts = self.fonts.len();
        let resources_id = font_start + num_fonts;
        let obj_offset = resources_id; // existing objects get IDs starting after resources

        // Remap page IDs and content IDs
        // self.objects contains interleaved content streams and page objects
        // self.pages contains the original IDs of page objects

        // Rebuild everything from scratch for clean serialization
        let mut final_objects: Vec<(usize, Vec<u8>)> = Vec::new(); // (id, data)

        // Object 1: Catalog
        let catalog = format!("<< /Type /Catalog /Pages 2 0 R >>");
        final_objects.push((1, catalog.into_bytes()));

        // Font objects (3..3+N-1)
        for (i, (tag, base_font)) in self.fonts.iter().enumerate() {
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
        // Each page references a content stream
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
