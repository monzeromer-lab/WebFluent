---
kind: "recipe"
route: "/recipes/active-tab-in-the-address"
group: "interaction"
description: "To make a tab survive a reload, read it from `hash`:"
from: ["/docs/guide/recipes#keep-the-active-tab-in-the-address"]
---
# Keep the active tab in the address

To make a tab survive a reload, read it from `hash`:

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
