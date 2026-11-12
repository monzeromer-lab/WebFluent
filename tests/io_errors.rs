//! A file that is there but cannot be read is an error that names it,
//! never taken for a file that is not there: a broken config does not
//! become the defaults, and an unreadable translation does not become a
//! locale with no messages.

use std::path::PathBuf;

/// A fresh project folder of this test's own.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-io-errors-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("public")).unwrap();
    dir
}

#[test]
fn a_broken_config_is_an_error_not_the_defaults() {
    let dir = project("broken-config");
    std::fs::write(dir.join("webfluent.app.json"), "{ not json").unwrap();
    let err = webfluent::assets::list(&dir).expect_err("a broken config is an error");
    assert!(err.to_string().contains("webfluent.app.json"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_project_with_no_config_is_listed_with_the_defaults() {
    let dir = project("no-config");
    assert!(webfluent::assets::list(&dir).is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_unreadable_translation_is_an_error_that_names_it() {
    let dir = project("bad-locale");
    let translations = dir.join("src/translations");
    std::fs::create_dir_all(&translations).unwrap();
    std::fs::write(translations.join("en.json"), r#"{ "hi": "Hello" }"#).unwrap();
    // Not UTF-8: a file that is there, and cannot be read as text.
    std::fs::write(translations.join("ar.json"), [0xff, 0xfe, 0x00, 0x7b]).unwrap();
    let config: webfluent::config::project::I18nConfig = serde_json::from_value(
        serde_json::json!({ "default_locale": "en", "locales": ["en", "ar", "fr"] }),
    )
    .unwrap();
    let err = webfluent::i18n::load(&dir, &config).expect_err("ar.json cannot be read");
    assert!(err.to_string().contains("ar.json"), "{err}");
    // `fr` has no file at all: that is a locale with no messages, not an
    // error, once the unreadable one is gone.
    std::fs::remove_file(translations.join("ar.json")).unwrap();
    let tables = webfluent::i18n::load(&dir, &config).expect("a missing locale is none");
    assert!(tables.contains_key("en") && !tables.contains_key("fr"));
    let _ = std::fs::remove_dir_all(&dir);
}
