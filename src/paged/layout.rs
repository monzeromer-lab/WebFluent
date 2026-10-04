//! Layout: boxes given sizes and places.
//!
//! taffy lays out block, flex and grid containers (a table is a grid of its
//! cells); a paragraph is a leaf taffy measures through the text engine, and
//! a picture a leaf with its own proportions. An inline-block is laid out on
//! its own, as narrow as its content allows, and placed on its line.
//!
//! The result is a tree of fragments with absolute positions on one tall
//! strip as wide as the page's content; pagination cuts the strip.

use std::cell::RefCell;
use std::collections::HashMap;

use taffy::prelude::*;
use taffy::{
    CompactLength, LayoutInput, LayoutOutput, MaxTrackSizingFunction, MinTrackSizingFunction,
    Overflow,
};

use super::boxes::{Asset, Kind, Tree};
use super::fonts::FontDb;
use super::style::{
    Align as A, Display as D, FlexDirection as FD, GridLine, Length as L, Position as P,
    Style as S, Track, VerticalAlign,
};
use super::text::Line;

/// A box placed: its border box on the strip, and what is inside it.
#[derive(Clone, Debug)]
pub struct Frag {
    pub boxed: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub children: Vec<Frag>,
    /// A paragraph's lines, relative to the frag's top-left.
    pub lines: Vec<Line>,
    /// The inline-blocks on those lines, placed absolutely.
    pub atoms: Vec<Frag>,
    /// The top and bottom of everything the frag and its descendants
    /// cover, once [`Frag::measure_extent`] has run: what a page culls by.
    pub extent: (f32, f32),
}

impl Frag {
    /// Move the fragment and everything in it.
    pub fn shift(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
        for c in &mut self.children {
            c.shift(dx, dy);
        }
        for a in &mut self.atoms {
            a.shift(dx, dy);
        }
    }

    /// Work out every frag's extent, bottom up.
    pub fn measure_extent(&mut self) -> (f32, f32) {
        let mut top = self.y;
        let mut bottom = self.y + self.h;
        for c in self.children.iter_mut().chain(self.atoms.iter_mut()) {
            let (t, b) = c.measure_extent();
            top = top.min(t);
            bottom = bottom.max(b);
        }
        if let Some(l) = self.lines.last() {
            bottom = bottom.max(self.y + l.top + l.height);
        }
        self.extent = (top, bottom);
        self.extent
    }

    /// The baseline of the first line inside, from the frag's top.
    pub fn first_baseline(&self) -> Option<f32> {
        if let Some(l) = self.lines.first() {
            return Some(l.top + l.baseline);
        }
        for c in &self.children {
            if let Some(b) = c.first_baseline() {
                return Some(c.y - self.y + b);
            }
        }
        None
    }
}

#[derive(Clone, Copy)]
enum Leaf {
    Text(usize),
    /// A picture's natural size, and whether a percentage sizes it (then
    /// its narrowest is nothing, as CSS's compressible replaced elements).
    Image {
        w: f32,
        h: f32,
        compressible: bool,
    },
    Empty,
}

pub struct Layout<'a> {
    pub tree: &'a Tree,
    pub db: &'a FontDb,
    /// An inline-block's size at an available width, once worked out.
    atom_cache: RefCell<HashMap<(usize, u32), AtomSize>>,
}

/// Width, height, and the baseline from the top.
type AtomSize = (f32, f32, f32);

fn dim(l: L) -> Dimension {
    match l {
        L::Auto => Dimension::auto(),
        L::Pt(p) => Dimension::length(p),
        L::Pct(p) => Dimension::percent(p / 100.0),
    }
}

fn lpa(l: L) -> LengthPercentageAuto {
    match l {
        L::Auto => LengthPercentageAuto::auto(),
        L::Pt(p) => LengthPercentageAuto::length(p),
        L::Pct(p) => LengthPercentageAuto::percent(p / 100.0),
    }
}

fn lp(l: L) -> LengthPercentage {
    match l {
        L::Auto => LengthPercentage::length(0.0),
        L::Pt(p) => LengthPercentage::length(p),
        L::Pct(p) => LengthPercentage::percent(p / 100.0),
    }
}

fn align_items(a: Option<A>) -> Option<AlignItems> {
    Some(match a? {
        A::Start => AlignItems::FLEX_START,
        A::End => AlignItems::FLEX_END,
        A::Center => AlignItems::CENTER,
        A::Stretch => AlignItems::STRETCH,
        A::Baseline => AlignItems::BASELINE,
        _ => AlignItems::STRETCH,
    })
}

fn align_content(a: Option<A>) -> Option<AlignContent> {
    Some(match a? {
        A::Start => AlignContent::FLEX_START,
        A::End => AlignContent::FLEX_END,
        A::Center => AlignContent::CENTER,
        A::Stretch => AlignContent::STRETCH,
        A::SpaceBetween => AlignContent::SPACE_BETWEEN,
        A::SpaceAround => AlignContent::SPACE_AROUND,
        A::SpaceEvenly => AlignContent::SPACE_EVENLY,
        A::Baseline => AlignContent::FLEX_START,
    })
}

fn min_track(t: &Track) -> MinTrackSizingFunction {
    match t {
        Track::Pt(p) => MinTrackSizingFunction::length(*p),
        Track::Pct(p) => MinTrackSizingFunction::percent(p / 100.0),
        Track::MinContent => MinTrackSizingFunction::min_content(),
        Track::MaxContent => MinTrackSizingFunction::max_content(),
        Track::Fr(_) | Track::Auto => MinTrackSizingFunction::auto(),
        Track::MinMax(a, _) => min_track(a),
    }
}

fn max_track(t: &Track) -> MaxTrackSizingFunction {
    match t {
        Track::Pt(p) => MaxTrackSizingFunction::length(*p),
        Track::Pct(p) => MaxTrackSizingFunction::percent(p / 100.0),
        Track::Fr(f) => MaxTrackSizingFunction::fr(*f),
        Track::MinContent => MaxTrackSizingFunction::min_content(),
        Track::MaxContent => MaxTrackSizingFunction::max_content(),
        Track::Auto => MaxTrackSizingFunction::auto(),
        Track::MinMax(_, b) => max_track(b),
    }
}

fn track(t: &Track) -> TrackSizingFunction {
    taffy::MinMax {
        min: min_track(t),
        max: max_track(t),
    }
}

fn placement(p: (GridLine, GridLine)) -> taffy::Line<GridPlacement> {
    let one = |g: GridLine| match g {
        GridLine::Auto => GridPlacement::Auto,
        GridLine::Line(n) => GridPlacement::from_line_index(n),
        GridLine::Span(n) => GridPlacement::from_span(n),
    };
    taffy::Line {
        start: one(p.0),
        end: one(p.1),
    }
}

impl<'a> Layout<'a> {
    pub fn new(tree: &'a Tree, db: &'a FontDb) -> Layout<'a> {
        Layout {
            tree,
            db,
            atom_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Lay out a box at a definite width (its own `width` may narrow it),
    /// with its top-left at the origin.
    pub fn layout(&self, root: usize, width: f32) -> Frag {
        self.layout_with(
            root,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::MaxContent,
            },
            None,
        )
    }

    /// Lay out a box at exactly the page's size: a full-bleed background.
    pub fn layout_page(&self, root: usize, width: f32, height: f32) -> Frag {
        let mut taffy: TaffyTree<Leaf> = TaffyTree::new();
        taffy.disable_rounding();
        let node = self.build(&mut taffy, root, true);
        let mut st = taffy.style(node).cloned().unwrap_or_default();
        st.size = Size {
            width: Dimension::length(width),
            height: Dimension::length(height),
        };
        let _ = taffy.set_style(node, st);
        let _ = taffy.compute_layout_with_measure(
            node,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
            |inputs, _id, ctx, style| self.measure(inputs, ctx, style),
        );
        self.collect(&taffy, node, root, 0.0, 0.0)
    }

    /// Lay out a box as wide as its content: a watermark.
    pub fn layout_natural(&self, root: usize) -> Frag {
        self.layout_with(
            root,
            Size {
                width: AvailableSpace::MaxContent,
                height: AvailableSpace::MaxContent,
            },
            None,
        )
    }

    fn layout_with(
        &self,
        root: usize,
        available: Size<AvailableSpace>,
        force_width: Option<f32>,
    ) -> Frag {
        let mut taffy: TaffyTree<Leaf> = TaffyTree::new();
        taffy.disable_rounding();
        let node = self.build(&mut taffy, root, true);
        // A box laid out at a width fills it, as a block does — taffy would
        // size a flex or grid root to its content.
        let fill = force_width.or(match available.width {
            AvailableSpace::Definite(w) if self.tree.boxes[root].style.width == L::Auto => Some(w),
            _ => None,
        });
        if let Some(w) = fill {
            let mut st = taffy.style(node).cloned().unwrap_or_default();
            st.size.width = Dimension::length(w);
            let _ = taffy.set_style(node, st);
        }
        let _ = taffy.compute_layout_with_measure(node, available, |inputs, _id, ctx, style| {
            self.measure(inputs, ctx, style)
        });
        self.collect(&taffy, node, root, 0.0, 0.0)
    }

    fn style_of(&self, b: usize, root: bool) -> Style {
        let lb = &self.tree.boxes[b];
        let s: &S = &lb.style;
        let mut st = Style {
            display: match s.display {
                D::None => Display::None,
                D::Flex | D::InlineFlex => Display::Flex,
                D::Grid | D::InlineGrid | D::Table => Display::Grid,
                _ => Display::Block,
            },
            box_sizing: if s.border_box {
                BoxSizing::BorderBox
            } else {
                BoxSizing::ContentBox
            },
            position: match s.position {
                P::Absolute | P::Fixed => Position::Absolute,
                _ => Position::Relative,
            },
            inset: taffy::Rect {
                top: if s.position == P::Static {
                    LengthPercentageAuto::auto()
                } else {
                    lpa(s.inset[0])
                },
                right: if s.position == P::Static {
                    LengthPercentageAuto::auto()
                } else {
                    lpa(s.inset[1])
                },
                bottom: if s.position == P::Static {
                    LengthPercentageAuto::auto()
                } else {
                    lpa(s.inset[2])
                },
                left: if s.position == P::Static {
                    LengthPercentageAuto::auto()
                } else {
                    lpa(s.inset[3])
                },
            },
            size: Size {
                width: dim(s.width),
                height: dim(s.height),
            },
            min_size: Size {
                width: lpa(s.min_width),
                height: lpa(s.min_height),
            },
            max_size: Size {
                width: lpa(s.max_width),
                height: lpa(s.max_height),
            },
            aspect_ratio: s.aspect_ratio,
            margin: taffy::Rect {
                top: lpa(s.margin[0]),
                right: lpa(s.margin[1]),
                bottom: lpa(s.margin[2]),
                left: lpa(s.margin[3]),
            },
            padding: taffy::Rect {
                top: lp(s.padding[0]),
                right: lp(s.padding[1]),
                bottom: lp(s.padding[2]),
                left: lp(s.padding[3]),
            },
            border: taffy::Rect {
                top: LengthPercentage::length(if s.border[0].visible() {
                    s.border[0].width
                } else {
                    0.0
                }),
                right: LengthPercentage::length(if s.border[1].visible() {
                    s.border[1].width
                } else {
                    0.0
                }),
                bottom: LengthPercentage::length(if s.border[2].visible() {
                    s.border[2].width
                } else {
                    0.0
                }),
                left: LengthPercentage::length(if s.border[3].visible() {
                    s.border[3].width
                } else {
                    0.0
                }),
            },
            overflow: if s.clip {
                taffy::Point {
                    x: Overflow::Hidden,
                    y: Overflow::Hidden,
                }
            } else {
                taffy::Point {
                    x: Overflow::Visible,
                    y: Overflow::Visible,
                }
            },
            flex_direction: match s.flex_direction {
                FD::Row => FlexDirection::Row,
                FD::RowReverse => FlexDirection::RowReverse,
                FD::Column => FlexDirection::Column,
                FD::ColumnReverse => FlexDirection::ColumnReverse,
            },
            flex_wrap: if s.flex_wrap {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            },
            justify_content: align_content(s.justify_content),
            align_items: align_items(s.align_items),
            align_self: align_items(s.align_self),
            align_content: align_content(s.align_content),
            justify_items: align_items(s.justify_items),
            flex_grow: s.flex_grow,
            flex_shrink: s.flex_shrink,
            flex_basis: dim(s.flex_basis),
            gap: Size {
                width: lp(s.column_gap),
                height: lp(s.row_gap),
            },
            grid_template_columns: s
                .grid_columns
                .iter()
                .map(|t| GridTemplateComponent::Single(track(t)))
                .collect(),
            grid_template_rows: s
                .grid_rows
                .iter()
                .map(|t| GridTemplateComponent::Single(track(t)))
                .collect(),
            grid_auto_rows: s.grid_auto_rows.iter().map(track).collect(),
            grid_column: placement(s.grid_column),
            grid_row: placement(s.grid_row),
            ..Default::default()
        };
        // Right to left: taffy runs a flex row and a grid from the right
        // itself; a block's children are mirrored after layout (below).
        if s.direction == super::style::Direction::Rtl {
            st.direction = taffy::Direction::Rtl;
        }
        // A box laid out on its own is placed where it is asked to be.
        if root {
            st.position = Position::Relative;
            st.inset = taffy::Rect {
                top: LengthPercentageAuto::auto(),
                right: LengthPercentageAuto::auto(),
                bottom: LengthPercentageAuto::auto(),
                left: LengthPercentageAuto::auto(),
            };
            st.margin = taffy::Rect {
                top: LengthPercentageAuto::length(0.0),
                right: LengthPercentageAuto::length(0.0),
                bottom: LengthPercentageAuto::length(0.0),
                left: LengthPercentageAuto::length(0.0),
            };
        }
        match &lb.kind {
            Kind::Table { columns, .. } => {
                st.display = Display::Grid;
                st.grid_template_columns = (0..*columns)
                    .map(|_| {
                        GridTemplateComponent::Single(taffy::MinMax {
                            min: MinTrackSizingFunction::auto(),
                            max: MaxTrackSizingFunction::auto(),
                        })
                    })
                    .collect();
                st.gap = Size {
                    width: LengthPercentage::length(0.0),
                    height: LengthPercentage::length(0.0),
                };
                // A table is as wide as its content unless it says otherwise.
                if s.width == L::Auto && !root {
                    st.size.width = Dimension::auto();
                }
            }
            Kind::Cell {
                row,
                col,
                colspan,
                rowspan,
            } => {
                st.grid_row = taffy::Line {
                    start: GridPlacement::from_line_index(*row as i16 + 1),
                    end: GridPlacement::from_span(*rowspan as u16),
                };
                st.grid_column = taffy::Line {
                    start: GridPlacement::from_line_index(*col as i16 + 1),
                    end: GridPlacement::from_span(*colspan as u16),
                };
                // A cell fills its row, and centres its content in it.
                st.display = Display::Flex;
                st.flex_direction = FlexDirection::Column;
                st.align_self = Some(AlignItems::STRETCH);
                st.justify_content = Some(match s.vertical_align {
                    VerticalAlign::Middle => AlignContent::CENTER,
                    VerticalAlign::Bottom => AlignContent::FLEX_END,
                    _ => AlignContent::FLEX_START,
                });
            }
            Kind::Image { .. } => {
                // A picture keeps its own size: it is not stretched as a
                // block's child is.
                st.display = Display::Block;
                st.item_is_replaced = true;
            }
            Kind::Progress(_) => {
                st.display = Display::Block;
            }
            Kind::PageBreak => {
                st.size = Size {
                    width: Dimension::percent(1.0),
                    height: Dimension::length(0.0),
                };
            }
            _ => {}
        }
        st
    }

    fn build(&self, taffy: &mut TaffyTree<Leaf>, b: usize, root: bool) -> NodeId {
        let lb = &self.tree.boxes[b];
        let mut style = self.style_of(b, root);
        let leaf = match &lb.kind {
            Kind::Inline(ix) => Some(Leaf::Text(*ix)),
            Kind::Image { asset, .. } => {
                let (w, h) = match asset.map(|a| &self.tree.assets[a]) {
                    Some(Asset::Raster { width, height, .. }) => {
                        (*width as f32 * 0.75, *height as f32 * 0.75)
                    }
                    Some(Asset::Svg { tree }) => {
                        (tree.size().width() * 0.75, tree.size().height() * 0.75)
                    }
                    None => (0.0, 0.0),
                };
                if style.aspect_ratio.is_none() && w > 0.0 && h > 0.0 {
                    style.aspect_ratio = Some(w / h);
                }
                let compressible =
                    matches!(lb.style.width, L::Pct(_)) || matches!(lb.style.max_width, L::Pct(_));
                Some(Leaf::Image { w, h, compressible })
            }
            Kind::Progress(_) | Kind::PageBreak => Some(Leaf::Empty),
            _ => None,
        };
        if let Some(leaf) = leaf {
            return taffy.new_leaf_with_context(style, leaf).expect("a leaf");
        }
        let mut kids: Vec<usize> = lb.children.clone();
        kids.sort_by_key(|&c| self.tree.boxes[c].style.order);
        let children: Vec<NodeId> = kids.iter().map(|&c| self.build(taffy, c, false)).collect();
        // A block container with only floats of text and nothing else is a
        // plain block; one with block children too.
        if matches!(style.display, Display::Block) && children.is_empty() {
            style.size.height = match style.size.height {
                d if d.is_auto() => Dimension::auto(),
                d => d,
            };
        }
        taffy.new_with_children(style, &children).expect("a node")
    }

    fn measure(&self, inputs: LayoutInput, ctx: Option<&mut Leaf>, style: &Style) -> LayoutOutput {
        let leaf = ctx.copied().unwrap_or(Leaf::Empty);
        taffy::compute_leaf_layout(
            inputs,
            style,
            |_, _| 0.0,
            |known, avail| match leaf {
                Leaf::Empty => Size {
                    width: known.width.unwrap_or(0.0),
                    height: known.height.unwrap_or(0.0),
                },
                Leaf::Image { w, h, compressible } => {
                    let ratio = if h > 0.0 { w / h } else { 1.0 };
                    if compressible
                        && known.width.is_none()
                        && avail.width == AvailableSpace::MinContent
                    {
                        return Size {
                            width: 0.0,
                            height: 0.0,
                        };
                    }
                    match (known.width, known.height) {
                        (Some(kw), Some(kh)) => Size {
                            width: kw,
                            height: kh,
                        },
                        (Some(kw), None) => Size {
                            width: kw,
                            height: kw / ratio,
                        },
                        (None, Some(kh)) => Size {
                            width: kh * ratio,
                            height: kh,
                        },
                        (None, None) => {
                            // A picture never overflows the space it is given.
                            let max = match avail.width {
                                AvailableSpace::Definite(a) => a,
                                _ => f32::INFINITY,
                            };
                            let width = w.min(max);
                            Size {
                                width,
                                height: width / ratio,
                            }
                        }
                    }
                }
                Leaf::Text(ix) => {
                    let shaped = &self.tree.shaped[ix];
                    let mut sizer = |b: usize, a: f32| self.atom_size(b, a);
                    let width = known.width.unwrap_or_else(|| match avail.width {
                        AvailableSpace::Definite(w) => shaped.max_content(&mut sizer).min(w),
                        AvailableSpace::MinContent => shaped.min_content(&mut sizer),
                        AvailableSpace::MaxContent => shaped.max_content(&mut sizer),
                    });
                    let lines = shaped.lines(width.max(0.0) + 0.001, &mut sizer);
                    let height: f32 = lines.iter().map(|l| l.height).sum();
                    Size {
                        width,
                        height: known.height.unwrap_or(height),
                    }
                }
            },
        )
    }

    /// An inline-block's size at an available width: as wide as its content,
    /// no wider than the width, no narrower than its narrowest content.
    pub fn atom_size(&self, b: usize, avail: f32) -> (f32, f32, f32) {
        let key = (b, avail.to_bits());
        if let Some(v) = self.atom_cache.borrow().get(&key) {
            return *v;
        }
        let natural = self.layout_with(
            b,
            Size {
                width: AvailableSpace::MaxContent,
                height: AvailableSpace::MaxContent,
            },
            None,
        );
        let frag = if avail.is_finite() && natural.w > avail && avail > 0.0 {
            self.layout_with(
                b,
                Size {
                    width: AvailableSpace::Definite(avail),
                    height: AvailableSpace::MaxContent,
                },
                None,
            )
        } else {
            natural
        };
        let s = &self.tree.boxes[b].style;
        let margin_w = s.margin[1].or_zero(avail.max(0.0)) + s.margin[3].or_zero(avail.max(0.0));
        let baseline = frag.first_baseline().unwrap_or(frag.h);
        let v = (frag.w + margin_w, frag.h, baseline);
        self.atom_cache.borrow_mut().insert(key, v);
        v
    }

    fn collect(&self, taffy: &TaffyTree<Leaf>, node: NodeId, b: usize, ox: f32, oy: f32) -> Frag {
        let l = taffy.layout(node).expect("laid out");
        let x = ox + l.location.x;
        let y = oy + l.location.y;
        let lb = &self.tree.boxes[b];
        let mut frag = Frag {
            boxed: b,
            x,
            y,
            w: l.size.width,
            h: l.size.height,
            children: Vec::new(),
            lines: Vec::new(),
            atoms: Vec::new(),
            extent: (y, y + l.size.height),
        };
        match &lb.kind {
            Kind::Inline(ix) => {
                let shaped = &self.tree.shaped[*ix];
                let mut sizer = |bb: usize, a: f32| self.atom_size(bb, a);
                let content_w = l.size.width
                    - l.padding.left
                    - l.padding.right
                    - l.border.left
                    - l.border.right;
                frag.lines = shaped.lines(content_w.max(0.0) + 0.001, &mut sizer);
                for line in &frag.lines {
                    for a in &line.atoms {
                        let s = &self.tree.boxes[a.boxed].style;
                        let ml = s.margin[3].or_zero(content_w);
                        let mut af = self.layout_with(
                            a.boxed,
                            Size {
                                width: AvailableSpace::Definite(a.width),
                                height: AvailableSpace::MaxContent,
                            },
                            Some(a.width - ml - s.margin[1].or_zero(content_w)),
                        );
                        af.shift(x + a.x + ml, y + line.top + a.y);
                        frag.atoms.push(af);
                    }
                }
            }
            _ => {
                let mut kids: Vec<usize> = lb.children.clone();
                kids.sort_by_key(|&c| self.tree.boxes[c].style.order);
                let nodes = taffy.children(node).unwrap_or_default();
                for (child_node, child_box) in nodes.into_iter().zip(kids) {
                    if self.tree.boxes[child_box].style.display == D::None {
                        continue;
                    }
                    frag.children
                        .push(self.collect(taffy, child_node, child_box, x, y));
                }
            }
        }
        // Right to left, a block's children and a column's sit against its
        // right edge: taffy places them from the left, so they are mirrored
        // in the content box. (A grid mirrors itself; a row was reversed.)
        let s = &lb.style;
        let column = matches!(s.display, D::Flex | D::InlineFlex)
            && matches!(s.flex_direction, FD::Column | FD::ColumnReverse);
        let block = matches!(s.display, D::Block | D::ListItem | D::InlineBlock)
            || matches!(lb.kind, Kind::Cell { .. });
        if s.direction == super::style::Direction::Rtl
            && (block || column)
            && !frag.children.is_empty()
        {
            let left = x + l.border.left + l.padding.left;
            let width =
                l.size.width - l.border.left - l.border.right - l.padding.left - l.padding.right;
            for c in &mut frag.children {
                let pos = self.tree.boxes[c.boxed].style.position;
                if matches!(pos, P::Absolute | P::Fixed) {
                    continue;
                }
                let mirrored = left + width - (c.x - left) - c.w;
                let dx = mirrored - c.x;
                if dx.abs() > 0.01 {
                    c.shift(dx, 0.0);
                }
            }
        }
        // `position: relative` offsets and `transform: translate` move a box
        // without moving what is around it.
        let s = &lb.style;
        let (tx, ty) = (s.translate.0.or_zero(frag.w), s.translate.1.or_zero(frag.h));
        if tx != 0.0 || ty != 0.0 {
            frag.shift(tx, ty);
        }
        let _ = CompactLength::auto();
        frag
    }
}

#[cfg(test)]
mod tests {
    use super::super::boxes::Builder;
    use super::super::cascade::{Styler, UA_SHEET};
    use super::super::css::Sheet;
    use super::super::html::Dom;
    use super::super::style::Units;
    use super::*;

    fn lay(html: &str, css: &str, width: f32) -> (Tree, Frag) {
        let dom = Dom::parse(html);
        let styler = Styler::new(
            &Sheet::parse(UA_SHEET),
            &Sheet::parse(css),
            Units {
                em: 12.0,
                rem: 12.0,
                vw: width,
                vh: 800.0,
            },
        );
        let mut db = FontDb::new(false);
        let read = |_: &str| -> Option<Vec<u8>> { None };
        let tree = Builder::new(&dom, &styler, &mut db, &read).build(&S::root(12.0, "sans-serif"));
        let frag = Layout::new(&tree, &db).layout(tree.flow, width);
        (tree, frag)
    }

    #[test]
    fn a_row_puts_its_children_side_by_side() {
        let (_, f) = lay(
            r#"<body><div class="row"><div class="c">One</div><div class="c">Two</div><div class="c">Three</div></div></body>"#,
            ".row { display: flex; gap: 12pt } .c { flex: 1; padding: 6pt; border: 1pt solid #ccc }",
            400.0,
        );
        let row = &f.children[0];
        let xs: Vec<f32> = row.children.iter().map(|c| c.x).collect();
        let ws: Vec<f32> = row.children.iter().map(|c| c.w).collect();
        assert_eq!(row.children.len(), 3);
        // Three equal columns with two 12pt gaps across 400pt.
        assert!((ws[0] - (400.0 - 24.0) / 3.0).abs() < 0.5, "{ws:?}");
        assert!((xs[1] - (ws[0] + 12.0)).abs() < 0.5);
        // All on one line.
        assert!(row.children.iter().all(|c| (c.y - row.y).abs() < 0.01));
    }

    #[test]
    fn right_to_left_a_row_starts_at_the_right_once() {
        // taffy runs a row from the right under `direction: rtl`; the row
        // must not be reversed again on top of that.
        let (_, f) = lay(
            r#"<body dir="rtl"><div class="row"><p>one</p><p>two</p><p>three</p></div><div class="rev"><p>a</p><p>b</p></div></body>"#,
            ".row { display: flex; width: 300pt } .rev { display: flex; flex-direction: row-reverse; width: 300pt } p { margin: 0; width: 50pt }",
            300.0,
        );
        let xs: Vec<f32> = f.children[0].children.iter().map(|c| c.x).collect();
        assert!(
            xs[0] > xs[1] && xs[1] > xs[2],
            "the first child is rightmost: {xs:?}"
        );
        assert!(
            (xs[0] - 250.0).abs() < 0.5,
            "against the right edge: {xs:?}"
        );
        let rev: Vec<f32> = f.children[1].children.iter().map(|c| c.x).collect();
        assert!(rev[0] < rev[1], "row-reverse runs from the left: {rev:?}");
    }

    #[test]
    fn a_grid_lays_out_columns_and_wraps_rows() {
        let (_, f) = lay(
            r#"<body><div class="g"><p>a</p><p>b</p><p>c</p><p>d</p></div></body>"#,
            ".g { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10pt } p { margin: 0 }",
            300.0,
        );
        let g = &f.children[0];
        assert_eq!(g.children.len(), 4);
        assert!((g.children[1].x - g.children[0].x - (280.0 / 3.0 + 10.0)).abs() < 0.5);
        assert!(g.children[3].y > g.children[0].y);
        assert!((g.children[3].x - g.children[0].x).abs() < 0.01);
    }

    #[test]
    fn text_wraps_to_the_width_and_sets_the_height() {
        let (_, f) = lay(
            "<body><p>The quick brown fox jumps over the lazy dog, again and again, until the line has to wrap.</p></body>",
            "p { margin: 0; line-height: 1.5 }",
            150.0,
        );
        let p = &f.children[0];
        let text = &p.children[0];
        assert!(text.lines.len() >= 3);
        assert!(
            (text.h - text.lines.len() as f32 * 18.0).abs() < 0.5,
            "{} lines, {}",
            text.lines.len(),
            text.h
        );
        assert!((p.h - text.h).abs() < 0.01);
    }

    #[test]
    fn padding_border_and_margins_place_the_content() {
        let (_, f) = lay(
            r#"<body><div class="card"><p>x</p></div><div class="next">y</div></body>"#,
            ".card { padding: 10pt; border: 2pt solid #000; margin-bottom: 20pt } p { margin: 0 }",
            300.0,
        );
        let card = &f.children[0];
        let p = &card.children[0];
        assert!((p.x - 12.0).abs() < 0.01 && (p.y - 12.0).abs() < 0.01);
        let next = &f.children[1];
        assert!((next.y - (card.h + 20.0)).abs() < 0.01);
    }

    #[test]
    fn an_inline_block_sits_on_the_line_as_wide_as_its_content() {
        let (_, f) = lay(
            r#"<body><p>Status <span class="pill">Paid</span> today</p></body>"#,
            ".pill { display: inline-flex; padding: 2pt 6pt; background: #0a0 } p { margin: 0 }",
            400.0,
        );
        let text = &f.children[0].children[0];
        assert_eq!(text.atoms.len(), 1);
        let pill = &text.atoms[0];
        // "Paid" at 12pt is about 24pt; plus 12pt of padding.
        assert!(pill.w > 25.0 && pill.w < 50.0, "{}", pill.w);
        assert!(pill.x > text.x + 20.0);
    }

    #[test]
    fn a_table_sizes_columns_to_their_content() {
        let (_, f) = lay(
            "<body><table style=\"width: 100%\"><tr><td>#</td><td>A much longer description of the line item</td><td>$1</td></tr></table></body>",
            "td { padding: 4pt }",
            400.0,
        );
        let table = &f.children[0];
        assert_eq!(table.children.len(), 3);
        let w: Vec<f32> = table.children.iter().map(|c| c.w).collect();
        // The description takes most of the width; the number column little.
        assert!(w[1] > w[0] * 3.0, "{w:?}");
        assert!((w.iter().sum::<f32>() - 400.0).abs() < 1.0, "{w:?}");
    }
}
