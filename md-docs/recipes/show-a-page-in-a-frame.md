---
kind: "recipe"
route: "/recipes/show-a-page-in-a-frame"
group: "look"
description: "To show another page inside yours (a video player, a map, a preview your app builds), use `Frame`."
from: ["/docs/guide/media#show-another-page-in-a-frame"]
---
# Show another page in a frame

To show another page inside yours (a video player, a map, a preview your
app builds), use `Frame`. Give it a `title:` that says what it shows: a
screen reader announces a frame by its title, and a frame without one draws
an `A17` warning.

This page embeds a video player and a preview of the site's own, and shows
what the preview sends back:

```wf
page Talk(path: "/", title: "The talk", description: "The launch talk, and what the preview says.") {
    state heard = ""
    Heading("The talk").h1
    Frame(src: "https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ", title: "The launch talk", allow: "fullscreen")
    Frame(src: "/preview/", title: "Preview", sandbox: "allow-scripts") {
        on message(m) { heard = m.data }
    }
    Text(heard)
}
```

- `on message(m)` hears what the framed page posts with
  `window.parent.postMessage(…)`, and nothing that any other page posts.
  `m.data` is what it sent.
- `sandbox:` limits what the framed page may do: `"allow-scripts
  allow-forms"` allows scripts and forms, and an empty string allows
  nothing.
- `allow:` names the browser features the framed page may use.

A frame from another origin must be named in `meta.frame`, or the page's
Content-Security-Policy refuses it ([Security](../about/security.md)).

### Draw HTML you give the frame

To draw HTML instead of loading an address (an example a page renders, or a
preview of what a reader is writing), use `srcdoc:`. The frame draws it
sandboxed as `sandbox:` says, so `sandbox: ""` draws it with no script and
no reach into your page. HTML from a value with no `sandbox:` is given
`sandbox=""` — what a value holds is somebody else's document, scripts and
all — and the build names it with a `V03`. A value that reads state draws
the new HTML each time it changes, and the pre-rendered page has the HTML
before any script runs.

A frame's `on message` hears what its own document posts, from the origin
that document has: its address's, or the opaque `"null"` of a sandbox
without `allow-same-origin`. A page the frame was navigated to is no
longer its own.

This page previews a note as the reader types it:

```wf
page Preview(path: "/preview-note", title: "Preview", description: "A note, drawn as it will look.") {
    state note = "Hello"
    Heading("Preview").h1
    Input(bind: note, label: "Note")
    Frame(srcdoc: "<p>{note}</p>", title: "The note, drawn", sandbox: "")
}
```

### Frame your own pages

A page of your own site (`src: "/preview/"`) needs no extra setting. The
site's pages are served refusing to be framed by anyone
(`X-Frame-Options: DENY`). A project that places a `Frame` with a
site-relative `src:` is served `SAMEORIGIN` instead, both by `wf serve` and
in the `_headers` file a build writes. Your site may frame its own pages,
and no other site may.
