## Migrating

`wf migrate` rewrites a project written for an older major version.

**WebFluent 4 → 5** needs no rewrite: 5.0 refuses what used to compile and
then fail in the browser. Run `wf check` (it writes nothing); every finding
has a code, a place and what to do, `wf explain CODE` prints its entry, and
the editor's quick fixes apply the ones with a known fix. While a project
catches up, `lints` lowers what ships no broken page and `// wf-allow(CODE)`
accepts one finding on one line.

**WebFluent 6 → 7** is strict, with no loose mode: what 6.x accepted and
7.0 refuses is an error with its fix, and `wf migrate` applies those fixes
until none is left — the request forms (`fetch` → `http.get`, `ready(v)` →
`.loaded(v)`, `-> T` → `-> T!`, `:id` → `{id}`, `headers { }` →
`middleware [header(…)]`), what each `state` holds, each action returns and
fails with, each parameter takes, a condition's comparison, a splice or a
unit, `try` for a result's value, an error enum's `http` case — then lists
what a person decides. `Any` and `Map` are no longer types (`T23`, `T24`;
the step writes `Json`). `lints` and `// wf-allow` lower only the
errors that ship no broken page (`R01`, `C02`, `S04`, `I01`, `I02`,
`I04`); `T20` is not one.

`wf migrate [path] [--check] [--stdout] [--wfx]` does five things.

**WebFluent 2 → 3** rewrites every `.wf` under `src/` in place, with a
note for anything that needed a decision. It is a change of spelling and
nothing else: the migrated project builds to what it built before. A
WebFluent 2 file given to `wf build` is refused with a pointer to it.

**WebFluent 3 → 4** is not a change of spelling but of what the compiler
allows, so it runs over the project rather than the files. It adds every
`env` name the pages already read to `public_env` — preserving what the
project did, and printing the list, because "public" means anyone who
opens the site may read the value. It then names, with the file and the
line, everything 4 refuses that 3 allowed: an `on*` attribute, a
`javascript:` or `data:` URL. Those have no automatic rewrite. Finally it
states the changes that need no edit and are worth knowing: a store is
built on first read rather than at boot, a `persist` value follows the
site's other tabs, and `WF.store`/`WF.host` were renamed.

**WebFluent 4 → 4.2** runs first, since 4.2 refuses `external`: a remote
module becomes a `meta.scripts` entry (`module: true, as:`, its integrity
moved to `meta.integrity`; its calls keep working, typed `Json`), a local one
a plain script under `src/` with its `export`s off and its declared
signatures as JSDoc (`X.f(…)` becomes `f(…)`), and an `external element`
call `Element("tag", …)`. It lists every `.js` already under `src/`, which
now ships on every page.

**WebFluent 6 → 7** runs over the project, as said above; `--check`
says what it would fix and writes nothing.

**WebFluent 7 → 8** runs last: every deprecated built-in (`V13`) whose
rewrite is exact is rewritten — `Dialog` → `Modal`, `Dropdown(label: x)` →
`Menu(trigger: x)`, `Tag` → `Chip`, `Code(…).block` → `CodeBlock(…)`,
`CircularProgress(…)` → `Progress(…).circular`, a `Toast` at the top of a
page → a `toast(…)` call — and the rest are listed. They keep working
through 8.x and are removed in 9.0.

