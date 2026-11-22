//! The `wf` command's arguments, and what each runs.

use crate::diagnostics;
use crate::diagnostics::error;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wf")]
#[command(about = "WebFluent — a language for websites: one source, compiled to a static site, a single-page app, a PDF or a slide deck", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// `wf commands` is a command too: its variant is named as it is typed.
#[derive(Subcommand)]
enum Commands {
    /// Create a new WebFluent project
    Init {
        /// Project name
        name: String,
        /// Template: "spa" (default), "static", "pdf", or "slides"
        #[arg(short, long, default_value = "spa", value_parser = ["spa", "static", "pdf", "slides"])]
        template: String,
        /// Start it translated: the locales, comma-separated, the default
        /// first ("en,ar"). Every visible string goes through t(), and
        /// src/translations/ holds a file for each. spa and static only
        #[expect(
            clippy::doc_markdown,
            reason = "this comment is the option's --help text, shown as written"
        )]
        #[arg(short, long, value_name = "LOCALES")]
        locales: Option<String>,
    },
    /// Build the project: a site or app (HTML, CSS, JS), a PDF, a slide deck or custom elements
    Build {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Print what the build weighs: every file, gzipped, and the runtime
        /// modules it kept and left out
        #[arg(long)]
        stats: bool,
        /// How findings are written: `human` (default), `json`, `sarif`
        /// (code scanning) or `github` (annotations on a pull request)
        #[arg(long, default_value = "human", value_parser = format_parser())]
        format: diagnostics::format::Format,
        /// Let a warning stop the build as an error does
        #[arg(long)]
        deny_warnings: bool,
        /// Reuse what earlier builds rendered, kept in `.wf-cache/build/`:
        /// only the pages an edit changed are rendered again
        #[arg(long)]
        cache: bool,
        /// For an editor's preview: stamp every element with its node id
        /// (`data-wf-node`) and write `nodes.json`, from each id to its source
        #[arg(long)]
        studio: bool,
        /// Let `build.output` name a directory outside the project; without
        /// it a build writes only inside the project
        #[arg(long)]
        allow_output_outside: bool,
    },
    /// Check the project — every finding a build would report — and write
    /// nothing
    Check {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// How findings are written: `human` (default), `json`, `sarif`
        /// (code scanning) or `github` (annotations on a pull request)
        #[arg(long, default_value = "human", value_parser = format_parser())]
        format: diagnostics::format::Format,
        /// Fail on a warning too
        #[arg(long)]
        deny_warnings: bool,
        /// Accept a `build.output` outside the project, as `wf build
        /// --allow-output-outside` does; without it the check reports what
        /// the build would refuse
        #[arg(long)]
        allow_output_outside: bool,
    },
    /// Read and change `webfluent.app.json`: every setting, what the
    /// project sets, and an edit that keeps the file's shape
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// A service's contract as files, from its OpenAPI spec: add one, sync
    /// it, check it is current, turn `api … from` into files, list them
    Api {
        #[command(subcommand)]
        action: ApiAction,
    },
    /// The animations an element can play, the eases and speeds, the route
    /// transitions and the project's default
    Motion {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// What an agent is given: the files, the open file, the problems, the
    /// components, the theme, the config with only its public env, the
    /// language reference, and what was withheld
    Context {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// The file open in the editor, and its selection as byte offsets:
        /// `src/pages/Home.wf` or `src/pages/Home.wf:120-180`
        #[arg(long)]
        open: Option<String>,
        /// The language reference (the skill's `references/`) as each file's
        /// title and sections (`index`), or whole (`full`)
        #[arg(long, default_value = "index")]
        guide: String,
        /// Print JSON, for tools; otherwise one Markdown document
        #[arg(long)]
        json: bool,
    },
    /// The translations as a table: each locale's plural forms and what it
    /// lacks, each key's messages, and where a `t()` names it
    Translations {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Only the keys this locale has no message for
        #[arg(long)]
        missing: Option<String>,
        /// Only the keys nothing names
        #[arg(long)]
        unused: bool,
        /// CLDR's plural rules instead: every locale's forms, and the form
        /// each of a run of counts takes
        #[arg(long)]
        plurals: bool,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// What a path picker lists at a place: the names in scope there, each
    /// with its type and where it comes from, and the fields to go on into
    Scope {
        /// The place: `src/pages/Signup.wf:24` or `…:24:9`
        place: String,
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// A template's data, a JSON file, to list after the scope
        #[arg(long)]
        data: Option<PathBuf>,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// The rules a `validate` block may hold, the named patterns, when a
    /// form shows its messages, and every state the project holds to a rule
    Rules {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// The theme the build uses: every token, declared or inherited, where
    /// each is used, and its contrast pairs
    Theme {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// Every component a call can name — the built-ins and the project's —
    /// with what each takes: its props and their kinds, events, slots,
    /// parts and children
    Components {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// What `public/` holds — each file's kind, size and dimensions, and
    /// where the project names it — or what the media pipeline will make of
    /// one image
    Assets {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// What the pipeline will make of this image of `public/`:
        /// `/hero.png`
        #[arg(long, value_name = "PATH")]
        plan: Option<String>,
        /// Only the files nothing names
        #[arg(long)]
        unused: bool,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// Every `wf` command, its options and what it writes, as data
    #[expect(
        clippy::enum_variant_names,
        reason = "the variant is the `wf commands` subcommand, named as it is typed"
    )]
    Commands {
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// Fix every finding that has a fix — or every one of a code — as
    /// `wf check` and the editor offer it, and say what is left
    Fix {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Only this code's findings: `V01`, `A01`
        #[arg(long)]
        code: Option<String>,
        /// Write nothing; say what would change
        #[arg(long)]
        dry_run: bool,
    },
    /// What a diagnostic code means, a program that draws it, and the fix;
    /// with no code, every code
    Explain {
        /// The code: `T05`, `A01`, `E101`
        code: Option<String>,
        /// Print JSON, for tools: one code, or every code without one
        #[arg(long)]
        json: bool,
    },
    /// Start the development server
    Serve {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// The address to listen on: this machine only by default; `0.0.0.0`
        /// lets a phone on the same network reach it
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Let `build.output` name a directory outside the project
        #[arg(long)]
        allow_output_outside: bool,
    },
    /// Serve a built project as a host would: `$PORT`, the build's headers
    /// and compressed files, a single-page build's shell for every route,
    /// and `--proxy` for an API elsewhere
    Start {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// The port (default: `$PORT`, else 3000)
        #[arg(long)]
        port: Option<u16>,
        /// The address to listen on (default: every interface)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// Forward requests under a path to another server, the path replaced
        /// by the URL: `--proxy /api=https://api.example.com` (repeatable;
        /// http or https)
        #[arg(long, value_name = "PREFIX=URL")]
        proxy: Vec<String>,
        /// Let `build.output` name a directory outside the project
        #[arg(long)]
        allow_output_outside: bool,
    },
    /// One page as a reader meets it, as text: landmarks, headings, links,
    /// buttons, fields, tables, and anything that failed
    Inspect {
        /// A route of the project's build (`/pokemon/pikachu`), or a URL of a
        /// server already running
        target: String,
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Forward requests under a path to another server, as `wf start` does
        #[arg(long, value_name = "PREFIX=URL")]
        proxy: Vec<String>,
        /// The window's width, in CSS pixels
        #[arg(long, default_value_t = 1280)]
        width: u32,
        /// The window's height, in CSS pixels
        #[arg(long, default_value_t = 800)]
        height: u32,
        /// A phone: 390×844, laid out as a phone lays it out
        #[arg(long)]
        mobile: bool,
        /// Also save a PNG of the page (to FILE, or a file it names)
        #[arg(long, value_name = "FILE", num_args = 0..=1, default_missing_value = "-")]
        screenshot: Option<PathBuf>,
        /// The whole page in the screenshot, not only the window
        #[arg(long)]
        full_page: bool,
        /// The most milliseconds the page is given to settle
        #[arg(long, default_value_t = 5000, value_name = "MS")]
        wait: u64,
        /// The most lines of outline it prints
        #[arg(long, default_value_t = 400)]
        lines: usize,
        /// A step to play before the page is read, as a test writes it:
        /// `click "Name"`, `type "pika" into "Search"`, `press "Enter"`,
        /// `expect "Saved"` (repeatable, in order)
        #[arg(long = "do", value_name = "STEP")]
        steps: Vec<String>,
        /// Build the site with these fakes answering its endpoints, as
        /// `wf verify --fakes` does, and open that
        #[arg(long, value_name = "FILE")]
        fakes: Option<PathBuf>,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
        /// Let `build.output` name a directory outside the project
        #[arg(long)]
        allow_output_outside: bool,
    },
    /// Generate a new page, component, or store — or a block, a whole page
    /// to own: sign-in, pricing, dashboard, settings
    Generate {
        /// What to generate: page, component, store, or block
        #[arg(value_parser = ["page", "component", "store", "block"])]
        kind: String,
        /// Name of the item to generate; for a block, which: sign-in,
        /// pricing, dashboard or settings
        name: String,
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Render a .wf template with JSON data
    Render {
        /// Template file (.wf or .wfx), or a directory: every template under
        /// it as one, components shared
        template: PathBuf,
        /// JSON data file (reads stdin if omitted)
        #[arg(long)]
        data: Option<PathBuf>,
        /// Output format: "html" (default), "html-fragment", "pdf", or "slides"
        #[arg(short, long, default_value = "html", value_parser = ["html", "html-fragment", "pdf", "slides"])]
        format: String,
        /// Output file (stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Which `Theme` declared in the template to render with. Only needed
        /// when the template declares more than one.
        #[arg(long)]
        theme: Option<String>,
        /// A design token over the theme's, `NAME=VALUE`; may be given more
        /// than once
        #[arg(long = "token", value_name = "NAME=VALUE")]
        tokens: Vec<String>,
        /// The page to render, by name, when the template has several
        #[arg(long)]
        page: Option<String>,
        /// The document's language, `<html lang>` (default: the page's `lang:`, else `en`)
        #[arg(long)]
        lang: Option<String>,
    },
    /// Carry a project forward: WebFluent 2 → 3, 3 → 4, 4 → 4.2 and 6 → 7, each step that applies
    Migrate {
        /// Project directory (default: current directory), or one .wf file
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Report what would change without writing anything
        #[arg(long)]
        check: bool,
        /// Print the migrated text of one file to stdout instead of writing it
        #[arg(long)]
        stdout: bool,
        /// Write the migrated files in the indented layout, as `.wfx`
        #[arg(long)]
        wfx: bool,
    },
    /// Document the project's components as a site, each `example` drawn live
    Docs {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Where to write the site (default: docs/)
        #[arg(short, long, default_value = "docs")]
        out: PathBuf,
    },
    /// Run the project's `test "…" { … }` declarations under tests/
    Test {
        /// Project directory (default: current directory), or one test file
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Rewrite every snapshot with the current render
        #[arg(long)]
        update: bool,
    },
    /// What this project trusts: markup it did not write, other origins,
    /// what it keeps on the reader's machine, its `env` names, its policy
    /// and its dependencies
    Audit {
        /// Project directory (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// Load every page of a built project in a real browser: errors,
    /// failed requests, paint timing, and which built-ins it drew
    Verify {
        /// Project directory (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Print JSON, for a pipeline
        #[arg(long)]
        json: bool,
        /// Fail a route whose first paint is slower than this, in milliseconds
        #[arg(long, value_name = "MS")]
        budget: Option<u64>,
        /// Visit every route again as a reader whose storage holds what the
        /// previous build's pages kept
        #[arg(long)]
        returning_visitor: bool,
        /// Let `build.output` name a directory outside the project
        #[arg(long)]
        allow_output_outside: bool,
        /// Build the site with these fakes answering its endpoints, then load
        /// that: for a site whose API is elsewhere
        #[arg(long, value_name = "FILE")]
        fakes: Option<PathBuf>,
    },
    /// Describe every built-in component: props, cases, flags, events, slots, parts
    Registry {
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
    },
    /// Describe what a project declares: enums, types, components, stores, pages
    Types {
        /// Project directory (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
        /// List every place a value's type is unknown, and why
        #[arg(long)]
        holes: bool,
    },
    /// Format a project's source files; with `--to`, rewrite them in the
    /// other layout: `.wfx` (blocks by indentation) or `.wf` (blocks in braces)
    Fmt {
        /// Project directory (default: current directory), or one file
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Change the layout instead: `wfx` or `wf`
        #[arg(long, value_parser = ["wfx", "wf"])]
        to: Option<String>,
        /// Write nothing; fail when a file is not formatted
        #[arg(long)]
        check: bool,
        /// Print the formatted text to stdout instead of writing it
        #[arg(long)]
        stdout: bool,
    },
}

#[derive(clap::Subcommand)]
enum ConfigAction {
    /// Every setting: its type, its default and what it does
    Schema {
        /// As JSON, for a tool
        #[arg(long)]
        json: bool,
    },
    /// What the project sets at PATH, or the default
    Get {
        /// A dotted path: `build.base_path`, `build.budget.app.js`
        path: String,
        /// The project's folder
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Set PATH to VALUE — JSON, or a word as a string — keeping the
    /// file's shape
    Set {
        /// A dotted path: `build.base_path`, `meta.site_url`
        path: String,
        /// The value: JSON (`true`, `3`, `["en", "ar"]`), or a word, which is
        /// taken as a string
        value: String,
        /// The project's folder
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Remove PATH, so its default applies
    Unset {
        /// A dotted path: `build.base_path`, `meta.site_url`
        path: String,
        /// The project's folder
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum ApiAction {
    /// Keep a service read from SPEC: record it in `apis`, write its
    /// `service.wf` once, and its `generated/` files
    Add {
        /// The OpenAPI spec, JSON or YAML, a file in the project
        spec: String,
        /// The service's name, as the program calls it
        #[arg(long, default_value = "Backend")]
        name: String,
        /// The folder its files go in (default: `src/api/<name>`)
        #[arg(long)]
        out: Option<String>,
        /// The project's folder
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Rewrite each service's `generated/` from its spec
    Sync {
        /// One service (default: every one `apis` keeps)
        name: Option<String>,
        /// Leave an operation out, by its name or `METHOD /path`; kept in the
        /// config, so the next sync and `check` leave it out too
        #[arg(long)]
        skip: Vec<String>,
        /// The project's folder
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Fail when `sync` would change anything: for CI
    Check {
        /// One service (default: every one `apis` keeps)
        name: Option<String>,
        /// The project's folder
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Turn `api NAME from "openapi.json"` into a kept service's files
    Eject {
        /// The service
        name: String,
        /// The folder its files go in (default: `src/api/<name>`)
        #[arg(long)]
        out: Option<String>,
        /// The project's folder
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Every service read from a spec: its spec, its folder, its groups and
    /// their endpoints
    List {
        /// Print JSON, for tools
        #[arg(long)]
        json: bool,
        /// The project's folder
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

/// Runs the `wf` command: parses the arguments, runs the command, and
/// exits with its status.
pub fn main() {
    let cli = Cli::parse();

    // `wf build` and `wf serve` print a build's findings as they find them.
    let reports_itself = matches!(
        cli.command,
        Commands::Build { .. } | Commands::Check { .. } | Commands::Serve { .. }
    );
    let result = match cli.command {
        Commands::Init {
            name,
            template,
            locales,
        } => super::init::run_init(&name, &template, locales.as_deref()),
        Commands::Build {
            dir,
            stats,
            format,
            deny_warnings,
            cache,
            studio,
            allow_output_outside,
        } => super::build::run_build_opts(
            &dir,
            super::build::Options {
                stats,
                format,
                deny_warnings,
                dev: false,
                cache,
                studio,
                allow_output_outside,
            },
        ),
        Commands::Check {
            dir,
            format,
            deny_warnings,
            allow_output_outside,
        } => super::build::run_check(
            &dir,
            super::build::Options {
                stats: false,
                format,
                deny_warnings,
                dev: false,
                cache: false,
                studio: false,
                allow_output_outside,
            },
        ),
        Commands::Config { action } => match action {
            ConfigAction::Schema { json } => super::config::run_schema(json),
            ConfigAction::Get { path, dir } => super::config::run_get(&dir, &path),
            ConfigAction::Set { path, value, dir } => {
                super::config::run_set(&dir, &path, Some(&value))
            }
            ConfigAction::Unset { path, dir } => super::config::run_set(&dir, &path, None),
        },
        Commands::Api { action } => match action {
            ApiAction::Add {
                spec,
                name,
                out,
                dir,
            } => super::api::run_add(&dir, &spec, &name, out.as_deref()),
            ApiAction::Sync { name, skip, dir } => {
                super::api::run_sync(&dir, name.as_deref(), &skip)
            }
            ApiAction::Check { name, dir } => super::api::run_check(&dir, name.as_deref()),
            ApiAction::Eject { name, out, dir } => {
                super::api::run_eject(&dir, &name, out.as_deref())
            }
            ApiAction::List { json, dir } => super::api::run_list(&dir, json),
        },
        Commands::Context {
            dir,
            open,
            guide,
            json,
        } => super::context::run_context(&dir, open.as_deref(), &guide, json),
        Commands::Translations {
            dir,
            missing,
            unused,
            plurals,
            json,
        } => super::translations::run_translations(
            &dir,
            missing.as_deref(),
            unused,
            plurals,
            super::Report::from_json_flag(json),
        ),
        Commands::Scope {
            place,
            dir,
            data,
            json,
        } => super::scope::run_scope(&dir, &place, data.as_deref(), json),
        Commands::Rules { dir, json } => super::rules::run_rules(&dir, json),
        Commands::Motion { dir, json } => super::motion::run_motion(&dir, json),
        Commands::Theme { dir, json } => super::theme::run_theme(&dir, json),
        Commands::Components { dir, json } => super::components::run_components(&dir, json),
        Commands::Assets {
            dir,
            plan,
            unused,
            json,
        } => super::assets::run_assets(
            &dir,
            plan.as_deref(),
            unused,
            super::Report::from_json_flag(json),
        ),
        Commands::Commands { json } => {
            let data = describe_commands();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&data).unwrap_or_default()
                );
            } else {
                print_commands(&data);
            }
            Ok(())
        }
        Commands::Fix { dir, code, dry_run } => super::fix::run_fix(&dir, code.as_deref(), dry_run),
        Commands::Explain { code, json } => super::explain::run_explain(code.as_deref(), json),
        Commands::Serve {
            dir,
            host,
            allow_output_outside,
        } => super::serve::run_serve(&dir, &host, allow_output_outside),
        Commands::Generate { kind, name, dir } => super::generate::run_generate(&kind, &name, &dir),
        Commands::Render {
            template: tpl,
            data,
            format,
            output,
            theme,
            tokens,
            page,
            lang,
        } => super::render::run_render(
            &tpl,
            data.as_deref(),
            &super::render::RenderOptions {
                format: &format,
                output: output.as_deref(),
                theme: theme.as_deref(),
                page: page.as_deref(),
                lang: lang.as_deref(),
                tokens: &tokens,
            },
        ),
        Commands::Migrate {
            path,
            check,
            stdout,
            wfx,
        } => super::migrate::run_migrate(
            &path,
            check,
            stdout,
            if wfx {
                super::Layout::Indented
            } else {
                super::Layout::Braces
            },
        ),
        Commands::Fmt {
            path,
            to,
            check,
            stdout,
        } => super::fmt::run_fmt(&path, to.as_deref(), check, stdout),
        Commands::Test { path, update } => super::test::run_test(&path, update),
        Commands::Docs { dir, out } => super::docs::run_docs(&dir, &out),
        Commands::Audit { path, json } => super::audit::run_audit(&path, json),
        Commands::Verify {
            path,
            json,
            budget,
            returning_visitor,
            allow_output_outside,
            fakes,
        } => super::verify::run_verify(
            &path,
            super::Report::from_json_flag(json),
            budget,
            returning_visitor,
            allow_output_outside,
            fakes.as_deref(),
        ),
        Commands::Start {
            dir,
            port,
            host,
            proxy,
            allow_output_outside,
        } => super::start::run_start(&dir, port, &host, &proxy, allow_output_outside),
        Commands::Inspect {
            target,
            dir,
            proxy,
            width,
            height,
            mobile,
            screenshot,
            full_page,
            wait,
            lines,
            steps,
            fakes,
            json,
            allow_output_outside,
        } => {
            let (width, height) = if mobile { (390, 844) } else { (width, height) };
            let screenshot = screenshot.map(|p| {
                if p.as_os_str() == "-" {
                    super::inspect::default_screenshot(&dir, &target)
                } else {
                    p
                }
            });
            super::inspect::run_inspect(
                &target,
                &dir,
                &proxy,
                &super::inspect::Look {
                    width,
                    height,
                    mobile,
                    screenshot: screenshot.as_deref(),
                    full_page,
                    wait_ms: wait,
                    lines,
                    steps: &steps,
                    fakes: fakes.as_deref(),
                },
                super::Report::from_json_flag(json),
                allow_output_outside,
            )
        }
        Commands::Registry { json } => super::describe::run_registry(json),
        Commands::Types { path, json, holes } => {
            if holes {
                super::describe::run_holes(&path, json)
            } else {
                super::describe::run_types(&path, json)
            }
        }
    };

    if let Err(e) = result {
        match &e {
            error::WebFluentError::Diagnostics(list) if !reports_itself => {
                eprint!(
                    "{}",
                    diagnostics::render::human(
                        list,
                        &|file| std::fs::read_to_string(file).ok(),
                        diagnostics::render::stderr_wants_color(),
                    )
                );
                eprintln!("{}", diagnostics::summary(list));
            }
            error::WebFluentError::Diagnostics(_) => {}
            _ => eprintln!("{e}"),
        }
        std::process::exit(e.exit_code());
    }
}

/// `--format`'s values, said to the command line so they are in its
/// description, and read as a [`diagnostics::format::Format`].
fn format_parser() -> impl clap::builder::TypedValueParser<Value = diagnostics::format::Format> {
    use clap::builder::TypedValueParser;
    clap::builder::PossibleValuesParser::new(["human", "json", "sarif", "github"]).map(|s| {
        s.parse::<diagnostics::format::Format>()
            .expect("one of the four")
    })
}

/// Every command of the command line, from its own definition, with what
/// [`crate::commands::EFFECTS`] says each writes: what
/// `src/tools/commands.json` holds.
fn describe_commands() -> serde_json::Value {
    use clap::CommandFactory;
    fn options(cmd: &clap::Command) -> Vec<serde_json::Value> {
        cmd.get_arguments()
            .filter(|a| !matches!(a.get_id().as_str(), "help" | "version"))
            .map(|a| {
                // What a control for it takes: read from the type the
                // command line parses the value into. Compared as types:
                // clap prints a type's name only in a debug build, and a
                // release `wf` said every path was text.
                use std::any::TypeId;

                let flag = matches!(a.get_action(), clap::ArgAction::SetTrue | clap::ArgAction::SetFalse);
                let values: Vec<String> = a.get_possible_values().iter().map(|v| v.get_name().to_string()).collect();
                let defaults: Vec<String> = a.get_default_values().iter().map(|v| v.to_string_lossy().to_string()).collect();
                let parsed = a.get_value_parser().type_id();
                let numbers = [
                    TypeId::of::<u8>(),
                    TypeId::of::<u16>(),
                    TypeId::of::<u32>(),
                    TypeId::of::<u64>(),
                    TypeId::of::<usize>(),
                    TypeId::of::<i32>(),
                    TypeId::of::<i64>(),
                    TypeId::of::<f64>(),
                ];
                let takes = if flag {
                    "nothing"
                } else if !values.is_empty() {
                    "word"
                } else if numbers.iter().any(|n| parsed == *n) {
                    "number"
                } else if parsed == TypeId::of::<PathBuf>() {
                    "path"
                } else {
                    "text"
                };
                serde_json::json!({
                    "name": a.get_id().as_str(),
                    "takes": takes,
                    "long": a.get_long(),
                    "short": a.get_short().map(|c| c.to_string()),
                    "positional": a.is_positional(),
                    "flag": flag && values.iter().all(|v| v == "true" || v == "false"),
                    "repeats": matches!(a.get_action(), clap::ArgAction::Append),
                    "required": a.is_required_set(),
                    "value_name": a.get_value_names().and_then(|n| n.first()).map(std::string::ToString::to_string),
                    "default": defaults.first(),
                    "values": if flag { Vec::new() } else { values },
                    "help": a.get_help().map(std::string::ToString::to_string).unwrap_or_default(),
                })
            })
            .collect()
    }
    fn one(cmd: &clap::Command, path: &str) -> serde_json::Value {
        let effect = crate::commands::effect(path);
        let mut out = serde_json::json!({
            "name": cmd.get_name(),
            "summary": cmd.get_about().map(std::string::ToString::to_string).unwrap_or_default(),
            "writes": effect.map_or("", |e| e.writes),
            "preview": effect.and_then(|e| e.preview),
            "options": options(cmd),
        });
        let subs: Vec<serde_json::Value> = cmd
            .get_subcommands()
            .filter(|s| s.get_name() != "help")
            .map(|s| one(s, &format!("{path} {}", s.get_name())))
            .collect();
        if !subs.is_empty() {
            out["subcommands"] = serde_json::Value::Array(subs);
        }
        out
    }
    let cli = Cli::command();
    serde_json::json!({
        "commands": cli
            .get_subcommands()
            .filter(|s| s.get_name() != "help")
            .map(|s| one(s, s.get_name()))
            .collect::<Vec<_>>(),
    })
}

/// `wf commands`: each command, what it writes, and its options.
fn print_commands(data: &serde_json::Value) {
    fn show(c: &serde_json::Value, prefix: &str) {
        let name = format!("{prefix}{}", c["name"].as_str().unwrap_or(""));
        if let Some(subs) = c.get("subcommands").and_then(|s| s.as_array()) {
            for s in subs {
                show(s, &format!("{name} "));
            }
            return;
        }
        println!("\nwf {name} — {}", c["summary"].as_str().unwrap_or(""));
        if let Some(w) = c["writes"].as_str().filter(|w| !w.is_empty()) {
            let preview = c["preview"]
                .as_str()
                .map(|p| format!("; `{p}` shows it first"))
                .unwrap_or_default();
            println!("  writes {w}{preview}");
        }
        for o in c["options"].as_array().into_iter().flatten() {
            let spelled = match (o["long"].as_str(), o["positional"].as_bool()) {
                (Some(long), _) => format!("--{long}"),
                (None, Some(true)) => format!("<{}>", o["name"].as_str().unwrap_or("")),
                _ => o["name"].as_str().unwrap_or("").to_string(),
            };
            let values = o["values"]
                .as_array()
                .filter(|v| !v.is_empty())
                .map(|v| {
                    format!(
                        " ({})",
                        v.iter()
                            .filter_map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .join("|")
                    )
                })
                .unwrap_or_default();
            println!("  {spelled}{values}  {}", o["help"].as_str().unwrap_or(""));
        }
    }
    for c in data["commands"].as_array().into_iter().flatten() {
        show(c, "");
    }
}

#[cfg(test)]
mod commands_json {
    /// `src/tools/commands.json` is the command line's own description; it is
    /// rewritten here when it is stale, and the test fails once so the
    /// change is seen and committed.
    #[test]
    fn the_commands_json_is_current() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/tools/commands.json");
        let want = format!(
            "{}\n",
            serde_json::to_string_pretty(&super::describe_commands()).unwrap()
        );
        let have = std::fs::read_to_string(path).unwrap_or_default();
        if have != want {
            std::fs::write(path, &want).expect("write commands.json");
            panic!("src/tools/commands.json was stale; it has been rewritten");
        }
    }
}
