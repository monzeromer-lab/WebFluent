---
kind: "recipe"
route: "/recipes/active-tab-in-the-address"
group: "interaction"
description: "Keep the active tab in the address's #fragment, so it survives a reload and a shared link opens on the same tab."
outputs: ["website", "app"]
from: ["/docs/guide/recipes#keep-the-active-tab-in-the-address"]
---
# Keep the active tab in the address

A tab held in a `state` goes back to the first one on every reload. To
make the tab survive a reload, and a shared link open on the same tab,
read it from the address's `#fragment`:

```wf
page Account(path: "/account", title: "Account", description: "Your profile and billing.") {
    derived tab = if hash != "" { hash } else { "profile" }
    Heading("Account").h1
    Row(gap: .sm, role: "tablist", aria-label: "Account") {
        Button("Profile", role: "tab", aria-selected: tab == "profile", outlined: tab != "profile") {
            on click { navigate("/account#profile") }
        }
        Button("Billing", role: "tab", aria-selected: tab == "billing", outlined: tab != "billing") {
            on click { navigate("/account#billing") }
        }
    }
    if tab == "profile" { Text("Profile settings", role: "tabpanel") }
    if tab == "billing" { Text("Billing settings", role: "tabpanel") }
}
```

Open `/account#billing` and the Billing tab is selected; click Profile and
the address becomes `/account#profile`.

## How it works

- [`hash`](../learn/05-pages-and-routing.md#query-strings-and-fragments) is
  the address's fragment without the `#`, kept current by the runtime.
- `tab` is a [`derived`](../learn/07-state-and-reactivity.md#derived) value:
  the fragment when there is one, `"profile"` when there is none.
- Each tab is a `Button` that navigates to the same page with another
  fragment. `hash` changes, `tab` follows, and the matching `if` draws its
  panel. Each click is a step in the history, so the back button steps
  through the tabs the reader opened.
- `role: "tablist"`, `role: "tab"` and `aria-selected:` tell a screen reader
  these are tabs and which one is open; `outlined:` shows the same thing to
  the eye. Both follow `tab`.

Why not a `Link` per tab? The router marks a link current by its path, not
its fragment: on `/account#billing`, both `/account#profile` and
`/account#billing` are links to the current page, so both get the `active`
class and `aria-current="page"`, and neither the look nor a screen reader
says which tab is open.

To keep several settings in the address, such as a sort and a filter, use
the query string instead: see
[Keep filters in the address](filters-in-the-address.md).
