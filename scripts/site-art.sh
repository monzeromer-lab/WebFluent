#!/usr/bin/env bash
# The documentation site's icons and link-preview card, from their sources in
# site/art/: written as SVG, exported with Inkscape, packed with ImageMagick.
#
#   scripts/site-art.sh
#
# Writes into site/public/ (copied to the site's root by the build):
#   favicon.svg           the tab icon, its letters turned to outlines so it
#                         does not depend on the reader's fonts
#   apple-touch-icon.png  180px and square-cornered: iOS rounds it itself
#   icon-512.png          a large copy, for anything that asks for one
#   og.png                1200x630, the card a shared link shows
#
# Needs `inkscape` and `magick`, and the fonts the sources name: JetBrains
# Mono and Lato.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
art=site/art
out=site/public
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

svg() { inkscape "$1" --export-text-to-path --export-plain-svg --export-filename="$2" >/dev/null 2>&1; }
png() { inkscape "$1" --export-type=png --export-width="$3" --export-height="$4" --export-filename="$2" >/dev/null 2>&1; }

svg "$art/favicon.src.svg" "$out/favicon.svg"

# The home-screen icon is full-bleed: the same mark without the corner radius.
sed 's/ rx="12"//' "$art/favicon.src.svg" > "$tmp/square.src.svg"
svg "$tmp/square.src.svg" "$tmp/square.svg"
png "$tmp/square.svg" "$tmp/touch.png" 180 180
magick "$tmp/touch.png" -strip -alpha off "$out/apple-touch-icon.png"
png "$out/favicon.svg" "$tmp/icon-512.png" 512 512
magick "$tmp/icon-512.png" -strip "$out/icon-512.png"

png "$art/og.src.svg" "$tmp/og.png" 1200 630
magick "$tmp/og.png" -strip -alpha off -define png:compression-level=9 "$out/og.png"

ls -l "$out"/favicon.svg "$out"/apple-touch-icon.png "$out"/icon-512.png "$out"/og.png
