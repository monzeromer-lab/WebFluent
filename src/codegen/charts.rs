//! Charts and QR codes, drawn as SVG at build time: the same picture in a
//! PDF, a slide, a rendered template and a page.

use serde_json::Value;

/// What a `Chart` draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bar,
    Line,
    Area,
    Pie,
    Donut,
}

impl Kind {
    pub fn parse(s: &str) -> Kind {
        match s.trim_start_matches('.') {
            "line" => Kind::Line,
            "area" => Kind::Area,
            "pie" => Kind::Pie,
            "donut" => Kind::Donut,
            _ => Kind::Bar,
        }
    }
}

/// Everything a chart is told.
#[derive(Debug, Clone)]
pub struct Chart {
    pub kind: Kind,
    /// The rows: each a map (`{ month: "Jan", p95: 41 }`) or a bare number.
    pub data: Vec<Value>,
    /// The key of each row's label.
    pub x: Option<String>,
    /// The keys of each row's values: one per series.
    pub y: Vec<String>,
    pub colors: Vec<String>,
    pub width: f64,
    pub height: f64,
    /// Axis labels and legend text.
    pub ink: String,
    /// Grid lines.
    pub grid: String,
    pub legend: bool,
    pub stacked: bool,
    /// The value labels on bars and slices.
    pub labels: bool,
    /// A unit after each value: `ms`, `%`.
    pub unit: String,
}

/// A chart from its arguments, as values: what the template engine and the
/// web build both read.
pub fn chart_from(named: &[(String, Value)], modifiers: &[String]) -> Chart {
    let mut c = Chart::default();
    let text = |v: &Value| label(v).trim_start_matches('.').to_string();
    let truthy = |v: &Value| match v {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::String(s) => !s.is_empty(),
        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        _ => true,
    };
    for (key, v) in named {
        match key.as_str() {
            "kind" | "type" => c.kind = Kind::parse(&text(v)),
            "data" => c.data = v.as_array().cloned().unwrap_or_default(),
            "x" => c.x = Some(text(v)),
            "y" => {
                c.y = match v {
                    Value::Array(a) => a.iter().map(label).collect(),
                    other => vec![text(other)],
                }
            }
            "colors" => {
                if let Value::Array(a) = v {
                    c.colors = a.iter().map(label).collect();
                }
            }
            "width" => c.width = v.as_f64().unwrap_or(c.width),
            "height" => c.height = v.as_f64().unwrap_or(c.height),
            "ink" => c.ink = text(v),
            "grid" => c.grid = text(v),
            "legend" => c.legend = truthy(v),
            "stacked" => c.stacked = truthy(v),
            "labels" => c.labels = truthy(v),
            "unit" => c.unit = text(v),
            _ => {}
        }
    }
    for m in modifiers {
        match m.as_str() {
            "bar" | "line" | "area" | "pie" | "donut" => c.kind = Kind::parse(m),
            "stacked" => c.stacked = true,
            "labels" => c.labels = true,
            "legend" => c.legend = true,
            _ => {}
        }
    }
    c
}

/// A `Chart` or a `QrCode`'s SVG from its arguments.
pub fn graphic(
    name: &str,
    positional: Option<&Value>,
    named: &[(String, Value)],
    modifiers: &[String],
) -> Option<String> {
    match name {
        "Chart" => Some(chart_from(named, modifiers).svg()),
        "QrCode" => {
            let get = |k: &str| named.iter().find(|(n, _)| n == k).map(|(_, v)| label(v));
            let value = positional.map(label).or_else(|| get("value"))?;
            qr_svg(
                &value,
                &get("color").unwrap_or_else(|| "#000000".into()),
                &get("background").unwrap_or_else(|| "#ffffff".into()),
            )
        }
        _ => None,
    }
}

/// The palette a chart uses when it is given none.
pub const PALETTE: &[&str] = &[
    "#3B82F6", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#06B6D4", "#EC4899", "#84CC16",
];

impl Default for Chart {
    fn default() -> Self {
        Chart {
            kind: Kind::Bar,
            data: Vec::new(),
            x: None,
            y: Vec::new(),
            colors: Vec::new(),
            width: 600.0,
            height: 260.0,
            ink: "#64748B".into(),
            grid: "#E2E8F0".into(),
            legend: true,
            stacked: false,
            labels: false,
            unit: String::new(),
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn num(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.trim().trim_end_matches('%').parse().unwrap_or(0.0),
        Value::Bool(b) => *b as i32 as f64,
        _ => 0.0,
    }
}

fn label(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.fract() == 0.0 {
                format!("{}", f as i64)
            } else {
                format!("{f}")
            }
        }
        other => other.to_string(),
    }
}

/// A short form of a number for an axis: `1.2k`, `3M`, `0.5`.
fn short(v: f64) -> String {
    let a = v.abs();
    if a >= 1_000_000.0 {
        format!("{}M", trim(v / 1_000_000.0))
    } else if a >= 10_000.0 {
        format!("{}k", trim(v / 1000.0))
    } else {
        trim(v)
    }
}

fn trim(v: f64) -> String {
    let s = format!("{:.1}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// A round step for an axis from 0 to `max`, about five ticks.
fn nice_step(max: f64) -> f64 {
    if max <= 0.0 {
        return 1.0;
    }
    let rough = max / 5.0;
    let mag = 10f64.powf(rough.log10().floor());
    let r = rough / mag;
    let nice = if r <= 1.0 {
        1.0
    } else if r <= 2.0 {
        2.0
    } else if r <= 2.5 {
        2.5
    } else if r <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * mag
}

impl Chart {
    fn color(&self, i: usize) -> String {
        self.colors
            .get(i % self.colors.len().max(1))
            .cloned()
            .unwrap_or_else(|| PALETTE[i % PALETTE.len()].to_string())
    }

    fn series(&self) -> Vec<String> {
        if self.y.is_empty() {
            vec![String::new()]
        } else {
            self.y.clone()
        }
    }

    fn value(&self, row: &Value, key: &str) -> f64 {
        if key.is_empty() {
            return match row {
                Value::Object(m) => m.values().find(|v| v.is_number()).map(num).unwrap_or(0.0),
                other => num(other),
            };
        }
        row.get(key).map(num).unwrap_or(0.0)
    }

    fn row_label(&self, row: &Value, i: usize) -> String {
        match (&self.x, row) {
            (Some(k), Value::Object(m)) => m.get(k).map(label).unwrap_or_default(),
            (None, Value::Object(m)) => m
                .values()
                .find(|v| v.is_string())
                .map(label)
                .unwrap_or_else(|| format!("{}", i + 1)),
            _ => format!("{}", i + 1),
        }
    }

    /// The chart as an SVG document.
    pub fn svg(&self) -> String {
        let (w, h) = (self.width, self.height);
        let mut out = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" class="wf-chart" viewBox="0 0 {w} {h}" width="{w}" height="{h}" font-family="Liberation Sans, Helvetica, Arial, sans-serif" font-size="11">"#
        );
        match self.kind {
            Kind::Pie | Kind::Donut => self.pie(&mut out),
            _ => self.axes(&mut out),
        }
        out.push_str("</svg>");
        out
    }

    fn legend(&self, out: &mut String, names: &[String], y: f64) {
        if !self.legend || names.len() < 2 {
            return;
        }
        let mut x = 0.0;
        for (i, name) in names.iter().enumerate() {
            out.push_str(&format!(
                r#"<rect x="{x}" y="{}" width="10" height="10" rx="2" fill="{}"/><text x="{}" y="{}" fill="{}">{}</text>"#,
                y - 9.0,
                self.color(i),
                x + 14.0,
                y,
                self.ink,
                esc(name)
            ));
            x += 22.0 + name.chars().count() as f64 * 6.2;
        }
    }

    fn axes(&self, out: &mut String) {
        let (w, h) = (self.width, self.height);
        let series = self.series();
        let n = self.data.len();
        let legend_h = if self.legend && series.len() > 1 {
            22.0
        } else {
            0.0
        };
        let values: Vec<Vec<f64>> = self
            .data
            .iter()
            .map(|r| series.iter().map(|k| self.value(r, k)).collect())
            .collect();
        let max = values
            .iter()
            .map(|vs| {
                if self.stacked {
                    vs.iter().sum()
                } else {
                    vs.iter().cloned().fold(0.0, f64::max)
                }
            })
            .fold(0.0f64, f64::max);
        let step = nice_step(max);
        let top = (max / step).ceil().max(1.0) * step;
        let axis_w = (0..=((top / step) as usize))
            .map(|t| short(t as f64 * step).len() + self.unit.len())
            .max()
            .unwrap_or(1) as f64
            * 6.5
            + 8.0;
        let (left, right, plot_top, bottom) = (axis_w, w - 6.0, legend_h + 8.0, h - 22.0);
        let (pw, ph) = (right - left, bottom - plot_top);
        self.legend(out, &series, 12.0);
        // The grid and the value axis.
        let ticks = (top / step).round() as usize;
        for t in 0..=ticks {
            let v = t as f64 * step;
            let y = bottom - v / top * ph;
            out.push_str(&format!(r#"<line x1="{left}" y1="{y:.2}" x2="{right}" y2="{y:.2}" stroke="{}" stroke-width="1"/>"#, self.grid));
            out.push_str(&format!(
                r#"<text x="{}" y="{:.2}" text-anchor="end" fill="{}">{}{}</text>"#,
                left - 6.0,
                y + 3.5,
                self.ink,
                short(v),
                esc(&self.unit)
            ));
        }
        if n == 0 {
            return;
        }
        let band = pw / n as f64;
        // The labels along the bottom, thinned to fit.
        let every = ((n as f64 * 34.0) / pw).ceil().max(1.0) as usize;
        for (i, row) in self.data.iter().enumerate() {
            if i % every != 0 {
                continue;
            }
            let x = left + band * (i as f64 + 0.5);
            out.push_str(&format!(
                r#"<text x="{x:.2}" y="{}" text-anchor="middle" fill="{}">{}</text>"#,
                h - 6.0,
                self.ink,
                esc(&self.row_label(row, i))
            ));
        }
        match self.kind {
            Kind::Bar => {
                let groups = if self.stacked { 1 } else { series.len() };
                let gap = band * 0.22;
                let bw = (band - gap) / groups as f64;
                for (i, vs) in values.iter().enumerate() {
                    let mut stack = 0.0;
                    for (s, v) in vs.iter().enumerate() {
                        let x = left
                            + band * i as f64
                            + gap / 2.0
                            + if self.stacked { 0.0 } else { bw * s as f64 };
                        let base = if self.stacked { stack } else { 0.0 };
                        let y0 = bottom - (base + v) / top * ph;
                        let bh = v / top * ph;
                        out.push_str(&format!(
                            r#"<rect x="{x:.2}" y="{y0:.2}" width="{:.2}" height="{bh:.2}" rx="{:.2}" fill="{}"/>"#,
                            (bw - 1.0).max(1.0),
                            (bw * 0.12).min(3.0),
                            self.color(s)
                        ));
                        if self.labels && bh > 12.0 {
                            out.push_str(&format!(
                                r#"<text x="{:.2}" y="{:.2}" text-anchor="middle" fill="{}">{}</text>"#,
                                x + bw / 2.0,
                                y0 - 4.0,
                                self.ink,
                                short(*v)
                            ));
                        }
                        stack += v;
                    }
                }
            }
            Kind::Line | Kind::Area => {
                for s in 0..series.len() {
                    let points: Vec<(f64, f64)> = values
                        .iter()
                        .enumerate()
                        .map(|(i, vs)| (left + band * (i as f64 + 0.5), bottom - vs[s] / top * ph))
                        .collect();
                    let path: String = points
                        .iter()
                        .enumerate()
                        .map(|(i, (x, y))| {
                            format!("{}{x:.2} {y:.2}", if i == 0 { "M" } else { " L" })
                        })
                        .collect();
                    let color = self.color(s);
                    if self.kind == Kind::Area {
                        let (fx, _) = points[0];
                        let (lx, _) = points[points.len() - 1];
                        out.push_str(&format!(
                            r#"<path d="{path} L{lx:.2} {bottom} L{fx:.2} {bottom} Z" fill="{color}" fill-opacity="0.18"/>"#
                        ));
                    }
                    out.push_str(&format!(
                        r#"<path d="{path}" fill="none" stroke="{color}" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>"#
                    ));
                    if points.len() <= 24 {
                        for (x, y) in &points {
                            out.push_str(&format!(
                                r#"<circle cx="{x:.2}" cy="{y:.2}" r="2.5" fill="{color}"/>"#
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn pie(&self, out: &mut String) {
        let (w, h) = (self.width, self.height);
        let key = self.series().into_iter().next().unwrap_or_default();
        let values: Vec<f64> = self
            .data
            .iter()
            .map(|r| self.value(r, &key).max(0.0))
            .collect();
        let total: f64 = values.iter().sum();
        let legend_w = if self.legend {
            (w * 0.42).min(220.0)
        } else {
            0.0
        };
        let r = ((h - 8.0) / 2.0).min((w - legend_w - 8.0) / 2.0).max(4.0);
        let (cx, cy) = (r + 4.0, h / 2.0);
        let inner = if self.kind == Kind::Donut {
            r * 0.58
        } else {
            0.0
        };
        let mut angle = -std::f64::consts::FRAC_PI_2;
        for (i, v) in values.iter().enumerate() {
            if total <= 0.0 {
                break;
            }
            let sweep = v / total * std::f64::consts::TAU;
            let color = self.color(i);
            if sweep >= std::f64::consts::TAU - 1e-6 {
                if inner > 0.0 {
                    out.push_str(&format!(
                        r#"<path d="M{:.2} {cy:.2} A{r:.2} {r:.2} 0 1 1 {:.2} {cy:.2} A{r:.2} {r:.2} 0 1 1 {:.2} {cy:.2} Z M{:.2} {cy:.2} A{inner:.2} {inner:.2} 0 1 0 {:.2} {cy:.2} A{inner:.2} {inner:.2} 0 1 0 {:.2} {cy:.2} Z" fill="{color}" fill-rule="evenodd"/>"#,
                        cx - r, cx + r, cx - r, cx - inner, cx + inner, cx - inner
                    ));
                } else {
                    out.push_str(&format!(
                        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{color}"/>"#
                    ));
                }
            } else if sweep > 0.0 {
                let end = angle + sweep;
                let (x0, y0) = (cx + r * angle.cos(), cy + r * angle.sin());
                let (x1, y1) = (cx + r * end.cos(), cy + r * end.sin());
                let large = (sweep > std::f64::consts::PI) as u8;
                // A gap between slices, the width of a hairline.
                if inner > 0.0 {
                    let (ix1, iy1) = (cx + inner * end.cos(), cy + inner * end.sin());
                    let (ix0, iy0) = (cx + inner * angle.cos(), cy + inner * angle.sin());
                    out.push_str(&format!(
                        r#"<path d="M{x0:.2} {y0:.2} A{r:.2} {r:.2} 0 {large} 1 {x1:.2} {y1:.2} L{ix1:.2} {iy1:.2} A{inner:.2} {inner:.2} 0 {large} 0 {ix0:.2} {iy0:.2} Z" fill="{color}"/>"#
                    ));
                } else {
                    out.push_str(&format!(
                        r#"<path d="M{cx:.2} {cy:.2} L{x0:.2} {y0:.2} A{r:.2} {r:.2} 0 {large} 1 {x1:.2} {y1:.2} Z" fill="{color}"/>"#
                    ));
                }
                if self.labels && sweep > 0.3 {
                    let mid = angle + sweep / 2.0;
                    let lr = if inner > 0.0 {
                        (r + inner) / 2.0
                    } else {
                        r * 0.62
                    };
                    out.push_str(&format!(
                        r##"<text x="{:.2}" y="{:.2}" text-anchor="middle" fill="#fff" font-weight="bold">{}%</text>"##,
                        cx + lr * mid.cos(),
                        cy + lr * mid.sin() + 4.0,
                        (v / total * 100.0).round()
                    ));
                }
            }
            angle += sweep;
        }
        if self.legend {
            let x = cx + r + 18.0;
            let line = 18.0;
            let start = cy - (values.len() as f64 * line) / 2.0 + 10.0;
            for (i, row) in self.data.iter().enumerate() {
                let y = start + i as f64 * line;
                out.push_str(&format!(
                    r#"<rect x="{x:.2}" y="{:.2}" width="10" height="10" rx="2" fill="{}"/><text x="{:.2}" y="{y:.2}" fill="{}">{} — {}{}</text>"#,
                    y - 9.0,
                    self.color(i),
                    x + 16.0,
                    self.ink,
                    esc(&self.row_label(row, i)),
                    short(values[i]),
                    esc(&self.unit)
                ));
            }
        }
    }
}

/// A QR code as an SVG document: one path of dark modules on a light
/// ground, with the quiet zone the standard asks for.
pub fn qr_svg(value: &str, dark: &str, light: &str) -> Option<String> {
    let code =
        qrcode::QrCode::with_error_correction_level(value.as_bytes(), qrcode::EcLevel::M).ok()?;
    let width = code.width();
    let quiet = 4;
    let size = width + quiet * 2;
    let colors = code.to_colors();
    let mut path = String::new();
    for y in 0..width {
        let mut x = 0;
        while x < width {
            if colors[y * width + x] == qrcode::Color::Dark {
                let start = x;
                while x < width && colors[y * width + x] == qrcode::Color::Dark {
                    x += 1;
                }
                path.push_str(&format!(
                    "M{} {}h{}v1h-{}z",
                    start + quiet,
                    y + quiet,
                    x - start,
                    x - start
                ));
            } else {
                x += 1;
            }
        }
    }
    Some(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" class="wf-qr" viewBox="0 0 {size} {size}" width="{size}" height="{size}" shape-rendering="crispEdges"><rect width="{size}" height="{size}" fill="{light}"/><path d="{path}" fill="{dark}"/></svg>"#
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_bar_chart_draws_a_bar_per_value_on_a_round_axis() {
        let c = Chart {
            data: vec![
                json!({"m": "Jan", "v": 12}),
                json!({"m": "Feb", "v": 30}),
                json!({"m": "Mar", "v": 18}),
            ],
            x: Some("m".into()),
            y: vec!["v".into()],
            ..Chart::default()
        };
        let svg = c.svg();
        assert_eq!(svg.matches("<rect").count(), 3);
        assert!(svg.contains(">Feb<"));
        // 30 tops an axis of 0, 10, 20, 30.
        assert!(
            svg.contains(">30<") && svg.contains(">10<") && svg.contains(">20<"),
            "{svg}"
        );
        assert!(usvg::Tree::from_str(&svg, &usvg::Options::default()).is_ok());
    }

    #[test]
    fn lines_pies_and_series() {
        let rows = vec![
            json!({"d": "Mon", "p50": 20, "p95": 60}),
            json!({"d": "Tue", "p50": 25, "p95": 70}),
        ];
        let line = Chart {
            kind: Kind::Line,
            data: rows.clone(),
            x: Some("d".into()),
            y: vec!["p50".into(), "p95".into()],
            ..Chart::default()
        };
        let svg = line.svg();
        assert_eq!(svg.matches("stroke-width=\"2\"").count(), 2);
        // Two series: a legend.
        assert!(svg.contains(">p95<"));
        let pie = Chart {
            kind: Kind::Donut,
            data: rows,
            x: Some("d".into()),
            y: vec!["p95".into()],
            ..Chart::default()
        };
        let svg = pie.svg();
        assert_eq!(svg.matches("<path").count(), 2);
        assert!(svg.contains("Mon — 60"));
        for s in [line.svg(), pie.svg()] {
            assert!(usvg::Tree::from_str(&s, &usvg::Options::default()).is_ok());
        }
        assert_eq!(nice_step(99.0), 20.0);
        assert_eq!(short(12500.0), "12.5k");
    }

    #[test]
    fn a_qr_code_is_square_and_reads_as_svg() {
        let svg = qr_svg("https://example.com/pay/INV-0042", "#000", "#fff").unwrap();
        assert!(svg.contains("viewBox=\"0 0 "));
        assert!(usvg::Tree::from_str(&svg, &usvg::Options::default()).is_ok());
    }
}
