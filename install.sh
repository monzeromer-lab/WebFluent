#!/bin/bash
set -e

REPO="monzeromer-lab/WebFluent"
INSTALL_DIR="${WF_INSTALL_DIR:-$HOME/.webfluent/bin}"

OS="$(uname -s)"
case "$OS" in
    Linux*)  TARGET="x86_64-unknown-linux-gnu"; EXT="tar.gz";;
    *)       echo "Unsupported OS: $OS (use install.ps1 for Windows)"; exit 1;;
esac

VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
if [ -z "$VERSION" ]; then
    echo "Could not determine latest version"
    exit 1
fi

URL="https://github.com/$REPO/releases/download/$VERSION/wf-$VERSION-$TARGET.$EXT"

echo "Installing wf $VERSION..."
mkdir -p "$INSTALL_DIR"

curl -sL "$URL" | tar xz -C "$INSTALL_DIR"
chmod +x "$INSTALL_DIR/wf"

echo "Installed wf to $INSTALL_DIR/wf"

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    SHELL_NAME=$(basename "$SHELL")
    case "$SHELL_NAME" in
        zsh)  RC="$HOME/.zshrc";;
        bash) RC="$HOME/.bashrc";;
        *)    RC="$HOME/.profile";;
    esac
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$RC"
    echo "Added $INSTALL_DIR to PATH in $RC"
    echo "Run: source $RC"
fi

echo "Done! Run 'wf --help' to get started."
