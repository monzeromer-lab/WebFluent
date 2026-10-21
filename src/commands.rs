//! The `wf` subcommands as data: each one's name, what it does, and its
//! options — their flags, what they take, their defaults and allowed
//! values — so a tool page, a palette or a script is built from the
//! command line itself rather than written out again.
//!
//! `commands.json` is written from the command line's own definition (the
//! `wf` binary's, by `wf commands --json`; a test of the binary fails when
//! it is stale), so this module needs no argument parser and is there
//! with the `cli` feature off. What each command writes, and the flag that
//! shows it first instead, is said here: [`EFFECTS`].

use serde_json::Value;

/// What a command writes, and how to see it first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Effect {
    /// The command, as it is typed: `fmt`, `config set`.
    pub command: &'static str,
    /// What it writes, in a few words; empty when it writes nothing.
    pub writes: &'static str,
    /// The flag that shows what it would write and writes nothing, when
    /// there is one: `--check`, `--dry-run`.
    pub preview: Option<&'static str>,
}

const fn e(command: &'static str, writes: &'static str, preview: Option<&'static str>) -> Effect {
    Effect {
        command,
        writes,
        preview,
    }
}

/// Every command's effect. A test of the binary holds this to its commands:
/// each has one entry, and each entry is a command.
pub const EFFECTS: &[Effect] = &[
    e("init", "a new project's folder", None),
    e("build", "the site, in the output folder", None),
    e("check", "", None),
    e("config schema", "", None),
    e("config get", "", None),
    e("config set", "webfluent.app.json", None),
    e("config unset", "webfluent.app.json", None),
    e("fix", "the project's source files", Some("--dry-run")),
    e("explain", "", None),
    e(
        "serve",
        "the site, in the output folder, on every save",
        None,
    ),
    e("generate", "a new source file", None),
    e("render", "the file `--output` names", None),
    e("migrate", "the project's source files", Some("--check")),
    e("docs", "a gallery, in the folder `--out` names", None),
    e("test", "the snapshots, with `--update`", None),
    e("audit", "", None),
    e("verify", "", None),
    e("registry", "", None),
    e("types", "", None),
    e("fmt", "the project's source files", Some("--check")),
    e("assets", "", None),
    e("components", "", None),
    e("commands", "", None),
];

/// What `command` writes, if it is a command.
pub fn effect(command: &str) -> Option<&'static Effect> {
    EFFECTS.iter().find(|e| e.command == command)
}

/// The commands, as `commands.json` holds them: each with `name`,
/// `summary`, `writes`, `preview` and `options`, and `subcommands` for one
/// that has them.
pub fn all() -> Value {
    serde_json::from_str(include_str!("commands.json")).expect("commands.json is JSON")
}

/// The command typed as `name` — `fmt`, `config set` — as data.
pub fn command(name: &str) -> Option<Value> {
    let mut words = name.split_whitespace();
    let first = words.next()?;
    let mut found = all()
        .get("commands")?
        .as_array()?
        .iter()
        .find(|c| c["name"] == first)?
        .clone();
    for word in words {
        found = found
            .get("subcommands")?
            .as_array()?
            .iter()
            .find(|c| c["name"] == word)?
            .clone();
    }
    Some(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_says_what_it_writes() {
        let all = all();
        let mut names = Vec::new();
        for c in all["commands"].as_array().unwrap() {
            let name = c["name"].as_str().unwrap().to_string();
            match c.get("subcommands").and_then(Value::as_array) {
                Some(subs) => {
                    for s in subs {
                        names.push(format!("{name} {}", s["name"].as_str().unwrap()));
                    }
                }
                None => names.push(name),
            }
        }
        for n in &names {
            assert!(
                effect(n).is_some(),
                "`wf {n}` has no entry in commands::EFFECTS"
            );
        }
        for e in EFFECTS {
            assert!(
                names.iter().any(|n| n == e.command),
                "`{}` in EFFECTS is no command",
                e.command
            );
        }
    }

    #[test]
    fn a_command_is_found_by_how_it_is_typed() {
        let fmt = command("fmt").expect("fmt");
        assert_eq!(fmt["preview"], "--check");
        let to = fmt["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["long"] == "to")
            .unwrap();
        assert_eq!(to["values"], serde_json::json!(["wfx", "wf"]));
        let set = command("config set").expect("config set");
        assert_eq!(set["writes"], "webfluent.app.json");
        assert!(command("config nothing").is_none());
        assert!(command("nothing").is_none());
    }
}
