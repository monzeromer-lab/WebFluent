#!/usr/bin/env bash
# Build the compiler as WebAssembly into `pkg/`, for the playground and the
# demo page beside this script. Needs the `wasm32-unknown-unknown` target and
# a `wasm-bindgen` CLI of the version in Cargo.toml
# (`cargo install wasm-bindgen-cli --version 0.2.130`); runs `wasm-opt -Oz`
# when one is on the path.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
cargo build --manifest-path "$root/Cargo.toml" -p wf-wasm --profile wasm-release --target wasm32-unknown-unknown
wasm-bindgen --target web --remove-name-section --remove-producers-section --out-dir "$here/pkg" \
    "$root/target/wasm32-unknown-unknown/wasm-release/wf_wasm.wasm"
wasm="$here/pkg/wf_wasm_bg.wasm"
if command -v wasm-opt >/dev/null; then
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
        --enable-sign-ext --enable-mutable-globals --enable-reference-types \
        "$wasm" -o "$wasm.opt" && mv "$wasm.opt" "$wasm"
fi
raw=$(wc -c <"$wasm")
gz=$(gzip -9c "$wasm" | wc -c)
echo "wf_wasm_bg.wasm: $raw bytes, $gz gzipped"
