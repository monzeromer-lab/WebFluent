"""Media, typography, and what a PDF or a slide deck is made of."""

EXAMPLES = {
    # ---- Media -----------------------------------------------------------
    "Lightbox": {
        "code": '''Lightbox(label: "Launch week", previous: "Previous photo", next: "Next photo", close: "Close", zoom: "Zoom", of: "of") {
    Gallery(columns: 3, ratio: 4 / 3) {
        Image(src: "/og.png", alt: "The launch card, as it was shared")
        Image(src: "/icon-512.png", alt: "The app icon on a white tile")
        Image(src: "/og.png", alt: "The launch card, cropped to its title") {
            style { object-position: 20% 50% }
        }
    }
}''',
        "variations": [
            ("A larger picture for the full-screen view", '''Lightbox(label: "Screenshots") {
    Image(src: "/icon-512.png", alt: "The app icon", data-full: "/og.png")
}'''),
            ("Pictures in a row of their own", '''Lightbox(label: "Brand assets") {
    Row(gap: .md) {
        Image(src: "/icon-512.png", alt: "The icon", width: 120, height: 120)
        Image(src: "/og.png", alt: "The sharing card", width: 228, height: 120)
    }
}'''),
        ],
    },
    "Gallery": {
        "code": '''Gallery(columns: { base: 2, md: 4 }, ratio: 1) {
    Image(src: "/og.png", alt: "The sharing card")
    Image(src: "/icon-512.png", alt: "The app icon")
    Image(src: "/og.png", alt: "The sharing card, its left edge") {
        style { object-position: 0% 50% }
    }
    Image(src: "/og.png", alt: "The sharing card, its right edge") {
        style { object-position: 100% 50% }
    }
}''',
        "variations": [
            ("Wide tiles, as many as fit", '''Gallery(ratio: 16 / 9) {
    Image(src: "/og.png", alt: "The sharing card")
    Image(src: "/og.png", alt: "The sharing card again")
}'''),
            ("Each opens full screen inside a Lightbox", '''Lightbox(label: "Photos") {
    Gallery(columns: 2, ratio: 4 / 3) {
        Image(src: "/og.png", alt: "The sharing card")
        Image(src: "/icon-512.png", alt: "The app icon")
    }
}'''),
        ],
    },
    "Image": {
        "code": '''Stack(gap: .md) {
    Image(src: "/og.png", alt: "WebFluent's sharing card: the name over a dark field",
          sizes: "(max-width: 768px) 100vw, 640px", placeholder: .blur).rounded
    Row(gap: .md, align: .center) {
        Image(src: "/icon-512.png", alt: "The app icon", width: 64, height: 64).circle
        Text("Pictures from public/ are resized to WebP at every width a page asks for.").muted.sm
    }
}''',
        "variations": [
            ("Decorative: an empty alt, so a screen reader skips it", '''Image(src: "/icon-512.png", alt: "", width: 32, height: 32)'''),
            ("The average colour while it loads, and loaded at once", '''Image(src: "/og.png", alt: "The sharing card", placeholder: .color, loading: .eager)'''),
            ("From an address elsewhere: used as written", '''Image(src: "https://example.com/team.jpg", alt: "The team at the launch", width: 1200, height: 630).lazy'''),
        ],
    },
    "Video": {
        "code": '''Video(src: "/media/launch.mp4", poster: "/og.png", captions: "/media/launch.en.vtt",
      width: 1280, height: 720).controls.playsinline''',
        "variations": [
            ("A silent loop in place of a GIF", '''Video(src: "/media/editor-demo.mp4", captions: "/media/editor-demo.en.vtt").autoplay.muted.loop.playsinline.controls'''),
        ],
        "preview": False,
    },
    "Audio": {
        "code": '''Stack(gap: .sm) {
    Heading("Episode 12: Shipping 7.0").h3
    Text("Forty minutes on types, results and what changed.").muted
    Audio(src: "/media/episode-12.mp3", transcript: "/media/episode-12.txt").controls
}''',
        "variations": [
            ("Background music that loops", '''Audio(src: "/media/ambient.mp3", transcript: "/media/ambient.txt").controls.loop'''),
        ],
        "preview": False,
    },
    "Frame": {
        "code": '''state version = 2
state heard = "nothing yet"
Stack(gap: .sm) {
    Row(gap: .sm) {
        Button("Version 1") { on click { version = 1 } }
        Button("Version 2") { on click { version = 2 } }
    }
    Frame(srcdoc: "<p style='font-family:sans-serif'>Release notes, version {version}</p>",
          title: "Release notes preview", sandbox: "") {
        style { width: 100%; height: 120px; border: 1px solid $color-border; border-radius: 8px }
        on load { heard = "version {version} loaded" }
    }
    Text("Frame says: {heard}").muted.sm
}''',
        "variations": [
            ("A page of the site, and what it posts", '''state heard = ""
Frame(src: "/playground", title: "The playground", sandbox: "allow-scripts allow-same-origin") {
    on message(m) { heard = JSON.stringify(m.data) }
}'''),
            ("Loaded only when scrolled near, allowed full screen", '''Frame(src: "/showcase", title: "The showcase", allow: "fullscreen").lazy'''),
        ],
    },
    "Icon": {
        "code": '''Row(gap: .lg, align: .center) {
    Row(gap: .xs, align: .center) { Icon("check").success  Text("Deployed") }
    Row(gap: .xs, align: .center) { Icon("warning").warning  Text("2 warnings") }
    Row(gap: .xs, align: .center) { Icon("alert-circle").danger  Text("1 failed") }
    Row(gap: .xs, align: .center) { Icon("clock").muted  Text("Queued") }
}''',
        "variations": [
            ("Sizes", '''Row(gap: .md, align: .center) {
    Icon("star").sm
    Icon("star")
    Icon("star").lg.primary
}'''),
            ("Beside a button's words", '''Button("Download report") { Icon("download")  Text("Download report") }'''),
        ],
    },
    "Carousel": {
        "code": '''Carousel(label: "Featured work", autoplay: true, interval: 6000) {
    Carousel.Slide {
        Image(src: "/og.png", alt: "The sharing card")
    }
    Carousel.Slide {
        Image(src: "/og.png", alt: "The sharing card, its title in close-up") {
            style { object-fit: cover; object-position: 15% 50%; aspect-ratio: 1200 / 630 }
        }
    }
    Carousel.Slide {
        Center {
            style { aspect-ratio: 1200 / 630; background: $color-surface }
            Image(src: "/icon-512.png", alt: "The app icon", width: 160, height: 160)
        }
    }
}''',
        "variations": [
            ("Slides of words, moved by hand", '''Carousel(label: "What readers say") {
    Carousel.Slide { Blockquote("It built the first time.") }
    Carousel.Slide { Blockquote("The errors tell you what to do.") }
}'''),
        ],
    },

    # ---- Typography --------------------------------------------------------
    "Highlight": {
        "code": '''state q = "state"
Stack(gap: .sm) {
    Input(bind: q, label: "Search the guide").search
    Highlight("Pages, components and stores hold state; a derived value reads it.", query: q)
    Highlight("A state change repaints only what reads it.", query: q)
}''',
        "variations": [
            ("In a list of results", '''state q = "form"
for title in ["Forms and validation", "Formatting numbers", "Stores"] {
    Text { Highlight(title, query: q) }
}'''),
        ],
    },
    "Text": {
        "code": '''Stack(gap: .xs) {
    Text("Billing").uppercase.sm.muted
    Text("Your plan renews on 1 December.").lg
    Text("Payment failed: the card was declined.").danger.bold
    Text("Updated 3 minutes ago").muted.sm.italic
}''',
        "variations": [
            ("Tones", '''Stack {
    Text("Primary").primary
    Text("Secondary").secondary
    Text("Saved").success
    Text("Needs attention").warning
    Text("Failed").danger
    Text("Did you know?").info
}'''),
            ("Alignment", '''Stack {
    Text("Left").left
    Text("Centred").center
    Text("Right").right
}'''),
            ("As a heading or a subtitle, without being one", '''Stack {
    Text("Release notes").heading
    Text("Everything in 7.0.1").subtitle
}'''),
            ("Live: a splice that reads state", '''state count = 3
Text("{count} items in your cart").bold.underline'''),
        ],
    },
    "Heading": {
        "code": '''Stack(gap: .xs) {
    Heading("Pricing").h1
    Heading("Plans for every team").h2.muted
    Heading("What every plan has").h3
}''',
        "variations": [
            ("Centred, over a section", '''Heading("Frequently asked questions").h2.center'''),
            ("The deeper levels", '''Stack {
    Heading("Settings").h2
    Heading("Notifications").h3
    Heading("Email").h4
    Heading("Weekly digest").h5
    Heading("Footnote").h6
}'''),
        ],
    },
    "Code": {
        "code": '''Stack(gap: .sm) {
    Text { Text("Start the dev server with ")  Code("wf serve")  Text(", then open the page.") }
    Text { Text("A page opens with ")  Code("page Home(path: \\"/\\")", language: "wf")  Text(".") }
}''',
        "variations": [
            ("A block of code is a CodeBlock", '''CodeBlock("wf build --stats", language: "bash")'''),
        ],
    },
    "Markdown": {
        "code": '''Markdown("""
    ## Getting started

    Install `wf`, then:

    - write a page in **src/pages**
    - run `wf serve` and open it
    - read [the guide](/learn) when you are stuck

    ```wf
    page Home(path: "/", title: "Home") { Heading("Hello").h1 }
    ```
    """)''',
        "variations": [
            ("Somebody else's text: entities decoded, links drawn as words", '''state docs = "Returns `List&lt;T&gt;`, as [the guide](/learn) says."
Markdown(docs).inert'''),
            ("A table", '''Markdown("""
    | Plan | Seats |
    |---|--:|
    | Free | 1 |
    | Pro | 10 |
    """)'''),
        ],
    },
    "Unsafe": {
        "code": '''Unsafe.Html(sanitize("<p>From the <strong>CMS</strong>: <a href='/learn'>read the guide</a>.</p><script>alert(1)</script>"))''',
        "variations": [
            ("Markup from state, sanitised as it changes", '''state body = "<p>Draft <em>saved</em>.</p>"
Unsafe.Html(sanitize(body))'''),
        ],
    },
    "Blockquote": {
        "code": '''Blockquote {
    Text("We moved the whole docs site over in a weekend, and the compiler caught every broken link.").italic
    Text("Ada Lovelace, platform team").muted.sm
}''',
        "variations": [
            ("One line, given as text", '''Blockquote("Make it work, make it right, make it fast.")'''),
        ],
    },

    # ---- PDF ---------------------------------------------------------------
    "Document": {
        "code": '''Document(size: "A4", margin: 56, title: "Invoice INV-2041", author: "Northwind Ltd",
         subject: "Invoice", keywords: "invoice, november", lang: "en") {
    Header { Text("Northwind Ltd · Invoice INV-2041").muted.sm.right }
    Footer { Text("Northwind Ltd · 12 Harbour Street · Bristol").muted.sm.center }
    Heading("Invoice INV-2041").h1
    Text("Issued 17 November 2026 · Due in 30 days").muted
    Table(caption: "Items") {
        Table.Head { Table.Row { Table.Cell("Item")  Table.Cell("Amount") } }
        Table.Body {
            Table.Row { Table.Cell("Design review")  Table.Cell("€1,200.00") }
            Table.Row { Table.Cell("Hosting, November")  Table.Cell("€90.00") }
        }
    }
    Text("Total: €1,290.00").bold.right
}''',
        "variations": [
            ("Landscape Letter, by flag", '''Document(title: "Quarterly figures").Letter.landscape {
    Heading("Quarterly figures").h1
}'''),
        ],
        "preview": False,
    },
    "Section": {
        "code": '''Document(title: "Service agreement") {
    Section {
        Heading("1. Payment").h2
        Paragraph("Invoices are due within 30 days of issue.")
    }
    Section {
        style { break-inside: avoid }
        Heading("2. Signatures").h2
        Paragraph("Signed for Northwind Ltd: ______________________")
    }
}''',
        "preview": False,
    },
    "Paragraph": {
        "code": '''Document(title: "Q3 report") {
    Heading("Summary").h1
    Paragraph("Revenue grew 15% on the quarter, led by the new self-serve plan.")
    Paragraph("Churn held at 2.1%; support response times halved.").muted
    Paragraph("Figures are unaudited.").sm.italic
}''',
        "preview": False,
    },
    "Header": {
        "code": '''Document(title: "Annual report 2026") {
    Header(on: .rest) {
        Row(justify: .between) {
            Text("Northwind Ltd").bold.sm
            Text("Annual report 2026").muted.sm
        }
    }
    Heading("Annual report 2026").h1
    PageBreak
    Heading("The year").h2
}''',
        "variations": [
            ("Different on left and right pages", '''Document(title: "Handbook") {
    Header(on: .odd) { Text("Handbook").muted.sm.right }
    Header(on: .even) { Text("Northwind Ltd").muted.sm.left }
    Heading("Handbook").h1
}'''),
        ],
        "preview": False,
    },
    "Footer": {
        "code": '''Document(title: "Invoice INV-2041") {
    Footer {
        Row(justify: .between) {
            Text("Northwind Ltd · VAT GB123456789").muted.sm
            Text("Page {page} of {pages}").muted.sm
        }
    }
    Heading("Invoice INV-2041").h1
}''',
        "variations": [
            ("Only on the first page", '''Document(title: "Letter") {
    Footer(on: .first) { Text("Registered in England, no. 01234567").muted.sm.center }
    Heading("Dear Ada").h1
}'''),
        ],
        "preview": False,
    },
    "Background": {
        "code": '''Document(title: "Q3 report") {
    Background(on: .first) {
        Stack { style { height: 100%; background: linear-gradient(160deg, #0B1220, #1E3A8A) } }
    }
    Stack {
        style { color: #ffffff; padding-top: 240px }
        Heading("Q3 report").h1
        Text("Northwind Ltd · October 2026")
    }
}''',
        "variations": [
            ("A band down every page's side", '''Document(title: "Handbook") {
    Background { Stack { style { width: 24px; height: 100%; background: #0F766E } } }
    Heading("Handbook").h1
}'''),
        ],
        "preview": False,
    },
    "Watermark": {
        "code": '''Document(title: "Contract draft") {
    Watermark("DRAFT")
    Heading("Service agreement").h1
    Paragraph("This draft is for review and is not binding.")
}''',
        "preview": False,
    },
    "TableOfContents": {
        "code": '''Document(title: "Employee handbook") {
    Heading("Employee handbook").h1
    TableOfContents(levels: 2, title: "Contents")
    PageBreak
    Heading("Your first week").h2
    Heading("Equipment").h3
    Heading("Time off").h2
}''',
        "preview": False,
    },
    "PageBreak": {
        "code": '''Document(title: "Report") {
    Heading("Summary").h1
    Paragraph("The quarter in one page.")
    PageBreak
    Heading("Appendix: the figures").h2
    Paragraph("Every number, month by month.")
}''',
        "preview": False,
    },

    # ---- Slides ------------------------------------------------------------
    "Presentation": {
        "code": '''Presentation {
    TitleSlide("Q3 review", subtitle: "Northwind Ltd · October 2026")
    SectionSlide("The numbers").primary
    Slide {
        Heading("Highlights").h1
        List {
            List.Item { Text("Revenue up 15%") }
            List.Item { Text("Three new enterprise customers") }
        }
    }
    TwoColumn {
        Container { Heading("Wins").h2  Text("Self-serve plan launched") }
        Container { Heading("Risks").h2  Text("Hiring behind plan") }
    }
    ImageSlide(src: "/og.png", caption: "The launch card")
}''',
        "preview": False,
    },
    "Slide": {
        "code": '''Presentation {
    Slide {
        style { background: linear-gradient(135deg, #1E3A8A, #0F172A); color: #ffffff }
        Heading("Revenue").h1
        Text("Up 15% on last quarter, to €2.4M.").lg
        Text("Source: finance, unaudited").sm
    }
}''',
        "preview": False,
    },
    "TitleSlide": {
        "code": '''Presentation {
    TitleSlide("Shipping WebFluent 7", subtitle: "Platform team · 17 November 2026")
}''',
        "preview": False,
    },
    "SectionSlide": {
        "code": '''Presentation {
    SectionSlide("Part one: the numbers").primary
}''',
        "variations": [
            ("The other tones", '''Presentation {
    SectionSlide("Wins").success
    SectionSlide("Risks").warning
    SectionSlide("Incidents").danger
    SectionSlide("Next steps").info
}'''),
        ],
        "preview": False,
    },
    "TwoColumn": {
        "code": '''Presentation {
    TwoColumn {
        Container {
            Heading("Before").h2
            Text("A build took four minutes.")
        }
        Container {
            Heading("After").h2
            Text("It takes forty seconds.")
        }
    }
}''',
        "preview": False,
    },
    "ImageSlide": {
        "code": '''Presentation {
    ImageSlide(src: "/og.png", caption: "The card every shared link shows")
}''',
        "preview": False,
    },
}
