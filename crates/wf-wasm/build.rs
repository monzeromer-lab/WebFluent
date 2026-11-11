//! On `wasm32`, the libc libwebp is compiled against (`libc/`): its
//! headers are handed to every C build by `build.sh`, and `shim.c` is
//! compiled here. A native build compiles nothing.

fn main() {
    println!("cargo:rerun-if-changed=libc");
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return;
    }
    cc::Build::new()
        .file("libc/shim.c")
        .include("libc/include")
        .flag("-ffreestanding")
        .flag("-nostdlibinc")
        .warnings(false)
        .compile("wfwasmlibc");
}
