//! Painting: fragments drawn onto a krilla surface.
//!
//! Each box paints in CSS's order — its shadows, its background, its border,
//! then its content (text, list marker, children, inline-blocks) — inside its
//! opacity, rotation and clip. Text is drawn as the glyphs the text engine
//! shaped, with the source text attached so a reader can copy and search it.

use std::collections::HashMap;

use krilla::color::rgb;
use krilla::geom::{Path, PathBuilder, Point, Rect, Size, Transform};
use krilla::num::NormalizedF32;
use krilla::paint::{
    Fill, FillRule, LineCap, LinearGradient, RadialGradient, SpreadMethod, Stop, Stroke, StrokeDash,
};
use krilla::surface::Surface;
use krilla::tagging::{Artifact, ArtifactType, ContentTag, Identifier, SpanTag};
use krilla::text::{Font, GlyphId, KrillaGlyph};
use krilla_svg::{SurfaceExt, SvgSettings};

use super::boxes::{Asset, Kind, Tree};
use super::fonts::{FontDb, Pick};
use super::layout::Frag;
use super::style::{
    BorderStyle, GradientStop, Image as BgImage, Length, ObjectFit, Position, Rgba, Style,
};
use super::text::{Item, Line};

/// A link drawn on a page: where, to what, and — for a tagged PDF — the
/// element it belongs to and where it falls in the reading order.
#[derive(Debug)]
pub struct LinkBox {
    pub rect: (f32, f32, f32, f32),
    pub href: String,
    pub node: Option<usize>,
    pub seq: u64,
}

/// A piece of tagged content: the element it belongs to, its place in the
/// order it was drawn (which is the reading order within an element), and
/// whether it is a list item's marker.
#[derive(Debug)]
pub struct Leaf {
    pub node: usize,
    pub seq: u64,
    pub label: bool,
    pub id: Identifier,
}

/// What the content being drawn is marked as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Artifact,
    Content { node: usize, label: bool, image: bool },
}

#[derive(Debug)]
pub struct Painter<'a> {
    /// Whether what is drawn is tagged: content as a leaf of its element,
    /// decoration as an artifact.
    pub tagging: bool,
    /// Inside an artifact the caller opened — a running header, a fixed
    /// box, a repeated table header — nothing more is marked.
    pub artifact: bool,
    /// The tagged content drawn so far, every page.
    pub leaves: Vec<Leaf>,
    seq: u64,
    /// The elements being painted, innermost last: what content belongs to.
    owner: Vec<usize>,
    /// Painting a list item's marker.
    label: bool,
    /// The mark open on the surface, if any.
    open: Option<Mark>,
    /// The tree being painted: the document's, or a running element's.
    pub tree: &'a Tree,
    pub db: &'a FontDb,
    fonts: HashMap<(usize, u16), Option<Font>>,
    images: HashMap<usize, Option<krilla::image::Image>>,
    /// The links painted on the current page, in page coordinates.
    pub links: Vec<LinkBox>,
    /// What maps the strip onto the current page: `page_y = strip_y + dy`.
    pub dx: f32,
    pub dy: f32,
    /// The part of the strip the current page shows.
    pub band: (f32, f32),
    /// The text drawn on the current page, a line at a time, in reading
    /// order: what a test or a reader of the build can check.
    pub text: Vec<String>,
}

fn rgb_of(c: Rgba) -> rgb::Color {
    rgb::Color::new(
        (c.r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.b.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn paint(c: Rgba) -> krilla::paint::Paint {
    rgb_of(c).into()
}

fn opacity(a: f32) -> NormalizedF32 {
    NormalizedF32::new(a.clamp(0.0, 1.0)).unwrap_or(NormalizedF32::ONE)
}

fn fill(c: Rgba) -> Fill {
    Fill {
        paint: paint(c),
        opacity: opacity(c.a),
        rule: FillRule::NonZero,
    }
}

/// A rectangle with each corner rounded: top-left, top-right, bottom-right,
/// bottom-left.
pub fn rounded(x: f32, y: f32, w: f32, h: f32, r: [f32; 4]) -> Option<Path> {
    const K: f32 = 0.552_284_8;

    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let max = (w.min(h)) / 2.0;
    let r = r.map(|v| v.clamp(0.0, max));
    let mut pb = PathBuilder::new();
    if r.iter().all(|v| *v < 0.01) {
        pb.push_rect(Rect::from_xywh(x, y, w, h)?);
        return pb.finish();
    }
    pb.move_to(x + r[0], y);
    pb.line_to(x + w - r[1], y);
    pb.cubic_to(
        x + w - r[1] + r[1] * K,
        y,
        x + w,
        y + r[1] - r[1] * K,
        x + w,
        y + r[1],
    );
    pb.line_to(x + w, y + h - r[2]);
    pb.cubic_to(
        x + w,
        y + h - r[2] + r[2] * K,
        x + w - r[2] + r[2] * K,
        y + h,
        x + w - r[2],
        y + h,
    );
    pb.line_to(x + r[3], y + h);
    pb.cubic_to(
        x + r[3] - r[3] * K,
        y + h,
        x,
        y + h - r[3] + r[3] * K,
        x,
        y + h - r[3],
    );
    pb.line_to(x, y + r[0]);
    pb.cubic_to(x, y + r[0] - r[0] * K, x + r[0] - r[0] * K, y, x + r[0], y);
    pb.close();
    pb.finish()
}

fn line_path(x1: f32, y1: f32, x2: f32, y2: f32) -> Option<Path> {
    let mut pb = PathBuilder::new();
    pb.move_to(x1, y1);
    pb.line_to(x2, y2);
    pb.finish()
}

/// The resolved corner radii of a box.
pub fn radii(s: &Style, w: f32, h: f32) -> [f32; 4] {
    s.radius.map(|r| match r {
        Length::Pt(p) => p,
        Length::Pct(p) => w.min(h) * p / 100.0,
        Length::Mix(v, p) => v + w.min(h) * p / 100.0,
        Length::Auto => 0.0,
    })
}

/// Stops with their positions filled in, as CSS spreads unpositioned ones.
fn stops(list: &[GradientStop]) -> Vec<Stop> {
    let n = list.len();
    let mut at: Vec<Option<f32>> = list.iter().map(|s| s.at).collect();
    if at[0].is_none() {
        at[0] = Some(0.0);
    }
    if at[n - 1].is_none() {
        at[n - 1] = Some(1.0);
    }
    let mut i = 0;
    while i < n {
        if at[i].is_none() {
            let start = i - 1;
            let mut end = i;
            while at[end].is_none() {
                end += 1;
            }
            // The first and the last stop are always measured, so a gap is
            // bounded by measured stops on both sides.
            let measured = "a gap lies between measured stops";
            let (a, b) = (at[start].expect(measured), at[end].expect(measured));
            for (k, slot) in at.iter_mut().enumerate().take(end).skip(i) {
                *slot = Some(a + (b - a) * (k - start) as f32 / (end - start) as f32);
            }
            i = end;
        }
        i += 1;
    }
    let mut last = 0.0f32;
    list.iter()
        .zip(at)
        .map(|(s, a)| {
            let pos = a.unwrap_or(0.0).max(last).clamp(0.0, 1.0);
            last = pos;
            Stop {
                offset: NormalizedF32::new(pos).unwrap_or(NormalizedF32::ZERO),
                color: rgb_of(s.color).into(),
                opacity: opacity(s.color.a),
            }
        })
        .collect()
}

impl<'a> Painter<'a> {
    pub fn new(tree: &'a Tree, db: &'a FontDb) -> Painter<'a> {
        Painter {
            tagging: false,
            artifact: false,
            leaves: Vec::new(),
            seq: 0,
            owner: Vec::new(),
            label: false,
            open: None,
            tree,
            db,
            fonts: HashMap::new(),
            images: HashMap::new(),
            links: Vec::new(),
            dx: 0.0,
            dy: 0.0,
            band: (f32::MIN, f32::MAX),
            text: Vec::new(),
        }
    }

    /// The next place in the order things are drawn.
    pub fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    /// Mark what is drawn next as an artifact — decoration, no part of what
    /// the document says.
    fn mark_artifact(&mut self, s: &mut Surface<'_>) {
        self.mark(s, Mark::Artifact);
    }

    /// Mark what is drawn next as content of `node` (else of the element
    /// being painted), recorded as a leaf of it.
    fn mark_content(&mut self, s: &mut Surface<'_>, node: Option<usize>, image: bool) {
        match node.or_else(|| self.owner.last().copied()) {
            Some(node) => self.mark(
                s,
                Mark::Content {
                    node,
                    label: self.label,
                    image,
                },
            ),
            None => self.mark(s, Mark::Artifact),
        }
    }

    /// Go on with the mark that is open when it is the one wanted — the runs
    /// of a line in one element are one piece of content — or close it and
    /// open that one.
    fn mark(&mut self, s: &mut Surface<'_>, want: Mark) {
        if !self.tagging || self.artifact || self.open == Some(want) {
            return;
        }
        self.close_mark(s);
        match want {
            Mark::Artifact => {
                s.start_tagged(ContentTag::Artifact(Artifact::new(
                    ArtifactType::Layout,
                    None,
                )));
            }
            Mark::Content { node, label, image } => {
                let id = s.start_tagged(if image {
                    ContentTag::Other
                } else {
                    ContentTag::Span(SpanTag::empty())
                });
                let seq = self.next_seq();
                self.leaves.push(Leaf {
                    node,
                    seq,
                    label,
                    id,
                });
            }
        }
        self.open = Some(want);
    }

    /// Close the open mark: before the graphics state is pushed or popped,
    /// which a marked sequence may not straddle, and when painting stops.
    pub fn close_mark(&mut self, s: &mut Surface<'_>) {
        if self.open.take().is_some() {
            s.end_tagged();
        }
    }

    fn font(&mut self, pick: Pick) -> Option<Font> {
        let db = self.db;
        self.fonts
            .entry((pick.face, pick.weight))
            .or_insert_with(|| {
                let face = &db.faces[pick.face];
                let data: krilla::Data = std::sync::Arc::clone(&face.data).into();
                if face.variable {
                    Font::new_variable(
                        data,
                        face.index,
                        &[(krilla::text::Tag::new(b"wght"), f32::from(pick.weight))],
                    )
                } else {
                    Font::new(data, face.index)
                }
            })
            .clone()
    }

    fn image(&mut self, asset: usize) -> Option<krilla::image::Image> {
        let tree = self.tree;
        self.images
            .entry(asset)
            .or_insert_with(|| match &tree.assets[asset] {
                Asset::Raster { data, .. } => {
                    let d: krilla::Data = data.clone().into();
                    if data.starts_with(&[0xFF, 0xD8]) {
                        krilla::image::Image::from_jpeg(d, true).ok()
                    } else if data.starts_with(b"\x89PNG") {
                        krilla::image::Image::from_png(d, true).ok()
                    } else {
                        let img = image::load_from_memory(data).ok()?.to_rgba8();
                        let (w, h) = img.dimensions();
                        Some(krilla::image::Image::from_rgba8(img.into_raw(), w, h))
                    }
                }
                Asset::Svg { .. } => None,
            })
            .clone()
    }

    /// Paint a fragment and everything in it, leaving no mark open.
    pub fn frag(&mut self, s: &mut Surface<'_>, f: &Frag) {
        self.paint_frag(s, f);
        self.close_mark(s);
    }

    fn paint_frag(&mut self, s: &mut Surface<'_>, f: &Frag) {
        if f.extent.1 <= self.band.0 + 0.01 || f.extent.0 >= self.band.1 - 0.01 {
            return;
        }
        let tree = self.tree;
        let lb = &tree.boxes[f.boxed];
        let style = &lb.style;
        if !style.visible && f.children.is_empty() {
            return;
        }
        let mut pushed = 0;
        self.close_mark(s);
        if style.opacity < 0.999 {
            s.push_opacity(opacity(style.opacity));
            pushed += 1;
        }
        if style.rotate.abs() > 0.01 || (style.scale - 1.0).abs() > 0.001 {
            let (cx, cy) = (f.x + f.w / 2.0, f.y + f.h / 2.0);
            let t = Transform::from_translate(cx, cy);
            s.push_transform(&t);
            s.push_transform(&Transform::from_rotate(style.rotate));
            s.push_transform(&Transform::from_scale(style.scale, style.scale));
            s.push_transform(&Transform::from_translate(-cx, -cy));
            pushed += 4;
        }
        if let Some(n) = lb.node {
            self.owner.push(n);
        }
        if style.visible {
            let draws = style.background_color.is_visible()
                || !style.background_images.is_empty()
                || !style.shadows.is_empty()
                || style.border.iter().any(super::style::BorderSide::visible);
            if draws {
                self.mark_artifact(s);
                self.decoration(s, f, style);
            }
        }
        match &lb.kind {
            Kind::Image { asset, alt } => {
                // A picture with an empty `alt` (an icon beside its words) is
                // decoration.
                if alt.trim().is_empty() {
                    self.mark_artifact(s);
                } else {
                    self.mark_content(s, lb.node, true);
                }
                self.picture(s, f, *asset, style);
            }
            Kind::Progress(frac) => {
                self.mark_artifact(s);
                self.progress(s, f, *frac, style);
            }
            _ => {}
        }
        let clip = style.clip && !matches!(lb.kind, Kind::Inline(_));
        if clip && let Some(p) = rounded(f.x, f.y, f.w, f.h, radii(style, f.w, f.h)) {
            self.close_mark(s);
            s.push_clip_path(&p, &FillRule::NonZero);
            pushed += 1;
        }
        if let Some(m) = lb.marker {
            self.marker(s, f, m);
        }
        if let Kind::Inline(ix) = lb.kind {
            self.lines(s, f, ix);
        }
        // Positioned children paint above the rest.
        for c in f
            .children
            .iter()
            .filter(|c| tree.boxes[c.boxed].style.position == Position::Static)
        {
            self.paint_frag(s, c);
        }
        for c in f
            .children
            .iter()
            .filter(|c| {
                let p = tree.boxes[c.boxed].style.position;
                // A fixed box is drawn on its own, on every page.
                p != Position::Static && p != Position::Fixed
            })
        {
            self.paint_frag(s, c);
        }
        for a in f
            .atoms
            .iter()
            .filter(|a| tree.boxes[a.boxed].style.position != Position::Fixed)
        {
            self.paint_frag(s, a);
        }
        if pushed > 0 {
            self.close_mark(s);
        }
        for _ in 0..pushed {
            s.pop();
        }
        if lb.node.is_some() {
            self.owner.pop();
        }
    }

    /// Shadows, background and border.
    fn decoration(&mut self, s: &mut Surface<'_>, f: &Frag, style: &Style) {
        let r = radii(style, f.w, f.h);
        for sh in style
            .shadows
            .iter()
            .rev()
            .filter(|sh| !sh.inset && sh.color.a > 0.0)
        {
            let steps = (sh.blur / 1.5).clamp(1.0, 10.0) as usize;
            for i in 0..steps {
                let t = (i as f32 + 0.5) / steps as f32;
                let grow = sh.spread - sh.blur / 2.0 + sh.blur * t;
                let alpha = if steps == 1 {
                    sh.color.a
                } else {
                    sh.color.a * 2.0 * (1.0 - t) / steps as f32
                };
                if let Some(p) = rounded(
                    f.x + sh.x - grow,
                    f.y + sh.y - grow,
                    f.w + 2.0 * grow,
                    f.h + 2.0 * grow,
                    r.map(|v| (v + grow).max(0.0)),
                ) {
                    s.set_stroke(None);
                    s.set_fill(Some(fill(Rgba {
                        a: alpha,
                        ..sh.color
                    })));
                    s.draw_path(&p);
                }
            }
        }
        let Some(path) = rounded(f.x, f.y, f.w, f.h, r) else {
            return;
        };
        if style.background_color.is_visible() {
            s.set_stroke(None);
            s.set_fill(Some(fill(style.background_color)));
            s.draw_path(&path);
        }
        for img in style.background_images.iter().rev() {
            match img {
                BgImage::Linear { angle, stops: st } => {
                    let a = angle.to_radians();
                    let (dx, dy) = (a.sin(), -a.cos());
                    let len = (f.w * a.sin()).abs() + (f.h * a.cos()).abs();
                    let (cx, cy) = (f.x + f.w / 2.0, f.y + f.h / 2.0);
                    let g = LinearGradient {
                        x1: cx - dx * len / 2.0,
                        y1: cy - dy * len / 2.0,
                        x2: cx + dx * len / 2.0,
                        y2: cy + dy * len / 2.0,
                        transform: Transform::default(),
                        spread_method: SpreadMethod::Pad,
                        stops: stops(st),
                        anti_alias: true,
                    };
                    s.set_stroke(None);
                    s.set_fill(Some(Fill {
                        paint: g.into(),
                        opacity: NormalizedF32::ONE,
                        rule: FillRule::NonZero,
                    }));
                    s.draw_path(&path);
                }
                BgImage::Radial { stops: st } => {
                    let (cx, cy) = (f.x + f.w / 2.0, f.y + f.h / 2.0);
                    let r = (f.w * f.w + f.h * f.h).sqrt() / 2.0;
                    let g = RadialGradient {
                        fx: cx,
                        fy: cy,
                        fr: 0.0,
                        cx,
                        cy,
                        cr: r,
                        transform: Transform::default(),
                        spread_method: SpreadMethod::Pad,
                        stops: stops(st),
                        anti_alias: true,
                    };
                    s.set_stroke(None);
                    s.set_fill(Some(Fill {
                        paint: g.into(),
                        opacity: NormalizedF32::ONE,
                        rule: FillRule::NonZero,
                    }));
                    s.draw_path(&path);
                }
                BgImage::Url(url) => {
                    let Some(&asset) = self.tree.backgrounds.get(url) else {
                        continue;
                    };
                    // `cover` fills the box, cropping what overflows; otherwise
                    // the picture at its own size from the top-left, clipped.
                    let (iw, ih) = match &self.tree.assets[asset] {
                        Asset::Raster { width, height, .. } => {
                            (*width as f32 * 0.75, *height as f32 * 0.75)
                        }
                        Asset::Svg { tree } => {
                            (tree.size().width() * 0.75, tree.size().height() * 0.75)
                        }
                    };
                    if iw <= 0.0 || ih <= 0.0 {
                        continue;
                    }
                    let (dw, dh) = if style.background_size_cover {
                        let k = (f.w / iw).max(f.h / ih);
                        (iw * k, ih * k)
                    } else {
                        (iw, ih)
                    };
                    let (ox, oy) = if style.background_size_cover {
                        (f.x + (f.w - dw) / 2.0, f.y + (f.h - dh) / 2.0)
                    } else {
                        (f.x, f.y)
                    };
                    s.push_clip_path(&path, &FillRule::NonZero);
                    s.push_transform(&Transform::from_translate(ox, oy));
                    match &self.tree.assets[asset] {
                        Asset::Svg { tree } => {
                            if let Some(size) = Size::from_wh(dw, dh) {
                                let _ = s.draw_svg(tree, size, SvgSettings::default());
                            }
                        }
                        Asset::Raster { .. } => {
                            if let (Some(img), Some(size)) =
                                (self.image(asset), Size::from_wh(dw, dh))
                            {
                                s.draw_image(img, size);
                            }
                        }
                    }
                    s.pop();
                    s.pop();
                }
            }
        }
        self.border(s, f, style, r);
    }

    #[expect(clippy::float_cmp, reason = "two sides drawn alike only when their widths are the same number")]
    fn border(&mut self, s: &mut Surface<'_>, f: &Frag, style: &Style, r: [f32; 4]) {
        let b = &style.border;
        if !b.iter().any(super::style::BorderSide::visible) {
            return;
        }
        let uniform = b.iter().all(|side| {
            side.width == b[0].width && side.style == b[0].style && side.color == b[0].color
        }) && b[0].visible();
        let dash = |side: &super::style::BorderSide| match side.style {
            BorderStyle::Dashed => Some(StrokeDash {
                array: vec![side.width * 3.0, side.width * 2.0],
                offset: 0.0,
            }),
            BorderStyle::Dotted => Some(StrokeDash {
                array: vec![0.0, side.width * 2.0],
                offset: 0.0,
            }),
            _ => None,
        };
        if uniform {
            let w = b[0].width;
            let Some(p) = rounded(
                f.x + w / 2.0,
                f.y + w / 2.0,
                f.w - w,
                f.h - w,
                r.map(|v| (v - w / 2.0).max(0.0)),
            ) else {
                return;
            };
            s.set_fill(None);
            s.set_stroke(Some(Stroke {
                paint: paint(style.border_color(0)),
                width: w,
                opacity: opacity(style.border_color(0).a),
                line_cap: if b[0].style == BorderStyle::Dotted {
                    LineCap::Round
                } else {
                    LineCap::Butt
                },
                dash: dash(&b[0]),
                ..Default::default()
            }));
            s.draw_path(&p);
            s.set_stroke(None);
            return;
        }
        // Each side on its own: a line down the middle of its width.
        let sides = [
            (
                f.x,
                f.y + b[0].width / 2.0,
                f.x + f.w,
                f.y + b[0].width / 2.0,
            ),
            (
                f.x + f.w - b[1].width / 2.0,
                f.y,
                f.x + f.w - b[1].width / 2.0,
                f.y + f.h,
            ),
            (
                f.x,
                f.y + f.h - b[2].width / 2.0,
                f.x + f.w,
                f.y + f.h - b[2].width / 2.0,
            ),
            (
                f.x + b[3].width / 2.0,
                f.y,
                f.x + b[3].width / 2.0,
                f.y + f.h,
            ),
        ];
        for (i, (x1, y1, x2, y2)) in sides.into_iter().enumerate() {
            if !b[i].visible() {
                continue;
            }
            if let Some(p) = line_path(x1, y1, x2, y2) {
                s.set_fill(None);
                s.set_stroke(Some(Stroke {
                    paint: paint(style.border_color(i)),
                    width: b[i].width,
                    opacity: opacity(style.border_color(i).a),
                    line_cap: if b[i].style == BorderStyle::Dotted {
                        LineCap::Round
                    } else {
                        LineCap::Butt
                    },
                    dash: dash(&b[i]),
                    ..Default::default()
                }));
                s.draw_path(&p);
            }
        }
        s.set_stroke(None);
    }

    fn content_box(f: &Frag, style: &Style) -> (f32, f32, f32, f32) {
        let pl = style.padding[3].or_zero(f.w)
            + if style.border[3].visible() {
                style.border[3].width
            } else {
                0.0
            };
        let pr = style.padding[1].or_zero(f.w)
            + if style.border[1].visible() {
                style.border[1].width
            } else {
                0.0
            };
        let pt = style.padding[0].or_zero(f.w)
            + if style.border[0].visible() {
                style.border[0].width
            } else {
                0.0
            };
        let pb = style.padding[2].or_zero(f.w)
            + if style.border[2].visible() {
                style.border[2].width
            } else {
                0.0
            };
        (
            f.x + pl,
            f.y + pt,
            (f.w - pl - pr).max(0.0),
            (f.h - pt - pb).max(0.0),
        )
    }

    fn picture(&mut self, s: &mut Surface<'_>, f: &Frag, asset: Option<usize>, style: &Style) {
        let Some(asset) = asset else {
            return;
        };
        let (x, y, w, h) = Self::content_box(f, style);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let (iw, ih) = match &self.tree.assets[asset] {
            Asset::Raster { width, height, .. } => (*width as f32, *height as f32),
            Asset::Svg { tree } => (tree.size().width(), tree.size().height()),
        };
        // `object-fit`: where in the box the picture goes, and how large.
        let (dw, dh) = match style.object_fit {
            ObjectFit::Contain | ObjectFit::ScaleDown => {
                let k = (w / iw).min(h / ih);
                (iw * k, ih * k)
            }
            ObjectFit::Cover => {
                let k = (w / iw).max(h / ih);
                (iw * k, ih * k)
            }
            ObjectFit::None => (iw * 0.75, ih * 0.75),
            ObjectFit::Fill => (w, h),
        };
        let (ox, oy) = (x + (w - dw) / 2.0, y + (h - dh) / 2.0);
        let r = radii(style, f.w, f.h);
        let clip = style.object_fit == ObjectFit::Cover || r.iter().any(|v| *v > 0.0);
        if clip && let Some(p) = rounded(x, y, w, h, r) {
            s.push_clip_path(&p, &FillRule::NonZero);
        }
        s.push_transform(&Transform::from_translate(ox, oy));
        match &self.tree.assets[asset] {
            Asset::Svg { tree } => {
                if let Some(size) = Size::from_wh(dw, dh) {
                    let _ = s.draw_svg(tree, size, SvgSettings::default());
                }
            }
            Asset::Raster { .. } => {
                if let (Some(img), Some(size)) = (self.image(asset), Size::from_wh(dw, dh)) {
                    s.draw_image(img, size);
                }
            }
        }
        s.pop();
        if clip {
            s.pop();
        }
    }

    fn progress(&mut self, s: &mut Surface<'_>, f: &Frag, frac: f32, style: &Style) {
        let r = f.h / 2.0;
        let track = if style.background_color.is_visible() {
            style.background_color
        } else {
            Rgba::rgb(0.9, 0.91, 0.93)
        };
        let bar = style
            .vars
            .get("--color-primary")
            .and_then(|v| super::style::color(v))
            .unwrap_or(Rgba::rgb(0.23, 0.51, 0.96));
        s.set_stroke(None);
        if let Some(p) = rounded(f.x, f.y, f.w, f.h, [r; 4]) {
            s.set_fill(Some(fill(track)));
            s.draw_path(&p);
        }
        if frac > 0.0
            && let Some(p) = rounded(f.x, f.y, f.w * frac, f.h, [r; 4])
        {
            s.set_fill(Some(fill(bar)));
            s.draw_path(&p);
        }
    }

    fn marker(&mut self, s: &mut Surface<'_>, f: &Frag, m: usize) {
        let shaped = &self.tree.shaped[m];
        let mut none = |_: usize, _: f32| (0.0, 0.0, 0.0);
        let lines = shaped.lines(f32::INFINITY, &mut none);
        let Some(line) = lines.first() else {
            return;
        };
        let style = &self.tree.boxes[f.boxed].style;
        let (cx, cy, _, _) = Self::content_box(f, style);
        let baseline = f.first_baseline().map_or(cy + line.baseline, |b| f.y + b);
        // Outside the item, on the side its text starts from.
        let (_, _, cw, _) = Self::content_box(f, style);
        let x = if style.direction == super::style::Direction::Rtl {
            cx + cw + style.font_size * 0.5
        } else {
            cx - line.width - style.font_size * 0.5
        };
        self.label = true;
        self.runs(s, shaped, line, x, baseline - line.baseline);
        self.label = false;
    }

    fn lines(&mut self, s: &mut Surface<'_>, f: &Frag, ix: usize) {
        let shaped = &self.tree.shaped[ix];
        for line in &f.lines {
            let top = f.y + line.top;
            if top >= self.band.1 - 0.01 || top + line.height <= self.band.0 + 0.01 {
                continue;
            }
            self.spans(s, shaped, line, f.x, top);
            self.runs(s, shaped, line, f.x, top);
            let start = line
                .runs
                .iter()
                .flat_map(|r| r.glyphs.iter().map(|g| g.range.start))
                .min();
            let end = line
                .runs
                .iter()
                .flat_map(|r| r.glyphs.iter().map(|g| g.range.end))
                .max();
            if let (Some(a), Some(b)) = (start, end) {
                let t = shaped.text[a..b].replace(super::text::ATOM, " ");
                let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
                if !t.is_empty() {
                    self.text.push(t);
                }
            }
        }
    }

    /// Inline elements' backgrounds and borders, and their links.
    fn spans(
        &mut self,
        s: &mut Surface<'_>,
        shaped: &super::text::Shaped,
        line: &Line,
        x0: f32,
        top: f32,
    ) {
        for ext in &line.spans {
            let span = &self.tree.spans[ext.span];
            let st = &span.style;
            let fs = st.font_size;
            let pl = st.padding[3].or_zero(0.0);
            let pr = st.padding[1].or_zero(0.0);
            let pt = st.padding[0].or_zero(0.0);
            let pb = st.padding[2].or_zero(0.0);
            let base = top + line.baseline;
            let (x, y, w, h) = (
                x0 + ext.x - pl,
                base - fs * 0.9 - pt,
                ext.width + pl + pr,
                fs * 1.15 + pt + pb,
            );
            let decorated = st.background_color.is_visible()
                || st.border.iter().any(super::style::BorderSide::visible);
            if decorated {
                self.mark_artifact(s);
            }
            if st.background_color.is_visible()
                && let Some(p) = rounded(x, y, w, h, radii(st, w, h))
            {
                s.set_stroke(None);
                s.set_fill(Some(fill(st.background_color)));
                s.draw_path(&p);
            }
            if st.border.iter().any(super::style::BorderSide::visible) {
                let fr = Frag {
                    boxed: 0,
                    x,
                    y,
                    w,
                    h,
                    children: Vec::new(),
                    lines: Vec::new(),
                    atoms: Vec::new(),
                    extent: (y, y + h),
                };
                self.border(s, &fr, st, radii(st, w, h));
            }
            if let Some(href) = &span.href {
                let seq = self.next_seq();
                self.links.push(LinkBox {
                    rect: (x0 + ext.x + self.dx, top + self.dy, ext.width, line.height),
                    href: href.clone(),
                    node: (self.tagging && !self.artifact).then_some(span.node),
                    seq,
                });
            }
        }
        let _ = shaped;
    }

    fn runs(
        &mut self,
        s: &mut Surface<'_>,
        shaped: &super::text::Shaped,
        line: &Line,
        x0: f32,
        top: f32,
    ) {
        let line_end = line
            .runs
            .iter()
            .flat_map(|r| r.glyphs.iter().map(|g| g.range.end))
            .max();
        for run in &line.runs {
            let style = std::rc::Rc::clone(shaped.style_of(run.item));
            if !style.visible {
                continue;
            }
            let Some(font) = self.font(run.pick) else {
                continue;
            };
            let Some(start) = run.glyphs.iter().map(|g| g.range.start).min() else {
                continue;
            };
            let end = run
                .glyphs
                .iter()
                .map(|g| g.range.end)
                .max()
                .unwrap_or(start);
            // In a tagged PDF a reader joins the lines of a paragraph as
            // its structure has them: the space a line was broken at, which
            // draws nothing, goes with the line's last glyph, so the words
            // either side of the break stay two words.
            let tail = if self.tagging && !self.artifact && Some(end) == line_end {
                shaped.text[end..]
                    .chars()
                    .next()
                    .filter(|c| matches!(c, ' ' | '\n' | '\t'))
                    .map_or(0, char::len_utf8)
            } else {
                0
            };
            let text = &shaped.text[start..end + tail];
            let size = run.size;
            let mut tailed = tail == 0;
            let glyphs: Vec<KrillaGlyph> = run
                .glyphs
                .iter()
                .map(|g| {
                    let mut range = (g.range.start - start)..(g.range.end - start);
                    if !tailed && g.range.end == end {
                        range.end += tail;
                        tailed = true;
                    }
                    KrillaGlyph::new(
                        GlyphId::new(u32::from(g.id)),
                        g.advance / size,
                        g.x_offset / size,
                        g.y_offset / size,
                        0.0,
                        range,
                        None,
                    )
                })
                .collect();
            let x = x0 + run.x;
            let baseline = top + line.baseline - run.rise;
            let mut pushed = 0;
            if run.pick.synthetic_italic {
                self.close_mark(s);
                // An oblique: a 12° skew about the baseline.
                let k = 0.21;
                s.push_transform(&Transform::from_row(1.0, 0.0, -k, 1.0, k * baseline, 0.0));
                pushed += 1;
            }
            // The shadows under the text, the first on top. Each is drawn
            // as outlines, so the text is copied and searched once; a blur
            // is approximated by fainter copies spread around the offset
            // over half the blur radius (a PDF has no Gaussian blur short of
            // a soft mask per run).
            if !style.text_shadows.is_empty() {
                self.mark_artifact(s);
            }
            for sh in style.text_shadows.iter().rev() {
                let c = sh.color.unwrap_or(style.color);
                if c.a <= 0.0 {
                    continue;
                }
                let at = |dx: f32, dy: f32| Point::from_xy(x + sh.x + dx, baseline + sh.y + dy);
                s.set_stroke(None);
                if sh.blur < 0.5 {
                    s.set_fill(Some(fill(c)));
                    s.draw_glyphs(at(0.0, 0.0), &glyphs, font.clone(), text, size, true);
                    continue;
                }
                let r = sh.blur / 2.0;
                s.set_fill(Some(fill(c.with_alpha(c.a * 0.35))));
                s.draw_glyphs(at(0.0, 0.0), &glyphs, font.clone(), text, size, true);
                s.set_fill(Some(fill(c.with_alpha(c.a * 0.12))));
                for k in 0..8 {
                    let a = k as f32 * std::f32::consts::FRAC_PI_4;
                    s.draw_glyphs(
                        at(r * a.cos(), r * a.sin()),
                        &glyphs,
                        font.clone(),
                        text,
                        size,
                        true,
                    );
                }
            }
            // The text, as content of the inline element it is in (else of
            // the block being painted).
            let node = match shaped.items.get(run.item) {
                Some(Item::Text {
                    span: Some(span), ..
                }) => self.tree.spans.get(*span).map(|sp| sp.node),
                _ => None,
            };
            self.mark_content(s, node, false);
            s.set_fill(Some(fill(style.color)));
            s.set_stroke(if run.pick.synthetic_bold {
                Some(Stroke {
                    paint: paint(style.color),
                    width: size * 0.035,
                    opacity: opacity(style.color.a),
                    ..Default::default()
                })
            } else {
                None
            });
            s.draw_glyphs(
                Point::from_xy(x, baseline),
                &glyphs,
                font,
                text,
                size,
                false,
            );
            s.set_stroke(None);
            if pushed > 0 {
                self.close_mark(s);
            }
            for _ in 0..pushed {
                s.pop();
            }
            // Decorations, the width of the run.
            let decorated = style.underline || style.line_through || style.overline;
            if decorated {
                self.mark_artifact(s);
            }
            let deco = style.decoration_color.unwrap_or(style.color);
            let thick = (size * 0.06).max(0.5);
            let mut deco_line = |y: f32| {
                if let Some(p) = line_path(x, y, x + run.width, y) {
                    s.set_fill(None);
                    s.set_stroke(Some(Stroke {
                        paint: paint(deco),
                        width: thick,
                        opacity: opacity(deco.a),
                        ..Default::default()
                    }));
                    s.draw_path(&p);
                    s.set_stroke(None);
                }
            };
            if style.underline {
                deco_line(baseline + size * 0.12);
            }
            if style.line_through {
                deco_line(baseline - size * 0.3);
            }
            if style.overline {
                deco_line(baseline - size * 0.9);
            }
        }
    }
}
