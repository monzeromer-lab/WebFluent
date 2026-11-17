# Tabs

> Tab strip switching between its pages: the arrow keys move between the tabs (the other way round in a right-to-left page), Home and End go to the ends.

The page: https://webfluent.monzeromer.dev/reference/tabs

Tab strip switching between its pages: the arrow keys move between the tabs (the other way round in a right-to-left page), Home and End go to the ends.

```wf
Tabs(bind: State, value: String or Number, orientation: Enum, …) { … }
```

## Example

```wf
state tab = "profile"
Tabs(bind: tab, label: "Account") {
    Tabs.Page("Profile", value: "profile") {
        Text("Your name, photo and the email we write to.")
    }
    Tabs.Page("Billing", value: "billing") {
        Text("Pro plan, renews on 1 March.")
    }
    Tabs.Page("Danger zone", value: "danger", disabled: true) {
        Text("Only an owner can delete the account.")
    }
}
Text("Open: {tab}").muted.sm
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The open tab, read and written: a page's `value:`, or its place from 0 when the pages give none |
| `value:` | `String or Number` | The open tab, read only: `on change(v)` hears the one the reader picks |
| `orientation:` | `.horizontal .vertical` | Which way the tabs run |
| `activation:` | `.automatic .manual` | Whether moving the focus opens a tab |
| `label:` | `String` | The tab strip's accessible name |

## Flags

`.horizontal` `.vertical` `.automatic` `.manual`

## Events

- `change`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Tabs`.

### Tabs.Page

One tab and its panel.

```wf
Tabs.Page(_ label: String, value: String or Number, disabled: Bool) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The tab's label |
| `value:` | `String or Number` | What the Tabs' `bind:` holds while this tab is open |
| `disabled:` | `Bool` | Shown, but not opened: the arrow keys pass over it |

Flags: `.disabled`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
