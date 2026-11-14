# AppShell

> An app's frame: AppShell.Header across the top, a Sidebar down the start (a drawer on a narrow screen), the Router's page beside it, and an AppShell.Footer.

The page: https://webfluent.monzeromer.dev/reference/appshell

An app's frame: AppShell.Header across the top, a Sidebar down the start (a drawer on a narrow screen), the Router's page beside it, and an AppShell.Footer.

```wf
AppShell { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `AppShell.AppShell.Header`
- `AppShell.AppShell.Footer`

## Example

```wf
AppShell {
    AppShell.Header { Text("Ledger").bold  Spacer  ThemeToggle }
    Sidebar { Sidebar.Item(to: "/", icon: "home") { Text("Home") } }
    Router
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
