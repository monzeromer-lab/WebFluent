# WebFluent Animation System Specification

> Version 1.0 — Draft
> Author: Monzer Omer
> Date: 2026-03-24

---

## Overview

WebFluent provides a **declarative animation system** that lets you add motion to your applications without writing CSS keyframes or JavaScript animation code. Animations are built into the language at four levels:

1. **Animation Modifiers** — one-word mount animations on any component
2. **Transition Blocks** — declarative CSS transitions on property changes
3. **Control Flow Animations** — enter/exit animations on `if`, `for`, and `show`
4. **Imperative Animations** — trigger animations from actions and event handlers

---

## 1. Animation Modifiers

Apply a built-in animation to any component. The animation plays when the element enters the DOM (mounts).

### Syntax

```
Component(modifiers..., <animationName>)
```

### Built-in Animations

| Modifier | Effect |
|----------|--------|
| `fadeIn` | Fade from 0 to full opacity |
| `fadeOut` | Fade from full opacity to 0 |
| `slideUp` | Slide in from below |
| `slideDown` | Slide in from above |
| `slideLeft` | Slide in from the right |
| `slideRight` | Slide in from the left |
| `scaleIn` | Scale up from 0 to full size |
| `scaleOut` | Scale down from full size to 0 |
| `bounce` | Bouncy entrance |
| `shake` | Horizontal shake |
| `pulse` | Gentle scale pulse |
| `spin` | 360° rotation |

### Duration Variants

Combine with a speed modifier:

| Modifier | Duration |
|----------|----------|
| `fast` | 150ms |
| *(default)* | 300ms |
| `slow` | 500ms |

### Examples

```
// Fade in a card on mount
Card(elevated, fadeIn) {
    Text("Welcome!")
}

// Slide up with slow speed
Heading("Hello", h1, slideUp, slow)

// Bounce a button
Button("Click me", primary, bounce)

// Multiple elements with different animations
Container {
    Heading("Dashboard", h1, fadeIn)
    Row(slideUp) {
        Card(scaleIn) { Text("Users: 42") }
        Card(scaleIn) { Text("Revenue: $1.2k") }
    }
}
```

---

## 2. Transition Blocks

Declare CSS transitions on a component. These apply smooth interpolation when reactive properties change (hover states, style changes, class toggling).

### Syntax

```
Component {
    transition {
        <property> <duration> [easing]
        ...
    }
}
```

### Properties

Any CSS property name, or `all` for all properties.

### Easing Functions

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

### Examples

```
// Smooth background and color transitions
Button("Hover me", primary) {
    transition {
        background 200ms ease
        color 200ms ease
        transform 150ms spring
    }
}

// Transition all properties
Card(elevated) {
    transition {
        all 300ms smooth
    }
    style {
        background: surface
    }
}
```

---

## 3. Control Flow Animations

Add enter/exit animations to conditional rendering, lists, and show/hide.

### Syntax

```
// On if/else
if condition, animate(<enter>, <exit>) {
    ...
}

// On for loops (with optional stagger)
for item in list, animate(<enter>, <exit>, stagger: "<delay>") {
    ...
}

// On show/hide
show condition, animate(<enter>, <exit>) {
    ...
}
```

The `animate(...)` clause is **optional** — if omitted, elements appear/disappear instantly (current behavior).

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| 1st positional | Animation name | Yes | Enter animation |
| 2nd positional | Animation name | No | Exit animation (reverse of enter if omitted) |
| `duration` | String | No | Override default duration (e.g., `"500ms"`) |
| `delay` | String | No | Delay before animation starts |
| `stagger` | String | No | Delay between each item (for loops only) |
| `easing` | String | No | Easing function name |

### Enter/Exit Behavior

- **`if`**: When condition becomes true, enter animation plays. When false, exit animation plays, then elements are removed.
- **`for`**: When items are added, enter animation plays on new items. When items are removed, exit animation plays, then elements are removed. Stagger delays each item.
- **`show`**: Similar to `if`, but elements stay in the DOM. Enter animation plays on show, exit animation plays on hide, then `display: none` is set.

### Auto Exit Animations

If only one animation is provided, the exit animation is the **reverse** of the enter:

| Enter | Auto Exit |
|-------|-----------|
| `fadeIn` | `fadeOut` |
| `slideUp` | `slideDown` |
| `slideDown` | `slideUp` |
| `slideLeft` | `slideRight` |
| `slideRight` | `slideLeft` |
| `scaleIn` | `scaleOut` |
| `bounce` | `fadeOut` |

### Examples

```
// Fade a message in/out based on state
if showMessage, animate(fadeIn, fadeOut) {
    Alert("Operation successful!", success)
}

// Slide cards up with stagger
for product in products, animate(slideUp, fadeOut, stagger: "50ms") {
    Card(elevated) {
        Text(product.name, heading)
        Text("${product.price}")
    }
}

// Scale a modal
show isModalOpen, animate(scaleIn, scaleOut, duration: "200ms") {
    Modal(title: "Confirm") {
        Text("Are you sure?")
        Button("Yes", primary) { confirm() }
        Button("No") { isModalOpen = false }
    }
}

// Custom duration and easing
if isExpanded, animate(slideDown, slideUp, duration: "400ms", easing: "spring") {
    Stack(gap: md) {
        Text("Expanded content line 1")
        Text("Expanded content line 2")
        Text("Expanded content line 3")
    }
}

// No exit animation — just fade in, instant removal
if hasData, animate(fadeIn) {
    Table { ... }
}
```

---

## 4. Imperative Animations

Trigger animations from actions, event handlers, and effects. Useful for feedback animations (shake on error, pulse on update).

### Syntax

```
animate(<target>, <animationName>)
animate(<target>, <animationName>, <duration>)
```

Where `<target>` is an identifier referencing a component (via `ref`).

### Component References

Use `ref` to name a component for imperative animation:

```
Button("Submit", primary, ref: submitBtn)

action onError() {
    animate(submitBtn, shake)
}
```

### Examples

```
Page Form (path: "/form") {
    state error = false

    Form {
        Input(text, bind: username, ref: usernameInput)
        Button("Submit", primary, ref: submitBtn) {
            if username == "" {
                error = true
                animate(usernameInput, shake)
            } else {
                submitForm()
            }
        }
    }
}
```

---

## 5. Animation Design Tokens

The animation system uses design tokens for consistent timing and easing across the app.

### Tokens

```
animation-duration-fast:    150ms
animation-duration-normal:  300ms
animation-duration-slow:    500ms

animation-easing-default:   cubic-bezier(0.4, 0, 0.2, 1)
animation-easing-bounce:    cubic-bezier(0.68, -0.55, 0.265, 1.55)
animation-easing-spring:    cubic-bezier(0.175, 0.885, 0.32, 1.275)
```

These can be overridden in `webfluent.app.json`:

```json
{
    "theme": {
        "tokens": {
            "animation-duration-normal": "200ms",
            "animation-easing-default": "ease-in-out"
        }
    }
}
```

---

## 6. CSS Keyframe Definitions

All built-in animations are defined as CSS keyframes:

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
```

### Utility Classes

```css
.wf-animate-fadeIn    { animation: wf-fadeIn var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-slideUp   { animation: wf-slideUp var(--animation-duration-normal) var(--animation-easing-default) both; }
/* ... etc for all animations */

.wf-animate--fast     { animation-duration: var(--animation-duration-fast); }
.wf-animate--slow     { animation-duration: var(--animation-duration-slow); }
```

---

## 7. Compilation Output

### Modifier animations → CSS classes

```wf
Card(elevated, fadeIn) { Text("Hello") }
```

Compiles to:

```javascript
const _e0 = WF.h("div", { className: "wf-card wf-card--elevated wf-animate-fadeIn" });
```

### Control flow animations → runtime config

```wf
if isVisible, animate(fadeIn, fadeOut) {
    Text("Hello")
}
```

Compiles to:

```javascript
WF.condRender(_root,
  () => _isVisible(),
  () => { /* then branch */ },
  null,
  { enter: "fadeIn", exit: "fadeOut" }
);
```

### Transition blocks → inline styles

```wf
Button("Click") {
    transition {
        background 200ms ease
        transform 150ms spring
    }
}
```

Compiles to:

```javascript
const _e0 = WF.h("button", {
  className: "wf-btn",
  style: { transition: "background 200ms ease, transform 150ms cubic-bezier(0.175, 0.885, 0.32, 1.275)" }
});
```

---

## 8. Grammar Additions (EBNF)

```ebnf
// Updated control flow with optional animate clause
IfStmt     = "if" Expression ("," AnimateClause)? Block ("else" "if" Expression Block)* ("else" Block)? ;
ForStmt    = "for" IDENT ("," IDENT)? "in" Expression ("," AnimateClause)? Block ;
ShowStmt   = "show" Expression ("," AnimateClause)? Block ;

// Animate clause
AnimateClause = "animate" "(" AnimationName ("," AnimationName)? ("," AnimateOpt)* ")" ;
AnimateOpt    = IDENT ":" Expression ;
AnimationName = IDENT ;   // fadeIn, slideUp, scaleIn, bounce, shake, pulse, spin, etc.

// Transition block (inside UI elements)
TransitionBlock = "transition" "{" TransitionProp+ "}" ;
TransitionProp  = IDENT Duration IDENT? ;   // property duration [easing]

// Imperative animate statement
AnimateStmt = "animate" "(" IDENT "," AnimationName ("," Duration)? ")" ;

// Animation modifiers (added to existing modifier list)
AnimationModifier = "fadeIn" | "fadeOut" | "slideUp" | "slideDown" | "slideLeft" | "slideRight"
                  | "scaleIn" | "scaleOut" | "bounce" | "shake" | "pulse" | "spin" ;
```
