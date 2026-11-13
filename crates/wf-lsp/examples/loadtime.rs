//! How long the language server takes to load a project and to check it,
//! four times over, so the second and later rounds show what its caches
//! save: `cargo run --release -p wf-lsp --example loadtime -- FILE.wf`.

use std::time::Instant;
use wf_lsp::diagnostics::project_diagnostics;
use wf_lsp::project::{FileCache, Project};
fn main() {
    let path = std::env::args().nth(1).expect("a .wf file of a project");
    let path = std::fs::canonicalize(path).expect("a file that exists");
    let uri = lsp_types::Url::from_file_path(path).expect("an absolute path");
    let cache = FileCache::default();
    for round in 0..4 {
        let t = Instant::now();
        let p = Project::load(&uri, &|_| None, &cache);
        let load = t.elapsed();
        let t = Instant::now();
        let d = project_diagnostics(&p);
        let diag = t.elapsed();
        println!(
            "round {round}: load {:>7.1} ms  diagnostics {:>7.1} ms  ({} files, {} findings)",
            load.as_secs_f64() * 1e3,
            diag.as_secs_f64() * 1e3,
            p.files.len(),
            d.iter().map(Vec::len).sum::<usize>()
        );
    }
}
