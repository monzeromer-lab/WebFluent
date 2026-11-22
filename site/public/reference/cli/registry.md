# wf registry

> Describe every built-in component: props, cases, flags, events, slots, parts

The page: https://webfluent.monzeromer.dev/reference/cli/registry

Describe every built-in component: props, cases, flags, events, slots, parts

```bash
wf registry [options]
```

## Options

| Option | Meaning |
|---|---|
| `--json` | Print JSON, for tools |

Describes every built-in component: props, cases, flags, events, slots and parts.

```bash
wf registry --json | jq '.components[] | select(.name == "Button") | .props[].name'
```

The registry is the compiler's own table: what it prints is exactly what the checker accepts.

See also: [Built-in components](https://webfluent.monzeromer.dev/reference.md).
