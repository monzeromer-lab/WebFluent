//! The scope as a path picker lists it, over every project in the
//! repository (and `WF_SCOPE_PROJECTS` — Halyard): at the start of every
//! element, the picker lists exactly the values the editor's completion
//! scope holds there, each typed as the checker types it, in the picker's
//! order; an action's `let` is its own, never the page's or a store's
//! member; and on a page built for it, each row is the one it should be —
//! its group, its type, where it comes from, the fields to go on into —
//! with a template's data after it and an unsaved edit the one read.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use webfluent::parser::ast::{Declaration, Statement, StatementKind};
use webfluent::scope::paths::{self, Group, Shape};
use webfluent::scope::{BindingKind, body_of, child_bodies, scope_at};

fn projects() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = vec![root.join("site")];
    for base in [
        "examples/tutorials",
        "examples/decks",
        "examples/documents",
        "tests/fixtures",
    ] {
        if let Ok(entries) = std::fs::read_dir(root.join(base)) {
            let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            dirs.sort();
            out.extend(
                dirs.into_iter()
                    .filter(|d| d.join("webfluent.app.json").is_file()),
            );
        }
    }
    if let Ok(more) = std::env::var("WF_SCOPE_PROJECTS") {
        out.extend(more.split(':').filter(|d| !d.is_empty()).map(PathBuf::from));
    }
    out
}

/// Every element's start in a body, at any depth, drawn or run.
fn places(stmts: &[Statement], out: &mut Vec<usize>) {
    for s in stmts {
        if matches!(s.kind, StatementKind::UIElement(_)) {
            out.push(s.span.start as usize + 1);
        }
        for b in child_bodies(s) {
            places(b, out);
        }
    }
}

/// What the picker lists, of everything the completion scope holds.
fn readable(kind: BindingKind) -> bool {
    !matches!(
        kind,
        BindingKind::Action
            | BindingKind::Store
            | BindingKind::Socket
            | BindingKind::Stream
            | BindingKind::Channel
            | BindingKind::Peer
            | BindingKind::Worker
            | BindingKind::ElementHandle
    )
}

#[test]
fn the_picker_lists_what_completion_holds_typed_as_the_checker_types_it() {
    let mut checked = 0;
    let mut wrong = Vec::new();
    for dir in projects() {
        let Ok((program, files)) = webfluent::build::read_project(&dir) else {
            continue;
        };
        let info =
            webfluent::sema::types::check(&program, &|i| files.get(i).cloned().unwrap_or_default());
        for (ix, decl) in program.declarations.iter().enumerate() {
            let mut at = Vec::new();
            places(body_of(decl), &mut at);
            // A sample of each declaration's places, so a large project is
            // read in a reasonable time.
            let step = (at.len() / 25).max(1);
            for offset in at.into_iter().step_by(step) {
                checked += 1;
                let rows = paths::at(&program, &info, ix, offset);
                let listed: HashSet<&str> = rows
                    .iter()
                    .filter(|r| r.group != Group::Stores)
                    .map(|r| r.path.as_str())
                    .collect();
                let scope = scope_at(decl, offset);
                let mut want: HashSet<&str> = HashSet::new();
                for b in scope.iter().filter(|b| readable(b.kind)) {
                    want.insert(b.name.as_str());
                }
                if listed != want {
                    let missing: Vec<_> = want.difference(&listed).collect();
                    let extra: Vec<_> = listed.difference(&want).collect();
                    wrong.push(format!(
                        "{}: {} at {offset}: missing {missing:?}, extra {extra:?}",
                        dir.display(),
                        files[ix]
                    ));
                }
                // The picker's order.
                if rows.windows(2).any(|w| w[0].group > w[1].group) {
                    wrong.push(format!(
                        "{}: {} at {offset}: out of order",
                        dir.display(),
                        files[ix]
                    ));
                }
                // Each own row is typed as the checker typed its name.
                for r in rows.iter().filter(|r| {
                    matches!(r.kind, Some(BindingKind::State | BindingKind::Derived))
                        && r.group == Group::Own
                }) {
                    if let Some(b) = scope.iter().find(|b| b.name == r.name)
                        && let Some(t) = info.type_at(ix, &b.name, b.span)
                        && r.ty != t.to_string()
                        && !t.is_any()
                    {
                        wrong.push(format!(
                            "{}: {} is {} in the picker, {t} to the checker",
                            files[ix], r.name, r.ty
                        ));
                    }
                }
                // Every field path reads as the row it hangs from.
                fn under(rows: &[paths::Entry], wrong: &mut Vec<String>) {
                    for r in rows {
                        for m in &r.members {
                            if m.path != format!("{}.{}", r.path, m.name) {
                                wrong.push(format!("{} under {}", m.path, r.path));
                            }
                        }
                        under(&r.members, wrong);
                    }
                }
                under(&rows, &mut wrong);
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} wrong:\n{}",
        wrong.len(),
        wrong
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(checked > 2_000, "{checked} places");
}

#[test]
fn an_action_s_let_is_its_own() {
    let mut actions = 0;
    let mut wrong = Vec::new();
    for dir in projects() {
        let Ok((program, files)) = webfluent::build::read_project(&dir) else {
            continue;
        };
        for (ix, decl) in program.declarations.iter().enumerate() {
            let body = body_of(decl);
            let own: HashSet<&str> = body
                .iter()
                .filter_map(|s| match &s.kind {
                    StatementKind::State(d) => Some(d.name.as_str()),
                    _ => None,
                })
                .collect();
            for s in body {
                let StatementKind::Action(a) = &s.kind else {
                    continue;
                };
                let lets: Vec<(&str, usize)> = a
                    .body
                    .iter()
                    .filter_map(|t| match &t.kind {
                        StatementKind::State(d) if !own.contains(d.name.as_str()) => {
                            Some((d.name.as_str(), t.span.end as usize))
                        }
                        _ => None,
                    })
                    .collect();
                if lets.is_empty() {
                    continue;
                }
                actions += 1;
                // Outside the action — at the declaration's first element —
                // none of its lets is in scope.
                let mut at = Vec::new();
                places(body, &mut at);
                if let Some(outside) = at
                    .iter()
                    .find(|o| **o < s.span.start as usize || **o > s.span.end as usize)
                {
                    let names: Vec<String> = scope_at(decl, *outside)
                        .into_iter()
                        .map(|b| b.name)
                        .collect();
                    for (l, _) in &lets {
                        if names.iter().any(|n| n == l) {
                            wrong.push(format!(
                                "{}: `{l}` of {} is in scope outside it",
                                files[ix], a.name
                            ));
                        }
                    }
                }
                // After it, inside the action, it is.
                let (name, end) = lets[0];
                if end < s.span.end as usize {
                    let names: Vec<String> = scope_at(decl, end + 1)
                        .into_iter()
                        .map(|b| b.name)
                        .collect();
                    if !names.iter().any(|n| n == name) {
                        wrong.push(format!(
                            "{}: `{name}` is not in scope after it in {}",
                            files[ix], a.name
                        ));
                    }
                }
            }
            // A store's members are its own, not its actions' lets.
            if let Declaration::Store(store) = decl {
                let members: HashSet<String> = webfluent::scope::store_members(store)
                    .into_iter()
                    .map(|b| b.name)
                    .collect();
                for s in &store.body {
                    if let StatementKind::Action(a) = &s.kind {
                        for t in &a.body {
                            if let StatementKind::State(d) = &t.kind
                                && members.contains(&d.name)
                                && !own.contains(d.name.as_str())
                            {
                                wrong.push(format!(
                                    "{}: {}.{} is a let of {}",
                                    files[ix], store.name, d.name, a.name
                                ));
                            }
                        }
                    }
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
    assert!(actions >= 10, "{actions} actions with lets");
}

const DEPLOYS: &str = r#"type Region {
    name: String
    code: String
}

type Deploy {
    id: Number
    name: String
    live: Bool
    region: Region
    started: Date
    cost: Money?
}

type Node {
    label: String
    children: [Node]
    parent: Node?
}

enum Stage { draft, live, retired }

store Deploys {
    state envs: [String] = ["prod", "staging"]
    state items: [Deploy] = []
    derived count = items.length
    action add(d: Deploy) {
        let copy = items.slice()
        copy.push(d)
        items = copy
    }
}

component DeployRow(deploy: Deploy, compact: Bool = false) {
    Text(deploy.name)
}

page DeployList(path: "/deploys/:team", team: String, title: "Deploys", description: "Every deploy.") {
    use Deploys
    state stage: Stage = .draft
    state tree: Node? = null
    derived live = Deploys.items.filter(d => d.live)
    action pick(e: String) {
        let chosen = e.trim()
        log(chosen)
    }
    Heading("Deploys").h1
    Form(bind: form) {
        Input(bind: team, label: "Team")
    }
    for e, i in Deploys.envs {
        Button(e) { on click { pick(e) } }
        for d in live {
            DeployRow(deploy: d)
        }
    }
}
"#;

fn row<'a>(rows: &'a [paths::Entry], path: &str) -> &'a paths::Entry {
    fn find<'a>(rows: &'a [paths::Entry], path: &str) -> Option<&'a paths::Entry> {
        rows.iter().find_map(|r| {
            if r.path == path {
                Some(r)
            } else {
                find(&r.members, path)
            }
        })
    }
    find(rows, path).unwrap_or_else(|| {
        panic!(
            "no row {path}: {:#?}",
            rows.iter().map(|r| &r.path).collect::<Vec<_>>()
        )
    })
}

#[test]
fn each_row_is_the_one_it_should_be() {
    let program = webfluent::parse_source(DEPLOYS, "deploys.wf").unwrap();
    let info = webfluent::sema::types::check(&program, &|_| "deploys.wf".into());
    let ix = program
        .declarations
        .iter()
        .position(|d| matches!(d, Declaration::Page(_)))
        .unwrap();
    let at = DEPLOYS.find("DeployRow(deploy: d)").unwrap() + 1;
    let rows = paths::at(&program, &info, ix, at);
    let top: Vec<(&str, Group)> = rows.iter().map(|r| (r.path.as_str(), r.group)).collect();
    assert_eq!(
        top,
        [
            ("stage", Group::Own),
            ("tree", Group::Own),
            ("live", Group::Own),
            ("form", Group::Own),
            ("team", Group::Props),
            ("d", Group::Bindings),
            ("e", Group::Bindings),
            ("i", Group::Bindings),
            ("Deploys.envs", Group::Stores),
            ("Deploys.items", Group::Stores),
            ("Deploys.count", Group::Stores),
        ],
        "own, props, the bindings innermost first, then the stores' members; never an action, nor its lets"
    );
    let stage = row(&rows, "stage");
    assert_eq!(
        (stage.ty.as_str(), stage.shape, stage.from.as_str()),
        ("Stage", Shape::Enum, "state in DeployList")
    );
    let live = row(&rows, "live");
    assert_eq!(
        (live.shape, live.item.as_deref(), live.from.as_str()),
        (Shape::List, Some("Deploy"), "derived in DeployList")
    );
    let d = row(&rows, "d");
    assert_eq!(
        (d.ty.as_str(), d.from.as_str()),
        ("Deploy", "for d in live")
    );
    assert_eq!(row(&rows, "d.region.code").ty, "String");
    assert_eq!(row(&rows, "d.started").shape, Shape::Moment);
    let cost = row(&rows, "d.cost");
    assert!(cost.optional);
    assert_eq!(row(&rows, "d.cost.amount").shape, Shape::Number);
    let e = row(&rows, "e");
    assert_eq!(
        (e.ty.as_str(), e.from.as_str()),
        ("String", "for e, i in Deploys.envs")
    );
    assert_eq!(
        (row(&rows, "i").shape, row(&rows, "i").from.as_str()),
        (Shape::Number, "for e, i in Deploys.envs")
    );
    let team = row(&rows, "team");
    assert_eq!(
        (team.ty.as_str(), team.from.as_str()),
        ("String", "the route's :team")
    );
    let form = row(&rows, "form");
    assert_eq!(
        (form.shape, row(&rows, "form.valid").shape),
        (Shape::Form, Shape::Bool)
    );
    let count = row(&rows, "Deploys.count");
    assert_eq!(
        (count.shape, count.from.as_str()),
        (Shape::Number, "member of Deploys")
    );
    // A record that holds itself is followed a few levels, then stops.
    let tree = row(&rows, "tree");
    assert!(tree.optional);
    assert_eq!(row(&rows, "tree.parent.label").ty, "String");
    fn depth(r: &paths::Entry) -> usize {
        1 + r.members.iter().map(depth).max().unwrap_or(0)
    }
    assert!(depth(tree) <= 4, "{}", depth(tree));

    // Inside an action: its parameter and its let, before the page's.
    let at = DEPLOYS.find("log(chosen)").unwrap() + 1;
    let rows = paths::at(&program, &info, ix, at);
    let chosen = row(&rows, "chosen");
    assert_eq!(
        (chosen.group, chosen.from.as_str()),
        (Group::Bindings, "let in pick")
    );
    assert_eq!(row(&rows, "e").from, "parameter of pick");

    // A component's props, with their types.
    let cx = program
        .declarations
        .iter()
        .position(|d| matches!(d, Declaration::Component(_)))
        .unwrap();
    let rows = paths::at(
        &program,
        &info,
        cx,
        DEPLOYS.find("Text(deploy.name)").unwrap() + 1,
    );
    let deploy = row(&rows, "deploy");
    assert_eq!(
        (deploy.group, deploy.ty.as_str(), deploy.from.as_str()),
        (Group::Props, "Deploy", "prop of DeployRow")
    );
    assert_eq!(row(&rows, "compact").shape, Shape::Bool);
}

#[test]
fn a_template_s_data_is_a_tree() {
    let data: serde_json::Value = serde_json::from_str(
        r#"{ "title": "Invoice", "paid": false, "issued": "2026-03-14", "total": 1299,
             "customer": { "name": "Ada", "address": { "city": "Leeds" } },
             "lines": [ { "item": "Hosting", "qty": 1 }, { "item": "Support", "price": 40 } ],
             "notes": null }"#,
    )
    .unwrap();
    let rows = paths::data(&data);
    assert!(
        rows.iter()
            .all(|r| r.group == Group::Data && r.from == "the template's data")
    );
    assert_eq!(row(&rows, "title").shape, Shape::Text);
    assert_eq!(row(&rows, "paid").shape, Shape::Bool);
    assert_eq!(row(&rows, "issued").shape, Shape::Moment);
    assert_eq!(row(&rows, "customer.address.city").ty, "String");
    let lines = row(&rows, "lines");
    assert_eq!(
        (lines.shape, lines.item.as_deref()),
        (Shape::List, Some("Map"))
    );
    let keys: Vec<&str> = lines.members.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(
        keys,
        ["item", "price", "qty"],
        "every key any item has, in name order"
    );
    assert!(row(&rows, "notes").optional);
}

#[test]
fn an_unsaved_edit_is_the_one_read_and_the_command_prints_it() {
    let dir = std::env::temp_dir().join(format!("wf-scope-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), "{\"name\":\"s\"}").unwrap();
    let file = dir.join("src/pages/Deploys.wf");
    std::fs::write(&file, DEPLOYS).unwrap();
    let at = DEPLOYS.find("DeployRow(deploy: d)").unwrap() + 1;
    let saved = paths::at_via(&webfluent::vfs::FsVfs, &dir, "src/pages/Deploys.wf", at).unwrap();
    assert!(!saved.iter().any(|r| r.path == "picked"));
    let edited = DEPLOYS.replacen(
        "    state stage: Stage = .draft\n",
        "    state stage: Stage = .draft\n    state picked = 0\n",
        1,
    );
    let mut vfs = webfluent::vfs::OverlayVfs::over_disk();
    vfs.set(&file, edited.clone());
    let unsaved = paths::at_via(
        &vfs,
        &dir,
        "src/pages/Deploys.wf",
        edited.find("DeployRow(deploy: d)").unwrap() + 1,
    )
    .unwrap();
    assert_eq!(row(&unsaved, "picked").shape, Shape::Number);

    let line = DEPLOYS[..at].lines().count();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args([
            "scope",
            &format!("src/pages/Deploys.wf:{line}:13"),
            "--json",
            "-d",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rows: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let paths: Vec<&str> = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["path"].as_str().unwrap())
        .collect();
    assert!(
        paths.contains(&"d") && paths.contains(&"Deploys.items"),
        "{paths:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
