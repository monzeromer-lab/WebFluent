---
kind: "recipe"
route: "/recipes/call-wf-from-a-script"
group: "interop"
description: "The runtime is `window.WF`."
from: ["/docs/guide/interop#call-wf-from-a-script-of-your-own"]
---
# Call WF from a script

The runtime is `window.WF`. A script under `src/`, the console, or a
library's callback can call it: `WF.navigate("/about")`,
`WF.toast("Saved", "success")`, `WF.storeSnapshot("Cart")`. The
[runtime reference](../reference/runtime-api.md) lists every function.

> [!WARNING]
> A build keeps only the runtime modules its own code reaches. A script that
> calls a module nothing else uses needs `"build": { "runtime": "full" }`,
> or the call fails in the browser.

A script from another origin must be listed in `meta.scripts`, which widens
the policy for it. The build's CSP check refuses an inline `<script>`
([Security](../about/security.md#ship-a-content-security-policy)).
