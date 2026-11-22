#!/usr/bin/env bash
# The runtime benchmark: js-framework-benchmark, keyed, for WebFluent and the
# frameworks it is compared with, in a headless Chrome. Builds WebFluent's entry
# (setup.sh), starts the benchmark's server, runs it, and copies the results to
# bench/runtime/results/, which bench/report.mjs reads; the checkout in .upstream/
# can then be deleted.
#
# Usage: bench/runtime/run.sh [FRAMEWORK…]   (default: the seven on the page)
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
UP="$HERE/.upstream"
CHROME="${CHROME:-/usr/bin/google-chrome}"
FRAMEWORKS=("$@")
[ ${#FRAMEWORKS[@]} -gt 0 ] || FRAMEWORKS=(webfluent vanillajs solid svelte vue react-hooks preact-hooks)

bash "$HERE/setup.sh"
(cd "$UP" && npm ci --silent) > "$HERE/install.log" 2>&1
for f in "${FRAMEWORKS[@]}"; do
  [ "$f" = webfluent ] && continue
  (cd "$UP/frameworks/keyed/$f" && npm ci --silent && npm run build-prod --silent) > "$HERE/build-$f.log" 2>&1
done

(cd "$UP/server" && npm ci --silent && npm start) > "$HERE/server.log" 2>&1 &
SERVER=$!
trap 'kill $SERVER 2>/dev/null || true' EXIT
until curl -s -o /dev/null http://localhost:8080/; do sleep 1; done

cd "$UP/webdriver-ts"
npm ci --silent
args=()
for f in "${FRAMEWORKS[@]}"; do args+=("keyed/$f"); done
rm -rf results
npm run bench -- --headless --chromeBinary "$CHROME" --framework "${args[@]}" > "$HERE/bench.log" 2>&1

mkdir -p "$HERE/results"
for f in "${FRAMEWORKS[@]}"; do rm -f "$HERE/results/$f-"*.json; done
cp results/*.json "$HERE/results/"
echo "wrote $(ls "$HERE/results" | wc -l) results to bench/runtime/results/"
