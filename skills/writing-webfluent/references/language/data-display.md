### Data Display

| Component | Usage |
|-----------|-------|
| `Card` | `Card.elevated { Card.Header { ... } Card.Body { ... } Card.Footer { ... } }` — variants: `.flat`, `.elevated`, `.outlined` |
| `Tree` | `Tree(items: top, children: f => f.children, key: f => f.path, bind: chosen, label: "Files") { row(f) { Icon("file")  Text(f.name) } }` — nested items as `role="tree"`, one list of rows (`aria-level`, `aria-expanded`, `aria-setsize`, `aria-posinset`): Up/Down through what shows, Right opens or goes in, Left closes or goes to the parent (swapped right to left), Home/End, a letter; Enter or Space chooses its key; `.multiple` adds Shift with an arrow and Ctrl/⌘+A; `open:` the keys open at first |
| `DataTable` | `DataTable(rows: items, key: d => d.id, bind: chosen, label: "Deploys").multiple { empty { … }  DataTable.Column("Service", value: d => d.service).sortable  DataTable.Column("Status") { cell(d) { Badge(d.status) } } }` — a table of records: a `.sortable` column's header is a button with `aria-sort`, its rows slide to their places; `cell(r)` draws a cell (else `value:` as text); `empty` when there are none; `bind:` a box per row (a radio without `.multiple`) and one for all; the header row sticks; the caption is `label:`; rows the build can read are painted, cells and all; `filter: q` keeps the rows whose columns hold the text, `pageSize: 20` a page at a time behind Previous and Next |
| `Table` | `Table(caption: "Deployments") { Table.Head { Table.Row { Table.Cell("Col") } } Table.Body { Table.Row { Table.Cell("Val") } } }` — cells inside `Table.Head` are `<th scope="col">`, as is `Table.Cell("Build").header` anywhere (a header cell a component renders); `caption` is the table's accessible name, rendered visually hidden |
| `List` | `List { List.Item { Text("Item 1") } List.Item { Text("Item 2") } }` — `List.ordered` for numbered |
| `Badge` | `Badge("Label").primary` — tones: `.primary`, `.secondary`, `.success`, `.danger`, `.warning`, `.info`; `.pill` |
| `Tag` | Deprecated since 8.0 (`V13`): `Chip("JavaScript")` draws the same static label |
| `Avatar` | `Avatar(src: "/photo.jpg", alt: "User")` or `Avatar(initials: "MO").primary` — flags: `.sm`, `.md`, `.lg`, `.primary` |
| `Tooltip` | `Tooltip("Save", shortcut: "Ctrl+S", delay: 400, placement: .bottom) { Button("Save").primary }` — wraps children; shows above everything after `delay:` ms of hover (unset, 500), at once on keyboard focus |
| `DescriptionList` | `DescriptionList.grid { DescriptionList.Item("Region", value: "eu-west")  DescriptionList.Item("Status") { Badge("Live").success } }` — terms and their values (`<dl>`), in a column or, `.grid`, each term beside its value |
| `Timeline` | `Timeline { for r in releases by r.version { Timeline.Item(r.version, at: r.date) { Markdown(r.notes) } } }` — events in order (`<ol>`), each a dot on a line, its title, its date (`<time datetime>`) and its block |
| `AvatarGroup` | `AvatarGroup(max: 3, label: "Team") { for p in people by p.id { Avatar(initials: p.initials) } }` — avatars overlapping in a row; past `max:` the rest are one `+2` (said "2 more"), counted again as the list changes |
| `Item` | `Item(title: "Notifications", description: "Sent to your phone") { Item.Media { Icon("bell") }  Item.Actions { Switch(bind: on, aria-label: "Notifications") } }` — a row of a list, a settings page or search results: media, its words and its block, then its actions |
| `Sparkline` | `Sparkline(signups, label: "Sign-ups this week")` or `.bar` — a tiny chart in a line of text, an SVG made when the page is built (`E109` when what it reads is not known then); `width:`, `height:`, `color:` |
| `RelativeTime` | `RelativeTime(post.date)` — `3 minutes ago`, kept current, the exact time its title (`<time datetime title>`); the static paint shows the exact time until the page runs |

