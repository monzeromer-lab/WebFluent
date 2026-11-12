//! Code generation — compiles the AST to HTML, CSS, JavaScript, SSG pages, and PDF.
//!
//! Each output format has its own module:
//! - [`html`] — generates the HTML entry point (`index.html`)
//! - [`css`] — generates design tokens and component styles (`styles.css`)
//! - [`js`] — generates the JavaScript bundle with reactivity and routing (`app.js`)
//! - [`ssg`] — pre-renders pages to static HTML for SSG mode
//! - [`node_id`] — deterministic node identity (`data-wf-node`) for the studio
//!
//! [`builtin`] holds the built-in component table every renderer reads, so the
//! SPA bundle, the static paint and the template engine cannot disagree about
//! what a `Card` or a `Heading` is.

pub use crate::project::builtin;
pub mod capture;
pub mod charts;
pub mod csp;
pub mod css;
pub mod elements;
pub use crate::project::format;
pub mod gzip;
pub mod highlight;
pub mod html;
pub mod js;
pub mod jscheck;
pub mod markdown;
pub mod minify;
pub use crate::syntax::node_id;
pub mod offline;
pub mod project_css;
pub use crate::project::sanitize;
pub use crate::project::scoped_css;
pub mod seo;
pub mod sourcemap;
pub mod ssg;
pub use crate::project::static_eval;
pub mod style;
pub use crate::project::url;
pub use crate::registry::style_tokens;

pub use css::{dark_css, generate_css, generate_css_for, generate_css_with};
pub use html::generate_html;
pub use js::JsCodegen;
pub use ssg::render_page_html;
// The studio node-identity API (`node_id::{NodeMap, NodeInfo, build_node_map}`
// and `ssg::render_page_html_studio`) is reachable via `pub mod node_id` and
// `pub mod ssg`; Milestone 2 will re-export the ergonomic subset it consumes.
