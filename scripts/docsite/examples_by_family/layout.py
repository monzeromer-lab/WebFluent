"""Layout: the boxes a page is arranged in, and the doors to other code."""

EXAMPLES = {
    "Container": {
        "code": '''Container {
    style { background: $color-surface; border: 1px dashed $color-border; padding: 1rem }
    Heading("Release notes").h3
    Text("Version 7.0 makes the compiler strict: what 6.x accepted and 7.0 refuses is an error with its fix.").muted
    Row(gap: .sm) {
        Badge("Compiler").primary
        Badge("Breaking").warning
    }
}''',
        "variations": [
            ("Full width, with no maximum", '''Container.fluid {
    style { background: $color-surface; padding: 1rem }
    Text("This band runs edge to edge of whatever holds it.")
}'''),
            ("A page's main column", '''Container {
    Heading("Settings").h3
    Text("Changes are saved as you make them.").muted
}'''),
        ],
    },
    "Row": {
        "code": '''Row(gap: .md, align: .center, justify: .between) {
    style { padding: 0.75rem 1rem; border: 1px solid $color-border; border-radius: $radius-md }
    Row(gap: .sm, align: .center) {
        Avatar(initials: "AL").primary
        Stack(gap: .xs) {
            Text("Ada Lovelace").bold
            Text("Owner").muted.sm
        }
    }
    Badge("Active").success
    Button("Manage").sm
}''',
        "variations": [
            ("A column on a narrow screen", '''Row(gap: .md).stacks {
    Card { Card.Body { Text("Plan: Pro") } }
    Card { Card.Body { Text("Seats: 12") } }
    Card { Card.Body { Text("Renews: March 14") } }
}'''),
            ("A direction per breakpoint", '''Row(direction: { base: .column, md: .row }, gap: .sm) {
    Button("Save draft")
    Button("Publish").primary
}'''),
            ("Spread evenly", '''Row(justify: .evenly) {
    Text("Mon")  Text("Tue")  Text("Wed")  Text("Thu")  Text("Fri")
}'''),
        ],
    },
    "Column": {
        "code": '''Row(gap: .md) {
    Column(span: { base: 12, md: 8 }) {
        Card {
            Card.Body {
                Heading("Article").h3
                Text("The main column takes eight of the twelve on a wide screen.")
            }
        }
    }
    Column(span: { base: 12, md: 4 }) {
        Card.outlined {
            Card.Body {
                Heading("Related").h3
                Link("Layout", to: "/learn")
            }
        }
    }
}''',
        "variations": [
            ("Halves that stack on a phone", '''Row(gap: .md) {
    Column(span: 6).stacks { Card { Card.Body { Text("Monthly") } } }
    Column(span: 6).stacks { Card { Card.Body { Text("Yearly") } } }
}'''),
        ],
    },
    "Grid": {
        "code": '''Grid(columns: { base: 1, sm: 2, md: 3 }, gap: .md) {
    Card.elevated {
        Card.Body {
            Text("Visitors").muted.sm
            Heading("12,480").h3
            Badge("+8%").success
        }
    }
    Card.elevated {
        Card.Body {
            Text("Sign-ups").muted.sm
            Heading("342").h3
            Badge("+3%").success
        }
    }
    Card.elevated {
        Card.Body {
            Text("Churn").muted.sm
            Heading("1.2%").h3
            Badge("-0.4%").danger
        }
    }
}''',
        "variations": [
            ("A fixed number of columns", '''Grid(columns: 4, gap: .sm) {
    Badge("A")  Badge("B")  Badge("C")  Badge("D")
    Badge("E")  Badge("F")  Badge("G")  Badge("H")
}'''),
            ("One column on a narrow screen", '''Grid(columns: 3, gap: .lg).stacks {
    Card { Card.Body { Text("Design") } }
    Card { Card.Body { Text("Build") } }
    Card { Card.Body { Text("Ship") } }
}'''),
        ],
    },
    "Stack": {
        "code": '''Stack(gap: .md) {
    style { max-width: 22rem; padding: 1rem; border: 1px solid $color-border; border-radius: $radius-md }
    Heading("Sign in").h3
    Text("Use the address you signed up with.").muted
    Button("Continue with email").primary.full
    Button("Continue with a passkey").full
}''',
        "variations": [
            ("Centred, tight", '''Stack(gap: .xs, align: .center) {
    Avatar(initials: "GH").lg.primary
    Text("Grace Hopper").bold
    Text("Admiral").muted.sm
}'''),
            ("Spread to fill its height", '''Stack(justify: .between) {
    style { height: 12rem; border: 1px dashed $color-border; padding: 0.5rem }
    Text("Top")
    Text("Bottom")
}'''),
        ],
    },
    "Spacer": {
        "code": '''Heading("Before you deploy").h3
Text("Run the checks, then the tests.")
Spacer.lg
Button("Deploy").primary
Spacer.sm
Text("Last deployed 3 hours ago.").muted.sm''',
        "variations": [
            ("Every size", '''Text("xs")
Spacer.xs
Text("sm")
Spacer.sm
Text("md")
Spacer.md
Text("lg")
Spacer.lg
Text("xl")
Spacer.xl
Text("end")'''),
        ],
    },
    "Divider": {
        "code": '''Card {
    Card.Body {
        Heading("Billing").h3
        Text("Pro plan, billed yearly.")
        Divider
        Row(justify: .between) {
            Text("Next invoice").muted
            Text("$240.00").bold
        }
    }
}''',
        "variations": [
            ("Between sections of a menu", '''Stack(gap: .sm) {
    Link("Profile", to: "/learn")
    Link("Settings", to: "/reference")
    Divider
    Link("Help", to: "/recipes")
}'''),
        ],
    },
    "VirtualList": {
        "code": '''state rows = (0..500).map(i => "Commit #{i + 1}")
state picked = "Commit #250"
Text("Picked: {picked}").muted
VirtualList(rowHeight: 36, reveal: picked, label: "Commits") {
    style { height: 260px; border: 1px solid $color-border; border-radius: $radius-md }
    for r in rows by r {
        Row(justify: .between, align: .center) {
            style { height: 36px; padding: 0 0.75rem }
            Text(r)
            Button("Pick").sm { on click { picked = r } }
        }
    }
}''',
        "variations": [
            ("Rows of different heights, measured", '''state notes = (0..300).map(i => if i % 5 == 0 { "Note {i}: a longer line that wraps onto a second line in a narrow box." } else { "Note {i}" })
VirtualList(estimate: 40, label: "Notes") {
    style { max-height: 300px }
    for n in notes by n { Text(n) { style { padding: 0.5rem } } }
}'''),
        ],
    },
    "Splitter": {
        "code": '''state side = 220
Text("Sidebar: {side}px").muted.sm
Row {
    style { height: 180px; border: 1px solid $color-border; border-radius: $radius-md; overflow: hidden }
    Stack(gap: .xs) {
        style { width: {side}px; padding: 0.75rem; background: $color-surface }
        Text("Files").bold
        Text("App.wf")
        Text("Home.wf")
    }
    Splitter(bind: side, min: 140, max: 400, reset: 220, label: "Sidebar width")
    Stack {
        style { flex: 1; padding: 0.75rem }
        Text("Drag the handle, or focus it and use the arrow keys. Double-click puts it back.")
    }
}''',
        "variations": [
            ("Between a pane and a panel below it", '''state panel = 120
Stack {
    style { height: 320px }
    Container { style { flex: 1 } Text("Editor") }
    Splitter(bind: panel, min: 80, max: 240, reset: 120, label: "Panel height").horizontal.reverse
    Container { style { height: {panel}px } Text("Terminal") }
}'''),
        ],
    },
    "Masonry": {
        "code": '''Masonry(columns: { base: 1, sm: 2, md: 3 }) {
    Card { Card.Body { Heading("Shipping").h3  Text("Free over $50.") } }
    Card {
        Card.Body {
            Heading("Returns").h3
            Text("Thirty days, no questions. Keep the box and the receipt, and send it back from any post office.")
        }
    }
    Card { Card.Body { Heading("Support").h3  Text("Every day, 8–8.") } }
    Card { Image(src: "/og.png", alt: "The WebFluent card") }
    Card { Card.Body { Heading("Gift cards").h3  Text("Sent by email, any amount.") } }
    Card { Card.Body { Heading("Warranty").h3  Text("Two years on everything we make.") } }
}''',
        "variations": [
            ("Two columns", '''Masonry(columns: 2) {
    Card { Card.Body { Text("Short") } }
    Card { Card.Body { Text("A card with a good deal more to say than the one beside it.") } }
    Card { Card.Body { Text("Another") } }
}'''),
        ],
    },
    "AspectRatio": {
        "code": '''Grid(columns: 2, gap: .md) {
    AspectRatio(ratio: 16 / 9) {
        Image(src: "/og.png", alt: "The WebFluent card, in 16:9")
    }
    AspectRatio(ratio: 1) {
        Image(src: "/icon-512.png", alt: "The WebFluent icon, square")
    }
}''',
        "variations": [
            ("A box for a frame", '''AspectRatio(ratio: 4 / 3) {
    Center {
        style { background: $color-surface; border: 1px dashed $color-border }
        Text("4 : 3").muted
    }
}'''),
        ],
    },
    "ScrollArea": {
        "code": '''ScrollArea(height: "200px", label: "Build log") {
    style { border: 1px solid $color-border; border-radius: $radius-md; padding: 0.5rem 0.75rem }
    for line in (1..=40).map(i => "[{i}] compiled src/pages/Page{i}.wf") by line {
        Text(line).sm
    }
}''',
        "variations": [
            ("Sideways", '''ScrollArea(label: "Tags").horizontal {
    Row(gap: .sm) {
        style { width: max-content }
        for tag in ["Layout", "Forms", "Routing", "Stores", "Styling", "Motion", "Tests", "PDF", "Slides", "i18n"] by tag { Badge(tag).pill }
    }
}'''),
        ],
    },
    "Center": {
        "code": '''Center {
    style { height: 200px; background: $color-surface; border-radius: $radius-md }
    Stack(gap: .sm, align: .center) {
        Spinner
        Text("Loading your projects…").muted
    }
}''',
        "variations": [
            ("An icon in a round badge", '''Center {
    style { width: 48px; height: 48px; border-radius: 50%; background: $color-primary; color: $color-on-primary }
    Icon("check")
}'''),
        ],
    },
    "Sticky": {
        "code": '''ScrollArea(height: "240px", label: "Changelog") {
    style { border: 1px solid $color-border; border-radius: $radius-md }
    Sticky {
        style { background: $color-background; padding: 0.5rem 0.75rem; border-bottom: 1px solid $color-border }
        Row(justify: .between, align: .center) {
            Text("Changelog").bold
            Badge("7.0").primary
        }
    }
    for v in (0..20).map(i => 19 - i).map(n => "7.0.{n}: fixes and small changes") by v {
        Text(v) { style { padding: 0.5rem 0.75rem } }
    }
}''',
        "variations": [
            ("Below a fixed header", '''Sticky(offset: "64px") {
    Row(gap: .sm) { Button("Bold").sm  Button("Italic").sm }
}'''),
            ("Stuck to the bottom", '''Sticky.bottom {
    Row(justify: .end, gap: .sm) { Button("Cancel")  Button("Save").primary }
}'''),
        ],
    },
    "PanelGroup": {
        "code": '''state sizes: [Number] = [30, 70]
Text("Sizes: {sizes[0] ?? 0}% and {sizes[1] ?? 0}%").muted.sm
PanelGroup(bind: sizes, label: "Resize the sidebar") {
    style { height: 200px; border: 1px solid $color-border; border-radius: $radius-md }
    PanelGroup.Panel(min: 15) {
        Stack(gap: .xs) {
            style { padding: 0.75rem }
            Text("Inbox").bold
            Text("Archive")
            Text("Spam")
        }
    }
    PanelGroup.Panel(min: 40) {
        Text("Drag the handle, or use the arrow keys on it.") { style { padding: 0.75rem } }
    }
}''',
        "variations": [
            ("One above another, a panel that folds away", '''state split: [Number] = [60, 40]
PanelGroup(bind: split, label: "Resize the output").vertical {
    style { height: 300px }
    PanelGroup.Panel { Text("Source") }
    PanelGroup.Panel(collapsible: true) { Text("Output") }
}'''),
            ("Kept across visits", '''persist widths: [Number] = [25, 50, 25]
PanelGroup(bind: widths, label: "Resize the columns") {
    style { height: 160px }
    PanelGroup.Panel { Text("Files") }
    PanelGroup.Panel { Text("Editor") }
    PanelGroup.Panel { Text("Outline") }
}'''),
        ],
    },
    "ClientOnly": {
        "code": '''Card {
    Card.Body {
        Text("Your local time").muted.sm
        ClientOnly {
            placeholder { Skeleton(height: "28px", width: "120px") }
            Heading("{now(every: 1.seconds):.time}").h3
        }
    }
}''',
        "variations": [
            ("The window's width, which only a browser knows", '''ClientOnly {
    placeholder { Text("…") }
    Text("This window is {viewport.width}px wide.")
}'''),
        ],
    },
    "AppShell": {
        "code": '''AppShell {
    style { min-height: 320px; border: 1px solid $color-border }
    AppShell.Header {
        Row(justify: .between, align: .center) {
            style { flex: 1 }
            Text("Ledger").bold
            Button("Sign out").sm
        }
    }
    Sidebar {
        style { height: auto }
        Sidebar.Item(to: "/learn", icon: "book") { Text("Learn") }
        Sidebar.Item(to: "/reference", icon: "list") { Text("Reference") }
    }
    Stack(gap: .sm) {
        style { padding: 1rem }
        Heading("Dashboard").h3
        Text("The page the router draws sits here.")
    }
    AppShell.Footer { Text("© 2026 Ledger").muted.sm }
}''',
    },
    "Host": {
        "code": '''state level = 40
Slider(bind: level, min: 0, max: 100, label: "Level")
Host(tag: "canvas", aria-label: "A bar at {level}%",
     mount: (node) => node.getBoundingClientRect(),
     update: (box) => console.log("redraw at {level}% across {box.width}px"),
     cleanup: (box) => console.log("canvas gone")) {
    style { width: 100%; height: 120px; border: 1px dashed $color-border }
}''',
        "preview": False,
    },
    "Portal": {
        "code": '''state count = 0
Stack(gap: .sm, ref: panel) {
    style { padding: 1rem; border: 1px dashed $color-border; border-radius: $radius-md }
    Text("A node somebody else's code owns:").muted.sm
}
Portal(into: panel) {
    Row(gap: .sm, align: .center) {
        Text("Clicked {count} times")
        Button("Click").primary.sm { on click { count = count + 1 } }
    }
}''',
    },
    "Element": {
        "code": '''state loaded = false
Element("stripe-pricing-table", pricingTableId: "prctbl_1Pq", publishableKey: "pk_test_51Hx") {
    on ready { loaded = true }
    if !loaded { Text("Loading the prices…").muted }
}''',
        "variations": [
            ("Children and a class", '''Element("relative-time", datetime: "2026-03-14T09:30:00Z", class: "muted") {
    Text("March 14, 2026")
}'''),
        ],
        "preview": False,
    },
    "Router": {
        "code": '''Stack {
    Navbar {
        Navbar.Brand { Link("Ledger", to: "/") }
        Navbar.Links {
            Link("Learn", to: "/learn")
            Link("Reference", to: "/reference")
        }
    }
    Router(transition: .fade, duration: "200ms")
    Text("© 2026 Ledger").muted.sm
}''',
        "variations": [
            ("Beside a sidebar, pages that slide", '''Row {
    Sidebar {
        Sidebar.Item(to: "/learn", icon: "book") { Text("Learn") }
        Sidebar.Item(to: "/recipes", icon: "list") { Text("Recipes") }
    }
    Container { Router(transition: .slide, duration: "250ms") }
}'''),
        ],
        "preview": False,
    },
}
