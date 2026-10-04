//! The built-in icons, drawn from the same table the runtime draws them from
//! (`runtime/modules/icons.js`), so an icon on paper is the icon on screen.

use std::collections::HashMap;
use std::sync::OnceLock;

const SOURCE: &str = include_str!("../runtime/modules/icons.js");

/// Icon name → the SVG elements inside its 24×24 view box.
fn table() -> &'static HashMap<String, String> {
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

/// A whole SVG document for an icon, in the given colour.
pub fn svg(name: &str, color: &str) -> Option<String> {
    let inner = table().get(name)?;
    Some(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" color="{color}">{}</svg>"#,
        inner.replace("currentColor", color)
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_icon_the_registry_names_is_drawn() {
        for name in crate::registry::ICONS {
            let markup = super::svg(name, "#000").unwrap_or_else(|| panic!("no icon {name}"));
            assert!(
                usvg::Tree::from_str(&markup, &usvg::Options::default()).is_ok(),
                "{name}"
            );
        }
    }
}
