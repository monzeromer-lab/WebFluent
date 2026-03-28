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
//! let tpl = Template::from_str(r#"
//!     Page Home (path: "/", title: "Hello") {
//!         Container {
//!             Heading("Hello, {name}!", h1)
//!             Text("Welcome to WebFluent.")
//!         }
//!     }
//! "#).unwrap();
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
//! Override design tokens or switch themes:
//!
//! ```rust,no_run
//! use webfluent::Template;
//! use serde_json::json;
//!
//! let html = Template::from_str("Page P (path: \"/\") { Container { Text(\"Hello\") } }")
//!     .unwrap()
//!     .with_theme("dark")
//!     .with_tokens(&[("color-primary", "#8B5CF6")])
//!     .render_html(&json!({}))
//!     .unwrap();
//! ```
//!
//! ## Architecture
//!
//! The compilation pipeline:
//!
//! 1. **Lexer** ([`lexer`]) — tokenizes `.wf` source into a token stream
//! 2. **Parser** ([`parser`]) — builds an AST from tokens
//! 3. **Linter** ([`linter`]) — runs accessibility and PDF validation checks
//! 4. **Codegen** ([`codegen`]) — generates HTML, CSS, JS, SSG pages, or PDF output
//! 5. **Themes** ([`themes`]) — provides design tokens and component CSS
//! 6. **Runtime** ([`runtime`]) — embeds the JavaScript runtime for reactivity and routing
//!
//! ## Crate Features
//!
//! This crate exposes the full compiler pipeline. For most use cases, the [`Template`] API
//! is the simplest entry point. For full control, use the lexer, parser, and codegen modules
//! directly.

#![allow(dead_code)]

/// Lexical analysis — tokenizes `.wf` source code.
///
/// The lexer converts raw source text into a stream of [`lexer::Token`]s, handling
/// keywords, identifiers, string literals (with `{var}` interpolation), numbers,
/// operators, and punctuation.
pub mod lexer;

/// Parsing — builds an abstract syntax tree from tokens.
///
/// The parser consumes tokens from the lexer and produces a [`parser::Program`] AST
/// containing pages, components, stores, and an optional app declaration.
pub mod parser;

/// Code generation — compiles the AST to various output formats.
///
/// Supports HTML ([`codegen::generate_html`]), CSS ([`codegen::generate_css`]),
/// JavaScript ([`codegen::JsCodegen`]), static site generation ([`codegen::render_page_html`]),
/// and PDF ([`codegen::PdfCodegen`]).
pub mod codegen;

/// JavaScript runtime — embedded runtime for reactivity, routing, and DOM helpers.
///
/// The runtime provides signal-based reactivity, conditional/list rendering,
/// client-side routing, store management, i18n, animations, and toast notifications.
/// It is embedded as a constant string ([`runtime::RUNTIME_JS`]) and included in
/// compiled output.
pub mod runtime;

/// Design system — theme tokens and component CSS.
///
/// Provides built-in themes (`default`, `dark`, `minimal`, `brutalist`) via
/// [`themes::get_theme_tokens`] and component-level CSS via [`themes::component_css`].
pub mod themes;

/// Project configuration — loads and manages `webfluent.app.json`.
///
/// The [`config::ProjectConfig`] struct holds all project settings: theme, build options,
/// dev server config, meta tags, and i18n settings.
pub mod config;

/// Error types and diagnostics.
///
/// Provides [`WebFluentError`] (with variants for lexer, parser, codegen, config, and I/O errors),
/// [`error::Diagnostic`] (with file, line, column, and optional hint),
/// and [`error::A11yWarning`] for accessibility lint results.
pub mod error;

/// Linting — compile-time checks for accessibility and PDF validation.
///
/// [`linter::lint_accessibility`] runs 12 WCAG-based checks (missing alt text, form labels,
/// heading hierarchy, etc.). [`linter::validate_for_pdf`] ensures interactive components
/// aren't used in PDF output.
pub mod linter;

/// Template engine — render `.wf` templates with JSON data.
///
/// The [`Template`] struct is the primary public API for using WebFluent as a library.
/// It supports rendering to HTML documents, HTML fragments, and PDF files.
pub mod template;

pub use template::Template;
pub use error::{WebFluentError, Result};
