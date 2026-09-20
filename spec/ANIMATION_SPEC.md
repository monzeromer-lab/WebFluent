# WebFluent Animation System Specification

> Version 3.0
> Author: Monzer Omer
> Date: 2026-09-20 (first draft 2026-03-24)

---

## Overview

WebFluent provides a **declarative animation system** that lets you add motion to your applications without writing CSS keyframes or JavaScript animation code. Motion is built into the language at five levels:

1. **Motion props** — the enter and exit animation of any element, its timing, and the stagger of a list
2. **Transition blocks** — declarative CSS transitions on property changes
3. **Branches and lists** — what `if`, `for`, `show` and `match` play when their content comes and goes, and how a keyed list moves its items
4. **Route transitions** — how one page gives way to the next
5. **Replay** — restart an animation from a handler

Every animation respects `prefers-reduced-motion`: a reader who asked for less motion gets none, and nothing waits for an animation that did not play.

---

## 1. Motion Props

Every element — built-in or a component of the project — takes the same motion props. An enter animation is written as a flag; the rest are named.

### Syntax

```
Element(args, exit: .<name>, duration: "<time>", delay: "<time>", easing: "<function>", stagger: "<time>").<enter>.<speed>
```

| Prop | Written | Meaning |
|------|---------|---------|
| enter | `.fadeIn` — or `animate: .fadeIn` | Plays when the element appears |
| `exit` | `exit: .fadeOut` | Plays before the element is removed |
| speed | `.fast` (150ms), `.slow` (500ms) | The duration, by name |
| `duration` | `duration: "300ms"` | The duration, exactly |
| `delay` | `delay: "100ms"` | Wait before the animation starts |
| `easing` | `easing: "ease-out"` | Timing function; any CSS `animation-timing-function` |
| `stagger` | `stagger: "50ms"` | Inside a `for`: delay added per item, by its index |

### Built-in Animations

| Name | Effect |
|------|--------|
| `fadeIn` | Fade from 0 to full opacity |
| `fadeOut` | Fade from full opacity to 0 |
| `slideUp` | Slide in from below |
| `slideDown` | Slide in from above |
| `slideLeft` | Slide in from the right |
| `slideRight` | Slide in from the left |
| `scaleIn` | Scale up from 0.9 to full size |
| `scaleOut` | Scale down from full size to 0.9 |
| `bounce` | Bouncy entrance |
| `shake` | Horizontal shake |
| `pulse` | Gentle scale pulse, repeating |
| `spin` | 360° rotation, repeating |

### Examples

```wf
// Fade in a card on mount
Card.elevated.fadeIn {
    Text("Welcome!")
}

// Slide up, slowly
Heading("Hello").h1.slideUp.slow

// Bounce a button
Button("Click me").primary.bounce

// Several elements, each with its own motion
Container {
    Heading("Dashboard").h1.fadeIn
    Row.slideUp {
        Card.scaleIn { Text("Users: 42") }
        Card(delay: "100ms").scaleIn { Text("Revenue: $1.2k") }
    }
}

// A component takes the same props; they land on its root element
UserCard("Sam", role: "Designer", exit: .fadeOut).fadeIn
```

---

## 2. Transition Blocks

Declare CSS transitions on an element. These apply smooth interpolation when reactive properties change (hover states, style changes, class toggling).

### Syntax

```
Element {
    transition {
        <property>: <duration> [easing]
        ...
    }
}
```

A property is any CSS property name, or `all`. The duration and the easing may be design tokens (`$animation-duration-fast`, or a theme's own `$ease-standard`), which compile to `var(--…)`.

### Easing Names

| Name | Value |
|------|-------|
| `ease` | `ease` (default) |
| `linear` | `linear` |
| `easeIn` | `ease-in` |
| `easeOut` | `ease-out` |
| `easeInOut` | `ease-in-out` |
| `spring` | `cubic-bezier(0.175, 0.885, 0.32, 1.275)` |
| `bouncy` | `cubic-bezier(0.68, -0.55, 0.265, 1.55)` |
| `smooth` | `cubic-bezier(0.4, 0, 0.2, 1)` |

Any other word is passed through as written.

### Examples

```wf
// Smooth background and color transitions
Button("Hover me").primary {
    transition {
        background: 200ms ease
        color: 200ms ease
        transform: 150ms spring
    }
}

// Transition all properties, with the theme's timing
Card.elevated {
    style {
        background: $surface
    }
    transition {
        all: $animation-duration-normal $animation-easing-default
    }
}
```

---

## 3. Branches and Lists

`if`, `for`, `show` and `match` play the motion of the elements they put in and take out.

### Enter and exit

An enter animation on an element plays when the element is created — on first paint, or when its branch becomes true. An `exit:` plays before the element is removed:

- On the **root elements of a branch** — what an `if`, `for`, `show` or `match` arm holds directly — the enter and exit become the branch's own: the branch waits for every root's exit before it removes the old content.
- On **any other element**, the exit is the element's own: when the element leaves the page for any reason — its branch closed, its list dropped it, its page was left — the exit plays first, and the removal waits for it.

Without an `exit:`, content is removed at once.

### Lists

`stagger:` delays each item by its index, so a list slides in one item after another. A **keyed** list (`for item in items by item.id`) keeps the node of an item that is still there, so when items are reordered the nodes move, sliding from where they were to where they now sit (first-last-invert-play); a list without a key is rebuilt.

### `show`

`show` keeps the content in the DOM and toggles `display: none`. The enter plays when it is shown, the exit before it is hidden.

### Examples

```wf
// Fade a message in and out
if showMessage {
    Alert("Operation successful!", exit: .fadeOut).success.fadeIn
}

// Slide cards up with stagger; items keep their nodes and move
for product in products by product.id {
    Card(exit: .fadeOut, stagger: "50ms").elevated.slideUp {
        Text(product.name).heading
        Text("${product.price}")
    }
}

// Scale a dialog
show isModalOpen {
    Modal(visible: isModalOpen, title: "Confirm", exit: .scaleOut, duration: "200ms").scaleIn {
        Text("Are you sure?")
        Button("Yes").primary { on click { confirm() } }
        Button("No") { on click { isModalOpen = false } }
    }
}

// Custom duration and easing
if isExpanded {
    Stack(gap: .md, exit: .slideUp, duration: "400ms", easing: "spring").slideDown {
        Text("Expanded content line 1")
        Text("Expanded content line 2")
    }
}

// Fade in only; removed at once when the data goes
if hasData {
    Table.fadeIn { ... }
}
```

---

## 4. Route Transitions

The `Router` takes a transition that plays between pages: the old page animates out, the new one in, and the focus and scroll settling of a route change runs after the new page has arrived.

```wf
app {
    Navbar { ... }
    Router(transition: .fade, duration: "200ms")
}
```

| `transition` | Out | In |
|--------------|-----|----|
| `.none` (default) | — | — |
| `.fade` | `fadeOut` | `fadeIn` |
| `.slide` | `slideLeft` | `slideRight` |

Static output (SSG) is unaffected: a page is painted whole.

---

## 5. Replay

Restart an element's animation from a handler, for feedback (shake on error, pulse on update):

```wf
Button("Submit").primary.shake {
    on click(event) {
        if username == "" {
            replayAnimation(event.currentTarget, "shake")
        }
    }
}
```

`replayAnimation(node, name)` takes the node and the animation's name; the runtime removes and re-adds the animation class so it plays again.

---

## 6. Reduced Motion

When the reader's system asks for reduced motion (`prefers-reduced-motion: reduce`):

- The stylesheet cuts every animation and transition to 0.01ms.
- The runtime adds no animation class, plays no exit, and removes content at once; a keyed list does not slide its items; a route changes without a transition.

Nothing in a program needs to check for it.

---

## 7. Animation Design Tokens

Timing and easing come from design tokens, so a theme sets them once:

```
animation-duration-fast:    150ms
animation-duration-normal:  300ms
animation-duration-slow:    500ms

animation-easing-default:   cubic-bezier(0.4, 0, 0.2, 1)
animation-easing-bounce:    cubic-bezier(0.68, -0.55, 0.265, 1.55)
animation-easing-spring:    cubic-bezier(0.175, 0.885, 0.32, 1.275)
```

A theme overrides them like any token:

```wf
theme Brand {
    animation-duration-normal: 200ms
    animation-easing-default: ease-in-out
}
```

---

## 8. CSS Keyframe Definitions

All built-in animations are defined as CSS keyframes in the engine's stylesheet:

```css
@keyframes wf-fadeIn    { from { opacity: 0; } to { opacity: 1; } }
@keyframes wf-fadeOut   { from { opacity: 1; } to { opacity: 0; } }

@keyframes wf-slideUp   { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: none; } }
@keyframes wf-slideDown { from { opacity: 0; transform: translateY(-20px); } to { opacity: 1; transform: none; } }
@keyframes wf-slideLeft { from { opacity: 0; transform: translateX(20px); } to { opacity: 1; transform: none; } }
@keyframes wf-slideRight{ from { opacity: 0; transform: translateX(-20px); } to { opacity: 1; transform: none; } }

@keyframes wf-scaleIn   { from { opacity: 0; transform: scale(0.9); } to { opacity: 1; transform: none; } }
@keyframes wf-scaleOut  { from { opacity: 1; transform: none; } to { opacity: 0; transform: scale(0.9); } }

@keyframes wf-bounce    { 0% { opacity: 0; transform: scale(0.3); } 50% { transform: scale(1.05); } 70% { transform: scale(0.9); } 100% { opacity: 1; transform: none; } }
@keyframes wf-shake     { 0%, 100% { transform: none; } 10%, 30%, 50%, 70%, 90% { transform: translateX(-4px); } 20%, 40%, 60%, 80% { transform: translateX(4px); } }
@keyframes wf-pulse     { 0%, 100% { transform: scale(1); } 50% { transform: scale(1.05); } }
@keyframes wf-spin      { to { transform: rotate(360deg); } }
```

### Utility Classes

```css
.wf-animate-fadeIn    { animation: wf-fadeIn var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-slideUp   { animation: wf-slideUp var(--animation-duration-normal) var(--animation-easing-default) both; }
/* ... etc for all animations */

.wf-animate--fast     { animation-duration: var(--animation-duration-fast) !important; }
.wf-animate--slow     { animation-duration: var(--animation-duration-slow) !important; }
```

---

## 9. Compilation Output

### An enter animation → a class

```wf
Card.elevated.fadeIn { Text("Hello") }
```

```javascript
const _e0 = WF.el("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
```

### Motion on a branch root → the branch's config

```wf
if isVisible {
    Text("Hello", exit: .fadeOut).fadeIn
}
```

```javascript
WF.when(_root, () => _isVisible(), () => { /* the branch */ }, null, { enter: "fadeIn", exit: "fadeOut" });
```

A list passes the same config with its stagger and its key:

```javascript
WF.each(_root, () => _items(), (item, _idx) => { /* one item */ }, { enter: "slideUp", exit: "fadeOut", stagger: "50ms", key: (item) => item.id });
```

### Motion elsewhere → markers

Off a branch root, an element's exit and timing are written as `data-wf-*` attributes; the runtime reads them when the element first animates in and when it leaves:

```wf
Card { Text("x", exit: .fadeOut, delay: "100ms", easing: "ease-out") }
```

```javascript
const _e1 = WF.el("p", { className: "wf-text", "data-wf-exit": "fadeOut", "data-wf-delay": "100ms", "data-wf-easing": "ease-out" }, "x");
```

On a component, the markers land on its root element:

```javascript
const _e2 = Component_Chip({ label: "y" });
WF.mark(_e2, { "data-wf-exit": "fadeOut", "data-wf-animate": "fadeIn", "data-wf-duration": "150ms" });
```

### Transition blocks → an inline style

```wf
Button("Click") {
    transition {
        background: 200ms ease
        transform: 150ms spring
    }
}
```

```javascript
_e3.style.transition = "background 200ms ease, transform 150ms cubic-bezier(0.175, 0.885, 0.32, 1.275)";
```

### Route transitions → the router's options

```javascript
WF.router(_routes, _routerEl, { transition: "slide", duration: "150ms" });
```

### Replay → the runtime

```javascript
WF.replay(event.currentTarget, "shake");
```

---

## 10. Grammar

Motion adds nothing to the grammar: an enter animation is a flag, the rest are named props every element takes, and a transition block is a clause of an element (see `SYNTAX_V2.md`).

```ebnf
Flag            = "." IDENT ;                       // .fadeIn, .fast, .slow, …
MotionProp      = ("exit" ":" "." IDENT) | (("duration" | "delay" | "easing" | "stagger") ":" STRING) ;
TransitionBlock = "transition" "{" TransitionProp+ "}" ;
TransitionProp  = IDENT ":" Duration Easing? ;      // property: duration [easing]; either may be a $token
```
