#!/bin/bash
# Install the WebFluent compiler, `wf`, from the latest GitHub release.
#
#   curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
#
# Set WF_INSTALL_DIR to choose where the binary lands, WF_VERSION to pin a
# release (`v3.2.1`). Prefer `cargo install webfluent` if you have Rust — it
# covers every platform, including the ones without a prebuilt binary here.
set -euo pipefail

REPO="monzeromer-lab/WebFluent"
INSTALL_DIR="${WF_INSTALL_DIR:-$HOME/.webfluent/bin}"

die() { echo "error: $*" >&2; exit 1; }

command -v curl >/dev/null 2>&1 || die "curl is required"
command -v tar  >/dev/null 2>&1 || die "tar is required"

# The release publishes `<os>` as `linux`/`macos` and `<arch>` as
# `x86_64`/`aarch64`; the asset is `wf-<version>-<arch>-<os>.tar.gz`. These
# used to be spelled as Rust target triples, which named no asset that exists,
# so every run of this script downloaded a 404 page and fed it to tar.
case "$(uname -s)" in
    Linux)  OS="linux";;
    Darwin) OS="macos";;
    *)      die "unsupported OS: $(uname -s). On Windows use install.ps1; otherwise: cargo install webfluent";;
esac

case "$(uname -m)" in
    x86_64|amd64)  ARCH="x86_64";;
    arm64|aarch64) ARCH="aarch64";;
    *)             die "unsupported architecture: $(uname -m). Try: cargo install webfluent";;
esac

# Only macOS ships an arm64 build; Linux arm64 has to come from source.
if [ "$OS" = "linux" ] && [ "$ARCH" = "aarch64" ]; then
    die "no prebuilt binary for linux/arm64 yet. Install with Rust instead: cargo install webfluent"
fi

if [ -n "${WF_VERSION:-}" ]; then
    VERSION="$WF_VERSION"
else
    VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name"' | head -n1 | cut -d'"' -f4)
fi
[ -n "$VERSION" ] || die "could not determine the latest version (GitHub API rate limit?). Set WF_VERSION=v3.2.1 to pin one."

ASSET="wf-$VERSION-$ARCH-$OS.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"

echo "Installing wf $VERSION ($ARCH-$OS)…"

# Unpack through a temporary directory so a failed download cannot leave a
# half-written binary where a working one used to be. `-f` makes curl fail on
# a 404 rather than hand the error page to tar.
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$URL" -o "$TMP/wf.tar.gz" \
    || die "could not download $ASSET. Check https://github.com/$REPO/releases for what that release carries."
tar xzf "$TMP/wf.tar.gz" -C "$TMP"
[ -f "$TMP/wf" ] || die "the archive did not contain a wf binary"

mkdir -p "$INSTALL_DIR"
mv "$TMP/wf" "$INSTALL_DIR/wf"
chmod +x "$INSTALL_DIR/wf"

echo "Installed $("$INSTALL_DIR/wf" --version 2>/dev/null || echo wf) to $INSTALL_DIR/wf"

# Put it on PATH, once. The guard reads the startup file rather than $PATH:
# $PATH in this shell does not change when the line is appended, so running
# the installer twice used to add the line twice.
case "$(basename "${SHELL:-sh}")" in
    zsh)  RC="$HOME/.zshrc";;
    bash) RC="$HOME/.bashrc";;
    fish) RC="$HOME/.config/fish/config.fish";;
    *)    RC="$HOME/.profile";;
esac

if command -v wf >/dev/null 2>&1 && [ "$(command -v wf)" = "$INSTALL_DIR/wf" ]; then
    echo "Run 'wf --help' to get started."
elif [ -f "$RC" ] && grep -qF "$INSTALL_DIR" "$RC"; then
    echo "$INSTALL_DIR is already added in $RC — open a new terminal, then run 'wf --help'."
else
    mkdir -p "$(dirname "$RC")"
    if [ "$(basename "$RC")" = "config.fish" ]; then
        echo "fish_add_path $INSTALL_DIR" >> "$RC"
    else
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$RC"
    fi
    echo "Added $INSTALL_DIR to PATH in $RC"
    echo "Run: source $RC   (then 'wf --help')"
fi
