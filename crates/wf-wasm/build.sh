#!/usr/bin/env bash
# Build the compiler as WebAssembly, twice:
#
#   pkg/       the compiler: sites, apps, every check — what a page loads on
#              its first edit
#   pkg-pdf/   the same with the PDF and slide-deck renderer, and the fonts
#              it is set in under pkg-pdf/fonts/ — loaded only when a project
#              asks for a document (`wf.js` decides)
#
# Needs the `wasm32-unknown-unknown` target, a `wasm-bindgen` CLI of the
# version in Cargo.toml (`cargo install wasm-bindgen-cli --version 0.2.130`),
# and a clang that targets wasm32 with its llvm-ar, for libwebp (WebP
# encoding is C; `libc/` is the C library it is compiled against). Runs
# `wasm-opt -Oz` when one is on the path.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."

export CC_wasm32_unknown_unknown="${CC_wasm32_unknown_unknown:-clang}"
export AR_wasm32_unknown_unknown="${AR_wasm32_unknown_unknown:-llvm-ar}"
export CFLAGS_wasm32_unknown_unknown="-ffreestanding -nostdlibinc -isystem $here/libc/include"

build() { # <out dir> <out name> [cargo args…]
    local out="$1" name="$2"
    shift 2
    cargo build --manifest-path "$root/Cargo.toml" -p wf-wasm \
        --profile wasm-release --target wasm32-unknown-unknown "$@"
    wasm-bindgen --target web --remove-name-section --remove-producers-section \
        --out-dir "$out" --out-name "$name" \
        "$root/target/wasm32-unknown-unknown/wasm-release/wf_wasm.wasm"
    local wasm="$out/${name}_bg.wasm"
    if command -v wasm-opt >/dev/null; then
        wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
            --enable-sign-ext --enable-mutable-globals --enable-reference-types \
            "$wasm" -o "$wasm.opt" && mv "$wasm.opt" "$wasm"
    fi
    printf '%-28s %9d bytes, %9d gzipped\n' "$(basename "$out")/${name}_bg.wasm" \
        "$(wc -c <"$wasm")" "$(gzip -9c "$wasm" | wc -c)"
}

build "$here/pkg" wf_wasm
build "$here/pkg-pdf" wf_wasm_pdf --features pdf

# The faces a document is set in by default, zlib-compressed as `wf` keeps
# them; `register_font` takes them as they are.
mkdir -p "$here/pkg-pdf/fonts"
cp "$root"/src/paged/fonts/*.ttf.z "$root/src/paged/fonts/LICENSE" "$here/pkg-pdf/fonts/"
printf '%-28s %9d bytes\n' "pkg-pdf/fonts/ (12 faces)" \
    "$(cat "$here"/pkg-pdf/fonts/*.ttf.z | wc -c)"

# A face for Arabic, which the Liberation faces do not have and a browser
# gives no machine fonts to fall back on: `wf.js` fetches it only for a
# document whose text has Arabic in it. SIL Open Font License 1.1.
gzip -9 -n -c "$root/examples/documents/fonts/NotoSansArabic.ttf" \
    >"$here/pkg-pdf/fonts/NotoSansArabic.ttf.gz"
cat >"$here/pkg-pdf/fonts/LICENSE-NotoSansArabic.md" <<'LICENSE'
Noto Sans Arabic: Copyright 2022 The Noto Project Authors
(https://github.com/notofonts/arabic), under the SIL Open Font License 1.1
(https://openfontlicense.org).
LICENSE
printf '%-28s %9d bytes\n' "pkg-pdf/fonts/NotoSansArabic" \
    "$(wc -c <"$here/pkg-pdf/fonts/NotoSansArabic.ttf.gz")"
