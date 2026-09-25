#!/usr/bin/env bash
# This release's section of RELEASE_NOTES.md, without its heading: the body
# of the GitHub release, so the page says what the notes say.
#
#   scripts/release-notes.sh v4.0.2 > notes.md
set -euo pipefail
version="${1:?usage: release-notes.sh vX.Y.Z}"
version="${version#v}"
short="${version%.0}"
cd "$(git rev-parse --show-toplevel)"
awk -v v="$version" -v s="$short" '
    /^# WebFluent v/ {
        if (inside) exit
        if ($0 == "# WebFluent v" v " Release Notes" || $0 == "# WebFluent v" s " Release Notes") { inside = 1; next }
    }
    inside { print }
' RELEASE_NOTES.md | sed -e '/./,$!d' | tac | sed -e '/./,$!d' | tac
