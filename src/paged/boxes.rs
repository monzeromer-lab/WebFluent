//! The styled tree turned into boxes: what layout lays out.
//!
//! Each element becomes a box with its computed style. Runs of inline
//! content — text, inline elements, inline-blocks among them — become one
//! anonymous paragraph box each, shaped once. A table becomes a grid of its
//! cells. The paged elements leave the flow: a document's `Header`,
//! `Footer`, `Background` and `Watermark` are kept to be drawn on each page.

use std::rc::Rc;

use super::cascade::Styler;
use super::fonts::FontDb;
use super::html::{Dom, NodeKind};
use super::style::{Display, ListStyle, Style};
use super::text::{Item, Shaped};

/// A picture a box draws: decoded later, sized now.
pub enum Asset {
    Raster {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    Svg {
        tree: Box<usvg::Tree>,
    },
}

pub enum Kind {
    /// A block, flex or grid container (by its style's `display`).
    Container,
    /// A paragraph: the index of its shaped text.
    Inline(usize),
    /// A picture: the asset, or `None` when it could not be read.
    Image { asset: Option<usize>, alt: String },
    /// A `<progress>`: how full.
    Progress(f32),
    /// A table: laid out as a grid of its cells.
    Table {
        columns: usize,
        header_rows: usize,
        row_styles: Vec<Rc<Style>>,
    },
    Cell {
        row: usize,
        col: usize,
        colspan: usize,
        rowspan: usize,
    },
    /// Where a page must end.
    PageBreak,
}

pub struct LBox {
    pub style: Rc<Style>,
    pub kind: Kind,
    pub children: Vec<usize>,
    /// The element it came from, for ids, links and headings.
    pub node: Option<usize>,
    /// A list item's marker, shaped: an index into the tree's paragraphs.
    pub marker: Option<usize>,
}

/// An inline element's style, for its background and its link.
pub struct Span {
    pub style: Rc<Style>,
    pub href: Option<String>,
    pub node: usize,
}

/// Which pages a running element is drawn on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum On {
    All,
    First,
    Rest,
    Odd,
    Even,
}

impl On {
    pub fn parse(v: Option<&str>) -> On {
        match v.unwrap_or("all") {
            "first" => On::First,
            "rest" | "notFirst" => On::Rest,
            "odd" => On::Odd,
            "even" => On::Even,
            _ => On::All,
        }
    }
    pub fn applies(self, page: usize) -> bool {
        match self {
            On::All => true,
            On::First => page == 1,
            On::Rest => page > 1,
            On::Odd => page % 2 == 1,
            On::Even => page % 2 == 0,
        }
    }
}

#[derive(Default)]
/// The running elements, by the element each is built from.
pub struct Running {
    pub headers: Vec<(On, usize)>,
    pub footers: Vec<(On, usize)>,
    pub backgrounds: Vec<(On, usize)>,
    pub watermarks: Vec<(On, usize)>,
}

pub struct Tree {
    pub boxes: Vec<LBox>,
    pub shaped: Vec<Shaped>,
    pub spans: Vec<Span>,
    pub assets: Vec<Asset>,
    /// The box the document flows from.
    pub flow: usize,
    pub running: Running,
    /// The `wf-document` element, whose attributes set the paper.
    pub document: Option<usize>,
    /// Characters no face had, and the element that held them.
    pub missing: Vec<(char, Option<usize>)>,
    /// The heading boxes and their levels, in document order: what keeps
    /// with what follows, and what the outline lists.
    pub headings: Vec<(usize, u8)>,
}

/// Reads an asset by the address a page names it by.
pub type AssetReader<'a> = dyn Fn(&str) -> Option<Vec<u8>> + 'a;

pub struct Builder<'a> {
    pub dom: &'a Dom,
    pub styler: &'a Styler,
    pub db: &'a mut FontDb,
    pub assets: &'a AssetReader<'a>,
    tree: Tree,
    /// The page number and the page count, written where a running element
    /// names `page` and `pages`.
    page_vars: Option<(usize, usize)>,
    /// Leave the running elements in place: building one of them.
    keep_running: bool,
}

/// What the template engine writes for `page` and `pages` inside a running
/// element: private-use characters, replaced on each page.
pub const PAGE_MARK: char = '\u{F8F0}';
pub const PAGES_MARK: char = '\u{F8F1}';

#[derive(Clone, Copy, PartialEq)]
enum Ctx {
    /// Block flow: inline content becomes paragraphs.
    Flow,
    /// A flex or grid container: every child is an item.
    Items,
}

impl<'a> Builder<'a> {
    pub fn new(
        dom: &'a Dom,
        styler: &'a Styler,
        db: &'a mut FontDb,
        assets: &'a AssetReader<'a>,
    ) -> Builder<'a> {
        Builder {
            dom,
            styler,
            db,
            assets,
            tree: Tree {
                boxes: Vec::new(),
                shaped: Vec::new(),
                spans: Vec::new(),
                assets: Vec::new(),
                flow: 0,
                running: Running::default(),
                document: None,
                missing: Vec::new(),
                headings: Vec::new(),
            },
            page_vars: None,
            keep_running: false,
        }
    }

    /// Build one running element (a header, a footer, a background, a
    /// watermark) as a flow of its own, for page `page` of `pages`.
    pub fn build_running(mut self, node: usize, page: usize, pages: usize) -> Tree {
        self.page_vars = Some((page, pages));
        self.keep_running = true;
        let dom = self.dom;
        let mut chain = vec![];
        let mut cur = dom.nodes[node].parent;
        while let Some(p) = cur {
            if dom.element(p).is_some() {
                chain.push(p);
            }
            cur = dom.nodes[p].parent;
        }
        let mut style = Style::root(self.styler.units.rem, "sans-serif");
        for n in chain.into_iter().rev() {
            style = self.styler.compute(dom, n, &style);
        }
        let own = self.styler.compute(dom, node, &style);
        let b = self.element(node, own, &style).unwrap_or_else(|| {
            self.push(LBox {
                style: Rc::new(style.clone()),
                kind: Kind::Container,
                children: Vec::new(),
                node: None,
                marker: None,
            })
        });
        self.tree.flow = b;
        self.tree
    }

    /// Build from the document's `<body>` (or its root), with the page's
    /// root style.
    pub fn build(mut self, root_style: &Style) -> Tree {
        let dom = self.dom;
        let html = dom.find_tag("html");
        let mut style = root_style.clone();
        if let Some(h) = html {
            style = self.styler.compute(dom, h, &style);
        }
        let body = dom.find_tag("body");
        let start = body.unwrap_or(0);
        if let Some(b) = body {
            style = self.styler.compute(dom, b, &style);
        }
        // The flow is the body's content; a `wf-document` inside it is where
        // the paged elements live.
        let document = dom
            .descendants(start)
            .into_iter()
            .find(|&n| dom.element(n).is_some_and(|e| e.has_class("wf-document")));
        self.tree.document = document;
        let flow_style = Rc::new(style.clone());
        let children = self.children(start, &style, Ctx::Flow);
        let flow = self.push(LBox {
            style: flow_style,
            kind: Kind::Container,
            children,
            node: body,
            marker: None,
        });
        self.tree.flow = flow;
        self.tree
    }

    fn push(&mut self, b: LBox) -> usize {
        self.tree.boxes.push(b);
        self.tree.boxes.len() - 1
    }

    /// The boxes for a node's children.
    fn children(&mut self, node: usize, style: &Style, ctx: Ctx) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        let mut items: Vec<Item> = Vec::new();
        self.pseudo(node, style, true, ctx, &mut items, &mut out);
        let kids = self.dom.nodes[node].children.clone();
        for child in kids {
            self.child(child, style, ctx, None, &mut items, &mut out);
        }
        self.pseudo(node, style, false, ctx, &mut items, &mut out);
        self.flush(&mut items, style, &mut out);
        out
    }

    fn pseudo(
        &mut self,
        node: usize,
        style: &Style,
        before: bool,
        ctx: Ctx,
        items: &mut Vec<Item>,
        out: &mut Vec<usize>,
    ) {
        if self.dom.element(node).is_none() {
            return;
        }
        let Some(ps) = self.styler.compute_pseudo(self.dom, node, style, before) else {
            return;
        };
        let text = ps.content.clone().unwrap_or_default();
        if text.is_empty() && ps.background_color.a == 0.0 && !ps.border.iter().any(|b| b.visible())
        {
            return;
        }
        let ps = Rc::new(ps);
        if ps.display.is_inline_level() && ps.display == Display::Inline && ctx == Ctx::Flow {
            items.push(Item::Text {
                text,
                style: ps,
                span: None,
            });
        } else {
            // A generated box of its own: a rule, a dot, a label.
            self.flush(items, style, out);
            let mut kids = Vec::new();
            if !text.is_empty() {
                let mut its = vec![Item::Text {
                    text,
                    style: ps.clone(),
                    span: None,
                }];
                self.flush(&mut its, &ps, &mut kids);
            }
            let b = self.push(LBox {
                style: ps,
                kind: Kind::Container,
                children: kids,
                node: None,
                marker: None,
            });
            if ctx == Ctx::Flow && self.tree.boxes[b].style.display.is_inline_level() {
                items.push(Item::Atom {
                    boxed: b,
                    style: self.tree.boxes[b].style.clone(),
                });
            } else {
                out.push(b);
            }
        }
    }

    /// Turn the pending inline content into a paragraph box.
    fn flush(&mut self, items: &mut Vec<Item>, parent: &Style, out: &mut Vec<usize>) {
        if items.is_empty() {
            return;
        }
        let taken = std::mem::take(items);
        // Nothing but collapsible whitespace makes no paragraph.
        let meaningful = taken.iter().any(|i| match i {
            Item::Text { text, style, .. } => {
                !text.chars().all(char::is_whitespace)
                    || !matches!(
                        style.white_space,
                        super::style::WhiteSpace::Normal
                            | super::style::WhiteSpace::NoWrap
                            | super::style::WhiteSpace::PreLine
                    )
            }
            _ => true,
        });
        if !meaningful {
            return;
        }
        let mut block = parent.inherit();
        block.display = Display::Block;
        let block = Rc::new(block);
        let first_node = taken.iter().find_map(|i| match i {
            Item::Text { span: Some(s), .. } => Some(self.tree.spans[*s].node),
            _ => None,
        });
        let shaped = Shaped::new(taken, block.clone(), self.db);
        for &ch in &shaped.missing {
            if !self.tree.missing.iter().any(|(c, _)| *c == ch) {
                self.tree.missing.push((ch, first_node));
            }
        }
        if shaped.clusters.is_empty() {
            return;
        }
        self.tree.shaped.push(shaped);
        let ix = self.tree.shaped.len() - 1;
        let b = self.push(LBox {
            style: block,
            kind: Kind::Inline(ix),
            children: Vec::new(),
            node: None,
            marker: None,
        });
        out.push(b);
    }

    fn child(
        &mut self,
        node: usize,
        parent: &Style,
        ctx: Ctx,
        span: Option<usize>,
        items: &mut Vec<Item>,
        out: &mut Vec<usize>,
    ) {
        match &self.dom.nodes[node].kind {
            NodeKind::Text(t) => {
                if ctx == Ctx::Items && t.trim().is_empty() {
                    return;
                }
                let t = match self.page_vars {
                    Some((page, pages)) if t.contains([PAGE_MARK, PAGES_MARK]) => t
                        .replace(PAGE_MARK, &page.to_string())
                        .replace(PAGES_MARK, &pages.to_string()),
                    _ => t.clone(),
                };
                let t = &t;
                let style = match span {
                    Some(s) => self.tree.spans[s].style.clone(),
                    None => Rc::new(parent.inherit()),
                };
                items.push(Item::Text {
                    text: t.clone(),
                    style,
                    span,
                });
                if ctx == Ctx::Items {
                    // Text straight inside a flex or grid container is an
                    // anonymous item of its own.
                    self.flush(items, parent, out);
                }
            }
            NodeKind::Root => {}
            NodeKind::Element(el) => {
                let tag = el.tag.clone();
                if self.is_running(node) {
                    return;
                }
                let style = self.styler.compute(self.dom, node, parent);
                if style.display == Display::None {
                    return;
                }
                if tag == "br" {
                    items.push(Item::Break {
                        style: Rc::new(style),
                    });
                    return;
                }
                if style.display == Display::Contents {
                    let kids = self.dom.nodes[node].children.clone();
                    for k in kids {
                        self.child(k, &style, ctx, span, items, out);
                    }
                    return;
                }
                let replaced = matches!(tag.as_str(), "img" | "svg" | "progress" | "i")
                    && self.is_replaced(node);
                let inline = style.display == Display::Inline && !replaced;
                if inline && ctx == Ctx::Flow {
                    // An inline element: its text joins the paragraph, styled.
                    let href = el.attr("href").map(|h| h.to_string());
                    let st = Rc::new(style.clone());
                    self.tree.spans.push(Span {
                        style: st,
                        href,
                        node,
                    });
                    let my_span = self.tree.spans.len() - 1;
                    self.pseudo(node, &style, true, ctx, items, out);
                    let kids = self.dom.nodes[node].children.clone();
                    for k in kids {
                        self.child(k, &style, ctx, Some(my_span), items, out);
                    }
                    self.pseudo(node, &style, false, ctx, items, out);
                    return;
                }
                let atom = ctx == Ctx::Flow
                    && (style.display.is_inline_level()
                        || replaced && style.display == Display::Inline);
                if !atom {
                    self.flush(items, parent, out);
                }
                let Some(b) = self.element(node, style, parent) else {
                    return;
                };
                if atom {
                    let st = self.tree.boxes[b].style.clone();
                    items.push(Item::Atom {
                        boxed: b,
                        style: st,
                    });
                } else {
                    out.push(b);
                }
            }
        }
    }

    /// Whether an element is drawn as a picture rather than as content.
    fn is_replaced(&self, node: usize) -> bool {
        let el = self.dom.element(node).expect("an element");
        match el.tag.as_str() {
            "i" | "span" => el.attr("data-icon").is_some(),
            _ => true,
        }
    }

    /// A running element: a document's header, footer, background or
    /// watermark, drawn on every page rather than in the flow.
    fn is_running(&mut self, node: usize) -> bool {
        if self.keep_running {
            return false;
        }
        let Some(doc) = self.tree.document else {
            return false;
        };
        if self.dom.nodes[node].parent != Some(doc) {
            return false;
        }
        let el = self.dom.element(node).expect("an element");
        let kind = if el.has_class("wf-header") {
            0
        } else if el.has_class("wf-footer") {
            1
        } else if el.has_class("wf-background") {
            2
        } else if el.has_class("wf-watermark") {
            3
        } else {
            return false;
        };
        let on = On::parse(el.attr("data-on"));
        // Recorded by element: each page builds its own, with its number.
        let list = match kind {
            0 => &mut self.tree.running.headers,
            1 => &mut self.tree.running.footers,
            2 => &mut self.tree.running.backgrounds,
            _ => &mut self.tree.running.watermarks,
        };
        list.push((on, node));
        true
    }

    /// The box for an element, built as a block-level box (an inline
    /// element here is blockified: it is a flex item, or an inline-block).
    fn element(&mut self, node: usize, style: Style, _parent: &Style) -> Option<usize> {
        let el = self.dom.element(node)?.clone();
        let tag = el.tag.as_str();
        if el.has_class("wf-page-break") {
            return Some(self.push(LBox {
                style: Rc::new(style),
                kind: Kind::PageBreak,
                children: Vec::new(),
                node: Some(node),
                marker: None,
            }));
        }
        match tag {
            "img" => {
                let asset = el.attr("src").and_then(|src| self.load(src));
                let alt = el.attr("alt").unwrap_or("").to_string();
                // `width="200"` is a size in CSS pixels where no rule sets one,
                // as a browser reads it.
                let mut style = style;
                for (attr, slot) in [("width", 0), ("height", 1)] {
                    if let Some(px) = el
                        .attr(attr)
                        .and_then(|v| v.trim_end_matches("px").parse::<f32>().ok())
                    {
                        let target = if slot == 0 {
                            &mut style.width
                        } else {
                            &mut style.height
                        };
                        if *target == super::style::Length::Auto {
                            *target = super::style::Length::Pt(px * 0.75);
                        }
                    }
                }
                return Some(self.push(LBox {
                    style: Rc::new(style),
                    kind: Kind::Image { asset, alt },
                    children: Vec::new(),
                    node: Some(node),
                    marker: None,
                }));
            }
            "svg" => {
                let markup = self.dom.outer_html(node);
                let asset = self.svg(&markup);
                return Some(self.push(LBox {
                    style: Rc::new(style),
                    kind: Kind::Image {
                        asset,
                        alt: String::new(),
                    },
                    children: Vec::new(),
                    node: Some(node),
                    marker: None,
                }));
            }
            "i" | "span" if el.attr("data-icon").is_some() => {
                let c = style.color;
                let hex = format!(
                    "#{:02x}{:02x}{:02x}",
                    (c.r * 255.0) as u8,
                    (c.g * 255.0) as u8,
                    (c.b * 255.0) as u8
                );
                let asset = el
                    .attr("data-icon")
                    .and_then(|n| super::icons::svg(n, &hex))
                    .and_then(|m| self.svg(&m));
                let mut style = style;
                // An icon is an em square unless its rules say otherwise.
                if style.width == super::style::Length::Auto {
                    style.width = super::style::Length::Pt(style.font_size);
                }
                if style.height == super::style::Length::Auto {
                    style.height = super::style::Length::Pt(style.font_size);
                }
                if style.display == Display::Inline {
                    style.display = Display::InlineBlock;
                }
                return Some(self.push(LBox {
                    style: Rc::new(style),
                    kind: Kind::Image {
                        asset,
                        alt: String::new(),
                    },
                    children: Vec::new(),
                    node: Some(node),
                    marker: None,
                }));
            }
            "progress" | "meter" => {
                let value: f32 = el.attr("value").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let max: f32 = el.attr("max").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                let mut style = style;
                if style.width == super::style::Length::Auto {
                    style.width = super::style::Length::Pct(100.0);
                }
                if style.height == super::style::Length::Auto {
                    style.height = super::style::Length::Pt(6.0);
                }
                let frac = if max > 0.0 {
                    (value / max).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                return Some(self.push(LBox {
                    style: Rc::new(style),
                    kind: Kind::Progress(frac),
                    children: Vec::new(),
                    node: Some(node),
                    marker: None,
                }));
            }
            _ => {}
        }
        if style.display == Display::Table {
            return Some(self.table(node, style));
        }
        let ctx = match style.display {
            Display::Flex | Display::InlineFlex | Display::Grid | Display::InlineGrid => Ctx::Items,
            _ => Ctx::Flow,
        };
        let marker = (style.display == Display::ListItem)
            .then(|| self.marker(node, &style))
            .flatten()
            .map(|text| {
                let st = Rc::new(style.inherit());
                let shaped = Shaped::new(
                    vec![Item::Text {
                        text,
                        style: st.clone(),
                        span: None,
                    }],
                    st,
                    self.db,
                );
                self.tree.shaped.push(shaped);
                self.tree.shaped.len() - 1
            });
        let mut style = style;
        if style.display == Display::Inline {
            style.display = Display::Block;
        }
        let children = self.children(node, &style, ctx);
        let b = self.push(LBox {
            style: Rc::new(style),
            kind: Kind::Container,
            children,
            node: Some(node),
            marker,
        });
        if let Some(level) = tag
            .strip_prefix('h')
            .and_then(|n| n.parse::<u8>().ok())
            .filter(|n| (1..=6).contains(n))
        {
            self.tree.headings.push((b, level));
        }
        Some(b)
    }

    fn marker(&self, node: usize, style: &Style) -> Option<String> {
        if style.list_style == ListStyle::None {
            return None;
        }
        let parent = self.dom.nodes[node].parent?;
        let start: i64 = self
            .dom
            .element(parent)
            .and_then(|p| p.attr("start"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let index = self
            .dom
            .element_children(parent)
            .take_while(|&c| c != node)
            .filter(|&c| {
                self.dom.tag(c) == Some("li")
                    || self
                        .dom
                        .element(c)
                        .is_some_and(|e| e.has_class("wf-list__item"))
            })
            .count() as i64;
        let n = self
            .dom
            .element(node)
            .and_then(|e| e.attr("value"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(start + index);
        Some(match style.list_style {
            ListStyle::Disc => "•".to_string(),
            ListStyle::Circle => "◦".to_string(),
            ListStyle::Square => "▪".to_string(),
            ListStyle::Decimal => format!("{n}."),
            ListStyle::LowerAlpha => format!("{}.", alpha(n, false)),
            ListStyle::UpperAlpha => format!("{}.", alpha(n, true)),
            ListStyle::LowerRoman => format!("{}.", roman(n).to_lowercase()),
            ListStyle::UpperRoman => format!("{}.", roman(n)),
            ListStyle::None => return None,
        })
    }

    fn table(&mut self, node: usize, style: Style) -> usize {
        let dom = self.dom;
        let mut rows: Vec<(usize, bool)> = Vec::new(); // (tr node, in head)
        let mut groups: Vec<(usize, Style)> = Vec::new();
        for child in dom.element_children(node).collect::<Vec<_>>() {
            let cs = self.styler.compute(dom, child, &style);
            match cs.display {
                Display::TableHeaderGroup | Display::TableRowGroup | Display::TableFooterGroup => {
                    let head = cs.display == Display::TableHeaderGroup;
                    for r in dom.element_children(child).collect::<Vec<_>>() {
                        rows.push((r, head));
                    }
                    groups.push((child, cs));
                }
                Display::TableRow => rows.push((child, false)),
                _ => {}
            }
        }
        // Header rows first, as a browser draws them.
        rows.sort_by_key(|(_, head)| !*head);
        let header_rows = rows.iter().filter(|(_, h)| *h).count();
        let mut cells = Vec::new();
        let mut row_styles = Vec::new();
        let mut occupied: Vec<Vec<bool>> = Vec::new();
        let mut columns = 0;
        for (r, (tr, _)) in rows.iter().enumerate() {
            let group_style = dom.nodes[*tr]
                .parent
                .and_then(|p| groups.iter().find(|(g, _)| *g == p).map(|(_, s)| s.clone()))
                .unwrap_or_else(|| style.clone());
            let rs = self.styler.compute(dom, *tr, &group_style);
            if occupied.len() <= r {
                occupied.resize(r + 1, Vec::new());
            }
            let mut col = 0;
            for td in dom.element_children(*tr).collect::<Vec<_>>() {
                let el = dom.element(td).expect("an element");
                let colspan: usize = el
                    .attr("colspan")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1)
                    .max(1);
                let rowspan: usize = el
                    .attr("rowspan")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1)
                    .max(1);
                while occupied[r].get(col).copied().unwrap_or(false) {
                    col += 1;
                }
                let cs = self.styler.compute(dom, td, &rs);
                if cs.display == Display::None {
                    continue;
                }
                for rr in r..r + rowspan {
                    if occupied.len() <= rr {
                        occupied.resize(rr + 1, Vec::new());
                    }
                    if occupied[rr].len() < col + colspan {
                        occupied[rr].resize(col + colspan, false);
                    }
                    occupied[rr][col..col + colspan].fill(true);
                }
                let children = self.children(td, &cs, Ctx::Flow);
                let mut cs = cs;
                cs.display = Display::Block;
                let b = self.push(LBox {
                    style: Rc::new(cs),
                    kind: Kind::Cell {
                        row: r,
                        col,
                        colspan,
                        rowspan,
                    },
                    children,
                    node: Some(td),
                    marker: None,
                });
                cells.push(b);
                col += colspan;
                columns = columns.max(col);
            }
            row_styles.push(Rc::new(rs));
        }
        self.push(LBox {
            style: Rc::new(style),
            kind: Kind::Table {
                columns,
                header_rows,
                row_styles,
            },
            children: cells,
            node: Some(node),
            marker: None,
        })
    }

    fn load(&mut self, src: &str) -> Option<usize> {
        let bytes = (self.assets)(src)?;
        let lower = src.to_ascii_lowercase();
        if lower.ends_with(".svg") || bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
            let text = String::from_utf8_lossy(&bytes).to_string();
            return self.svg(&text);
        }
        let (width, height) = image::ImageReader::new(std::io::Cursor::new(&bytes))
            .with_guessed_format()
            .ok()?
            .into_dimensions()
            .ok()?;
        self.tree.assets.push(Asset::Raster {
            data: bytes,
            width,
            height,
        });
        Some(self.tree.assets.len() - 1)
    }

    fn svg(&mut self, markup: &str) -> Option<usize> {
        let options = usvg::Options {
            fontdb: super::fonts::svg_fontdb(),
            ..usvg::Options::default()
        };
        let markup = if markup.contains("xmlns") {
            markup.to_string()
        } else {
            markup.replacen("<svg", r#"<svg xmlns="http://www.w3.org/2000/svg""#, 1)
        };
        let tree = usvg::Tree::from_str(&markup, &options).ok()?;
        self.tree.assets.push(Asset::Svg { tree: Box::new(tree) });
        Some(self.tree.assets.len() - 1)
    }
}

fn alpha(n: i64, upper: bool) -> String {
    let mut n = n.max(1);
    let mut s = Vec::new();
    while n > 0 {
        n -= 1;
        let c = (b'a' + (n % 26) as u8) as char;
        s.push(if upper { c.to_ascii_uppercase() } else { c });
        n /= 26;
    }
    s.iter().rev().collect()
}

fn roman(mut n: i64) -> String {
    let table = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (v, s) in table {
        while n >= v {
            out.push_str(s);
            n -= v;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::css::Sheet;
    use super::super::style::Units;
    use super::*;

    fn build(html: &str, css: &str) -> Tree {
        let dom = Dom::parse(html);
        let styler = Styler::new(
            &Sheet::parse(super::super::cascade::UA_SHEET),
            &Sheet::parse(css),
            Units {
                em: 12.0,
                rem: 12.0,
                vw: 600.0,
                vh: 800.0,
            },
        );
        let mut db = FontDb::new(false);
        let read = |_: &str| -> Option<Vec<u8>> { None };
        let b = Builder::new(&dom, &styler, &mut db, &read);
        b.build(&Style::root(12.0, "sans-serif"))
    }

    fn kinds(t: &Tree, b: usize) -> String {
        let lb = &t.boxes[b];
        let name = match &lb.kind {
            Kind::Container => format!("{:?}", lb.style.display),
            Kind::Inline(_) => "Text".into(),
            Kind::Image { .. } => "Image".into(),
            Kind::Progress(_) => "Progress".into(),
            Kind::Table { columns, .. } => format!("Table{columns}"),
            Kind::Cell { row, col, .. } => format!("Cell{row}{col}"),
            Kind::PageBreak => "Break".into(),
        };
        if lb.children.is_empty() {
            name
        } else {
            format!(
                "{name}({})",
                lb.children
                    .iter()
                    .map(|&c| kinds(t, c))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        }
    }

    #[test]
    fn inline_content_becomes_paragraphs_between_blocks() {
        let t = build(
            "<body>loose <b>text</b><div>block</div>more <span class=\"pill\">pill</span> after</body>",
            ".pill { display: inline-flex }",
        );
        assert_eq!(kinds(&t, t.flow), "Block(Text Block(Text) Text)");
        // The pill is an atom inside the last paragraph.
        let last = *t.boxes[t.flow].children.last().unwrap();
        let Kind::Inline(ix) = t.boxes[last].kind else {
            panic!()
        };
        assert!(
            t.shaped[ix]
                .items
                .iter()
                .any(|i| matches!(i, Item::Atom { .. }))
        );
    }

    #[test]
    fn flex_children_are_items() {
        let t = build(
            r#"<body><div class="row"><span>a</span> <p>b</p> loose</div></body>"#,
            ".row { display: flex }",
        );
        assert_eq!(
            kinds(&t, t.flow),
            "Block(Flex(Block(Text) Block(Text) Text))"
        );
    }

    #[test]
    fn tables_become_cells_with_their_places() {
        let t = build(
            "<body><table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td colspan=\"2\">wide</td></tr><tr><td>1</td><td>2</td></tr></tbody></table></body>",
            "",
        );
        assert_eq!(
            kinds(&t, t.flow),
            "Block(Table2(Cell00(Text) Cell01(Text) Cell10(Text) Cell20(Text) Cell21(Text)))"
        );
        let table = t.boxes[t.flow].children[0];
        let Kind::Table { header_rows, .. } = &t.boxes[table].kind else {
            panic!()
        };
        assert_eq!(*header_rows, 1);
    }

    #[test]
    fn running_elements_leave_the_flow() {
        let t = build(
            r#"<body><div class="wf-document"><header class="wf-header" data-on="rest">H</header><p>body</p><footer class="wf-footer">F</footer><div class="wf-page-break"></div></div></body>"#,
            "",
        );
        assert_eq!(kinds(&t, t.flow), "Block(Block(Block(Text) Break))");
        assert_eq!(t.running.headers.len(), 1);
        assert_eq!(t.running.headers[0].0, On::Rest);
        assert_eq!(t.running.footers.len(), 1);
    }

    #[test]
    fn list_items_carry_their_markers_and_icons_are_pictures() {
        let t = build(
            r#"<body><ol start="3"><li>a</li><li>b</li></ol><ul><li>c</li></ul><p>x <i class="wf-icon" data-icon="check"></i></p></body>"#,
            "",
        );
        let markers: Vec<String> = t
            .boxes
            .iter()
            .filter_map(|b| b.marker.map(|m| t.shaped[m].text.clone()))
            .collect();
        assert_eq!(markers, vec!["3.", "4.", "•"]);
        assert!(t.assets.iter().any(|a| matches!(a, Asset::Svg { .. })));
        assert_eq!(roman(1994), "MCMXCIV");
        assert_eq!(alpha(28, false), "ab");
    }
}
