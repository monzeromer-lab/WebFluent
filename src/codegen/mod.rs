//! Code generation — compiles the AST to HTML, CSS, JavaScript, SSG pages, and PDF.
//!
//! Each output format has its own module:
//! - [`html`] — generates the HTML entry point (`index.html`)
//! - [`css`] — generates design tokens and component styles (`styles.css`)
//! - [`js`] — generates the JavaScript bundle with reactivity and routing (`app.js`)
//! - [`ssg`] — pre-renders pages to static HTML for SSG mode
//! - [`pdf`] — generates PDF documents with layout, tables, and typography
//! - [`slides`] — generates PDF slide decks (one Slide = one page)

pub mod html;
pub mod css;
pub mod js;
pub mod ssg;
pub mod pdf;
pub mod slides;
pub mod style;

pub use html::generate_html;
pub use css::generate_css;
pub use js::JsCodegen;
pub use ssg::render_page_html;
pub use pdf::PdfCodegen;
pub use slides::SlidesCodegen;
