//! W7: `persist`'s `key:` is followed as state, on a page and in a store,
//! and `in:` names a storage the project gives itself. What the runtime
//! does with them is `tests/js/keep.test.mjs` and
//! `tests/browser/storage.mjs`.

mod common;

use common::{page, spa_generated};

#[test]
fn a_page_s_key_is_a_function_the_runtime_follows() {
    let src = "store Project {\n    state id = \"halyard\"\n}\npage P(path: \"/\", title: \"T\") {\n    persist side = 240 {\n        key: Project.id\n        in: .app\n    }\n    Text(\"{side}\")\n}\n";
    let js = spa_generated(src);
    assert!(
        js.contains("WF.persist(\"P.side\", 240, { key: () => Project.id, in: \"app\" })"),
        "{js}"
    );
}

#[test]
fn a_store_s_key_is_kept_and_may_read_the_store() {
    let src = "store Layout {\n    state project = \"halyard\"\n    persist side = 240 {\n        key: project\n    }\n}\npage P(path: \"/\", title: \"T\") { Text(\"{Layout.side}\") }\n";
    let js = spa_generated(src);
    assert!(
        js.contains("side: { key: (store) => store.project },"),
        "a store's key used to be dropped: {js}"
    );
}

#[test]
fn in_names_any_storage_the_project_gives_itself() {
    for place in ["local", "session", "app", "extension"] {
        let src = page(&format!(
            "persist n = 0 {{\n        in: .{place}\n    }}\n    Text(\"{{n}}\")"
        ));
        assert!(
            webfluent::parse_source(&src, "t.wf").is_ok(),
            "`.{place}` is somewhere to keep it"
        );
    }
}
