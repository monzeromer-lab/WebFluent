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
//! ## What this crate offers
//!
//! The documented API is what semver covers:
//!
//! - **Templates** ([`template`], [`Template`]) — render `.wf` templates
//!   with JSON data, to HTML or PDF.
//! - **The front end** ([`syntax`]) — [`parse_source`] reads a `.wf` file
//!   into its tree ([`syntax::parser::ast`]), and [`syntax::fmt`] and
//!   [`syntax::layout`] write it back.
//! - **Builds** ([`build`]) — `wf build` as a library call, through a
//!   [`vfs::Vfs`], writing through [`outfs`]; [`memory`] builds a project
//!   held in memory, as the browser's playground does.
//! - **Configuration** ([`config`]), **findings** ([`diagnostics`],
//!   [`Diagnostic`]), **new projects** ([`init`]) and the **studio's**
//!   compile ([`studio`]).
//!
//! The rest of the compiler — names and types, the linters, code
//! generation, the runtime, the paged engine — is reachable, since the `wf`
//! command and the language server are built on it, but is not documented,
//! and a release may change it without a major version.
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
//!   `webfluent = { version = "6", default-features = false }`.
//!
//! For most uses the [`Template`] API is the entry point; [`parse_source`]
//! and [`build`] are there for a tool that needs the tree or a whole site.

// Much of the compiler is there for the language server and the tests,
// which reach it through the `internals` feature; without it, those parts
// look unused. With it, everything is public, as it always was.
#![cfg_attr(
    not(feature = "internals"),
    allow(dead_code, reason = "the language server and the tests use these")
)]

/// The compiler's version, as `wf --version` prints it: what a program
/// that builds through the library says it built with.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Declares one of the compiler's own parts: public, and hidden from the
/// docs, with the `internals` feature (the language server, the WebAssembly
/// build and the tests turn it on); the crate's own otherwise. Either way it
/// is outside what semver covers.
macro_rules! internal {
    ($(#[$meta:meta])* pub $($item:tt)*) => {
        #[cfg(feature = "internals")]
        $(#[$meta])*
        #[doc(hidden)]
        pub $($item)*

        #[cfg(not(feature = "internals"))]
        $(#[$meta])*
        // A re-export that only the language server, the WebAssembly build
        // or the tests read is unused without the feature.
        #[allow(unused_imports, reason = "read only through the feature")]
        pub(crate) $($item)*
    };
}

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
internal! {
    pub mod paged;
}
pub(crate) mod project;
internal! {
    pub mod registry;
}
/// The `wf` command, run by `src/main.rs`.
#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;
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

// The documented API, which semver covers: the template engine, the front
// end and its tree, the build, the config, the findings, the file systems,
// a project built from memory, a new project, and the studio's compile.
pub use base::{outfs, vfs};
pub use project::config;
pub use tools::{init, memory, studio};

// The compiler's own parts. They stay reachable — the `wf` command and the
// language server are built on them — but they are not documented, and a
// release may change them without a major version.
internal! {
    pub use base::platform;
}
internal! {
    pub use check::{edit, linter, scope, sema, validation};
}
internal! {
    pub use project::{data, i18n, media, openapi, project_js, themes};
}
internal! {
    pub use tools::{
        agent, audit, browser, commands, gallery, generate, migrate, patch, testing, verify,
    };
}
internal! {
    pub use web::{codegen, motion, runtime};
}

pub use check::edit::{ArgRef, EditOp, apply_edits};
pub use check::linter::lint_vocabulary as validate_vocabulary;
pub use check::linter::validate_semantics;
pub use diagnostics::error::{Diagnostic, Result, WebFluentError};
pub use project::config::project::{PdfConfig, PdfMargins, SlidesConfig};
pub use syntax::node_id::{NodeInfo, NodeMap};
pub use syntax::{Dialect, detect_dialect, parse_source};
pub use template::{PdfReport, Template};
pub use tools::studio::{CompiledPage, CompiledSite, ThemeInfo, compile_studio};
