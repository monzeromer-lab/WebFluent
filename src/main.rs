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
        /// Template: "spa" (default), "static", or "pdf"
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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name, template } => cli::init::run_init(&name, &template),
        Commands::Build { dir } => cli::build::run_build(&dir),
        Commands::Serve { dir } => cli::serve::run_serve(&dir),
        Commands::Generate { kind, name, dir } => cli::generate::run_generate(&kind, &name, &dir),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
