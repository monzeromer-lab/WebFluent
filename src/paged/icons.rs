//! The built-in icons, drawn from the same table the runtime draws them from
//! (`runtime/modules/icons.js`), so an icon on paper is the icon on screen.

/// A whole SVG document for an icon, in the given colour.
pub fn svg(name: &str, color: &str) -> Option<String> {
    let inner = crate::web::icons::inner(name)?;
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
