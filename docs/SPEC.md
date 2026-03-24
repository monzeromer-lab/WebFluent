# WebFluent Language Specification

> Version 0.1.0 — Draft
> Author: Monzer Omer
> Date: 2026-03-24

---

## Table of Contents

1. [Lexical Structure](#1-lexical-structure)
2. [Types](#2-types)
3. [Expressions](#3-expressions)
4. [Statements](#4-statements)
5. [Pages](#5-pages)
6. [Components](#6-components)
7. [State & Reactivity](#7-state--reactivity)
8. [Events](#8-events)
9. [Control Flow](#9-control-flow)
10. [Routing](#10-routing)
11. [Data Fetching](#11-data-fetching)
12. [Stores](#12-stores)
13. [Styling](#13-styling)
14. [Built-in Components Reference](#14-built-in-components-reference)
15. [Design System](#15-design-system)
16. [Project Configuration](#16-project-configuration)
17. [Compilation Output](#17-compilation-output)

---

## 1. Lexical Structure

### 1.1 Character Set

WebFluent source files are UTF-8 encoded.

### 1.2 Comments

```
// Single-line comment

/*
   Multi-line comment
*/
```

### 1.3 Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores. Component and Page names must start with an uppercase letter.

```
validName
_private
UserCard        // Component/Page name (PascalCase)
userName        // variable/state (camelCase)
```

### 1.4 Reserved Keywords

```
Page            Component       Store           App
Router          Route           Link

state           action          use             fetch
if              else            for             in
show            navigate        value

on:click        on:submit       on:input        on:change
on:focus        on:blur         on:keydown      on:keyup
on:mouseover    on:mouseout     on:mount        on:unmount

true            false           null

style           loading         error           success

String          Number          Bool            List
Map
```

### 1.5 Literals

#### String Literals
```
"Hello, world"                          // Plain string
"Hello, {userName}"                     // String interpolation with { }
```

#### Number Literals
```
42                                      // Integer
3.14                                    // Float
```

#### Boolean Literals
```
true
false
```

#### Null Literal
```
null
```

#### List Literals
```
[1, 2, 3]
["apple", "banana", "cherry"]
[UserCard(name: "Ali"), UserCard(name: "Sara")]
```

#### Map Literals
```
{ name: "Monzer", age: 25, active: true }
```

### 1.6 Operators

| Operator | Description |
|----------|-------------|
| `+` | Addition / String concatenation |
| `-` | Subtraction |
| `*` | Multiplication |
| `/` | Division |
| `%` | Modulo |
| `=` | Assignment |
| `==` | Equality |
| `!=` | Inequality |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |
| `&&` | Logical AND |
| `||` | Logical OR |
| `!` | Logical NOT |
| `.` | Property access |

### 1.7 Punctuation

| Symbol | Usage |
|--------|-------|
| `( )` | Attribute lists, function calls |
| `{ }` | Blocks (children, event handlers, maps) |
| `[ ]` | List literals, index access |
| `:` | Key-value separator |
| `,` | Separator in lists and attributes |

### 1.8 Token Types

```
// Keywords
Page, Component, Store, App, Router, Route, Link,
state, action, use, fetch, if, else, for, in, show, navigate, style

// Identifiers & Literals
Identifier, StringLiteral, NumberLiteral, BoolLiteral, Null

// Built-in Components (see Section 14)
Container, Row, Column, Grid, Stack, Spacer, Divider,
Navbar, Sidebar, Breadcrumb, Menu, Tabs, TabPage,
Card, Table, Thead, Tbody, Trow, Tcell, List, Badge, Avatar, Tooltip, Tag,
Input, Select, Checkbox, Radio, Switch, Slider, DatePicker, FileUpload, Form,
Alert, Toast, Modal, Dialog, Spinner, Progress, Skeleton,
Button, IconButton, ButtonGroup, Dropdown,
Image, Video, Icon, Carousel,
Text, Heading, Code, Blockquote

// Operators
Plus, Minus, Star, Slash, Percent,
Equals, DoubleEquals, NotEquals,
LessThan, GreaterThan, LessEquals, GreaterEquals,
And, Or, Not,
Dot

// Punctuation
OpenParen, CloseParen, OpenBrace, CloseBrace,
OpenBracket, CloseBracket, Colon, Comma

// Events
Event  // on:click, on:submit, etc.

// Special
EOF
```

---

## 2. Types

WebFluent has a simple type system. Types are inferred from values but can be explicitly annotated on component props.

### 2.1 Primitive Types

| Type | Description | Examples |
|------|-------------|----------|
| `String` | Text values | `"hello"`, `"Hello, {name}"` |
| `Number` | Integer or float | `42`, `3.14` |
| `Bool` | Boolean | `true`, `false` |
| `null` | Absence of value | `null` |

### 2.2 Collection Types

| Type | Description | Examples |
|------|-------------|----------|
| `List` | Ordered collection | `[1, 2, 3]` |
| `Map` | Key-value pairs | `{ name: "Ali", age: 25 }` |

### 2.3 Type Inference

State variables and local variables infer their type from the assigned value:

```
state count = 0                  // inferred as Number
state name = "Monzer"            // inferred as String
state items = []                 // inferred as List
state visible = true             // inferred as Bool
```

### 2.4 Prop Type Annotations

Component props require explicit types:

```
Component UserCard (name: String, age: Number, active: Bool) {
    // ...
}
```

Optional props use `?`:

```
Component UserCard (name: String, avatar?: String) {
    // avatar defaults to null if not provided
}
```

Default values:

```
Component Button (label: String, variant: String = "primary") {
    // ...
}
```

---

## 3. Expressions

### 3.1 Arithmetic

```
count + 1
price * quantity
total / items.length
index % 2
```

### 3.2 String Interpolation

Curly braces inside strings evaluate expressions:

```
"Hello, {user.name}!"
"Total: {price * quantity} USD"
"Item {index + 1} of {items.length}"
```

### 3.3 Comparison

```
count > 0
name == "admin"
age >= 18
status != "inactive"
```

### 3.4 Logical

```
isLoggedIn && isAdmin
showA || showB
!isHidden
```

### 3.5 Property Access

```
user.name
items.length
response.data.users
```

### 3.6 Index Access

```
items[0]
matrix[row][col]
```

### 3.7 Method Calls

Built-in methods on types:

```
// List methods
items.push(newItem)
items.remove(index)
items.filter(item => item.active)
items.map(item => item.name)
items.length

// String methods
name.toUpper()
name.toLower()
name.contains("search")
name.trim()
name.split(",")
```

---

## 4. Statements

### 4.1 Variable Declaration (state)

```
state count = 0
state users = []
state form = { name: "", email: "" }
```

### 4.2 Assignment

```
count = count + 1
user.name = "New Name"
items[0] = "Updated"
```

### 4.3 Function-Like Actions

Actions are named blocks of logic:

```
action increment() {
    count = count + 1
}

action addUser(name: String) {
    users.push({ name: name, active: true })
}
```

### 4.4 Log (Debug)

```
log("Current count: {count}")
log(user)
```

---

## 5. Pages

### 5.1 Page Declaration

```
Page <Name> (path: "<route>") {
    <body>
}
```

**Grammar:**

```
PageDecl     = "Page" Identifier "(" PageAttrs ")" "{" PageBody "}"
PageAttrs    = "path" ":" StringLiteral ("," PageAttr)*
PageAttr     = Identifier ":" Expression
PageBody     = (StateDecl | ActionDecl | UIElement | ControlFlow)*
```

### 5.2 Page Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | String | Yes | URL route for this page |
| `title` | String | No | Document title (`<title>` tag) |
| `guard` | Expression | No | Navigation guard (redirects if false) |

### 5.3 Examples

```
Page Home (path: "/", title: "Home") {
    Heading("Welcome to WebFluent", h1)
    Text("Build SPAs with ease.")
}

Page Dashboard (path: "/dashboard", guard: AuthStore.isLoggedIn) {
    state activeTab = "overview"

    Navbar {
        Text("Dashboard", heading)
    }

    Tabs {
        TabPage("Overview") {
            Text("Overview content")
        }
        TabPage("Analytics") {
            Text("Analytics content")
        }
    }
}
```

### 5.4 Dynamic Routes

```
Page UserProfile (path: "/user/:id") {
    // 'params.id' is automatically available
    fetch user from "/api/users/{params.id}" {
        success {
            Heading(user.name, h1)
            Text(user.bio)
        }
    }
}
```

---

## 6. Components

### 6.1 Component Declaration

```
Component <Name> (<props>) {
    <body>
}
```

**Grammar:**

```
ComponentDecl = "Component" Identifier "(" PropList ")" "{" ComponentBody "}"
PropList      = (PropDecl ("," PropDecl)*)?
PropDecl      = Identifier ("?")? ":" Type ("=" Expression)?
ComponentBody = (StateDecl | ActionDecl | UIElement | ControlFlow)*
```

### 6.2 Props

```
Component ProductCard (
    name: String,
    price: Number,
    image?: String,
    onSale: Bool = false
) {
    Card {
        if image != null {
            Image(src: image, alt: name)
        }
        Text(name, heading)
        Text("${price}", subtitle)
        if onSale {
            Badge("Sale", success)
        }
    }
}
```

### 6.3 Using Components

```
// Positional arguments (matching prop order)
ProductCard("Laptop", 999)

// Named arguments
ProductCard(name: "Laptop", price: 999, onSale: true)

// With children (if component accepts children)
Card {
    Text("I am a child element")
}
```

### 6.4 Component Children (Slots)

Components can accept children using the `children` keyword:

```
Component Panel (title: String) {
    Card {
        Heading(title, h3)
        Divider()
        children
    }
}

// Usage
Panel(title: "Settings") {
    Input(text, placeholder: "Username")
    Input(email, placeholder: "Email")
    Button("Save", primary)
}
```

### 6.5 Component Internal State

Components can have their own private state:

```
Component Toggle (label: String, initial: Bool = false) {
    state isOn = initial

    Row {
        Text(label)
        Switch(checked: isOn) {
            on:change { isOn = !isOn }
        }
    }
}
```

---

## 7. State & Reactivity

### 7.1 State Declaration

State is declared with the `state` keyword. State is **reactive** — any UI that references it will automatically update when it changes.

```
state count = 0
state message = "Hello"
state items = ["Apple", "Banana"]
state user = { name: "Monzer", loggedIn: true }
```

### 7.2 Reactivity Model

WebFluent uses a **signal-based reactivity system**. Each `state` declaration creates a signal. UI elements that read a signal automatically subscribe to it.

```
Page Counter (path: "/counter") {
    state count = 0                          // Signal created

    Text("Count: {count}")                   // Subscribes to 'count'
    Button("+1") { count = count + 1 }       // Updates signal → Text re-renders
}
```

**Rules:**
- State mutations trigger synchronous UI updates
- Only the DOM nodes that read the changed state are updated (fine-grained)
- State is scoped to the Page or Component that declares it
- For cross-component state, use Stores (Section 12)

### 7.3 Derived State (Computed)

Derived values that automatically recompute when their dependencies change:

```
state items = [
    { name: "Apples", price: 3 },
    { name: "Bread", price: 2 }
]

derived total = items.map(i => i.price).sum()
derived itemCount = items.length
derived isEmpty = itemCount == 0
```

### 7.4 Effects

Side effects that run when dependencies change:

```
state searchQuery = ""

effect {
    log("Search changed to: {searchQuery}")
}
```

---

## 8. Events

### 8.1 Event Syntax

Events are declared with `on:<event>` inside a component block:

```
Button("Click Me") {
    on:click {
        count = count + 1
    }
}
```

### 8.2 Supported Events

| Event | Applies To | Description |
|-------|-----------|-------------|
| `on:click` | Any element | Click/tap |
| `on:submit` | Form | Form submission |
| `on:input` | Input, Select | Value changed (real-time) |
| `on:change` | Input, Select, Checkbox, Switch | Value committed |
| `on:focus` | Input, Select | Element focused |
| `on:blur` | Input, Select | Element lost focus |
| `on:keydown` | Input | Key pressed |
| `on:keyup` | Input | Key released |
| `on:mouseover` | Any element | Mouse entered |
| `on:mouseout` | Any element | Mouse left |
| `on:mount` | Any element | Element added to DOM |
| `on:unmount` | Any element | Element removed from DOM |

### 8.3 Event Object

Events provide contextual data via implicit variables:

```
Input(text, placeholder: "Type here") {
    on:input {
        searchQuery = value              // 'value' is the current input value
    }
    on:keydown {
        if key == "Enter" {              // 'key' is the pressed key
            performSearch()
        }
    }
}

Form {
    on:submit {
        // 'event' prevents default automatically
        processForm()
    }
}
```

### 8.4 Inline Event Shorthand

For simple events, use the shorthand block directly on the element:

```
// These two are equivalent:

// Shorthand
Button("Add") { items.push("New Item") }

// Explicit
Button("Add") {
    on:click {
        items.push("New Item")
    }
}
```

When a `Button` or `Link` has a block with no `on:` prefix, it defaults to `on:click`.

---

## 9. Control Flow

### 9.1 Conditional Rendering (if / else)

```
if condition {
    // rendered when condition is true
}

if condition {
    // true branch
} else {
    // false branch
}

if role == "admin" {
    AdminPanel()
} else if role == "editor" {
    EditorPanel()
} else {
    ViewerPanel()
}
```

**Behavior:** Elements are **created/destroyed** based on the condition. When the condition changes, the old branch is removed from the DOM and the new branch is inserted.

### 9.2 List Rendering (for)

```
for item in items {
    Text(item)
}

for user in users {
    UserCard(name: user.name, role: user.role)
}

// With index
for item, index in items {
    Text("{index + 1}. {item}")
}
```

**Behavior:** The loop body is re-rendered when the list changes. Items are keyed by identity for efficient updates.

### 9.3 Show/Hide

```
show condition {
    // content stays in DOM, visibility toggled via CSS
}
```

**Difference from `if`:** `show` uses `display: none` to hide elements. The element remains in the DOM and preserves its state. Use `show` for frequently toggled content (like dropdowns, tooltips) and `if` for content that changes rarely.

---

## 10. Routing

### 10.1 App & Router Declaration

The `App.wf` file is the entry point. It defines the router and global layout:

```
App {
    Navbar {
        Link(to: "/") { Text("Home") }
        Link(to: "/about") { Text("About") }
        Link(to: "/dashboard") { Text("Dashboard") }
    }

    Router {
        Route(path: "/", page: Home)
        Route(path: "/about", page: About)
        Route(path: "/dashboard", page: Dashboard)
        Route(path: "/user/:id", page: UserProfile)
        Route(path: "*", page: NotFound)
    }

    Footer {
        Text("© 2026 My App")
    }
}
```

### 10.2 Route Parameters

Dynamic segments in paths start with `:`:

```
Route(path: "/user/:id", page: UserProfile)
Route(path: "/post/:slug", page: PostDetail)
Route(path: "/category/:catId/product/:prodId", page: ProductDetail)
```

Parameters are available via `params`:

```
Page UserProfile (path: "/user/:id") {
    Text("User ID: {params.id}")
}
```

### 10.3 Navigation

```
// Declarative (renders as <a> tag)
Link(to: "/about") {
    Text("About Page")
}

// Programmatic
navigate("/dashboard")
navigate("/user/{userId}")

// Back/Forward
navigate.back()
navigate.forward()
```

### 10.4 Navigation Guards

```
Page AdminPanel (path: "/admin", guard: AuthStore.isAdmin) {
    // Redirects to "/" if guard evaluates to false
}

// Custom redirect
Page AdminPanel (path: "/admin", guard: AuthStore.isAdmin, redirect: "/login") {
    // Redirects to "/login" if not admin
}
```

### 10.5 Active Route

```
// Link automatically gets an 'active' class when its route matches
Link(to: "/about") { Text("About") }

// Check current route in code
if route.path == "/dashboard" {
    Text("You are on the dashboard")
}
```

---

## 11. Data Fetching

### 11.1 Fetch Declaration

```
fetch <variable> from "<url>" {
    loading {
        // Shown while request is in flight
    }
    error (err) {
        // Shown on failure, 'err' contains error info
    }
    success {
        // Shown on success, '<variable>' contains response data
    }
}
```

### 11.2 Fetch Options

```
fetch result from "/api/users" (
    method: "GET",                       // GET (default), POST, PUT, DELETE, PATCH
    headers: { "Authorization": "Bearer {token}" },
    body: { name: "Ali", email: "ali@example.com" },
    cache: true,                         // Cache the response
    retry: 3                             // Retry on failure
) {
    loading { Spinner() }
    error (err) { Alert(err.message, danger) }
    success {
        for user in result {
            Text(user.name)
        }
    }
}
```

### 11.3 Refetching

```
state page = 1

// Fetch re-runs whenever 'page' changes (reactive dependency)
fetch users from "/api/users?page={page}" {
    success {
        for user in users {
            UserCard(name: user.name)
        }
    }
}

Button("Next Page") { page = page + 1 }
```

### 11.4 Imperative Fetch (inside actions)

```
action submitForm() {
    fetch result from "/api/submit" (method: "POST", body: formData) {
        success {
            Toast("Saved successfully!", success)
            navigate("/dashboard")
        }
        error (err) {
            Toast(err.message, danger)
        }
    }
}
```

---

## 12. Stores

### 12.1 Store Declaration

Stores hold shared state accessible from any page or component:

```
Store <Name> {
    state <name> = <value>
    ...

    derived <name> = <expression>
    ...

    action <name>(<params>) {
        <body>
    }
    ...
}
```

### 12.2 Example Store

```
Store CartStore {
    state items = []

    derived total = items.map(i => i.price * i.quantity).sum()
    derived count = items.length
    derived isEmpty = count == 0

    action addItem(product: Map) {
        state existing = items.filter(i => i.id == product.id)
        if existing.length > 0 {
            existing[0].quantity = existing[0].quantity + 1
        } else {
            items.push({ id: product.id, name: product.name, price: product.price, quantity: 1 })
        }
    }

    action removeItem(id: Number) {
        items = items.filter(i => i.id != id)
    }

    action clear() {
        items = []
    }
}
```

### 12.3 Using Stores

```
Page Cart (path: "/cart") {
    use CartStore

    if CartStore.isEmpty {
        Text("Your cart is empty")
        Link(to: "/products") { Button("Browse Products") }
    } else {
        for item in CartStore.items {
            Row {
                Text(item.name)
                Text("x{item.quantity}")
                Text("${item.price * item.quantity}")
                Button("Remove", danger, small) { CartStore.removeItem(item.id) }
            }
        }
        Divider()
        Text("Total: ${CartStore.total}", heading)
        Button("Clear Cart", danger) { CartStore.clear() }
    }
}
```

---

## 13. Styling

### 13.1 Variant Modifiers

All built-in components accept modifier keywords for common styling:

**Size:** `small`, `medium` (default), `large`
**Color:** `primary`, `secondary`, `success`, `danger`, `warning`, `info`
**Shape:** `rounded`, `pill`, `square`
**Elevation:** `flat`, `elevated`, `outlined`
**Width:** `full` (full-width), `fit` (fit-content)
**Text:** `bold`, `italic`, `underline`, `uppercase`, `lowercase`
**Alignment:** `left`, `center`, `right`

```
Button("Save", primary, large, rounded)
Text("Warning!", danger, bold, uppercase)
Card(elevated, rounded) { ... }
Input(text, full, rounded)
```

### 13.2 Style Blocks

For custom styling beyond modifiers:

```
Button("Custom") {
    style {
        background: "#8B5CF6"
        color: "#FFFFFF"
        padding: xl
        radius: lg
        shadow: md
        font-size: lg
    }
}
```

**Token references** — Style values can reference design tokens by name:

```
style {
    background: primary            // References color-primary token
    padding: md                    // References spacing-md token
    radius: lg                     // References radius-lg token
    shadow: sm                     // References shadow-sm token
    font-size: xl                  // References font-size-xl token
}
```

**Raw CSS values** — Or use raw CSS values directly:

```
style {
    background: "#custom"
    padding: "2rem"
    border: "1px solid #ccc"
    width: "300px"
}
```

### 13.3 Responsive Modifiers

```
// Breakpoint-specific modifiers
Column(span: 12, md: 6, lg: 4) {
    // Full width on mobile, half on medium, third on large
}

// Responsive visibility
show screen.md {
    Sidebar { ... }
}
```

### 13.4 Scoped Styles

Styles defined in a component only affect that component — no leaking:

```
Component MyButton (label: String) {
    Button(label) {
        style {
            background: "#FF6B6B"
        }
    }
    // This style does NOT affect Button elements outside this component
}
```

---

## 14. Built-in Components Reference

### 14.1 Layout

#### Container
Centered content container with max-width.
```
Container {
    // Content is centered with responsive max-width
}
Container(fluid) {
    // Full-width container
}
```

#### Row
Horizontal flex container.
```
Row {
    Column { ... }
    Column { ... }
}
Row(gap: md, align: center, justify: between) {
    // With spacing and alignment
}
```

#### Column
Flex child with optional span (12-column grid).
```
Column {
    // Auto width
}
Column(span: 6) {
    // Half width (6/12)
}
Column(span: 12, md: 6, lg: 4) {
    // Responsive: full → half → third
}
```

#### Grid
CSS Grid container.
```
Grid(columns: 3, gap: md) {
    Card { ... }
    Card { ... }
    Card { ... }
}
```

#### Stack
Vertical flex container with consistent spacing.
```
Stack(gap: md) {
    Text("First")
    Text("Second")
    Text("Third")
}
```

#### Spacer
Empty space.
```
Spacer()              // Default spacing
Spacer(xl)            // Extra-large space
```

#### Divider
Horizontal line separator.
```
Divider()
Divider(label: "OR")  // Divider with centered label
```

---

### 14.2 Navigation

#### Navbar
Top navigation bar.
```
Navbar {
    Navbar.Brand {
        Text("MyApp", heading)
    }
    Navbar.Links {
        Link(to: "/") { Text("Home") }
        Link(to: "/about") { Text("About") }
    }
    Navbar.Actions {
        Button("Sign In", primary)
    }
}
```

#### Sidebar
Side navigation panel.
```
Sidebar {
    Sidebar.Header {
        Text("Menu", heading)
    }
    Sidebar.Item(to: "/dashboard", icon: "home") { Text("Dashboard") }
    Sidebar.Item(to: "/settings", icon: "gear") { Text("Settings") }
    Sidebar.Divider()
    Sidebar.Item(to: "/logout", icon: "logout") { Text("Log Out") }
}
```

#### Breadcrumb
Navigation trail.
```
Breadcrumb {
    Breadcrumb.Item(to: "/") { Text("Home") }
    Breadcrumb.Item(to: "/products") { Text("Products") }
    Breadcrumb.Item { Text("Laptop Pro") }  // Current (no link)
}
```

#### Link
Navigation link (renders as `<a>`, navigates via router).
```
Link(to: "/about") { Text("About Us") }
Link(to: "/profile", active: route.path == "/profile") { Text("Profile") }
```

#### Menu
Dropdown menu.
```
Menu(trigger: "Options") {
    Menu.Item { Text("Edit") }
    Menu.Item { Text("Duplicate") }
    Menu.Divider()
    Menu.Item(danger) { Text("Delete") }
}
```

#### Tabs
Tabbed content panels.
```
Tabs {
    TabPage("General") {
        Text("General settings")
    }
    TabPage("Security") {
        Text("Security settings")
    }
    TabPage("Notifications") {
        Text("Notification preferences")
    }
}
```

---

### 14.3 Data Display

#### Card
Contained content surface.
```
Card {
    Card.Header { Text("Title", heading) }
    Card.Body { Text("Content goes here") }
    Card.Footer { Button("Action", primary) }
}

// Simple card
Card(elevated) {
    Text("Simple card content")
}
```

#### Table
Data table.
```
Table {
    Thead {
        Tcell("Name")
        Tcell("Email")
        Tcell("Role")
    }
    for user in users {
        Trow {
            Tcell(user.name)
            Tcell(user.email)
            Tcell(user.role)
        }
    }
}
```

#### List
Styled list.
```
List {
    List.Item { Text("First item") }
    List.Item { Text("Second item") }
}

// From data
List {
    for item in items {
        List.Item {
            Text(item.name)
            Badge(item.status)
        }
    }
}
```

#### Badge
Small label.
```
Badge("New")
Badge("Active", success)
Badge("3", primary, pill)
Badge("Archived", secondary)
```

#### Avatar
User avatar.
```
Avatar(src: "/images/user.png", alt: "Monzer")
Avatar(initials: "MO", primary)           // Letter avatar
Avatar(src: "/images/user.png", size: large)
```

#### Tooltip
Hover tooltip.
```
Tooltip(text: "Click to save") {
    Button("Save", primary)
}
```

#### Tag
Removable label.
```
Tag("JavaScript")
Tag("TypeScript", primary) { on:remove { removeTag("TypeScript") } }
```

---

### 14.4 Data Input

#### Input
Text input field.
```
Input(text, placeholder: "Enter name")
Input(email, placeholder: "Email address")
Input(password, placeholder: "Password")
Input(number, placeholder: "Age", min: 0, max: 120)
Input(search, placeholder: "Search...")
Input(tel, placeholder: "Phone number")
Input(url, placeholder: "https://...")
```

**Input types:** `text`, `email`, `password`, `number`, `search`, `tel`, `url`, `date`, `time`, `datetime`, `color`

**Two-way binding:**
```
state username = ""

Input(text, bind: username, placeholder: "Username")
Text("Hello, {username}")    // Updates as user types
```

#### Select
Dropdown select.
```
Select(bind: selectedCountry, placeholder: "Choose country") {
    Option("us", "United States")
    Option("uk", "United Kingdom")
    Option("de", "Germany")
}
```

#### Checkbox
```
state agreed = false

Checkbox(bind: agreed, label: "I agree to the terms")
```

#### Radio
```
state plan = "free"

Radio(bind: plan, value: "free", label: "Free")
Radio(bind: plan, value: "pro", label: "Pro")
Radio(bind: plan, value: "enterprise", label: "Enterprise")
```

#### Switch
Toggle switch.
```
state darkMode = false

Switch(bind: darkMode, label: "Dark Mode")
```

#### Slider
Range slider.
```
state volume = 50

Slider(bind: volume, min: 0, max: 100, step: 1, label: "Volume")
```

#### DatePicker
```
state selectedDate = ""

DatePicker(bind: selectedDate, label: "Start Date", min: "2026-01-01")
```

#### FileUpload
```
FileUpload(accept: "image/*", label: "Upload Photo") {
    on:change {
        uploadFile(value)
    }
}
```

#### Form
Form wrapper with submission handling.
```
Form {
    Input(text, bind: name, placeholder: "Name", required: true)
    Input(email, bind: email, placeholder: "Email", required: true)
    Select(bind: role, placeholder: "Role") {
        Option("dev", "Developer")
        Option("design", "Designer")
    }
    Button("Submit", primary, type: "submit")

    on:submit {
        createUser(name, email, role)
    }
}
```

---

### 14.5 Feedback

#### Alert
Static notification banner.
```
Alert("Operation successful!", success)
Alert("Something went wrong.", danger)
Alert("Please review your input.", warning)
Alert("New update available.", info)

// Dismissible
Alert("Notice", info, dismissible) {
    on:dismiss { hideNotice() }
}
```

#### Toast
Temporary notification popup.
```
// Triggered from an action
action saveItem() {
    fetch result from "/api/save" (method: "POST", body: data) {
        success { Toast("Saved!", success) }
        error { Toast("Failed to save", danger) }
    }
}
```

Toast options:
```
Toast("Message", success, duration: 3000, position: "top-right")
```

#### Modal
Full modal dialog with overlay.
```
state showModal = false

Button("Open Modal") { showModal = true }

Modal(visible: showModal, title: "Confirm Action") {
    Text("Are you sure you want to proceed?")

    Modal.Footer {
        Button("Cancel") { showModal = false }
        Button("Confirm", primary) {
            confirmAction()
            showModal = false
        }
    }
}
```

#### Dialog
Simple dialog (lighter than Modal).
```
state showDialog = false

Dialog(visible: showDialog, title: "Delete Item?") {
    Text("This action cannot be undone.")
    Button("Cancel") { showDialog = false }
    Button("Delete", danger) { deleteItem() }
}
```

#### Spinner
Loading indicator.
```
Spinner()
Spinner(large, primary)
```

#### Progress
Progress bar.
```
Progress(value: 75, max: 100)
Progress(value: uploadProgress, label: "{uploadProgress}%")
```

#### Skeleton
Loading placeholder.
```
Skeleton(height: "20px", width: "200px")    // Text placeholder
Skeleton(height: "200px", rounded)           // Image placeholder
Skeleton(circle, size: "48px")               // Avatar placeholder
```

---

### 14.6 Actions

#### Button
```
Button("Label")
Button("Save", primary)
Button("Delete", danger, small)
Button("Submit", primary, large, full)
Button("Icon", icon: "plus", primary)
```

#### IconButton
Button with only an icon.
```
IconButton(icon: "close")
IconButton(icon: "menu", large)
IconButton(icon: "search", primary)
```

#### ButtonGroup
Grouped buttons.
```
ButtonGroup {
    Button("Left")
    Button("Center")
    Button("Right")
}
```

#### Dropdown
Dropdown button with menu.
```
Dropdown(label: "Actions") {
    Dropdown.Item { Text("Edit") }
    Dropdown.Item { Text("Copy") }
    Dropdown.Divider()
    Dropdown.Item(danger) { Text("Delete") }
}
```

---

### 14.7 Media

#### Image
```
Image(src: "/images/photo.jpg", alt: "Description")
Image(src: user.avatar, alt: user.name, rounded, size: large)
```

#### Video
```
Video(src: "/videos/intro.mp4", controls: true, autoplay: false)
```

#### Icon
Built-in icon set.
```
Icon("home")
Icon("search", large)
Icon("close", danger)
```

#### Carousel
Image/content slider.
```
Carousel(autoplay: true, interval: 5000) {
    Carousel.Slide {
        Image(src: "/images/slide1.jpg", alt: "Slide 1")
    }
    Carousel.Slide {
        Image(src: "/images/slide2.jpg", alt: "Slide 2")
    }
}
```

---

### 14.8 Typography

#### Text
Inline text.
```
Text("Hello world")
Text("Bold text", bold)
Text("Primary colored", primary)
Text(dynamicValue)
Text("Count: {count}", large, center)
```

#### Heading
Block-level heading.
```
Heading("Page Title", h1)
Heading("Section", h2)
Heading("Subsection", h3)
```

#### Code
Code block.
```
Code("const x = 42")
Code("npm install webfluent", block)       // Block-level code
```

#### Blockquote
```
Blockquote("The best way to predict the future is to create it.")
```

---

## 15. Design System

### 15.1 Token Categories

The design system is organized into token categories. All built-in components reference these tokens.

#### Colors
```
color-primary           // Primary brand color
color-secondary         // Secondary brand color
color-success           // Success / positive
color-danger            // Error / destructive
color-warning           // Warning / caution
color-info              // Informational
color-background        // Page background
color-surface           // Card/panel background
color-text              // Primary text color
color-text-muted        // Secondary text color
color-border            // Border color
```

#### Typography
```
font-family             // Base font family
font-family-mono        // Monospace font family
font-size-xs            // 0.75rem
font-size-sm            // 0.875rem
font-size-base          // 1rem
font-size-lg            // 1.25rem
font-size-xl            // 1.5rem
font-size-2xl           // 2rem
font-size-3xl           // 2.5rem
font-weight-normal      // 400
font-weight-medium      // 500
font-weight-bold        // 700
line-height-tight       // 1.25
line-height-normal      // 1.5
line-height-loose       // 1.75
```

#### Spacing
```
spacing-xs              // 0.25rem
spacing-sm              // 0.5rem
spacing-md              // 1rem
spacing-lg              // 1.5rem
spacing-xl              // 2rem
spacing-2xl             // 3rem
spacing-3xl             // 4rem
```

#### Border Radius
```
radius-none             // 0
radius-sm               // 0.25rem
radius-md               // 0.5rem
radius-lg               // 1rem
radius-xl               // 1.5rem
radius-full             // 9999px (circle/pill)
```

#### Shadows
```
shadow-none             // none
shadow-sm               // Subtle shadow
shadow-md               // Medium shadow
shadow-lg               // Large shadow
shadow-xl               // Extra-large shadow
```

#### Breakpoints
```
screen-sm               // 640px
screen-md               // 768px
screen-lg               // 1024px
screen-xl               // 1280px
```

### 15.2 Default Theme Values

```json
{
    "color-primary": "#3B82F6",
    "color-secondary": "#64748B",
    "color-success": "#22C55E",
    "color-danger": "#EF4444",
    "color-warning": "#F59E0B",
    "color-info": "#06B6D4",
    "color-background": "#FFFFFF",
    "color-surface": "#F8FAFC",
    "color-text": "#0F172A",
    "color-text-muted": "#64748B",
    "color-border": "#E2E8F0",

    "font-family": "Inter, system-ui, -apple-system, sans-serif",
    "font-family-mono": "JetBrains Mono, Fira Code, monospace",
    "font-size-base": "1rem",

    "spacing-md": "1rem",

    "radius-md": "0.5rem",

    "shadow-md": "0 4px 6px -1px rgba(0, 0, 0, 0.1)"
}
```

### 15.3 Dark Theme Values

```json
{
    "color-background": "#0F172A",
    "color-surface": "#1E293B",
    "color-text": "#F1F5F9",
    "color-text-muted": "#94A3B8",
    "color-border": "#334155"
}
```

### 15.4 Custom Theme in Config

```json
// webfluent.app.json
{
    "theme": {
        "name": "custom",
        "extends": "default",
        "tokens": {
            "color-primary": "#8B5CF6",
            "color-secondary": "#EC4899",
            "font-family": "Poppins, sans-serif",
            "radius-md": "1rem"
        }
    }
}
```

### 15.5 Theme Override in Code

```
// src/theme/overrides.wf
Theme {
    token color-primary = "#8B5CF6"
    token radius-md = "1rem"
    token font-family = "Poppins, sans-serif"
}
```

---

## 16. Project Configuration

### 16.1 webfluent.app.json

```json
{
    "name": "My App",
    "version": "1.0.0",
    "author": "Monzer Omer",

    "theme": {
        "name": "default",
        "mode": "light",
        "tokens": {}
    },

    "build": {
        "output": "./build",
        "minify": true,
        "sourcemap": false
    },

    "dev": {
        "port": 3000,
        "hotReload": true
    },

    "meta": {
        "title": "My App",
        "description": "Built with WebFluent",
        "favicon": "/public/favicon.ico",
        "lang": "en"
    }
}
```

---

## 17. Compilation Output

### 17.1 Output Structure

```
build/
├── index.html              # Single HTML entry point
├── app.js                  # Bundled JavaScript (reactivity, routing, state, events)
├── styles.css              # All styles (design tokens + component styles)
└── public/                 # Copied static assets
    ├── images/
    └── fonts/
```

### 17.2 HTML Output

The compiler generates a single `index.html` that loads the JS and CSS:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My App</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div id="app"></div>
    <script src="app.js"></script>
</body>
</html>
```

### 17.3 CSS Output

Generated CSS uses custom properties for design tokens:

```css
:root {
    --color-primary: #3B82F6;
    --color-secondary: #64748B;
    --font-family: Inter, system-ui, sans-serif;
    --spacing-md: 1rem;
    --radius-md: 0.5rem;
    /* ... all tokens ... */
}

/* Component styles reference tokens */
.wf-button {
    font-family: var(--font-family);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    cursor: pointer;
}
.wf-button--primary {
    background: var(--color-primary);
    color: #fff;
}
```

### 17.4 JavaScript Output

Generated JS includes only what's used:

```javascript
// Reactivity runtime (signals)
// Router (if Router is used)
// Event handlers
// Store logic
// Fetch wrappers
// DOM creation and update functions
```

The JS runtime is minimal — no virtual DOM library, no framework. Just surgical DOM updates driven by signals.

---

## Appendix A: Full Example Application

```
// src/App.wf
App {
    use AuthStore

    Navbar {
        Navbar.Brand {
            Text("TaskFlow", heading)
        }
        Navbar.Links {
            Link(to: "/") { Text("Tasks") }
            Link(to: "/about") { Text("About") }
        }
        Navbar.Actions {
            if AuthStore.user != null {
                Avatar(initials: AuthStore.user.initials, primary)
                Button("Logout") { AuthStore.logout() }
            } else {
                Link(to: "/login") { Button("Login", primary) }
            }
        }
    }

    Router {
        Route(path: "/", page: TaskList)
        Route(path: "/login", page: Login)
        Route(path: "/about", page: About)
    }
}
```

```
// src/stores/auth.wf
Store AuthStore {
    state user = null
    state token = ""

    derived isLoggedIn = user != null

    action login(email: String, password: String) {
        fetch result from "/api/auth/login" (method: "POST", body: { email, password }) {
            success {
                user = result.user
                token = result.token
                navigate("/")
            }
            error (err) {
                Toast(err.message, danger)
            }
        }
    }

    action logout() {
        user = null
        token = ""
        navigate("/login")
    }
}
```

```
// src/stores/tasks.wf
Store TaskStore {
    state tasks = []
    state filter = "all"

    derived filtered = if filter == "all" {
        tasks
    } else if filter == "active" {
        tasks.filter(t => !t.done)
    } else {
        tasks.filter(t => t.done)
    }

    derived remaining = tasks.filter(t => !t.done).length

    action add(title: String) {
        tasks.push({ id: tasks.length + 1, title: title, done: false })
    }

    action toggle(id: Number) {
        state task = tasks.filter(t => t.id == id)[0]
        task.done = !task.done
    }

    action remove(id: Number) {
        tasks = tasks.filter(t => t.id != id)
    }
}
```

```
// src/pages/TaskList.wf
Page TaskList (path: "/", title: "My Tasks") {
    use TaskStore

    Container {
        Heading("My Tasks", h1)
        Text("{TaskStore.remaining} tasks remaining", muted)

        Spacer()

        Row(gap: md) {
            Input(text, bind: newTask, placeholder: "Add a new task...", full) {
                on:keydown {
                    if key == "Enter" && newTask != "" {
                        TaskStore.add(newTask)
                        newTask = ""
                    }
                }
            }
            Button("Add", primary) {
                TaskStore.add(newTask)
                newTask = ""
            }
        }

        Spacer()

        ButtonGroup {
            Button("All") { TaskStore.filter = "all" }
            Button("Active") { TaskStore.filter = "active" }
            Button("Done") { TaskStore.filter = "done" }
        }

        Spacer()

        if TaskStore.filtered.length == 0 {
            Text("No tasks found.", muted, center)
        } else {
            Stack(gap: sm) {
                for task in TaskStore.filtered {
                    Card(outlined) {
                        Row(align: center, justify: between) {
                            Row(align: center, gap: md) {
                                Checkbox(checked: task.done) {
                                    on:change { TaskStore.toggle(task.id) }
                                }
                                Text(task.title)
                            }
                            IconButton(icon: "trash", danger, small) {
                                TaskStore.remove(task.id)
                            }
                        }
                    }
                }
            }
        }
    }
}
```

```
// src/pages/Login.wf
Page Login (path: "/login", title: "Login") {
    use AuthStore

    Container {
        Card(elevated) {
            Heading("Login", h2, center)
            Spacer()
            Form {
                state email = ""
                state password = ""

                Input(email, bind: email, placeholder: "Email", required: true, full)
                Spacer(sm)
                Input(password, bind: password, placeholder: "Password", required: true, full)
                Spacer()
                Button("Log In", primary, full, type: "submit")

                on:submit {
                    AuthStore.login(email, password)
                }
            }
        }
    }
}
```

```
// src/components/TaskItem.wf
Component TaskItem (task: Map) {
    use TaskStore

    Card(outlined) {
        Row(align: center, justify: between) {
            Row(align: center, gap: md) {
                Checkbox(checked: task.done) {
                    on:change { TaskStore.toggle(task.id) }
                }
                Text(task.title)
            }
            IconButton(icon: "trash", danger, small) {
                TaskStore.remove(task.id)
            }
        }
    }
}
```

---

## Appendix B: Grammar Summary (EBNF)

```ebnf
Program         = (PageDecl | ComponentDecl | StoreDecl | AppDecl)* ;

AppDecl         = "App" Block ;

PageDecl        = "Page" IDENT "(" PageAttrs ")" Block ;
PageAttrs       = PageAttr ("," PageAttr)* ;
PageAttr        = IDENT ":" Expression ;

ComponentDecl   = "Component" IDENT "(" PropList? ")" Block ;
PropList        = PropDecl ("," PropDecl)* ;
PropDecl        = IDENT "?"? ":" Type ("=" Expression)? ;

StoreDecl       = "Store" IDENT Block ;

Block           = "{" Statement* "}" ;

Statement       = StateDecl
                | DerivedDecl
                | EffectDecl
                | ActionDecl
                | UIElement
                | ControlFlow
                | FetchDecl
                | Assignment
                | MethodCall
                | EventHandler ;

StateDecl       = "state" IDENT "=" Expression ;
DerivedDecl     = "derived" IDENT "=" Expression ;
EffectDecl      = "effect" Block ;
ActionDecl      = "action" IDENT "(" ParamList? ")" Block ;

UIElement       = COMPONENT_NAME ("(" ArgList? ")")? Block?
                | IDENT ("(" ArgList? ")")? Block? ;

ArgList         = Arg ("," Arg)* ;
Arg             = Expression | IDENT ":" Expression ;

ControlFlow     = IfStmt | ForStmt | ShowStmt ;
IfStmt          = "if" Expression Block ("else" "if" Expression Block)* ("else" Block)? ;
ForStmt         = "for" IDENT ("," IDENT)? "in" Expression Block ;
ShowStmt        = "show" Expression Block ;

FetchDecl       = "fetch" IDENT "from" STRING ("(" FetchOpts ")")? FetchBlock ;
FetchBlock      = "{" ("loading" Block)? ("error" "(" IDENT ")" Block)? ("success" Block)? "}" ;
FetchOpts       = FetchOpt ("," FetchOpt)* ;
FetchOpt        = IDENT ":" Expression ;

EventHandler    = "on:" EVENT_NAME Block ;

Assignment      = AccessExpr "=" Expression ;

Expression      = LogicalOr ;
LogicalOr       = LogicalAnd ("||" LogicalAnd)* ;
LogicalAnd      = Equality ("&&" Equality)* ;
Equality        = Comparison (("==" | "!=") Comparison)* ;
Comparison      = Addition (("<" | ">" | "<=" | ">=") Addition)* ;
Addition        = Multiplication (("+" | "-") Multiplication)* ;
Multiplication  = Unary (("*" | "/" | "%") Unary)* ;
Unary           = ("!" | "-") Unary | Primary ;
Primary         = NUMBER | STRING | BOOL | "null"
                | IDENT ("." IDENT)* ("[" Expression "]")*
                | "(" Expression ")"
                | ListLiteral | MapLiteral
                | Lambda ;

ListLiteral     = "[" (Expression ("," Expression)*)? "]" ;
MapLiteral      = "{" (MapEntry ("," MapEntry)*)? "}" ;
MapEntry        = IDENT ":" Expression ;
Lambda          = IDENT "=>" Expression ;

Type            = "String" | "Number" | "Bool" | "List" | "Map" ;

AccessExpr      = IDENT ("." IDENT)* ("[" Expression "]")* ;
MethodCall      = AccessExpr "(" ArgList? ")" ;
```

---

## Appendix C: Compilation Examples

### Input (WebFluent)
```
Page Hello (path: "/") {
    state name = "World"

    Container {
        Heading("Hello, {name}!", h1)
        Input(text, bind: name, placeholder: "Enter your name")
    }
}
```

### Output (JavaScript — conceptual)
```javascript
import { signal, effect, mount, h, text, input } from './runtime.js';

function PageHello() {
    const name = signal("World");

    return h('div', { class: 'wf-container' }, [
        h('h1', { class: 'wf-heading wf-heading--h1' }, [
            text(() => `Hello, ${name()}!`)
        ]),
        input({
            type: 'text',
            class: 'wf-input',
            placeholder: 'Enter your name',
            value: name,
            onInput: (e) => name(e.target.value)
        })
    ]);
}

router.register('/', PageHello);
```
