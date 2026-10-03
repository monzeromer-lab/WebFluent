//! Every check, once, over a whole project.
//!
//! The build, the language server, the docs tests and the studio used to
//! carry their own copy of this list, and the copies disagreed: the editor
//! never ran the `env`, script, output-mode or config checks the build
//! refused. [`check_project`] is the one list. Every stage runs whatever the
//! one before it found — a type error does not hide an accessibility
//! warning — and the findings come back coded, de-duplicated and in order.

use super::{Diagnostic, dedupe};
use crate::config::ProjectConfig;
use crate::config::project::OutputType;
use crate::parser::ast::{Declaration, Program};
use crate::project_js::Script;
use std::path::Path;

/// What the checks read.
pub struct Project<'a> {
    /// The program as parsed, before lowering: every file that parsed.
    pub program: &'a Program,
    /// The label each declaration's findings carry — its file's path for
    /// the build, whatever routes it back for an editor.
    pub file_of: &'a dyn Fn(usize) -> String,
    /// The text of each declaration's file, which places a finding at the
    /// expression it names rather than its statement.
    pub source_of: &'a dyn Fn(usize) -> Option<String>,
    /// The project directory, for the checks that read files: `env`, the
    /// scripts' addresses, the config's unknown keys.
    pub dir: Option<&'a Path>,
    pub config: Option<&'a ProjectConfig>,
    /// The file of each declaration, as the project names it.
    pub declaration_files: &'a [String],
    /// The project's own scripts.
    pub scripts: &'a [Script],
    /// The project's own stylesheets, bundled: a flag one of them styles
    /// is a real one.
    pub stylesheets: &'a str,
    /// Whether some file did not parse. Findings that a declaration it
    /// would have held could answer — a name or a component nothing
    /// declares — are left out, so one broken file does not bury the
    /// project in consequences of itself.
    pub incomplete: bool,
}

/// What came back.
pub struct Checked {
    /// Every finding, coded, de-duplicated, in order.
    pub diagnostics: Vec<Diagnostic>,
    /// The program lowered onto the vocabulary the code generators read.
    pub lowered: Program,
}

/// The codes a missing declaration could explain.
const REFERENCES: &[&str] = &["E101", "E104", "T13", "T06", "R01", "U03", "U04", "U05"];

/// Every check over a project.
pub fn check_project(p: &Project) -> Checked {
    let mut out = Vec::new();
    if let Some(dir) = p.dir {
        out.extend(script_checks(dir, p.scripts));
        if let Some(config) = p.config {
            out.extend(config_checks(dir, config, p.program));
            out.extend(crate::linter::lint_env(dir, config, p.declaration_files));
        }
    }
    out.extend(program_checks(p.program, p.file_of, p.source_of));

    let lowered = crate::sema::lower(p.program.clone());
    let mut lints: Vec<crate::error::A11yWarning> =
        crate::linter::lint_accessibility_in(&lowered, p.file_of);
    let theme = p.config.map(|c| c.theme.clone()).unwrap_or_default();
    if let Ok(tokens) = crate::themes::resolve_tokens(&lowered, &theme) {
        lints.extend(crate::linter::lint_contrast_in(
            &lowered, &tokens, p.file_of,
        ));
    }
    lints.extend(crate::linter::lint_unused_in(&lowered, p.file_of));
    // A component the build publishes as a custom element is placed by
    // whoever loads it: nothing in the project places it, and that is the
    // point.
    if let Some(config) = p.config
        && !config.build.elements.is_empty()
    {
        lints.retain(|w| {
            w.rule_id != "U03"
                || !config
                    .build
                    .elements
                    .iter()
                    .any(|name| w.message.contains(&format!("`{name}`")))
        });
    }
    out.extend(lints.into_iter().map(Diagnostic::from));
    out.extend(
        crate::linter::lint_vocabulary_with(&lowered, p.stylesheets, p.file_of)
            .into_iter()
            .map(Diagnostic::from),
    );
    if let Some(config) = p.config {
        out.extend(output_checks(&lowered, config, p.file_of));
    }

    if p.incomplete {
        out.retain(|d| !REFERENCES.contains(&d.code));
    }
    dedupe(&mut out);
    Checked {
        diagnostics: out,
        lowered,
    }
}

/// What a program says about itself, with nothing around it: the
/// structure, the registry's vocabulary and the types. A template rendered
/// with data is held to this.
pub fn program_checks(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    source_of: &dyn Fn(usize) -> Option<String>,
) -> Vec<Diagnostic> {
    let mut out = crate::linter::validate_semantics_in(program, file_of);
    let findings = crate::sema::check(program, file_of);
    out.extend(findings.errors);
    out.extend(findings.warnings);
    let typed = crate::sema::types::check_in(program, file_of, source_of);
    out.extend(typed.findings.errors);
    out.extend(typed.findings.warnings);
    out.extend(typed.unresolved);
    out
}

/// The project's own scripts: a module cannot be linked as a plain
/// script; a file in `public/` at the same address would replace one; a
/// file the scanner could not read is linked, but its names are not in
/// scope.
pub fn script_checks(dir: &Path, scripts: &[Script]) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for script in scripts {
        if let Some((line, col)) = script.scan.module_at {
            out.push(
                Diagnostic::coded(
                    "E110",
                    format!(
                        "`{}` is an ES module, and a script under `src/` is a plain browser script",
                        script.path
                    ),
                    &script.path,
                    line,
                    col,
                )
                .with_hint(
                    "Take off its `import` and `export`: a top-level `function` is global as it is",
                ),
            );
        }
        if dir.join("public").join(&script.href).exists() {
            out.push(
                Diagnostic::coded(
                    "E110",
                    format!(
                        "`{}` and `public/{}` both claim `/{}`",
                        script.path, script.href, script.href
                    ),
                    &script.path,
                    1,
                    1,
                )
                .with_hint(
                    "Keep one of the two: a script under `src/` is written to `js/` in the output",
                ),
            );
        }
        for problem in &script.scan.problems {
            out.push(
                Diagnostic::coded(
                    "E115",
                    format!(
                        "{} — the names it declares are not in scope",
                        problem.message
                    ),
                    &script.path,
                    problem.line,
                    problem.col,
                )
                .with_hint("The file is still linked; the compiler only could not read it"),
            );
        }
    }
    out
}

/// A finding about the config, at the first place `needle` appears in it,
/// the whole of `needle` underlined.
fn config_finding(code: &'static str, message: String, text: &str, needle: &str) -> Diagnostic {
    let place = text.find(needle).and_then(|at| {
        Some((
            super::line_col(text, at)?,
            super::line_col(text, at + needle.len())?,
        ))
    });
    match place {
        Some(((line, col), (end_line, end_col))) => {
            Diagnostic::coded(code, message, CONFIG, line, col).with_end(end_line, end_col)
        }
        None => Diagnostic::coded(code, message, CONFIG, 1, 1),
    }
}

const CONFIG: &str = "webfluent.app.json";

/// What the configuration asks for that cannot work, or that nothing reads.
pub fn config_checks(dir: &Path, config: &ProjectConfig, program: &Program) -> Vec<Diagnostic> {
    let text = std::fs::read_to_string(dir.join(CONFIG)).unwrap_or_default();
    let mut out = Vec::new();
    for unknown in ProjectConfig::unknown_keys(dir) {
        let key = unknown
            .split('`')
            .nth(1)
            .and_then(|path| path.rsplit('.').next())
            .unwrap_or("");
        let quoted = format!("\"{key}\"");
        let mut d = config_finding("E112", unknown, &text, &quoted);
        // The key, inside its quotes.
        if d.end_column > d.column + 1 {
            d.column += 1;
            d.end_column -= 1;
        }
        out.push(d);
    }

    // A stylesheet, font or script from another origin is code that origin
    // can change after it was read. A hash is how the browser checks it
    // has not; the build cannot work one out without fetching the file.
    for url in config
        .meta
        .fonts
        .iter()
        .chain(config.meta.stylesheets.iter())
        .map(String::as_str)
        .chain(config.meta.scripts.iter().map(|s| s.src()))
        .collect::<std::collections::BTreeSet<_>>()
    {
        // A Google Fonts stylesheet is written per user-agent, so its bytes
        // differ between readers and a hash cannot match.
        if !url.contains("://")
            || config.meta.integrity.contains_key(url)
            || url.contains("fonts.googleapis.com")
        {
            continue;
        }
        out.push(
            config_finding(
                "D05",
                format!("`{url}` is loaded from another origin with no integrity hash"),
                &text,
                url,
            )
            .with_hint(
                "Add one to `meta.integrity`, so a change at that origin cannot reach your readers",
            ),
        );
    }

    // A module is imported and put on a global; without one to put it on,
    // nothing could reach it.
    for entry in &config.meta.scripts {
        if let crate::config::project::ScriptEntry::Spec(spec) = entry
            && spec.module
            && spec.as_name.is_none()
        {
            out.push(
                config_finding(
                    "E111",
                    format!(
                        "`meta.scripts` imports `{}` as a module but names no global for it",
                        spec.src
                    ),
                    &text,
                    &spec.src,
                )
                .with_hint("Add `\"as\": \"Name\"`: its exports are put on `window.Name`"),
            );
        }
    }

    if config.build.output_type == OutputType::Elements {
        for problem in crate::codegen::elements::check(program, config) {
            out.push(config_finding("E111", problem, &text, "\"elements\""));
        }
    }

    if let Some(offline) = &config.offline {
        let mut problem = |message: String| {
            out.push(config_finding("E111", message, &text, "\"offline\""));
        };
        if !matches!(config.build.output_type, OutputType::Spa) {
            problem("`offline` is for a web build; this one writes a document".to_string());
        }
        for (path, strategy) in &offline.cache {
            if !crate::config::OFFLINE_STRATEGIES.contains(&strategy.as_str()) {
                problem(format!(
                    "`offline.cache` asks for `{strategy}` on `{path}`; the policies are {}",
                    crate::config::OFFLINE_STRATEGIES
                        .iter()
                        .map(|s| format!("`{s}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
        if let Some(fallback) = &offline.fallback {
            let known = program
                .declarations
                .iter()
                .any(|d| matches!(d, Declaration::Page(p) if &p.path == fallback));
            if !known {
                problem(format!(
                    "`offline.fallback` is `{fallback}`, which no page's `path` is; \
                     declare `page Offline(path: \"{fallback}\")` or name a page that exists"
                ));
            }
        }
    }
    out
}

/// What the output this build writes cannot draw: a PDF or a deck has no
/// buttons, handlers or routes.
pub fn output_checks(
    lowered: &Program,
    config: &ProjectConfig,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    match config.build.output_type {
        OutputType::Pdf => {
            for e in crate::linter::validate_for_pdf(lowered) {
                out.push(Diagnostic::coded(
                    "E109",
                    format!(
                        "`{}` cannot be drawn in a PDF ({}): {}",
                        e.component, e.context, e.reason
                    ),
                    file_of(e.decl),
                    e.span.line.max(1) as usize,
                    e.span.col.max(1) as usize,
                ));
            }
        }
        OutputType::Slides => {
            for e in crate::linter::validate_for_slides(lowered) {
                out.push(Diagnostic::coded(
                    "E109",
                    format!("`{}` in {}: {}", e.component, e.context, e.reason),
                    file_of(e.decl),
                    e.span.line.max(1) as usize,
                    e.span.col.max(1) as usize,
                ));
            }
        }
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<Diagnostic> {
        let program = crate::syntax::parse_source(src, "t.wf").unwrap();
        let files = vec!["t.wf".to_string(); program.declarations.len()];
        let source = src.to_string();
        check_project(&Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| Some(source.clone()),
            dir: None,
            config: None,
            declaration_files: &files,
            scripts: &[],
            stylesheets: "",
            incomplete: false,
        })
        .diagnostics
    }

    #[test]
    fn every_stage_runs_whatever_the_one_before_found() {
        // A type error and an unknown component used to stop the build
        // before the accessibility lint ran; now all three are reported.
        let found = check(
            "page P(path: \"/\", title: \"T\", description: \"D\") {\n    state n: Number = \"x\"\n    Heading(\"T\").h1\n    Buton(\"x\")\n    Image(src: \"/a.png\")\n    Text(\"{n}\")\n}\n",
        );
        let codes: Vec<&str> = found.iter().map(|d| d.code).collect();
        assert!(codes.contains(&"T01"), "{codes:?}");
        assert!(codes.contains(&"E101"), "{codes:?}");
        assert!(codes.contains(&"A01"), "{codes:?}");
        assert!(
            found.iter().all(|d| !d.code.is_empty()),
            "every finding is coded"
        );
    }

    #[test]
    fn an_incomplete_project_leaves_out_what_a_missing_file_could_answer() {
        let program = crate::syntax::parse_source(
            "page P(path: \"/\") { Heading(\"T\").h1  Card2(\"x\") }",
            "t.wf",
        )
        .unwrap();
        let found = check_project(&Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| None,
            dir: None,
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete: true,
        })
        .diagnostics;
        assert!(!found.iter().any(|d| d.code == "E101"), "{found:?}");
    }
}
