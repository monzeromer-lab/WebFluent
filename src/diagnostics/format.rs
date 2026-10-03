//! Findings for a tool: `--format json`, `sarif` (GitHub code scanning and
//! every other SARIF reader) and `github` (`::error` annotations on a pull
//! request), beside the human renderer.

use super::{Diagnostic, Severity, codes};
use serde_json::{Value, json};

/// How a command writes its findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Format {
    /// For a person, with source lines (`render::human`).
    #[default]
    Human,
    /// Every finding as the `Diagnostic` struct, and the counts.
    Json,
    /// SARIF 2.1.0.
    Sarif,
    /// GitHub Actions workflow commands.
    Github,
}

impl std::str::FromStr for Format {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "human" => Ok(Format::Human),
            "json" => Ok(Format::Json),
            "sarif" => Ok(Format::Sarif),
            "github" => Ok(Format::Github),
            other => Err(format!(
                "`{other}` is not a format: `human`, `json`, `sarif` or `github`"
            )),
        }
    }
}

/// `{ "diagnostics": [ … ], "errors": n, "warnings": m }`.
pub fn json(diagnostics: &[Diagnostic]) -> String {
    let (errors, warnings) = super::counts(diagnostics);
    serde_json::to_string_pretty(&json!({
        "diagnostics": diagnostics,
        "errors": errors,
        "warnings": warnings,
    }))
    .unwrap_or_default()
}

fn level(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    }
}

fn region(line: usize, column: usize, end_line: usize, end_column: usize) -> Value {
    let mut r = json!({ "startLine": line.max(1), "startColumn": column.max(1) });
    if end_line > 0 {
        r["endLine"] = json!(end_line);
        r["endColumn"] = json!(end_column);
    }
    r
}

/// SARIF 2.1.0: one run of `wf`, a rule for every code that appears, and a
/// result per finding with its place, its related places and its fixes.
pub fn sarif(diagnostics: &[Diagnostic]) -> String {
    let mut seen: Vec<&str> = Vec::new();
    for d in diagnostics {
        if !d.code.is_empty() && !seen.contains(&d.code) {
            seen.push(d.code);
        }
    }
    let rules: Vec<Value> = seen
        .iter()
        .filter_map(|code| {
            let info = codes::info(code)?;
            Some(json!({
                "id": info.code,
                "shortDescription": { "text": info.title },
                "fullDescription": { "text": info.summary },
                "helpUri": codes::docs_url(info.code),
                "defaultConfiguration": { "level": level(info.severity) },
            }))
        })
        .collect();
    let results: Vec<Value> = diagnostics
        .iter()
        .map(|d| {
            let text = match &d.hint {
                Some(h) => format!("{}\n{h}", d.message),
                None => d.message.clone(),
            };
            let mut r = json!({
                "ruleId": d.code,
                "level": level(d.severity),
                "message": { "text": text },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": d.file },
                        "region": region(d.line, d.column, d.end_line, d.end_column),
                    }
                }],
            });
            if let Some(index) = seen.iter().position(|c| *c == d.code) {
                r["ruleIndex"] = json!(index);
            }
            if !d.related.is_empty() {
                r["relatedLocations"] = Value::Array(
                    d.related
                        .iter()
                        .enumerate()
                        .map(|(i, rel)| {
                            json!({
                                "id": i,
                                "message": { "text": rel.message },
                                "physicalLocation": {
                                    "artifactLocation": { "uri": rel.file },
                                    "region": region(rel.line, rel.column, 0, 0),
                                }
                            })
                        })
                        .collect(),
                );
            }
            if !d.fixes.is_empty() {
                r["fixes"] = Value::Array(
                    d.fixes
                        .iter()
                        .map(|f| {
                            json!({
                                "description": { "text": f.title },
                                "artifactChanges": f.edits.iter().map(|e| json!({
                                    "artifactLocation": { "uri": e.file },
                                    "replacements": [{
                                        "deletedRegion": region(e.line, e.column, e.end_line, e.end_column),
                                        "insertedContent": { "text": e.text },
                                    }],
                                })).collect::<Vec<_>>(),
                            })
                        })
                        .collect(),
                );
            }
            r
        })
        .collect();
    serde_json::to_string_pretty(&json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": {
                "name": "wf",
                "version": env!("CARGO_PKG_VERSION"),
                "informationUri": "https://monzeromer-lab.github.io/WebFluent/",
                "rules": rules,
            }},
            "results": results,
        }],
    }))
    .unwrap_or_default()
}

/// A workflow command's data, escaped as GitHub reads it.
fn escape_data(s: &str) -> String {
    s.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

/// A workflow command's property, escaped as GitHub reads it.
fn escape_property(s: &str) -> String {
    escape_data(s).replace(':', "%3A").replace(',', "%2C")
}

/// `::error file=…,line=…,col=…,title=T05::message`, one line a finding.
pub fn github(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let kind = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "notice",
        };
        let mut props = format!(
            "file={},line={},col={}",
            escape_property(&d.file),
            d.line.max(1),
            d.column.max(1)
        );
        if d.end_line > 0 {
            props.push_str(&format!(
                ",endLine={},endColumn={}",
                d.end_line, d.end_column
            ));
        }
        if !d.code.is_empty() {
            props.push_str(&format!(",title={}", escape_property(d.code)));
        }
        let text = match &d.hint {
            Some(h) => format!("{}\n{h}", d.message),
            None => d.message.clone(),
        };
        out.push_str(&format!("::{kind} {props}::{}\n", escape_data(&text)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Diagnostic> {
        vec![
            Diagnostic::coded("T05", "`User` has no field `nmae`", "src/P.wf", 3, 18)
                .with_end(3, 22)
                .with_hint("Its fields are `id`, `name`"),
            Diagnostic::coded("A01", "no alt, here", "src/P.wf", 5, 5),
        ]
    }

    #[test]
    fn github_annotations_are_escaped() {
        let out = github(&sample());
        assert!(out.starts_with("::error file=src/P.wf,line=3,col=18,endLine=3,endColumn=22,title=T05::`User` has no field `nmae`%0AIts fields are `id`, `name`\n"), "{out}");
        assert!(
            out.contains("::warning file=src/P.wf,line=5,col=5,title=A01::no alt, here\n"),
            "{out}"
        );
    }

    #[test]
    fn sarif_names_each_rule_once_and_places_each_result() {
        let v: Value = serde_json::from_str(&sarif(&sample())).unwrap();
        assert_eq!(v["version"], "2.1.0");
        let run = &v["runs"][0];
        assert_eq!(run["tool"]["driver"]["rules"].as_array().unwrap().len(), 2);
        assert_eq!(run["results"][0]["ruleId"], "T05");
        assert_eq!(run["results"][0]["level"], "error");
        assert_eq!(
            run["results"][0]["locations"][0]["physicalLocation"]["region"]["endColumn"],
            22
        );
        assert_eq!(run["results"][1]["level"], "warning");
    }

    #[test]
    fn json_carries_the_struct_and_the_counts() {
        let v: Value = serde_json::from_str(&json(&sample())).unwrap();
        assert_eq!(v["errors"], 1);
        assert_eq!(v["warnings"], 1);
        assert_eq!(v["diagnostics"][0]["code"], "T05");
        assert_eq!(v["diagnostics"][0]["end_column"], 22);
    }
}
