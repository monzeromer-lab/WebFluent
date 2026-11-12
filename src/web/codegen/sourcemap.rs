//! Source maps (`build.sourcemap`): where in the `.wf` sources each part of
//! the bundle and the page chunks came from, so that a stack trace from the
//! running site can be read back to the line its author wrote.
//!
//! The code generator writes a marker before the code of each statement it
//! emits, naming the file, line and column it came from, and an end marker
//! after each declaration. The minifier keeps the markers, placed right
//! before the next token. Last, `extract` takes them out of the finished
//! file and writes their places as a source map (version 3): the file is
//! then byte for byte what it is without `build.sourcemap`, with only the
//! `//# sourceMappingURL=` line added.
//!
//! A marker is a comment, `/*\u{1}wf:<source>:<line>:<column>*/`, or
//! `/*\u{1}wf:-*/` for the end of a declaration. The control character
//! never appears in what the compiler writes otherwise: a string literal
//! escapes it.

/// How a marker starts.
pub const MARK: &str = "/*\u{1}wf:";

/// The marker for source `file` (an index into the build's list of sources),
/// at `line` and `col` (both from 1, as a span has them).
pub fn marker(file: usize, line: u32, col: u32) -> String {
    format!("{MARK}{file}:{line}:{col}*/")
}

/// The marker that ends a declaration: what follows it is the compiler's own.
pub fn end_marker() -> String {
    format!("{MARK}-*/")
}

/// One place in the generated file and where it came from: its column (in
/// UTF-16 units, from 0), and the source's index, line and column (from 0),
/// or none for the compiler's own code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Segment {
    col: u32,
    from: Option<(u32, u32, u32)>,
}

/// The file without its markers, and its source map, as JSON: `file` is the
/// generated file's name, `sources` the source paths as the map names them,
/// and `contents` each one's text, so a browser's tools can show it (none
/// where it could not be read).
pub fn extract(
    js: &str,
    file: &str,
    sources: &[String],
    contents: &[Option<String>],
) -> (String, String) {
    let mut code = String::with_capacity(js.len());
    let mut lines: Vec<Vec<Segment>> = vec![Vec::new()];
    // The UTF-16 column the next character written takes.
    let mut col: u32 = 0;
    let mut rest = js;
    while let Some(at) = rest.find(MARK) {
        let (before, after) = rest.split_at(at);
        for ch in before.chars() {
            code.push(ch);
            if ch == '\n' {
                lines.push(Vec::new());
                col = 0;
            } else {
                col += ch.len_utf16() as u32;
            }
        }
        let body = &after[MARK.len()..];
        let Some(end) = body.find("*/") else {
            // Not a marker after all: keep it as it is.
            code.push_str(after);
            rest = "";
            break;
        };
        let from = parse(&body[..end]);
        let line = lines.last_mut().expect("there is always a line");
        let segment = Segment { col, from };
        // Two at one place: the last is the code's.
        match line.last_mut() {
            Some(last) if last.col == col => *last = segment,
            _ => line.push(segment),
        }
        rest = &body[end + 2..];
    }
    code.push_str(rest);
    let map = serde_json::json!({
        "version": 3,
        "file": file,
        "sources": sources,
        "sourcesContent": contents,
        "names": [],
        "mappings": mappings(&lines),
    });
    (code, map.to_string())
}

/// `<source>:<line>:<column>` (from 1), or `-`.
fn parse(body: &str) -> Option<(u32, u32, u32)> {
    let mut parts = body.split(':');
    let source = parts.next()?.parse().ok()?;
    let line: u32 = parts.next()?.parse().ok()?;
    let col: u32 = parts.next()?.parse().ok()?;
    Some((source, line.saturating_sub(1), col.saturating_sub(1)))
}

/// The `mappings` field: each generated line's segments, each a column and,
/// when it has one, its source, line and column, as base-64 VLQs relative
/// to the one before (the column to the line's last; the rest to the last
/// in the file).
fn mappings(lines: &[Vec<Segment>]) -> String {
    let mut out = String::new();
    let (mut source, mut line, mut col) = (0i64, 0i64, 0i64);
    for (n, segments) in lines.iter().enumerate() {
        if n > 0 {
            out.push(';');
        }
        let mut gen_col = 0i64;
        for (k, s) in segments.iter().enumerate() {
            if k > 0 {
                out.push(',');
            }
            vlq(&mut out, s.col as i64 - gen_col);
            gen_col = s.col as i64;
            if let Some((f, l, c)) = s.from {
                vlq(&mut out, f as i64 - source);
                vlq(&mut out, l as i64 - line);
                vlq(&mut out, c as i64 - col);
                (source, line, col) = (f as i64, l as i64, c as i64);
            }
        }
    }
    out
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn vlq(out: &mut String, value: i64) {
    let mut v = if value < 0 {
        ((-value) << 1) | 1
    } else {
        value << 1
    };
    loop {
        let mut digit = (v & 31) as usize;
        v >>= 5;
        if v > 0 {
            digit |= 32;
        }
        out.push(BASE64[digit] as char);
        if v == 0 {
            break;
        }
    }
}

/// A source map read back: what a stack trace's place came from.
#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    pub sources: Vec<String>,
    lines: Vec<Vec<Segment>>,
}

/// Where a generated place came from: the source's path as the map names
/// it, and its line and column, both from 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origin {
    pub source: String,
    pub line: u32,
    pub col: u32,
}

impl SourceMap {
    /// A version 3 source map, as JSON; none when it is not one.
    pub fn parse(json: &str) -> Option<SourceMap> {
        let v: serde_json::Value = serde_json::from_str(json).ok()?;
        if v["version"].as_u64()? != 3 {
            return None;
        }
        let sources = v["sources"]
            .as_array()?
            .iter()
            .map(|s| s.as_str().unwrap_or_default().to_string())
            .collect();
        let mut lines = Vec::new();
        let (mut source, mut line, mut col) = (0i64, 0i64, 0i64);
        for text in v["mappings"].as_str()?.split(';') {
            let mut segments = Vec::new();
            let mut gen_col = 0i64;
            for part in text.split(',').filter(|p| !p.is_empty()) {
                let fields = unvlq(part)?;
                gen_col += *fields.first()?;
                let from = if fields.len() >= 4 {
                    source += fields[1];
                    line += fields[2];
                    col += fields[3];
                    Some((source as u32, line as u32, col as u32))
                } else {
                    None
                };
                segments.push(Segment {
                    col: gen_col as u32,
                    from,
                });
            }
            lines.push(segments);
        }
        Some(SourceMap { sources, lines })
    }

    /// Where `line` and `col` (both from 1, as a stack trace gives them) of
    /// the generated file came from: the last place mapped at or before it
    /// on its line. None for the compiler's own code.
    pub fn lookup(&self, line: u32, col: u32) -> Option<Origin> {
        let segments = self.lines.get(line.checked_sub(1)? as usize)?;
        let at = col.saturating_sub(1);
        let s = segments.iter().rev().find(|s| s.col <= at)?;
        let (source, l, c) = s.from?;
        Some(Origin {
            source: self.sources.get(source as usize)?.clone(),
            line: l + 1,
            col: c + 1,
        })
    }
}

fn unvlq(text: &str) -> Option<Vec<i64>> {
    let mut out = Vec::new();
    let (mut value, mut shift) = (0i64, 0u32);
    for b in text.bytes() {
        let digit = BASE64.iter().position(|&c| c == b)? as i64;
        value |= (digit & 31) << shift;
        if digit & 32 != 0 {
            shift += 5;
            continue;
        }
        out.push(if value & 1 == 1 {
            -(value >> 1)
        } else {
            value >> 1
        });
        value = 0;
        shift = 0;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_markers_go_and_their_places_are_the_map() {
        let js = format!(
            "const a = 1;\nfunction Row() {{\n  {}const el = WF.el(\"div\");\n  {}el.textContent = \"é𝄞\"; {}x();\n}}\n{}WF.page(\"P\", Row);\n",
            marker(0, 3, 5),
            marker(1, 7, 9),
            marker(1, 8, 1),
            end_marker(),
        );
        let (code, map) = extract(
            &js,
            "app.js",
            &["src/a.wf".into(), "src/b.wf".into()],
            &[Some("page A {}".into()), None],
        );
        assert_eq!(
            code,
            "const a = 1;\nfunction Row() {\n  const el = WF.el(\"div\");\n  el.textContent = \"é𝄞\"; x();\n}\nWF.page(\"P\", Row);\n"
        );
        let json: serde_json::Value = serde_json::from_str(&map).unwrap();
        assert_eq!(
            json["sourcesContent"],
            serde_json::json!(["page A {}", null])
        );
        let map = SourceMap::parse(&map).unwrap();
        assert_eq!(map.sources, ["src/a.wf", "src/b.wf"]);
        let at = |line, col| {
            map.lookup(line, col)
                .map(|o| format!("{}:{}:{}", o.source, o.line, o.col))
        };
        // Before the first marker, nothing.
        assert_eq!(at(1, 1), None);
        // From a marker on, its place, to the end of its line.
        assert_eq!(at(3, 3).as_deref(), Some("src/a.wf:3:5"));
        assert_eq!(at(3, 20).as_deref(), Some("src/a.wf:3:5"));
        assert_eq!(at(4, 3).as_deref(), Some("src/b.wf:7:9"));
        // Columns count UTF-16 units: `é` is one, `𝄞` two.
        let x = "  el.textContent = \"é𝄞\"; ".encode_utf16().count() as u32 + 1;
        assert_eq!(at(4, x).as_deref(), Some("src/b.wf:8:1"));
        assert_eq!(at(4, x - 1).as_deref(), Some("src/b.wf:7:9"));
        // After the end marker, the compiler's own code is no one's.
        assert_eq!(at(6, 1), None);
        assert_eq!(at(99, 1), None);
    }

    #[test]
    fn vlqs_go_and_come_back() {
        for v in [0i64, 1, -1, 15, 16, -16, 31, 32, 1000, -123456] {
            let mut s = String::new();
            vlq(&mut s, v);
            assert_eq!(unvlq(&s), Some(vec![v]), "{v} as {s}");
        }
        // The examples every source map tool agrees on.
        let mut s = String::new();
        for v in [0, 0, 16, 1] {
            vlq(&mut s, v);
        }
        assert_eq!(s, "AAgBC");
    }

    #[test]
    fn a_map_that_is_not_one_is_none() {
        assert!(SourceMap::parse("{}").is_none());
        assert!(SourceMap::parse("{\"version\":2,\"sources\":[],\"mappings\":\"\"}").is_none());
        assert!(SourceMap::parse("not json").is_none());
        assert!(SourceMap::parse("{\"version\":3,\"sources\":[],\"mappings\":\"!\"}").is_none());
    }

    #[test]
    fn a_comment_that_only_starts_like_a_marker_is_kept() {
        let js = format!("a();{MARK}");
        let (code, _) = extract(&js, "app.js", &[], &[]);
        assert_eq!(code, js);
    }
}
