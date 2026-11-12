# Releasing

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
checks, the binaries for every platform, the GitHub release with that
version's section of the notes as its body, and `cargo publish`.

`cargo publish` needs a crates.io token saved once as the repository secret
`CARGO_REGISTRY_TOKEN`. Until it exists that job warns and passes; add the
secret and re-run the job to publish a release that went out without it.

The Node binding (`bindings/node`) goes to npm as `webfluent`, at the same
version — `just bump` sets it. It needs an npm automation token saved as
`NPM_TOKEN`; without it that job warns and passes, the same way.

The editors are released from their own repositories, which this one
links as submodules under `editors/`. The one tie a release here checks:
the Zed extension must pin the grammar commit this repository links. The
preflight refuses a tag where they differ. To bring them together, in
`editors/tree-sitter-webfluent` push the grammar change; then
`just zed-pin-grammar`, commit and push that in `editors/zed`, and commit
both bumped submodules here.

## The editor extensions

- **VS Code** — [webfluent-vscode](https://github.com/monzeromer-lab/webfluent-vscode). Tag `v<version>` there (its own version, in its
  `package.json`) and its release workflow attaches the `.vsix` and
  publishes it to the marketplaces once their tokens exist, as secrets of
  *that* repository:
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
