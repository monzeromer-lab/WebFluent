//! Small text utilities the rest of the crate shares.

/// A number as the language writes it: a whole number with no fraction
/// (`-0` as `0`), any other in Rust's shortest form that reads back the same.
/// A whole number past what an `i64` holds keeps that form too, since `as`
/// saturates and the round trip then misses.
pub(crate) fn number(n: f64) -> String {
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "the round trip through i64 is the test"
    )]
    let truncated = n as i64;
    #[expect(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        clippy::float_cmp,
        reason = "the round trip through i64 is the test"
    )]
    let whole = truncated as f64 == n;
    if whole {
        truncated.to_string()
    } else {
        n.to_string()
    }
}

/// Plain Levenshtein distance; the inputs are single words, so O(n·m) is fine.
pub(crate) fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != cb);
            cur[j + 1] = sub.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// `fontSize` → `font-size`: a camel-cased name as CSS and HTML write it.
pub(crate) fn camel_to_kebab(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.extend(ch.to_lowercase());
    }
    result
}

/// `publishableKey` as an attribute is `publishable-key`: what a custom
/// element's `observedAttributes` says, and what HTML can carry. A name
/// that already has a hyphen is written as it is.
pub(crate) fn kebab_case(name: &str) -> String {
    if name.contains('-') {
        return name.to_string();
    }
    let mut out = String::with_capacity(name.len() + 4);
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            if !out.is_empty() {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// `font-size` → `fontSize`: a CSS name as a style object's key.
pub(crate) fn to_camel_case(kebab: &str) -> String {
    let mut result = String::with_capacity(kebab.len());
    let mut capitalize_next = false;
    for ch in kebab.chars() {
        if ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// A word with its first letter upper-cased: `message` → `Message`.
pub(crate) fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_uppercase().collect::<String>(), chars.as_str()),
        None => String::new(),
    }
}
