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
| `bind:` | `State` | The open tab, read and written: a page's `value:`, or its place from 0 when the pages give none; a value no page has opens the first |
| `value:` | `String or Number` | The open tab, read only: a press does not change it, only `on change(v)` hears the one picked — for a page that sets it itself |
| `orientation:` | `one of its cases` | Which way the tabs run, and which arrow keys move between them; a flag, `.vertical`. Written as a flag, `.horizontal`, or by name, `orientation: .horizontal`.

- `.horizontal` (the default): Tabs in a row; ArrowLeft and ArrowRight move (the default)
- `.vertical`: Tabs in a column (`aria-orientation="vertical"`); ArrowUp and ArrowDown move |
| `activation:` | `one of its cases` | Whether moving the focus opens a tab; a flag, `.manual`. Written as a flag, `.automatic`, or by name, `activation: .automatic`.

- `.automatic` (the default): A tab opens as the focus reaches it (the default)
- `.manual`: The focus moves alone; Enter or Space opens the tab, for panels slow to draw |
| `label:` | `String` | The tab strip's accessible name (`aria-label` on its `tablist`) |

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
| `label` | `String (positional)` | The words on its tab button |
| `value:` | `String or Number` | What the Tabs' `bind:` holds while this tab is open |
| `disabled:` | `Bool` | Shown, but cannot be opened: the button is disabled and the arrow keys pass over it; `.disabled`, or `disabled: value` to follow state. Written as a flag, `.disabled`, or `disabled: true`. |

Flags: `.disabled`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
