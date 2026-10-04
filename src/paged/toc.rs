//! A table of contents: the document's headings, each a link to itself,
//! with the page it lands on — known only once the document is laid out, so
//! the entries are written with a placeholder, the document laid out, and
//! the numbers filled in.

use super::html::{Dom, NodeKind};

/// One entry: the heading it names, and the text node its page goes in.
pub struct Entry {
    pub heading: usize,
    pub page_text: usize,
    pub page: Option<usize>,
}

/// What a page number is before it is known: wide enough for most documents,
/// so filling it in rarely moves anything.
const PLACEHOLDER: &str = "00";

fn level(dom: &Dom, n: usize) -> Option<u8> {
    let tag = dom.tag(n)?;
    let level = tag.strip_prefix('h')?.parse::<u8>().ok()?;
    (1..=6).contains(&level).then_some(level)
}

/// Whether a node is inside a table of contents or a running element,
/// whose headings are not the document's.
fn excluded(dom: &Dom, mut n: usize) -> bool {
    while let Some(p) = dom.nodes[n].parent {
        if let Some(e) = dom.element(p)
            && (e.has_class("wf-toc")
                || e.has_class("wf-header")
                || e.has_class("wf-footer")
                || e.has_class("wf-background")
                || e.has_class("wf-watermark"))
        {
            return true;
        }
        n = p;
    }
    false
}

/// Write the entries into every table of contents; `None` when the document
/// has none.
pub fn insert(dom: &mut Dom) -> Option<Vec<Entry>> {
    let tocs: Vec<usize> = dom
        .descendants(0)
        .into_iter()
        .filter(|&n| dom.element(n).is_some_and(|e| e.has_class("wf-toc")))
        .collect();
    if tocs.is_empty() {
        return None;
    }
    let headings: Vec<(usize, u8)> = dom
        .descendants(0)
        .into_iter()
        .filter_map(|n| level(dom, n).map(|l| (n, l)))
        .filter(|&(n, _)| !excluded(dom, n))
        .collect();
    // Every heading gets an id to be linked by.
    for (i, &(n, _)) in headings.iter().enumerate() {
        if dom.element(n).and_then(|e| e.attr("id")).is_none() {
            dom.set_attr(n, "id", &format!("wf-h-{}", i + 1));
        }
    }
    let mut entries = Vec::new();
    for toc in tocs {
        let levels: u8 = dom
            .element(toc)
            .and_then(|e| e.attr("data-levels"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);
        // The shallowest level listed reads as level 1.
        let top = headings
            .iter()
            .filter(|(_, l)| *l <= levels)
            .map(|(_, l)| *l)
            .min()
            .unwrap_or(1);
        for &(n, l) in &headings {
            if l > levels {
                continue;
            }
            let id = dom
                .element(n)
                .and_then(|e| e.attr("id"))
                .unwrap_or("")
                .to_string();
            let text = dom
                .text_content(n)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let depth = (l - top + 1).min(3);
            let a = dom.append_element(
                toc,
                "a",
                vec![
                    (
                        "class".to_string(),
                        format!("wf-toc__entry wf-toc__entry--{depth}"),
                    ),
                    ("href".to_string(), format!("#{id}")),
                ],
            );
            let label = dom.append_element(
                a,
                "span",
                vec![("class".to_string(), "wf-toc__label".to_string())],
            );
            dom.append_text(label, &text);
            dom.append_element(
                a,
                "span",
                vec![("class".to_string(), "wf-toc__leader".to_string())],
            );
            let page = dom.append_element(
                a,
                "span",
                vec![("class".to_string(), "wf-toc__page".to_string())],
            );
            let page_text = dom.append_text(page, PLACEHOLDER);
            entries.push(Entry {
                heading: n,
                page_text,
                page: None,
            });
        }
    }
    Some(entries)
}

/// Fill in each entry's page; whether any changed since the last fill.
pub fn fill(dom: &mut Dom, entries: &[Entry], page_of: impl Fn(usize) -> Option<usize>) -> bool {
    let mut changed = false;
    for e in entries {
        let page = page_of(e.heading);
        let text = page.map(|p| p.to_string()).unwrap_or_default();
        let current = match &dom.nodes[e.page_text].kind {
            NodeKind::Text(t) => t.clone(),
            _ => String::new(),
        };
        if current != text {
            dom.set_text(e.page_text, &text);
            changed = true;
        }
    }
    changed
}
