//! Markdown from anywhere — a page's text, a language server's hover —
//! renders without panicking, plain and inert.
#![no_main]

libfuzzer_sys::fuzz_target!(|text: &str| {
    let _ = webfluent::codegen::markdown::render(text);
    let _ = webfluent::codegen::markdown::render_with_base(text, "/base");
});
