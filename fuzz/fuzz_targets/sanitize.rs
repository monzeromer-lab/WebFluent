//! `sanitize` on any markup: what it keeps runs no script — no `<script>`,
//! no `on…` handler inside a tag — and keeping it again changes nothing.
#![no_main]

/// The attribute names inside each tag of `html`, read as a browser reads
/// them: a quoted value is skipped whole, whatever it holds.
fn attribute_names(html: &str) -> Vec<String> {
    let mut names = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        // The tag's name.
        i += 1;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
            i += 1;
        }
        // Its attributes, up to the `>` that closes it.
        loop {
            while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] == b'>' {
                break;
            }
            let start = i;
            while i < bytes.len()
                && !bytes[i].is_ascii_whitespace()
                && !matches!(bytes[i], b'=' | b'>' | b'/')
            {
                i += 1;
            }
            names.push(String::from_utf8_lossy(&bytes[start..i]).to_ascii_lowercase());
            if i < bytes.len() && bytes[i] == b'=' {
                i += 1;
                match bytes.get(i) {
                    Some(&quote @ (b'"' | b'\'')) => {
                        i += 1;
                        while i < bytes.len() && bytes[i] != quote {
                            i += 1;
                        }
                        i += 1;
                    }
                    _ => {
                        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                            i += 1;
                        }
                    }
                }
            }
        }
    }
    names
}

libfuzzer_sys::fuzz_target!(|html: &str| {
    let clean = webfluent::codegen::sanitize::sanitize(html);
    assert!(
        !clean.to_ascii_lowercase().contains("<script"),
        "a script survived:\n{clean}"
    );
    assert!(
        !attribute_names(&clean).iter().any(|n| n.starts_with("on")),
        "an event handler survived:\n{clean}"
    );
    assert_eq!(
        webfluent::codegen::sanitize::sanitize(&clean),
        clean,
        "sanitising twice changed the markup"
    );
});
