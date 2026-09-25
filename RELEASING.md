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

If the grammar changed since the last release, point the Zed extension at
it before tagging: commit the grammar, then `just zed-pin-grammar` and
commit that. The preflight refuses a tag whose pinned grammar is stale.
