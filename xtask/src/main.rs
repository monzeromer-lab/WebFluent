//! The repository's own automation: `cargo xtask <command>`.
//!
//! ```text
//! cargo xtask check [--fix] [--test]   what CI's main job runs, one step at a time
//! cargo xtask api [--update]           the public API, held to tests/public-api.txt
//! cargo xtask golden [--update]        every project's output, held to tests/golden/
//! cargo xtask fuzz <target> [seconds]  one fuzz target (parse, fmt, layout, markdown, sanitize)
//! ```
//!
//! Each command runs `cargo` the way CI does, so a tree that passes here
//! passes there. It uses nothing but the standard library.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Instant;

/// The nightly the public API listing is written with: rustdoc's output
/// differs between nightlies, so CI pins the same one.
const API_TOOLCHAIN: &str = "nightly-2026-09-30";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = args.split_first() else {
        print_help();
        return ExitCode::from(2);
    };
    let root = repository_root();
    let result = match command.as_str() {
        "check" => check(&root, rest),
        "api" => api(&root, rest),
        "golden" => golden(&root, rest),
        "fuzz" => fuzz(&root, rest),
        "-h" | "--help" | "help" => {
            print_help();
            Ok(())
        }
        other => Err(format!(
            "unknown command `{other}` (try `cargo xtask help`)"
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("\n{message}");
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    let doc = include_str!("main.rs");
    for line in doc.lines().skip(3).take(4) {
        println!("{}", line.trim_start_matches("//! "));
    }
}

/// The workspace root: the directory above this crate's.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Run `program args` in `dir`, with `env` set, naming the step; an error
/// says which step failed.
fn run(
    dir: &Path,
    name: &str,
    program: &str,
    args: &[&str],
    env: &[(&str, &str)],
) -> Result<(), String> {
    println!("\n── {name}: {program} {}", args.join(" "));
    let started = Instant::now();
    let status = Command::new(program)
        .args(args)
        .envs(env.iter().copied())
        .current_dir(dir)
        .status()
        .map_err(|e| format!("could not run `{program}`: {e}"))?;
    if status.success() {
        println!("   ok ({}s)", started.elapsed().as_secs());
        Ok(())
    } else {
        Err(format!("FAILED: {name}"))
    }
}

/// Whether `cargo <subcommand>` is installed.
fn has_cargo_subcommand(subcommand: &str) -> bool {
    Command::new("cargo")
        .args([subcommand, "--version"])
        .output()
        .is_ok_and(|o| o.status.success())
}

/// What CI's "Build, lint and test" job runs first: formatting, clippy,
/// the build and the docs, and with `--test` the tests, each only when
/// the one before it passed, cheapest first.
fn check(root: &Path, args: &[String]) -> Result<(), String> {
    let (mut fix, mut test) = (false, false);
    for arg in args {
        match arg.as_str() {
            "--fix" => fix = true,
            "--test" => test = true,
            other => return Err(format!("unknown option `{other}` (try --fix or --test)")),
        }
    }
    if fix {
        run(root, "Format", "cargo", &["fmt", "--all"], &[])?;
    }
    run(
        root,
        "Formatting",
        "cargo",
        &["fmt", "--all", "--", "--check"],
        &[],
    )
    .map_err(|e| format!("{e}\n  `cargo xtask check --fix` formats the tree"))?;
    run(
        root,
        "Clippy",
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &[],
    )?;
    run(root, "Build", "cargo", &["build", "--workspace"], &[])?;
    run(
        root,
        "Docs",
        "cargo",
        &["doc", "--workspace", "--no-deps", "--quiet"],
        &[("RUSTDOCFLAGS", "-D warnings")],
    )?;
    if test {
        // nextest, where it is installed, runs every test side by side, as
        // CI does.
        if has_cargo_subcommand("nextest") {
            run(
                root,
                "Tests",
                "cargo",
                &["nextest", "run", "--workspace"],
                &[],
            )?;
            run(
                root,
                "Documentation examples",
                "cargo",
                &["test", "--workspace", "--doc"],
                &[],
            )?;
        } else {
            run(root, "Tests", "cargo", &["test", "--workspace"], &[])?;
        }
    }
    println!("\nAll passed.");
    Ok(())
}

/// The public API of the `webfluent` crate, held to `tests/public-api.txt`:
/// an item added, removed or changed fails, so it is seen in review; with
/// `--update`, the file is written again. Needs cargo-public-api and the
/// pinned nightly (`WF_API_TOOLCHAIN` names another).
fn api(root: &Path, args: &[String]) -> Result<(), String> {
    let update = match args {
        [] => false,
        [flag] if flag == "--update" => true,
        _ => return Err("usage: cargo xtask api [--update]".to_string()),
    };
    let toolchain = std::env::var("WF_API_TOOLCHAIN").unwrap_or_else(|_| API_TOOLCHAIN.to_string());
    let output = Command::new("cargo")
        .args([
            &format!("+{toolchain}"),
            "public-api",
            "-p",
            "webfluent",
            "--simplified",
        ])
        .current_dir(root)
        .output()
        .map_err(|e| format!("could not run cargo public-api: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo public-api failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let now = String::from_utf8_lossy(&output.stdout).into_owned();
    let listed = root.join("tests/public-api.txt");
    if update {
        std::fs::write(&listed, &now).map_err(|e| format!("{}: {e}", listed.display()))?;
        println!("wrote tests/public-api.txt ({} items)", now.lines().count());
        return Ok(());
    }
    let was = std::fs::read_to_string(&listed).map_err(|e| format!("{}: {e}", listed.display()))?;
    if was == now {
        println!(
            "the public API is as tests/public-api.txt lists it ({} items)",
            now.lines().count()
        );
        return Ok(());
    }
    let was_lines: std::collections::BTreeSet<&str> = was.lines().collect();
    let now_lines: std::collections::BTreeSet<&str> = now.lines().collect();
    for gone in was_lines.difference(&now_lines) {
        println!("-{gone}");
    }
    for added in now_lines.difference(&was_lines) {
        println!("+{added}");
    }
    Err("the public API changed (above). If that is meant, run `cargo xtask api --update` and commit tests/public-api.txt with the change.".to_string())
}

/// Every project in the repository built, and what it wrote and what the
/// compiler said held to `tests/golden/`; with `--update`, written again.
fn golden(root: &Path, args: &[String]) -> Result<(), String> {
    let env: &[(&str, &str)] = match args {
        [] => &[],
        [flag] if flag == "--update" => &[("UPDATE_GOLDEN", "1")],
        _ => return Err("usage: cargo xtask golden [--update]".to_string()),
    };
    run(
        root,
        "Golden manifests",
        "cargo",
        &["test", "--release", "--test", "it", "--", "golden::"],
        env,
    )
}

/// One fuzz target for `seconds` (60 unless given). Needs cargo-fuzz and the
/// pinned nightly; a crash lands in `fuzz/artifacts/<target>/`.
fn fuzz(root: &Path, args: &[String]) -> Result<(), String> {
    let (target, seconds) = match args {
        [target] => (target.as_str(), "60"),
        [target, seconds] => (target.as_str(), seconds.as_str()),
        _ => return Err("usage: cargo xtask fuzz <target> [seconds]".to_string()),
    };
    let toolchain = format!("+{API_TOOLCHAIN}");
    let limit = format!("-max_total_time={seconds}");
    run(
        &root.join("fuzz"),
        &format!("Fuzz {target}"),
        "cargo",
        &[&toolchain, "fuzz", "run", target, "--", &limit],
        &[],
    )
}
