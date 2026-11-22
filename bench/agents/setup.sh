#!/usr/bin/env bash
# Fetches Nolan Lawson's agent-framework-benchmark at the commit upstream.json pins,
# into bench/agents/.upstream (not committed), and applies patches/ on top: WebFluent as
# a framework, subscription-only credentials, podman or docker.
#
# Also fetches his published results, for comparing against (bench/agents/.lawson-results).
#
# Usage: bench/agents/setup.sh            (again at any time: it starts over from the pin)
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
UP="$HERE/.upstream"
RESULTS="$HERE/.lawson-results"
REPO="$(jq -r .repo "$HERE/upstream.json")"
COMMIT="$(jq -r .commit "$HERE/upstream.json")"
RESULTS_REPO="$(jq -r .resultsRepo "$HERE/upstream.json")"
RESULTS_COMMIT="$(jq -r .resultsCommit "$HERE/upstream.json")"

if [ ! -d "$UP/.git" ]; then
  git clone -q "$REPO" "$UP"
fi
git -C "$UP" fetch -q origin
git -C "$UP" checkout -q --detach "$COMMIT"
git -C "$UP" reset -q --hard "$COMMIT"
git -C "$UP" clean -qfdx -e runs/
for p in "$HERE"/patches/*.patch; do
  git -C "$UP" -c user.name=bench -c user.email=bench@localhost am -q "$p"
done
echo "Harness: $REPO @ ${COMMIT:0:7} + $(ls "$HERE"/patches/*.patch | wc -l) patch(es) -> $UP"

if [ ! -d "$RESULTS/.git" ]; then
  git clone -q "$RESULTS_REPO" "$RESULTS"
fi
git -C "$RESULTS" fetch -q origin
git -C "$RESULTS" checkout -q --detach "$RESULTS_COMMIT"
echo "Published results: $RESULTS_REPO @ ${RESULTS_COMMIT:0:7} -> $RESULTS"
