# InputGroup

> An input drawn as one box with what goes beside it: an icon, InputGroup.Addon("https://"), a button, or InputGroup.Clear, which empties the input and gives it the focus back.

The page: https://webfluent.monzeromer.dev/reference/inputgroup

An input drawn as one box with what goes beside it: an icon, InputGroup.Addon("https://"), a button, or InputGroup.Clear, which empties the input and gives it the focus back.

```wf
InputGroup { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `InputGroup.InputGroup.Addon`
- `InputGroup.InputGroup.Clear`

## Example

```wf
state q = ""
InputGroup {
    Icon("search")
    Input(bind: q, aria-label: "Search")
    InputGroup.Clear
}
state site = ""
InputGroup { InputGroup.Addon("https://")  Input(bind: site, aria-label: "Site") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
