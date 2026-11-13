use tower_lsp::{LspService, Server};
use wf_lsp::backend::Backend;

/// mimalloc, rather than the system's allocator, which a check that
/// allocates on several threads at once waits on.
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    // Asked what it is, it says so and exits; anything else an editor passes
    // (`--stdio`, which is how it is run anyway) starts the server.
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("wf-lsp {VERSION}");
                return;
            }
            "--help" | "-h" => {
                println!(
                    "wf-lsp {VERSION}\n\
                     The WebFluent language server. An editor runs it and speaks the\n\
                     Language Server Protocol over standard input and output.\n\n\
                     Usage: wf-lsp [--stdio]\n\n\
                     Options:\n  \
                     -V, --version  Print the version and exit\n  \
                     -h, --help     Print this and exit"
                );
                return;
            }
            _ => {}
        }
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);

    // Room for a burst of edits to wait out their pause side by side, with
    // requests still answered beside them.
    Server::new(stdin, stdout, socket)
        .concurrency_level(32)
        .serve(service)
        .await;
}
