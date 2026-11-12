# Releasing

First pick the version from what changed. CI's **Semver against the last
release** job (or `cargo semver-checks -p webfluent` locally) says whether
the public API changed in a way that needs a minor or a major version, and
`tests/public-api.txt` lists that API item by item; a change to it is in the
release's diff.

Write the version's section at the top of `RELEASE_NOTES.md`, then:

```bash
just ship 5.3.2
```

It does every step below, in order, and asks before it pushes anything:
CI and the checks on the commit being released, the Zed pin, the bump, the
docs site, the preflight, the commit, the push and the tag. By hand:

```bash
just bump 5.3.2            # both crates and the lock file
# write `# WebFluent v5.3.2 Release Notes` at the top of RELEASE_NOTES.md
just preflight v5.3.2      # what the release workflow checks first
git commit -am "Release 5.3.2" && git push
git tag -a v5.3.2 -m "WebFluent 5.3.2" && git push origin v5.3.2
```

The tag does the rest (`.github/workflows/release.yml`): the preflight
checks (the tag must be `vX.Y.Z`), the binaries for every platform, a
`SHA256SUMS` over every asset with a signed build-provenance attestation,
the GitHub release with that version's section of the notes as its body,
`cargo publish` and `npm publish`.

## What a release carries, and how to check it

Every asset is listed in `SHA256SUMS`, which is published with them.
`install.sh`, `install.ps1` and both editor extensions download it and
refuse an archive or a server whose hash differs, or that it does not
list. Every asset is also attested by `actions/attest-build-provenance`, so
anyone can check that a file was built by this repository's release
workflow from the tagged commit:

```bash
gh attestation verify wf-v5.15.0-x86_64-linux.tar.gz --repo monzeromer-lab/WebFluent
sha256sum --check --ignore-missing SHA256SUMS
```

Besides the archives, each platform's language server is published bare
(`wf-lsp-<tag>-<arch>-<os>`, `.exe` on Windows): the Zed extension cannot
unpack an archive and hash it, so it downloads that file and hashes it
itself.

A release made before checksums existed (5.14.0 and older) has no
`SHA256SUMS`, so the installers refuse it; `cargo install webfluent
--version X.Y.Z` still installs one.

## Publishing to crates.io and npm

Both registries are published to with **trusted publishing**: the job
trades its GitHub OIDC token for a short-lived registry token, and no
long-lived secret is needed. Each must be set up once on the registry, by
an owner of the package:

- **crates.io** — <https://crates.io/crates/webfluent/settings> →
  *Trusted Publishing* → *Add* → GitHub: owner `monzeromer-lab`, repository
  `WebFluent`, workflow file `release.yml`, environment empty. The job uses
  `rust-lang/crates-io-auth-action`.
- **npm** — <https://www.npmjs.com/package/webfluent/access> → *Trusted
  Publisher* → GitHub Actions: organization or user `monzeromer-lab`,
  repository `WebFluent`, workflow file `release.yml`, environment empty.
  The package then carries npm provenance. Once it works, *Require
  two-factor authentication and disallow tokens* on the same page closes
  the token path.

Until a registry is set up, the job falls back to a token saved as a
repository secret — `CARGO_REGISTRY_TOKEN` (a crates.io token scoped to
`publish-update` on `webfluent`) and `NPM_TOKEN` (an npm granular token
that can publish `webfluent`). With neither, that job warns and passes; set
one up and re-run the job to publish a release that went out without it.
Once trusted publishing works, delete both secrets and revoke the tokens.

The Node binding (`bindings/node`) goes to npm as `webfluent` at the
compiler's version — `just bump` sets it.

## What the workflows may do

`release.yml` starts from `permissions: {}`; each job names what it needs.
Only `release` can write to the repository (`contents: write`), only
`attest` can sign (`id-token: write`, `attestations: write`), and the two
publish jobs can mint an OIDC token for their registry and nothing else.
`ci.yml` reads (`contents: read`). Every action is pinned to a commit, with
its version in a comment; `.github/dependabot.yml` opens one pull request a
week to move the pins. Review those like code: the diff is the action's.
`cargo-deb` and `cargo-wix` are installed at a pinned version with
`--locked`.

`deny.toml` holds every dependency to RustSec's advisories, a licence
allow-list and crates.io as the only source (CI's `deny` job; locally
`cargo deny check`). The two unmaintained font crates the PDF engine reads
fonts with, `rustybuzz` and `ttf-parser`, are accepted by advisory id with
a reason; replacing them ends those entries.

Worth setting on GitHub, which no file here can: a tag ruleset on `v*`
that only maintainers can create, update or delete (a tag is what starts a
release), and *Require approval for all outside collaborators* for
workflows from forks.

The editors are released from their own repositories, which this one
links as submodules under `editors/`. The one tie a release here checks:
the Zed extension must pin the grammar commit this repository links. The
preflight refuses a tag where they differ. To bring them together, in
`editors/tree-sitter-webfluent` push the grammar change; then
`just zed-pin-grammar`, commit and push that in `editors/zed`, and commit
both bumped submodules here.

## The editor extensions

Neither extension downloads the latest release: each pins the WebFluent
release whose server it runs (`SERVER_VERSION` in webfluent-vscode's
`src/server.ts` and webfluent-zed's `src/webfluent.rs`) and checks the
download against that release's `SHA256SUMS`. After a compiler release,
raise both pins to the new tag and release both extensions; until then
they keep serving the server they pin. A pin must name a release that has
`SHA256SUMS` (5.15.0 or later).

- **VS Code** — [webfluent-vscode](https://github.com/monzeromer-lab/webfluent-vscode). Tag `v<version>` there (its own version, in its
  `package.json`) and its release workflow attaches the `.vsix` and
  publishes it to the marketplaces once their tokens exist, as secrets of
  *that* repository. Each marketplace publishes from a job of its own that
  only downloads the built `.vsix`, installs the locked `vsce`/`ovsx` with
  `--ignore-scripts`, and holds that one token:
  - **VS Code Marketplace**: a publisher named `monzeromer-lab`, and an
    Azure DevOps personal access token with *Marketplace › Manage*, saved
    as `VSCE_PAT`.
  - **Open VSX** (VSCodium, Cursor, Gitpod): the namespace created once with
    `npx ovsx create-namespace monzeromer-lab -p <token>`, and the token
    saved as `OVSX_PAT`.

  A version already published is skipped.
- **Zed** — [webfluent-zed](https://github.com/monzeromer-lab/webfluent-zed) (version in `extension.toml`), listed by a pull request
  to `zed-industries/extensions`: a submodule `extensions/webfluent`
  pointing at that repository, and

  ```toml
  [webfluent]
  submodule = "extensions/webfluent"
  version = "3.2.0"
  ```

  A later version is the same pull request with the submodule moved to the
  new commit and `version` raised.
- **Grammars** — [tree-sitter-webfluent](https://github.com/monzeromer-lab/tree-sitter-webfluent). Zed takes them by commit; nothing else is
  published yet.
