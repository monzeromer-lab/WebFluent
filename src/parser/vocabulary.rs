//! The modifier vocabulary as data (studio ask A-3, first slice).
//!
//! This set used to live only inside `parser::Parser::is_modifier` as a match
//! arm, hand-copied into the LSP's completion list and the studio's language
//! card — three copies across two repositories, and the first two had already
//! drifted by nine keywords. The parser now consults this table, so there is
//! exactly one place a modifier can be added, and downstream consumers (the
//! vocabulary lint, the LSP, the studio's language card) can read the same
//! truth instead of transcribing it.

/// Every identifier the parser accepts as a component modifier.
///
/// `success`, `error` and `loading` are lexed as keyword tokens rather than
/// identifiers, but they are modifier words all the same — `is_modifier`
/// accepts their token types directly, and suggestion machinery ("did you mean
/// …?") needs them in the table.
pub const MODIFIER_KEYWORDS: &[&str] = &[
    // Size
    "small",
    "medium",
    "large",
    // Color
    "primary",
    "secondary",
    "success",
    "danger",
    "warning",
    "info",
    // State words (keyword-lexed; see the doc comment)
    "error",
    "loading",
    // Shape
    "rounded",
    "pill",
    "square",
    // Elevation
    "flat",
    "elevated",
    "outlined",
    // Width
    "full",
    "fit",
    // Text
    "bold",
    "italic",
    "underline",
    "uppercase",
    "lowercase",
    // Alignment
    "left",
    "center",
    "right",
    // Typography
    "heading",
    "subtitle",
    "muted",
    // Heading levels
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    // Other
    "dismissible",
    "block",
    "bordered",
    "controls",
    "autoplay",
    // Input types
    "text",
    "email",
    "password",
    "number",
    "search",
    "tel",
    "url",
    "date",
    "time",
    "datetime",
    "color",
    // Button types
    "submit",
    "reset",
    // Animation modifiers
    "fadeIn",
    "fadeOut",
    "slideUp",
    "slideDown",
    "slideLeft",
    "slideRight",
    "scaleIn",
    "scaleOut",
    "bounce",
    "shake",
    "pulse",
    "spin",
    // Animation speed
    "fast",
    "slow",
];

/// Whether `word` is in the modifier vocabulary.
pub fn is_modifier_keyword(word: &str) -> bool {
    MODIFIER_KEYWORDS.contains(&word)
}
