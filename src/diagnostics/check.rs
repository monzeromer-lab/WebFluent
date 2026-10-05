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
        out.extend(crate::linter::project::missing_assets(
            p.program, dir, p.file_of,
        ));
        out.extend(crate::linter::project::persist_changes(
            p.program, dir, p.file_of,
        ));
        if let Some(config) = p.config {
            out.extend(config_checks(dir, config, p.program));
            out.extend(crate::linter::lint_env(dir, config, p.declaration_files));
            if let Some(i18n) = &config.i18n
                && let Ok(messages) = crate::i18n::load(dir, i18n)
            {
                out.extend(crate::linter::project::translations(
                    p.program, i18n, &messages, p.file_of,
                ));
            }
        }
    }
    out.extend(program_checks(p.program, p.file_of, p.source_of));
    out.extend(crate::linter::structure::lint_structure(
        p.program, p.file_of,
    ));

    let lowered = crate::sema::lower(p.program.clone());
    let mut lints: Vec<crate::error::A11yWarning> =
        crate::linter::lint_accessibility_in(&lowered, p.file_of);
    let theme = p.config.map(|c| c.theme.clone()).unwrap_or_default();
    if let Ok(mut tokens) = crate::themes::resolve_tokens(&lowered, &theme) {
        lints.extend(crate::linter::lint_contrast_in(
            &lowered, &tokens, p.file_of,
        ));
        if let Some(config) = p.config {
            crate::themes::apply_motion(&mut tokens, &config.motion);
        }
        out.extend(crate::linter::styles::lint_styles(
            &lowered,
            Some(&tokens),
            p.file_of,
        ));
    } else {
        out.extend(crate::linter::styles::lint_styles(
            &lowered, None, p.file_of,
        ));
    }
    let published = p
        .config
        .map(|c| c.build.elements.clone())
        .unwrap_or_default();
    lints.extend(crate::linter::lint_unused_in(
        &lowered, p.file_of, &published,
    ));
    out.extend(lints.into_iter().map(Diagnostic::from));
    out.extend(
        crate::linter::lint_vocabulary_with(&lowered, p.stylesheets, p.file_of)
            .into_iter()
            .map(Diagnostic::from),
    );
    if let Some(config) = p.config {
        out.extend(output_checks(&lowered, config, p.file_of));
        if config.build.csp {
            out.extend(crate::codegen::csp::head_violations(
                p.program,
                &crate::config::project::csp_meta_policy(config),
                p.file_of,
            ));
        }
    }

    // The text of each file, by the name its findings carry: for the
    // `wf-allow` comments, and the fixes known by their shape.
    let mut sources: Vec<(String, Option<String>)> = Vec::new();
    for i in 0..p.program.declarations.len() {
        let file = (p.file_of)(i);
        if !sources.iter().any(|(f, _)| *f == file) {
            sources.push((file, (p.source_of)(i)));
        }
    }
    let text_of = |file: &str| {
        sources
            .iter()
            .find(|(f, _)| f == file)
            .and_then(|(_, s)| s.clone())
    };
    let files: Vec<String> = sources.iter().map(|(f, _)| f.clone()).collect();
    super::allow::apply(&mut out, &files, &text_of);
    for d in out.iter_mut().filter(|d| !d.plans.is_empty()) {
        match text_of(&d.file) {
            Some(source) => super::fixes::realise(d, &source),
            None => d.plans.clear(),
        }
    }

    if p.incomplete {
        // What a file that did not parse could answer — and an allow the
        // finding it covers may be missing from — is left out.
        out.retain(|d| !REFERENCES.contains(&d.code) && d.code != "U07");
    }
    if let Some(config) = p.config {
        let text = p
            .dir
            .and_then(|dir| std::fs::read_to_string(dir.join(CONFIG)).ok())
            .unwrap_or_default();
        apply_lints(&mut out, &config.lints, &text);
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

/// [`config_finding`] at the first `needle` after `anchor` — a key inside
/// one object, rather than one of the same name elsewhere in the file.
fn config_finding_after(
    code: &'static str,
    message: String,
    text: &str,
    anchor: &str,
    needle: &str,
) -> Diagnostic {
    let start = text.find(anchor).unwrap_or(0);
    let Some(at) = text[start..].find(needle).map(|i| start + i) else {
        return config_finding(code, message, text, anchor);
    };
    match (
        super::line_col(text, at),
        super::line_col(text, at + needle.len()),
    ) {
        (Some((line, col)), Some((end_line, end_col))) => {
            Diagnostic::coded(code, message, CONFIG, line, col).with_end(end_line, end_col)
        }
        _ => config_finding(code, message, text, anchor),
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

    // A preload is a file on this site, of a kind a preload can name. One
    // from another origin would be refused by the policy the build ships;
    // that origin's font or script is declared in `fonts` or `scripts`.
    for path in &config.meta.preload {
        let elsewhere = path.contains("://") || path.starts_with("//");
        let problem = if elsewhere {
            Some((
                format!("`meta.preload` names `{path}`, on another origin"),
                "Serve the file from this site (`public/`), or declare a font stylesheet in `meta.fonts` and a script in `meta.scripts`",
            ))
        } else if crate::codegen::html::preload_kind(path).is_none() {
            Some((
                format!(
                    "`meta.preload` names `{path}`, which is no kind of file a preload can name"
                ),
                "A preload is a font (`woff2`, `woff`, `ttf`, `otf`), a stylesheet, a script or a picture",
            ))
        } else {
            None
        };
        if let Some((message, hint)) = problem {
            out.push(config_finding("E111", message, &text, path).with_hint(hint));
        }
    }

    // A `connect-src` source is an origin, not a path or a keyword: a
    // path is no narrower in a policy than its host, and `'unsafe-…'` has no
    // place in a list of where requests may go.
    for entry in &config.meta.connect {
        if crate::config::project::connect_origin(entry).is_none() {
            out.push(
                config_finding(
                    "E111",
                    format!("`meta.connect` names `{entry}`, which is not an origin"),
                    &text,
                    entry,
                )
                .with_hint("Write a scheme and a host — `https://api.example.com`, or `https://*.example.com` for its subdomains"),
            );
        }
    }

    // The owner node's identity, and what other settings write into it,
    // come from those settings; `owner_details` only adds to it.
    for key in config.meta.owner_details.keys() {
        let from = match key.as_str() {
            "@type" => Some("`meta.owner`"),
            "@id" | "url" => Some("`meta.site_url`"),
            "name" => Some("`meta.site_name`"),
            "jobTitle" => Some("`meta.job_title`"),
            "sameAs" => Some("`meta.same_as`"),
            "@context" => Some("the page's structured data"),
            _ => None,
        };
        if let Some(from) = from {
            out.push(
                config_finding_after(
                    "E111",
                    format!("`meta.owner_details` sets `{key}`, which comes from {from}"),
                    &text,
                    "\"owner_details\"",
                    &format!("\"{key}\""),
                )
                .with_hint(format!("Remove it here and set {from}")),
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
        // A page draws a chart or a QR code when it is built: what it reads
        // must be known then.
        _ => {
            let scope = crate::codegen::static_eval::Scope::from_program_with_env(
                lowered,
                &[],
                &config.env,
            );
            for (decl, span, name, arg) in unknown_graphic_args(lowered, &scope) {
                out.push(
                    Diagnostic::coded(
                        "E109",
                        format!("`{name}` on a page is drawn when the page is built, and `{arg}` reads a value known only in the browser"),
                        file_of(decl),
                        span.line.max(1) as usize,
                        span.col.max(1) as usize,
                    )
                    .with_hint("Give it a literal, a `const` or a `data` file; a PDF or a template render draws it from any value".to_string()),
                );
            }
        }
    }
    out
}

/// Every `Chart` and `QrCode` argument a build cannot work out: the
/// declaration, the element's place, its name and the argument's.
fn unknown_graphic_args(
    program: &Program,
    scope: &crate::codegen::static_eval::Scope,
) -> Vec<(usize, crate::parser::ast::Span, String, String)> {
    use crate::parser::{Arg, ComponentRef, Declaration, Statement, StatementKind};
    fn walk(
        stmts: &[Statement],
        decl: usize,
        scope: &crate::codegen::static_eval::Scope,
        out: &mut Vec<(usize, crate::parser::ast::Span, String, String)>,
    ) {
        for stmt in stmts {
            if let StatementKind::UIElement(ui) = &stmt.kind
                && let ComponentRef::BuiltIn(name) = &ui.component
                && (name == "Chart" || name == "QrCode")
            {
                for arg in &ui.args {
                    let (label, e) = match arg {
                        Arg::Named(k, e) => (k.clone(), e),
                        Arg::Positional(e) => ("value".to_string(), e),
                    };
                    if crate::codegen::static_eval::eval(e, scope).is_none() {
                        out.push((decl, ui.span, name.clone(), label));
                        break;
                    }
                }
            }
            if let StatementKind::UIElement(ui) = &stmt.kind {
                walk(&ui.children, decl, scope, out);
                for fill in &ui.slot_fills {
                    walk(&fill.body, decl, scope, out);
                }
            }
            for body in stmt.kind.bodies() {
                walk(body, decl, scope, out);
            }
        }
    }
    let mut out = Vec::new();
    for (i, d) in program.declarations.iter().enumerate() {
        let body: &[Statement] = match d {
            Declaration::Page(p) => &p.body,
            Declaration::Component(c) => &c.body,
            Declaration::App(a) => &a.body,
            _ => continue,
        };
        walk(body, i, scope, &mut out);
    }
    out
}

/// What `lints` says each finding counts as: `"off"` drops it, `"warn"`
/// makes it a warning, `"error"` an error — by its code first, then its
/// family. An error that ships a broken page cannot be lowered, and a
/// setting that asks to is itself refused; so is a key that is no code or
/// family, and a value that is not one of the three.
pub fn apply_lints(
    out: &mut Vec<Diagnostic>,
    lints: &std::collections::BTreeMap<String, String>,
    config_text: &str,
) {
    if lints.is_empty() {
        return;
    }
    let mut problems = Vec::new();
    for (key, value) in lints {
        let known = super::codes::info(key).is_some()
            || (key.len() == 1
                && super::codes::CODES
                    .iter()
                    .any(|c| c.code.starts_with(key.as_str())));
        if !known {
            problems.push(config_finding(
                "E111",
                format!("`lints` names `{key}`, which is no code or family"),
                config_text,
                &format!("\"{key}\""),
            ));
        }
        if !matches!(value.as_str(), "off" | "warn" | "error") {
            problems.push(config_finding(
                "E111",
                format!("`lints.{key}` is `{value}`; a finding counts as `off`, `warn` or `error`"),
                config_text,
                &format!("\"{key}\""),
            ));
        }
        if matches!(value.as_str(), "off" | "warn")
            && let Some(info) = super::codes::info(key)
            && !super::codes::lowerable(key)
        {
            let _ = info;
            problems.push(
                config_finding(
                    "E111",
                    format!("`lints` lowers `{key}`, an error that ships a broken page; it stays an error"),
                    config_text,
                    &format!("\"{key}\""),
                )
                .with_hint("Fix what it finds; `lints` lowers warnings, and the few errors that cannot break a page"),
            );
        }
    }
    out.retain_mut(|d| {
        let setting = lints
            .get(d.code)
            .or_else(|| lints.get(super::codes::family(d.code)));
        match setting.map(String::as_str) {
            Some("off") if super::codes::lowerable(d.code) => false,
            Some("warn") if super::codes::lowerable(d.code) => {
                d.severity = super::Severity::Warning;
                true
            }
            Some("error") => {
                d.severity = super::Severity::Error;
                true
            }
            _ => true,
        }
    });
    out.extend(problems);
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

    /// What `config_checks` says about this `webfluent.app.json`.
    fn config_findings(json: &str) -> Vec<Diagnostic> {
        let dir = std::env::temp_dir().join(format!(
            "wf-config-check-{}-{}",
            std::process::id(),
            json.len()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(CONFIG), json).unwrap();
        let config: ProjectConfig = serde_json::from_str(json).unwrap();
        let program =
            crate::syntax::parse_source("page P(path: \"/\") { Text(\"x\") }", "t.wf").unwrap();
        let out = config_checks(&dir, &config, &program);
        let _ = std::fs::remove_dir_all(&dir);
        out
    }

    #[test]
    fn a_preload_from_another_origin_or_of_no_kind_is_refused() {
        let found = config_findings(
            r#"{"name":"t","meta":{"preload":["https://cdn.example/a.woff2","/notes.txt","/fonts/a.woff2"]}}"#,
        );
        let e111: Vec<&str> = found
            .iter()
            .filter(|d| d.code == "E111")
            .map(|d| d.message.as_str())
            .collect();
        assert_eq!(e111.len(), 2, "{e111:?}");
        assert!(e111[0].contains("another origin"), "{e111:?}");
        assert!(e111[1].contains("/notes.txt"), "{e111:?}");
    }

    #[test]
    fn a_connect_entry_that_is_not_an_origin_is_refused() {
        let found = config_findings(
            r#"{"name":"t","meta":{"connect":["https://*.google-analytics.com","https://api.example.com/v1"]}}"#,
        );
        let e111: Vec<&str> = found
            .iter()
            .filter(|d| d.code == "E111")
            .map(|d| d.message.as_str())
            .collect();
        assert_eq!(
            e111,
            ["`meta.connect` names `https://api.example.com/v1`, which is not an origin"]
        );
    }

    #[test]
    fn owner_details_may_not_set_what_other_settings_do() {
        let json = "{\n  \"name\": \"t\",\n  \"meta\": {\n    \"owner_details\": {\n      \"name\": \"x\",\n      \"email\": \"mailto:a@b.c\"\n    }\n  }\n}";
        let found = config_findings(json);
        let e111: Vec<&Diagnostic> = found.iter().filter(|d| d.code == "E111").collect();
        assert_eq!(e111.len(), 1, "{found:?}");
        assert!(
            e111[0].message.contains("`meta.site_name`"),
            "{}",
            e111[0].message
        );
        assert_eq!(
            e111[0].line, 5,
            "the key inside owner_details, not the project's name"
        );
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
