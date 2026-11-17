# AppShell

> An app's frame: AppShell.Header across the top, a Sidebar down the start (a drawer on a narrow screen), the Router's page beside it, and an AppShell.Footer.

The page: https://webfluent.monzeromer.dev/reference/appshell

An app's frame: AppShell.Header across the top, a Sidebar down the start (a drawer on a narrow screen), the Router's page beside it, and an AppShell.Footer.

```wf
AppShell { … }
```

## Example

```wf
AppShell {
    style { min-height: 320px; border: 1px solid $color-border }
    AppShell.Header {
        Row(justify: .between, align: .center) {
            style { flex: 1 }
            Text("Ledger").bold
            Button("Sign out").sm
        }
    }
    Sidebar {
        style { height: auto }
        Sidebar.Item(to: "/learn", icon: "book") { Text("Learn") }
        Sidebar.Item(to: "/reference", icon: "list") { Text("Reference") }
    }
    Stack(gap: .sm) {
        style { padding: 1rem }
        Heading("Dashboard").h3
        Text("The page the router draws sits here.")
    }
    AppShell.Footer { Text("© 2026 Ledger").muted.sm }
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `AppShell`.

### AppShell.Header

The bar across the top.

```wf
AppShell.Header { … }
```

### AppShell.Footer

The bar along the bottom.

```wf
AppShell.Footer { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
