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
    Tabs(value: tab, label: "Account") {
        on change(v) { navigate("/account#{v}") }
        Tabs.Page("Profile", value: "profile") { Text("Profile settings") }
        Tabs.Page("Billing", value: "billing") { Text("Billing settings") }
    }
}
```

Open `/account#billing` and the Billing tab is selected; click Profile and
the address becomes `/account#profile`.

## How it works

- [`hash`](../learn/05-pages-and-routing.md#query-strings-and-fragments) is
  the address's fragment without the `#`, kept current by the runtime.
- `tab` is a [`derived`](../learn/07-state-and-reactivity.md#derived) value:
  the fragment when there is one, `"profile"` when there is none.
- `Tabs(value: tab)` opens the page whose `value:` is `tab`. It only reads
  it: picking a tab fires `on change(v)` with that page's value, which
  navigates to the same page with another fragment. `hash` changes, `tab`
  follows, and the tab opens. Each pick is a step in the history, so the
  back button steps through the tabs the reader opened.
- The tabs are a tab list to a screen reader, each linked to its panel.
  Only the open tab is in the Tab order; the arrow keys move between tabs
  (the other way round in a right-to-left page), and Home and End go to
  the ends.

For a tab that need not survive a reload, `Tabs(bind: tab)` reads and
writes a `state` instead.

Why not a `Link` per tab? The router marks a link current by its path, not
its fragment: on `/account#billing`, both `/account#profile` and
`/account#billing` are links to the current page, so both get the `active`
class and `aria-current="page"`, and neither the look nor a screen reader
says which tab is open.

To keep several settings in the address, such as a sort and a filter, use
the query string instead: see
[Keep filters in the address](filters-in-the-address.md).
