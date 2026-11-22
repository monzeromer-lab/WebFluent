### Component Details

#### Sidebar

Structural codegen with sub-components for app navigation:

```wf
Sidebar {
    Sidebar.Header { Text("My App").heading }
    Sidebar.Item(to: "/", icon: "home") { Text("Home") }
    Sidebar.Item(to: "/settings", icon: "settings") { Text("Settings") }
    Sidebar.Divider
    Sidebar.Item(to: "/logout", icon: "logout") { Text("Logout") }
}
```

Sub-components: `Sidebar.Header`, `Sidebar.Item`, `Sidebar.Divider`

A `Sidebar.Item` whose `to` matches the current route is marked `.active` +
`aria-current="page"`, like a `Link`; `active: .prefix` matches sub-routes.

#### Breadcrumb

Navigation breadcrumb with proper separator rendering:

```wf
Breadcrumb {
    Breadcrumb.Item(to: "/") { Text("Home") }
    Breadcrumb.Item(to: "/docs") { Text("Docs") }
    Breadcrumb.Item { Text("Components") }
}
```

The last item (without `to:`) renders as the current page (no link).

#### Tooltip

Wraps children, shows text on hover after `delay:` ms (unset, 500) and at
once on keyboard focus, above everything; `shortcut:` adds a chip. Escape
hides it. While focus is on its control the key is the tooltip's and goes
no further; otherwise it goes on — to the `Popover` or `Menu` below, which
it closes, or to the element with focus — so a tooltip under a resting
pointer never takes an editor's or a field's Escape:

```wf
Tooltip("Click to save", shortcut: "Ctrl+S") {
    Button("Save").primary
}
```

#### Avatar

Displays an image or initials:

```wf
Avatar(src: "/photo.jpg", alt: "User")
Avatar(initials: "MO").primary
Avatar(initials: "J").lg
```

Flags: `.sm`, `.md`, `.lg`, `.primary`

#### Skeleton

Loading placeholder:

```wf
Skeleton(height: "20px", width: "200px")
Skeleton(size: "48px").circle
```

Flags: `.circle`

#### Carousel

Slide track with dots and autoplay:

```wf
Carousel(autoplay: true, interval: 5000) {
    Carousel.Slide { Image(src: "/1.jpg", alt: "Slide 1") }
    Carousel.Slide { Image(src: "/2.jpg", alt: "Slide 2") }
}
```

#### IconButton

Icon-only button with aria-label:

```wf
IconButton(icon: "close", label: "Close")
IconButton(icon: "edit", label: "Edit").primary { on click { editItem() } }
IconButton(icon: "menu", label: "Actions", aria-haspopup: "menu", aria-expanded: open) { on click { toggleMenu() } }
```

Flags: `.sm`, `.lg`, `.primary`, `.danger`. The `label` is the accessible name (`aria-label`, and the `title` unless a `title:` is written), never visible text; every other named argument (`type`, `disabled`, `aria-*`, `data-*`) is an attribute as on `Button`.

#### Slider

Range input with reactive value:

```wf
Slider(bind: volume, min: 0, max: 100, step: 1, label: "Volume")
```

#### DatePicker

Date input. The bound state holds a `Date` (`F02` otherwise):

```wf
state selectedDate: Date? = null
DatePicker(bind: selectedDate, label: "Start Date", min: "2026-01-01")
```

#### FileUpload

Styled file input:

```wf
FileUpload(accept: "image/*", label: "Upload Photo") {
    on change(event) { handleFile(event) }
}
FileUpload(accept: ".pdf,.doc", label: "Documents").multiple
```

