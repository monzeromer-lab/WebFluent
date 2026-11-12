//! `wf fmt` on any file it accepts: what it writes still parses, and
//! formatting that again changes nothing.
#![no_main]

libfuzzer_sys::fuzz_target!(|source: &str| {
    let Ok(formatted) = webfluent::syntax::fmt::format_source(source, "fuzz.wf") else {
        return;
    };
    if webfluent::syntax::parse_source(source, "fuzz.wf").is_ok() {
        assert!(
            webfluent::syntax::parse_source(&formatted, "fuzz.wf").is_ok(),
            "formatting broke a file that parsed:\n{formatted}"
        );
    }
    let again = webfluent::syntax::fmt::format_source(&formatted, "fuzz.wf")
        .expect("what fmt wrote, fmt reads");
    assert_eq!(again, formatted, "formatting twice changed the file");
});
