//! # WebFluent
//!
//! **The Web-First Language** — a programming language that compiles to HTML, CSS, JavaScript, and PDF.
//!
//! WebFluent provides 50+ built-in UI components, signal-based reactivity, client-side routing,
//! internationalization (i18n), animations, static site generation (SSG), and PDF document output —
//! all with zero runtime dependencies.
//!
//! ## Quick Start
//!
//! Use WebFluent as a **template engine** to render `.wf` templates with JSON data:
//!
//! ```rust
//! use webfluent::Template;
//! use serde_json::json;
//!
//! let tpl = Template::from_str(r##"
//!     page Home(path: "/", title: "Hello") {
//!         Container {
//!             Heading("Hello, {name}!").h1
//!             Text("Welcome to WebFluent.")
//!         }
//!     }
//! "##).unwrap();
//!
//! let html = tpl.render_html(&json!({"name": "World"})).unwrap();
//! // Returns a full HTML document with embedded CSS
//!
//! let fragment = tpl.render_html_fragment(&json!({"name": "World"})).unwrap();
//! // Returns just the HTML fragment (no <html> wrapper)
//! ```
//!
//! ## Rendering to PDF
//!
//! ```rust,no_run
//! use webfluent::Template;
//! use serde_json::json;
//!
//! let tpl = Template::from_file("templates/invoice.wf").unwrap();
//! let pdf_bytes = tpl.render_pdf(&json!({
//!     "number": "INV-001",
//!     "customer": { "name": "Acme Corp" },
//!     "items": [{ "name": "Widget", "price": 9.99 }],
//!     "paid": true
//! })).unwrap();
//!
//! std::fs::write("invoice.pdf", pdf_bytes).unwrap();
//! ```
//!
//! ## Theming
//!
//! A theme is written in the template itself. Declare one and it is used; the
//! tokens it does not name keep their baseline values.
//!
//! ```rust
//! use webfluent::Template;
//! use serde_json::json;
//!
//! let html = Template::from_str(r##"
//!     theme Brand {
//!         color-primary: #0F766E
//!         radius-md: 14px
//!     }
//!     page P(path: "/") { Container { Text("Hello") } }
//! "##)
//!     .unwrap()
//!     .with_tokens(&[("color-secondary", "#8B5CF6")])
//!     .render_html(&json!({}))
//!     .unwrap();
//!
//! assert!(html.contains("--color-primary: #0F766E"));
//! assert!(html.contains("--color-secondary: #8B5CF6"), "config tokens layer on top");
//! ```
//!
//! ## Architecture
//!
//! The compilation pipeline:
//!
//! 1. **Syntax** ([`syntax`]) — [`parse_source`] reads a `.wf` file into an AST
//!    ([`lexer`] and [`parser`] beneath it); a file in the grammar of
//!    WebFluent 2 is refused with a pointer to `wf migrate` ([`migrate`])
//! 2. **Registry** ([`registry`]) — the one description of every built-in
//!    component, its props, flags, events, slots and parts
//! 3. **Semantics** ([`sema`]) — `check` resolves names, flags and cases against
//!    the registry and the program's declarations; `types::check` infers and
//!    checks types; `lower` rewrites the new grammar's forms into what the
//!    backends render
//! 4. **Linter** ([`linter`]) — accessibility, SEO, vocabulary and PDF checks
//! 5. **Codegen** ([`codegen`]) — generates HTML, CSS, JS, SSG pages, or PDF output
//! 6. **Themes** ([`themes`]) — design tokens and component CSS
//! 7. **Runtime** ([`runtime`]) — the JavaScript runtime for reactivity, routing and motion
//!
//! ## Templates in a server
//!
//! A [`Template`] parses and checks once and is `Send + Sync`, so a server
//! loads its templates at start-up and renders on every request. Data is any
//! `serde::Serialize` value; a directory of templates shares its components,
//! and [`Template::page`] picks the document to render:
//!
//! ```rust,no_run
//! use serde::Serialize;
//! use webfluent::Template;
//!
//! #[derive(Serialize)]
//! struct Receipt { number: u32, total: f64 }
//!
//! let templates = Template::from_dir("templates").unwrap();
//! let html = templates
//!     .page("Receipt").unwrap()
//!     .render_html(&Receipt { number: 7, total: 12.5 })
//!     .unwrap();
//! ```
//!
//! ## Crate Features
//!
//! - **`webp`**, **`pdf`**, **`bundled-fonts`** — for a WebAssembly build
//!   only: lossy WebP (libwebp), the PDF and slide-deck renderer, and the
//!   Liberation faces built in. A native build has all three whatever the
//!   features say; a WebAssembly one has each only when asked, since each
//!   costs a download (`crates/wf-wasm`).
//! - **`cli`** (default) — the `wf` command's own dependencies: its argument
//!   parser and the dev server. A program that only compiles or renders
//!   templates turns it off:
//!   `webfluent = { version = "5", default-features = false }`.
//!
//! This crate exposes the full compiler pipeline. For most use cases, the [`Template`] API
//! is the simplest entry point. For full control, use [`parse_source`], [`sema`] and the
//! codegen modules directly.

/// The compiler's version, as `wf --version` prints it: what a program
/// that builds through the library says it built with.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// The layers, bottom first: base, diagnostics, syntax, registry, project,
// check, web, paged, template, build, tools. A module uses only its own
// layer and the ones below it; `tests/module_layers.rs` holds it to that.
pub(crate) mod base;
/// `wf build` as a library call: [`build::build`] reads a project through a
/// [`vfs::Vfs`], writes the site, and reports every line the command prints
/// and every finding, with its code and place.
pub mod build;
pub(crate) mod check;
pub mod diagnostics;
pub mod paged;
pub(crate) mod project;
pub mod registry;
/// The front end's one entry point: [`syntax::parse_source`] reads a file in
/// whichever grammar it is written in.
pub mod syntax;
/// Template engine — render `.wf` templates with JSON data.
///
/// The [`Template`] struct is the primary public API for using WebFluent as a library.
/// It supports rendering to HTML documents, HTML fragments, and PDF files.
pub mod template;
pub(crate) mod tools;
pub(crate) mod web;

// Every module at the path it has always had.
pub use base::{outfs, platform, vfs};
pub use build::assets;
pub use check::{edit, linter, scope, sema, validation};
pub use diagnostics::error;
pub use paged::style_value;
pub use project::{config, data, i18n, media, openapi, project_js, themes};
pub use syntax::{fmt, layout, lexer, parser, print, reprint};
pub use tools::{
    agent, audit, browser, commands, gallery, generate, init, memory, migrate, patch, studio,
    testing, verify,
};
pub use web::{codegen, motion, runtime};

pub use check::edit::{ArgRef, EditOp, apply_edits};
pub use check::linter::lint_vocabulary as validate_vocabulary;
pub use check::linter::validate_semantics;
pub use diagnostics::error::{Diagnostic, Result, WebFluentError};
pub use project::config::project::{PdfConfig, PdfMargins, SlidesConfig};
pub use syntax::node_id::{NodeInfo, NodeMap};
pub use syntax::{Dialect, detect_dialect, parse_source};
pub use template::{PdfReport, Template};
pub use tools::studio::{
    CompiledPage, CompiledSite, Diagnostic as StudioDiagnostic, ThemeInfo, compile_studio,
};
