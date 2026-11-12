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
# The tag names files and directories in every later step; only vX.Y.Z.
if ! printf '%s' "$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "::error::the tag must be vX.Y.Z, not $tag"
    exit 1
fi
version="${tag#v}"
cd "$(git rev-parse --show-toplevel)"
failed=0
fail() { echo "::error::$*"; failed=1; }
ok() { echo "  ok  $*"; }

# 1. The tag is the version the workspace carries, and every crate takes it.
have=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
if [ "$have" = "$version" ]; then
    ok "the workspace is $version"
else
    fail "the workspace is $have, but the tag is $tag. Bump it with \`just bump $version\`."
fi
for manifest in Cargo.toml crates/wf-lsp/Cargo.toml crates/wf-wasm/Cargo.toml; do
    if grep -q '^version.workspace = true' "$manifest"; then
        ok "$manifest takes the workspace's version"
    else
        fail "$manifest sets a version of its own; give it \`version.workspace = true\`."
    fi
done

# 1a. The agents' skill says which compiler it was written against, and
#     carries that compiler's reference.
skill=skills/writing-webfluent/SKILL.md
have=$(sed -n 's/^  version: "\(.*\)"/\1/p' "$skill" | head -n1)
if [ "$have" = "$version" ] && grep -q "the reference of WebFluent $version" "$skill"; then
    ok "$skill is $version"
else
    fail "$skill is ${have:-unversioned}, but the tag is $tag. Bump it with \`just bump $version\`."
fi
if cmp -s AGENTS.md skills/writing-webfluent/references/language.md; then
    ok "the skill's references/language.md is this AGENTS.md"
else
    fail "skills/writing-webfluent/references/language.md is not AGENTS.md. \`just bump\` copies it."
fi

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
#    has to be the grammar this release links (the tree-sitter-webfluent
#    submodule). Both are submodules; `just editors-init` checks them out.
grammar=$(git ls-tree HEAD editors/tree-sitter-webfluent | awk '{print $3}')
if [ ! -f editors/zed/extension.toml ]; then
    fail "editors/zed is not checked out. Run \`just editors-init\`."
else
    rev=$(sed -n 's/^rev = "\([0-9a-f]*\)"/\1/p' editors/zed/extension.toml | sort -u)
    if [ "$(echo "$rev" | wc -l)" -ne 1 ] || [ -z "$rev" ]; then
        fail "editors/zed/extension.toml pins more than one grammar commit: $rev"
    elif [ "$rev" = "$grammar" ]; then
        ok "the Zed extension pins the linked grammar (${rev:0:7})"
    else
        fail "the Zed extension pins grammar ${rev:0:7}, but this repository links ${grammar:0:7}. Run \`just zed-pin-grammar\`, commit and push it in editors/zed, then commit the bumped submodule here."
    fi
fi

# 4. The docs site states the release it documents: the header's pill.
pill=$(sed -n 's/^const VERSION = "\(.*\)"/\1/p' site/src/model.wf)
if [ "$pill" = "${version%.*}" ]; then
    ok "the docs site's header says $pill"
else
    fail "site/src/model.wf says VERSION = \"$pill\", but this is ${version%.*}. \`just ship\` sets it."
fi

exit "$failed"
