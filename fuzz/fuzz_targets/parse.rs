//! Any text, read as a `.wf` file: the lexer and the recovering parser an
//! editor runs on every keystroke never panic, whatever is typed.
#![no_main]

libfuzzer_sys::fuzz_target!(|source: &str| {
    let _ = webfluent::syntax::tokens(source, "fuzz.wf");
    let _ = webfluent::syntax::parse_source_recovering(source, "fuzz.wf");
});
