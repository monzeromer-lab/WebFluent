//! Validation as data, both ways, over every `validate` block the
//! repository writes — its projects and the guide's examples: each block is
//! read as the rows an inspector shows, in order and at their lines; read
//! back and written unchanged it moves no byte; a rule added and taken out
//! again gives the file back; a block is made beside its state and goes
//! with its last rule. The catalogue's messages are the runtime's, the
//! named patterns match what they say in any script, and a refined type's
//! bounds reach the runtime as rules. What a real browser does with all of
//! it is `tests/browser/validation.mjs`.

use std::path::{Path, PathBuf};

mod common;

use webfluent::parser::ast::{Declaration, Program, StatementKind};
use webfluent::validation::{
    Message, PATTERNS, RuleText, catalogue, move_rule, must_be, must_be_via, remove_rule,
    rule_texts, set_rule, write_rules,
};

/// Every `.wf` source that holds a `validate` block: the projects' files,
/// and each guide example that parses on its own.
fn sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    let mut stack: Vec<PathBuf> = ["examples", "tests/fixtures", "site/src"]
        .iter()
        .map(|d| root.join(d))
        .collect();
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if !p.ends_with("build") && !p.ends_with("node_modules") {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "wf")
                && let Ok(text) = std::fs::read_to_string(&p)
                && text.contains("validate ")
                && webfluent::parse_source(&text, &p.to_string_lossy()).is_ok()
            {
                out.push((p.to_string_lossy().to_string(), text));
            }
        }
    }
    for e in std::fs::read_dir(root.join("md-docs")).unwrap().flatten() {
        let text = std::fs::read_to_string(e.path()).unwrap();
        for (i, block) in text.split("```wf\n").skip(1).enumerate() {
            let code = block.split("\n```").next().unwrap_or("").to_string() + "\n";
            if code.contains("validate ") && webfluent::parse_source(&code, "guide.wf").is_ok() {
                out.push((format!("{}#{i}", e.path().display()), code));
            }
        }
    }
    out.sort();
    out
}

/// Every block in `program`: its owner, its state, and its rules' names.
fn blocks(program: &Program) -> Vec<(String, String, Vec<String>, Vec<u32>)> {
    let mut out = Vec::new();
    for d in &program.declarations {
        let (owner, body) = match d {
            Declaration::Page(p) => (&p.name, &p.body),
            Declaration::Component(c) => (&c.name, &c.body),
            Declaration::Store(s) => (&s.name, &s.body),
            _ => continue,
        };
        for s in body {
            if let StatementKind::Validate(v) = &s.kind {
                out.push((
                    owner.clone(),
                    v.name.clone(),
                    v.rules.iter().map(|r| r.name.clone()).collect(),
                    v.rules.iter().map(|r| r.span.line).collect(),
                ));
            }
        }
    }
    out
}

#[test]
fn every_block_is_read_as_its_rows_and_written_back_unchanged() {
    let mut read = 0;
    let mut wrong = Vec::new();
    for (name, source) in sources() {
        let program = webfluent::parse_source(&source, &name).unwrap();
        for (owner, state, rules, lines) in blocks(&program) {
            let Some(must) = must_be(&program, &owner, &state, false) else {
                // A block for a value that is not a `state` here — a prop, a
                // store's field — is not one the inspector edits.
                continue;
            };
            read += 1;
            let written: Vec<_> = must.rows.iter().filter(|r| !r.from_type).collect();
            let got: Vec<&str> = written.iter().map(|r| r.rule.as_str()).collect();
            if got != rules {
                wrong.push(format!(
                    "{name}: {owner}.{state} reads {got:?}, the block has {rules:?}"
                ));
            }
            if written.iter().map(|r| r.line).collect::<Vec<_>>() != lines {
                wrong.push(format!(
                    "{name}: {owner}.{state}'s rows are not at their lines"
                ));
            }
            assert!(must.checked && must.block.is_some());
            for r in &written {
                if r.rule == "pattern" && r.args.len() != 1 {
                    wrong.push(format!("{name}: a pattern with {} arguments", r.args.len()));
                }
            }
            // Read back and written as they are: not a byte moves.
            let texts = rule_texts(&source, &name, &owner, &state).unwrap();
            match write_rules(&source, &name, &owner, &state, &texts) {
                Ok(out) if out == source => {}
                Ok(_) => wrong.push(format!(
                    "{name}: {owner}.{state} written unchanged moves bytes"
                )),
                Err(e) => wrong.push(format!("{name}: {owner}.{state} refused as it is: {e}")),
            }
            // A rule added at the end is one new line inside the block, and
            // taking it out again gives the file back.
            let mut extra = RuleText::new("custom");
            extra.message = Some("\"No\"".into());
            extra.check = Some("true".into());
            let added = set_rule(&source, &name, &owner, &state, None, extra).unwrap();
            let after = webfluent::parse_source(&added, &name).unwrap();
            let now = must_be(&after, &owner, &state, false).unwrap();
            assert_eq!(now.rows.last().unwrap().rule, "custom", "{name}");
            assert_eq!(
                now.rows.last().unwrap().message,
                Message::Written { text: "No".into() }
            );
            if !rules.is_empty() {
                let back = remove_rule(&added, &name, &owner, &state, rules.len()).unwrap();
                if back != source {
                    wrong.push(format!(
                        "{name}: {owner}.{state} added and removed is not the file it was"
                    ));
                }
            }
            // Moved to the front and back again: the same file.
            if rules.len() >= 2 {
                let moved = move_rule(&source, &name, &owner, &state, 0, rules.len() - 1).unwrap();
                let order: Vec<String> = must_be(
                    &webfluent::parse_source(&moved, &name).unwrap(),
                    &owner,
                    &state,
                    false,
                )
                .unwrap()
                .rows
                .into_iter()
                .filter(|r| !r.from_type)
                .map(|r| r.rule)
                .collect();
                let mut want = rules.clone();
                let first = want.remove(0);
                want.push(first);
                assert_eq!(order, want, "{name}");
                let back = move_rule(&moved, &name, &owner, &state, rules.len() - 1, 0).unwrap();
                if back != source {
                    wrong.push(format!(
                        "{name}: {owner}.{state} moved and moved back is not the file it was"
                    ));
                }
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    assert!(read >= 15, "{read} blocks read");
}

const SIGNUP: &str =
    "page Signup(path: \"/join\", title: \"Join\", description: \"Make an account.\") {
    state email = \"\"
    state password = \"\"
    state confirm = \"\"

    validate email {
        required
        email
    }
    validate confirm {
        matches(password) \"The two passwords differ\"
    }

    Heading(\"Join\").h1
    Form(bind: form) {
        Input(bind: email, label: \"Email\").email
        Input(bind: password, label: \"Password\").password
        Input(bind: confirm, label: \"Confirm\").password
    }
}
";

#[test]
fn a_block_is_made_beside_its_state_and_goes_with_its_last_rule() {
    // Written as the states and then their blocks: among the blocks, in
    // the states' order.
    let mut min = RuleText::new("minLength");
    min.args = vec!["8".into()];
    min.message = Some("\"Use at least 8 characters\"".into());
    let out = set_rule(SIGNUP, "signup.wf", "Signup", "password", None, min).unwrap();
    let want = SIGNUP.replace(
        "    validate confirm {",
        "    validate password {\n        minLength(8) \"Use at least 8 characters\"\n    }\n    validate confirm {",
    );
    assert_eq!(out, want);
    // Its last rule taken out, the block goes, and the file is as it was.
    assert_eq!(
        remove_rule(&out, "signup.wf", "Signup", "password", 0).unwrap(),
        SIGNUP
    );

    // No blocks yet: after the last of the states written together.
    let bare = "page P(path: \"/\") {\n    state a = \"\"\n    state b = \"\"\n\n    Text(a)\n}\n";
    let out = set_rule(bare, "p.wf", "P", "a", None, RuleText::new("required")).unwrap();
    assert_eq!(
        out,
        "page P(path: \"/\") {\n    state a = \"\"\n    state b = \"\"\n    validate a {\n        required\n    }\n\n    Text(a)\n}\n"
    );

    // Each state with its block beside it: right after the state.
    let beside = "page P(path: \"/\") {\n    state a = \"\"\n    validate a {\n        required\n    }\n    state b = \"\"\n    state c = \"\"\n    Text(a)\n}\n";
    let out = set_rule(beside, "p.wf", "P", "b", None, RuleText::new("email")).unwrap();
    assert!(
        out.contains(
            "    state b = \"\"\n    validate b {\n        email\n    }\n    state c = \"\"\n"
        ),
        "{out}"
    );

    // What is not a rule, an argument that does not read, a state that is
    // not there: refused, and nothing written.
    let mut bad = RuleText::new("minLength");
    bad.args = vec!["8 +".into()];
    assert!(set_rule(SIGNUP, "signup.wf", "Signup", "email", None, bad).is_err());
    assert!(
        set_rule(
            SIGNUP,
            "signup.wf",
            "Signup",
            "email",
            None,
            RuleText::new("mandatory")
        )
        .is_err()
    );
    assert!(
        set_rule(
            SIGNUP,
            "signup.wf",
            "Signup",
            "nobody",
            None,
            RuleText::new("required")
        )
        .is_err()
    );
    assert!(remove_rule(SIGNUP, "signup.wf", "Signup", "email", 5).is_err());
}

#[test]
fn each_message_is_said_as_the_page_will_say_it() {
    let src = "page P(path: \"/\") {
    state name = \"\"
    state why = \"\"
    validate name {
        required
        minLength(2) t(\"signup.short\")
        maxLength(40) \"Keep it short\"
        pattern(/[0-9]/) \"Include a number\"
    }
    validate why {
        custom \"Not {why}\" { why != \"because\" }
    }
    Input(bind: name, label: \"Name\")
    Input(bind: why, label: \"Why\")
}
";
    let program = webfluent::parse_source(src, "p.wf").unwrap();
    let m = must_be(&program, "P", "name", true).unwrap();
    assert!(m.translated);
    let messages: Vec<&Message> = m.rows.iter().map(|r| &r.message).collect();
    assert_eq!(
        messages,
        [
            &Message::Rule {
                text: "This is required",
                key: "form.required".into()
            },
            &Message::Key {
                key: "signup.short".into()
            },
            &Message::Written {
                text: "Keep it short".into()
            },
            &Message::Written {
                text: "Include a number".into()
            },
        ]
    );
    assert_eq!(m.rows[3].pattern, Some("a number"));
    assert_eq!(m.rows[3].args, ["/[0-9]/"]);
    let why = must_be(&program, "P", "why", false).unwrap();
    assert_eq!(
        why.rows[0].message,
        Message::Expression {
            source: "\"Not {why}\"".into()
        }
    );
    assert_eq!(why.rows[0].check.as_deref(), Some("why != \"because\""));
    // The control bound to it names it.
    let Declaration::Page(p) = &program.declarations[0] else {
        unreachable!()
    };
    let _ = p;
    let input = webfluent::parse_source("page Q(path: \"/\") {\n    state password = \"\"\n    Input(bind: password, label: \"P\")\n}\n", "q.wf").unwrap();
    let Declaration::Page(q) = &input.declarations[0] else {
        unreachable!()
    };
    let el = q.body.iter().find_map(|s| match &s.kind {
        StatementKind::UIElement(e) => Some(e),
        _ => None,
    });
    assert_eq!(
        webfluent::validation::bound_state(el.unwrap()),
        Some("password")
    );
}

#[test]
fn the_messages_are_the_runtime_s() {
    let js = include_str!("../src/runtime/modules/form.js");
    let table = &js[js.find("const RULE_MESSAGES = {").unwrap()..];
    let table = &table[..table.find("};").unwrap()];
    let re = regex::Regex::new(r#"(?m)^\s*(\w+): "([^"]*)","#).unwrap();
    let runtime: Vec<(String, String)> = re
        .captures_iter(table)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect();
    let c = catalogue();
    let ours: Vec<(String, String)> = c
        .rules
        .iter()
        .chain(&c.implied)
        .map(|r| (r.name.to_string(), r.message.to_string()))
        .collect();
    assert_eq!(runtime, ours);
    for r in c.rules.iter().chain(&c.implied) {
        assert_eq!(
            r.key,
            format!("form.{}", r.name),
            "the key the runtime asks for"
        );
    }
}

#[test]
fn each_named_pattern_says_what_it_matches_in_any_script() {
    let cases: &[(&str, &[&str], &[&str])] = &[
        ("a number", &["abc1", "9"], &["abc", ""]),
        ("a letter", &["1a", "é", "ب", "日"], &["123", "!?"]),
        ("a capital letter", &["aB", "É"], &["ab", "123"]),
        ("a small letter", &["Ab", "é"], &["AB", "123"]),
        ("a symbol", &["a!", "x-y", "€"], &["abc", "a b", "١٢"]),
        ("no spaces", &["abc", ""], &["a b", "a\tb"]),
        ("only numbers", &["123", ""], &["12a", "1 2"]),
        ("only letters", &["abc", "عربي", "Ünïcode"], &["ab1", "a b"]),
    ];
    assert_eq!(cases.len(), PATTERNS.len());
    for (name, yes, no) in cases {
        let p = PATTERNS.iter().find(|p| p.name == *name).unwrap();
        for t in *yes {
            assert_eq!(
                webfluent::codegen::static_eval::matches(t, p.source, p.flags),
                Some(true),
                "{name} matches {t:?}"
            );
        }
        for t in *no {
            assert_eq!(
                webfluent::codegen::static_eval::matches(t, p.source, p.flags),
                Some(false),
                "{name} does not match {t:?}"
            );
        }
        // Written in a block, it parses, is recognised, and reaches the
        // page as the literal it is.
        let src = format!(
            "page P(path: \"/\") {{\n    state v = \"\"\n    validate v {{\n        pattern(/{}/{}) \"{}\"\n    }}\n    Input(bind: v, label: \"V\")\n}}\n",
            p.source, p.flags, p.message
        );
        let program =
            webfluent::parse_source(&src, "p.wf").unwrap_or_else(|e| panic!("{name}: {e}"));
        let m = must_be(&program, "P", "v", false).unwrap();
        assert_eq!(m.rows[0].pattern, Some(*name));
        let js = webfluent_js(&src);
        assert!(
            js.contains(&format!("args:[/{}/{}]", p.source, p.flags)),
            "{name}: {js}"
        );
    }
}

fn webfluent_js(src: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "wf-validation-{}-{}",
        std::process::id(),
        src.len() ^ src.bytes().map(|b| b as usize).sum::<usize>()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), "{\"name\":\"v\"}").unwrap();
    std::fs::write(dir.join("src/pages/P.wf"), src).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut js = String::new();
    let mut stack = vec![dir.join("build")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "js") {
                js.push_str(&std::fs::read_to_string(&p).unwrap());
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    js
}

#[test]
fn a_refined_type_s_bounds_reach_the_runtime_as_rules() {
    let src = "page P(path: \"/\") {
    state nights: Number(1..10) = 2
    state start: Date(after: @2026-01-01, before: @2027-01-01) = @2026-03-14
    validate nights { required }
    validate start { required }
    Input(bind: nights, label: \"Nights\").number
    Input(bind: start, label: \"Start\").date
}
";
    let js = webfluent_js(src);
    assert!(js.contains("{name:\"min\",args:[1]}"), "{js}");
    assert!(
        js.contains("{name:\"below\",args:[10]}"),
        "`1..10` does not take 10: {js}"
    );
    assert!(js.contains("{name:\"after\",args:["), "{js}");
    assert!(js.contains("{name:\"before\",args:["), "{js}");
    let program = webfluent::parse_source(src, "p.wf").unwrap();
    let m = must_be(&program, "P", "nights", false).unwrap();
    let rows: Vec<(&str, bool)> = m
        .rows
        .iter()
        .map(|r| (r.rule.as_str(), r.from_type))
        .collect();
    assert_eq!(rows, [("min", true), ("below", true), ("required", false)]);
    assert_eq!(m.ty.as_deref(), Some("Number(1..10)"));
    // Without a block, the type's rules are shown, and said not to be
    // checked as the reader types.
    let bare = "page P(path: \"/\") {\n    state n: Number(0..=5) = 1\n    Text(\"x\")\n}\n";
    let m = must_be(
        &webfluent::parse_source(bare, "p.wf").unwrap(),
        "P",
        "n",
        false,
    )
    .unwrap();
    assert!(!m.checked && m.block.is_none() && m.rows.len() == 2);
}

#[test]
fn an_unsaved_block_is_the_one_read() {
    let dir = std::env::temp_dir().join(format!("wf-validation-vfs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        "{\"name\":\"v\",\"i18n\":{\"locales\":[\"en\"]}}",
    )
    .unwrap();
    let file = dir.join("src/pages/Signup.wf");
    std::fs::write(&file, SIGNUP).unwrap();
    let saved = must_be_via(&webfluent::vfs::FsVfs, &dir, "Signup", "password")
        .unwrap()
        .unwrap();
    assert!(saved.rows.is_empty() && saved.translated);
    let mut rule = RuleText::new("minLength");
    rule.args = vec!["8".into()];
    let edited = set_rule(SIGNUP, "Signup.wf", "Signup", "password", None, rule).unwrap();
    let mut vfs = webfluent::vfs::OverlayVfs::over_disk();
    vfs.set(&file, edited);
    let unsaved = must_be_via(&vfs, &dir, "Signup", "password")
        .unwrap()
        .unwrap();
    assert_eq!(unsaved.rows.len(), 1);
    assert_eq!(
        unsaved.rows[0].message,
        Message::Rule {
            text: "Too short",
            key: "form.minLength".into()
        }
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), SIGNUP);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A pre-rendered control is the one the live page makes: its `name:` is
/// what a form sends it as before the script has loaded, and the author's
/// `id:` is the one its label points at — never a second `id` beside it.
#[test]
fn a_pre_rendered_field_keeps_its_name_and_its_own_id() {
    let src = common::page(
        "state email = \"\"\n    Input(bind: email, name: \"email\", label: \"Email\", id: \"mail\", autocomplete: \"email\").email\n    Input(name: \"plain\", label: \"Plain\")\n    Icon(name: \"search\")",
    );
    let html = common::ssg_html(&src);
    assert!(
        html.contains("<label class=\"wf-label\" for=\"mail\">Email</label>"),
        "{html}"
    );
    let input = html
        .split("<input")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert_eq!(input.matches(" id=").count(), 1, "one id: {input}");
    assert!(
        input.contains("id=\"mail\"")
            && input.contains("name=\"email\"")
            && input.contains("autocomplete=\"email\""),
        "{input}"
    );
    let plain = html
        .split("<input")
        .nth(2)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(
        plain.contains("name=\"plain\"") && plain.contains("id=\"wf-field-"),
        "{plain}"
    );
    assert!(
        !html.contains("name=\"search\""),
        "an icon's name is its glyph: {html}"
    );
}

#[test]
fn text_that_reads_as_another_rule_is_refused() {
    // An argument that closes the rule and opens another, a message that
    // is not one: each refused, so a field typed in an inspector can only
    // ever be the one rule it says.
    let mut sneaky = RuleText::new("minLength");
    sneaky.args = vec!["8) required email(".into()];
    assert!(set_rule(SIGNUP, "signup.wf", "Signup", "email", None, sneaky).is_err());
    let mut two = RuleText::new("required");
    two.message = Some("\"a\" email".into());
    assert!(set_rule(SIGNUP, "signup.wf", "Signup", "email", None, two).is_err());
    let mut check = RuleText::new("custom");
    check.message = Some("\"no\"".into());
    check.check = Some("true } validate email { required".into());
    assert!(set_rule(SIGNUP, "signup.wf", "Signup", "email", None, check).is_err());
}
