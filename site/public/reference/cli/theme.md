# wf theme

> The theme the build uses: every token, declared or inherited, where each is used, and its contrast pairs

The page: https://webfluent.monzeromer.dev/reference/cli/theme

The theme the build uses: every token, declared or inherited, where each is used, and its contrast pairs

```bash
wf theme [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--json` | Print JSON, for tools |

Shows the theme the build uses, as an editor's theme panel shows it.

```bash
wf theme
wf theme --json
```

It lists every token by group (colour, spacing, font size, line height, radius, shadow, transition, font, motion, and the tokens the theme adds) with its value, the baseline's value, whether the theme declares or inherits it, the dark theme's value when `theme.dark` names one, the properties that draw from it, and every place the project names it. It then lists the colour pairings the stylesheet composes, with their contrast and what WCAG AA asks of each — a pairing that falls short is the `A13` the build reports — and the tokens one theme of the dark pair declares and the other does not.

See also: [Design tokens](https://webfluent.monzeromer.dev/reference/tokens.md).
