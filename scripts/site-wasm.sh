#!/usr/bin/env bash
# Ship the compiler with the docs site: `wf.js`, the preview worker and both
# WebAssembly builds (with the PDF build's fonts) copied into the site's
# output under /wasm/<version>/, where the live examples and the Playground
# load them on the reader's first edit — never on page load. The version is
# in the address, so a host may cache every file forever; `_headers` says so
# for the hosts that read it.
#
# Then the size budget: what a reader downloads the first time they edit,
# and the first time they ask for a PDF. Over budget fails.
#
#   crates/wf-wasm/build.sh          # pkg/ and pkg-pdf/ first
#   scripts/site-wasm.sh [OUT]       # OUT: the site's output, ../docs from site/
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/docs}"
here="$root/crates/wf-wasm"

stop() { echo "site-wasm: $*" >&2; exit 1; }

version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -n1)
[ -n "$version" ] || stop "no version in Cargo.toml"
grep -q "^const WF_RELEASE = \"$version\"$" "$root/site/src/model.wf" \
    || stop "site/src/model.wf's WF_RELEASE is not $version, so the pages would ask for a compiler that is not there"
for f in pkg/wf_wasm.js pkg/wf_wasm_bg.wasm pkg-pdf/wf_wasm_pdf.js pkg-pdf/wf_wasm_pdf_bg.wasm; do
    [ -f "$here/$f" ] || stop "crates/wf-wasm/$f is missing: run crates/wf-wasm/build.sh"
done

# One release at a time: the pages of this build ask for this one.
rm -rf "$out/wasm"
dest="$out/wasm/$version"
mkdir -p "$dest/pkg" "$dest/pkg-pdf/fonts"
cp "$here/wf.js" "$here/preview-sw.js" "$dest/"
cp "$here/pkg/wf_wasm.js" "$here/pkg/wf_wasm_bg.wasm" "$dest/pkg/"
cp "$here/pkg-pdf/wf_wasm_pdf.js" "$here/pkg-pdf/wf_wasm_pdf_bg.wasm" "$dest/pkg-pdf/"
cp "$here"/pkg-pdf/fonts/* "$dest/pkg-pdf/fonts/"

# Cached forever: a new release is a new address.
if [ -f "$out/_headers" ] && ! grep -q '^/wasm/\*$' "$out/_headers"; then
    printf '\n/wasm/*\n  Cache-Control: public, max-age=31536000, immutable\n' >>"$out/_headers"
fi

gz() { gzip -9c "$@" | wc -c; }
kb() { awk -v b="$1" 'BEGIN { printf "%.0f kB", b / 1000 }'; }
failed=0
budget() { # <label> <bytes> <limit>
    local mark="ok"
    if [ "$2" -gt "$3" ]; then mark="OVER"; failed=1; fi
    printf '  %-4s %-42s %9s gzipped (budget %s)\n' "$mark" "$1" "$(kb "$2")" "$(kb "$3")"
}

echo "The compiler, shipped under /wasm/$version/"
glue=$(cat "$dest/wf.js" "$dest/pkg/wf_wasm.js" | gzip -9c | wc -c)
budget "wf.js + bindings (the first edit's script)" "$glue" 12000
budget "pkg/wf_wasm_bg.wasm (the first edit)" "$(gz "$dest/pkg/wf_wasm_bg.wasm")" 1750000
budget "pkg-pdf/wf_wasm_pdf_bg.wasm (the first PDF)" "$(gz "$dest/pkg-pdf/wf_wasm_pdf_bg.wasm")" 2950000
fonts=$(cat "$dest"/pkg-pdf/fonts/*.ttf.z | wc -c)
budget "pkg-pdf/fonts/*.ttf.z (with the first PDF)" "$fonts" 2500000
[ "$failed" = 0 ] || stop "over the size budget"
