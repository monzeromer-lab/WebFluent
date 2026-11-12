#!/usr/bin/env bash
# The checks CI's "Build, lint and test" job runs first, one by one; see
# `cargo xtask check`, which this runs.
#
#   scripts/check.sh          formatting, clippy, build, docs
#   scripts/check.sh --fix    format the tree first, then the same checks
#   scripts/check.sh --test   and the test suite after the build
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
exec cargo xtask check "$@"
