# Tree

> Nested items — files and folders, an outline — as a tree a keyboard can work: Up and Down through what shows, Right to open and go in, Left to close and go out (the other way round right to left), Home and End, a letter. row(item) { … } draws each; bind: holds the chosen item's key, or a list of keys with .multiple.

The page: https://webfluent.monzeromer.dev/reference/tree

Nested items — files and folders, an outline — as a tree a keyboard can work: Up and Down through what shows, Right to open and go in, Left to close and go out (the other way round right to left), Home and End, a letter. row(item) { … } draws each; bind: holds the chosen item's key, or a list of keys with .multiple.

```wf
Tree(items: Any, children: Any, key: Any, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `items:` | `Any  (required)` | The items at the top |
| `children:` | `Any  (required)` | An item's children: `f => f.children`; none, or an empty list, for a leaf |
| `key:` | `Any  (required)` | What tells two items apart: `f => f.path` |
| `bind:` | `State` | The chosen key, or a list of them with `.multiple` |
| `multiple:` | `Bool` | Several at once: Space, Shift with an arrow, Ctrl or ⌘ with A |
| `open:` | `Any` | The keys open when it is drawn |
| `label:` | `String  (required)` | What the tree is, for a screen reader |

## Flags

`.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state files = [{ path: "src", name: "src", children: [{ path: "src/App.wf", name: "App.wf", children: [] }] }]
state file = ""
Tree(items: files, children: f => f.children, key: f => f.path, bind: file, label: "Files") {
    row(f) { Icon(if f.children.length > 0 { "folder" } else { "file" })  Text(f.name) }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
