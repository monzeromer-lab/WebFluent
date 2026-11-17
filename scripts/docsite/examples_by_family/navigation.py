"""Navigation: bars, menus, tabs, steps and the links between pages."""

EXAMPLES = {
    "Navbar": {
        "code": '''Navbar {
    Navbar.Brand { Link("WebFluent", to: "/") }
    Navbar.Links {
        Link("Learn", to: "/learn", active: .prefix)
        Link("Reference", to: "/reference", active: .prefix)
        Link("Recipes", to: "/recipes")
    }
    Navbar.Actions {
        ThemeToggle
        Button("Try it").primary.sm { on click { navigate("/playground") } }
    }
}''',
        "variations": [
            ("Brand and links only", '''Navbar {
    Navbar.Brand { Text("Ledger").bold }
    Navbar.Links {
        Link("Showcase", to: "/showcase")
        Link("Errors", to: "/errors")
    }
}'''),
        ],
    },
    "Sidebar": {
        "code": '''Row {
    style { height: 360px; border: 1px solid $color-border }
    Sidebar {
        style { height: 100% }
        Sidebar.Header { Text("Docs").bold }
        Sidebar.Item(to: "/", icon: "home") { Text("Home") }
        Sidebar.Item(to: "/learn", icon: "book", active: .prefix) { Text("Learn") }
        Sidebar.Item(to: "/reference", icon: "list", active: .prefix) { Text("Reference") }
        Sidebar.Item(to: "/recipes", icon: "layers") { Text("Recipes") }
        Sidebar.Divider
        Sidebar.Item(to: "/playground", icon: "play") { Text("Playground") }
    }
    Stack(gap: .sm) {
        style { flex: 1; padding: 24px }
        Heading("Reference").h3
        Text("Every built-in, with its props and an example.").muted
    }
}''',
        "variations": [
            ("On its own: as tall as the window, sticking as the page scrolls", '''Sidebar {
    Sidebar.Item(to: "/learn", icon: "book").prefix { Text("Learn") }
    Sidebar.Item(to: "/errors", icon: "alert-circle") { Text("Errors") }
}'''),
        ],
    },
    "Link": {
        "code": '''Stack(gap: .sm) {
    Link("Start the guide", to: "/learn")
    Link("Every component", to: "/reference", active: .prefix)
    Link(to: "/recipes") {
        Icon("book")
        Text("Recipes for common screens")
    }
    Link("The source on GitHub", to: "https://github.com/webfluent/webfluent", target: "_blank")
}''',
        "variations": [
            ("Active on every route beneath it", '''Link("Reference", to: "/reference").prefix'''),
            ("To a filtered view: active only with its query", '''Link("Open", to: "/reference?show=open")'''),
        ],
    },
    "Tabs": {
        "code": '''state tab = "profile"
Tabs(bind: tab, label: "Account") {
    Tabs.Page("Profile", value: "profile") {
        Text("Your name, photo and the email we write to.")
    }
    Tabs.Page("Billing", value: "billing") {
        Text("Pro plan, renews on 1 March.")
    }
    Tabs.Page("Danger zone", value: "danger", disabled: true) {
        Text("Only an owner can delete the account.")
    }
}
Text("Open: {tab}").muted.sm''',
        "variations": [
            ("Down the side, opened with Enter", '''state section = "general"
Tabs(bind: section, label: "Settings").vertical.manual {
    Tabs.Page("General", value: "general") { Text("Name and language") }
    Tabs.Page("Members", value: "members") { Text("Who can see the project") }
}'''),
            ("Hearing the change", '''state seen = ""
Tabs(label: "Report") {
    on change(v) { seen = "{v}" }
    Tabs.Page("Week") { Text("This week") }
    Tabs.Page("Month") { Text("This month") }
}'''),
        ],
    },
    "Breadcrumb": {
        "code": '''Breadcrumb {
    Breadcrumb.Item(to: "/") { Text("Home") }
    Breadcrumb.Item(to: "/reference") { Text("Reference") }
    Breadcrumb.Item { Text("Breadcrumb") }
}''',
        "variations": [
            ("With an icon for home", '''Breadcrumb(aria-label: "You are here") {
    Breadcrumb.Item(to: "/") { Icon("home") }
    Breadcrumb.Item(to: "/learn") { Text("Learn") }
    Breadcrumb.Item { Text("Data") }
}'''),
        ],
    },
    "Menu": {
        "code": '''state said = "Nothing chosen yet"
state pinned = false
Menu(trigger: "File actions").bottomStart {
    Menu.Item(shortcut: "F2") {
        on click { said = "Rename" }
        Text("Rename")
    }
    Menu.Item(shortcut: "mod+d") {
        on click { said = "Duplicate" }
        Text("Duplicate")
    }
    Menu.Item(checked: pinned) {
        on click { pinned = !pinned }
        Text("Pinned")
    }
    Menu.Sub(label: "Move to") {
        Menu.Item { on click { said = "Moved to Archive" }  Text("Archive") }
        Menu.Item { on click { said = "Moved to Drafts" }  Text("Drafts") }
    }
    Menu.Divider
    Menu.Item(shortcut: "mod+backspace").danger {
        on click { said = "Delete" }
        Text("Delete")
    }
}
Text(said).muted.sm''',
        "variations": [
            ("A context menu, opened by state at a point", '''state asked = false
state point = { x: 0, y: 0 }
Card {
    on contextmenu(e) { e.preventDefault()  point = { x: e.clientX, y: e.clientY }  asked = true }
    Text("Right-click here")
}
Menu(visible: asked, at: point) {
    Menu.Item { Text("Copy") }
    Menu.Item { Text("Paste") }
}'''),
            ("A disabled item", '''Menu(trigger: "Share") {
    Menu.Item { Text("Copy link") }
    Menu.Item(disabled: true) { Text("Publish (owners only)") }
}'''),
        ],
    },
    "NavigationMenu": {
        "code": '''NavigationMenu(label: "Main") {
    NavigationMenu.Item("Learn") {
        Stack(gap: .sm) {
            Link("The guide", to: "/learn")
            Link("Data and requests", to: "/learn/15-data")
            Link("Recipes", to: "/recipes")
        }
    }
    NavigationMenu.Item("Reference") {
        Stack(gap: .sm) {
            Link("Components", to: "/reference")
            Link("Diagnostics", to: "/errors")
        }
    }
    NavigationMenu.Link("Playground", to: "/playground")
    NavigationMenu.Link("Showcase", to: "/showcase")
}''',
        "variations": [
            ("Only links, no panels", '''NavigationMenu(label: "Site") {
    NavigationMenu.Link("Home", to: "/")
    NavigationMenu.Link("Reference", to: "/reference").prefix
}'''),
        ],
    },
    "Menubar": {
        "code": '''state said = "Pick a command"
Menubar(label: "Editor") {
    Menubar.Menu("File") {
        Menu.Item(shortcut: "mod+n") { on click { said = "New file" }  Text("New file") }
        Menu.Item(shortcut: "mod+o") { on click { said = "Open…" }  Text("Open…") }
        Menu.Divider
        Menu.Item(shortcut: "mod+s") { on click { said = "Saved" }  Text("Save") }
    }
    Menubar.Menu("Edit") {
        Menu.Item(shortcut: "mod+z") { on click { said = "Undo" }  Text("Undo") }
        Menu.Item(shortcut: "mod+shift+z") { on click { said = "Redo" }  Text("Redo") }
    }
    Menubar.Menu("View") {
        Menu.Sub(label: "Zoom") {
            Menu.Item(shortcut: "mod+=") { on click { said = "Zoom in" }  Text("Zoom in") }
            Menu.Item(shortcut: "mod+-") { on click { said = "Zoom out" }  Text("Zoom out") }
        }
    }
}
Text(said).muted.sm''',
    },
    "ContextMenu": {
        "code": '''state said = "Right-click the file, or focus it and press Shift+F10"
ContextMenu {
    Card.outlined {
        Row(gap: .sm, align: .center) {
            Icon("file")
            Text("quarterly-report.pdf")
        }
    }
    ContextMenu.Items(label: "File actions") {
        Menu.Item(shortcut: "mod+o") { on click { said = "Opened" }  Text("Open") }
        Menu.Item(shortcut: "F2") { on click { said = "Renamed" }  Text("Rename") }
        Menu.Divider
        Menu.Item.danger { on click { said = "Deleted" }  Text("Delete") }
    }
}
Text(said).muted.sm''',
    },
    "BottomNavigation": {
        "code": '''Stack(gap: .sm) {
    style { width: 360px; height: 240px; padding: 16px; contain: layout; overflow: hidden; border: 1px solid $color-border; border-radius: 24px }
    Heading("Inbox").h3
    Text("Three new replies since this morning.").muted
    BottomNavigation(label: "Sections") {
        BottomNavigation.Item("Home", to: "/", icon: "home")
        BottomNavigation.Item("Learn", to: "/learn", icon: "book").prefix
        BottomNavigation.Item("Reference", to: "/reference", icon: "list").prefix
        BottomNavigation.Item("Play", to: "/playground", icon: "play")
    }
}''',
        "variations": [
            ("On a page: the bar along the bottom of the screen", '''BottomNavigation(label: "Sections") {
    BottomNavigation.Item("Home", to: "/", icon: "home")
    BottomNavigation.Item("Recipes", to: "/recipes", icon: "book").prefix
}'''),
        ],
    },
    "PrevNext": {
        "code": '''PrevNext {
    PrevNext.Prev("Forms", to: "/learn")
    PrevNext.Next("Data and requests", to: "/learn/15-data")
}''',
        "variations": [
            ("The last page: only the way back, in your words", '''PrevNext(label: "More of the guide") {
    PrevNext.Prev("Recipes", to: "/recipes", label: "Back to")
}'''),
        ],
    },
    "OnThisPage": {
        "code": '''Row(gap: .lg, align: .start) {
    Stack {
        Heading("Deploying").h1
        Heading("Build the site").h2
        Text("Run wf build; the output is in build/.")
        Heading("Choose a host").h2
        Heading("Static hosts").h3
        Text("Any host that serves files.")
        Heading("Set the base path").h2
        Text("Under a folder, name it in the config.")
    }
    OnThisPage(levels: 2, label: "On this page")
}''',
    },
    "Pagination": {
        "code": '''state page = 4
Pagination(bind: page, pages: 12, siblings: 1, label: "Results")
Text("Showing page {page} of 12").muted.sm''',
        "variations": [
            ("More pages around the current one", '''state page = 10
Pagination(bind: page, pages: 40, siblings: 2)'''),
        ],
    },
    "Stepper": {
        "code": '''state step = 0
state billed = false
Stepper(bind: step) {
    Stepper.Step("Account", done: step > 0) {
        Text("Your name and email.")
        Button("Next").primary { on click { step = 1 } }
    }
    Stepper.Step("Billing", done: billed) {
        Text("A card, or an invoice each month.")
        Button("Add a card").primary { on click { billed = true  step = 2 } }
    }
    Stepper.Step("Confirm") {
        Text("Check it over, then start.")
        Button("Start the trial").success
    }
}
Text("Step {step + 1} of 3").muted.sm''',
        "variations": [
            ("Linear: no skipping ahead", '''state step = 0
Stepper(bind: step).linear {
    Stepper.Step("Upload") { Text("Choose the file") }
    Stepper.Step("Review") { Text("Check the rows") }
}'''),
            ("A step with an error", '''state step = 1
Stepper(bind: step) {
    Stepper.Step("Details", done: true) { Text("Saved") }
    Stepper.Step("Payment", error: true) { Text("The card was declined.") }
}'''),
        ],
    },
    "CommandPalette": {
        "code": '''state palette = false
state said = "Nothing run yet"
Button("Search commands…") { on click { palette = true } }
Text(said).muted.sm
CommandPalette(open: palette, label: "Commands", placeholder: "Type a command or a page") {
    CommandPalette.Group("Pages") {
        CommandPalette.Item("Reference", icon: "list") { on select { navigate("/reference") } }
        CommandPalette.Item("Recipes", icon: "book") { on select { navigate("/recipes") } }
    }
    CommandPalette.Group("Actions") {
        CommandPalette.Item("New file", icon: "file-plus", shortcut: "mod+n") { on select { said = "New file" } }
        CommandPalette.Item("Copy link", icon: "copy", shortcut: "mod+shift+c") { on select { said = "Link copied" } }
        CommandPalette.Item("Switch theme", icon: "moon") { on select { said = "Theme switched" } }
    }
}''',
        "variations": [
            ("Opened from anywhere with mod+k", '''state palette = false
on key("mod+k") { palette = true }
Text { Text("Press ")  Kbd("mod+k")  Text(" to open the palette") }
CommandPalette(open: palette) {
    CommandPalette.Group("Go to") {
        CommandPalette.Item("Learn") { on select { navigate("/learn") } }
    }
}'''),
        ],
    },
}
