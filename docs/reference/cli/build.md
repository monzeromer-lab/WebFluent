# wf build

> Build the project (compile .wf files to HTML + CSS + JS)

The page: https://webfluent.monzeromer.dev/reference/cli/build

Build the project (compile .wf files to HTML + CSS + JS)

```bash
wf build [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--stats` | Print what the build weighs: every file, gzipped, and the runtime modules it kept and left out |
| `--format FORMAT` | How findings are written: `human` (default), `json`, `sarif` (code scanning) or `github` (annotations on a pull request) |
| `--deny-warnings` | Let a warning stop the build as an error does |
| `--cache` | Reuse what earlier builds rendered, kept in `.wf-cache/build/`: only the pages an edit changed are rendered again |
| `--studio` | For an editor's preview: stamp every element with its node id (`data-wf-node`) and write `nodes.json`, from each id to its source |
| `--allow-output-outside` | Let `build.output` name a directory outside the project; without it a build writes only inside the project |

**Writes:** the site, in the output folder
