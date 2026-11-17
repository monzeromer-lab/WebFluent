"""Feedback and actions: alerts, overlays, progress, and the buttons that act."""

EXAMPLES = {
    # ---------------------------------------------------------------- Feedback
    "Alert": {
        "code": '''Stack(gap: .sm) {
    Alert("Your changes were saved.").success
    Alert("Your trial ends in 3 days.").warning
    Alert("The server is not answering. We will try again in a minute.").danger
    Alert("A new version of the editor is available.").info
}''',
        "variations": [
            ("With actions in its block", '''Alert("Billing failed for the card ending 4242.").danger {
    Button("Update card").sm
}'''),
            ("Shown while something holds", '''state failed = true
if failed {
    Alert("Could not reach the build server.").danger {
        Button("Dismiss").sm { on click { failed = false } }
    }
}'''),
        ],
    },
    "Toast": {
        "code": '''state archived = 0
Row(gap: .sm) {
    Button("Archive conversation") {
        on click {
            archived = archived + 1
            toast("Conversation archived", action: "Undo", onAction: () => toast("Restored"))
        }
    }
    Button("Save draft").primary { on click { toast("Draft saved", tone: .success) } }
}
Text("{archived} archived").muted''',
        "variations": [
            ("The element, while a state holds", '''state saved = false
Button("Save") { on click { saved = true } }
if saved { Toast("Saved").success }'''),
            ("Following a promise", '''Button("Copy the invite link") {
    on click { toast.promise(navigator.clipboard.writeText("https://ledger.example.com/invite/42"), loading: "Copying…", done: "Link copied", failed: "Could not copy") }
}'''),
        ],
    },
    "Modal": {
        "code": '''state open = false
state name = "Q3 report"
Button("Rename") { on click { open = true } }
Modal(visible: open, title: "Rename the report").closable {
    on close { open = false }
    Input(bind: name, label: "Name")
    Modal.Footer {
        Button("Cancel") { on click { open = false } }
        Button("Save").primary { on click { open = false } }
    }
}
Text("Report: {name}").muted''',
        "variations": [
            ("Escape only, focus on a field", '''state open = false
state email = ""
Button("Invite") { on click { open = true } }
Modal(visible: open, title: "Invite a teammate", dismiss: .escape, initialFocus: emailField) {
    Input(bind: email, label: "Email", ref: emailField).email
    Modal.Footer { Button("Send invite").primary { on click { open = false } } }
}'''),
            ("A heading with markup", '''state open = false
Button("Delete") { on click { open = true } }
Modal(visible: open, title: "Delete deploys.wf?").closable {
    Modal.Title { Markdown("Delete `deploys.wf`?") }
    Text("This cannot be undone.")
    Modal.Footer { Button("Delete").danger { on click { open = false } } }
}'''),
        ],
    },
    "Dialog": {
        "code": '''state asking = false
Button("Close the editor") { on click { asking = true } }
Dialog(visible: asking, title: "Leave without saving?").closable {
    Text("Your draft of notes.md has changes that are not saved.")
    Row(gap: .sm, justify: .end) {
        Button("Stay") { on click { asking = false } }
        Button("Leave").danger { on click { asking = false } }
    }
}''',
        "variations": [
            ("A heading with markup", '''state asking = false
Button("Leave") { on click { asking = true } }
Dialog(visible: asking, title: "Leave without saving notes.md?") {
    Dialog.Title { Markdown("Leave without saving `notes.md`?") }
    Button("Stay") { on click { asking = false } }
}'''),
            ("Must be answered", '''state asking = false
Button("Sign out") { on click { asking = true } }
Dialog(visible: asking, title: "Sign out of every device?", dismiss: .none) {
    Button("Sign out").primary { on click { asking = false } }
}'''),
        ],
    },
    "AlertDialog": {
        "code": '''state sure = false
state said = "Draft: 3 unsaved edits"
Button("Discard changes").danger.outlined { on click { sure = true } }
AlertDialog(visible: sure, title: "Discard your changes?") {
    Text("Your 3 edits to the pricing page will be lost.")
    AlertDialog.Footer {
        Button("Keep editing") { on click { sure = false } }
        Button("Discard").danger { on click { sure = false  said = "Draft discarded" } }
    }
}
Text(said).muted''',
        "variations": [
            ("A heading with markup", '''state sure = false
Button("Delete branch") { on click { sure = true } }
AlertDialog(visible: sure, title: "Delete the branch fix-login?") {
    AlertDialog.Title { Markdown("Delete the branch `fix-login`?") }
    AlertDialog.Footer { Button("Cancel") { on click { sure = false } } }
}'''),
        ],
    },
    "Sheet": {
        "code": '''state filters = false
state sort = "new"
state inStock = true
Button("Filters") { on click { filters = true } }
Sheet(visible: filters, title: "Filters", snap: [0.5, 1]).closable {
    Radio(bind: sort, value: "new", label: "Newest first")
    Radio(bind: sort, value: "price", label: "Lowest price")
    Switch(bind: inStock, label: "In stock only")
    Sheet.Footer { Button("Show results").primary.full { on click { filters = false } } }
}
Text("Sorted by {sort}").muted''',
        "variations": [
            ("From the end of the page", '''state details = false
Button("Details") { on click { details = true } }
Sheet(visible: details, side: .end, title: "Deploy #418").closable {
    Text("Opened 3 days ago by Ada")
}'''),
            ("From the start, named by a label", '''state nav = false
Button("Menu") { on click { nav = true } }
Sheet(visible: nav, side: .start, label: "Navigation") {
    Link("Learn", to: "/learn")
    Link("Reference", to: "/reference")
}'''),
        ],
    },
    "Popover": {
        "code": '''state open = false
state branch = "main"
Button("Branch: {branch}", ref: branchBtn) { on click { open = !open } }
Popover(visible: open, anchor: branchBtn, placement: .bottomStart, label: "Branches").arrow {
    on close(reason) { open = false }
    Stack(gap: .xs) {
        Button("main") { on click { branch = "main"  open = false } }
        Button("release-7.0") { on click { branch = "release-7.0"  open = false } }
        Button("fix-login") { on click { branch = "fix-login"  open = false } }
    }
}''',
        "variations": [
            ("Above, closed by its own button only", '''state open = false
Button("Share", ref: shareBtn) { on click { open = true } }
Popover(visible: open, anchor: shareBtn, placement: .top, label: "Share", dismiss: .none) {
    Text("Anyone with the link can view.")
    Button("Done") { on click { open = false } }
}'''),
        ],
    },
    "Popconfirm": {
        "code": '''state files = ["notes.md", "draft.md", "todo.md"]
Stack(gap: .sm) {
    for f in files by f {
        Row(gap: .sm, align: .center, justify: .between) {
            Text(f)
            Popconfirm(title: "Delete {f}?", yes: "Delete", no: "Keep").danger {
                on confirm { files = files.filter(x => x != f) }
                Button("Delete").sm.outlined
            }
        }
    }
}''',
        "variations": [
            ("Hearing no", '''state said = "Draft"
Popconfirm(title: "Publish to everyone?", yes: "Publish") {
    on confirm { said = "Published" }
    on cancel { said = "Not yet" }
    Button("Publish").primary
}
Text(said)'''),
        ],
    },
    "HoverCard": {
        "code": '''Text("Reviewed by") {
    HoverCard(delay: 300) {
        Link("@ada", to: "/showcase")
        HoverCard.Content {
            Row(gap: .sm, align: .center) {
                Avatar(initials: "AL").primary
                Stack(gap: .xs) {
                    Text("Ada Lovelace").bold
                    Text("Wrote the first program.").muted.sm
                }
            }
        }
    }
}''',
        "variations": [
            ("At once, on a button", '''HoverCard(delay: 0) {
    Button("Pro plan")
    HoverCard.Content { Text("Unlimited projects and priority support.") }
}'''),
        ],
    },
    "LoadingOverlay": {
        "code": '''state saving = false
state name = "Ada Lovelace"
LoadingOverlay(visible: saving, label: "Saving your profile") {
    Card {
        Input(bind: name, label: "Name")
        Button("Save").primary { on click { saving = true } }
    }
}
Button("Finish saving") { on click { saving = false } }''',
        "variations": [
            ("With the default label", '''state busy = true
LoadingOverlay(visible: busy) {
    Card { Text("Monthly revenue") }
}'''),
        ],
    },
    "Tour": {
        "code": '''state touring = false
state toured = "Not toured yet"
Row(gap: .sm) {
    Button("New project", ref: newBtn).primary
    Button("Import", ref: importBtn)
    Button("Take the tour") { on click { touring = true } }
}
Tour(open: touring) {
    on finish { toured = "Tour finished" }
    Tour.Step(target: newBtn, title: "Start here") { Text("Make a project from a template.") }
    Tour.Step(target: importBtn, title: "Bring your work") { Text("Import a folder you already have.") }
}
Text(toured).muted''',
        "variations": [
            ("In your words", '''state touring = false
Button("Settings", ref: settingsBtn)
Button("Show me") { on click { touring = true } }
Tour(open: touring, back: "Previous", next: "Continue", done: "Got it", close: "Skip", of: "of") {
    on close { touring = false }
    Tour.Step(target: settingsBtn, title: "Settings") { Text("Your account lives here.") }
}'''),
        ],
    },
    "ConsentBanner": {
        "code": '''ConsentBanner("We would like to count visits to improve the site. Nothing is shared.", accept: "Allow", decline: "No thanks") {
    Link("How we count", to: "/learn")
}''',
        "preview": False,
        "variations": [
            ("With its own words only", '''ConsentBanner("May we use analytics cookies?")'''),
        ],
    },
    "ConsentSettings": {
        "code": '''Row(gap: .md, align: .center) {
    Text("© 2026 Ledger").muted.sm
    Link("Privacy", to: "/learn")
    ConsentSettings("Cookie settings")
}
ConsentBanner("We would like to count visits.", accept: "Allow", decline: "No thanks")''',
        "preview": False,
    },
    "CircularProgress": {
        "code": '''state done = 40
Row(gap: .lg, align: .center) {
    CircularProgress(value: done, max: 100, label: "Upload") { Text("{done}%").sm }
    Stack(gap: .xs) {
        Text("Uploading photos.zip").bold
        Button("Add 20%").sm { on click { done = if done >= 80 { 100 } else { done + 20 } } }
    }
}''',
        "variations": [
            ("Turning, with no value", '''CircularProgress(label: "Loading the report")'''),
        ],
    },
    "Meter": {
        "code": '''state used = 0.72
Stack(gap: .sm) {
    Text("Disk: {used:.percent} used")
    Meter(value: used, max: 1, high: 0.9, optimum: 0, label: "Disk")
    Slider(bind: used, min: 0, max: 1, step: 0.01, label: "Used")
}''',
        "variations": [
            ("Past its high mark", '''Meter(value: 0.95, high: 0.9, label: "Memory")'''),
            ("Where high is good", '''Meter(value: 82, min: 0, max: 100, low: 40, optimum: 100, label: "Battery")'''),
        ],
    },
    "Spinner": {
        "code": '''Row(gap: .lg, align: .center) {
    Spinner.sm
    Spinner
    Spinner.lg
}''',
        "variations": [
            ("Beside what is loading", '''Row(gap: .sm, align: .center) {
    Spinner.sm
    Text("Loading deploys…").muted
}'''),
        ],
    },
    "Progress": {
        "code": '''state sent = 3
Stack(gap: .sm) {
    Text("Sending invitations: {sent} of 8")
    Progress(value: sent, max: 8)
    Button("Send one more") { on click { sent = if sent < 8 { sent + 1 } else { 0 } } }
}''',
        "variations": [
            ("Out of a hundred", '''Progress(value: 64, max: 100)'''),
        ],
    },
    "Skeleton": {
        "code": '''Row(gap: .md, align: .center) {
    Skeleton(size: "48px").circle
    Stack(gap: .sm) {
        Skeleton(height: "16px", width: "180px")
        Skeleton(height: "12px", width: "260px")
        Skeleton(height: "12px", width: "220px")
    }
}''',
        "variations": [
            ("While a list loads", '''state loading = true
if loading {
    Stack(gap: .sm) {
        Skeleton(height: "20px")
        Skeleton(height: "20px")
    }
} else {
    Text("Loaded")
}'''),
        ],
    },
    # ----------------------------------------------------------------- Actions
    "Button": {
        "code": '''state saved = 0
Row(gap: .sm, justify: .end) {
    Button("Cancel").outlined
    Button("Save draft") { on click { saved = saved + 1 } }
    Button("Publish").primary { on click { toast("Published", tone: .success) } }
}
Text("Saved {saved} times").muted''',
        "variations": [
            ("Tones", '''Row(gap: .sm) {
    Button("Primary").primary
    Button("Secondary").secondary
    Button("Success").success
    Button("Danger").danger
    Button("Warning").warning
    Button("Info").info
}'''),
            ("Sizes, shapes and widths", '''Stack(gap: .sm) {
    Row(gap: .sm, align: .center) {
        Button("Small").sm
        Button("Default")
        Button("Large").lg
    }
    Row(gap: .sm) { Button("Rounded").rounded  Button("Pill").pill.primary  Button("Outlined").outlined.danger }
    Button("Continue to checkout").primary.full
}'''),
            ("Busy and disabled", '''state agreed = false
Checkbox(bind: agreed, label: "I agree to the terms")
Row(gap: .sm) {
    Button("Create account", disabled: !agreed).primary
    Button("Saving", loading: true)
}'''),
            ("A link and a submit", '''state email = ""
Form {
    on submit { toast("Subscribed") }
    Input(bind: email, label: "Email").email
    Row(gap: .sm) {
        Button("Subscribe", type: .submit).primary
        Button("Read the guide", to: "/learn").outlined
    }
}'''),
        ],
    },
    "IconButton": {
        "code": '''state stars = 0
Row(gap: .sm, align: .center) {
    IconButton(icon: "star", label: "Star this project").primary { on click { stars = stars + 1 } }
    IconButton(icon: "edit", label: "Edit")
    IconButton(icon: "copy", label: "Copy link")
    IconButton(icon: "trash", label: "Delete").danger
    Text("{stars} stars").muted
}''',
        "variations": [
            ("Sizes", '''Row(gap: .sm, align: .center) {
    IconButton(icon: "close", label: "Close").sm
    IconButton(icon: "close", label: "Close")
    IconButton(icon: "close", label: "Close").lg
}'''),
            ("Saying what it opens", '''state open = false
IconButton(icon: "menu", label: "Actions", aria-haspopup: "menu", aria-expanded: open) { on click { open = !open } }'''),
            ("Disabled", '''IconButton(icon: "download", label: "Download", disabled: true)'''),
        ],
    },
    "ButtonGroup": {
        "code": '''state range = "Week"
ButtonGroup {
    Button("Day", outlined: range != "Day") { on click { range = "Day" } }
    Button("Week", outlined: range != "Week") { on click { range = "Week" } }
    Button("Month", outlined: range != "Month") { on click { range = "Month" } }
}
Text("Showing one {range}").muted''',
        "variations": [
            ("Small actions together", '''ButtonGroup {
    Button("Copy").sm
    Button("Cut").sm
    Button("Paste").sm
}'''),
        ],
    },
    "Dropdown": {
        "code": '''state said = "Nothing chosen"
Dropdown("Actions") {
    Dropdown.Item { Button("Duplicate") { on click { said = "Duplicated" } } }
    Dropdown.Item { Button("Rename") { on click { said = "Renamed" } } }
    Dropdown.Divider
    Dropdown.Item { Button("Archive").danger { on click { said = "Archived" } } }
}
Text(said).muted''',
        "variations": [
            ("Items of plain text", '''Dropdown("Sort by") {
    Dropdown.Item { Text("Newest") }
    Dropdown.Item { Text("Oldest") }
}'''),
        ],
    },
    "Toggle": {
        "code": '''state bold = false
state italic = true
Row(gap: .sm) {
    Toggle("Bold", bind: bold, icon: "type")
    Toggle("Italic", bind: italic)
}
Text("The quick brown fox") {
    style {
        font-weight: {if bold { "700" } else { "400" }}
        font-style: {if italic { "italic" } else { "normal" }}
    }
}''',
        "variations": [
            ("Without an icon", '''state pinned = false
Toggle("Pin to the sidebar", bind: pinned)'''),
        ],
    },
    "ToggleGroup": {
        "code": '''state view = "grid"
ToggleGroup(bind: view, label: "View") {
    ToggleGroup.Item("Grid", value: "grid", icon: "grid")
    ToggleGroup.Item("List", value: "list", icon: "list")
}
Text("Showing the {view} view").muted''',
        "variations": [
            ("Several at once", '''state styles: [String] = ["b"]
ToggleGroup(bind: styles, label: "Text style").multiple {
    ToggleGroup.Item("Bold", value: "b")
    ToggleGroup.Item("Italic", value: "i")
    ToggleGroup.Item("Underline", value: "u")
}
Text("{styles.length} on")'''),
        ],
    },
    "CopyButton": {
        "code": '''Row(gap: .sm, align: .center) {
    Code("https://ledger.example.com/invite/42")
    CopyButton("Copy link", value: "https://ledger.example.com/invite/42", copied: "Link copied")
}''',
        "variations": [
            ("With its own words", '''CopyButton(value: "wf init my-site -t static")'''),
        ],
    },
    "ThemeToggle": {
        "code": '''Row(gap: .sm, align: .center, justify: .between) {
    Text("Appearance").bold
    ThemeToggle
}
Text("The theme is {theme}.").muted''',
        "variations": [
            ("Beside a choice of three", '''Row(gap: .sm) {
    ThemeToggle
    Button("Follow the system").sm { on click { setTheme("system") } }
}'''),
        ],
    },
    "LocaleSwitch": {
        "code": '''Row(gap: .sm, align: .center, justify: .between) {
    Text("Language").bold
    LocaleSwitch(label: "Language")
}''',
        "preview": False,
    },
    "Fab": {
        "code": '''state said = "Nothing made yet"
Fab(icon: "plus", label: "Create") {
    Fab.Action("Note", icon: "file") { on click { said = "A new note" } }
    Fab.Action("Folder", icon: "folder") { on click { said = "A new folder" } }
    Fab.Action("Upload", icon: "upload") { on click { said = "An upload" } }
}
Text(said).muted''',
        "variations": [
            ("One action", '''state notes = 0
Fab(icon: "plus", label: "New note") { on click { notes = notes + 1 } }
Text("{notes} notes")'''),
        ],
        "preview": False,
    },
    "BackToTop": {
        "code": '''Stack(gap: .md) {
    for n in 1..=12 {
        Text("Chapter {n}: what the page says, long enough to scroll past.")
    }
}
BackToTop(after: 600, label: "Back to the top")''',
        "variations": [
            ("At its default", '''Text("A long page of release notes.")
BackToTop'''),
        ],
        "preview": False,
    },
}
