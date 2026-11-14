//! The script a step plays in a page, as another host than Chrome runs it:
//! one expression, whatever the names hold.

use std::process::Command;

use webfluent::testing::step_script;

/// Whether `node` parses `src` as one expression.
fn parses(src: &str) -> Option<bool> {
    let check = format!(
        "new Function({});",
        serde_json::to_string(&format!("return {src}")).unwrap()
    );
    let out = Command::new("node").args(["-e", &check]).output().ok()?;
    Some(out.status.success())
}

#[test]
fn a_step_is_one_expression_whatever_its_name_holds() {
    for (what, name, value) in [
        ("click", "Save", ""),
        ("type", "Project name", "Halyard"),
        ("press", "", "Escape"),
        ("click", "Say \"hi\"\n</script>", ""),
        ("type", "Name", "a`${x}`\\u0000 '"),
    ] {
        let src = step_script(what, name, value);
        assert!(
            src.starts_with("(() => {") && src.ends_with("})()"),
            "{src}"
        );
        assert!(
            src.contains(&serde_json::to_string(name).unwrap()),
            "the name is a JSON string"
        );
        match parses(&src) {
            Some(ok) => assert!(ok, "does not parse: {src}"),
            None => eprintln!("node is not installed: the parse is not checked"),
        }
    }
}

/// The key events `press value` sends into a page whose focused element
/// records them, on `platform`: `[type, key, ctrl, shift, alt, meta]` each.
fn pressed(value: &str, platform: &str) -> Option<serde_json::Value> {
    let src = step_script("press", "", value);
    let harness = format!(
        r#"
const sent = [];
const el = {{ dispatchEvent(e) {{ sent.push([e.type, e.key, !!e.ctrlKey, !!e.shiftKey, !!e.altKey, !!e.metaKey]); return true; }} }};
Object.defineProperty(globalThis, "navigator", {{ value: {{ platform: {platform} }}, configurable: true }});
globalThis.document = {{ activeElement: el, body: el, querySelectorAll: () => [] }};
globalThis.KeyboardEvent = class {{ constructor(type, init) {{ Object.assign(this, init, {{ type }}); }} }};
globalThis.Event = class {{ constructor(type) {{ this.type = type; }} }};
{src};
console.log(JSON.stringify(sent));
"#,
        platform = serde_json::to_string(platform).unwrap(),
    );
    let out = Command::new("node").args(["-e", &harness]).output().ok()?;
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

#[test]
fn a_pressed_combination_holds_its_modifiers() {
    // `press "ctrl+k"` sent a key named "ctrl+k" with nothing held, so no
    // shortcut could be tested.
    let Some(sent) = pressed("ctrl+k", "Linux x86_64") else {
        eprintln!("node is not installed: the press is not checked");
        return;
    };
    assert_eq!(
        sent,
        serde_json::json!([
            ["keydown", "k", true, false, false, false],
            ["keyup", "k", true, false, false, false]
        ])
    );
    let mod_on_mac = pressed("mod+shift+p", "MacIntel").unwrap();
    assert_eq!(
        mod_on_mac[0],
        serde_json::json!(["keydown", "p", false, true, false, true]),
        "mod is ⌘ on a Mac"
    );
    let mod_elsewhere = pressed("mod+s", "Linux x86_64").unwrap();
    assert_eq!(
        mod_elsewhere[0],
        serde_json::json!(["keydown", "s", true, false, false, false]),
        "and Ctrl elsewhere"
    );
    let named = pressed("esc", "Linux x86_64").unwrap();
    assert_eq!(named[0][1], "Escape", "a short name is the key's");
    let plus = pressed("ctrl++", "Linux x86_64").unwrap();
    assert_eq!(
        plus[0],
        serde_json::json!(["keydown", "+", true, false, false, false]),
        "a plus is a key too"
    );
    let bare = pressed("Enter", "Linux x86_64").unwrap();
    assert_eq!(
        bare[0],
        serde_json::json!(["keydown", "Enter", false, false, false, false])
    );
}
