# Combobox

> A text field that offers what matches as the reader types: from options:, or search: asked of a server. The arrow keys go through the offers, Enter takes one, Escape closes them; .multiple keeps several, each a chip Backspace takes back. option(o) { … } draws an offer; empty { … } shows when nothing matches.

The page: https://webfluent.monzeromer.dev/reference/combobox

A text field that offers what matches as the reader types: from options:, or search: asked of a server. The arrow keys go through the offers, Enter takes one, Escape closes them; .multiple keeps several, each a chip Backspace takes back. option(o) { … } draws an offer; empty { … } shows when nothing matches.

```wf
Combobox(bind: State, options: Any, search: Any, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The chosen option's key (or the option itself without `key:`), or a list with `.multiple` |
| `options:` | `Any` | Every option, filtered as the reader types |
| `search:` | `Any` | Options asked for as the reader types: `q => Geo.cities(q: q)` |
| `text:` | `Any  (required)` | An option's words: `p => p.name` |
| `key:` | `Any` | What tells two options apart: `p => p.id` |
| `debounce:` | `Any` | How long the typing rests before `search:` is asked (unset, 200ms) |
| `multiple:` | `Bool` | Several at once, each a chip |
| `label:` | `String  (required)` | Its name for a screen reader |
| `placeholder:` | `String` | Shown while it is empty |

## Flags

`.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state city = ""
state cities = [{ id: "par", name: "Paris", country: "France" }, { id: "por", name: "Porto", country: "Portugal" }]
Combobox(bind: city, options: cities, text: c => c.name, key: c => c.id, label: "City") {
    option(c) { Text(c.name)  Text(c.country).muted }
    empty { Text("No city by that name") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
