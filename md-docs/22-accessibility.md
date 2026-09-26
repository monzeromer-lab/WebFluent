# 22. Accessibility

<!--
route: guide/accessibility
group: building
blurb: What the built-ins do for readers who use a keyboard, a screen reader or voice control, what the compiler checks on every build, and what is still yours to do.
description: What the built-ins do for keyboard and screen-reader users, the A01–A15 checks, what is still yours to do, and how to test it.
-->

Accessibility is not a plugin here: the built-ins render the markup assistive
technology expects, the router and the form manage focus, and the compiler
checks every page on every build.

## What you get without asking

- **Semantic elements.** A `Button` is a `<button>`, a `Link` an `<a>`,
  `Heading(…).h2` an `<h2>`, `Header`/`Section`/`Footer` landmarks, a
  `Table` a `<table>` whose head cells are `<th scope="col">`, a `List` a
  `<ul>` or `<ol>`, the page's content a `<main>`.
- **A skip link**, first in the tab order and visible when focused, jumping
  past the navigation to the content.
- **Route changes that behave like page loads.** Focus moves to the new
  page's heading, the scroll returns to the top, and the new title is
  announced.
- **Labelled fields.** `label:` is a real `<label for>`; `hint:` and the
  error are tied to the control with `aria-describedby`; an error sets
  `aria-invalid` and is announced when it appears.
- **Forms that point at the problem.** A submit that fails moves focus to
  the first field with an error and announces it.
- **Dialogs that trap focus.** `Modal` and `Dialog` are real `<dialog>`
  elements: the rest of the page is inert, Escape closes, and focus returns
  to what opened them.
- **Menus with the keys people expect.** Arrows move, Home and End jump,
  Enter and Space choose, Escape closes.
- **Announcements.** A `Toast` is spoken politely; `WF.announce(text)` says
  anything else.
- **The active link** carries `aria-current="page"`.
- **Reduced motion.** A reader who asked for it gets no animation at all, and
  a carousel does not auto-play.
- **Visible focus.** Every interactive built-in has a focus ring for the
  keyboard (`:focus-visible`), not on every click.

## What the compiler checks

Warnings on every build and in the editor ([Diagnostics](39-diagnostics.md#accessibility)):

| Code | Checks |
|---|---|
| `A01` | Every `Image` has `alt:` (empty for decoration) |
| `A02` | Every `IconButton` has a `label:` |
| `A03`, `A04` | Every `Input`, `Checkbox`, `Radio`, `Switch`, `Slider` and `Textarea` has a label |
| `A05`, `A06`, `A07` | Buttons, links and headings have text |
| `A08` | Every `Modal` and `Dialog` has a `title:` |
| `A09` | Video has captions and controls; audio has a transcript |
| `A10` | Tables have a header row |
| `A11`, `A12` | One `h1` per page, and no skipped heading levels |
| `A13` | The theme's text and background colours have AA contrast |
| `A14` | A `role:` holds children of the roles it requires |
| `A15` | A control's `aria-label` starts with its visible text, for voice control |

## What is still yours

The compiler can see markup; it cannot see meaning.

- **Alt text that says what the picture shows**, in context. "Photo" passes
  `A01` and helps nobody.
- **Link text that says where it goes.** Twelve "Read more" links pass `A06`.
- **An order that makes sense.** The reading order is the source order; do
  not rearrange it visually with `order:` or absolute positioning.
- **Colour that is not the only signal.** An error in red also says so in
  words; a chart has labels.
- **Your own styles' contrast.** `A13` checks the theme's tokens, not a
  colour written raw in a `style { }`.
- **Custom controls.** A `Card` made clickable with `on click` is not a
  button to a keyboard. Use `Button` — or give it `role: "button"`,
  `tabindex: 0` and `on key("Enter") { }` and `on key("space") { }`.
- **Time limits.** A `Toast` disappears; anything the reader must act on
  belongs in an `Alert` that stays.

## Labelling things well

```wf
page Search(path: "/", title: "Search", description: "Find a page.") {
    state q = ""
    Heading("Search").h1
    Input(bind: q, label: "Search the docs", hint: "Try “routing” or “forms”").search
    IconButton(icon: "close", label: "Clear the search") { on click { q = "" } }
    Button("Search", aria-label: "Search the docs") { on click { log(q) } }
}
```

- A visible `label:` is better than a placeholder, which disappears.
- An icon-only control is named by `label:` (`IconButton`) or `aria-label:`.
- An `aria-label` begins with the visible words, so someone saying "click
  Search" hits the right button (`A15`).

## Testing with assistive technology

- **Keyboard.** Put the mouse away: Tab through the page, open and close
  every menu and dialog, submit every form. Focus should always be visible
  and never lost.
- **A screen reader.** VoiceOver (macOS: ⌘F5), NVDA (Windows, free) or
  TalkBack (Android). Listen to a page's headings list and landmarks, and to
  what a form says when it fails.
- **Zoom** to 200%, and a narrow window: nothing should need horizontal
  scrolling.
- **Reduced motion**: turn it on in your system settings and reload.
- **Tests that find things by name.** `wf test`'s `click "Save"` and
  `type "…" into "Email"` find controls the way assistive technology does,
  so a test that passes is a page someone can operate ([Testing](24-testing.md)).

## Next

[Security](23-security.md).
