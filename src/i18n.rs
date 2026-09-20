//! Messages with plural forms and placeholders, resolved at build time the
//! way the runtime resolves them: `t("items", { count: n })` reads
//! `items.one` when `n` is one and `items.other` otherwise (`items.zero`,
//! `items.two`, `items.few`, `items.many` when the file has them and the
//! locale's rules call for them — at build time the English rules stand
//! in), falling back to `items`; `{name}` in the message is filled from
//! the values given.

use std::collections::HashMap;

use crate::codegen::static_eval::Static;

/// The plural category of `count` under the English rules: `one` for
/// exactly one, `other` for the rest.
pub fn plural_category(count: f64) -> &'static str {
    if count == 1.0 { "one" } else { "other" }
}

/// The message for `key` over `params`.
pub fn message(
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
        let category = plural_category(n);
        text = messages
            .get(&format!("{key}.{category}"))
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
        assert_eq!(message(&messages, "items", &params(1.0)), "1 item for Sam");
        assert_eq!(message(&messages, "items", &params(3.0)), "3 items for Sam");
        assert_eq!(message(&messages, "items", &params(0.0)), "0 items for Sam");
        assert_eq!(message(&messages, "plain", &params(2.0)), "hello Sam");
        assert_eq!(message(&messages, "missing", &params(2.0)), "missing");
    }
}
