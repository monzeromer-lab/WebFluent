#!/usr/bin/env bash
# Release a version, start to finish: everything RELEASING.md lists, in
# order, stopping at the first thing that is not right.
#
#   scripts/ship.sh 4.2.1          asks before it pushes anything
#   scripts/ship.sh 4.2.1 --yes    does not ask
#
# Before running it, write the release notes: a `# WebFluent v4.2.1 Release
# Notes` section at the top of RELEASE_NOTES.md. That file may be the one
# uncommitted change; anything else must be committed and pushed already,
# because the Zed extension pins the grammar by a commit GitHub has.
#
# What it does: checks the tree, the tag and CI on the commit being
# released; runs scripts/check.sh; re-pins the Zed grammar when it changed;
# bumps both crates and the Node binding; rebuilds the docs site; runs the
# release preflight; then commits, pushes, tags and pushes the tag — which
# is what starts the release workflow. Saying no at the prompt puts every
# file back as it was.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

version=""
yes=0
for arg in "$@"; do
    case "$arg" in
        --yes | -y) yes=1 ;;
        -h | --help)
            sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        -*)
            echo "unknown option: $arg (try --help)" >&2
            exit 2
            ;;
        *) version="$arg" ;;
    esac
done
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "usage: scripts/ship.sh X.Y.Z [--yes]" >&2
    exit 2
fi
tag="v$version"

stop() {
    echo
    echo "STOPPED: $*" >&2
    exit 1
}
step() { echo; echo "── $*"; }

# ── What has to be true before anything changes ──

step "The tree, the branch and the tag"
branch=$(git rev-parse --abbrev-ref HEAD)
upstream=$(git rev-parse --abbrev-ref '@{upstream}' 2>/dev/null) || stop "$branch tracks no remote branch"
git fetch --quiet --tags origin
[ "$(git rev-parse HEAD)" = "$(git rev-parse "$upstream")" ] ||
    stop "$branch is not the same commit as $upstream — push or pull first"
dirty=$(git status --porcelain | grep -v ' RELEASE_NOTES.md$' || true)
[ -z "$dirty" ] || stop "uncommitted changes besides RELEASE_NOTES.md:
$dirty"
git rev-parse -q --verify "refs/tags/$tag" >/dev/null && stop "$tag already exists"
grep -Eq "^# WebFluent v$version Release Notes" RELEASE_NOTES.md ||
    stop "RELEASE_NOTES.md has no \`# WebFluent v$version Release Notes\` section — write it first"
echo "  ok  $branch is $upstream at $(git rev-parse --short HEAD); $tag is free; the notes are written"

step "CI on $(git rev-parse --short HEAD)"
if command -v gh >/dev/null; then
    run=$(gh run list --commit "$(git rev-parse HEAD)" --workflow CI --limit 1 \
        --json databaseId,status,conclusion --jq '.[0] | "\(.databaseId) \(.status) \(.conclusion)"' || true)
    [ -n "$run" ] || stop "CI has not run on this commit"
    read -r id status conclusion <<<"$run"
    if [ "$status" != "completed" ]; then
        echo "  CI is $status; waiting for it"
        gh run watch "$id" --exit-status >/dev/null || stop "CI failed: gh run view $id"
    elif [ "$conclusion" != "success" ]; then
        stop "CI $conclusion on this commit: gh run view $id"
    fi
    echo "  ok  CI passed"
else
    echo "  !!  gh is not installed, so CI was not checked"
fi

step "Formatting, clippy, build"
scripts/check.sh

# ── The release commit: from here on, saying no puts everything back ──

restore() {
    git restore --staged --worktree -- . ':!RELEASE_NOTES.md' 2>/dev/null || true
    git clean -fdq -- docs
}

step "The Zed extension's grammar"
rev=$(sed -n 's/^rev = "\([0-9a-f]*\)"/\1/p' editors/zed/extension.toml | head -n1)
if git diff --quiet "$rev" HEAD -- editors/tree-sitter-webfluent editors/tree-sitter-webfluentx; then
    echo "  ok  the pinned grammar (${rev:0:7}) is current"
else
    just zed-pin-grammar
fi

step "Version $version"
just bump "$version"

step "The docs site"
cargo build --release --quiet
python3 scripts/site-from-guide.py >/dev/null
python3 scripts/site-data.py >/dev/null
target/release/wf build -d site | tail -n1

step "Release preflight"
scripts/release-preflight.sh "$tag" || { restore; stop "the preflight failed; every file is back as it was"; }

step "What will be released"
git status --short
echo
echo "  commit \"Release $version\", push it to $upstream, tag $tag and push the tag."
echo "  The tag starts the release workflow: binaries, the GitHub release, crates.io."
if [ "$yes" != 1 ]; then
    read -r -p "  Go ahead? [y/N] " answer
    if [ "$answer" != "y" ] && [ "$answer" != "Y" ]; then
        restore
        echo "  Nothing was committed; every file is back as it was."
        exit 1
    fi
fi

step "Commit, push, tag"
git add -A -- Cargo.toml Cargo.lock crates/wf-lsp/Cargo.toml bindings/node/package.json \
    RELEASE_NOTES.md editors/zed/extension.toml site docs
git commit --quiet -m "Release $version"
git push --quiet origin "HEAD:${upstream#origin/}"
git tag -a "$tag" -m "WebFluent $version"
git push --quiet origin "$tag"
echo "  ok  $tag is pushed"

if command -v gh >/dev/null; then
    sleep 5
    echo
    gh run list --workflow Release --limit 1 || true
    echo "  Follow it with: gh run watch \$(gh run list --workflow Release --limit 1 --json databaseId --jq '.[0].databaseId')"
fi
