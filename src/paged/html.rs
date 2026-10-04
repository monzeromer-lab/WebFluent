//! The HTML the template engine writes, read back into a tree.
//!
//! The engine's own output is well formed, but a page may carry Markdown and
//! sanitised markup from outside the project, so the reader forgives what a
//! browser forgives: a missing end tag closes with its parent, a stray one is
//! ignored, a void element needs no `/>`.

/// One node of the tree: an element, a run of text, or the root.
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Root,
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct Element {
    /// Lower case: `div`, `h1`, `svg`.
    pub tag: String,
    pub attrs: Vec<(String, String)>,
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn classes(&self) -> impl Iterator<Item = &str> {
        self.attr("class").unwrap_or("").split_whitespace()
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.classes().any(|c| c == class)
    }
}

/// A parsed document: node 0 is the root.
#[derive(Debug, Clone)]
pub struct Dom {
    pub nodes: Vec<Node>,
}

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// Elements whose content is text up to their end tag, never markup.
const RAW_TEXT: &[&str] = &["script", "style", "textarea", "title"];

impl Dom {
    pub fn parse(html: &str) -> Dom {
        let mut dom = Dom {
            nodes: vec![Node {
                kind: NodeKind::Root,
                parent: None,
                children: Vec::new(),
            }],
        };
        let mut stack: Vec<usize> = vec![0];
        let bytes = html.as_bytes();
        let mut i = 0;
        let mut text_start = 0;
        while i < bytes.len() {
            if bytes[i] != b'<' {
                i += 1;
                continue;
            }
            let rest = &html[i..];
            // A `<` that opens nothing is text, and stays in the run.
            let opens = rest.starts_with("<!")
                || rest.starts_with("<?")
                || rest.starts_with("</")
                || rest
                    .as_bytes()
                    .get(1)
                    .is_some_and(|c| c.is_ascii_alphabetic());
            if !opens {
                i += 1;
                continue;
            }
            // Whatever text came before the tag.
            if text_start < i {
                dom.push_text(*stack.last().unwrap(), &html[text_start..i]);
            }
            if rest.starts_with("<!--") {
                let end = rest.find("-->").map(|e| i + e + 3).unwrap_or(bytes.len());
                i = end;
                text_start = i;
                continue;
            }
            if rest.starts_with("<!") || rest.starts_with("<?") {
                let end = rest.find('>').map(|e| i + e + 1).unwrap_or(bytes.len());
                i = end;
                text_start = i;
                continue;
            }
            if let Some(after) = rest.strip_prefix("</") {
                let name_len = after
                    .find(|c: char| c == '>' || c.is_whitespace())
                    .unwrap_or(after.len());
                let name = after[..name_len].to_ascii_lowercase();
                let end = rest.find('>').map(|e| i + e + 1).unwrap_or(bytes.len());
                // Close the nearest open element of that name, and whatever
                // was left open inside it; an end tag nothing opened is noise.
                if let Some(pos) = stack
                    .iter()
                    .rposition(|&n| dom.tag(n) == Some(name.as_str()))
                {
                    stack.truncate(pos.max(1));
                }
                i = end;
                text_start = i;
                continue;
            }
            // A start tag — or a `<` that is just text.
            let Some((element, self_closing, consumed)) = parse_start_tag(rest) else {
                text_start = i;
                i += 1;
                continue;
            };
            let tag = element.tag.clone();
            let parent = *stack.last().unwrap();
            let id = dom.push(parent, NodeKind::Element(element));
            i += consumed;
            text_start = i;
            if RAW_TEXT.contains(&tag.as_str()) {
                let close = format!("</{tag}");
                let end = html[i..]
                    .to_ascii_lowercase()
                    .find(&close)
                    .map(|e| i + e)
                    .unwrap_or(bytes.len());
                if end > i {
                    let text = html[i..end].to_string();
                    dom.push(id, NodeKind::Text(text));
                }
                i = html[end..]
                    .find('>')
                    .map(|e| end + e + 1)
                    .unwrap_or(bytes.len());
                text_start = i;
            } else if !self_closing && !VOID.contains(&tag.as_str()) {
                stack.push(id);
            }
        }
        if text_start < bytes.len() {
            dom.push_text(*stack.last().unwrap(), &html[text_start..]);
        }
        dom
    }

    fn push(&mut self, parent: usize, kind: NodeKind) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node {
            kind,
            parent: Some(parent),
            children: Vec::new(),
        });
        self.nodes[parent].children.push(id);
        id
    }

    fn push_text(&mut self, parent: usize, raw: &str) {
        if raw.is_empty() {
            return;
        }
        self.push(parent, NodeKind::Text(decode_entities(raw)));
    }

    /// Append an element to a node; its id.
    pub fn append_element(
        &mut self,
        parent: usize,
        tag: &str,
        attrs: Vec<(String, String)>,
    ) -> usize {
        self.push(
            parent,
            NodeKind::Element(Element {
                tag: tag.to_string(),
                attrs,
            }),
        )
    }

    /// Append a run of text to a node.
    pub fn append_text(&mut self, parent: usize, text: &str) -> usize {
        self.push(parent, NodeKind::Text(text.to_string()))
    }

    /// Set an attribute on an element.
    pub fn set_attr(&mut self, id: usize, name: &str, value: &str) {
        if let NodeKind::Element(e) = &mut self.nodes[id].kind {
            match e.attrs.iter_mut().find(|(k, _)| k == name) {
                Some(slot) => slot.1 = value.to_string(),
                None => e.attrs.push((name.to_string(), value.to_string())),
            }
        }
    }

    /// Replace the text of a text node.
    pub fn set_text(&mut self, id: usize, text: &str) {
        if let NodeKind::Text(t) = &mut self.nodes[id].kind {
            *t = text.to_string();
        }
    }

    pub fn element(&self, id: usize) -> Option<&Element> {
        match &self.nodes[id].kind {
            NodeKind::Element(e) => Some(e),
            _ => None,
        }
    }

    pub fn tag(&self, id: usize) -> Option<&str> {
        self.element(id).map(|e| e.tag.as_str())
    }

    /// The element children of a node, in order.
    pub fn element_children(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
        self.nodes[id]
            .children
            .iter()
            .copied()
            .filter(|&c| matches!(self.nodes[c].kind, NodeKind::Element(_)))
    }

    /// Every element under `id`, depth first, in document order.
    pub fn descendants(&self, id: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack: Vec<usize> = self.nodes[id].children.iter().rev().copied().collect();
        while let Some(n) = stack.pop() {
            out.push(n);
            stack.extend(self.nodes[n].children.iter().rev().copied());
        }
        out
    }

    /// The first element with this tag, anywhere.
    pub fn find_tag(&self, tag: &str) -> Option<usize> {
        self.descendants(0)
            .into_iter()
            .find(|&n| self.tag(n) == Some(tag))
    }

    /// All the text under a node, as written.
    pub fn text_content(&self, id: usize) -> String {
        let mut out = String::new();
        for n in std::iter::once(id).chain(self.descendants(id)) {
            if let NodeKind::Text(t) = &self.nodes[n].kind {
                out.push_str(t);
            }
        }
        out
    }

    /// A node written back as markup — how an inline `<svg>` reaches the SVG
    /// reader.
    pub fn outer_html(&self, id: usize) -> String {
        let mut out = String::new();
        self.write(id, &mut out);
        out
    }

    fn write(&self, id: usize, out: &mut String) {
        match &self.nodes[id].kind {
            NodeKind::Root => {
                for &c in &self.nodes[id].children {
                    self.write(c, out);
                }
            }
            NodeKind::Text(t) => out.push_str(&escape(t)),
            NodeKind::Element(e) => {
                out.push('<');
                out.push_str(&e.tag);
                for (k, v) in &e.attrs {
                    out.push(' ');
                    out.push_str(k);
                    out.push_str("=\"");
                    out.push_str(&escape(v));
                    out.push('"');
                }
                if VOID.contains(&e.tag.as_str()) {
                    out.push_str("/>");
                    return;
                }
                out.push('>');
                for &c in &self.nodes[id].children {
                    self.write(c, out);
                }
                out.push_str("</");
                out.push_str(&e.tag);
                out.push('>');
            }
        }
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `<tag a="1" b='2' c d=e/>` → the element, whether it closed itself, and
/// how many bytes it took. `None` when what follows `<` is not a tag.
fn parse_start_tag(s: &str) -> Option<(Element, bool, usize)> {
    let b = s.as_bytes();
    let mut i = 1;
    if i >= b.len() || !b[i].is_ascii_alphabetic() {
        return None;
    }
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b':') {
        i += 1;
    }
    let tag = s[1..i].to_ascii_lowercase();
    let mut attrs = Vec::new();
    let mut self_closing = false;
    loop {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= b.len() {
            return Some((Element { tag, attrs }, self_closing, i));
        }
        match b[i] {
            b'>' => {
                i += 1;
                break;
            }
            b'/' => {
                self_closing = true;
                i += 1;
                continue;
            }
            _ => {}
        }
        let name_start = i;
        while i < b.len() && !b[i].is_ascii_whitespace() && !matches!(b[i], b'=' | b'>' | b'/') {
            i += 1;
        }
        if i == name_start {
            i += 1;
            continue;
        }
        // An SVG keeps its attribute names' case (`viewBox`).
        let name = if tag == "svg" || is_svg_child(&tag) {
            s[name_start..i].to_string()
        } else {
            s[name_start..i].to_ascii_lowercase()
        };
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut value = String::new();
        if i < b.len() && b[i] == b'=' {
            i += 1;
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < b.len() && (b[i] == b'"' || b[i] == b'\'') {
                let quote = b[i];
                let start = i + 1;
                let end = s[start..]
                    .find(quote as char)
                    .map(|e| start + e)
                    .unwrap_or(b.len());
                value = decode_entities(&s[start..end]);
                i = (end + 1).min(b.len());
            } else {
                let start = i;
                while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'>' {
                    i += 1;
                }
                value = decode_entities(&s[start..i]);
            }
        }
        attrs.push((name, value));
    }
    Some((Element { tag, attrs }, self_closing, i))
}

fn is_svg_child(tag: &str) -> bool {
    matches!(
        tag,
        "path"
            | "circle"
            | "rect"
            | "line"
            | "polyline"
            | "polygon"
            | "ellipse"
            | "g"
            | "defs"
            | "lineargradient"
            | "radialgradient"
            | "stop"
            | "text"
            | "tspan"
            | "use"
            | "clippath"
            | "mask"
            | "symbol"
    )
}

/// `&amp;`, `&#8212;`, `&#x2014;`, `&nbsp;` and the named entities a page is
/// likely to carry.
pub fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        let end = after.find(';').filter(|&e| e <= 10);
        let decoded = end.and_then(|e| {
            let name = &after[..e];
            let ch = if let Some(num) = name.strip_prefix('#') {
                let code = if let Some(hex) = num.strip_prefix(['x', 'X']) {
                    u32::from_str_radix(hex, 16).ok()?
                } else {
                    num.parse().ok()?
                };
                char::from_u32(code)?
            } else {
                named_entity(name)?
            };
            Some((ch, e))
        });
        match decoded {
            Some((ch, e)) => {
                out.push(ch);
                rest = &after[e + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn named_entity(name: &str) -> Option<char> {
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "copy" => '©',
        "reg" => '®',
        "trade" => '™',
        "mdash" => '—',
        "ndash" => '–',
        "hellip" => '…',
        "lsquo" => '‘',
        "rsquo" => '’',
        "ldquo" => '“',
        "rdquo" => '”',
        "laquo" => '«',
        "raquo" => '»',
        "bull" => '•',
        "middot" => '·',
        "times" => '×',
        "divide" => '÷',
        "deg" => '°',
        "euro" => '€',
        "pound" => '£',
        "yen" => '¥',
        "cent" => '¢',
        "sect" => '§',
        "para" => '¶',
        "larr" => '←',
        "rarr" => '→',
        "uarr" => '↑',
        "darr" => '↓',
        "le" => '≤',
        "ge" => '≥',
        "ne" => '≠',
        "plusmn" => '±',
        "shy" => '\u{ad}',
        "zwj" => '\u{200d}',
        "zwnj" => '\u{200c}',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(dom: &Dom, id: usize) -> String {
        match &dom.nodes[id].kind {
            NodeKind::Root => dom.nodes[id]
                .children
                .iter()
                .map(|&c| shape(dom, c))
                .collect::<Vec<_>>()
                .join(""),
            NodeKind::Text(t) => t.trim().to_string(),
            NodeKind::Element(e) => format!(
                "{}({})",
                e.tag,
                dom.nodes[id]
                    .children
                    .iter()
                    .map(|&c| shape(dom, c))
                    .collect::<Vec<_>>()
                    .join("")
            ),
        }
    }

    #[test]
    fn reads_what_the_template_engine_writes() {
        let dom = Dom::parse(
            r#"<div class="wf-row wf-gap--md"><p class="wf-text">a &amp; b</p><img src="x.png" alt=""><br><span>c</span></div>"#,
        );
        assert_eq!(shape(&dom, 0), "div(p(a & b)img()br()span(c))");
        let div = dom.element(1).unwrap();
        assert!(div.has_class("wf-gap--md"));
        assert_eq!(dom.element_children(1).count(), 4);
    }

    #[test]
    fn forgives_what_a_browser_forgives() {
        // An end tag nothing opened is ignored; an unclosed element closes
        // with its parent.
        let dom = Dom::parse("<div><p>one</span><b>two</div><p>three</p>");
        assert_eq!(shape(&dom, 0), "div(p(oneb(two)))p(three)");
        // A `<` that starts no tag is text.
        let dom = Dom::parse("<p>1 < 2 and 3 > 2</p>");
        assert_eq!(dom.text_content(0), "1 < 2 and 3 > 2");
    }

    #[test]
    fn keeps_an_svg_as_written() {
        let dom =
            Dom::parse(r#"<svg viewBox="0 0 24 24"><path d="M3 12h18" stroke-width="2"/></svg>"#);
        let svg = dom.find_tag("svg").unwrap();
        assert_eq!(dom.element(svg).unwrap().attr("viewBox"), Some("0 0 24 24"));
        assert_eq!(
            dom.outer_html(svg),
            r#"<svg viewBox="0 0 24 24"><path d="M3 12h18" stroke-width="2"></path></svg>"#
        );
    }

    #[test]
    fn raw_text_is_not_markup() {
        let dom = Dom::parse("<style>.a > .b { color: red }</style><p>x</p>");
        let style = dom.find_tag("style").unwrap();
        assert_eq!(dom.text_content(style), ".a > .b { color: red }");
        assert!(dom.find_tag("p").is_some());
    }

    #[test]
    fn decodes_entities() {
        assert_eq!(
            decode_entities("a&nbsp;b &#8212; &#x41;&copy; &bogus; &"),
            "a\u{a0}b — A© &bogus; &"
        );
    }
}
