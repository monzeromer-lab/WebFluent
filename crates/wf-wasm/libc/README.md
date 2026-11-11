# A libc for libwebp on `wasm32-unknown-unknown`

`wasm32-unknown-unknown` has no C library, and libwebp (the lossy WebP
encoder the image pipeline uses) is C. These headers declare the little of
libc it calls; `shim.c` defines what is not better done in Rust, and
`src/libc.rs` the rest — the allocator (Rust's), the maths (the `libm`
crate). `build.sh` points `cc` at them with `CFLAGS_wasm32_unknown_unknown`.

What libwebp asks of `stdio` it asks to report errors in debug builds: here
it writes nothing.
