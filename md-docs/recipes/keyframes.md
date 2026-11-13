---
kind: "recipe"
route: "/recipes/keyframes"
group: "look"
description: "To animate in a way the built-ins do not, declare keyframes with `animation Name { }` and use the name where a built-in animation goes:"
from: ["/docs/guide/motion#declare-your-own-keyframes"]
---
# Declare your own keyframes

To animate in a way the built-ins do not, declare keyframes with
`animation Name { }` and use the name where a built-in animation goes:

```wf
animation Pulse {
    from { opacity: 1 }
    50% { opacity: 0.4; transform: scale(0.98) }
    to { opacity: 1 }
}

animation Drop {
    from { transform: translateY(-12px); opacity: 0 }
    to { transform: none; opacity: 1 }
}

page Custom(path: "/", title: "Custom", description: "Keyframes of your own.") {
    state saving = false
    Heading("Custom").h1
    Card(animate: .Drop) {
        Button("Save") { on click { saving = !saving } }
        if saving { Badge("Saving", animate: .Pulse).info }
    }
}
```

The rules for keyframes:

- A step is `from`, `to` or a percentage, and holds raw CSS.
- The declaration is written once to the shared stylesheet as
  `@keyframes Name`.
- Your own animation is always written `animate: .Name` (or `exit:
  .Name`). The flags belong to the built-in animations.
- It plays wherever a built-in one would — on an element that stands still,
  from the stylesheet, before any script has loaded.

You can also name your keyframes in a style, as CSS does:
`animation: Pulse 1s infinite`.
