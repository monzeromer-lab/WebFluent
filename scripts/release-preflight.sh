#!/usr/bin/env bash
# What a release tag must agree with before anything is built for it.
#
#   scripts/release-preflight.sh v4.0.2
#
# The release workflow runs this first and builds nothing when it fails; run
# it locally before tagging. Each check is one a release of 4.0 got wrong by
# hand: a version left behind, notes not written, the Zed extension pinned
# to a grammar two releases old.
set -euo pipefail

tag="${1:?usage: release-preflight.sh vX.Y.Z}"
version="${tag#v}"
cd "$(git rev-parse --show-toplevel)"
failed=0
fail() { echo "::error::$*"; failed=1; }
ok() { echo "  ok  $*"; }

# 1. The tag is the version both crates carry.
for manifest in Cargo.toml crates/wf-lsp/Cargo.toml; do
    have=$(sed -n 's/^version = "\(.*\)"/\1/p' "$manifest" | head -n1)
    if [ "$have" = "$version" ]; then
        ok "$manifest is $version"
    else
        fail "$manifest is $have, but the tag is $tag. Bump it with \`just bump $version\`."
    fi
done

# 1b. The Node binding is published at the compiler's version.
have=$(sed -n 's/^  "version": "\(.*\)",/\1/p' bindings/node/package.json | head -n1)
if [ "$have" = "$version" ]; then
    ok "bindings/node/package.json is $version"
else
    fail "bindings/node/package.json is $have, but the tag is $tag. Bump it with \`just bump $version\`."
fi

# 2. The release notes say what this release is. 4.0.0 was written
#    `# WebFluent v4.0 Release Notes`, so a .0 release may drop its patch.
short="${version%.0}"
if grep -qE "^# WebFluent v(${version//./\\.}|${short//./\\.}) Release Notes" RELEASE_NOTES.md; then
    ok "RELEASE_NOTES.md has a section for $version"
else
    fail "RELEASE_NOTES.md has no \`# WebFluent v$version Release Notes\` section."
fi

# 3. The Zed extension builds the grammar at the commit it pins, so that
#    commit's grammar has to be this one's.
rev=$(sed -n 's/^rev = "\([0-9a-f]*\)"/\1/p' editors/zed/extension.toml | sort -u)
if [ "$(echo "$rev" | wc -l)" -ne 1 ] || [ -z "$rev" ]; then
    fail "editors/zed/extension.toml pins more than one grammar commit: $rev"
elif ! git cat-file -e "$rev^{commit}" 2>/dev/null; then
    fail "editors/zed/extension.toml pins $rev, which this checkout does not have."
elif git diff --quiet "$rev" HEAD -- \
        editors/tree-sitter-webfluent/grammar.js editors/tree-sitter-webfluent/src \
        editors/tree-sitter-webfluentx/grammar.js editors/tree-sitter-webfluentx/src; then
    ok "the Zed extension pins the current grammar (${rev:0:7})"
else
    fail "the grammar changed after ${rev:0:7}, the commit the Zed extension pins. Run \`just zed-pin-grammar\` and commit it."
fi

exit "$failed"
