"""Data display: cards, lists, badges and the rest."""

EXAMPLES = {
    "Card": {
        "code": '''Card.elevated {
    Card.Header {
        Row(justify: .between, align: .center) {
            Heading("Pro").h3
            Badge("Popular").primary.pill
        }
    }
    Card.Body {
        Text("Everything in Free, and more.").muted
        List {
            List.Item { Text("Unlimited projects") }
            List.Item { Text("Priority support") }
        }
    }
    Card.Footer { Button("Choose Pro").primary.full }
}''',
        "variations": [
            ("A plain box: no parts, only what it holds", '''Card.outlined { Text("Drafts are kept for 30 days.") }'''),
            ("Flat, in a grid of cards", '''Grid(columns: { base: 1, md: 3 }, gap: .md) {
    Card.flat { Heading("Free").h3  Text("For trying it out") }
    Card.elevated { Heading("Pro").h3  Text("For a team") }
    Card.flat { Heading("Scale").h3  Text("For many teams") }
}'''),
        ],
    },
}

EXAMPLES.update({
    "DescriptionList": {
        "code": '''DescriptionList.grid {
    DescriptionList.Item("Service", value: "checkout-api")
    DescriptionList.Item("Region", value: "eu-west-1")
    DescriptionList.Item("Replicas", value: 3)
    DescriptionList.Item("Status") { Badge("Live").success }
    DescriptionList.Item("Last deploy") { RelativeTime(@2026-11-14T16:05Z) }
}''',
        "variations": [
            ("Stacked: each term above its value", '''DescriptionList {
    DescriptionList.Item("Plan", value: "Pro, billed yearly")
    DescriptionList.Item("Renews", value: "1 March 2027")
}'''),
            ("A term and its details written by hand", '''DescriptionList.grid {
    DescriptionList.Term("Owner")
    DescriptionList.Details { Link("Platform team", to: "/reference") }
}'''),
        ],
    },
    "Timeline": {
        "code": '''Timeline {
    Timeline.Item("7.0 released", at: @2026-11-10) {
        Text("Typed requests, results and error enums.")
    }
    Timeline.Item("6.1 released", at: @2026-09-02) {
        Text("Charts and QR codes on a web page.")
    }
    Timeline.Item("Public beta", at: @2026-06-15) {
        Text("The first hundred projects moved over.").muted
    }
}''',
        "variations": [
            ("Drawn from a list, notes as Markdown", '''state releases = [{ version: "7.0.1", date: "2026-11-14", notes: "Fixes a **refused write**." }, { version: "7.0.0", date: "2026-11-10", notes: "See the [changelog](/learn)." }]
Timeline {
    for r in releases by r.version {
        Timeline.Item(r.version, at: r.date) { Markdown(r.notes) }
    }
}'''),
            ("A step with no date yet", '''Timeline {
    Timeline.Item("Order placed", at: @2026-11-15T10:12Z) { Text("Paid by card.") }
    Timeline.Item("Shipped") { Text("Waiting for the courier.").muted }
}'''),
        ],
    },
    "AvatarGroup": {
        "code": '''AvatarGroup(max: 4, label: "Reviewers") {
    Avatar(src: "/icon-512.png", alt: "Ada Lovelace")
    Avatar(initials: "GH").primary
    Avatar(initials: "AT")
    Avatar(initials: "ED")
    Avatar(initials: "KJ")
    Avatar(initials: "MH")
}''',
        "variations": [
            ("From a list, small", '''state team = [{ id: "1", initials: "AL" }, { id: "2", initials: "GH" }, { id: "3", initials: "AT" }]
AvatarGroup(max: 2, label: "Team") {
    for p in team by p.id { Avatar(initials: p.initials).sm }
}'''),
        ],
    },
    "Item": {
        "code": '''state push = true
state digest = false
Stack(gap: .sm) {
    Item(title: "Push notifications", description: "Sent to your phone when a deploy fails") {
        Item.Media { Icon("bell") }
        Item.Actions { Switch(bind: push, aria-label: "Push notifications") }
    }
    Item(title: "Weekly digest", description: "A summary every Monday morning") {
        Item.Media { Icon("mail") }
        Item.Actions { Switch(bind: digest, aria-label: "Weekly digest") }
    }
}''',
        "variations": [
            ("A search result with a link and a badge", '''Item(title: "Forms and validation", description: "Rules beside the state they guard.") {
    Item.Media { Icon("book") }
    Item.Actions { Badge("Guide").info }
}'''),
            ("Its words written by hand", '''Item {
    Item.Media { Avatar(initials: "AL").sm }
    Item.Content {
        Text("Ada Lovelace").bold
        Text("Commented on your pull request").muted.sm
    }
    Item.Actions { Button("Open").sm }
}'''),
        ],
    },
    "Sparkline": {
        "code": '''Stack(gap: .sm) {
    Row(gap: .sm, align: .center) {
        Text("Sign-ups this week")
        Sparkline([12, 18, 9, 24, 31, 27, 40], label: "Sign-ups this week")
    }
    Row(gap: .sm, align: .center) {
        Text("Errors by day")
        Sparkline([4, 2, 7, 1, 0, 3, 2], label: "Errors by day").bar
    }
}''',
        "variations": [
            ("Its size and colour", '''Sparkline([3, 5, 4, 8, 6, 9], label: "Latency", width: 160, height: 32, color: "#0F766E")'''),
        ],
    },
    "RelativeTime": {
        "code": '''Stack(gap: .xs) {
    Row(gap: .xs) { Text("Deployed")  RelativeTime(@2026-11-01T09:30Z) }
    Row(gap: .xs) { Text("Created")  RelativeTime(@2026-01-15) }
}''',
        "variations": [
            ("From data, as an ISO string", '''state comment = { author: "Ada", at: "2026-11-16T08:00:00Z" }
Row(gap: .xs) { Text("{comment.author} replied")  RelativeTime(comment.at) }'''),
        ],
    },
    "Table": {
        "code": '''state regions = [{ name: "fra1", latency: 42, status: "Healthy" }, { name: "iad1", latency: 88, status: "Healthy" }, { name: "sin1", latency: 190, status: "Degraded" }]
Table(caption: "Regions") {
    Table.Head {
        Table.Row { Table.Cell("Region")  Table.Cell("Latency")  Table.Cell("Status") }
    }
    Table.Body {
        for r in regions by r.name {
            Table.Row {
                Table.Cell(r.name).header
                Table.Cell("{r.latency} ms")
                Table.Cell { Badge(r.status, tone: if r.status == "Healthy" { .success } else { .warning }) }
            }
        }
    }
}''',
        "variations": [
            ("Written out by hand", '''Table(caption: "Plans") {
    Table.Head { Table.Row { Table.Cell("Plan")  Table.Cell("Price") } }
    Table.Body {
        Table.Row { Table.Cell("Free")  Table.Cell("$0") }
        Table.Row { Table.Cell("Pro")  Table.Cell("$12 a month") }
    }
}'''),
        ],
    },
    "List": {
        "code": '''Stack(gap: .md) {
    List {
        List.Item { Text("Unlimited projects") }
        List.Item { Text("Custom domains") }
        List.Item { Text("Priority support").bold }
    }
    List.ordered {
        List.Item { Code("wf init shop") }
        List.Item { Text("Write your first page") }
        List.Item { Code("wf serve") }
    }
}''',
        "variations": [
            ("Drawn from a list", '''state groceries = ["Milk", "Eggs", "Bread"]
List { for g in groceries by g { List.Item { Text(g) } } }'''),
        ],
    },
    "Badge": {
        "code": '''Row(gap: .sm, align: .center) {
    Heading("Billing").h3
    Badge("Beta").info
    Badge("3 overdue").danger.pill
}''',
        "variations": [
            ("Every tone", '''Row(gap: .sm) {
    Badge("Default")
    Badge("Primary").primary
    Badge("Secondary").secondary
    Badge("Live").success
    Badge("Failed").danger
    Badge("Pending").warning
    Badge("New").info
}'''),
            ("A pill, as a count", '''Row(gap: .xs, align: .center) { Text("Inbox")  Badge("12").primary.pill }'''),
            ("A tone that follows state", '''state failing = true
Badge(if failing { "Failing" } else { "Passing" }, tone: if failing { .danger } else { .success })'''),
        ],
    },
    "Tag": {
        "code": '''Stack(gap: .sm) {
    Heading("Slow roasting, explained").h3
    Row(gap: .xs) {
        Tag("Cooking")
        Tag("Technique")
        Tag("Long read")
    }
}''',
        "variations": [
            ("Drawn from a list", '''state topics = ["rust", "web", "compilers"]
Row(gap: .xs) { for topic in topics by topic { Tag(topic) } }'''),
        ],
    },
    "Avatar": {
        "code": '''Row(gap: .md, align: .center) {
    Avatar(src: "/icon-512.png", alt: "Ada Lovelace").lg
    Avatar(initials: "GH").primary
    Avatar(initials: "AT").sm
}''',
        "variations": [
            ("Beside a name", '''Row(gap: .sm, align: .center) {
    Avatar(initials: "AL").primary
    Stack(gap: .xs) { Text("Ada Lovelace").bold  Text("Maintainer").muted.sm }
}'''),
            ("Every size", '''Row(gap: .sm, align: .center) {
    Avatar(initials: "SM").sm
    Avatar(initials: "MD").md
    Avatar(initials: "LG").lg
}'''),
        ],
    },
    "Tooltip": {
        "code": '''Row(gap: .sm) {
    Tooltip("Copy the link") { IconButton(icon: "copy", label: "Copy link") }
    Tooltip("Edit the page", shortcut: "mod+e") { IconButton(icon: "edit", label: "Edit") }
    Tooltip("Delete it for good", placement: .bottom, delay: 200) {
        IconButton(icon: "trash", label: "Delete").danger
    }
}''',
        "variations": [
            ("Around a button, with its shortcut", '''Tooltip("Save your changes", shortcut: "mod+s") { Button("Save").primary }'''),
            ("Placed to the side", '''Tooltip("Collapse the sidebar").right { IconButton(icon: "chevron-left", label: "Collapse") }'''),
        ],
    },
    "Tree": {
        "code": '''state file = "src/pages/Home.wf"
Tree(items: [
    { path: "src", name: "src", children: [
        { path: "src/App.wf", name: "App.wf" },
        { path: "src/pages", name: "pages", children: [
            { path: "src/pages/Home.wf", name: "Home.wf" },
            { path: "src/pages/About.wf", name: "About.wf" },
        ] },
    ] },
    { path: "webfluent.app.json", name: "webfluent.app.json" },
], children: f => f.children ?? [], key: f => f.path, bind: file, open: ["src", "src/pages"], label: "Files") {
    row(f) {
        Icon(if f.children != null { "folder" } else { "file" })
        Text(f.name)
    }
}
Text("Open: {file}").muted''',
        "variations": [
            ("Several chosen", '''state picked: [String] = []
Tree(items: [
    { id: "eu", name: "Europe", children: [{ id: "fra1", name: "Frankfurt" }, { id: "lhr1", name: "London" }] },
], children: r => r.children ?? [], key: r => r.id, bind: picked, label: "Regions").multiple {
    row(r) { Text(r.name) }
}'''),
        ],
    },
    "DataTable": {
        "code": '''state deploys = [{ id: "d1", service: "checkout", region: "fra1", minutes: 4, status: "Live" }, { id: "d2", service: "search", region: "iad1", minutes: 11, status: "Failed" }, { id: "d3", service: "accounts", region: "sin1", minutes: 7, status: "Live" }]
state chosen: [String] = []
DataTable(rows: deploys, key: d => d.id, bind: chosen, label: "Deploys").multiple {
    DataTable.Column("Service", value: d => d.service).sortable
    DataTable.Column("Region", value: d => d.region)
    DataTable.Column("Minutes", value: d => d.minutes).sortable
    DataTable.Column("Status") {
        cell(d) { Badge(d.status, tone: if d.status == "Live" { .success } else { .danger }) }
    }
}
Text("{chosen.length} chosen").muted''',
        "variations": [
            ("What it shows with no rows", '''state deploys = [{ id: "d1", service: "checkout", status: "Live" }]
DataTable(rows: deploys.filter(d => d.status == "Paused"), key: d => d.id, label: "Paused deploys") {
    empty { EmptyState(icon: "list", title: "Nothing is paused") }
    DataTable.Column("Service", value: d => d.service)
}'''),
            ("One row at a time, filtered and paged", '''state q = ""
state picked = ""
state people = [{ id: "1", name: "Ada Lovelace", team: "Compiler" }, { id: "2", name: "Grace Hopper", team: "Runtime" }, { id: "3", name: "Alan Turing", team: "Compiler" }]
Input(bind: q, label: "Search").search
DataTable(rows: people, key: p => p.id, bind: picked, label: "People", filter: q, pageSize: 2) {
    DataTable.Column("Name", value: p => p.name).sortable
    DataTable.Column("Team", value: p => p.team)
}'''),
        ],
    },
    "Chart": {
        "code": '''Chart(kind: .bar, data: [
    { month: "Jan", web: 41, store: 22 },
    { month: "Feb", web: 48, store: 25 },
    { month: "Mar", web: 56, store: 24 },
    { month: "Apr", web: 61, store: 30 },
], x: "month", y: ["web", "store"], unit: "k").legend''',
        "variations": [
            ("A line over time", '''Chart(kind: .line, data: [
    { week: "W1", users: 120 }, { week: "W2", users: 180 }, { week: "W3", users: 260 }, { week: "W4", users: 310 },
], x: "week", y: "users", height: 220)'''),
            ("A donut with its labels", '''Chart(kind: .donut, data: [
    { channel: "Search", share: 52 }, { channel: "Direct", share: 31 }, { channel: "Social", share: 17 },
], x: "channel", y: "share", unit: "%").labels'''),
            ("Stacked, in your colours", '''Chart(kind: .bar, data: [
    { q: "Q1", new: 30, renewals: 52 }, { q: "Q2", new: 41, renewals: 55 },
], x: "q", y: ["new", "renewals"], colors: ["#0F766E", "#B45309"]).stacked.legend'''),
        ],
    },
    "QrCode": {
        "code": '''Card.outlined {
    Row(gap: .md, align: .center) {
        QrCode("https://example.com/tickets/QX-4821")
        Stack(gap: .xs) {
            Text("Ticket QX-4821").bold
            Text("Show this at the door.").muted
        }
    }
}''',
        "variations": [
            ("In your colours", '''QrCode("https://example.com/menu", color: "#1E3A8A", background: "#F8FAFC")'''),
        ],
    },
    "Callout": {
        "code": '''Callout("Every finding has a code, and a page that says how to fix it.", title: "Before you push").tip {
    Link("Read about the checks", to: "/errors")
}''',
        "variations": [
            ("Every kind", '''Stack(gap: .sm) {
    Callout("Drafts are kept for 30 days.").note
    Callout("Press mod+k to search.").tip
    Callout("A route change drops a route store.").important
    Callout("The key is shown only once.").warning
    Callout("Deleting a project deletes its builds.").caution
}'''),
            ("A title of your own", '''Callout.warning {
    Callout.Title { Text("Rotating keys")  Badge("Beta").info }
    Text("The old key keeps working for an hour.")
}'''),
        ],
    },
    "Kbd": {
        "code": '''Stack(gap: .xs) {
    Row(gap: .xs, align: .center) { Text("Press")  Kbd("mod+k")  Text("to search the docs.") }
    Row(gap: .xs, align: .center) { Text("Send with")  Kbd("shift+Enter")  Text("and close with")  Kbd("Escape") }
}''',
        "variations": [
            ("A chord", '''Row(gap: .xs, align: .center) { Text("Save all:")  Kbd("mod+k mod+s") }'''),
        ],
    },
    "EmptyState": {
        "code": '''state count = 0
EmptyState(icon: "folder", title: "No projects yet", text: "A project holds your pages, components and stores.") {
    Button("New project").primary { on click { count = count + 1 } }
    Link("Read the guide", to: "/learn")
}
if count > 0 { Text("{count} project(s) asked for").muted }''',
        "variations": [
            ("A search with no results", '''state q = "zebra"
EmptyState(icon: "search", title: "No results for “{q}”", text: "Try a shorter word.") {
    Button("Clear the search") { on click { q = "" } }
}'''),
        ],
    },
    "Stat": {
        "code": '''Grid(columns: { base: 1, md: 3 }, gap: .md) {
    Stat("Revenue", value: 48210, format: .currency, change: 0.124, trend: [31, 40, 28, 44, 52, 48])
    Stat("Active users", value: 18400, format: .compact, change: 0.031, trend: [15, 16, 16, 17, 18, 18])
    Stat("Churn", value: 0.021, format: .percent, change: -0.004)
}''',
        "variations": [
            ("Counting up as it arrives", '''Stat("Orders today", value: 1284, format: .integer, count: "600ms")'''),
            ("Money in its own currency", '''Stat("Average order", value: €42.50, change: 0.08)'''),
            ("A figure written as text", '''Stat("Uptime", value: "99.98%")'''),
        ],
    },
    "CodeBlock": {
        "code": '''CodeBlock("""
    page Home(path: "/", title: "Home") \\{
        state count = 0
        Heading("Welcome").h1
        Button("+1").primary \\{ on click \\{ count = count + 1 \\} \\}
    \\}
    """, language: "wf", title: "Home.wf", highlight: [2, 4]).copy.lines''',
        "variations": [
            ("A tab a file", '''CodeBlock(files: [
    { name: "Home.wf", code: #"page Home(path: "/") { Text("Hi") }"#, language: "wf" },
    { name: "styles.css", code: ".hero { padding: 2rem; }", language: "css" },
]).copy'''),
            ("A shell command", '''CodeBlock("wf init shop -t spa\\ncd shop && wf serve", language: "bash").copy'''),
        ],
    },
    "Chip": {
        "code": '''state langs = ["Rust", "TypeScript", "Go", "Python"]
state openOnly = false
Stack(gap: .sm, align: .start) {
    Row(gap: .xs) {
        for l in langs by l {
            Chip(l, icon: "code").removable { on remove { langs = langs.filter(x => x != l) } }
        }
    }
    Chip("Open issues only", selected: openOnly) { on click { openOnly = !openOnly } }
    Text("{langs.length} languages, open only: {openOnly}").muted
}''',
        "variations": [
            ("A filter row of toggles", '''state picked: [String] = ["web"]
Row(gap: .xs) {
    for f in ["web", "api", "worker"] by f {
        Chip(f, selected: picked.includes(f)) {
            on click { picked = if picked.includes(f) { picked.filter(x => x != f) } else { picked.concat([f]) } }
        }
    }
}'''),
        ],
    },
    "VisuallyHidden": {
        "code": '''Row(gap: .md, align: .center) {
    Link(to: "/reference") {
        Text("Read more")
        VisuallyHidden("about the reference")
    }
    Row {
        Text("★★★★☆", aria-hidden: "true")
        VisuallyHidden("Rated 4 out of 5")
    }
}''',
        "variations": [
            ("A label only a screen reader needs", '''Stack(gap: .xs) {
    VisuallyHidden { Heading("Search results").h3 }
    Text("12 pages match.")
}'''),
        ],
    },
    "Accordion": {
        "code": '''Accordion.single {
    Accordion.Item("How long does shipping take?") {
        Text("Two to four working days in the EU, a week elsewhere.")
    }
    Accordion.Item("Can I return an order?", open: true) {
        Text("Within thirty days, unused and in its box.")
        Link("Start a return", to: "/learn")
    }
    Accordion.Item("Do you ship abroad?") {
        Text("To 40 countries; the checkout lists them.")
    }
}''',
        "variations": [
            ("Several open at once", '''Accordion {
    Accordion.Item("Account", open: true) { Text("Your name and email.") }
    Accordion.Item("Billing", open: true) { Text("Your plan and invoices.") }
}'''),
            ("A title of your own", '''Accordion {
    Accordion.Item("Errors") {
        Accordion.Summary { Text("Errors ")  Badge("3").danger.pill }
        Accordion.Panel { Text("Three pages failed to build.") }
    }
}'''),
        ],
    },
})
