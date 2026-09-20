#![allow(dead_code)]

mod cli;
mod codegen;
mod config;
mod edit;
mod error;
mod layout;
mod lexer;
mod linter;
mod migrate;
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
#[command(about = "WebFluent — Build SPAs with a web-first language", long_about = None)]
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
    /// Rewrite a project's .wf files from the original grammar to WebFluent 3
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
    /// Rewrite a project's source files in the other layout: `.wfx` (blocks
    /// by indentation) or `.wf` (blocks in braces)
    Fmt {
        /// Project directory (default: current directory), or one file
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The layout to write: `wfx` or `wf`
        #[arg(long)]
        to: String,
        /// Print the converted text of one file to stdout instead of writing it
        #[arg(long)]
        stdout: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name, template } => cli::init::run_init(&name, &template),
        Commands::Build { dir } => cli::build::run_build(&dir),
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
        Commands::Fmt { path, to, stdout } => cli::fmt::run_fmt(&path, &to, stdout),
        Commands::Registry { json } => cli::describe::run_registry(json),
        Commands::Types { path, json } => cli::describe::run_types(&path, json),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
