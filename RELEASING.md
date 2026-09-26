# Releasing

```bash
just bump 4.0.2            # both crates and the lock file
# write `# WebFluent v4.0.2 Release Notes` at the top of RELEASE_NOTES.md
just preflight v4.0.2      # what the release workflow checks first
git commit -am "Release 4.0.2" && git push
git tag -a v4.0.2 -m "WebFluent 4.0.2" && git push origin v4.0.2
```

The tag does the rest (`.github/workflows/release.yml`): the preflight
checks, the binaries for every platform, the GitHub release with that
version's section of the notes as its body, and `cargo publish`.

`cargo publish` needs a crates.io token saved once as the repository secret
`CARGO_REGISTRY_TOKEN`. Until it exists that job warns and passes; add the
secret and re-run the job to publish a release that went out without it.

The Node binding (`bindings/node`) goes to npm as `webfluent`, at the same
version — `just bump` sets it. It needs an npm automation token saved as
`NPM_TOKEN`; without it that job warns and passes, the same way.

If the grammar changed since the last release, point the Zed extension at
it before tagging: commit the grammar, then `just zed-pin-grammar` and
commit that. The preflight refuses a tag whose pinned grammar is stale.

## The editor extensions

Each release attaches the VS Code extension as a `.vsix`, built from the
tagged commit; it installs from a file with *Extensions: Install from
VSIX…*. Its version is its own (`editors/vscode/package.json`): bump it
when the extension changes, and add its `CHANGELOG.md` entry.

The same release publishes it to the marketplaces once their tokens exist:

- **VS Code Marketplace** — a publisher named `monzeromer-lab`, and an Azure
  DevOps personal access token with *Marketplace › Manage*, saved as the
  secret `VSCE_PAT`.
- **Open VSX** (VSCodium, Cursor, Gitpod) — an account, the namespace
  created once with `npx ovsx create-namespace monzeromer-lab -p <token>`,
  and the token saved as `OVSX_PAT`.

A version already published is skipped, so a release that leaves the
extension alone leaves the marketplaces alone.

The Zed extension (`editors/zed`, version in `extension.toml`) is listed by
a pull request to `zed-industries/extensions`: a submodule
`extensions/webfluent` pointing at this repository, and

```toml
[webfluent]
submodule = "extensions/webfluent"
path = "editors/zed"
version = "3.0.0"
```

A later version is the same pull request with the submodule moved to the
new commit and `version` raised.
