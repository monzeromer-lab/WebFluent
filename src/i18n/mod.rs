//! Messages with plural forms and placeholders, resolved at build time the
//! way the runtime resolves them: `t("items", { count: n })` reads the form
//! the locale's CLDR rules pick for `n` ([`plurals`]) — `items.one`,
//! `items.few` … — and `items.zero` for exactly 0 when the file has one,
//! in any language; then `items.other`, then `items`. `{name}` in the
//! message is filled from the values given. [`table`] is the translations
//! as an editor's table, both ways.

use std::collections::HashMap;
use std::path::Path;

pub mod plurals;
pub mod table;
#[rustfmt::skip]
pub mod plurals_data;

use crate::codegen::static_eval::Static;

/// Every locale's messages, read from `<dir>/<locale>.json`. A locale with
/// no file has none; a file that is not a JSON map of strings is an error.
pub fn load(
    project_dir: &Path,
    config: &crate::config::project::I18nConfig,
) -> crate::error::Result<HashMap<String, HashMap<String, String>>> {
    load_via(&crate::vfs::FsVfs, project_dir, config)
}

/// [`load`], reading through `vfs`.
pub fn load_via(
    vfs: &dyn crate::vfs::Vfs,
    project_dir: &Path,
    config: &crate::config::project::I18nConfig,
) -> crate::error::Result<HashMap<String, HashMap<String, String>>> {
    let mut out = HashMap::new();
    let dir = project_dir.join(&config.dir);
    for locale in &config.locales {
        let file = dir.join(format!("{locale}.json"));
        let Ok(content) = vfs.read_to_string(&file) else {
            continue;
        };
        let messages: HashMap<String, String> = serde_json::from_str(&content).map_err(|e| {
            crate::error::WebFluentError::ConfigError(format!("Failed to parse {locale}.json: {e}"))
        })?;
        out.insert(locale.clone(), messages);
    }
    Ok(out)
}

/// The plural form `count` takes in `locale`, by CLDR's rules.
pub fn plural_category(locale: &str, count: f64) -> &'static str {
    plurals::select_number(locale, count)
}

/// The message for `key` over `params`, in `locale`'s messages.
pub fn message(
    locale: &str,
    messages: &HashMap<String, String>,
    key: &str,
    params: &[(String, Static)],
) -> String {
    let count = params
        .iter()
        .find(|(k, _)| k == "count")
        .and_then(|(_, v)| match v {
            Static::Num(n) => Some(*n),
            _ => None,
        });
    let mut text = None;
    if let Some(n) = count {
        let category = plural_category(locale, n);
        // Exactly none is the `zero` message wherever the file has one —
        // "No entries" — though English's rules have no such form.
        let zero = (n == 0.0)
            .then(|| messages.get(&format!("{key}.zero")))
            .flatten();
        text = zero
            .or_else(|| messages.get(&format!("{key}.{category}")))
            .or_else(|| messages.get(&format!("{key}.other")))
            .cloned();
    }
    let mut text = text
        .or_else(|| messages.get(key).cloned())
        .unwrap_or_else(|| key.to_string());
    for (k, v) in params {
        text = text.replace(&format!("{{{k}}}"), &v.to_text());
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plural_forms_are_picked_by_count_and_placeholders_filled() {
        let messages: HashMap<String, String> = [
            ("items.one", "{count} item for {name}"),
            ("items.other", "{count} items for {name}"),
            ("plain", "hello {name}"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        let params = |n: f64| {
            vec![
                ("count".to_string(), Static::Num(n)),
                ("name".to_string(), Static::Str("Sam".to_string())),
            ]
        };
        assert_eq!(
            message("en", &messages, "items", &params(1.0)),
            "1 item for Sam"
        );
        assert_eq!(
            message("en", &messages, "items", &params(3.0)),
            "3 items for Sam"
        );
        assert_eq!(
            message("en", &messages, "items", &params(0.0)),
            "0 items for Sam"
        );
        assert_eq!(message("en", &messages, "plain", &params(2.0)), "hello Sam");
        assert_eq!(message("en", &messages, "missing", &params(2.0)), "missing");
    }

    #[test]
    fn the_locale_s_rules_pick_the_form_and_zero_is_none() {
        let messages: HashMap<String, String> = [
            ("files.zero", "No files"),
            ("files.one", "One file"),
            ("files.two", "Two files"),
            ("files.few", "{count} files (few)"),
            ("files.many", "{count} files (many)"),
            ("files.other", "{count} files"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        let n = |x: f64| vec![("count".to_string(), Static::Num(x))];
        assert_eq!(message("ar", &messages, "files", &n(2.0)), "Two files");
        assert_eq!(message("ar", &messages, "files", &n(5.0)), "5 files (few)");
        assert_eq!(
            message("ar", &messages, "files", &n(11.0)),
            "11 files (many)"
        );
        assert_eq!(message("ar", &messages, "files", &n(100.0)), "100 files");
        assert_eq!(
            message("en", &messages, "files", &n(2.0)),
            "2 files",
            "English has no `two`"
        );
        assert_eq!(
            message("en", &messages, "files", &n(0.0)),
            "No files",
            "exactly none, in any language"
        );
        assert_eq!(
            message("fr", &messages, "files", &n(0.0)),
            "No files",
            "though French counts 0 as `one`"
        );
        assert_eq!(message("fr", &messages, "files", &n(1.0)), "One file");
    }
}
