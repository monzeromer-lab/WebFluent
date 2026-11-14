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

The [full entry for wf context](https://webfluent.monzeromer.dev/reference/cli.md#wf-context) has its examples and the details this page leaves out.
