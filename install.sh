#!/bin/bash
# Install the WebFluent compiler, `wf`, from the latest GitHub release.
#
#   curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
#
# Set WF_INSTALL_DIR to choose where the binary lands, WF_VERSION to pin a
# release (`v5.15.0`). Prefer `cargo install webfluent` if you have Rust — it
# covers every platform, including the ones without a prebuilt binary here.
#
# The archive is checked against the release's SHA256SUMS before anything is
# unpacked, and nothing is installed when the two disagree or the sums cannot
# be read. Every line runs inside `main`, called on the last line, so a
# download cut short runs nothing at all.
set -euo pipefail

REPO="monzeromer-lab/WebFluent"

die() { echo "error: $*" >&2; exit 1; }

# Where the release assets are fetched from: GitHub, over HTTPS only.
# WF_INSTALL_BASE_URL replaces it for testing this script against a local
# copy of a release (`http://127.0.0.1:8000`); plain HTTP is accepted for a
# loopback address and nothing else.
base_url() {
    local version="$1"
    if [ -n "${WF_INSTALL_BASE_URL:-}" ]; then
        echo "${WF_INSTALL_BASE_URL%/}/$version"
    else
        echo "https://github.com/$REPO/releases/download/$version"
    fi
}

curl_proto() {
    case "${WF_INSTALL_BASE_URL:-}" in
        http://127.0.0.1:*|http://127.0.0.1/*|http://localhost:*|http://localhost/*) echo "=https,http";;
        ""|https://*) echo "=https";;
        *) die "WF_INSTALL_BASE_URL must be https://, or http:// on 127.0.0.1 or localhost";;
    esac
}

fetch() {
    curl --proto "$PROTO" --tlsv1.2 -fsSL "$@"
}

# The SHA-256 of a file, by whichever tool this system has.
sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d' ' -f1
    elif command -v openssl >/dev/null 2>&1; then
        openssl dgst -sha256 -r "$1" | cut -d' ' -f1
    else
        die "no SHA-256 tool (sha256sum, shasum or openssl) to check the download with"
    fi
}

# Holds `file` to the line for `name` in a SHA256SUMS file; dies otherwise.
verify() {
    local file="$1" name="$2" sums="$3" want have
    want=$(awk -v n="$name" '$2 == n || $2 == "*" n { print $1; exit }' "$sums")
    [ -n "$want" ] || die "SHA256SUMS lists no $name; refusing to install an archive that cannot be checked"
    printf '%s' "$want" | grep -Eq '^[0-9a-fA-F]{64}$' || die "SHA256SUMS has a malformed entry for $name"
    have=$(sha256_of "$file")
    if [ "$(printf '%s' "$have" | tr 'A-F' 'a-f')" != "$(printf '%s' "$want" | tr 'A-F' 'a-f')" ]; then
        die "$name does not match the release's SHA256SUMS (expected $want, got $have). Nothing was installed."
    fi
}

main() {
    local INSTALL_DIR OS ARCH VERSION ASSET BASE TMP RC
    INSTALL_DIR="${WF_INSTALL_DIR:-$HOME/.webfluent/bin}"
    PROTO=$(curl_proto)

    command -v curl >/dev/null 2>&1 || die "curl is required"
    command -v tar  >/dev/null 2>&1 || die "tar is required"

    # The release publishes `<os>` as `linux`/`macos` and `<arch>` as
    # `x86_64`/`aarch64`; the asset is `wf-<version>-<arch>-<os>.tar.gz`.
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
    elif [ -n "${WF_INSTALL_BASE_URL:-}" ]; then
        die "WF_INSTALL_BASE_URL needs WF_VERSION too"
    else
        VERSION=$(fetch "https://api.github.com/repos/$REPO/releases/latest" \
            | grep '"tag_name"' | head -n1 | cut -d'"' -f4) || true
    fi
    [ -n "$VERSION" ] || die "could not determine the latest version (GitHub API rate limit?). Set WF_VERSION=v5.15.0 to pin one."
    printf '%s' "$VERSION" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$' \
        || die "a version is vX.Y.Z, not $VERSION"

    ASSET="wf-$VERSION-$ARCH-$OS.tar.gz"
    BASE=$(base_url "$VERSION")

    echo "Installing wf $VERSION ($ARCH-$OS)…"

    # Unpack through a temporary directory so a failed download cannot leave
    # a half-written binary where a working one used to be. `-f` makes curl
    # fail on a 404 rather than hand the error page to tar.
    TMP=$(mktemp -d)
    # shellcheck disable=SC2064 # TMP is fixed now; expand it now.
    trap "rm -rf '$TMP'" EXIT

    fetch "$BASE/$ASSET" -o "$TMP/$ASSET" \
        || die "could not download $ASSET. Check https://github.com/$REPO/releases for what that release carries."
    fetch "$BASE/SHA256SUMS" -o "$TMP/SHA256SUMS" \
        || die "could not download SHA256SUMS for $VERSION, so $ASSET cannot be checked. Releases before checksums were published can be installed with: cargo install webfluent --version ${VERSION#v}"
    verify "$TMP/$ASSET" "$ASSET" "$TMP/SHA256SUMS"
    echo "Checked $ASSET against SHA256SUMS."

    mkdir "$TMP/unpacked"
    tar xzf "$TMP/$ASSET" -C "$TMP/unpacked"
    if [ ! -f "$TMP/unpacked/wf" ] || [ -L "$TMP/unpacked/wf" ]; then
        die "the archive did not contain a wf binary"
    fi

    mkdir -p "$INSTALL_DIR"
    mv "$TMP/unpacked/wf" "$INSTALL_DIR/wf"
    chmod +x "$INSTALL_DIR/wf"

    echo "Installed $("$INSTALL_DIR/wf" --version 2>/dev/null || echo wf) to $INSTALL_DIR/wf"

    # Put it on PATH, once. The guard reads the startup file rather than
    # $PATH: $PATH in this shell does not change when the line is appended,
    # so running the installer twice used to add the line twice.
    # WF_NO_MODIFY_PATH=1 leaves the startup files alone.
    if [ -n "${WF_NO_MODIFY_PATH:-}" ]; then
        echo "Add $INSTALL_DIR to PATH to run 'wf'."
        return
    fi
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
}

main "$@"
