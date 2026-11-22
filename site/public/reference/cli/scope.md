# wf scope

> What a path picker lists at a place: the names in scope there, each with its type and where it comes from, and the fields to go on into

The page: https://webfluent.monzeromer.dev/reference/cli/scope

What a path picker lists at a place: the names in scope there, each with its type and where it comes from, and the fields to go on into

```bash
wf scope PLACE [options]
```

## Options

| Option | Meaning |
|---|---|
| `PLACE` | The place: `src/pages/Signup.wf:24` or `…:24:9` |
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--data DATA` | A template's data, a JSON file, to list after the scope |
| `--json` | Print JSON, for tools |

Lists the names that can be read at one place in a source file — what an editor's path picker and completion offer there.

```bash
wf scope src/pages/Deploys.wf:24
wf scope src/pages/Deploys.wf:24:9 --json
wf scope src/templates/invoice.wf:3 --data data/invoice.json
```

The names come in the order a path picker lists them:

1. the page's or component's own `state` and `derived` values, and the form it binds;
2. its props, and a page's route parameters;
3. what the statements around the place bind — a loop's item and index, an `if let`, a `match` arm, a slot's fill, an action's parameters and the `let`s before the place — innermost first;
4. the members of every store it `use`s;
5. with `--data`, the template's data.

Each row has its type as the checker infers it, its shape (`bool`, `number`, `text`, `moment`, `list`, `record`, `enum` …), whether it may be `null`, where it comes from (`for d in live`, `member of Deploys`), and a record's fields to go on into (`d.region.code`). The library has it as `webfluent::scope` (with the `internals` feature, outside semver).
