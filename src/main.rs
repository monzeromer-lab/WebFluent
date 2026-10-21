#![allow(dead_code)]

mod assets;
mod audit;
mod browser;
mod build;
mod cli;
mod codegen;
mod config;
mod data;
mod diagnostics;
mod edit;
mod error;
mod fmt;
mod gallery;
mod generate;
mod i18n;
mod init;
mod layout;
mod lexer;
mod linter;
mod media;
mod migrate;
mod motion;
mod openapi;
mod paged;
mod parser;
mod print;
mod project_js;
mod registry;
mod reprint;
mod runtime;
mod scope;
mod sema;
mod style_value;
mod syntax;
mod template;
mod testing;
mod themes;
mod validation;
mod verify;
mod vfs;

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
#[allow(clippy::enum_variant_names)]
#[derive(Subcommand)]
enum Commands {
    /// Create a new WebFluent project
    Init {
        /// Project name
        name: String,
        /// Template: "spa" (default), "static", "pdf", or "slides"
        #[arg(short, long, default_value = "spa", value_parser = ["spa", "static", "pdf", "slides"])]
        template: String,
    },
    /// Build the project (compile .wf files to HTML + CSS + JS)
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
    },
    /// Read and change `webfluent.app.json`: every setting, what the
    /// project sets, and an edit that keeps the file's shape
    Config {
        #[command(subcommand)]
        action: ConfigAction,
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
    },
    /// Start the development server
    Serve {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Generate a new page, component, or store
    Generate {
        /// What to generate: page, component, or store
        #[arg(value_parser = ["page", "component", "store"])]
        kind: String,
        /// Name of the item to generate
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
        /// The document's language, `<html lang>` (default `en`)
        #[arg(long)]
        lang: Option<String>,
    },
    /// Carry a project forward: WebFluent 2 sources to the current grammar, then a 3 project to what 4 allows
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
    /// Write a component gallery: every built-in, and what the project declares
    Docs {
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Where to write index.html (default: docs/)
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
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Set PATH to VALUE — JSON, or a word as a string — keeping the
    /// file's shape
    Set {
        path: String,
        value: String,
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Remove PATH, so its default applies
    Unset {
        path: String,
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    // `wf build` and `wf serve` print a build's findings as they find them.
    let reports_itself = matches!(
        cli.command,
        Commands::Build { .. } | Commands::Check { .. } | Commands::Serve { .. }
    );
    let result = match cli.command {
        Commands::Init { name, template } => cli::init::run_init(&name, &template),
        Commands::Build {
            dir,
            stats,
            format,
            deny_warnings,
            cache,
            studio,
        } => cli::build::run_build_opts(
            &dir,
            cli::build::Options {
                stats,
                format,
                deny_warnings,
                dev: false,
                cache,
                studio,
            },
        ),
        Commands::Check {
            dir,
            format,
            deny_warnings,
        } => cli::build::run_check(
            &dir,
            cli::build::Options {
                stats: false,
                format,
                deny_warnings,
                dev: false,
                cache: false,
                studio: false,
            },
        ),
        Commands::Config { action } => match action {
            ConfigAction::Schema { json } => cli::config::run_schema(json),
            ConfigAction::Get { path, dir } => cli::config::run_get(&dir, &path),
            ConfigAction::Set { path, value, dir } => {
                cli::config::run_set(&dir, &path, Some(&value))
            }
            ConfigAction::Unset { path, dir } => cli::config::run_set(&dir, &path, None),
        },
        Commands::Translations {
            dir,
            missing,
            unused,
            plurals,
            json,
        } => cli::translations::run_translations(&dir, missing.as_deref(), unused, plurals, json),
        Commands::Scope {
            place,
            dir,
            data,
            json,
        } => cli::scope::run_scope(&dir, &place, data.as_deref(), json),
        Commands::Rules { dir, json } => cli::rules::run_rules(&dir, json),
        Commands::Motion { dir, json } => cli::motion::run_motion(&dir, json),
        Commands::Theme { dir, json } => cli::theme::run_theme(&dir, json),
        Commands::Components { dir, json } => cli::components::run_components(&dir, json),
        Commands::Assets {
            dir,
            plan,
            unused,
            json,
        } => cli::assets::run_assets(&dir, plan.as_deref(), unused, json),
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
        Commands::Fix { dir, code, dry_run } => cli::fix::run_fix(&dir, code.as_deref(), dry_run),
        Commands::Explain { code } => cli::explain::run_explain(code.as_deref()),
        Commands::Serve { dir } => cli::serve::run_serve(&dir),
        Commands::Generate { kind, name, dir } => cli::generate::run_generate(&kind, &name, &dir),
        Commands::Render {
            template: tpl,
            data,
            format,
            output,
            theme,
            tokens,
            page,
            lang,
        } => cli::render::run_render(
            &tpl,
            data.as_deref(),
            &cli::render::RenderOptions {
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
        } => cli::migrate::run_migrate(&path, check, stdout, wfx),
        Commands::Fmt {
            path,
            to,
            check,
            stdout,
        } => cli::fmt::run_fmt(&path, to.as_deref(), check, stdout),
        Commands::Test { path, update } => cli::test::run_test(&path, update),
        Commands::Docs { dir, out } => cli::docs::run_docs(&dir, &out),
        Commands::Audit { path, json } => cli::audit::run_audit(&path, json),
        Commands::Verify {
            path,
            json,
            budget,
            returning_visitor,
        } => cli::verify::run_verify(&path, json, budget, returning_visitor),
        Commands::Registry { json } => cli::describe::run_registry(json),
        Commands::Types { path, json } => cli::describe::run_types(&path, json),
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
            _ => eprintln!("{}", e),
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
/// [`webfluent::commands::EFFECTS`] says each writes: what
/// `src/commands.json` holds.
fn describe_commands() -> serde_json::Value {
    use clap::CommandFactory;
    fn options(cmd: &clap::Command) -> Vec<serde_json::Value> {
        cmd.get_arguments()
            .filter(|a| !matches!(a.get_id().as_str(), "help" | "version"))
            .map(|a| {
                let flag = matches!(a.get_action(), clap::ArgAction::SetTrue | clap::ArgAction::SetFalse);
                let values: Vec<String> = a.get_possible_values().iter().map(|v| v.get_name().to_string()).collect();
                let defaults: Vec<String> = a.get_default_values().iter().map(|v| v.to_string_lossy().to_string()).collect();
                // What a control for it takes: read from the type the
                // command line parses the value into.
                // clap names the type in how it shows the id.
                let parsed = format!("{:?}", a.get_value_parser().type_id());
                let takes = if flag {
                    "nothing"
                } else if !values.is_empty() {
                    "word"
                } else if ["u8", "u16", "u32", "u64", "usize", "i32", "i64", "f64"].contains(&parsed.as_str()) {
                    "number"
                } else if parsed.ends_with("PathBuf") {
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
                    "value_name": a.get_value_names().and_then(|n| n.first()).map(|n| n.to_string()),
                    "default": defaults.first(),
                    "values": if flag { Vec::new() } else { values },
                    "help": a.get_help().map(|h| h.to_string()).unwrap_or_default(),
                })
            })
            .collect()
    }
    fn one(cmd: &clap::Command, path: &str) -> serde_json::Value {
        let effect = webfluent::commands::effect(path);
        let mut out = serde_json::json!({
            "name": cmd.get_name(),
            "summary": cmd.get_about().map(|h| h.to_string()).unwrap_or_default(),
            "writes": effect.map(|e| e.writes).unwrap_or(""),
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
    /// `src/commands.json` is the command line's own description; it is
    /// rewritten here when it is stale, and the test fails once so the
    /// change is seen and committed.
    #[test]
    fn the_commands_json_is_current() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/commands.json");
        let want = format!(
            "{}\n",
            serde_json::to_string_pretty(&super::describe_commands()).unwrap()
        );
        let have = std::fs::read_to_string(path).unwrap_or_default();
        if have != want {
            std::fs::write(path, &want).expect("write commands.json");
            panic!("src/commands.json was stale; it has been rewritten");
        }
    }
}
