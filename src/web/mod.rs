//! The web output: code generation, the runtime it ships, motion, and the
//! whole-project check that reads them.

pub mod check;
/// Code generation — compiles the AST to various output formats.
///
/// Supports HTML ([`codegen::generate_html`]), CSS ([`codegen::generate_css`]),
/// JavaScript ([`codegen::JsCodegen`]), static site generation ([`codegen::render_page_html`]),
/// and PDF (the [`paged`] engine lays the HTML out on paper).
pub mod codegen;
/// Motion as data: the animations, eases and speeds an element can take,
/// the route transition, the project's default, and how each is written.
pub mod motion;
/// JavaScript runtime — embedded runtime for reactivity, routing, and DOM helpers.
///
/// The runtime provides signal-based reactivity, conditional/list rendering,
/// client-side routing, store management, i18n, animations, and toast notifications.
/// It is a set of feature modules, and a build ships only the ones its program
/// reaches ([`runtime::assemble`]); [`runtime::full`] is every module at once.
pub mod runtime;
