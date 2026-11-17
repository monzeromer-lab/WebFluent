# wf context

> What an agent is given: the files, the open file, the problems, the components, the theme, the config with only its public env, the guide, and what was withheld

The page: https://webfluent.monzeromer.dev/reference/cli/context

What an agent is given: the files, the open file, the problems, the components, the theme, the config with only its public env, the guide, and what was withheld

```bash
wf context [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--open OPEN` | The file open in the editor, and its selection as byte offsets: `src/pages/Home.wf` or `src/pages/Home.wf:120-180` |
| `--guide GUIDE` | The guide as its chapters' sections (`index`) or whole (`full`) (default `index`) |
| `--json` | Print JSON, for tools; otherwise one Markdown document |

Prints what a coding agent working on the project is given, so nothing it is
told is hidden from you.

```bash
wf context
wf context --open src/pages/Home.wf:120-180
wf context --guide full --json
```

It includes:

- every file of the project — not the build's output, `.git/`,
  dependencies or a `.env` file;
- the open file and its selection;
- the problems the build reports;
- every component a call can name, with its props, cases, events and slots;
- the theme's tokens;
- `webfluent.app.json`, with every `env` value that `public_env` does not
  list withheld;
- the guide: each chapter's title, blurb and sections to cite
  (`16 Forms and validation › What a value must be: validate`), or with
  `--guide full` the chapters whole;
- what was withheld, and why.

The library has it as `webfluent::agent::context` (with the `internals` feature, outside semver). An agent's edits go
through `webfluent::agent::changeset::ChangeSet`: a layer over the editor's
buffers that the compiler builds before anyone reads it. It refuses paths
outside the project, in `.git/`, in the build's output and `.env` files;
reads the config with its private values withheld and keeps them through a
write; and is accepted whole, by file or by hunk. `ChangeSet::edit` makes an
agent's anchored edits (`webfluent::agent::edit`): each `old_text` must be
found once in a file the set has read — exactly, then ignoring what ends a
line, then the indent, then typographic quotes and dashes — and a file that
moved under the set since it was read is refused until it is read again.
Text found twice names its lines; text found nowhere names the nearest.
