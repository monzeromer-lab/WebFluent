# Keep the active tab in the address

> Keep the active tab in the address's #fragment, so it survives a reload and a shared link opens on the same tab.

The page: https://webfluent.monzeromer.dev/recipes/active-tab-in-the-address

A tab held in a `state` goes back to the first one on every reload. To
make the tab survive a reload, and a shared link open on the same tab,
read it from the address's `#fragment`:

```wf
page Account(path: "/account", title: "Account", description: "Your profile and billing.") {
    derived tab = if hash != "" { hash } else { "profile" }
    Heading("Account").h1
    Row(gap: .sm) {
        Link("Profile", to: "/account#profile")
        Link("Billing", to: "/account#billing")
    }
    if tab == "profile" { Text("Profile settings") }
    if tab == "billing" { Text("Billing settings") }
}
```

## How it works

- [`hash`](https://webfluent.monzeromer.dev/learn/05-pages-and-routing.md#query-strings-and-fragments) is
  the address's fragment without the `#`, kept current by the runtime.
- `tab` is a [`derived`](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#derived) value:
  the fragment when there is one, `"profile"` when there is none.
- Each tab is a `Link` to the same page with another fragment. Following it
  changes `hash`, `tab` follows, and the matching `if` draws its panel. The
  back button steps through the tabs the reader opened.

To keep several settings in the address, such as a sort and a filter, use
the query string instead: see
[Keep filters in the address](https://webfluent.monzeromer.dev/recipes/filters-in-the-address.md).
