# wf rules

> The rules a `validate` block may hold, the named patterns, when a form shows its messages, and every state the project holds to a rule

The page: https://webfluent.monzeromer.dev/reference/cli/rules

The rules a `validate` block may hold, the named patterns, when a form shows its messages, and every state the project holds to a rule

```bash
wf rules [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--json` | Print JSON, for tools |

Lists what a `validate` block can hold, and every state the project holds to
a rule.

```bash
wf rules
wf rules --json
```

It lists:

- every rule: what it takes between its parentheses, when it holds, the
  message it shows when none is written, the translation key
  (`form.minLength`) that replaces that message in a translated project,
  whether an empty value passes it, and whether it is checked last;
- the bounds a refined type implies (`below`, `after`, `before`);
- the patterns people often write, by name, each with its expression and a
  message;
- `Form(show:)`'s cases, in the reader's terms;
- `data-wf-validation="shown"`, the attribute a preview sets on the page's
  root to show every message at once;
- every state the project holds to a rule, with its rules in order (those
  its type implies first) and whether the page checks them as the reader
  types.

The library has it as `webfluent::validation` (with the `internals` feature, outside semver).

See also: [Forms](../learn/14-forms.md).
