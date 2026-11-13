# Build times

Measured 2026-11-12 on master (`c29ae804`), to compare against after the
modules move into their layers and the largest files are split.

Machine: 16 cores, 30 GB, rustc 1.99.0, `CARGO_BUILD_JOBS=6`, the normal
dev profile (incremental on), a fresh target directory.

| What | Time |
|---|---|
| Clean `cargo build --workspace` | 55 s |
| Rebuild after touching `src/codegen/js.rs` | 36 s |
| Rebuild with every test binary (`--tests`) after touching `js.rs` | 69 s |

The slowest units of the clean build (`cargo build --timings`):

| Unit | Time |
|---|---|
| `webfluent` (the library) | 19.8 s |
| `webfluent` (the `wf` binary) | 17.1 s |
| `write-fonts` | 7.2 s |
| `lsp-types` | 6.3 s |
| `read-fonts` | 6.2 s |
| `libwebp-sys` build script | 5.6 s |
| `moxcms` | 4.9 s |
| `tokio` | 4.2 s |

A one-line change anywhere in `src/` recompiles all of `webfluent` and
then links the binary and every test binary, which is why the rebuild is
two thirds of the clean build. To measure again:

```bash
CARGO_TARGET_DIR=/tmp/wf-timing cargo build --workspace --timings
touch src/codegen/js.rs && CARGO_TARGET_DIR=/tmp/wf-timing cargo build --workspace --timings
```
