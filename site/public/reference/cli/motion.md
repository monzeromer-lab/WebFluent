# wf motion

> The animations an element can play, the eases and speeds, the route transitions and the project's default

The page: https://webfluent.monzeromer.dev/reference/cli/motion

The animations an element can play, the eases and speeds, the route transitions and the project's default

```bash
wf motion [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--json` | Print JSON, for tools |

Lists what an element's motion props can name.

```bash
wf motion
wf motion --json
```

It lists:

- every animation `animate:` and `exit:` take: the engine's, each with its
  keyframes, the rule its stylesheet plays it by, whether an element leaves
  with it and whether it repeats; then the project's own `animation`
  declarations;
- the named eases, with the timing function each stands for;
- the three speeds and their lengths;
- the `Router`'s transitions;
- the project's default length and pacing (`motion` in the config, or the
  engine's);
- `data-wf-motion="reduce"`, the attribute an editor sets on the page's
  root to show the page as a reader who asked for less motion sees it.

The library has it as `webfluent::motion::catalogue` (with the `internals` feature, outside semver).

See also: [Motion defaults](../recipes/motion-defaults.md).
