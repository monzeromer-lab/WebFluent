---
kind: "recipe"
route: "/recipes/migrate-from-external"
group: "interop"
description: "Before 4.2, you described JavaScript with `external` declarations; 4.2 removed them."
from: ["/docs/guide/interop#migrate-from-external"]
---
# Migrate from external

Before 4.2, you described JavaScript with `external` declarations; 4.2
removed them. `wf migrate` rewrites a project:

- A remote module becomes a `meta.scripts` entry. Its calls keep working,
  typed `Any`.
- A local module becomes a plain script under `src/`, with its exports
  removed and its signatures as JSDoc. `X.f(…)` is rewritten to `f(…)`.
- An `external element` call site becomes `Element("tag-name", …)`.
