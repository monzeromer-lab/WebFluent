# wf migrate

> Carry a project forward: WebFluent 2 → 3, 3 → 4, 4 → 4.2 and 6 → 7, each step that applies

The page: https://webfluent.monzeromer.dev/reference/cli/migrate

Carry a project forward: WebFluent 2 → 3, 3 → 4, 4 → 4.2 and 6 → 7, each step that applies

```bash
wf migrate [PATH] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[PATH]` | Project directory (default: current directory), or one .wf file |
| `--check` | Report what would change without writing anything |
| `--stdout` | Print the migrated text of one file to stdout instead of writing it |
| `--wfx` | Write the migrated files in the indented layout, as `.wfx` |

**Writes:** the project's source files

**Preview:** `--check` says what it would do without doing it.

Carries a project written for an older WebFluent forward.

```bash
wf migrate --check
wf migrate
wf migrate --wfx
```

`wf migrate` makes four passes, in this order:

1. **4.1 → 4.2**, first, because 4.2 refuses `external`. A remote module
   becomes a `meta.scripts` entry. A local one becomes a plain script under
   `src/`, with its signatures as JSDoc, and `X.f(…)` becomes `f(…)`. An
   `external element` call becomes `Element("tag", …)`. The pass lists
   every `.js` already under `src/`, since each now ships on every page.
2. **2 → 3** rewrites every `.wf` under `src/` in place, with a note for
   anything that needed a decision. It changes spelling only: the migrated
   project builds to what it built before.
3. **3 → 4** changes what the compiler allows, so it runs over the project
   rather than the files. It adds every `env` name your pages already read
   to `public_env`, which preserves what the project did, and prints the
   list, because *public* means anyone who opens the site may read the
   value. It then names, with file and line, everything 4 refuses that 3
   allowed: an `on*` attribute, a `javascript:` or `data:` URL. Those have
   no automatic rewrite, because the old value was a string of JavaScript
   or a link that ran one. Finally it lists the changes that need no edit:
   a store is built on first read rather than at boot, a `persist` value
   follows the site's other tabs, and `WF.store`/`WF.host` were renamed.
4. **6 → 7** applies every fix the findings 7.0 brings carry — the request
   forms 6.x had (`N08`), what each declaration holds, returns and fails
   with (`T25`, `T26`, `N03`, `T32`, `T31`), the comparison a condition
   meant (`T29`), a splice or a unit (`T34`) — until nothing is left to
   fix, then lists every error a person decides, by place, and the changes
   that need no edit. With `--check` it says what it would fix and writes
   nothing.

See also: [Upgrading](../errors/upgrading.md).
