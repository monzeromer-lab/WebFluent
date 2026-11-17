# wf test

> Run the project's `test "…" { … }` declarations under tests/

The page: https://webfluent.monzeromer.dev/reference/cli/test

Run the project's `test "…" { … }` declarations under tests/

```bash
wf test [PATH] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[PATH]` | Project directory (default: current directory), or one test file |
| `--update` | Rewrite every snapshot with the current render |

**Writes:** the snapshots, with `--update`

Runs the project's `test "…" { }` declarations.

```bash
wf test
wf test tests/cart.wf
wf test --update
```

`wf test` runs every test under `tests/` (and in `src/`), or the tests in
one file. A test that clicks or types runs in headless Chrome, on a page
that links the project's scripts.

See also: [Testing](../learn/16-testing.md).
