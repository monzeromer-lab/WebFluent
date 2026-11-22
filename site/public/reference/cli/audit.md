# wf audit

> What this project trusts: markup it did not write, other origins, what it keeps on the reader's machine, its `env` names, its policy and its dependencies

The page: https://webfluent.monzeromer.dev/reference/cli/audit

What this project trusts: markup it did not write, other origins, what it keeps on the reader's machine, its `env` names, its policy and its dependencies

```bash
wf audit [PATH] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[PATH]` | Project directory (default: current directory) |
| `--json` | Print JSON, for tools |

Prints what a security review asks about the project. It reports and never fails.

```bash
wf audit
wf audit --json
```

The report lists:

- every `Unsafe.*` and whether its input is sanitised;
- the project's own scripts and what each declares;
- every other origin the pages load from;
- everything kept on the reader's machine;
- every `env` name and whether it is public;
- the policy the build ships;
- what the compiler depends on.

See also: [Review a site's security](https://webfluent.monzeromer.dev/recipes/review-security.md) and [Security](https://webfluent.monzeromer.dev/about/security.md).
