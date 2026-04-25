#![allow(dead_code)]

mod cli;
mod lexer;
mod parser;
mod codegen;
mod runtime;
mod themes;
mod config;
mod error;
mod linter;
mod template;

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
        /// Theme name
        #[arg(long, default_value = "default")]
        theme: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name, template } => cli::init::run_init(&name, &template),
        Commands::Build { dir } => cli::build::run_build(&dir),
        Commands::Serve { dir } => cli::serve::run_serve(&dir),
        Commands::Generate { kind, name, dir } => cli::generate::run_generate(&kind, &name, &dir),
        Commands::Render { template: tpl, data, format, output, theme } => {
            cli::render::run_render(&tpl, data.as_deref(), &format, output.as_deref(), &theme)
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
