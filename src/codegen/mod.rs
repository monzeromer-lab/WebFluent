//! Code generation — compiles the AST to HTML, CSS, JavaScript, SSG pages, and PDF.
//!
//! Each output format has its own module:
//! - [`html`] — generates the HTML entry point (`index.html`)
//! - [`css`] — generates design tokens and component styles (`styles.css`)
//! - [`js`] — generates the JavaScript bundle with reactivity and routing (`app.js`)
//! - [`ssg`] — pre-renders pages to static HTML for SSG mode
//! - [`pdf`] — generates PDF documents with layout, tables, and typography
//! - [`slides`] — generates PDF slide decks (one Slide = one page)
//! - [`node_id`] — deterministic node identity (`data-wf-node`) for the studio
//!
//! [`builtin`] holds the built-in component table every renderer reads, so the
//! SPA bundle, the static paint and the template engine cannot disagree about
//! what a `Card` or a `Heading` is.

pub mod builtin;
pub mod css;
pub mod html;
pub mod js;
pub mod node_id;
pub mod pdf;
pub mod seo;
pub mod slides;
pub mod ssg;
pub mod static_eval;
pub mod style;
pub mod style_tokens;

#[allow(unused_imports)]
pub use css::{generate_css, generate_css_with};
pub use html::generate_html;
pub use js::JsCodegen;
pub use pdf::PdfCodegen;
pub use slides::SlidesCodegen;
pub use ssg::render_page_html;
// The studio node-identity API (`node_id::{NodeMap, NodeInfo, build_node_map}`
// and `ssg::render_page_html_studio`) is reachable via `pub mod node_id` and
// `pub mod ssg`; Milestone 2 will re-export the ergonomic subset it consumes.
