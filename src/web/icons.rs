//! The built-in icons: the table the runtime draws them from
//! (`runtime/modules/icons.js`), read once at build time, so the static paint
//! and a PDF draw the icon the live page draws.

use std::collections::HashMap;
use std::sync::OnceLock;

const SOURCE: &str = include_str!("runtime/modules/icons.js");

/// Icon name → the SVG elements inside its 24×24 view box.
pub(crate) fn table() -> &'static HashMap<String, String> {
    static TABLE: OnceLock<HashMap<String, String>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut out = HashMap::new();
        let Some(start) = SOURCE.find("const _ICONS = {") else {
            return out;
        };
        let body = &SOURCE[start..];
        let end = body.find("\n  };").unwrap_or(body.len());
        for line in body[..end].lines().skip(1) {
            let line = line.trim();
            let Some((key, rest)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim().trim_matches(['"', '\'']);
            let rest = rest.trim().trim_end_matches(',');
            let Some(markup) = rest.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) else {
                continue;
            };
            out.insert(key.to_string(), markup.to_string());
        }
        out
    })
}

/// The SVG elements of an icon, inside its 24×24 view box.
pub(crate) fn inner(name: &str) -> Option<&'static str> {
    table().get(name).map(String::as_str)
}

/// An icon as the runtime draws it into its `data-icon` element.
pub(crate) fn drawn(name: &str) -> Option<String> {
    inner(name).map(|svg| {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24" fill="none">{svg}</svg>"#
        )
    })
}
