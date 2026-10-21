use std::path::Path;

use crate::error::Result;

/// `wf rules [-d DIR] [--json]`: every rule a `validate` block may hold,
/// the named patterns and when a form shows its messages; then every state
/// in the project held to a rule, with its rules in order.
pub fn run_rules(dir: &Path, json: bool) -> Result<()> {
    let catalogue = crate::validation::catalogue();
    let translated = crate::config::ProjectConfig::load(dir)
        .map(|c| c.i18n.is_some())
        .unwrap_or(false);
    let states = crate::build::read_project(dir)
        .ok()
        .map(|(p, _)| crate::validation::all(&p, translated))
        .unwrap_or_default();
    if json {
        let out = serde_json::json!({ "catalogue": catalogue, "states": states });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return Ok(());
    }
    println!("rules");
    for r in &catalogue.rules {
        println!(
            "  {:<10} {:<28} {}",
            r.name,
            format!("\"{}\"", r.message),
            r.key
        );
    }
    println!("\npatterns");
    for p in catalogue.patterns {
        println!("  {:<18} /{}/{}", p.name, p.source, p.flags);
    }
    println!("\nmessages show");
    for s in &catalogue.showing {
        println!(
            "  .{:<9} {}{}",
            s.name,
            s.phrase,
            if s.default { " (the default)" } else { "" }
        );
    }
    if !states.is_empty() {
        println!();
    }
    for m in &states {
        let rules: Vec<String> = m
            .rows
            .iter()
            .map(|r| {
                let args = if r.args.is_empty() {
                    String::new()
                } else {
                    format!("({})", r.args.join(", "))
                };
                format!(
                    "{}{args}{}",
                    r.rule,
                    if r.from_type { " [type]" } else { "" }
                )
            })
            .collect();
        println!(
            "{}.{}: {}{}",
            m.owner,
            m.state,
            rules.join(", "),
            if m.checked {
                ""
            } else {
                " (not checked as the reader types: no validate block)"
            }
        );
    }
    Ok(())
}
