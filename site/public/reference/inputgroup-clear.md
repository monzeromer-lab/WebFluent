# InputGroup.Clear

> A button that empties the input and puts the focus back in it; hidden while it is empty.

The page: https://webfluent.monzeromer.dev/reference/inputgroup-clear

A button that empties the input and puts the focus back in it; hidden while it is empty.

```wf
InputGroup.Clear
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
