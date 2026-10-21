use std::path::Path;

use crate::error::Result;

/// `wf translations [-d DIR] [--missing LOCALE] [--unused] [--json]`: the
/// translations as a table — each locale with its plural forms and what it
/// lacks, each key with its messages and where a `t()` names it.
pub fn run_translations(
    dir: &Path,
    missing: Option<&str>,
    unused: bool,
    plurals: bool,
    json: bool,
) -> Result<()> {
    if plurals {
        return print_plurals(json);
    }
    let Some(table) = crate::i18n::table::table_via(&crate::vfs::FsVfs, dir)? else {
        println!("This project has no `i18n` in webfluent.app.json.");
        return Ok(());
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&table).unwrap_or_default()
        );
        return Ok(());
    }
    let total = table.keys.len();
    for l in &table.locales {
        println!(
            "{}{}{}  {} of {total} missing{}{}  forms: {}{}",
            l.code,
            if l.default { " (default)" } else { "" },
            if l.rtl { " rtl" } else { "" },
            l.missing,
            if l.forms_missing > 0 {
                format!(", {} lack a form", l.forms_missing)
            } else {
                String::new()
            },
            if l.mismatched > 0 {
                format!(", {} placeholders differ", l.mismatched)
            } else {
                String::new()
            },
            l.forms.join(" "),
            if l.exists {
                String::new()
            } else {
                format!("  — no {}", l.file)
            },
        );
    }
    let shown: Vec<&crate::i18n::table::Key> = table
        .keys
        .iter()
        .filter(|k| missing.is_none_or(|m| k.missing.iter().any(|x| x == m)))
        .filter(|k| !unused || k.unused)
        .collect();
    if missing.is_some() || unused {
        println!();
        for k in &shown {
            let at = k
                .uses
                .first()
                .map(|u| format!("  {}:{}", u.file, u.line))
                .unwrap_or_default();
            println!("  {}{}{at}", k.key, if k.plural { " (plural)" } else { "" });
        }
    } else {
        let unused_count = table.keys.iter().filter(|k| k.unused).count();
        println!("\n{total} keys, {unused_count} that nothing names");
    }
    for u in &table.unknown {
        println!(
            "{}:{}: t(\"{}\") names a key no locale has",
            u.file, u.line, u.key
        );
    }
    Ok(())
}

/// `--plurals`: CLDR's plural rules as the build and the page use them —
/// every locale's forms, and the form each of a run of counts takes.
fn print_plurals(json: bool) -> Result<()> {
    use crate::i18n::plurals;
    let mut counts: Vec<f64> = (0..=120).map(f64::from).collect();
    counts.extend([200.0, 1000.0, 1_000_000.0, 0.5, 1.5, 2.5]);
    let locales: Vec<&str> = crate::i18n::plurals_data::RULES
        .iter()
        .map(|(l, _)| *l)
        .filter(|l| *l != "root")
        .collect();
    if json {
        let forms: serde_json::Map<String, serde_json::Value> = locales
            .iter()
            .map(|l| (l.to_string(), serde_json::json!(plurals::categories(l))))
            .collect();
        let picks: serde_json::Map<String, serde_json::Value> = locales
            .iter()
            .map(|l| {
                let p: Vec<&str> = counts
                    .iter()
                    .map(|n| plurals::select_number(l, *n))
                    .collect();
                (l.to_string(), serde_json::json!(p))
            })
            .collect();
        let out = serde_json::json!({
            "cldr": crate::i18n::plurals_data::CLDR_VERSION,
            "counts": counts,
            "forms": forms,
            "picks": picks,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return Ok(());
    }
    println!("CLDR {}", crate::i18n::plurals_data::CLDR_VERSION);
    for l in locales {
        println!("  {l:<8} {}", plurals::categories(l).join(" "));
    }
    Ok(())
}
