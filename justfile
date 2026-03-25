# WebFluent — build, package, and release tasks

set dotenv-load := false

version := `grep '^version' Cargo.toml | head -1 | cut -d'"' -f2`
target_linux := "x86_64-unknown-linux-gnu"
target_windows := "x86_64-pc-windows-gnu"

# ── Build ────────────────────────────────────────────

# Build debug binary
build:
    cargo build

# Build release binary
release:
    cargo build --release

# Build for a specific target
build-target target:
    cargo build --release --target {{target}}

# Cross-compile for Windows (requires mingw-w64)
build-windows:
    cargo build --release --target {{target_windows}}

# ── Run ──────────────────────────────────────────────

# Run the CLI with arguments
run *args:
    cargo run -- {{args}}

# Build and serve the docs site locally
site:
    cargo build --release
    target/release/wf build -d site
    target/release/wf dev -d site

# Build the docs site for production
site-build:
    cargo build --release
    target/release/wf build -d site

# ── Package ──────────────────────────────────────────

# Build .deb package (installs cargo-deb if needed)
deb:
    @which cargo-deb > /dev/null 2>&1 || cargo install cargo-deb
    cargo deb

# Build .deb without rebuilding (uses existing release binary)
deb-fast:
    @which cargo-deb > /dev/null 2>&1 || cargo install cargo-deb
    cargo deb --no-build

# Build Windows .msi installer (requires wix on Windows or cross-build)
msi:
    @echo "Building Windows installer..."
    just build-windows
    just _package-msi

# Build both .deb and tarball
package-linux: release
    just deb-fast
    just tarball-linux

# Build ALL packages (Linux .deb + tarball, Windows .zip, docs site)
package-all: release
    @echo "══════════════════════════════════════════"
    @echo "  Packaging WebFluent v{{version}}"
    @echo "══════════════════════════════════════════"
    mkdir -p dist
    @echo "\n── Linux .deb ──"
    just deb-fast
    cp target/debian/*.deb dist/ 2>/dev/null || true
    @echo "\n── Linux tarball ──"
    just tarball-linux
    @echo "\n── Windows .exe ──"
    just build-windows
    just zip-windows
    @echo "\n── Docs site ──"
    just site-build
    @echo "\n══════════════════════════════════════════"
    @echo "  Done! Artifacts in dist/"
    @echo "══════════════════════════════════════════"
    @ls -lh dist/

# Create Linux tarball
tarball-linux:
    mkdir -p dist
    tar czf dist/wf-{{version}}-{{target_linux}}.tar.gz -C target/release wf

# Create Windows zip (after cross-compiling)
zip-windows:
    mkdir -p dist
    zip -j dist/wf-{{version}}-{{target_windows}}.zip target/{{target_windows}}/release/wf.exe

# ── Test ─────────────────────────────────────────────

# Run all tests
test:
    cargo test

# Run clippy lints
lint:
    cargo clippy -- -W clippy::all

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# ── Clean ────────────────────────────────────────────

# Clean build artifacts
clean:
    cargo clean
    rm -rf dist/

# ── Release ──────────────────────────────────────────

# Tag and push a release (usage: just tag v0.1.0)
tag version:
    git tag {{version}}
    git push origin {{version}}

# Show current version
version:
    @echo {{version}}

# ── Helpers ──────────────────────────────────────────

# Install dev dependencies
setup:
    cargo install cargo-deb
    rustup target add {{target_windows}}
    @echo "Install mingw-w64 for Windows cross-compilation:"
    @echo "  sudo apt install gcc-mingw-w64-x86-64"

# List available recipes
[private]
_package-msi:
    @echo "MSI packaging requires Windows. Use the GitHub Actions release workflow."
    @echo "Or install 'cargo-wix' and run: cargo wix"
