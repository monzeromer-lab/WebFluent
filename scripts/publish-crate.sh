#!/usr/bin/env bash
# Publish `webfluent` to crates.io, for the release workflow.
#
#   scripts/publish-crate.sh 4.0.2
#
# Safe to run twice: a version already on crates.io is left alone. crates.io
# answered the 4.0.0 upload with a 408 three times, and a 408 does not say
# whether the upload landed, so every failure asks crates.io before trying
# again. Without `CARGO_REGISTRY_TOKEN` it warns and stops, so a release still
# completes before the secret exists. `WF_PUBLISH_DRY_RUN=1` goes through the
# motions with `cargo publish --dry-run`.
set -euo pipefail

version="${1:?usage: publish-crate.sh X.Y.Z}"
version="${version#v}"
cd "$(git rev-parse --show-toplevel)"

published() {
    curl -fs -o /dev/null \
        -A "webfluent-release (https://github.com/monzeromer-lab/WebFluent)" \
        "https://crates.io/api/v1/crates/webfluent/$version"
}

if [ -z "${CARGO_REGISTRY_TOKEN:-}" ] && [ -z "${WF_PUBLISH_DRY_RUN:-}" ]; then
    echo "::warning::CARGO_REGISTRY_TOKEN is not set, so webfluent $version was not published to crates.io. Add a crates.io token as that repository secret, then re-run this job."
    exit 0
fi

if published; then
    echo "webfluent $version is already on crates.io; nothing to do."
    exit 0
fi

args=(publish -p webfluent --locked)
[ -n "${WF_PUBLISH_DRY_RUN:-}" ] && args+=(--dry-run --allow-dirty)

for attempt in 1 2 3; do
    echo "cargo ${args[*]} (attempt $attempt)"
    if cargo "${args[@]}"; then
        echo "webfluent $version: done."
        exit 0
    fi
    [ -n "${WF_PUBLISH_DRY_RUN:-}" ] && { echo "::error::the dry run failed"; exit 1; }
    sleep 20
    if published; then
        echo "The upload reported a failure, but webfluent $version is on crates.io."
        exit 0
    fi
done
echo "::error::webfluent $version could not be published to crates.io after three attempts."
exit 1
