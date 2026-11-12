//! A file in braces, written in the indented layout and back: when both
//! steps succeed, what comes back parses as the original did.
#![no_main]

libfuzzer_sys::fuzz_target!(|source: &str| {
    if webfluent::syntax::parse_source(source, "fuzz.wf").is_err() {
        return;
    }
    let Ok(offside) = webfluent::syntax::layout::to_offside(source, "fuzz.wf") else {
        return;
    };
    let braces = webfluent::syntax::layout::to_braces(&offside, "fuzz.wfx")
        .expect("the indented layout it wrote, it reads back");
    assert!(
        webfluent::syntax::parse_source(&braces, "fuzz.wf").is_ok(),
        "a file that parsed did not survive the round trip:\n{braces}"
    );
});
