#!/usr/bin/env bash
# The public API of the `webfluent` crate, held to tests/public-api.txt: every
# item a user of the published crate can name. A change to it — an item added,
# removed or changed — fails here, so it is seen in review; when it is meant,
# write the file again and commit it with the change.
#
# Usage: scripts/public-api.sh            check
#        scripts/public-api.sh --update   write tests/public-api.txt again
#
# Needs cargo-public-api and a nightly toolchain (rustdoc's JSON is nightly
# only). WF_API_TOOLCHAIN names the nightly; CI pins the one the file was
# written with, since rustdoc's output can differ between nightlies.
set -euo pipefail
cd "$(dirname "$0")/.."

toolchain="${WF_API_TOOLCHAIN:-nightly}"
now="$(mktemp)"
trap 'rm -f "$now"' EXIT

cargo "+$toolchain" public-api -p webfluent --simplified > "$now"

if [ "${1:-}" = "--update" ]; then
    cp "$now" tests/public-api.txt
    echo "wrote tests/public-api.txt ($(wc -l < tests/public-api.txt) items)"
    exit 0
fi

if ! diff -u tests/public-api.txt "$now"; then
    echo "the public API changed (above). If that is meant, run scripts/public-api.sh --update and commit tests/public-api.txt with the change." >&2
    exit 1
fi
echo "the public API is as tests/public-api.txt lists it ($(wc -l < "$now") items)"
