"""Forms: the controls a reader types, picks and chooses with, and what groups them."""

EXAMPLES = {
    "Input": {
        "code": '''state name = "Ada Lovelace"
state email = ""
Stack(gap: .md) {
    Input(bind: name, label: "Full name", placeholder: "Ada Lovelace").text
    Input(bind: email, label: "Work email", hint: "We send the invoice here.", placeholder: "ada@example.com").email.required
    Text(if email == "" { "Hello, {name}" } else { "Hello, {name}: we write to {email}" }).muted
}''',
        "variations": [
            ("A password with a button that shows it", '''state password = ""
Input(bind: password, label: "Password", hint: "At least 12 characters").password.reveal'''),
            ("With an error message", '''state handle = "ada lovelace"
derived handleError = if handle.includes(" ") { "A handle has no spaces." } else { "" }
Input(bind: handle, label: "Handle", error: handleError).text'''),
            ("A number between two ends", '''state seats = 5
Input(bind: seats, label: "Seats", min: 1, max: 50, step: 1).number
Text("{seats} seats")'''),
            ("Disabled", '''state workspace = "acme-prod"
Input(bind: workspace, label: "Workspace", hint: "Ask an owner to rename it.").disabled'''),
        ],
    },
    "Select": {
        "code": '''state region = "eu-west"
Select(bind: region, label: "Region", hint: "Where your data is stored.") {
    Select.Option("Europe (Ireland)", value: "eu-west")
    Select.Option("US East (Virginia)", value: "us-east")
    Select.Option("Asia Pacific (Tokyo)", value: "ap-northeast")
}
Text("Chosen: {region}").muted''',
        "variations": [
            ("With an error message", '''state role = ""
derived roleError = if role == "" { "Choose a role for this person." } else { "" }
Select(bind: role, label: "Role", error: roleError) {
    Select.Option("Choose…", value: "")
    Select.Option("Viewer", value: "viewer")
    Select.Option("Editor", value: "editor")
    Select.Option("Owner", value: "owner")
}'''),
            ("Disabled", '''state plan = "team"
Select(bind: plan, label: "Plan").disabled {
    Select.Option("Free", value: "free")
    Select.Option("Team", value: "team")
}'''),
        ],
    },
    "Checkbox": {
        "code": '''state terms = false
state newsletter = true
Stack(gap: .sm) {
    Checkbox(bind: terms, label: "I accept the terms of service")
    Checkbox(bind: newsletter, label: "Send me the monthly newsletter")
    Text(if terms { "Ready to sign up." } else { "Accept the terms to go on." }).muted
}''',
        "variations": [
            ("Disabled", '''state backups = true
Checkbox(bind: backups, label: "Daily backups (included in your plan)").disabled'''),
        ],
    },
    "Radio": {
        "code": '''state shipping = "standard"
Stack(gap: .sm) {
    Radio(bind: shipping, value: "standard", label: "Standard — 5 to 7 days, free")
    Radio(bind: shipping, value: "express", label: "Express — 2 days, $9")
    Radio(bind: shipping, value: "overnight", label: "Overnight — $24")
    Text("Chosen: {shipping}").muted
}''',
        "variations": [
            ("One that cannot be chosen", '''state size = "m"
Radio(bind: size, value: "m", label: "Medium")
Radio(bind: size, value: "xl", label: "Extra large (sold out)").disabled'''),
        ],
    },
    "Switch": {
        "code": '''state replies = true
state digest = false
Stack(gap: .sm) {
    Switch(bind: replies, label: "Email me when someone replies")
    Switch(bind: digest, label: "Send a weekly digest")
    Text("Replies {if replies { "on" } else { "off" }}, digest {if digest { "on" } else { "off" }}").muted
}''',
        "variations": [
            ("Disabled", '''state sso = true
Switch(bind: sso, label: "Single sign-on (set by your organisation)").disabled'''),
        ],
    },
    "Slider": {
        "code": '''state volume = 60
Slider(bind: volume, min: 0, max: 100, step: 5, label: "Volume")
Text("Volume: {volume}%").muted''',
        "variations": [
            ("A range, with two thumbs", '''state price: [Number] = [20, 80]
Slider(bind: price, min: 0, max: 200, step: 10, label: "Price").range
Text("From ${price[0] ?? 0} to ${price[1] ?? 0}")'''),
            ("Disabled", '''state brightness = 40
Slider(bind: brightness, min: 0, max: 100, label: "Brightness").disabled'''),
        ],
    },
    "DatePicker": {
        "code": '''state start: Date? = @2026-11-17
DatePicker(bind: start, label: "Start date", min: "2026-01-01", max: "2026-12-31")
if let d = start {
    Text("Starts {d:.date(long)}").muted
} else {
    Text("No date chosen").muted
}''',
        "variations": [
            ("Empty until chosen", '''state due: Date? = null
DatePicker(bind: due, label: "Due date")'''),
            ("Disabled", '''state renewal: Date? = @2027-01-01
DatePicker(bind: renewal, label: "Renews on").disabled'''),
        ],
    },
    "FileUpload": {
        "code": '''state photo: File? = null
FileUpload(bind: photo, accept: "image/*", label: "Profile picture")
if let p = photo {
    Text("Chosen: {p.name}").muted
} else {
    Text("No picture chosen").muted
}''',
        "variations": [
            ("A drop zone for several files", '''state files: [File] = []
FileUpload(bind: files, accept: ".pdf,.png,.jpg", label: "Attachments").dropzone.multiple
Text("{files.length} file(s) attached")'''),
            ("Disabled", '''state contract: File? = null
FileUpload(bind: contract, accept: ".pdf", label: "Signed contract").disabled'''),
        ],
    },
    "Form": {
        "code": '''state email = ""
state password = ""
state sent = ""
validate email {
    required "Enter your email"
    email "That is not an email address"
}
validate password {
    required
    minLength(8) "Use at least 8 characters"
}
Form(bind: form, show: .onBlur) {
    on submit { sent = "Signed in as {email}" }
    Input(bind: email, label: "Email").email
    Input(bind: password, label: "Password").password
    Button("Sign in", type: .submit, disabled: !form.valid).primary
}
Text(if sent == "" { "Not signed in yet." } else { sent }).muted''',
        "variations": [
            ("Messages only after a submit", '''state name = ""
validate name { required "Tell us your name" }
Form(bind: form, show: .onSubmit) {
    on submit { log(name) }
    Input(bind: name, label: "Name")
    Button("Save", type: .submit).primary
}'''),
            ("Starting again", '''state title = "Untitled"
Form(bind: draft) {
    on submit { log(title) }
    Input(bind: title, label: "Title")
    Row(gap: .sm) {
        Button("Publish", type: .submit).primary
        Button("Reset") { on click { draft.reset() } }
    }
}'''),
        ],
    },
    "Textarea": {
        "code": '''state feedback = ""
Textarea(bind: feedback, label: "What could we do better?", hint: "Be as specific as you like.", placeholder: "The export was slow when…", rows: 4, maxLength: 280)
Text("{feedback.length} of 280 characters").muted''',
        "variations": [
            ("Grows with its text", '''state note = ""
Textarea(bind: note, label: "Release notes", rows: 2).autosize'''),
            ("With an error message", '''state bio = ""
derived bioError = if bio.length < 20 { "Write at least 20 characters." } else { "" }
Textarea(bind: bio, label: "About you", error: bioError, rows: 3).required'''),
            ("Disabled", '''state terms = "These terms apply to every workspace."
Textarea(bind: terms, label: "Terms", rows: 3).disabled'''),
        ],
    },
    "Field": {
        "code": '''state color = "#0f766e"
derived colorError = if color == "#ffffff" { "White disappears on the page." } else { "" }
Field(label: "Brand colour", hint: "Used for buttons and links.", error: colorError) {
    ColorPicker(bind: color, label: "Brand colour", swatches: ["#0f766e", "#1d4ed8", "#b91c1c", "#ffffff"])
}
Text("Chosen: {color}").muted''',
        "variations": [
            ("Around a control of your own", '''state plan = "team"
Field(label: "Plan", hint: "You can change it at any time.") {
    ToggleGroup(bind: plan, label: "Plan") {
        ToggleGroup.Item("Free", value: "free")
        ToggleGroup.Item("Team", value: "team")
        ToggleGroup.Item("Scale", value: "scale")
    }
}'''),
        ],
    },
    "Fieldset": {
        "code": '''state street = ""
state city = ""
state postcode = ""
Fieldset(legend: "Shipping address") {
    Input(bind: street, label: "Street")
    Row(gap: .md) {
        Input(bind: city, label: "City")
        Input(bind: postcode, label: "Postcode")
    }
}
Text("{street}, {postcode} {city}").muted''',
    },
    "InputGroup": {
        "code": '''state search = ""
state domain = "acme"
Stack(gap: .md) {
    InputGroup {
        Icon("search")
        Input(bind: search, placeholder: "Search the docs", aria-label: "Search the docs")
        InputGroup.Clear
    }
    InputGroup {
        InputGroup.Addon("https://")
        Input(bind: domain, aria-label: "Subdomain")
        InputGroup.Addon(".example.com")
    }
    Text("Searching for “{search}” on https://{domain}.example.com").muted
}''',
    },
    "RadioGroup": {
        "code": '''state plan = "team"
RadioGroup(bind: plan, label: "Plan").cards {
    RadioGroup.Option("Free", value: "free", hint: "For trying it out")
    RadioGroup.Option("Team", value: "team", hint: "Up to 20 people, $12 each")
    RadioGroup.Option("Scale", value: "scale", hint: "Unlimited people, single sign-on")
}
Text("Chosen: {plan}").muted''',
        "variations": [
            ("As a plain list", '''state billing = "monthly"
RadioGroup(bind: billing, label: "Billing") {
    RadioGroup.Option("Monthly", value: "monthly")
    RadioGroup.Option("Yearly", value: "yearly", hint: "Two months free")
}'''),
        ],
    },
    "NumberInput": {
        "code": '''state tickets = 2
NumberInput(bind: tickets, min: 1, max: 10, step: 1, label: "Tickets", hint: "Ten at most per order.")
Text("Total: ${tickets * 45}").muted''',
        "variations": [
            ("Larger steps", '''state storage = 50
NumberInput(bind: storage, min: 10, max: 1000, step: 10, label: "Storage (GB)")'''),
        ],
    },
    "Calendar": {
        "code": '''state delivery: Date? = @2026-11-17
Calendar(bind: delivery, min: @2026-11-17, disabled: d => d.weekday() >= 6, label: "Delivery day").single
if let d = delivery {
    Text("Delivered on {d:.date(long)}").muted
}''',
        "variations": [
            ("A range", '''state stay = { start: @2026-12-20, end: @2026-12-27 }
Calendar(bind: stay, mode: .range, label: "Your stay")
Text("{stay.start:.date(medium)} to {stay.end:.date(medium)}")'''),
            ("Several days", '''state days: [Date] = [@2026-11-18, @2026-11-20]
Calendar(bind: days, mode: .multiple, label: "Days you can attend")
Text("{days.length} day(s) chosen")'''),
        ],
    },
    "Listbox": {
        "code": '''state fruit = "apple"
Listbox(bind: fruit, label: "Fruit") {
    Listbox.Option("Apple", value: "apple")
    Listbox.Option("Banana", value: "banana")
    Listbox.Option("Cherry", value: "cherry")
    Listbox.Option("Mango", value: "mango")
}
Text("Chosen: {fruit}").muted''',
        "variations": [
            ("Several at once", '''state toppings: [String] = ["cheese"]
Listbox(bind: toppings, label: "Toppings").multiple {
    Listbox.Option("Cheese", value: "cheese")
    Listbox.Option("Mushrooms", value: "mushrooms")
    Listbox.Option("Olives", value: "olives")
}
Text("Chosen: {toppings.join(", ")}")'''),
        ],
    },
    "Combobox": {
        "code": '''state city = "lis"
Combobox(bind: city, options: [
    { id: "ams", name: "Amsterdam" },
    { id: "ber", name: "Berlin" },
    { id: "lis", name: "Lisbon" },
    { id: "mad", name: "Madrid" },
    { id: "par", name: "Paris" },
], text: c => c.name, key: c => c.id, label: "City", placeholder: "Type a city")
Text("Chosen: {city}").muted''',
        "variations": [
            ("Several, each a chip", '''state reviewers: [String] = ["ada"]
Combobox(bind: reviewers, options: [
    { id: "ada", name: "Ada Lovelace" },
    { id: "grace", name: "Grace Hopper" },
    { id: "alan", name: "Alan Turing" },
], text: p => p.name, key: p => p.id, label: "Reviewers").multiple'''),
        ],
    },
    "CheckboxGroup": {
        "code": '''state channels: [String] = ["email"]
CheckboxGroup(bind: channels, label: "Notify me by", all: "Every channel") {
    CheckboxGroup.Option("Email", value: "email")
    CheckboxGroup.Option("Text message", value: "sms")
    CheckboxGroup.Option("Push notification", value: "push")
}
Text("Chosen: {channels.join(", ")}").muted''',
    },
    "InputOTP": {
        "code": '''state code = ""
derived left = 6 - code.length
InputOTP(bind: code, length: 6, label: "Code from your authenticator app")
Text(if code.length == 6 { "Checking {code}…" } else { "{left} digit(s) to go" }).muted''',
    },
    "TagInput": {
        "code": '''state tags: [String] = ["design", "frontend"]
TagInput(bind: tags, label: "Tags", placeholder: "Type a tag, then Enter")
Text("{tags.length} tag(s): {tags.join(", ")}").muted''',
    },
    "Rating": {
        "code": '''state stars = 4
Rating(bind: stars, max: 5, label: "Your rating")
Text("You gave it {stars} of 5").muted''',
        "variations": [
            ("Half stars", '''state score = 3.5
Rating(bind: score, max: 5, label: "Score").half'''),
            ("Read-only: an average shown, not asked", '''state average = 4
Rating(bind: average, max: 5, label: "Average rating").readonly'''),
        ],
    },
    "DateRangePicker": {
        "code": '''state period = { start: @2026-11-01, end: @2026-11-17 }
DateRangePicker(bind: period, label: "Report period", presets: [.today, .last7, .last30, .thisMonth, .lastMonth])
Text("{period.start:.date(medium)} to {period.end:.date(medium)}").muted''',
        "variations": [
            ("Held between two dates", '''state stay = { start: @2026-12-20, end: @2026-12-27 }
DateRangePicker(bind: stay, label: "Stay", min: @2026-11-17, max: @2027-11-17)'''),
        ],
    },
    "ColorPicker": {
        "code": '''state accent = "#0f766e"
ColorPicker(bind: accent, label: "Accent colour", swatches: ["#0f766e", "#1d4ed8", "#7c3aed", "#b91c1c", "#c2410c"])
Button("Preview").primary { style { background: {accent} } }
Text("Chosen: {accent}").muted''',
    },
    "TransferList": {
        "code": '''state picked: [String] = ["ada"]
TransferList(bind: picked, options: [
    { id: "ada", name: "Ada Lovelace" },
    { id: "grace", name: "Grace Hopper" },
    { id: "alan", name: "Alan Turing" },
    { id: "katherine", name: "Katherine Johnson" },
], text: p => p.name, key: p => p.id, label: "Reviewers", from: "Everyone")
Text("Reviewers: {picked.join(", ")}").muted''',
    },
    "Editable": {
        "code": '''state project = "Quarterly report"
Editable(bind: project, label: "Project name", placeholder: "Name the project")
Text("Saved as “{project}”").muted''',
    },
}
