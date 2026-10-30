#!/usr/bin/env bash
# Write src/codegen/browser_globals.txt: every name a browser's global
# object has — its own properties and those of each prototype above it —
# read from a headless Chrome. A program's top-level name (a `const`, a
# `data` or `image` constant, a store, a service) that is one of these is
# renamed in the generated code, so it never hides what the runtime, a
# project script or a library reads (`const Map = 3` stopped every page).
#
# Rerun when a browser grows new globals: `scripts/browser-globals.sh`.
set -euo pipefail
cd "$(dirname "$0")/.."
chrome="${WF_CHROME:-$(command -v google-chrome || command -v chromium || command -v chromium-browser)}"
page="$(mktemp --suffix=.html)"
trap 'rm -f "$page"' EXIT
cat > "$page" <<'HTML'
<!doctype html><script>
const names = new Set();
for (let o = window; o; o = Object.getPrototypeOf(o)) for (const n of Object.getOwnPropertyNames(o)) names.add(n);
document.write('<pre id=o>' + [...names].filter((n) => /^[A-Za-z_$][\w$]*$/.test(n)).join('\n') + '</pre>');
</script>
HTML
{
  echo "# Every name a browser's global object has. Written by scripts/browser-globals.sh."
  "$chrome" --headless=new --disable-gpu --no-sandbox --dump-dom "file://$page" 2>/dev/null \
    | sed -n '/<pre id="o">/,/<\/pre>/p' | sed 's/<[^>]*>//g' | grep -v '^$' | LC_ALL=C sort -u
} > src/codegen/browser_globals.txt
echo "$(($(wc -l < src/codegen/browser_globals.txt) - 1)) names"
