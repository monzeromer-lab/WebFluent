#!/usr/bin/env bash
# The public API of the `webfluent` crate, held to tests/public-api.txt; see
# `cargo xtask api`, which this runs.
#
# Usage: scripts/public-api.sh            check
#        scripts/public-api.sh --update   write tests/public-api.txt again
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo xtask api "$@"
