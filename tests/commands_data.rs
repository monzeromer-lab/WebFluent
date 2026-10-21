//! C13: the `wf` commands as data are the command line: what the binary
//! says of itself is what the library holds; every option the data names
//! is one the binary takes, with the values it lists; and a command line
//! built from the data — as a tool page builds one — runs.

use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn wf(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("wf runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// Every command as it is typed, with its data.
fn leaves(data: &Value) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    for c in data["commands"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap().to_string();
        match c.get("subcommands").and_then(Value::as_array) {
            Some(subs) => {
                for s in subs {
                    out.push((format!("{name} {}", s["name"].as_str().unwrap()), s.clone()));
                }
            }
            None => out.push((name, c.clone())),
        }
    }
    out
}

#[test]
fn what_the_binary_says_of_itself_is_what_the_library_holds() {
    let (code, out, err) = wf(Path::new("."), &["commands", "--json"]);
    assert_eq!(code, 0, "{err}");
    let said: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        said,
        webfluent::commands::all(),
        "src/commands.json is stale: run `cargo test --bin wf`"
    );
}

#[test]
fn every_option_the_data_names_is_one_the_binary_takes() {
    let tmp = std::env::temp_dir();
    let mut checked = 0;
    for (name, c) in leaves(&webfluent::commands::all()) {
        let words: Vec<&str> = name.split(' ').collect();
        for o in c["options"].as_array().unwrap() {
            let Some(long) = o["long"].as_str() else {
                continue;
            };
            // The flag, with a value it takes when it takes one, before
            // `--help`: an option the binary did not know would be an error
            // before the help is printed.
            let mut args: Vec<String> = words.iter().map(|w| w.to_string()).collect();
            args.push(format!("--{long}"));
            if !o["flag"].as_bool().unwrap() {
                let value = match o["takes"].as_str().unwrap() {
                    "word" => o["values"][0].as_str().unwrap(),
                    "number" => "100",
                    "path" => ".",
                    "text" => o["default"].as_str().unwrap_or("x"),
                    other => panic!("`--{long}` takes `{other}`"),
                };
                args.push(value.to_string());
            }
            args.push("--help".to_string());
            let argv: Vec<&str> = args.iter().map(String::as_str).collect();
            let (code, out, err) = wf(&tmp, &argv);
            assert_eq!(code, 0, "`wf {}`: {err}", argv.join(" "));
            assert!(
                out.contains(&format!("--{long}")),
                "`wf {name} --help` names `--{long}`"
            );
            checked += 1;
        }
        // A value the data does not list is refused, where it lists them.
        for o in c["options"].as_array().unwrap() {
            if o["values"].as_array().is_some_and(|v| !v.is_empty())
                && let Some(long) = o["long"].as_str()
            {
                let mut args: Vec<String> = words.iter().map(|w| w.to_string()).collect();
                args.push(format!("--{long}"));
                args.push("not-a-value".to_string());
                let argv: Vec<&str> = args.iter().map(String::as_str).collect();
                let (code, _, err) = wf(&tmp, &argv);
                assert_ne!(code, 0, "`wf {}` was taken", argv.join(" "));
                assert!(err.contains("not-a-value"), "{err}");
            }
        }
    }
    assert!(checked > 40, "{checked} options");
    // The kinds a control is drawn for.
    let verify = webfluent::commands::command("verify").unwrap();
    let budget = verify["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["long"] == "budget")
        .unwrap();
    assert_eq!(budget["takes"], "number");
    let render = webfluent::commands::command("render").unwrap();
    let output = render["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["long"] == "output")
        .unwrap();
    assert_eq!(output["takes"], "path");
}

#[test]
fn a_command_line_built_from_the_data_runs() {
    let dir = std::env::temp_dir().join(format!("wf-commands-data-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "t" }"#).unwrap();
    // Written unformatted, so `fmt --check` has something to say.
    std::fs::write(
        dir.join("src/pages/Home.wf"),
        "page Home(path: \"/\", title: \"Home\", description: \"Home.\") {\n        Heading(\"Home\").h1\n}\n",
    )
    .unwrap();
    // As a tool page would: the command, then its preview flag, from the
    // data alone.
    let fmt = webfluent::commands::command("fmt").unwrap();
    let preview = fmt["preview"].as_str().unwrap();
    let before = std::fs::read_to_string(dir.join("src/pages/Home.wf")).unwrap();
    let (code, _, _) = wf(&dir, &["fmt", preview]);
    assert_ne!(code, 0, "a file that is not formatted fails the check");
    assert_eq!(
        std::fs::read_to_string(dir.join("src/pages/Home.wf")).unwrap(),
        before,
        "a preview writes nothing"
    );
    let (code, _, err) = wf(&dir, &["fmt"]);
    assert_eq!(code, 0, "{err}");
    let (code, _, _) = wf(&dir, &["fmt", preview]);
    assert_eq!(code, 0, "formatted now");
    // Every command that writes says so, and says how to see it first
    // where it can.
    for (name, c) in leaves(&webfluent::commands::all()) {
        if let Some(p) = c["preview"].as_str() {
            assert!(
                c["options"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|o| o["long"].as_str() == Some(p.trim_start_matches("--"))),
                "`wf {name}`'s preview `{p}` is one of its options"
            );
            assert!(
                !c["writes"].as_str().unwrap().is_empty(),
                "`wf {name}` previews what it writes"
            );
        }
    }
}
