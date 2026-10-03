#!/usr/bin/env bash
# The checks CI's "Build, lint and test" job runs first, one by one, with the
# same commands — so a tree that passes here passes there.
#
#   scripts/check.sh          formatting, clippy, build, docs
#   scripts/check.sh --fix    format the tree first, then the same checks
#   scripts/check.sh --test   and the test suite after the build
#
# Each step runs only when the one before it passed, cheapest first:
# formatting is the quickest failure to read, and a build is wasted on code
# clippy would refuse.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

fix=0
test=0
for arg in "$@"; do
    case "$arg" in
        --fix) fix=1 ;;
        --test) test=1 ;;
        -h | --help)
            sed -n '2,11p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "unknown option: $arg (try --help)" >&2
            exit 2
            ;;
    esac
done

passed=()
step() {
    local name="$1"
    shift
    echo
    echo "── $name: $*"
    local start=$SECONDS
    if "$@"; then
        passed+=("$name ($((SECONDS - start))s)")
    else
        echo
        echo "FAILED: $name"
        [ "$name" = "Formatting" ] && echo "  scripts/check.sh --fix formats the tree"
        exit 1
    fi
}

if [ "$fix" = 1 ]; then
    step "Format" cargo fmt --all
fi
step "Formatting" cargo fmt --all -- --check
step "Clippy" cargo clippy --workspace --all-targets -- -D warnings
step "Build" cargo build --workspace
step "Docs" env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --quiet
if [ "$test" = 1 ]; then
    step "Tests" cargo test --workspace
fi

echo
echo "All passed:"
printf '  ok  %s\n' "${passed[@]}"
