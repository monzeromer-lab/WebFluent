//! The `wf` command. Everything it does is in the library, behind the
//! `cli` feature.

/// mimalloc, rather than the system's allocator, which a build that
/// allocates on every core waits on.
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    webfluent::cli::main();
}
