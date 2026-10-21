use std::path::Path;

use crate::assets::{self, Kind};
use crate::error::Result;
use crate::media::plan::size_text;

/// `wf assets [--unused] [--plan PATH] [--json]`: what `public/` holds and
/// where each file is named, or what the media pipeline will make of one
/// image.
pub fn run_assets(dir: &Path, plan: Option<&str>, unused: bool, json: bool) -> Result<()> {
    if let Some(path) = plan {
        let plan = assets::plan(dir, path)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&plan.as_json()).unwrap_or_default()
            );
            return Ok(());
        }
        println!(
            "{path}  {}×{} · {}",
            plan.width,
            plan.height,
            size_text(plan.original.bytes)
        );
        if !plan.pipeline {
            println!("  build.media.pipeline is off: it is copied as it is.");
            return Ok(());
        }
        for f in &plan.formats {
            match &f.error {
                Some(e) => println!("  {:<5} {e}", f.format),
                None => {
                    let widths: Vec<String> = f.files.iter().map(|f| f.width.to_string()).collect();
                    let about = if f.files.iter().any(|f| f.estimated) {
                        "about "
                    } else {
                        ""
                    };
                    println!(
                        "  {:<5} {:<24} {about}{}",
                        f.format,
                        widths.join(" · "),
                        size_text(f.bytes())
                    );
                }
            }
        }
        println!("  {}", plan.summary());
        println!(
            "  The original is written too, as {} ({}).",
            plan.original.name,
            size_text(plan.original.bytes)
        );
        if plan.estimated() {
            println!(
                "  A size marked `about` is estimated; `wf build` writes the files and makes it exact."
            );
        }
        return Ok(());
    }

    let listing = assets::list(dir)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&listing.as_json()).unwrap_or_default()
        );
        return Ok(());
    }
    if listing.assets.is_empty() {
        println!("public/ is empty: nothing is copied to the root of the build.");
        return Ok(());
    }
    let counts = listing.counts();
    let mut by_kind: Vec<String> = Kind::ALL
        .iter()
        .filter(|k| counts[k.word()] > 0)
        .map(|k| format!("{} {}", k.word(), counts[k.word()]))
        .collect();
    if counts["icon"] > 0 {
        by_kind.push(format!("icon {}", counts["icon"]));
    }
    by_kind.push(format!("unused {}", counts["unused"]));
    println!(
        "public/  {} files · {}   {}\n",
        listing.assets.len(),
        size_text(listing.bytes()),
        by_kind.join(" · ")
    );
    let wide = listing
        .assets
        .iter()
        .map(|a| a.path.chars().count())
        .max()
        .unwrap_or(0);
    for a in &listing.assets {
        if unused && !a.unused() {
            continue;
        }
        let size = match (a.width, a.height) {
            (Some(w), Some(h)) => format!("{w}×{h}"),
            _ => String::new(),
        };
        let used = if a.unused() {
            "unused".to_string()
        } else if a.uses.is_empty() {
            "by convention".to_string()
        } else {
            let n = a.used_in().len();
            format!(
                "used in {n} file{}{}",
                if n == 1 { "" } else { "s" },
                if a.processed() { ", processed" } else { "" }
            )
        };
        println!(
            "  {:<wide$}  {:<8} {:<10} {:>7}   {used}",
            a.path,
            a.kind.word(),
            size,
            size_text(a.bytes)
        );
    }
    if listing.unused() > 0 && !unused {
        println!(
            "\n{} nothing names: `wf assets --unused` lists them.",
            if listing.unused() == 1 {
                "1 file".to_string()
            } else {
                format!("{} files", listing.unused())
            }
        );
    }
    Ok(())
}
