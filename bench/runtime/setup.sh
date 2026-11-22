#!/usr/bin/env bash
# Puts the WebFluent entry (bench/runtime/webfluent) into js-framework-benchmark as
# frameworks/keyed/webfluent, built with the wf given (default: this checkout's release
# build). The benchmark's shared stylesheet is copied into the entry's public/, since a
# WebFluent build addresses its files under its base path.
#
# Usage: bench/runtime/setup.sh [WF]
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
UP="$HERE/.upstream"
WF="${1:-$HERE/../../target/release/wf}"
[ -d "$UP" ] || git clone -q https://github.com/krausest/js-framework-benchmark "$UP"
VERSION="$("$WF" --version | sed 's/^wf //')"
DEST="$UP/frameworks/keyed/webfluent"
rm -rf "$DEST" && mkdir -p "$DEST"
cp -r "$HERE/webfluent/src" "$HERE/webfluent/webfluent.app.json" "$DEST/"
mkdir -p "$DEST/public" && cp -r "$UP/css" "$DEST/public/css"
cat > "$DEST/package.json" <<JSON
{
  "name": "js-framework-benchmark-webfluent",
  "version": "1.0.0",
  "description": "WebFluent, keyed",
  "js-framework-benchmark": {
    "frameworkVersion": "$VERSION",
    "frameworkHomeURL": "https://webfluent.monzeromer.dev",
    "customURL": "/build",
    "language": "WebFluent"
  },
  "scripts": { "dev": "exit 0", "build-prod": "wf build" },
  "license": "MPL-2.0"
}
JSON
# The benchmark lists a framework only with a lock file; this one has no npm dependencies.
printf '{\n  "name": "js-framework-benchmark-webfluent",\n  "version": "1.0.0",\n  "lockfileVersion": 3,\n  "requires": true,\n  "packages": { "": { "name": "js-framework-benchmark-webfluent", "version": "1.0.0", "license": "MPL-2.0" } }\n}\n' > "$DEST/package-lock.json"
(cd "$DEST" && "$WF" build)
echo "WebFluent $VERSION -> $DEST/build"
