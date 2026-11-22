//! `wf test`: a project's tests, rendered or played in a browser, and what
//! each came to.
//!
//! Running a project's tests: every `test "…" { … }` under `tests/` (and in
//! `src/`), rendered through the template engine over its data, held to what
//! it expects and to its snapshot; and a test that acts — or one that looks
//! at a `Portal`, whose block only a running page draws — built as a site of
//! one page and played in a browser.
//!
//! A test's body renders with the project's components, stores, types,
//! enums and constants at hand, as a page's would; a `state` in the body
//! is seeded with its initial value, and `data: { … }` supplies the rest
//! by name. The rendered fragment must contain every `expect "text"` and
//! none of the `expect not "text"`; it is then compared to
//! `tests/__snapshots__/<file>/<test>.html`, written when missing and
//! rewritten with `update`. A `t("key")` reads the project's messages in
//! its default locale, as the site does.
//!
//! `wf test` prints what this finds; the mobile editor shows it. The
//! browser a test that acts needs is theirs: Chrome for `wf test`, the
//! editor's WebView ([the `Browser` trait](crate::tools::testing::Browser)).

use crate::base::outfs::At;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::diagnostics::error::Result;
use crate::project::config::project::ThemeConfig;
use crate::syntax::parser::ast::{
    ComponentRef, Declaration, Expr, PageDecl, Program, Statement, StatementKind, Step, TestDecl,
};

/// A project's tests, read and ready to run.
#[derive(Debug)]
pub struct Tests {
    /// What every test renders with: everything but pages, the app and the
    /// tests, of the project's sources.
    pub shared: Vec<Declaration>,
    /// What a test file under `tests/` declares beside its tests — a
    /// `const`, a component — by file: it is that file's tests' alone, so
    /// two files may each declare a `const` of one name.
    pub own: HashMap<String, Vec<Declaration>>,
    /// Each test, with the file it is in (project-relative).
    pub tests: Vec<(String, TestDecl)>,
    /// `tests/__snapshots__`.
    pub snapshots: PathBuf,
    /// The project's theme settings, for a test that acts: a project that
    /// declares a dark theme declares two, and a page built without knowing
    /// which to use refuses to build.
    pub theme: ThemeConfig,
    /// The project's own scripts, linked on every page a test that acts
    /// builds as a site links them, so a test that clicks runs the code
    /// they define.
    pub scripts: Vec<crate::project::project_js::Script>,
    /// The project's translations, so `t("key")` in a test reads the same
    /// messages the site does.
    pub i18n: Option<I18n>,
    /// The project's own styles, which a test that acts is drawn with as
    /// the built page is.
    pub styles: Styles,
}

/// What a project styles its pages with beyond the theme and its `style { }`
/// blocks, for a test's page to be drawn as the built one is.
#[derive(Debug, Clone, Default)]
pub struct Styles {
    /// Every `.css` file under `src/`, bundled as the build bundles them
    /// into `styles.css`.
    pub css: String,
    /// The stylesheets `meta.stylesheets` links, by URL or site-relative
    /// path, with the `meta.integrity` of any that has one.
    pub linked: Vec<String>,
    pub integrity: HashMap<String, String>,
    /// The project's `public/` folder, when it has one: what a site-relative
    /// stylesheet, or a font a stylesheet names, is served from.
    pub public: Option<PathBuf>,
}

/// A project's `i18n` settings and every locale's messages.
pub type I18n = (
    crate::project::config::project::I18nConfig,
    HashMap<String, HashMap<String, String>>,
);

/// How one test went.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Passed,
    /// It passed, and its snapshot was written: there was none, or
    /// `update` asked for the render to be kept.
    Written,
    /// Why it failed, one reason a line; the last may be what it rendered.
    Failed(Vec<String>),
}

/// The tests of the project at `path`, or of the one test file `path` names.
///
/// # Errors
///
/// What [`read_project`](crate::project::sources::read_project) returns;
/// [`IoError`](crate::diagnostics::error::WebFluentError::IoError) when a test file cannot be read or the config cannot
/// be, [`ConfigError`](crate::diagnostics::error::WebFluentError::ConfigError) when the config is not one,
/// [`LexerError`](crate::diagnostics::error::WebFluentError::LexerError) or [`ParseError`](crate::diagnostics::error::WebFluentError::ParseError) when a test file does not
/// parse, and [`Diagnostics`](crate::diagnostics::error::WebFluentError::Diagnostics) (`E102`) when a test file declares a
/// name the project already has.
pub fn collect(path: &Path) -> Result<Tests> {
    // A file's project is the nearest folder above it with a
    // `webfluent.app.json` — `wf test tests/cart.wf` runs that file's tests
    // in the project it belongs to.
    let project_dir = if path.is_file() {
        let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        absolute
            .ancestors()
            .skip(1)
            .find(|dir| dir.join("webfluent.app.json").is_file())
            .map_or_else(
                || path.parent().unwrap_or(Path::new(".")).to_path_buf(),
                Path::to_path_buf,
            )
    } else {
        path.to_path_buf()
    };
    let (program, files) = crate::build::read_project(&project_dir)?;
    let mut declarations = program.declarations;
    let mut owners: Vec<String> = files;
    let mut own: HashMap<String, Vec<Declaration>> = HashMap::new();
    // The test files: `tests/*.wf` and `tests/*.wfx`, and the one named.
    // Every one is read — a fake one declares serves the others' tests —
    // and with a file named, only its tests run.
    let test_files: Vec<PathBuf> = {
        let dir = project_dir.join("tests");
        let mut found = Vec::new();
        if dir.is_dir() {
            for entry in std::fs::read_dir(&dir).at(&dir)? {
                let p = entry?.path();
                if p.extension().is_some_and(|e| e == "wf" || e == "wfx") {
                    found.push(p);
                }
            }
        }
        found.sort();
        if path.is_file() {
            let named = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            let listed = found
                .iter()
                .any(|f| f.canonicalize().is_ok_and(|f| f == named));
            if !listed {
                found.push(named);
            }
        }
        found
    };
    let only: Option<PathBuf> = path
        .is_file()
        .then(|| path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
    let mut running: Vec<String> = Vec::new();
    for file in &test_files {
        let source = std::fs::read_to_string(file).at(file)?;
        let relative = file
            .canonicalize()
            .ok()
            .zip(project_dir.canonicalize().ok())
            .and_then(|(f, p)| f.strip_prefix(&p).ok().map(Path::to_path_buf));
        let name = relative
            .as_deref()
            .unwrap_or_else(|| file.strip_prefix(&project_dir).unwrap_or(file))
            .to_string_lossy()
            .to_string();
        if only
            .as_ref()
            .is_none_or(|o| file.canonicalize().is_ok_and(|f| f == *o))
        {
            running.push(name.clone());
        }
        // A file whose tests do not run is read for its fakes alone: one
        // that does not parse is left to its own run to report.
        let parsed = match crate::syntax::parse_source(&source, &name) {
            Ok(parsed) => parsed,
            Err(_) if !running.contains(&name) => continue,
            Err(e) => return Err(e),
        };
        own.insert(
            name.clone(),
            parsed
                .declarations
                .iter()
                .filter(|d| !is_drawn(d))
                .cloned()
                .collect(),
        );
        owners.extend(parsed.declarations.iter().map(|_| name.clone()));
        declarations.extend(parsed.declarations);
    }
    // Every test, for what is checked of them; `tests` below is those run.
    let all_tests: Vec<(String, TestDecl)> = declarations
        .iter()
        .zip(&owners)
        .filter_map(|(d, file)| match d {
            Declaration::Test(t) => Some((file.clone(), t.clone())),
            _ => None,
        })
        .collect();
    // A seed is read as the state it seeds: a record it leaves a field of
    // out has that field's default, as one the program builds does.
    let defaults = crate::project::defaults::Defaults::of(&declarations);
    let tests: Vec<(String, TestDecl)> = all_tests
        .into_iter()
        .filter(|(file, _)| only.is_none() || running.contains(file))
        .map(|(file, mut test)| {
            if let Some(data) = &mut test.data {
                defaults.fill_seeds(data, &test.body, &declarations);
            }
            (file, test)
        })
        .collect();
    let mut shared: Vec<Declaration> = declarations
        .iter()
        .zip(&owners)
        .filter(|(d, file)| !is_drawn(d) && !own.contains_key(*file))
        .map(|(d, _)| d.clone())
        .collect();
    // A test file's own declaration may not take a name the project gives
    // something else: the two would be one name in the test's site, and
    // the page would stop before any test ran.
    let project_names: Vec<(&str, &String, crate::syntax::parser::ast::Span)> = declarations
        .iter()
        .zip(&owners)
        .filter(|(d, file)| !is_drawn(d) && !own.contains_key(*file))
        .filter_map(|(d, file)| {
            Some((
                crate::check::edit::declaration_name(d)?,
                file,
                crate::check::scope::decl_span(d).unwrap_or_default(),
            ))
        })
        .collect();
    let mut clashes = Vec::new();
    for (file, decls) in &own {
        for d in decls {
            let Some(name) = crate::check::edit::declaration_name(d) else {
                continue;
            };
            if let Some((_, first_file, at)) = project_names.iter().find(|(n, _, _)| *n == name) {
                let span = crate::check::scope::decl_span(d).unwrap_or_default();
                clashes.push(
                    crate::diagnostics::Diagnostic::coded(
                        "E102",
                        format!(
                            "`{name}` is already declared in the project, at {first_file}:{}",
                            at.line
                        ),
                        file,
                        span.line as usize,
                        span.col as usize,
                    )
                    .with_hint(format!(
                        "A test's own declaration is drawn with the project's: rename this `{name}`"
                    )),
                );
            }
        }
    }
    // A seed is held to the state it seeds: `data: { count: "two" }` for a
    // `state count = 0` is the test's mistake, said before anything runs.
    for (file, test) in &tests {
        clashes.extend(seed_faults(test, &declarations, file));
    }
    // `fake … from examples` answers with the spec's examples.
    let specs = crate::project::openapi::specs_of(
        &declarations,
        &crate::project::config::ProjectConfig::load(&project_dir)
            .map(|c| c.apis)
            .unwrap_or_default(),
    );
    let read_spec = |file: &str| {
        [project_dir.join(file), project_dir.join("src").join(file)]
            .iter()
            .find_map(|at| std::fs::read_to_string(at).ok())
    };
    crate::project::openapi::answer_from_examples(&mut declarations, &specs, &read_spec);
    for list in own.values_mut() {
        crate::project::openapi::answer_from_examples(list, &specs, &read_spec);
    }
    // A fake is held to the contract it answers for, and a test to the
    // fakes and endpoints it names (`N06`).
    clashes.extend(fake_faults(&declarations, &owners, &own, &tests));
    if !clashes.is_empty() {
        return Err(crate::diagnostics::error::WebFluentError::Diagnostics(
            clashes,
        ));
    }
    // A page a test can place by its name, as it places a component: `Tasks`
    // draws the Tasks page's body — its state, its stores, its handlers — so
    // a test can click through a page as a reader does. A page that takes a
    // route parameter, or whose name a component has, is left out.
    let components: Vec<String> = shared
        .iter()
        .chain(own.values().flatten())
        .filter_map(|d| match d {
            Declaration::Component(c) => Some(c.name.clone()),
            _ => None,
        })
        .collect();
    for d in &declarations {
        if let Declaration::Page(page) = d
            && page.params.is_empty()
            && !components.contains(&page.name)
        {
            shared.push(Declaration::Component(
                crate::syntax::parser::ast::ComponentDecl {
                    name: page.name.clone(),
                    props: Vec::new(),
                    events: Vec::new(),
                    slots: Vec::new(),
                    parts: Vec::new(),
                    doc: None,
                    body: page.body.clone(),
                    span: page.span,
                    header_span: page.header_span,
                    body_span: page.body_span,
                },
            ));
        }
    }
    let config = crate::project::config::ProjectConfig::load_if_present_via(
        &crate::base::vfs::FsVfs,
        &project_dir,
    )?;
    // The project's translations: a test draws `t("key")` as the site does,
    // in its default locale.
    let i18n = config.as_ref().and_then(|c| c.i18n.clone()).and_then(|i| {
        crate::project::i18n::load(&project_dir, &i)
            .ok()
            .map(|tables| (i, tables))
    });
    let scripts = crate::project::project_js::load(&project_dir, &project_dir.join("src"))
        .unwrap_or_default();
    // Absolute: the test's site links to it from somewhere else.
    let public = project_dir.join("public");
    let public = public.canonicalize().unwrap_or(public);
    let styles = Styles {
        css: crate::web::codegen::project_css::bundle(&project_dir, &project_dir.join("src"))?,
        linked: config
            .as_ref()
            .map(|c| c.meta.stylesheets.clone())
            .unwrap_or_default(),
        integrity: config
            .as_ref()
            .map(|c| c.meta.integrity.clone())
            .unwrap_or_default(),
        public: public.is_dir().then_some(public),
    };
    Ok(Tests {
        shared,
        own,
        tests,
        snapshots: project_dir.join("tests").join("__snapshots__"),
        theme: config.map(|c| c.theme).unwrap_or_default(),
        scripts,
        i18n,
        styles,
    })
}

/// What is wrong with the fakes and with what the tests name of them
/// (`N06`): a fake in a test file checked against the contract it answers
/// for, as the build checks one in `src/`; a test's `fakes:` naming
/// something no fake is, or two fakes answering one endpoint; an `expect
/// called` naming no endpoint.
fn fake_faults(
    declarations: &[Declaration],
    owners: &[String],
    own: &HashMap<String, Vec<Declaration>>,
    tests: &[(String, TestDecl)],
) -> Vec<crate::diagnostics::Diagnostic> {
    let mut out = Vec::new();
    // What the type checker says of the fakes the test files declare,
    // checked beside everything the project and those files declare.
    let mut program = Program {
        declarations: Vec::new(),
    };
    let mut files = Vec::new();
    // Each fake of a test file, as the lines from its own to the next
    // declaration's: what is found there is the fake's.
    let mut fakes: Vec<(&str, usize, usize)> = Vec::new();
    for (i, (d, file)) in declarations.iter().zip(owners).enumerate() {
        if is_drawn(d) {
            continue;
        }
        if let Declaration::Fake(f) = d
            && own.contains_key(file)
        {
            let next = declarations
                .iter()
                .zip(owners)
                .skip(i + 1)
                .find(|(_, o)| *o == file)
                .and_then(|(d, _)| crate::check::scope::decl_span(d))
                .map_or(usize::MAX, |s| s.line as usize);
            fakes.push((file, f.span.line as usize, next));
        }
        program.declarations.push(d.clone());
        files.push(file.clone());
    }
    if !fakes.is_empty() {
        let info = crate::check::sema::types::check(&program, &|i| {
            files.get(i).cloned().unwrap_or_default()
        });
        out.extend(info.findings.errors.into_iter().filter(|d| {
            fakes
                .iter()
                .any(|(file, from, to)| d.file == *file && d.line >= *from && d.line < *to)
        }));
    }
    // What a test names.
    // A test's own file and the project come first; a fake another test
    // file declares is the tests' too — they are one program — so a file of
    // fakes serves every test that names one.
    let fake_decls = |file: &str| -> Vec<&crate::syntax::parser::ast::FakeDecl> {
        let near = |f: &String| !own.contains_key(f) || f == file;
        let all = || {
            declarations
                .iter()
                .zip(owners)
                .filter_map(|(d, f)| match d {
                    Declaration::Fake(fake) => Some((fake, f)),
                    _ => None,
                })
        };
        let mut known: Vec<&crate::syntax::parser::ast::FakeDecl> =
            all().filter(|(_, f)| near(f)).map(|(d, _)| d).collect();
        for (d, f) in all() {
            if !near(f) && !known.iter().any(|k| k.name == d.name) {
                known.push(d);
            }
        }
        known
    };
    // A fake no nearer file declares, which two other test files each do:
    // which one is meant cannot be told.
    let elsewhere = |file: &str, name: &str| -> Vec<&String> {
        let mut files: Vec<&String> = declarations
            .iter()
            .zip(owners)
            .filter(|(d, f)| {
                own.contains_key(*f)
                    && *f != file
                    && matches!(d, Declaration::Fake(fake) if fake.name == name)
            })
            .map(|(_, f)| f)
            .collect();
        files.dedup();
        files
    };
    let apis: Vec<&crate::syntax::parser::ast::ApiDecl> = declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Api(a) => Some(a),
            _ => None,
        })
        .collect();
    for (file, test) in tests {
        let at = |code: &'static str, message: String, hint: &str| {
            crate::diagnostics::Diagnostic::coded(
                code,
                message,
                file,
                test.span.line as usize,
                test.span.col as usize,
            )
            .with_hint(hint)
        };
        let known = fake_decls(file);
        let mut answered: Vec<(String, &str)> = Vec::new();
        for name in &test.fakes {
            let Some(fake) = known.iter().find(|f| f.name == *name) else {
                let names: Vec<&str> = known.iter().map(|f| f.name.as_str()).collect();
                let hint = match crate::diagnostics::fixes::nearest(name, names.iter().copied()) {
                    Some(n) => format!("Did you mean `{n}`?"),
                    None if names.is_empty() => {
                        "Declare one beside the test: `fake OneUser for Backend.users { … }`"
                            .to_string()
                    }
                    None => format!(
                        "The fakes are {}",
                        names
                            .iter()
                            .map(|n| format!("`{n}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                };
                out.push(at(
                    "N06",
                    format!(
                        "test \"{}\" names the fake `{name}`, and nothing declares it",
                        test.name
                    ),
                    &hint,
                ));
                continue;
            };
            let near = declarations.iter().zip(owners).any(|(d, f)| {
                (!own.contains_key(f) || f == file)
                    && matches!(d, Declaration::Fake(x) if x.name == *name)
            });
            let others = elsewhere(file, name);
            if !near && others.len() > 1 {
                out.push(at(
                    "N06",
                    format!(
                        "test \"{}\" names the fake `{name}`, which {} each declare",
                        test.name,
                        others
                            .iter()
                            .map(|f| format!("`{f}`"))
                            .collect::<Vec<_>>()
                            .join(" and ")
                    ),
                    "Keep one fake of a name among the test files, or declare it beside the test",
                ));
                continue;
            }
            for a in &fake.answers {
                let endpoint = format!("{}.{}", fake.target, a.endpoint);
                if let Some((_, first)) = answered.iter().find(|(e, _)| *e == endpoint) {
                    out.push(at(
                        "N06",
                        format!(
                            "test \"{}\": `{first}` and `{name}` both answer `{endpoint}`",
                            test.name
                        ),
                        "Keep one fake an endpoint in a test",
                    ));
                } else {
                    answered.push((endpoint, name));
                }
            }
        }
        for step in &test.steps {
            let Step::ExpectCalled { endpoint, .. } = step else {
                continue;
            };
            let found = endpoint.rsplit_once('.').is_some_and(|(api, ep)| {
                apis.iter()
                    .any(|a| a.name == api && a.endpoints.iter().any(|e| e.name == ep))
            });
            if !found {
                out.push(at(
                    "N06",
                    format!(
                        "test \"{}\" expects a call of `{endpoint}`, which no `api` declares",
                        test.name
                    ),
                    "Name the endpoint as a call names it: `expect called Backend.users.create`",
                ));
            }
        }
    }
    out
}

/// What a test's `data:` gives that the state it seeds cannot hold (`T33`),
/// and a store's member it names that the store does not have (`T05`). A
/// key that names neither a state of the body nor a store is data the body
/// reads by name, as a template's is, and is not checked here.
fn seed_faults(
    test: &TestDecl,
    declarations: &[Declaration],
    file: &str,
) -> Vec<crate::diagnostics::Diagnostic> {
    use crate::check::sema::types::shape::{Shape, Shapes, fault};
    let Some(Expr::MapLiteral(pairs)) = &test.data else {
        return Vec::new();
    };
    let shapes = Shapes::of(declarations);
    let scope = crate::project::static_eval::Scope::of([]);
    let value_of = |e: &Expr| crate::project::static_eval::eval(e, &scope).map(|v| v.to_json());
    // What a state holds: its declared type, else what its first value is
    // when that is a plain number, string or boolean.
    let shape_of = |state: &crate::syntax::parser::ast::StateDecl| match &state.ty {
        Some(ty) => Some(shapes.shape(ty)),
        None => match value_of(&state.value)? {
            serde_json::Value::Number(_) => Some(Shape::Number),
            serde_json::Value::String(_) => Some(Shape::String),
            serde_json::Value::Bool(_) => Some(Shape::Bool),
            _ => None,
        },
    };
    let states_of = |body: &[Statement]| -> Vec<crate::syntax::parser::ast::StateDecl> {
        body.iter()
            .filter_map(|s| match &s.kind {
                StatementKind::State(st) => Some(st.clone()),
                _ => None,
            })
            .collect()
    };
    let mut out = Vec::new();
    let mut report = |code: &'static str, message: String, hint: &str| {
        out.push(
            crate::diagnostics::Diagnostic::coded(
                code,
                message,
                file,
                test.span.line as usize,
                test.span.col as usize,
            )
            .with_hint(hint),
        );
    };
    // Why `given` does not fit `state`, seeded as `name`.
    let held = |name: &str, state: &crate::syntax::parser::ast::StateDecl, given: &Expr| {
        let f = fault(&value_of(given)?, &shape_of(state)?)?;
        Some(format!(
            "test \"{}\": `data:` seeds `{name}` with what it cannot hold: {name}{} {}",
            test.name,
            &f.at[1..],
            f.message
        ))
    };
    let seed_hint = "Give the seed a value of the state's type";
    let body_states = states_of(&test.body);
    for (key, given) in pairs {
        let key = key.trim_matches('"');
        if let Some(state) = body_states.iter().find(|s| s.name == key) {
            if let Some(message) = held(key, state, given) {
                report("T33", message, seed_hint);
            }
            continue;
        }
        let Some(store) = declarations.iter().find_map(|d| match d {
            Declaration::Store(st) if st.name == key => Some(st),
            _ => None,
        }) else {
            continue;
        };
        let Expr::MapLiteral(members) = given else {
            continue;
        };
        let states = states_of(&store.body);
        for (member, value) in members {
            let member = member.trim_matches('"');
            if let Some(state) = states.iter().find(|s| s.name == member) {
                if let Some(message) = held(&format!("{key}.{member}"), state, value) {
                    report("T33", message, seed_hint);
                }
            } else {
                let names: Vec<String> = states.iter().map(|s| format!("`{}`", s.name)).collect();
                report(
                    "T05",
                    format!(
                        "test \"{}\": `data:` seeds `{key}.{member}`, but `{key}` has no state `{member}`",
                        test.name
                    ),
                    &format!("Its states are {}", names.join(", ")),
                );
            }
        }
    }
    out
}

/// A page, the app or a test: what a test's render is not drawn with.
fn is_drawn(d: &Declaration) -> bool {
    matches!(
        d,
        Declaration::Page(_) | Declaration::App(_) | Declaration::Test(_) | Declaration::Example(_)
    )
}

impl Tests {
    /// What `test` renders with: the project's declarations, and its own
    /// file's.
    pub fn declarations_for(&self, test: &TestDecl) -> Vec<Declaration> {
        let file = self
            .tests
            .iter()
            .find(|(_, t)| std::ptr::eq(t, test))
            .or_else(|| {
                self.tests
                    .iter()
                    .find(|(_, t)| t.name == test.name && t.span == test.span)
            })
            .map(|(file, _)| file.as_str());
        self.declarations_in(file, &test.fakes)
    }

    /// The project's declarations and `file`'s own, and — for a fake of
    /// `fakes` neither has — the test file that declares it, with what it
    /// declares beside it that the test does not already have.
    fn declarations_in(&self, file: Option<&str>, fakes: &[String]) -> Vec<Declaration> {
        let mut all = self.shared.clone();
        if let Some(own) = file.and_then(|f| self.own.get(f)) {
            all.extend(own.iter().cloned());
        }
        let mut others: Vec<(&String, &Vec<Declaration>)> = self
            .own
            .iter()
            .filter(|(f, _)| Some(f.as_str()) != file)
            .collect();
        others.sort_by_key(|(f, _)| *f);
        for name in fakes {
            let has = |all: &[Declaration]| {
                all.iter()
                    .any(|d| matches!(d, Declaration::Fake(f) if f.name == *name))
            };
            if has(&all) {
                continue;
            }
            let Some((_, decls)) = others.iter().find(|(_, decls)| has(decls)) else {
                continue;
            };
            for d in *decls {
                let taken = match crate::check::edit::declaration_name(d) {
                    Some(n) => all
                        .iter()
                        .any(|a| crate::check::edit::declaration_name(a) == Some(n)),
                    None => false,
                };
                let wanted = !matches!(d, Declaration::Fake(f) if !fakes.contains(&f.name));
                if !taken && wanted {
                    all.push(d.clone());
                }
            }
        }
        all
    }

    /// How many of the tests are run in a browser ([`Tests::in_browser`]).
    pub fn acting(&self) -> usize {
        self.tests
            .iter()
            .filter(|(_, t)| self.in_browser(t))
            .count()
    }

    /// Whether `test` is run in a browser rather than rendered: it acts —
    /// a click is not something a renderer can do — or what it draws holds
    /// a `Portal`, whose block only a running page draws.
    pub fn in_browser(&self, test: &TestDecl) -> bool {
        runs_in_browser(test, &self.declarations_for(test))
    }

    /// Run a test that only looks: render it, hold it to its expects and its
    /// snapshot (written when missing, or with `update`).
    pub fn run_rendered(&self, file: &str, test: &TestDecl, update: bool) -> Outcome {
        let messages = self.messages();
        let locale = self.locale();
        let literals = self.i18n.as_ref().is_some_and(|(i, _)| i.literals);
        match run_one(
            &self.declarations_in(Some(file), &test.fakes),
            test,
            &self.snapshots,
            file,
            update,
            &messages,
            &locale,
            literals,
        ) {
            Ok(written) if written => Outcome::Written,
            Ok(_) => Outcome::Passed,
            Err(reasons) => Outcome::Failed(reasons),
        }
    }

    /// What a page of the project is drawn with beside its program.
    pub fn site(&self) -> Site<'_> {
        Site {
            theme: &self.theme,
            scripts: &self.scripts,
            i18n: self.i18n.as_ref(),
            styles: &self.styles,
        }
    }

    /// Run a test that acts in `browser`, its site built under `work`.
    pub fn run_acting(&self, browser: &mut dyn Browser, test: &TestDecl, work: &Path) -> Outcome {
        let site = self.site();
        match act(browser, &self.declarations_for(test), test, work, &site) {
            Ok(Ok(())) => Outcome::Passed,
            Ok(Err(reasons)) => Outcome::Failed(reasons),
            Err(e) => Outcome::Failed(vec![e.to_string()]),
        }
    }

    /// The project's default locale, whose plural rules a `t()` with a
    /// `count` follows; English without translations.
    pub fn locale(&self) -> String {
        self.i18n
            .as_ref()
            .map_or_else(|| "en".to_string(), |(i, _)| i.default_locale.clone())
    }

    /// The messages of the default locale, which a test that looks reads.
    pub fn messages(&self) -> HashMap<String, String> {
        self.i18n
            .as_ref()
            .and_then(|(i, tables)| tables.get(&i.default_locale).cloned())
            .unwrap_or_default()
    }
}

/// Whether `test` is run in a browser, with `declarations` the components
/// it may place: it acts, or what it draws holds a `Portal`.
pub fn runs_in_browser(test: &TestDecl, declarations: &[Declaration]) -> bool {
    test.acts() || draws_portal(&test.body, declarations, &mut Vec::new())
}

/// Whether `body`, or a component it places, draws a `Portal`. A render
/// paints only a portal's place: its block is drawn into a node the running
/// page has, so a test that looks at one is run in a browser.
fn draws_portal<'a>(
    body: &'a [Statement],
    shared: &'a [Declaration],
    seen: &mut Vec<&'a str>,
) -> bool {
    body.iter().any(|stmt| match &stmt.kind {
        StatementKind::UIElement(el) => {
            let own = match &el.component {
                ComponentRef::BuiltIn(name) => name == "Portal",
                ComponentRef::UserDefined(name) if !seen.contains(&name.as_str()) => {
                    seen.push(name);
                    shared.iter().any(|d| {
                        matches!(d, Declaration::Component(c)
                            if c.name == *name && draws_portal(&c.body, shared, seen))
                    })
                }
                _ => false,
            };
            own || draws_portal(&el.children, shared, seen)
                || el
                    .slot_fills
                    .iter()
                    .any(|f| draws_portal(&f.body, shared, seen))
        }
        StatementKind::If(i) => {
            draws_portal(&i.then_body, shared, seen)
                || i.else_if_branches
                    .iter()
                    .any(|(_, b)| draws_portal(b, shared, seen))
                || i.else_body
                    .as_deref()
                    .is_some_and(|b| draws_portal(b, shared, seen))
        }
        StatementKind::For(f) => draws_portal(&f.body, shared, seen),
        StatementKind::Show(s) => draws_portal(&s.body, shared, seen),
        StatementKind::Match(m) => m.arms.iter().any(|a| draws_portal(&a.body, shared, seen)),
        _ => false,
    })
}

/// The rendered fragment of one test: its body as a page over the shared
/// declarations, seeded with its `state` and its `data`.
///
/// # Errors
///
/// What [`render_program_fragment_in`](crate::template::render_program_fragment_in)
/// returns.
pub fn render_test(
    shared: &[Declaration],
    test: &TestDecl,
    messages: &HashMap<String, String>,
    locale: &str,
    literals: bool,
) -> Result<String> {
    let mut declarations = shared.to_vec();
    declarations.push(Declaration::Page(PageDecl {
        name: "__Test".to_string(),
        path: "/__test".to_string(),
        title: None,
        title_expr: None,
        description_expr: None,
        image_expr: None,
        guard: None,
        redirect: None,
        description: None,
        image: None,
        image_alt: None,
        page_type: None,
        noindex: false,
        lang: None,
        translates: None,
        layout: None,
        params: Vec::new(),
        head: Vec::new(),
        paths: None,
        body: test.body.clone(),
        span: test.span,
        header_span: test.span,
        body_span: test.span,
    }));
    // The project's own component wins over a built-in of its name here as
    // in a build: a starter's `ThemeToggle` is its own, not the built-in.
    let mut program = Program { declarations };
    crate::syntax::prefer_own_components(&mut program);
    let mut program = crate::check::sema::lower(program);
    // Translated by its own words, as the build translates the page: the
    // default locale's table, its plural forms too, are what a test sees.
    if literals {
        crate::project::i18n::literals::translate(&mut program);
    }
    // The data: the test's own, and the body's initial state on top.
    let mut data = match &test.data {
        Some(expr) => {
            crate::project::static_eval::eval(expr, &crate::project::static_eval::Scope::of([]))
                .map_or(serde_json::Value::Object(serde_json::Map::default()), |v| {
                    v.to_json()
                })
        }
        None => serde_json::Value::Object(serde_json::Map::default()),
    };
    // Every store the test can read, as a value: its initial state with
    // whatever the test's `data` named on top, and its derived values
    // computed from the result. The template engine reads data, not
    // declarations, so without this a `Cart.count` in a test rendered as
    // nothing.
    if let serde_json::Value::Object(map) = &mut data {
        let seeds = crate::project::static_eval::stores_as_values(&program, Some(map));
        for (name, value) in seeds {
            map.insert(name, value);
        }
    }
    let scope = crate::project::static_eval::Scope::from_program(&program, &test.body);
    if let serde_json::Value::Object(map) = &mut data {
        for stmt in &test.body {
            if let StatementKind::State(s) = &stmt.kind
                && let Some(v) = crate::project::static_eval::eval(&s.value, &scope)
            {
                map.entry(s.name.clone()).or_insert(v.to_json());
            }
        }
    }
    crate::template::render_program_fragment_in(&program, &data, messages.clone(), locale)
}

/// The text a render shows, as a browser lays it out and a reader reads
/// it: tags gone, the common entities decoded. An element laid out as a
/// block — a block by its tag, or any element a layout box (`Row`, `Stack`,
/// `Grid`) lays out, as a browser blockifies a flex or grid item — has a
/// space at each edge; an inline one has none, so `<code>live</code><code>:
/// </code>` reads `live:`.
///
/// The render is written for people to read: each element on a line of its
/// own, indented. That formatting is not text — the line break and the
/// indentation the renderer puts before a tag, after one or around an
/// element's own words are dropped — so two inline elements that touch read
/// as touching, as they do on the live page.
fn rendered_text(html: &str) -> String {
    const INLINE: &[&str] = &[
        "a", "abbr", "b", "bdi", "bdo", "code", "em", "i", "kbd", "mark", "q", "s", "small",
        "span", "strong", "sub", "sup", "time", "u",
    ];
    // Elements with no end tag, which open nothing.
    const VOID: &[&str] = &[
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
        "track", "wbr",
    ];
    // The engine's own layout boxes: their children are flex or grid items.
    const LAYOUT: &[&str] = &["wf-row", "wf-stack", "wf-grid"];
    // Each open element: its name, whether it is laid out as a block, and
    // whether it lays its children out.
    let mut open: Vec<(String, bool, bool)> = Vec::new();
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    loop {
        let at = rest.find('<').unwrap_or(rest.len());
        out.push_str(unformatted(&rest[..at]));
        if at == rest.len() {
            break;
        }
        rest = &rest[at..];
        // A comment says nothing.
        if let Some(after) = rest.strip_prefix("<!--") {
            rest = after.find("-->").map_or("", |end| &after[end + 3..]);
            continue;
        }
        let Some(close) = rest.find('>') else {
            break;
        };
        let tag = &rest[1..close];
        rest = &rest[close + 1..];
        let closing = tag.starts_with('/');
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_ascii_lowercase();
        if name.is_empty() {
            continue; // `<!doctype …>`, or a `<` that opens no tag
        }
        if closing {
            if let Some(at) = open.iter().rposition(|(n, ..)| *n == name) {
                if open[at].1 {
                    out.push(' ');
                }
                open.truncate(at);
            }
            continue;
        }
        let laid_out = open.last().is_some_and(|(.., lays_out)| *lays_out);
        let block = laid_out || !INLINE.contains(&name.as_str());
        if block {
            out.push(' ');
        }
        if !VOID.contains(&name.as_str()) && !tag.ends_with('/') {
            let lays_out = class_of(tag)
                .is_some_and(|class| class.split_whitespace().any(|c| LAYOUT.contains(&c)));
            open.push((name, block, lays_out));
        }
    }
    decode_entities(&out)
}

/// A run of text between two tags without the renderer's formatting: the
/// line break and indentation it starts with, and the one it ends with.
/// A run that is only formatting is nothing; a reader's own spaces stay.
fn unformatted(run: &str) -> &str {
    let mut run = run;
    if let Some(after) = run.strip_prefix('\n') {
        run = after.trim_start_matches(' ');
    }
    if let Some(at) = run.rfind('\n')
        && run[at + 1..].bytes().all(|b| b == b' ')
    {
        run = &run[..at];
    }
    run
}

/// The value of a tag's `class` attribute, as the renderers write it.
fn class_of(tag: &str) -> Option<&str> {
    let at = tag.find(" class=\"")? + " class=\"".len();
    let end = tag[at..].find('"')?;
    Some(&tag[at..at + end])
}

/// The text an HTML escape stands for: the named entities the renderers
/// write, and any numeric one, decimal (`&#39;`) or hex (`&#x27;`, how an
/// apostrophe is escaped). One the reader would see as written stays so.
fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let end = tail.find(';').filter(|&e| e <= 10);
        let decoded = end.and_then(|e| {
            let name = &tail[1..e];
            let c = match name {
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "amp" => Some('&'),
                "nbsp" => Some(' '),
                _ => name
                    .strip_prefix("#x")
                    .or_else(|| name.strip_prefix("#X"))
                    .and_then(|h| u32::from_str_radix(h, 16).ok())
                    .or_else(|| name.strip_prefix('#').and_then(|d| d.parse().ok()))
                    .and_then(char::from_u32),
            }?;
            Some((c, e + 1))
        });
        if let Some((c, len)) = decoded {
            out.push(c);
            rest = &tail[len..];
        } else {
            out.push('&');
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod entity_tests {
    use super::{decode_entities, rendered_text, squeeze};

    /// What an `expect` reads in a render, white space collapsed.
    fn read(html: &str) -> String {
        squeeze(&rendered_text(html))
    }

    #[test]
    fn inline_elements_that_touch_read_as_touching_whatever_the_formatting() {
        let head = "    <p class=\"wf-text\">\n        <code class=\"wf-code\">derived </code>\n        <code class=\"wf-code\">live</code>\n        <code class=\"wf-code\">: </code>\n        <code class=\"wf-code\">[Deploy]</code>\n    </p>\n";
        assert_eq!(read(head), "derived live: [Deploy]");
        // A word of its own before the children keeps the space it was
        // given, and only that.
        let words = "<p class=\"wf-text\">\n    Hello \n    <strong>world</strong>\n</p>\n<p class=\"wf-text\">\n    Hello\n    <strong>world</strong>\n</p>\n";
        assert_eq!(read(words), "Hello world Helloworld");
        // Blocks are apart, and so is what a layout box lays out: a flex or
        // grid item is a block to the browser, inline by its tag or not.
        let row = "<div class=\"wf-row p-line\">\n    <code class=\"wf-code\">a</code>\n    <code class=\"wf-code\">b</code>\n</div>\n<p>c</p><p>d<br>e</p>";
        assert_eq!(read(row), "a b c d e");
        // Inside a laid-out item, inline is inline again.
        let nested =
            "<div class=\"wf-stack\"><p class=\"x\"><code>b</code>\n<code>ack</code></p></div>";
        assert_eq!(read(nested), "back");
        // A comment and an empty element say nothing; entities are decoded.
        assert_eq!(
            read("<!--wf-portal--><span></span><b>a &amp;</b><i> b</i>"),
            "a & b"
        );
    }

    #[test]
    fn what_an_escape_stands_for_is_what_a_reader_sees() {
        assert_eq!(
            decode_entities("isn&#x27;t &#39;so&#39; &amp; &lt;b&gt; &quot;q&quot; &#X41;&#66;"),
            "isn't 'so' & <b> \"q\" AB"
        );
        assert_eq!(
            decode_entities("a & b &unknown; &#xZZ; &amp;lt;"),
            "a & b &unknown; &#xZZ; &lt;",
            "one it does not know, or not an escape, stays"
        );
        assert_eq!(decode_entities("&#x1F600; ok"), "😀 ok");
    }
}

/// Runs of whitespace as one space, for comparing what text says.
fn squeeze(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One test that looks: `Ok(true)` when it passed and its snapshot was
/// written, `Ok(false)` when it passed against the one there.
#[expect(
    clippy::too_many_arguments,
    reason = "the project context a test renders in; grouped into a struct when the tools layer is split"
)]
fn run_one(
    shared: &[Declaration],
    test: &TestDecl,
    snapshots: &Path,
    file: &str,
    update: bool,
    messages: &HashMap<String, String>,
    locale: &str,
    literals: bool,
) -> std::result::Result<bool, Vec<String>> {
    let html =
        render_test(shared, test, messages, locale, literals).map_err(|e| vec![e.to_string()])?;
    let mut reasons = Vec::new();
    let empty = crate::project::static_eval::Scope::of([]);
    // What a reader sees: the text, not the markup — `expect "wf-text"` is
    // not satisfied by a class name.
    let visible = rendered_text(&html);
    for step in &test.steps {
        let Step::Expect { text, negated, .. } = step else {
            continue; // a test that acts does not come this way
        };
        let text = crate::project::static_eval::eval(text, &empty)
            .map(|v| v.to_text())
            .unwrap_or_default();
        let found = squeeze(&visible).contains(&squeeze(&text));
        if found == *negated {
            reasons.push(if *negated {
                format!("expected not to find {text:?}, but the render holds it")
            } else {
                format!("expected {text:?}, which the render does not hold")
            });
        }
    }
    if !reasons.is_empty() {
        reasons.push(format!("rendered:\n{}", indent(&html)));
        return Err(reasons);
    }
    let snapshot = snapshot_path(snapshots, file, test);
    if snapshot.exists() && !update {
        let stored = std::fs::read_to_string(&snapshot)
            .at(&snapshot)
            .map_err(|e| vec![e.to_string()])?;
        if stored != html {
            return Err(vec![
                format!("the render differs from {}", snapshot.display()),
                "run `wf test --update` to accept the render".to_string(),
                format!("rendered:\n{}", indent(&html)),
            ]);
        }
        return Ok(false);
    }
    if let Some(dir) = snapshot.parent() {
        std::fs::create_dir_all(dir)
            .at(dir)
            .map_err(|e| vec![e.to_string()])?;
    }
    std::fs::write(&snapshot, &html)
        .at(&snapshot)
        .map_err(|e| vec![e.to_string()])?;
    Ok(true)
}

/// The file a test's snapshot is kept in.
pub fn snapshot_path(snapshots: &Path, file: &str, test: &TestDecl) -> PathBuf {
    let stem = Path::new(file)
        .file_stem()
        .map_or_else(|| "tests".to_string(), |s| s.to_string_lossy().to_string());
    snapshots
        .join(stem)
        .join(format!("{}.html", slug(&test.name)))
}

pub(crate) fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|l| format!("          {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Tests that act ─────────────────────────────────────────────────────────
//
// A test that only looks is rendered: the template engine paints its body
// and the expects read the result. A test that **clicks** cannot be — a
// click runs a handler, which runs an action, which changes a store, which
// repaints. There is one thing that does all of that, and it is a browser.
//
// So a test with an interaction in it is compiled into a real project of
// one page, built, served, and driven. It is the same compiler, the same
// runtime and the same output a reader would get; nothing about the test
// is special except that it is thrown away afterwards.

/// A page of a test's site, open in a browser.
pub trait Page {
    /// Run `script` in the page and hand back what it gave.
    ///
    /// # Errors
    ///
    /// The browser's error when `script` throws or the page cannot be reached.
    fn eval(&mut self, script: &str) -> Result<serde_json::Value>;
    /// Give the page `ms` to react, listening to what it says meanwhile.
    ///
    /// # Errors
    ///
    /// The browser's error when the page cannot be reached.
    fn settle(&mut self, ms: u64) -> Result<()>;
    /// Anything the page said went wrong, in the order it said it: an
    /// exception, a `console.error`.
    fn errors(&self) -> Vec<String>;
}

/// A browser a test that acts runs in: Chrome for `wf test`, the mobile
/// editor's WebView.
pub trait Browser {
    /// Serve the site in `dir` (`index.html`, `app.js`, `styles.css`), open
    /// its root and let it settle for `settle_ms`. The page closes when it is
    /// dropped.
    ///
    /// # Errors
    ///
    /// The browser's error when the site cannot be served or its page opened.
    fn open(&mut self, dir: &Path, settle_ms: u64) -> Result<Box<dyn Page + '_>>;
}

/// What a test's site is built with, beside its program: the project's
/// theme settings, its own scripts, its translations and its styles.
#[derive(Debug, Clone, Copy)]
pub struct Site<'a> {
    pub theme: &'a ThemeConfig,
    pub scripts: &'a [crate::project::project_js::Script],
    pub i18n: Option<&'a I18n>,
    pub styles: &'a Styles,
}

/// Build a test that acts as a site of one page under `work`, open it in
/// `browser`, and play its steps: what went wrong, if anything.
///
/// # Errors
///
/// What [`write_site`] and [`Browser::open`] return, what a step's
/// [`Page::eval`] or [`Page::settle`] returns, and [`IoError`](crate::diagnostics::error::WebFluentError::IoError) when the
/// site's folder cannot be made. A step that fails is `Ok(Err(reasons))`.
pub fn act(
    browser: &mut dyn Browser,
    shared: &[Declaration],
    test: &TestDecl,
    work: &Path,
    site: &Site<'_>,
) -> Result<std::result::Result<(), Vec<String>>> {
    let dir = work.join(slug_acting(&test.name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).at(&dir)?;
    // The program as written, with the test's body as its one page. It is
    // compiled here rather than written back out as source: there is no
    // printer from the tree to the text, and a test does not need one — the
    // compiler takes the tree.
    let mut program = as_program(shared, test);
    crate::syntax::prefer_own_components(&mut program);
    let mut program = crate::check::sema::lower(program);
    if site.i18n.is_some_and(|(i, _)| i.literals) {
        crate::project::i18n::literals::translate(&mut program);
    }
    write_site(&program, &dir, site)?;
    let outcome = match browser.open(&dir, 600) {
        Ok(mut page) => play(page.as_mut(), test),
        Err(e) => Err(e),
    };
    let _ = std::fs::remove_dir_all(&dir);
    outcome
}

/// A test's steps against its open page.
fn play(page: &mut dyn Page, test: &TestDecl) -> Result<std::result::Result<(), Vec<String>>> {
    let mut reasons = Vec::new();
    for step in &test.steps {
        if let Err(why) = step_one(page, step)? {
            reasons.push(why);
            break; // the steps after a failed one mean nothing
        }
    }
    // One last listen, for anything the page said after the last step and
    // has not been heard yet.
    page.settle(80)?;
    // Anything the page itself complained about is the test's problem too:
    // a handler that threw is a failure, not a passing click.
    for error in page.errors() {
        reasons.push(format!("the page reported: {error}"));
    }
    let shown = visible_text(page).unwrap_or_default();
    if reasons.is_empty() {
        return Ok(Ok(()));
    }
    reasons.push(format!("the page showed:\n{}", indent_acting(&shown)));
    Ok(Err(reasons))
}

/// The text a reader would see, for an `expect`.
///
/// # Errors
///
/// What [`Page::eval`] returns.
pub fn visible_text(page: &mut dyn Page) -> Result<String> {
    // The page, and what is drawn above it: an open dialog (an `ask`, a
    // modal) and the toasts — each once, whether or not it is in `<main>`.
    let value = page.eval(
        "(() => { const main = document.querySelector('main') || document.body; \
         const above = [...document.querySelectorAll('dialog[open], .wf-toast-container')] \
           .filter((el) => !main.contains(el)); \
         return [main, ...above].map((el) => el.innerText || '').join('\\n'); })()",
    )?;
    Ok(value.as_str().unwrap_or_default().to_string())
}

/// The steps a test's body may hold, read from `source` — `click "Save"`,
/// `type "Ada" into "Name"`, `press "Enter"`, `expect "Saved"` — as a test
/// writes them, for a command that plays them on a page of its own
/// (`wf inspect --do`).
///
/// # Errors
///
/// The parser's error when `source` is not steps.
pub fn parse_steps(source: &str) -> Result<Vec<Step>> {
    // A test must expect something; the steps alone are what was asked for,
    // so an expectation is added to read them and taken off after.
    let program = crate::syntax::parse_source(
        &format!("test \"steps\" {{\n{source}\nexpect not \"wf-inspect-unseen\"\n}}\n"),
        "--do",
    )?;
    let mut steps = program
        .declarations
        .into_iter()
        .find_map(|d| match d {
            Declaration::Test(t) => Some(t.steps),
            _ => None,
        })
        .unwrap_or_default();
    steps.pop();
    Ok(steps)
}

/// `step_one`, for a command that drives a page of its own.
///
/// # Errors
///
/// What `step_one` returns.
pub fn play_step(page: &mut dyn Page, step: &Step) -> Result<std::result::Result<(), String>> {
    step_one(page, step)
}

/// One step, against the open page.
fn step_one(page: &mut dyn Page, step: &Step) -> Result<std::result::Result<(), String>> {
    let text = |e: &Expr| {
        crate::project::static_eval::eval(e, &crate::project::static_eval::Scope::of([]))
            .map(|v| v.to_text())
            .unwrap_or_default()
    };
    match step {
        Step::Expect {
            text: want,
            negated,
            ..
        } => {
            let want = text(want);
            // Read as a rendered test reads it: runs of white space are one
            // space, so `expect "Total tasks 3"` finds a card's title and its
            // number on two lines, in a browser as in a render.
            // A page still moving — a row on its way out, a fetch on its
            // way in — is read again until it holds, for up to a second
            // and a half: a quick page answers at once.
            let until = std::time::Instant::now() + std::time::Duration::from_millis(1500);
            let mut found = squeeze(&visible_text(page)?).contains(&squeeze(&want));
            while found == *negated && std::time::Instant::now() < until {
                page.settle(50)?;
                found = squeeze(&visible_text(page)?).contains(&squeeze(&want));
            }
            if found == *negated {
                return Ok(Err(if *negated {
                    format!("expected not to find {want:?}, and the page shows it")
                } else {
                    format!("expected {want:?}, which the page does not show")
                }));
            }
        }
        Step::ExpectFocus { name, .. } => {
            let want = squeeze(&text(name));
            // What a reader hears the focused element called: its
            // `aria-label`, its label, its placeholder, or its text.
            let read = "(() => { const el = document.activeElement; if (!el || el === document.body) return ''; \
                const label = el.labels && el.labels[0] ? el.labels[0].innerText : ''; \
                return (el.getAttribute('aria-label') || label || el.getAttribute('placeholder') || el.innerText || el.value || '').trim(); })()";
            let until = std::time::Instant::now() + std::time::Duration::from_millis(1500);
            let mut now = squeeze(page.eval(read)?.as_str().unwrap_or_default());
            while now != want && !now.contains(&want) && std::time::Instant::now() < until {
                page.settle(50)?;
                now = squeeze(page.eval(read)?.as_str().unwrap_or_default());
            }
            if now != want && !now.contains(&want) {
                return Ok(Err(if now.is_empty() {
                    format!("expected the focus on {want:?}, and nothing has it")
                } else {
                    format!("expected the focus on {want:?}, and it is on {now:?}")
                }));
            }
        }
        Step::Click { target, .. } => {
            let name = text(target);
            let done = page.eval(&step_script("click", &name, ""))?;
            if done.as_str() != Some("ok") {
                return Ok(Err(cannot_act(
                    done.as_str(),
                    &format!("nothing to click called {name:?}"),
                    &name,
                )));
            }
            page.settle(120)?;
        }
        Step::Type {
            text: what, into, ..
        } => {
            let what = text(what);
            let into = text(into);
            let done = page.eval(&step_script("type", &into, &what))?;
            match done.as_str() {
                Some("ok") => {}
                Some("no-option") => {
                    return Ok(Err(format!("{into:?} has no option {what:?}")));
                }
                other => {
                    return Ok(Err(cannot_act(
                        other,
                        &format!("no control called {into:?} to type into"),
                        &into,
                    )));
                }
            }
            page.settle(120)?;
        }
        Step::ExpectCalled {
            endpoint, negated, ..
        } => {
            // Every endpoint the page asked, as the runtime counted them
            // when a fake answered — or no fake did.
            let read = format!(
                "(window.__wfCalls || (() => {{ try {{ return JSON.parse(sessionStorage.getItem('wf:calls') || '[]'); }} catch (e) {{ return []; }} }})()).includes({})",
                serde_json::to_string(endpoint).unwrap_or_default()
            );
            let called = |page: &mut dyn Page| -> Result<bool> {
                Ok(page.eval(&read)?.as_bool().unwrap_or(false))
            };
            let mut found = called(page)?;
            // A call still on its way is waited for; one that must not come
            // is given a moment to.
            let until = std::time::Instant::now()
                + std::time::Duration::from_millis(if *negated { 150 } else { 1500 });
            while (found == *negated || *negated) && std::time::Instant::now() < until {
                page.settle(50)?;
                found = called(page)?;
                if found && *negated {
                    break;
                }
            }
            if found == *negated {
                return Ok(Err(if *negated {
                    format!("expected no call of {endpoint}, and the page made one")
                } else {
                    format!("expected a call of {endpoint}, and the page made none")
                }));
            }
        }
        Step::Press { key, target, .. } => {
            let key = text(key);
            let where_ = target.as_ref().map(text).unwrap_or_default();
            let done = page.eval(&step_script("press", &where_, &key))?;
            if done.as_str() != Some("ok") {
                return Ok(Err(cannot_act(
                    done.as_str(),
                    &format!("nothing called {where_:?} to press {key:?} in"),
                    &where_,
                )));
            }
            page.settle(120)?;
        }
    }
    Ok(Ok(()))
}

/// Compile `program` into `dir`: the three files a page needs.
///
/// Not `wf build`, which reads a project from disk and writes a site with
/// everything a site has. This is the same code generators over a program
/// already in hand, for a page that exists for one test and is then thrown
/// away.
///
/// # Errors
///
/// [`IoError`](crate::diagnostics::error::WebFluentError::IoError) when a file of the site cannot be written or `public/`
/// cannot be served beside it, [`ConfigError`](crate::diagnostics::error::WebFluentError::ConfigError) when the theme config
/// names a dark theme nothing declares, and [`CodegenError`](crate::diagnostics::error::WebFluentError::CodegenError) when a
/// theme's token is not a literal.
pub fn write_site(program: &Program, dir: &Path, site: &Site<'_>) -> Result<()> {
    write_site_at(program, dir, site, "")
}

/// [`write_site`], for a site served under `base` (`/examples/card/1`)
/// rather than at the root: its files and its route are addressed from
/// there.
///
/// # Errors
///
/// What [`write_site`] returns.
pub fn write_site_at(program: &Program, dir: &Path, site: &Site<'_>, base: &str) -> Result<()> {
    let mut config = crate::project::config::ProjectConfig::default_config("test");
    config.build.base_path = base.to_string();
    config.theme = site.theme.clone();
    config.i18n = site.i18n.map(|(c, _)| c.clone());
    for script in site.scripts {
        let to = dir.join(&script.href);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).at(parent)?;
        }
        std::fs::write(&to, &script.source).at(&to)?;
        config.build.scripts.push(script.href.clone());
    }
    // One page, one file: a split build links a chunk that this does not
    // write, and the shell would be served in its place.
    config.build.split = false;
    let mut js = crate::web::codegen::JsCodegen::new();
    if !base.is_empty() {
        js.set_base_path(base.to_string());
    }
    // A test's page asks no network: what answers its endpoints is the
    // fakes it names, and a call none answers fails the test.
    js.set_fakes(true);
    if let Some((c, tables)) = site.i18n {
        js.set_i18n(c.default_locale.clone(), tables.clone());
    }
    std::fs::write(dir.join("app.js"), js.generate(program)).at(dir.join("app.js"))?;

    // The sheet in the build's order: the engine's rules and the theme's
    // tokens, the dark theme's, the project's own stylesheets, then what
    // its `style { }` blocks compile to.
    let tokens = crate::project::themes::resolve_tokens(program, &config.theme)?;
    let mut css = crate::web::codegen::generate_css_for(&tokens, config.theme.builtin, program);
    if let Some(dark) = crate::project::themes::resolve_dark_tokens(program, &config.theme)? {
        css.push_str(&crate::web::codegen::dark_css(&dark));
    }
    css.push_str(&site.styles.css);
    css.push_str(&crate::project::scoped_css::scoped_rules(program));
    std::fs::write(dir.join("styles.css"), css).at(dir.join("styles.css"))?;
    // The stylesheets the project links, linked as the built page links
    // them.
    config.meta.stylesheets.clone_from(&site.styles.linked);
    config.meta.integrity.clone_from(&site.styles.integrity);

    std::fs::write(
        dir.join("index.html"),
        crate::web::codegen::generate_html(&config, program),
    )?;
    // `public/` beside the page, as the build copies it: a site-relative
    // sheet, or a font a sheet names, is found where it is on the site.
    // Last, so nothing written above goes through a link into the project.
    if let Some(public) = &site.styles.public {
        serve_public(public, dir)?;
    }
    Ok(())
}

/// What is in `public/` (an absolute path), reachable from the test's site
/// `dir` as from the built one: each entry linked in, or copied where a
/// link cannot be made.
/// What the test's site wrote itself is its own, as the build's own files
/// are; a folder both have is gone into.
fn serve_public(public: &Path, dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(public).at(public)? {
        let from = entry?.path();
        let Some(name) = from.file_name() else {
            continue;
        };
        let to = dir.join(name);
        if to.symlink_metadata().is_ok() {
            if from.is_dir() && to.symlink_metadata()?.is_dir() {
                serve_public(&from, &to)?;
            }
            continue;
        }
        #[cfg(unix)]
        let linked = std::os::unix::fs::symlink(&from, &to).is_ok();
        #[cfg(not(unix))]
        let linked = false;
        if !linked {
            copy_into(&from, &to)?;
        }
    }
    Ok(())
}

fn copy_into(from: &Path, to: &Path) -> Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to).at(to)?;
        for entry in std::fs::read_dir(from).at(from)? {
            let path = entry?.path();
            if let Some(name) = path.file_name() {
                copy_into(&path, &to.join(name))?;
            }
        }
    } else {
        std::fs::copy(from, to).at(from)?;
    }
    Ok(())
}

/// The program a test becomes: everything the project declares, and the
/// test's body as the one page.
fn as_program(shared: &[Declaration], test: &TestDecl) -> Program {
    one_page(
        shared,
        &test.name,
        with_data(shared, &test.body, test.data.as_ref()),
        &test.fakes,
        test.data.as_ref(),
        test.span,
    )
}

/// The test's body with its `data` in it, as a render reads it: a `state`
/// (or `derived` value) the data names starts at the data's value, and a
/// name the data gives that nothing declares — `data: { s: … }` for a body
/// that places `Tile(s)` — is a `state` of the body, read by name as the
/// render reads it. A store's name is the store's seed ([`one_page`]).
fn with_data(shared: &[Declaration], body: &[Statement], data: Option<&Expr>) -> Vec<Statement> {
    let mut body = seeded(body, data);
    let Some(Expr::MapLiteral(pairs)) = data else {
        return body;
    };
    let declared = |name: &str| {
        body.iter().any(|s| match &s.kind {
            StatementKind::State(st) => st.name == name,
            StatementKind::Derived(d) => d.name == name,
            _ => false,
        }) || shared
            .iter()
            .any(|d| crate::check::edit::declaration_name(d) == Some(name))
    };
    let given: Vec<Statement> = pairs
        .iter()
        .map(|(k, v)| (k.trim_matches('"'), v))
        .filter(|(k, _)| !declared(k))
        .map(|(k, v)| {
            Statement::new(
                StatementKind::State(crate::syntax::parser::ast::StateDecl {
                    name: k.to_string(),
                    ty: None,
                    value: v.clone(),
                    persist: false,
                    policy: None,
                    doc: None,
                }),
                crate::syntax::parser::ast::Span::default(),
            )
        })
        .collect();
    body.splice(0..0, given);
    body
}

/// An example as a site of one page, written under `dir`: the project's
/// declarations, and its body as the page, drawn as the project draws its
/// own — its theme, its scripts, its translations, its styles. What
/// `wf docs` shows in a frame.
///
/// # Errors
///
/// What [`write_site`] returns.
pub fn example_site(
    shared: &[Declaration],
    example: &crate::syntax::parser::ast::ExampleDecl,
    dir: &Path,
    base: &str,
    site: &Site<'_>,
) -> Result<()> {
    std::fs::create_dir_all(dir).at(dir)?;
    let mut program = one_page(
        shared,
        &example.name,
        example.body.clone(),
        &example.fakes,
        None,
        example.span,
    );
    crate::syntax::prefer_own_components(&mut program);
    let mut program = crate::check::sema::lower(program);
    if site.i18n.is_some_and(|(i, _)| i.literals) {
        crate::project::i18n::literals::translate(&mut program);
    }
    write_site_at(&program, dir, site, base)
}

/// Everything the project declares but what is drawn, and `body` as the
/// one page, at `/`.
fn one_page(
    shared: &[Declaration],
    title: &str,
    body: Vec<Statement>,
    fakes: &[String],
    data: Option<&Expr>,
    span: crate::syntax::parser::ast::Span,
) -> Program {
    let mut declarations: Vec<Declaration> = shared
        .iter()
        .filter(|d| !is_drawn(d) && !matches!(d, Declaration::Fake(f) if !fakes.contains(&f.name)))
        .cloned()
        .map(|d| match d {
            // `data: { Cart: { items: […] } }` starts the store there, as a
            // test that only looks does.
            Declaration::Store(mut store) => {
                if let Some(Expr::MapLiteral(pairs)) = data
                    && let Some((_, given)) = pairs
                        .iter()
                        .find(|(k, _)| k.trim_matches('"') == store.name)
                {
                    store.body = seeded(&store.body, Some(given));
                }
                Declaration::Store(store)
            }
            // A component's own state — a page's, placed by its name — is
            // seeded by name too, as a render seeds every body's.
            Declaration::Component(mut c) => {
                c.body = seeded(&c.body, data);
                Declaration::Component(c)
            }
            other => other,
        })
        .collect();
    declarations.push(Declaration::Page(PageDecl {
        name: "Test".to_string(),
        path: "/".to_string(),
        title: Some(title.to_string()),
        title_expr: None,
        description_expr: None,
        image_expr: None,
        description: Some("A test.".to_string()),
        guard: None,
        redirect: None,
        image: None,
        image_alt: None,
        page_type: None,
        noindex: true,
        lang: None,
        translates: None,
        layout: None,
        params: Vec::new(),
        head: Vec::new(),
        paths: None,
        body,
        span,
        header_span: span,
        body_span: span,
    }));
    Program { declarations }
}

/// The body, with any `state` the test's `data` names starting at that
/// value — the same seeding a rendered test gets, for the one thing a built
/// page can take it from. A `derived` value the data names is the data's,
/// as in a render: a `state` that starts there.
fn seeded(body: &[Statement], data: Option<&Expr>) -> Vec<Statement> {
    let Some(Expr::MapLiteral(pairs)) = data else {
        return body.to_vec();
    };
    let given = |name: &str| {
        pairs
            .iter()
            .find(|(k, _)| k.trim_matches('"') == name)
            .map(|(_, v)| v.clone())
    };
    body.iter()
        .map(|stmt| {
            let mut stmt = stmt.clone();
            match &mut stmt.kind {
                StatementKind::State(state) => {
                    if let Some(value) = given(&state.name) {
                        state.value = value;
                    }
                }
                StatementKind::Derived(d) if !matches!(d.value, Expr::Lambda(..)) => {
                    if let Some(value) = given(&d.name) {
                        stmt.kind = StatementKind::State(crate::syntax::parser::ast::StateDecl {
                            name: d.name.clone(),
                            ty: d.ty.clone(),
                            value,
                            persist: false,
                            policy: None,
                            doc: d.doc.clone(),
                        });
                    }
                }
                _ => {}
            }
            stmt
        })
        .collect()
}

/// The page-side half of a step.
///
/// Everything is found the way a reader finds it: by the name on it. A
/// button is its text, a control is its label or its placeholder — which is
/// the same name the accessibility checks hold a page to, so a test that
/// passes here is a page somebody can use.
/// The script a step plays in its page: `what` is `"click"`, `"type"` or
/// `"press"`, `name` what a reader calls the thing it acts on (empty for a
/// `press` into what has focus), `value` what is typed or the key. It
/// evaluates to `"ok"`, `"none"` (nothing by that name) or `"no-option"` (a
/// select without that option).
///
/// A [`Page`] that is not Chrome — the editor's own window, playing its
/// scenarios — runs the same steps by evaluating this, so a scenario finds
/// a control exactly as a test does.
pub fn step_script(what: &str, name: &str, value: &str) -> String {
    format!(
        "(() => {{\n{FIND}\n  const what = {}, name = {}, value = {};\n{ACT}\n}})()",
        serde_json::to_string(what).unwrap_or_default(),
        serde_json::to_string(name).unwrap_or_default(),
        serde_json::to_string(value).unwrap_or_default(),
    )
}

const FIND: &str = r#"
  const words = (s) => (s || "").replace(/\s+/g, " ").trim();
  // Its text as a reader hears it: what is hidden from readers (a menu
  // item's shortcut, a glyph) is not part of its name, nor is a menu of
  // its own (a submenu's items).
  const heard = (el) => {
    let t = "";
    for (const c of el.childNodes) {
      if (c.nodeType === 3) t += c.textContent;
      else if (
        c.nodeType === 1 &&
        c.getAttribute("aria-hidden") !== "true" &&
        c.getAttribute("role") !== "menu"
      )
        t += heard(c);
    }
    return t;
  };
  const labelledBy = (el) => {
    const ids = el.getAttribute("aria-labelledby");
    if (!ids) return null;
    return ids
      .split(/\s+/)
      .map((id) => document.getElementById(id))
      .filter(Boolean)
      .map(heard)
      .join(" ");
  };
  const named = (el, n) => {
    const label = el.labels && el.labels[0] ? el.labels[0].textContent : "";
    return [
      el.getAttribute("aria-label"),
      labelledBy(el),
      label,
      el.getAttribute("placeholder"),
      el.getAttribute("title"),
      el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.tagName === "SELECT"
        ? null
        : heard(el),
    ].some((t) => words(t) === n);
  };
  // What a reader can reach: drawn, and not inside something hidden, inert
  // or a dialog that is not open — never a control in a closed sheet.
  const reachable = (el) => {
    if (el.closest("[hidden], [inert], dialog:not([open])")) return false;
    if (typeof el.checkVisibility === "function") return el.checkVisibility();
    return el.getClientRects().length > 0;
  };
  // How the last search went: how many a reader could reach, and how many
  // carried the name but were hidden.
  let seen = { reachable: 0, hidden: 0 };
  const find = (n, kinds) => {
    const all = [...document.querySelectorAll(kinds)].filter((el) => named(el, n));
    // The one whose own name it is, not an ancestor that contains it.
    const own = all.filter((el) => !all.some((o) => o !== el && el.contains(o)));
    const shown = own.filter(reachable);
    seen = { reachable: shown.length, hidden: own.length - shown.length };
    // The first in the page's order, as a reader meets it.
    return shown[0] || null;
  };
  // What a step says when it cannot act: nothing by the name, only hidden
  // ones, or more than one a reader could mean.
  const missing = () => (seen.hidden > 0 ? "hidden:" + seen.hidden : "none");
  const many = () => (seen.reachable > 1 ? "many:" + seen.reachable : null);
  // A reader brings what they act on into view first: a control below the
  // fold is scrolled to, so what it opens (a popconfirm, a menu) is drawn
  // on the screen, at it, as it would be for them.
  const inView = (el) => {
    if (el && el.scrollIntoView && el !== document.body) el.scrollIntoView({ block: "center", inline: "nearest", behavior: "instant" });
  };
"#;

const ACT: &str = r#"
  if (what === "click") {
    // Whatever a reader can press, by its name: a button or a link, a
    // control, or a widget's role — a tab, a menu's item, an option, a row
    // of a tree.
    const el = find(name, "button, a, [role=button], [role=link], [role=tab], [role=menuitem], [role=menuitemcheckbox], [role=menuitemradio], [role=option], [role=treeitem], [role=switch], [role=checkbox], [role=radio], input, select, summary, label, [onclick]");
    if (!el) return missing();
    if (many()) return many();
    inView(el);
    // A real press puts the focus on what it presses, when that can take
    // it — so a key pressed next goes there, as it would for a reader.
    if (el.focus && el.matches && el.matches("button, a[href], input, select, textarea, summary, [tabindex]")) el.focus({ preventScroll: true });
    el.click();
    return "ok";
  }
  if (what === "type") {
    const el = find(name, "input, textarea, select");
    if (!el) return missing();
    if (many()) return many();
    inView(el);
    el.focus();
    if (el.tagName === "SELECT") {
      // A select is chosen by what the reader sees: an option's text, or
      // failing that its value.
      const opt = [...el.options].find((o) => o.text.trim() === value) ||
        [...el.options].find((o) => o.value === value);
      if (!opt) return "no-option";
      el.value = opt.value;
    } else {
      const setter = Object.getOwnPropertyDescriptor(
        el.tagName === "TEXTAREA" ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype,
        "value",
      );
      if (setter && setter.set) setter.set.call(el, value);
      else el.value = value;
    }
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
    return "ok";
  }
  if (what === "press") {
    const el = name ? find(name, "input, textarea, select, button, a, [tabindex]") : (document.activeElement || document.body);
    if (!el) return missing();
    if (name && many()) return many();
    if (name) inView(el);
    // `press "ctrl+k"`, `press "mod+shift+p"`: the modifiers held with the
    // key, read as `on key("…")` reads them — `mod` is ⌘ on a Mac and Ctrl
    // elsewhere — and the key's short names (`esc`, `down`, `space`).
    const plus = value.length > 1 && value.endsWith("+");
    const parts = (plus ? value.slice(0, -1) : value).split("+");
    let key = plus ? "+" : parts.pop();
    if (plus && parts[parts.length - 1] === "") parts.pop();
    const names = { esc: "Escape", up: "ArrowUp", down: "ArrowDown", left: "ArrowLeft", right: "ArrowRight", space: " ", del: "Delete" };
    if (names[key.toLowerCase()]) key = names[key.toLowerCase()];
    const mac = /Mac|iP(hone|ad|od)/.test(navigator.platform || "");
    const held = { ctrlKey: false, shiftKey: false, altKey: false, metaKey: false };
    for (const m of value.length > 1 ? parts.map((p) => p.toLowerCase()) : []) {
      if (m === "ctrl" || m === "control") held.ctrlKey = true;
      else if (m === "shift") held.shiftKey = true;
      else if (m === "alt" || m === "option") held.altKey = true;
      else if (m === "meta" || m === "cmd" || m === "command") held.metaKey = true;
      else if (m === "mod") { if (mac) held.metaKey = true; else held.ctrlKey = true; }
    }
    const init = { key, code: key.length === 1 && /[a-z]/i.test(key) ? "Key" + key.toUpperCase() : key, bubbles: true, cancelable: true, ...held };
    const down = el.dispatchEvent(new KeyboardEvent("keydown", init));
    el.dispatchEvent(new KeyboardEvent("keyup", init));
    // What the browser does with a real Escape that nothing cancelled: the
    // open modal dialog is asked (`cancel`) and, unless it refuses, closed.
    // A synthetic key event does not reach that default.
    if (down && (key === "Escape" || key === "Esc")) {
      const open = [...document.querySelectorAll("dialog[open]")];
      const dialog = (el.closest && el.closest("dialog[open]")) || open[open.length - 1];
      if (dialog && dialog.dispatchEvent(new Event("cancel", { cancelable: true }))) dialog.close();
    }
    // A real Enter in a form's text field submits the form (its implicit
    // submission); a synthetic one does not, so the step does it.
    if (down && key === "Enter" && el.form && el.tagName === "INPUT"
        && !/^(button|submit|reset|checkbox|radio|file)$/.test(el.type) && el.form.requestSubmit) {
      el.form.requestSubmit();
    }
    return "ok";
  }
  return "none";
"#;

/// Why a step found nothing to act on: nothing by the name, only hidden
/// ones (in a closed dialog, a hidden branch), or several a reader could
/// mean.
fn cannot_act(answer: Option<&str>, none: &str, name: &str) -> String {
    match answer.and_then(|a| a.split_once(':')) {
        Some(("hidden", n)) => format!(
            "{none} that a reader can reach: {n} by that name are hidden (a closed dialog, a hidden branch)"
        ),
        Some(("many", n)) => format!(
            "{n} elements a reader can reach are called {name:?}: name the one meant more closely"
        ),
        _ => none.to_string(),
    }
}

fn indent_acting(text: &str) -> String {
    text.lines().take(24).fold(String::new(), |mut acc, l| {
        let _ = writeln!(acc, "        {l}");
        acc
    })
}

/// A test's folder name: act's own, which keeps a leading dash's absence.
fn slug_acting(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}
