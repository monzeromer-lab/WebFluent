# Make a page work with a keyboard and a screen reader

> Name every control, make a custom control work from the keyboard, then check the page with a keyboard, a screen reader, zoom and reduced motion.

The page: https://webfluent.monzeromer.dev/recipes/keyboard-and-screen-reader

The built-ins already render the markup assistive technology expects, and
the compiler reports what it can see in that markup
([Accessibility](https://webfluent.monzeromer.dev/about/accessibility.md)). What is left is yours: a name
for every control that matches what a sighted reader sees, and a keyboard
path through any element you turned into a control. This page has a
labelled search field, an icon-only button, and a card that acts as a
button:

```wf
page Plans(path: "/plans", title: "Plans", description: "Choose a plan.") {
    state q = ""
    state chosen = ""
    Heading("Plans").h1
    Input(bind: q, label: "Search the plans", hint: "Try “team” or “solo”").search
    IconButton(icon: "close", label: "Clear the search") { on click { q = "" } }
    Button("Search", aria-label: "Search the plans") { on click { log(q) } }
    Card(role: "button", tabindex: "0") {
        on click { chosen = "Team" }
        on key("Enter") { chosen = "Team" }
        on key("space") { chosen = "Team" }
        Text("Team plan")
    }
    Text("Chosen: {chosen}")
}
```

## How it works

- **A visible `label:`, not a placeholder.** `label:` is a real
  `<label for>`, and `hint:` is tied to the control with
  `aria-describedby`. A placeholder disappears as soon as the reader types,
  so prefer a label; `A03` warns of an `Input` with neither.
- **An icon-only control has a name.** `label:` on an `IconButton`, or
  `aria-label:` on anything else, is read aloud and never shown (`A02`).
- **An `aria-label` begins with the visible words.** Someone who says
  "click Search" to voice control then reaches the right button. One that
  does not contain them is an `A15`: a `Button("Save", aria-label: "Submit
  the form")` should say `aria-label: "Save the note"`.
- **A custom control needs a role, a place in the tab order, and keys.**
  Use `Button` whenever you can. When an element must act as one, as the
  `Card` here does, give it `role: "button"`, `tabindex: "0"` and handlers
  for the keys a button answers to, Enter and Space. Without `tabindex`,
  the card cannot be reached with Tab; without the key handlers, reaching
  it does nothing.

## Check it with assistive technology

The compiler's checks are a floor. Before you ship a page, use it the way
its readers do:

1. **Use only the keyboard.** Put the mouse away. Tab through the page,
   open and close every menu and dialog, and submit every form. Focus is
   always visible and never lost.
2. **Use a screen reader.** VoiceOver (macOS: ⌘F5), NVDA (Windows, free) or
   TalkBack (Android). Listen to the page's list of headings and landmarks,
   and to what a form says when it fails.
3. **Zoom to 200%**, and narrow the window. Nothing needs horizontal
   scrolling.
4. **Turn on reduced motion** in your system settings, and reload. Nothing
   moves.

Part of this can be automated. `wf test`'s `click "Save"` and
`type "…" into "Email"` find controls by the name a reader sees, the way
assistive technology does, so a test that passes is a page someone can
operate ([Test a page](https://webfluent.monzeromer.dev/recipes/test-a-page.md)).

## A checklist

- `wf check` reports no `A` warnings.
- Every picture's alt text says what it shows, and decorative ones have
  `alt: ""`.
- Every link and button says what it does without the text around it.
- Every control has a visible label, and every `aria-label` contains the
  visible words.
- Colour is never the only way something is said.
- Every custom control is reachable with Tab and works with Enter and
  Space.
- The page works with only a keyboard, with a screen reader, at 200% zoom,
  and with reduced motion.
