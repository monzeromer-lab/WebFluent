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

pub mod lexer;

/// A project's sources, read and merged into one program.
pub(crate) mod sources;

pub mod parser;

/// The front end's one entry point: [`syntax::parse_source`] reads a file in
/// whichever grammar it is written in.
pub mod syntax;

pub mod registry;

pub mod sema;

/// `wf migrate`: the original grammar rewritten as WebFluent 3.
pub mod migrate;

/// `wf test`: a project's tests, rendered or played in a browser, and what
/// each came to.
pub mod testing;

/// `wf docs`: the component gallery, one self-contained HTML page.
pub mod gallery;

/// `wf verify`: every page of a built project in a browser — what each
/// route did, and the built-ins no page drew.
pub mod verify;

/// `wf init`: a new project from a template.
pub mod init;

/// `wf audit`: what a project trusts — markup it did not write, other
/// origins, what it keeps on the reader's machine, its `env` names, its
/// policy and its dependencies.
pub mod audit;

/// `data` declarations: the JSON files and pictures a program names, read
/// at build time and made constants.
pub mod data;
/// `wf fmt --to wfx|wf`: the one grammar in either layout, braces or
/// indentation.
pub mod fmt;
pub mod i18n;
pub mod layout;

/// `public/` as data: every file, its kind, size and dimensions, where
/// the project names it, and what the media pipeline will make of an image.
pub mod assets;
pub mod browser;
/// Code generation — compiles the AST to various output formats.
///
/// Supports HTML ([`codegen::generate_html`]), CSS ([`codegen::generate_css`]),
/// JavaScript ([`codegen::JsCodegen`]), static site generation ([`codegen::render_page_html`]),
/// and PDF (the [`paged`] engine lays the HTML out on paper).
pub mod codegen;
pub mod commands;
pub mod media;
pub mod openapi;

pub mod project_js;

/// JavaScript runtime — embedded runtime for reactivity, routing, and DOM helpers.
///
/// The runtime provides signal-based reactivity, conditional/list rendering,
/// client-side routing, store management, i18n, animations, and toast notifications.
/// It is a set of feature modules, and a build ships only the ones its program
/// reaches ([`runtime::assemble`]); [`runtime::full`] is every module at once.
pub mod runtime;

pub mod themes;

/// Project configuration — loads and manages `webfluent.app.json`.
///
/// The [`config::ProjectConfig`] struct holds all project settings: theme, build options,
/// dev server config, meta tags, and i18n settings.
pub mod config;

pub mod diagnostics;
pub use diagnostics::error;

pub mod linter;

pub mod paged;
/// Template engine — render `.wf` templates with JSON data.
///
/// The [`Template`] struct is the primary public API for using WebFluent as a library.
/// It supports rendering to HTML documents, HTML fragments, and PDF files.
pub mod template;

pub mod edit;

/// The tree written back as source, one node at a time: what an editor
/// that changes a node writes in its place.
pub mod print;

/// The smallest edit: the patches that turn one node's source into an
/// edited copy's, touching nothing else.
pub mod reprint;

/// Style values as data: what shape a property's value has, the value read
/// into its parts, and the narrowest declaration written back.
pub mod style_value;

/// Motion as data: the animations, eases and speeds an element can take,
/// the route transition, the project's default, and how each is written.
pub mod motion;

/// Validation as data: every rule a `validate` block may hold, the named
/// patterns, when a form shows its messages, the rules one state is held
/// to, and the block written back one rule at a time.
pub mod validation;

/// What a name at a place can refer to — the editor's completion scope —
/// and, typed, what a path picker lists there.
pub mod scope;

/// What an agent works through: a change set held apart from the working
/// tree, built before it is read, and the context it is given.
pub mod agent;

/// Studio integration facade — [`studio::compile_studio`] turns a program into a
/// [`studio::CompiledSite`] (SSG pages + CSS + JS + node map) for the preview.
pub mod studio;

/// What a running page changes to follow an edit, with no reload.
pub mod patch;

/// `wf build` as a library call: [`build::build`] reads a project through a
/// [`vfs::Vfs`], writes the site, and reports every line the command prints
/// and every finding, with its code and place.
pub mod build;

/// `wf generate` as a library call: the source of a new page, component or
/// store, and [`generate::create`] to write one in the project's layout.
pub mod generate;

pub use base::vfs;

pub use base::outfs;

pub use base::platform;

pub(crate) mod base;
/// A project built from memory, with nothing read from or written to a disk:
/// what the browser's playground runs.
pub mod memory;

pub use codegen::node_id::{NodeInfo, NodeMap};
pub use config::project::{PdfConfig, PdfMargins, SlidesConfig};
pub use edit::{ArgRef, EditOp, apply_edits};
pub use error::{Diagnostic, Result, WebFluentError};
pub use linter::lint_vocabulary as validate_vocabulary;
pub use linter::validate_semantics;
pub use studio::{
    CompiledPage, CompiledSite, Diagnostic as StudioDiagnostic, ThemeInfo, compile_studio,
};
pub use syntax::{Dialect, detect_dialect, parse_source};
pub use template::{PdfReport, Template};
