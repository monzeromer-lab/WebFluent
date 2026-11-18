# NavigationMenu

> A site's main navigation whose entries open panels (a mega menu) as disclosures: one open at a time, the arrow keys along the row, Down into a panel, Escape, a press outside or the focus leaving closes it.

The page: https://webfluent.monzeromer.dev/reference/navigationmenu

A site's main navigation whose entries open panels (a mega menu) as disclosures: one open at a time, the arrow keys along the row, Down into a panel, Escape, a press outside or the focus leaving closes it.

```wf
NavigationMenu(label: String) { … }
```

## Example

```wf
NavigationMenu(label: "Main") {
    NavigationMenu.Item("Learn") {
        Stack(gap: .sm) {
            Link("The guide", to: "/learn")
            Link("Data and requests", to: "/learn/15-data")
            Link("Recipes", to: "/recipes")
        }
    }
    NavigationMenu.Item("Reference") {
        Stack(gap: .sm) {
            Link("Components", to: "/reference")
            Link("Diagnostics", to: "/errors")
        }
    }
    NavigationMenu.Link("Playground", to: "/playground")
    NavigationMenu.Link("Showcase", to: "/showcase")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The navigation's accessible name (`aria-label` on its `<nav>`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `NavigationMenu`.

### NavigationMenu.Item

An entry that opens a panel: its words are its button, its block the panel.

```wf
NavigationMenu.Item(_ label: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String (positional)` | The words on the button that opens its panel; the block is the panel |

### NavigationMenu.Link

An entry that is a link, beside the ones that open panels.

```wf
NavigationMenu.Link(_ label: String, to: Path, active: Enum)
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String (positional)` | The link's words |
| `to:` | `Path (required)` | The route it links to, navigated in place; while it matches the current route it carries `.active` and `aria-current="page"` |
| `active:` | `one of its cases` | How it decides it is the current page: unset, the route must equal `to`; `.prefix` also matches routes beneath it. Written as a flag, `.prefix`, or by name, `active: .prefix`.

- `.prefix`: Also current on every route beneath `to`: `/docs` for `/docs/forms` |

Flags: `.prefix`

## Drawn for it

What `NavigationMenu` draws itself, by its class; not written.

- `NavigationMenu.List` (`.`): Its entries.
- `NavigationMenu.Trigger` (`.`): An entry's button.
- `NavigationMenu.Panel` (`.`): An entry's panel.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
