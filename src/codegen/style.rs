//! Shared style-block parsing for the PDF and Slides backends.
//!
//! Both backends accept the same CSS-flavored properties on `Slide` (slides only),
//! `Container`, `Card`, etc. This module turns a `StyleBlock` AST into a flat
//! `StyleProps` value plus a list of unrecognized property names so each backend
//! can emit consistent diagnostics.

use crate::parser::{Expr, ast::StyleBlock};

pub type Color = (f64, f64, f64);

#[derive(Debug, Clone)]
pub enum Background {
    Color(Color),
    LinearGradient(LinearGradient),
}

#[derive(Debug, Clone)]
pub struct LinearGradient {
    /// Angle in degrees, CSS convention: 0deg = bottom-to-top, 90deg = left-to-right.
    pub angle_deg: f64,
    /// Start color (offset 0).
    pub start: Color,
    /// End color (offset 1).
    pub end: Color,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Sides {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Sides {
    pub fn uniform(v: f64) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }
    pub fn is_zero(&self) -> bool {
        self.top == 0.0 && self.right == 0.0 && self.bottom == 0.0 && self.left == 0.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Dimension {
    Points(f64),
    /// Fraction of the parent content area, in 0..1.
    Percent(f64),
}

impl Dimension {
    pub fn resolve(&self, parent: f64) -> f64 {
        match self {
            Dimension::Points(p) => *p,
            Dimension::Percent(f) => parent * f,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoxShadow {
    pub offset_x: f64,
    pub offset_y: f64,
    pub color: Color,
}

#[derive(Debug, Clone, Default)]
pub struct StyleProps {
    // Text
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub color: Option<Color>,
    pub text_align: Option<String>,

    // Background
    pub background: Option<Background>,

    // Box
    pub padding: Sides,
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,

    // Border
    pub border_color: Option<Color>,
    pub border_width: Option<f64>,
    pub border_radius: Option<f64>,

    // Shadow
    pub box_shadow: Option<BoxShadow>,

    // Property names we did not recognize (for diagnostics).
    pub unknown: Vec<String>,
}

impl StyleProps {
    pub fn from_block(block: &StyleBlock) -> Self {
        let mut s = StyleProps::default();
        for prop in &block.properties {
            apply(&mut s, &prop.name, &prop.value);
        }
        s
    }
}

fn apply(s: &mut StyleProps, name: &str, value: &Expr) {
    match name {
        // ─── Text ───────────────────────────────────────────
        "font-family" | "font" => {
            if let Some(v) = expr_str(value) { s.font_family = Some(v); }
        }
        "font-size" => {
            if let Some(v) = expr_dimension_pt(value) { s.font_size = Some(v); }
        }
        "color" => {
            if let Some(v) = expr_color(value) { s.color = Some(v); }
        }
        "text-align" => {
            if let Some(v) = expr_str(value) { s.text_align = Some(v); }
        }

        // ─── Background ─────────────────────────────────────
        "background" | "background-color" => {
            if let Some(v) = expr_str(value) {
                if let Some(g) = parse_linear_gradient(&v) {
                    s.background = Some(Background::LinearGradient(g));
                } else if let Some(c) = parse_color(&v) {
                    s.background = Some(Background::Color(c));
                }
            }
        }

        // ─── Padding ────────────────────────────────────────
        "padding" => {
            if let Some(v) = expr_dimension_pt(value) { s.padding = Sides::uniform(v); }
        }
        "padding-top"    => { if let Some(v) = expr_dimension_pt(value) { s.padding.top = v; } }
        "padding-right"  => { if let Some(v) = expr_dimension_pt(value) { s.padding.right = v; } }
        "padding-bottom" => { if let Some(v) = expr_dimension_pt(value) { s.padding.bottom = v; } }
        "padding-left"   => { if let Some(v) = expr_dimension_pt(value) { s.padding.left = v; } }

        // ─── Dimensions ─────────────────────────────────────
        "width"  => { s.width  = expr_dimension(value); }
        "height" => { s.height = expr_dimension(value); }

        // ─── Border ─────────────────────────────────────────
        "border" => {
            if let Some(v) = expr_str(value) {
                let (w, c) = parse_border_shorthand(&v);
                if let Some(w) = w { s.border_width = Some(w); }
                if let Some(c) = c { s.border_color = Some(c); }
                if s.border_width.is_none() && s.border_color.is_some() {
                    s.border_width = Some(1.0);
                }
            }
        }
        "border-color" => { if let Some(v) = expr_str(value).and_then(|v| parse_color(&v)) { s.border_color = Some(v); } }
        "border-width" => { if let Some(v) = expr_dimension_pt(value) { s.border_width = Some(v); } }
        "border-radius" => { if let Some(v) = expr_dimension_pt(value) { s.border_radius = Some(v); } }

        // ─── Shadow ─────────────────────────────────────────
        "box-shadow" => {
            if let Some(v) = expr_str(value) {
                if let Some(sh) = parse_box_shadow(&v) {
                    s.box_shadow = Some(sh);
                }
            }
        }

        // ─── Properties we silently accept but don't render ─
        // (Reserved for future support — don't spam warnings.)
        "transition" | "animation" | "transform" | "opacity" |
        "display" | "position" | "z-index" | "overflow" => {}

        _ => {
            s.unknown.push(name.to_string());
        }
    }
}

// ─── Expression coercers ────────────────────────────────────────────

fn expr_str(e: &Expr) -> Option<String> {
    if let Expr::StringLiteral(s) = e { Some(s.clone()) } else { None }
}

fn expr_color(e: &Expr) -> Option<Color> {
    expr_str(e).and_then(|s| parse_color(&s))
}

/// Parse a numeric dimension (always in points). Accepts string ("12pt", "16px", "10")
/// or a bare number.
fn expr_dimension_pt(e: &Expr) -> Option<f64> {
    match e {
        Expr::NumberLiteral(n) => Some(*n),
        Expr::StringLiteral(s) => parse_dimension_pt(s),
        _ => None,
    }
}

fn expr_dimension(e: &Expr) -> Option<Dimension> {
    match e {
        Expr::NumberLiteral(n) => Some(Dimension::Points(*n)),
        Expr::StringLiteral(s) => parse_dimension(s),
        _ => None,
    }
}

// ─── Value parsers ──────────────────────────────────────────────────

pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    let h = s.trim_start_matches('#');
    if h.len() == 6 {
        let r = u8::from_str_radix(&h[0..2], 16).ok()? as f64 / 255.0;
        let g = u8::from_str_radix(&h[2..4], 16).ok()? as f64 / 255.0;
        let b = u8::from_str_radix(&h[4..6], 16).ok()? as f64 / 255.0;
        return Some((r, g, b));
    }
    if h.len() == 3 {
        let r = u8::from_str_radix(&h[0..1].repeat(2), 16).ok()? as f64 / 255.0;
        let g = u8::from_str_radix(&h[1..2].repeat(2), 16).ok()? as f64 / 255.0;
        let b = u8::from_str_radix(&h[2..3].repeat(2), 16).ok()? as f64 / 255.0;
        return Some((r, g, b));
    }
    // Named color shortcuts
    match s {
        "black"       => Some((0.0, 0.0, 0.0)),
        "white"       => Some((1.0, 1.0, 1.0)),
        "transparent" => None,
        _ => None,
    }
}

/// Parse "12pt" / "16px" / "10" → points. 1px == 1pt for slides/PDF (point units).
fn parse_dimension_pt(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(n) = s.strip_suffix("pt") { return n.trim().parse().ok(); }
    if let Some(n) = s.strip_suffix("px") { return n.trim().parse().ok(); }
    if let Some(n) = s.strip_suffix("em") { return n.trim().parse::<f64>().ok().map(|v| v * 12.0); }
    s.parse().ok()
}

/// Parse "10pt" / "10px" / "10" / "50%" → Dimension.
fn parse_dimension(s: &str) -> Option<Dimension> {
    let s = s.trim();
    if let Some(n) = s.strip_suffix('%') {
        return n.trim().parse::<f64>().ok().map(|v| Dimension::Percent(v / 100.0));
    }
    parse_dimension_pt(s).map(Dimension::Points)
}

/// Parse `"1pt solid #color"` / `"#color"` / `"2pt #color"` (CSS shorthand, simplified).
fn parse_border_shorthand(s: &str) -> (Option<f64>, Option<Color>) {
    let mut width = None;
    let mut color = None;
    for token in s.split_whitespace() {
        if token.starts_with('#') {
            color = parse_color(token);
        } else if let Some(w) = parse_dimension_pt(token) {
            width = Some(w);
        }
        // "solid", "dashed", etc. — accepted but ignored (only solid is rendered)
    }
    (width, color)
}

/// Parse `"2pt 4pt #color"` → BoxShadow (offset_x, offset_y, color). Blur is ignored.
fn parse_box_shadow(s: &str) -> Option<BoxShadow> {
    let mut nums = Vec::new();
    let mut color = None;
    for token in s.split_whitespace() {
        if token.starts_with('#') {
            color = parse_color(token);
        } else if let Some(n) = parse_dimension_pt(token) {
            nums.push(n);
        }
    }
    let offset_x = *nums.first()?;
    let offset_y = *nums.get(1)?;
    let color = color.unwrap_or((0.0, 0.0, 0.0));
    Some(BoxShadow { offset_x, offset_y, color })
}

/// Parse `"linear-gradient(to bottom, #a, #b)"` or `"linear-gradient(45deg, #a, #b)"`.
/// Two color stops only (start + end).
fn parse_linear_gradient(s: &str) -> Option<LinearGradient> {
    let s = s.trim();
    let inner = s.strip_prefix("linear-gradient(")?.strip_suffix(')')?;
    let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
    if parts.len() < 2 { return None; }

    // Direction: optional first part if it doesn't start with '#'
    let (angle, color_parts) = if !parts[0].starts_with('#') {
        let a = parse_gradient_direction(parts[0])?;
        (a, &parts[1..])
    } else {
        (180.0, &parts[..]) // CSS default: top-to-bottom
    };

    if color_parts.len() < 2 { return None; }
    let start = parse_color(color_parts[0])?;
    let end = parse_color(color_parts[color_parts.len() - 1])?;
    Some(LinearGradient { angle_deg: angle, start, end })
}

fn parse_gradient_direction(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(deg) = s.strip_suffix("deg") {
        return deg.trim().parse().ok();
    }
    match s {
        "to top"          => Some(0.0),
        "to right"        => Some(90.0),
        "to bottom"       => Some(180.0),
        "to left"         => Some(270.0),
        "to top right"    => Some(45.0),
        "to bottom right" => Some(135.0),
        "to bottom left"  => Some(225.0),
        "to top left"     => Some(315.0),
        _ => None,
    }
}

// ─── Backend helpers ────────────────────────────────────────────────

/// Perceived luminance, 0..1. Used for chrome auto-flip.
pub fn luminance(c: Color) -> f64 {
    0.299 * c.0 + 0.587 * c.1 + 0.114 * c.2
}

/// Convert a CSS gradient angle to PDF axial-shading endpoints inside a box.
/// Returns ((x0, y0), (x1, y1)) in PDF coordinates (origin bottom-left).
///
/// CSS angle convention: 0deg = bottom-to-top, 90deg = left-to-right (clockwise).
/// PDF y-axis points up, so we mirror the CSS y-direction.
pub fn gradient_endpoints(
    angle_deg: f64,
    x: f64, y: f64, w: f64, h: f64,
) -> ((f64, f64), (f64, f64)) {
    let rad = angle_deg.to_radians();
    // CSS direction vector (in screen coords where y goes down): (sin, -cos).
    // Flip y to PDF: (sin, cos).
    let dx = rad.sin();
    let dy = rad.cos();

    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    // Project box corners onto direction; pick min/max.
    let corners = [
        (x, y), (x + w, y), (x, y + h), (x + w, y + h),
    ];
    let mut min_t = f64::INFINITY;
    let mut max_t = f64::NEG_INFINITY;
    for (px, py) in corners {
        let t = (px - cx) * dx + (py - cy) * dy;
        if t < min_t { min_t = t; }
        if t > max_t { max_t = t; }
    }
    (
        (cx + dx * min_t, cy + dy * min_t),
        (cx + dx * max_t, cy + dy * max_t),
    )
}
