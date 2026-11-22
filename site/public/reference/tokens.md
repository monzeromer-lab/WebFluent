# Design tokens

> Every baseline design token: its default value, what it is for, which built-ins draw from it, and how a style block or a stylesheet reads it.

The page: https://webfluent.monzeromer.dev/reference/tokens

This chapter lists every design token of the baseline theme with its value, and how a style reads one. A *design token* is a named value — a colour, a size, a duration — that every built-in is drawn with. Change a token and every element drawn with it changes.

A `theme` declaration changes the tokens it names and keeps the baseline value of every other one. A theme may also add tokens of its own. Each token becomes a CSS custom property on `:root`:

```wf
theme Brand {
    color-primary: #0F766E
    radius-md: 14px
    screen-md: 900px
    surface-raised: #F1F5F9
}
```

The build writes to `:root` only the tokens something in the output names, and the tokens those name in turn.

## Reading a token

A token has two spellings, one for each place a style is written:

| Where | Syntax | Example |
|---|---|---|
| A `style { }` block | `$name` | `background: $color-surface` |
| A `style { }` block, short form | `$suffix`, resolved by the property's group ([below](#short-names)) | `padding: $lg` is `spacing-lg` |
| A `.css` file under `src/` | `var(--name)` | `color: var(--color-text-muted)` |

This page reads the tokens of the theme above — one the theme adds, one it changes, one it keeps, and a breakpoint it moves:

```wf
theme Brand {
    color-primary: #0F766E
    radius-md: 14px
    screen-md: 900px
    surface-raised: #F1F5F9
}

page Pricing(path: "/pricing", title: "Pricing", description: "Plans and prices for every team.") {
    Heading("Pricing").h1
    Card {
        style {
            background: $surface-raised
            padding: $lg
            border-radius: $md
            border: 1px solid $border
            @md { padding: $xl }
        }
        Text("Team plan").bold
    }
}
```

The card's background is `#F1F5F9`, its radius `14px` and its border the baseline `color-border`. Its padding grows to `spacing-xl` from `900px` wide, because `@md` reads `screen-md`.

A `$token` the resolved theme does not declare draws the `V06` warning, since the property would fall back to nothing. `wf theme` lists every token the build uses, where each is read, and the contrast of the colour pairs ([Command line](https://webfluent.monzeromer.dev/reference/cli/theme.md)).

## Every built-in follows the theme

The built-ins draw their colours, sizes, radii, fonts and motion from these tokens and nothing else, so a theme restyles every one of them, the ones a page adds later included. A tone's label is a token too: `color-on-primary` is what a primary button, badge, pressed toggle, current page or selected day writes on `color-primary`, and so for `secondary`, `success`, `danger`, `warning` and `info`. A theme with a light primary names a dark `color-on-primary`, and `A13` holds each pair to WCAG AA where a page draws it. An alert's tint, a focus ring and a highlighted match are worked out from the tone and the page's colours (`color-mix`), so a dark theme gets dark alerts without naming any; `color-backdrop` dims the page behind a modal, a dialog, a sheet and a drawer. The only colours that stay fixed are the ones over a picture: a carousel's and a lightbox's controls.

## Short names

In a `style { }` block, a short name is resolved through the group of the property it is given to: `$lg` on `padding` is `spacing-lg`, and on `border-radius` it is `radius-lg`.

| Property | Group | Short names | Example |
|---|---|---|---|
| `padding`, `margin` and every `padding-*` and `margin-*` property (physical and logical), `gap`, `row-gap`, `column-gap`, `width`, `height`, `min-width`, `max-width`, `min-height`, `max-height`, `top`, `right`, `bottom`, `left`, `inset` and `inset-*` | spacing | `xs`, `sm`, `md`, `lg`, `xl`, `2xl`, `3xl` | `$lg` is `spacing-lg` |
| `color`, `background`, `background-color`, `border-color`, `fill`, `stroke`, `outline`, `outline-color`, `text-decoration-color`, `caret-color`, `accent-color`, and the `border` shorthands (`border`, `border-top` … `border-inline-end`) | colour | `primary`, `secondary`, `success`, `danger`, `warning`, `info`, `background`, `surface`, `text`, `text-muted`, `border` | `$primary` is `color-primary` |
| `font-size` | font size | `xs`, `sm`, `base`, `lg`, `xl`, `2xl`, `3xl` | `$xl` is `font-size-xl` |
| `line-height` | line height | `tight`, `normal`, `loose` | `$tight` is `line-height-tight` |
| `border-radius` | radius | `none`, `sm`, `md`, `lg`, `xl`, `full` | `$lg` is `radius-lg` |
| `box-shadow` | shadow | `none`, `sm`, `md`, `lg`, `xl` | `$md` is `shadow-md` |
| `transition` | transition | `fast`, `normal`, `slow` | `$fast` is `transition-fast` |

A short name your theme declares as a token of its own means that token, not the group's: with `md: 20px` in the theme, `$md` is `md` everywhere.

A full name — `$color-text-muted`, `$spacing-lg`, a token of your own — works in any property.

## The baseline

Every token a project starts with, by group, read from the compiler's baseline theme. A theme names only the ones it changes.

### Colour

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `color-backdrop` | `rgb(0 0 0 / 0.5)` | What dims the page behind a modal, a dialog, a sheet and a drawer | Sidebar, Modal, Dialog, Sheet |
| `color-background` | `#FFFFFF` | The page behind everything | Navbar, Tooltip, Input, Select, Switch, FileUpload, Alert, Toast +13 |
| `color-border` | `#E2E8F0` | A line between things: a field's edge, a divider, a card's border | Divider, Navbar, Sidebar, Breadcrumb, Menu, Popover, Tabs, Card +37 |
| `color-danger` | `#DC2626` | Failed, destructive, invalid: a danger button, an error message | Menu, Badge, Button, IconButton, Dropdown, Input, Alert, Toast +5 |
| `color-info` | `#0E7490` | For information: an info badge or alert, a note callout | Badge, Button, Alert, Toast, Icon, Callout, Markdown |
| `color-on-danger` | `#FFFFFF` | Text and icons written on `color-danger`; A13 checks the pair | Badge, Button, Toast |
| `color-on-info` | `#FFFFFF` | Text and icons written on `color-info`; A13 checks the pair | Badge, Button, Toast |
| `color-on-primary` | `#FFFFFF` | Text and icons written on `color-primary`; A13 checks the pair | Sidebar, Badge, Avatar, Button, Chip, Toggle, Pagination, Stepper +1 |
| `color-on-secondary` | `#FFFFFF` | Text and icons written on `color-secondary`; A13 checks the pair | Badge, Button |
| `color-on-success` | `#FFFFFF` | Text and icons written on `color-success`; A13 checks the pair | Badge, Button, Toast, Stepper |
| `color-on-warning` | `#000000` | Text and icons written on `color-warning`; A13 checks the pair | Badge, Button, Toast |
| `color-primary` | `#2563EB` | The main tone: a primary button, a link, a focus ring, what is chosen or current | Navbar, Sidebar, Link, Tabs, Badge, Avatar, Button, IconButton +35 |
| `color-secondary` | `#64748B` | A quieter tone, for a secondary button or badge | Badge, Button, Icon, Markdown |
| `color-success` | `#15803D` | Done, passing, saved: a success badge, alert or toast | Badge, Button, Alert, Toast, Icon, Callout, Stat, Stepper +1 |
| `color-surface` | `#F8FAFC` | What sits on the page: a card, a menu, a popover, a table header | Navbar, Sidebar, Menu, Popover, Card, Table, Badge, Tag +25 |
| `color-text` | `#0F172A` | The text | Navbar, Sidebar, Breadcrumb, Menu, Popover, Tabs, Badge, Tooltip +19 |
| `color-text-muted` | `#64748B` | Text that matters less: a hint, a caption, `.muted` | Navbar, Breadcrumb, Menu, Tabs, Table, List, Dropdown, Input +15 |
| `color-warning` | `#F59E0B` | Needs attention: a warning badge or alert, a rating's stars | Badge, Button, Alert, Toast, Icon, Callout, Rating, Markdown |

### Spacing

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `spacing-2xl` | `clamp(2rem, 1.5rem + 2.5vw, 3rem)` | A section's spacing, growing with the screen | — |
| `spacing-3xl` | `clamp(2.5rem, 1.75rem + 3.75vw, 4rem)` | A page's largest spacing, growing with the screen | — |
| `spacing-lg` | `1.5rem` | A large gap: `gap: .lg`, between sections | Grid, Spacer, Navbar, Menu, Button, Dropdown, Dialog, OnThisPage +3 |
| `spacing-md` | `1rem` | The usual gap: `gap: .md`, a card's padding | Grid, Spacer, Divider, Navbar, Sidebar, Menu, Tabs, Card +35 |
| `spacing-sm` | `0.5rem` | A small gap: `gap: .sm`, a control's padding | Grid, Spacer, Navbar, Sidebar, Menu, Popover, Tabs, Card +54 |
| `spacing-xl` | `clamp(1.5rem, 1.25rem + 1.25vw, 2rem)` | An extra-large gap, growing with the screen | Grid, Spacer, EmptyState, OnThisPage |
| `spacing-xs` | `0.25rem` | The smallest gap: `gap: .xs`, `$xs` in a padding | Grid, Spacer, Navbar, Sidebar, Breadcrumb, Menu, Tooltip, Tag +41 |

### Font size

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `font-size-2xl` | `clamp(1.5rem, 1.25rem + 1.25vw, 2rem)` | A heading (`h3`, `h2`), growing with the screen | — |
| `font-size-3xl` | `clamp(1.75rem, 1.38rem + 1.88vw, 2.5rem)` | The page's heading (`h1`), growing with the screen | — |
| `font-size-base` | `1rem` | Body text | Tabs, Button, IconButton, Input, Select, Checkbox, Radio |
| `font-size-lg` | `clamp(1.125rem, 1.06rem + 0.31vw, 1.25rem)` | Large text: `.lg`, a lede | Navbar, Sidebar, Avatar, Button, IconButton, Input, Alert, Toast +6 |
| `font-size-sm` | `0.875rem` | Small text: `.sm`, a hint, a table header | Navbar, Breadcrumb, Table, Avatar, Tag, Button, Input, Slider +16 |
| `font-size-xl` | `clamp(1.25rem, 1.13rem + 0.63vw, 1.5rem)` | A small heading (`h4`) | IconButton, Icon |
| `font-size-xs` | `0.75rem` | The smallest text: a badge, a caption | Sidebar, Menu, Badge, Avatar, Tooltip, Tag, Dropdown, Chip +1 |

### Line height

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `line-height-loose` | `1.75` | Line height for long reading | — |
| `line-height-normal` | `1.5` | Body text's line height | — |
| `line-height-tight` | `1.25` | Headings' line height | — |

### Radius

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `radius-full` | `9999px` | A pill or a circle: `.pill`, an avatar | Badge, Avatar, Button, Input, Switch, Progress, Chip |
| `radius-lg` | `1rem` | Rounder corners: a modal, a sheet | Button, Modal, Dialog, Sheet |
| `radius-md` | `0.5rem` | The usual corners: a button, a field, a card | Navbar, Sidebar, Menu, Popover, Card, Tag, Button, IconButton +26 |
| `radius-none` | `0` | Square corners | — |
| `radius-sm` | `0.25rem` | Slightly rounded: a badge, a checkbox, code | Tooltip, FileUpload, Modal, Carousel, Code, Kbd, Listbox, Tree +6 |
| `radius-xl` | `1.5rem` | Very round corners | — |

### Shadow

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `shadow-lg` | `0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1)` | A popover, a dropdown, a toast | Menu, Popover, Dropdown, Toast, NavigationMenu, HoverCard, Tour, ConsentBanner +3 |
| `shadow-md` | `0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1)` | `Card.elevated`, a menu | Card, Fab |
| `shadow-none` | `none` | No shadow | — |
| `shadow-sm` | `0 1px 2px 0 rgba(0,0,0,0.05)` | A slight lift: a button, a field | Switch, Sticky |
| `shadow-xl` | `0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1)` | A modal, a dialog, a sheet | Modal, Dialog, Sheet |

### Transition

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `transition-fast` | `150ms ease` | A quick change: a hover, a press | Navbar, Sidebar, Menu, Tabs, Button, IconButton, Dropdown, Input +8 |
| `transition-normal` | `250ms ease` | The usual change: an open, a slide | Sheet, Progress, Carousel, Accordion |
| `transition-slow` | `350ms ease` | A deliberate change | — |

### Font

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `font-family` | `system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif` | The text's typeface | Tabs, Button, Input, Select, Toast |
| `font-family-mono` | `ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace` | Code's typeface: `Code`, `CodeBlock`, `Kbd` | Code, CodeBlock, Kbd, ColorPicker, Markdown |
| `font-weight-bold` | `700` | `.bold`, a heading, a title | Navbar, Sidebar, Link, Table, Avatar, Toast, Callout, EmptyState +10 |
| `font-weight-medium` | `500` | A label's or a button's weight | Navbar, Breadcrumb, Tabs, Badge, Button, Input, Slider, Accordion +6 |
| `font-weight-normal` | `400` | Body text's weight | — |

### Motion

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `animation-duration-fast` | `150ms` | `.fast` and `speed: .fast` | Modal, Dialog |
| `animation-duration-normal` | `300ms` | An animation's length when it says nothing | — |
| `animation-duration-slow` | `500ms` | `.slow` and `speed: .slow` | — |
| `animation-easing-bounce` | `cubic-bezier(0.68, -0.55, 0.265, 1.55)` | `easing: .bouncy` | — |
| `animation-easing-default` | `cubic-bezier(0.4, 0, 0.2, 1)` | An animation's easing when it says nothing | — |
| `animation-easing-spring` | `cubic-bezier(0.175, 0.885, 0.32, 1.275)` | `easing: .spring`, sampled into `linear()` | — |
| `ease-in` | `cubic-bezier(0.4, 0, 1, 1)` | `easing: .easeIn`: something leaving | — |
| `ease-out` | `cubic-bezier(0, 0, 0.2, 1)` | `easing: .easeOut`: something arriving | — |
| `ease-spring` | `linear(0, 0.006, 0.079, 0.234, 0.44, 0.652, 0.828, 0.947, 1.008, 1.025, 1.016, 1)` | A spring that overshoots and settles | — |
| `ease-standard` | `cubic-bezier(0.2, 0, 0, 1)` | `easing: .standard`: the default for motion that moves | Modal, Dialog |

### Other

| Token | Value | What it is for | Built-ins that draw from it |
|---|---|---|---|
| `screen-lg` | `1024px` | Where `lg` starts: `{ lg: … }`, `@lg { }`, `viewport.lg` | — |
| `screen-md` | `768px` | Where `md` starts: `{ md: … }`, `@md { }`, `viewport.md` | — |
| `screen-sm` | `640px` | Where `sm` starts: `{ sm: … }`, `@sm { }`, `viewport.sm` | — |
| `screen-xl` | `1280px` | Where `xl` starts: `{ xl: … }`, `@xl { }`, `viewport.xl` | — |
| `syntax-comment` | `#959BA1` | `CodeBlock`'s comments | Highlight |
| `syntax-function` | `#6FD3E6` | `CodeBlock`'s calls and element names | Highlight |
| `syntax-keyword` | `#8FBDE7` | `CodeBlock`'s keywords | Highlight |
| `syntax-number` | `#E0B070` | `CodeBlock`'s numbers | Highlight |
| `syntax-punct` | `#B3BAC0` | `CodeBlock`'s punctuation | Highlight |
| `syntax-string` | `#7FD6A8` | `CodeBlock`'s strings | Highlight |
| `target-min` | `24px` | The least size of anything a pointer presses (WCAG 2.2's 24px) | Breadcrumb, Link, Menu, Tag, Checkbox, Radio, Switch, Slider +23 |
| `target-min-touch` | `44px` | The least size of anything a finger presses (44px) | BottomNavigation, Lightbox, Fab, BackToTop |
| `term-bg` | `#1B1F22` | A terminal block's background | Code, CodeBlock, Markdown |
| `term-dim` | `#959BA1` | A terminal block's quieter text: a prompt, a comment | CodeBlock, Highlight |
| `term-ink` | `#E6E9EC` | A terminal block's text | Code, CodeBlock, Markdown |
| `wf-header-height` | `56px` | How far a sticky element sits below the page's sticky header | Sidebar |
| `z-drawer` | `200` | Where a drawer, a `Sidebar` on a narrow screen, stacks | Sidebar |
| `z-dropdown` | `1000` | Where a dropdown not in the top layer stacks | NavigationMenu |
| `z-modal` | `1300` | Where a modal not in the top layer stacks | — |
| `z-skip-link` | `10000` | Where the skip link stacks, above everything | — |
| `z-sticky` | `100` | Where a sticky header or `Sticky` stacks | Navbar, AppShell, BottomNavigation, Sticky, Fab, BackToTop |
| `z-toast` | `1400` | Where the toasts stack | Toast, ConsentBanner |
| `z-tooltip` | `1500` | Where a tooltip not in the top layer stacks | — |
