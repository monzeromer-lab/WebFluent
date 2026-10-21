use std::path::Path;

use crate::error::Result;

/// `wf theme [-d DIR] [--json]`: the theme the build uses — every token,
/// declared or inherited, where each is used — and its contrast pairs.
pub fn run_theme(dir: &Path, json: bool) -> Result<()> {
    let view = crate::themes::view::view_via(&crate::vfs::FsVfs, dir)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&view).unwrap_or_default()
        );
        return Ok(());
    }
    match (&view.theme, &view.file, view.line) {
        (Some(t), Some(f), Some(l)) => println!("theme {t}   {f}:{l}"),
        _ => println!("No theme is declared: the baseline is used."),
    }
    if let Some(d) = &view.dark {
        println!("dark  {d}");
    }
    let mut group = "";
    for t in view
        .tokens
        .iter()
        .filter(|t| t.declared || t.dark.is_some())
    {
        if t.group != group {
            println!("\n{}", t.group);
            group = t.group;
        }
        let dark = t
            .dark
            .as_ref()
            .map(|d| format!("   dark {d}"))
            .unwrap_or_default();
        // Where the project's own sources name it; the built-in
        // stylesheet reads the baseline's names on top of that.
        let used = match t.uses.len() {
            0 => "named nowhere".to_string(),
            1 => "named once".to_string(),
            n => format!("named {n} times"),
        };
        println!("  {:<24} {:<28} {used}{dark}", t.name, t.value);
    }
    let inherited = view.tokens.iter().filter(|t| !t.declared).count();
    println!("\n{inherited} more inherited from the baseline (`--json` lists them).");
    println!("\ncontrast");
    for p in view.pairs.iter().filter(|p| p.drawn) {
        let ratio = p.ratio.map_or("—".to_string(), |r| format!("{r:.2}:1"));
        let mark = if p.passes() { "ok  " } else { "A13 " };
        println!(
            "  {mark} {ratio:>8}  {} (needs {}:1)",
            p.description, p.threshold
        );
    }
    if !view.unpaired.is_empty() {
        println!(
            "\nDeclared in one of the pair and not the other: {}",
            view.unpaired.join(", ")
        );
    }
    Ok(())
}
