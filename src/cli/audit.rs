//! `wf audit` — what this project trusts, in one place.
//!
//! A security review of a site is mostly a search: where does markup come
//! in, what other origins does it load from, what is written to the
//! reader's machine, and does the policy it ships match what it contains.
//! Every one of those answers is in the program the compiler already
//! reads, so it can be printed rather than hunted for.
//!
//! It reports; it never fails a build. What it finds is a conversation,
//! not a verdict.

use crate::config::ProjectConfig;
use crate::error::Result;
use crate::parser::ast::{
    Arg, ComponentRef, Declaration, Expr, Program, Statement, StatementKind, UIElement,
};
use std::path::Path;

pub fn run_audit(project_dir: &Path, json: bool) -> Result<()> {
    let mut config = ProjectConfig::load(project_dir)?;
    config.resolve_env(project_dir);
    let (program, files) = crate::cli::build::read_project_with(project_dir, None)?;
    let report = audit(&program, &config, &files);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        print(&report);
    }
    Ok(())
}

/// What a review would want to know, gathered from the program.
#[derive(Debug, Default, serde::Serialize)]
pub struct Report {
    /// Every `Unsafe.*`: where markup comes in, and whether it is
    /// sanitised on the way.
    pub unsafe_html: Vec<Finding>,
    /// Every origin the pages load from, beyond this site.
    pub origins: Vec<String>,
    /// Everything written to the reader's machine, and where.
    pub persisted: Vec<Persisted>,
    /// Every `env` name the pages read, and whether it is public.
    pub env: Vec<EnvName>,
    /// The Content-Security-Policy this build ships, or why it ships none.
    pub csp: String,
    /// What the compiler depends on, which is what a supply chain is.
    pub dependencies: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct Finding {
    pub at: String,
    pub sanitized: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct Persisted {
    pub name: String,
    pub owner: String,
    /// `local` (across visits) or `session` (this tab).
    pub storage: String,
}

#[derive(Debug, serde::Serialize)]
pub struct EnvName {
    pub name: String,
    pub public: bool,
}

pub fn audit(program: &Program, config: &ProjectConfig, files: &[String]) -> Report {
    let mut report = Report {
        csp: if config.build.csp {
            crate::config::project::csp_policy(config)
        } else {
            "none — `build.csp` is off, so the pages ship no policy".to_string()
        },
        dependencies: dependencies(),
        ..Default::default()
    };

    for (index, decl) in program.declarations.iter().enumerate() {
        let file = files.get(index).cloned().unwrap_or_else(|| "src/".into());
        let (owner, body): (String, &[Statement]) = match decl {
            Declaration::Page(p) => (p.name.clone(), &p.body),
            Declaration::Component(c) => (c.name.clone(), &c.body),
            Declaration::Store(s) => {
                let scope = s.scope.as_str();
                for stmt in &s.body {
                    if let StatementKind::State(state) = &stmt.kind
                        && state.persist
                    {
                        report.persisted.push(Persisted {
                            name: state.name.clone(),
                            owner: s.name.clone(),
                            storage: storage_of(state, scope),
                        });
                    }
                }
                (s.name.clone(), &s.body)
            }
            Declaration::App(a) => ("app".to_string(), &a.body),
            _ => continue,
        };
        for stmt in body {
            if let StatementKind::State(state) = &stmt.kind
                && state.persist
                && !matches!(decl, Declaration::Store(_))
            {
                report.persisted.push(Persisted {
                    name: state.name.clone(),
                    owner: owner.clone(),
                    storage: storage_of(state, "app"),
                });
            }
        }
        walk(body, &file, &mut report);
    }

    // The origins the config declares, beside the ones the pages name.
    for url in config
        .meta
        .fonts
        .iter()
        .chain(config.meta.stylesheets.iter())
    {
        if let Some(origin) = origin_of(url) {
            push_once(&mut report.origins, origin);
        }
    }
    report.origins.sort();
    report.origins.dedup();
    report.env.sort_by(|a, b| a.name.cmp(&b.name));
    report.env.dedup_by(|a, b| a.name == b.name);
    for name in config.env.keys() {
        if !report.env.iter().any(|e| &e.name == name) {
            report.env.push(EnvName {
                name: name.clone(),
                public: config.env_is_public(name),
            });
        }
    }
    for entry in &mut report.env {
        entry.public = config.env_is_public(&entry.name);
    }
    report.env.sort_by(|a, b| a.name.cmp(&b.name));
    report
}

fn storage_of(state: &crate::parser::ast::StateDecl, scope: &str) -> String {
    state
        .policy
        .as_ref()
        .and_then(|p| p.storage.clone())
        .unwrap_or_else(|| {
            if scope == "session" {
                "session".to_string()
            } else {
                "local".to_string()
            }
        })
}

fn walk(body: &[Statement], file: &str, report: &mut Report) {
    for stmt in body {
        match &stmt.kind {
            StatementKind::UIElement(el) => element(el, file, report),
            StatementKind::If(i) => {
                walk(&i.then_body, file, report);
                for (_, b) in &i.else_if_branches {
                    walk(b, file, report);
                }
                if let Some(b) = &i.else_body {
                    walk(b, file, report);
                }
            }
            StatementKind::For(f) => walk(&f.body, file, report),
            StatementKind::Show(s) => walk(&s.body, file, report),
            StatementKind::Match(m) => {
                for arm in &m.arms {
                    walk(&arm.body, file, report);
                }
            }
            _ => {}
        }
    }
}

fn element(el: &UIElement, file: &str, report: &mut Report) {
    let door = matches!(&el.component, ComponentRef::SubComponent(o, _) if o == "Unsafe")
        || matches!(&el.component, ComponentRef::BuiltIn(n) if n == "UnsafeHtml");
    if door {
        report.unsafe_html.push(Finding {
            at: format!("{file}:{}", el.span.line),
            sanitized: el
                .args
                .iter()
                .any(|a| matches!(a, Arg::Positional(Expr::FunctionCall(f, _)) if f == "sanitize")),
        });
    }
    for arg in &el.args {
        let (Arg::Named(_, value) | Arg::Positional(value)) = arg;
        if let Expr::StringLiteral(text) = value
            && let Some(origin) = origin_of(text)
        {
            push_once(&mut report.origins, origin);
        }
        if let Expr::PropertyAccess(base, name) = value
            && matches!(base.as_ref(), Expr::Identifier(n) if n == "env")
        {
            report.env.push(EnvName {
                name: name.clone(),
                public: false,
            });
        }
    }
    walk(&el.children, file, report);
    for fill in &el.slot_fills {
        walk(&fill.body, file, report);
    }
}

/// The origin `url` names, when it names one somewhere else.
fn origin_of(url: &str) -> Option<String> {
    let at = url.find("://")?;
    let scheme = &url[..at];
    if !scheme.chars().all(|c| c.is_ascii_alphabetic()) || scheme.is_empty() {
        return None;
    }
    let host = url[at + 3..].split('/').next()?;
    (!host.is_empty()).then(|| format!("{scheme}://{host}"))
}

fn push_once(list: &mut Vec<String>, value: String) {
    if !list.contains(&value) {
        list.push(value);
    }
}

/// What the compiler itself depends on. A supply chain is a thing you can
/// only reason about if you can see it, and this one is short on purpose.
fn dependencies() -> Vec<String> {
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    let mut out = Vec::new();
    let mut inside = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == "[dependencies]";
            continue;
        }
        if !inside || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, rest)) = line.split_once('=') {
            out.push(format!("{} {}", name.trim(), rest.trim()));
        }
    }
    out
}

fn print(report: &Report) {
    println!("\n  What this project trusts\n");

    println!("  Markup a page did not write");
    if report.unsafe_html.is_empty() {
        println!("    none — every string in this project goes in as text");
    }
    for finding in &report.unsafe_html {
        println!(
            "    {}  {}",
            finding.at,
            if finding.sanitized {
                "through `sanitize`"
            } else {
                "NOT sanitised"
            }
        );
    }

    println!("\n  Other origins");
    if report.origins.is_empty() {
        println!("    none — everything is served from this site");
    }
    for origin in &report.origins {
        println!("    {origin}");
    }

    println!("\n  Written to the reader's machine");
    if report.persisted.is_empty() {
        println!("    nothing");
    }
    for kept in &report.persisted {
        println!(
            "    {}.{}  in {}Storage",
            kept.owner, kept.name, kept.storage
        );
    }

    println!("\n  `env`");
    if report.env.is_empty() {
        println!("    none");
    }
    for name in &report.env {
        println!(
            "    {}  {}",
            name.name,
            if name.public {
                "public — it is in the bundle"
            } else {
                "not public — the pages may not read it"
            }
        );
    }

    println!("\n  Content-Security-Policy");
    println!("    {}", report.csp);

    println!("\n  What the compiler depends on");
    for dep in &report.dependencies {
        println!("    {dep}");
    }
    println!();
}
