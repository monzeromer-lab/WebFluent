# WebFluent Accessibility Linting Specification

> Version 1.0 — Draft
> Author: Monzer Omer
> Date: 2026-03-24

---

## Overview

WebFluent performs **compile-time accessibility checks** on every build. When you run `webfluent build`, the compiler analyzes your `.wf` files and emits warnings for common accessibility violations. Warnings are printed to the console but **do not block the build** — your app still compiles.

This catches issues like missing alt text, unlabeled form controls, and skipped heading levels before they reach users.

---

## 1. How It Works

The accessibility linter runs automatically during `webfluent build`, after parsing and before code generation:

```
.wf files → Lexer → Parser → ⚡ A11y Linter ⚡ → Codegen → Output
```

Warnings appear in the console:

```
Warning [A01]: Image missing "alt" attribute at src/pages/Home.wf:12:5
  Add alt text: Image(src: "photo.jpg", alt: "Description of image")

Warning [A04]: Checkbox missing "label" attribute at src/pages/Settings.wf:8:9
  Add a label: Checkbox(bind: agreed, label: "I agree to terms")

Build complete with 2 accessibility warnings.
```

---

## 2. Lint Rules

### A01 — Image missing `alt`

Every `Image` must have an `alt` attribute for screen readers and when the image fails to load.

```wf
// Bad — triggers warning
Image(src: "/photo.jpg")

// Good
Image(src: "/photo.jpg", alt: "Team photo from the 2026 retreat")

// Decorative image — use empty alt
Image(src: "/divider.png", alt: "")
```

### A02 — IconButton missing accessible label

`IconButton` renders only an icon with no visible text. It must have a `label` attribute so screen readers can announce its purpose.

```wf
// Bad — triggers warning
IconButton(icon: "close")

// Good
IconButton(icon: "close", label: "Close dialog")
```

### A03 — Input missing label

Every `Input` should have a `label` or at minimum a `placeholder` so users know what to type.

```wf
// Bad — triggers warning
Input(text)

// Good
Input(text, label: "Username")
Input(text, placeholder: "Enter your name")
```

### A04 — Form control missing label

`Checkbox`, `Radio`, `Switch`, and `Slider` must have a `label` attribute.

```wf
// Bad — triggers warning
Checkbox(bind: agreed)
Switch(bind: darkMode)

// Good
Checkbox(bind: agreed, label: "I agree to the terms")
Switch(bind: darkMode, label: "Dark mode")
```

### A05 — Button has no text content

Every `Button` needs visible text (first positional argument) so users know what it does.

```wf
// Bad — triggers warning
Button()

// Good
Button("Save")
Button("Delete", danger)
```

### A06 — Link has no text content

Every `Link` should contain text content (children) or a `label` attribute.

```wf
// Bad — triggers warning
Link(to: "/about") {}

// Good
Link(to: "/about") { Text("About Us") }
```

### A07 — Heading is empty

Headings must have text content.

```wf
// Bad — triggers warning
Heading("", h1)

// Good
Heading("Welcome", h1)
```

### A08 — Modal/Dialog missing title

`Modal` and `Dialog` should have a `title` attribute so screen readers announce what the dialog is about.

```wf
// Bad — triggers warning
Modal(visible: showModal) { Text("Content") }

// Good
Modal(visible: showModal, title: "Confirm Action") { Text("Are you sure?") }
```

### A09 — Video missing controls

`Video` elements should have `controls` enabled so users can play/pause/seek.

```wf
// Bad — triggers warning
Video(src: "/intro.mp4")

// Good
Video(src: "/intro.mp4", controls: true)
```

### A10 — Table missing header row

`Table` should contain a `Thead` element with column headers.

```wf
// Bad — triggers warning
Table {
    Trow { Tcell("Alice") Tcell("Admin") }
}

// Good
Table {
    Thead { Tcell("Name") Tcell("Role") }
    Trow { Tcell("Alice") Tcell("Admin") }
}
```

### A11 — Heading levels should not skip

Heading levels should follow a logical order: h1 → h2 → h3. Skipping from h1 to h3 creates confusion for screen reader users navigating by headings.

```wf
// Bad — triggers warning (skips h2)
Heading("Title", h1)
Heading("Section", h3)

// Good
Heading("Title", h1)
Heading("Section", h2)
Heading("Subsection", h3)
```

### A12 — Page should have exactly one h1

Each page should have one (and only one) `h1` heading as its main title.

```wf
// Bad — no h1
Page Home (path: "/") {
    Heading("Welcome", h2)
}

// Bad — multiple h1
Page Home (path: "/") {
    Heading("Title One", h1)
    Heading("Title Two", h1)
}

// Good
Page Home (path: "/") {
    Heading("Welcome", h1)
    Heading("Features", h2)
}
```

---

## 3. Warning Format

All warnings follow this format:

```
Warning [<ID>]: <message> at <file>:<line>:<column>
  <hint>
```

- **ID**: Rule identifier (A01–A12)
- **message**: What's wrong
- **file:line:column**: Exact source location
- **hint**: How to fix it

---

## 4. Build Output

After all warnings are printed, the build summary includes a count:

```
Building MyApp...
  Warning [A01]: Image missing "alt" attribute at src/pages/Home.wf:12:5
  Warning [A03]: Input missing "label" attribute at src/pages/Form.wf:8:9
  3 pages, 2 components, 1 stores
  Build complete with 2 accessibility warnings.
```

If there are no warnings:

```
Building MyApp...
  3 pages, 2 components, 1 stores
  Build complete.
```

---

## 5. Limitations

- **Dynamic values**: If an attribute value comes from a variable (`alt: altText`), the linter accepts it as present — it cannot verify the runtime value is non-empty.
- **User-defined components**: Custom components are not analyzed for internal accessibility. Only built-in components are checked.
- **Complex nesting**: The linter checks direct children but does not deeply traverse user component trees.
