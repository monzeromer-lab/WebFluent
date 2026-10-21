use std::path::Path;

use crate::error::Result;

/// `wf motion [-d DIR] [--json]`: the animations an element can play — the
/// engine's and the project's — the eases, the speeds, the route
/// transitions and the project's default.
pub fn run_motion(dir: &Path, json: bool) -> Result<()> {
    let program = crate::build::read_project(dir).ok().map(|(p, _)| p);
    let config = crate::config::ProjectConfig::load(dir)
        .map(|c| c.motion)
        .unwrap_or_default();
    let m = crate::motion::catalogue(program.as_ref(), &config);
    if json {
        println!("{}", serde_json::to_string_pretty(&m).unwrap_or_default());
        return Ok(());
    }
    println!("animations");
    for a in &m.animations {
        let mut notes = Vec::new();
        if a.leaves {
            notes.push("leaves");
        }
        if a.repeats {
            notes.push("repeats");
        }
        if a.project {
            notes.push("the project's");
        }
        let notes = if notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", notes.join(", "))
        };
        println!("  .{:<12} {}{notes}", a.name, a.summary);
    }
    println!("\neases");
    for e in &m.eases {
        println!("  .{:<12} {}", e.name, e.css);
    }
    println!("\nspeeds");
    for s in &m.speeds {
        println!(
            "  {:<8} {:<8} {}",
            s.name,
            s.value,
            s.flag.as_deref().unwrap_or("")
        );
    }
    println!("\nroute transitions: {}", m.route_transitions.join(", "));
    println!(
        "default: {} {}{}",
        m.default.duration,
        m.default.easing,
        if m.default.from_config {
            " (the config's)"
        } else {
            ""
        }
    );
    Ok(())
}
