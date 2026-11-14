---
chapter: 20
kind: "about"
route: "/about/accessibility"
from: ["/docs/guide/accessibility"]
group: "about"
blurb: "Pages that work with a keyboard, a screen reader or voice control: what the built-ins do, what the compiler checks, and what no tool can check for you."
description: "What the built-ins do for keyboard and screen-reader users, the A01–A18 checks, and the part of accessibility that stays a person's judgement."
---
# Accessibility

This essay is about the accessibility a WebFluent page has before you do
anything, the part the compiler checks on every build, and the part that
stays yours: building pages people can use with a keyboard, a screen
reader, voice control, zoom or reduced motion.

Most inaccessible pages were not made that way on purpose. A `<div>` with
a click handler looks like a button and is not one; a field whose label is
a placeholder looks labelled until the reader types. Each is a small
mistake that a checklist would catch, and checklists are read once.

WebFluent moves as much of the checklist as it can out of your hands. The
built-ins render the markup assistive technology expects, the router and
forms manage focus, and the compiler reports what it can see as a finding
with a code, on every build. That leaves the work no tool can do (meaning,
wording, order) and frees your attention for it.

## What the built-ins do

You get these behaviours without asking, as long as you use the built-in
elements rather than rebuilding them:

- **Semantic elements.** A `Button` is a `<button>`, a `Link` an `<a>`,
  `Heading(…).h2` an `<h2>`. `Header` and `Footer` are landmarks,
  and so is a `Section` given an `aria-label`. A `Table` is a `<table>`
  whose head cells are `<th scope="col">`, a `List` is a `<ul>` or `<ol>`,
  and the page's content is a `<main>`.
- **A skip link.** It is first in the tab order, visible when focused, and
  jumps past the navigation to the content.
- **Route changes that behave like page loads.** Focus moves to the new
  page's heading, the scroll returns to the top, and the new title is
  announced.
- **Labelled fields.** `label:` is a real `<label for>`. `hint:` and the
  error are tied to the control with `aria-describedby`. An error sets
  `aria-invalid` and is announced when it appears ([Forms](../learn/14-forms.md)).
- **Forms that point at the problem.** A submit that fails moves focus to
  the first field with an error and announces it.
- **Dialogs that hold focus.** `Modal` and `Dialog` are real `<dialog>`
  elements: the rest of the page is inert, Escape closes them, and focus
  returns to what opened them.
- **Focus held where you ask.** Any other element holds focus with
  `trapFocus: open` — a find bar, a panel. Focus moves in, Tab and
  Shift+Tab go round inside it, a click outside brings focus back, and when
  `open` turns false, focus returns to where it was.
- **Menus with the keys people expect.** The arrow keys move and skip a
  disabled item, Home and End jump to the ends, Enter and Space choose,
  ArrowRight opens a submenu and ArrowLeft leaves it, and Escape closes the
  innermost menu and returns focus. A check item is a `menuitemcheckbox`
  with `aria-checked`; an item's shortcut is its `aria-keyshortcuts`.
- **Splitters you can move without a mouse.** A `Splitter` is a
  `separator` with its size as `aria-valuenow`. It takes focus, the arrow
  keys move it, Home and End go to its ends, and Enter resets it.
- **Announcements.** A `Toast` is spoken politely. To announce anything
  else, call `WF.announce(text)`.
- **The active link.** The link to the current page carries
  `aria-current="page"`.
- **Reduced motion.** A reader who asked for it gets no animation at all,
  and a carousel does not play on its own ([Motion](../recipes/motion-defaults.md#respect-reduced-motion)).
- **Visible focus.** Every interactive built-in shows a focus ring for the
  keyboard (`:focus-visible`), but not on every click.

## What the compiler checks

The compiler checks every page for the mistakes it can see in the markup,
and reports each as a warning on every build and in the editor. Each code
has a full entry, with a fix, in [Diagnostics](../errors/diagnostics.md#accessibility),
and `wf explain CODE` prints it in your terminal.

| Code | What it checks |
|---|---|
| `A01` | Every `Image` has `alt:` (an empty one for a decorative picture) |
| `A02` | Every `IconButton` has a `label:` |
| `A03`, `A04` | Every `Input` has a label or at least a placeholder (prefer a label); every `Checkbox`, `Radio`, `Switch`, `Slider` and `Textarea` has a label |
| `A05`, `A06`, `A07` | Buttons, links and headings have text |
| `A08` | Every `Modal`, `Dialog`, `AlertDialog` and `Sheet` has a `title:` (a `Sheet` may have a `label:`) |
| `A09` | A video has captions and controls; audio has a transcript |
| `A10` | A table has a header row |
| `A11`, `A12` | No heading level is skipped, and each page has exactly one `h1` |
| `A13` | The theme's text and background colours meet WCAG AA contrast |
| `A14` | A `role:` holds children of the roles it requires |
| `A15` | A control's `aria-label` contains its visible text, for voice control |
| `A16` | No literal `id:` is shared by several elements, in a loop or a reused component |
| `A17` | Every `Frame` has a `title:` |
| `A18` | Every `Popover` drawn as a dialog has a `label:` |

For example, this button's `aria-label` does not contain the word it
shows, so a voice-control user who says "click Save" may not reach it:

```wf expect=A15
page Editor(path: "/", title: "Editor", description: "Write a note.") {
    Heading("Editor").h1
    Button("Save", aria-label: "Submit the form")
}
```

The fix is to start the `aria-label` with the visible words:
`aria-label: "Save the note"`.

## What the compiler cannot check

The compiler can see markup; it cannot see meaning. These are still yours:

- **Alt text that says what the picture shows**, in its context. "Photo"
  passes `A01` and helps nobody.
- **Link text that says where it goes.** Twelve "Read more" links pass
  `A06`.
- **An order that makes sense.** The reading order is the source order. Do
  not rearrange it visually with `order:` or absolute positioning.
- **Colour that is not the only signal.** An error in red also says so in
  words; a chart has labels.
- **The contrast of your own styles.** `A13` checks the theme's tokens, not
  a colour written directly in a `style { }` block.
- **Custom controls.** A `Card` made clickable with `on click` is not a
  button to a keyboard until it has a role, a place in the tab order and
  key handlers.
- **Time limits.** A `Toast` disappears. Anything the reader must act on
  belongs in an `Alert` that stays.

Each of these is a judgement about what a page means to the person using
it, which is why no build can make it. What a tool can do is make the
judgement cheaper:
[Make a page work with a keyboard and a screen reader](../recipes/keyboard-and-screen-reader.md)
names every control so it matches what a sighted reader sees, turns a card
into a control a keyboard can reach, and then checks the page the way its
readers use it — with only a keyboard, a screen reader, 200% zoom and
reduced motion — ending with a checklist. Because `wf test` finds a
control by the name a reader sees, as assistive technology does, a test
that clicks through a page is also a test that someone can operate it.

## Next

[Forms and validation](../learn/14-forms.md): bind controls to state, validate what the reader types, upload files, and show what the server said.
