# wf explain

> What a diagnostic code means, a program that draws it, and the fix; with no code, every code

The page: https://webfluent.monzeromer.dev/reference/cli/explain

What a diagnostic code means, a program that draws it, and the fix; with no code, every code

```bash
wf explain [CODE] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[CODE]` | The code: `T05`, `A01`, `E101` |
| `--json` | Print JSON, for tools: one code, or every code without one |

Prints a diagnostic code's entry from [Diagnostics](../errors/diagnostics.md) in
the terminal.

```bash
wf explain T05
wf explain
```

An entry says what the code means, shows a program that draws it, what the
compiler prints, the fix, and the link to the guide.
