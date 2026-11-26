//! The structure of a tagged PDF: what each piece of drawn content is.
//!
//! The painter marks every piece of content it draws and records it as a
//! leaf of the element it belongs to ([`Leaf`]); decoration — backgrounds,
//! borders, shadows, running headers and footers, a repeated table header —
//! is marked an artifact and has no place here. This module walks the
//! document's elements in order and gives each one that means something a
//! group: `h1`–`h6` a heading (`H1`…`H6`, its text its title), `p` a
//! paragraph, `ul` and `ol` a list (`L`) of items (`LI`, its marker the
//! label, `Lbl`, the rest its body, `LBody`), a table with its row groups,
//! rows, header and data cells (`TH` scoped to its column in a `thead`, else
//! as its `scope` says, else its row), a picture with an `alt` or an
//! `aria-label` a figure carrying that text, `a href` a link holding its
//! text and its annotation, `blockquote`, `caption`, `figcaption`, `code`,
//! `section` and `article`. Any other element adds no group: what it holds
//! goes to the nearest one that does, and text loose in a section or the
//! document is gathered into a paragraph. An element with a `lang` of its
//! own says so on its group; the document's language is the catalog's.

use std::collections::HashMap;
use std::num::NonZeroU16;

use krilla::tagging::{
    ListNumbering, Node, TableHeaderScope, Tag, TagGroup, TagKind, TagTree,
};

use super::html::Dom;
use super::paint::Leaf;

/// A piece of the structure as it is gathered: content, or an element's
/// group, each with the place of its first content in the reading order.
enum Item {
    Leaf { label: bool, node: Node },
    Group(TagGroup),
}

/// The tag tree of a document from the leaves its pages drew. Every leaf is
/// placed exactly once: one whose element is not in the document goes at
/// the end, in the order it was drawn.
pub fn tree(dom: &Dom, leaves: Vec<Leaf>) -> TagTree {
    let mut by_node: HashMap<usize, Vec<(u64, Item)>> = HashMap::new();
    for l in leaves {
        by_node.entry(l.node).or_default().push((
            l.seq,
            Item::Leaf {
                label: l.label,
                node: l.id.into(),
            },
        ));
    }
    let mut items = collect(dom, 0, &mut by_node, false);
    let mut rest: Vec<(u64, Item)> = by_node.into_values().flatten().collect();
    rest.sort_by_key(|(s, _)| *s);
    items.extend(rest);
    let mut tree = TagTree::new();
    for node in paragraphs(items) {
        tree.push(node);
    }
    tree
}

fn collect(
    dom: &Dom,
    node: usize,
    by_node: &mut HashMap<usize, Vec<(u64, Item)>>,
    in_head: bool,
) -> Vec<(u64, Item)> {
    let mut items = by_node.remove(&node).unwrap_or_default();
    let tag = dom.tag(node).unwrap_or("");
    let head = in_head || tag == "thead";
    let children: Vec<usize> = dom.element_children(node).collect();
    for c in children {
        items.extend(collect(dom, c, by_node, head));
    }
    items.sort_by_key(|(s, _)| *s);
    let Some(first) = items.first().map(|(s, _)| *s) else {
        return items;
    };
    let Some(kind) = kind_of(dom, node, in_head) else {
        return items;
    };
    let kind = match dom.element(node).and_then(|e| e.attr("lang")) {
        Some(lang) if !lang.is_empty() => kind.with_lang(Some(lang.to_string())),
        _ => kind,
    };
    let mut group = TagGroup::new(kind);
    if tag == "li" {
        // The marker is the item's label; what it says, its body.
        let (labels, body): (Vec<_>, Vec<_>) = items
            .into_iter()
            .partition(|(_, i)| matches!(i, Item::Leaf { label: true, .. }));
        if !labels.is_empty() {
            let mut lbl = TagGroup::new(Tag::Lbl);
            for (_, i) in labels {
                lbl.push(node_of(i));
            }
            group.push(lbl);
        }
        let mut lbody = TagGroup::new(Tag::LBody);
        for (_, i) in body {
            lbody.push(node_of(i));
        }
        group.push(lbody);
    } else if matches!(tag, "section" | "article" | "blockquote") {
        for n in paragraphs(items) {
            group.push(n);
        }
    } else {
        for (_, i) in items {
            group.push(node_of(i));
        }
    }
    vec![(first, Item::Group(group))]
}

fn node_of(item: Item) -> Node {
    match item {
        Item::Leaf { node, .. } => node,
        Item::Group(g) => g.into(),
    }
}

/// Items for a place that holds blocks: content loose among them gathered
/// into a paragraph, a run at a time.
fn paragraphs(items: Vec<(u64, Item)>) -> Vec<Node> {
    let mut out = Vec::new();
    let mut loose: Option<TagGroup> = None;
    for (_, item) in items {
        match item {
            Item::Leaf { node, .. } => loose.get_or_insert_with(|| TagGroup::new(Tag::P)).push(node),
            Item::Group(g) => {
                if let Some(p) = loose.take() {
                    out.push(p.into());
                }
                out.push(g.into());
            }
        }
    }
    if let Some(p) = loose {
        out.push(p.into());
    }
    out
}

/// The group an element makes, or `None` for one that makes none.
fn kind_of(dom: &Dom, node: usize, in_head: bool) -> Option<TagKind> {
    let el = dom.element(node)?;
    let text = || {
        dom.text_content(node)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    Some(match el.tag.as_str() {
        h @ ("h1" | "h2" | "h3" | "h4" | "h5" | "h6") => {
            let level = h[1..].parse::<u16>().ok().and_then(NonZeroU16::new)?;
            Tag::Hn(level, Some(text())).into()
        }
        "p" | "pre" => Tag::P.into(),
        "ul" => Tag::L(ListNumbering::Disc).into(),
        "ol" => Tag::L(ListNumbering::Decimal).into(),
        "li" => Tag::LI.into(),
        "table" => Tag::Table.into(),
        "thead" => Tag::THead.into(),
        "tbody" => Tag::TBody.into(),
        "tfoot" => Tag::TFoot.into(),
        "tr" => Tag::TR.into(),
        "th" => {
            let scope = match el.attr("scope") {
                Some("row") => TableHeaderScope::Row,
                Some("col") => TableHeaderScope::Column,
                _ if in_head => TableHeaderScope::Column,
                _ => TableHeaderScope::Row,
            };
            Tag::TH(scope).into()
        }
        "td" => Tag::TD.into(),
        "caption" | "figcaption" => Tag::Caption.into(),
        "blockquote" => Tag::BlockQuote.into(),
        "code" => Tag::Code.into(),
        "section" => Tag::Section.into(),
        "article" => Tag::Article.into(),
        "a" if el.attr("href").is_some() => Tag::Link.into(),
        "img" | "svg" => {
            let alt = el
                .attr("alt")
                .or_else(|| el.attr("aria-label"))
                .map(str::trim)
                .filter(|a| !a.is_empty())?;
            Tag::Figure(Some(alt.to_string())).into()
        }
        _ if el.attr("data-icon").is_some() => {
            let alt = el.attr("aria-label").map(str::trim).filter(|a| !a.is_empty())?;
            Tag::Figure(Some(alt.to_string())).into()
        }
        _ => return None,
    })
}
