//! Where a node of a program was written.
//!
//! Where a node of a program was written.

/// A source span: a byte range into the original `.wf` source plus the
/// 1-based line/column of its start.
///
/// `start`/`end` are **byte** offsets (`end` exclusive), so `&source[start..end]`
/// slices the exact text the node was parsed from. `line`/`col` describe `start`
/// and exist for diagnostics.
///
/// Spans are additive metadata: they power the studio's click-to-code and
/// structured-edit features (see the `STUDIO_INTEGRATION_PLAN`). Every code
/// generator (html/css/js/ssg/pdf/slides) ignores them, and nodes built outside
/// the parser can use [`Span::dummy`] / `..Default::default()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    /// Byte offset of the first byte of the node in the source.
    pub start: u32,
    /// Byte offset one past the last byte of the node (exclusive).
    pub end: u32,
    /// 1-based line number of `start`.
    pub line: u32,
    /// 1-based column number of `start`.
    pub col: u32,
}

impl Span {
    pub fn new(start: u32, end: u32, line: u32, col: u32) -> Self {
        Self {
            start,
            end,
            line,
            col,
        }
    }

    /// A placeholder span (all zeros) for nodes constructed outside the parser.
    pub fn dummy() -> Self {
        Self::default()
    }

    /// Slice the original `source` this span was taken from.
    ///
    /// Panics if `source` is not the exact string the span was produced against.
    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start as usize..self.end as usize]
    }
}
