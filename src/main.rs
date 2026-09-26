#![allow(dead_code)]

mod browser;
mod cli;
mod codegen;
mod config;
mod data;
mod edit;
mod error;
mod fmt;
mod i18n;
mod layout;
mod lexer;
mod linter;
mod media;
mod migrate;
mod openapi;
mod parser;
mod registry;
mod runtime;
mod sema;
mod syntax;
mod template;
mod themes;

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

#[derive(Subcommand)]
enum Commands {
    /// Create a new WebFluent project
    Init {
        /// Project name
        name: String,
        /// Template: "spa" (default), "static", "pdf", or "slides"
        #[arg(short, long, default_value = "spa")]
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
        kind: String,
        /// Name of the item to generate
        name: String,
        /// Project directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Render a .wf template with JSON data
    Render {
        /// Template file (.wf)
        template: PathBuf,
        /// JSON data file (reads stdin if omitted)
        #[arg(long)]
        data: Option<PathBuf>,
        /// Output format: "html" (default), "html-fragment", "pdf", or "slides"
        #[arg(short, long, default_value = "html")]
        format: String,
        /// Output file (stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Which `Theme` declared in the template to render with. Only needed
        /// when the template declares more than one.
        #[arg(long)]
        theme: Option<String>,
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
        #[arg(long)]
        to: Option<String>,
        /// Write nothing; fail when a file is not formatted
        #[arg(long)]
        check: bool,
        /// Print the formatted text to stdout instead of writing it
        #[arg(long)]
        stdout: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name, template } => cli::init::run_init(&name, &template),
        Commands::Build { dir, stats } => cli::build::run_build_with(&dir, stats),
        Commands::Serve { dir } => cli::serve::run_serve(&dir),
        Commands::Generate { kind, name, dir } => cli::generate::run_generate(&kind, &name, &dir),
        Commands::Render {
            template: tpl,
            data,
            format,
            output,
            theme,
        } => cli::render::run_render(
            &tpl,
            data.as_deref(),
            &format,
            output.as_deref(),
            theme.as_deref(),
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
        Commands::Verify { path, json, budget } => cli::verify::run_verify(&path, json, budget),
        Commands::Registry { json } => cli::describe::run_registry(json),
        Commands::Types { path, json } => cli::describe::run_types(&path, json),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
