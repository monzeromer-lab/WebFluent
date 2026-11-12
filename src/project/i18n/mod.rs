//! Messages with plural forms and placeholders, resolved at build time the
//! way the runtime resolves them: `t("items", { count: n })` reads the form
//! the locale's CLDR rules pick for `n` ([`plurals`]) — `items.one`,
//! `items.few` … — and `items.zero` for exactly 0 when the file has one,
//! in any language; then `items.other`, then `items`. `{name}` in the
//! message is filled from the values given. [`table`] is the translations
//! as an editor's table, both ways.

use std::collections::HashMap;
use std::path::Path;

pub mod literals;
pub mod plurals;
pub mod table;
#[rustfmt::skip]
pub mod plurals_data;

use crate::project::static_eval::Static;

/// Every locale's messages, read from `<dir>/<locale>.json`. A locale with
/// no file has none; a file that is not a JSON map of strings is an error.
pub fn load(
    project_dir: &Path,
    config: &crate::project::config::project::I18nConfig,
) -> crate::diagnostics::error::Result<HashMap<String, HashMap<String, String>>> {
    load_via(&crate::base::vfs::FsVfs, project_dir, config)
}

/// [`load`], reading through `vfs`.
pub fn load_via(
    vfs: &dyn crate::base::vfs::Vfs,
    project_dir: &Path,
    config: &crate::project::config::project::I18nConfig,
) -> crate::diagnostics::error::Result<HashMap<String, HashMap<String, String>>> {
    let mut out = HashMap::new();
    // The translations are the project's own: a directory outside it is
    // read as none (the build says so, as a finding on `i18n.dir`).
    if !crate::base::vfs::stays_inside(&config.dir) {
        return Ok(out);
    }
    let dir = project_dir.join(&config.dir);
    for locale in &config.locales {
        let file = dir.join(format!("{locale}.json"));
        let Ok(content) = vfs.read_to_string(&file) else {
            continue;
        };
        let messages: HashMap<String, String> = serde_json::from_str(&content).map_err(|e| {
            crate::diagnostics::error::WebFluentError::ConfigError(format!(
                "Failed to parse {locale}.json: {e}"
            ))
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
    let number = |v: &Static| match v {
        Static::Num(n) => Some(*n),
        _ => None,
    };
    let mut count = params
        .iter()
        .find(|(k, _)| k == "count")
        .and_then(|(_, v)| number(v));
    // Literal text names its number after the expression it splices —
    // `{files} files` — so a message with plural forms and exactly one
    // number takes its form by that number. Two numbers: none is guessed.
    let has_forms = || {
        ["zero", "one", "two", "few", "many", "other"]
            .iter()
            .any(|f| messages.contains_key(&format!("{key}.{f}")))
    };
    if count.is_none() && has_forms() {
        let numbers: Vec<f64> = params.iter().filter_map(|(_, v)| number(v)).collect();
        if let [n] = numbers[..] {
            count = Some(n);
        }
    }
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
    fn literal_text_takes_its_plural_form_by_its_one_number() {
        let ar: HashMap<String, String> = [
            ("{files} files.one", "ملف واحد"),
            ("{files} files.two", "ملفان"),
            ("{files} files.few", "{files} ملفات"),
            ("{files} files.many", "{files} ملفًا"),
            ("{files} files.other", "{files} ملف"),
            ("{a} of {b}", "{a} من {b}"),
            ("{a} of {b}.one", "wrong"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        let files = |n: f64| {
            message(
                "ar",
                &ar,
                "{files} files",
                &[("files".to_string(), Static::Num(n))],
            )
        };
        assert_eq!(files(1.0), "ملف واحد");
        assert_eq!(files(2.0), "ملفان");
        assert_eq!(files(3.0), "3 ملفات");
        assert_eq!(files(11.0), "11 ملفًا");
        assert_eq!(files(100.0), "100 ملف");
        let two = [
            ("a".to_string(), Static::Num(1.0)),
            ("b".to_string(), Static::Num(3.0)),
        ];
        assert_eq!(message("ar", &ar, "{a} of {b}", &two), "1 من 3");
    }

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
