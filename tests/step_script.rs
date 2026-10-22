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
