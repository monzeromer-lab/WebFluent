# wf verify

> Load every page of a built project in a real browser: errors, failed requests, paint timing, and which built-ins it drew

The page: https://webfluent.monzeromer.dev/reference/cli/verify

Load every page of a built project in a real browser: errors, failed requests, paint timing, and which built-ins it drew

```bash
wf verify [PATH] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[PATH]` | Project directory (default: current directory) |
| `--json` | Print JSON, for a pipeline |
| `--budget MS` | Fail a route whose first paint is slower than this, in milliseconds |
| `--returning-visitor` | Visit every route again as a reader whose storage holds what the previous build's pages kept |
| `--allow-output-outside` | Let `build.output` name a directory outside the project |
