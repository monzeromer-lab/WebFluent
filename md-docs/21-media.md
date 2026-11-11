# 21. Media

<!--
route: guide/media
group: ui
blurb: Add pictures the build resizes for every screen, video with captions, audio with a transcript, icons, carousels and frames.
description: Add images from public/ and the image pipeline, Video with captions, Audio with a transcript, the 83 built-in icons, Carousel and Frame.
-->

This chapter shows you how to put pictures, sound, video, icons and other
pages on a WebFluent page. The build resizes your pictures for every screen
and reserves their space, so the page does not jump while they load.

In this chapter:

- Add a picture from `public/`, and let the build process it.
- Declare a picture with `image` and read its size and colour.
- Tune the image pipeline, and see what it will write.
- Add video with captions and audio with a transcript.
- Show an icon, a carousel, and another page in a frame.

## Add a picture from `public/`

To show a picture, put the file in `public/` and name it in `Image(src:)`
with a site-relative path. Files in `public/` are served from the site's
root, so `public/hero.jpg` is `/hero.jpg`.

This page shows a photograph, a decorative picture, a video and a talk:

```wf
page Media(path: "/") {
    Image(src: "/hero.jpg", alt: "The team at the launch", width: 1200, height: 630)
    Image(src: "/deco.png", alt: "")
    Video(src: "/demo.mp4", captions: "/demo.en.vtt").controls
    Audio(src: "/talk.mp3", transcript: "/talk.txt").controls
}
```

Give every `Image` an `alt:` that says what the picture shows. A decorative
picture takes `alt: ""`, so a screen reader skips it.

When `src:` is a literal path into `public/` that names a PNG, JPEG, GIF or
WebP file, the build processes the file the same way it processes an
[`image` declaration](#process-a-picture-with-image):

- It writes the picture again at each width the page may ask for.
- It sets `width` and `height` from the file, so nothing on the page shifts
  when the picture arrives.
- It serves the picture as a `<picture>` element. `sizes:` and
  `placeholder:` work here too.

The build uses an SVG file, a URL, or a path built at run time as it is
written. Give those a `width:` and `height:` when you know them. To turn
processing off, set `build.media.pipeline` to `false`.

The first image on a page loads eagerly at high priority, because it is
usually the largest thing the reader sees first. Every later image loads
lazily. Set `loading:` on an `Image` to choose for yourself.

## Process a picture with `image`

To use a picture's real size and colour in your page, declare it with
`image`. The declaration hands the build the file itself. The build looks
for the path in the project's folder, then in `src/`, then in `public/`.

This page declares a hero picture, shows it with a blurred placeholder, and
prints its size and average colour:

```wf
image hero = "hero.jpg"

page Home(path: "/", title: "Home", description: "The front page.") {
    Image(hero, alt: "The team at the launch",
          sizes: "(max-width: 768px) 100vw, 1200px",
          placeholder: .blur)
    Text("It is {hero.width} by {hero.height}, and mostly {hero.color}.")
}
```

The build reads the picture's real size and average colour, writes it again
at every width in `media.widths` that is smaller than the original, and
gives the page a `<picture>` with:

- a `<source>` per format, each with a `srcset` of the widths, so the
  browser picks the file it needs for the space `sizes:` describes;
- `width` and `height` from the file, so the box has the right shape before
  any of the picture arrives and **nothing on the page moves**;
- a placeholder while the picture loads: `.blur` inlines a copy sixteen
  pixels wide as a data URI, `.color` fills the box with the average colour,
  and `.none` does neither;
- a content hash in every file's name, so a host can cache each file forever.

The name `hero` is also a value. You can read `hero.src`, `hero.width`,
`hero.height`, `hero.color`, `hero.placeholder` and `hero.srcset`.

The attributes any element takes (`id:`, `aria-*`, `data-*`, `ref:`, an
`on load` handler) go on the `<img>` inside the `<picture>`. That is where
its `alt:` is and where `load` fires. The width, the height and the sources
stay the build's.

## Tune the image pipeline

To change which formats and widths the build writes, set `build.media` in
`webfluent.app.json`. These are the defaults:

```json
{ "build": { "media": {
    "formats": ["webp"],
    "widths": [480, 960, 1440, 1920],
    "quality": 78,
    "pipeline": true
} } }
```

| Key | What it does |
|---|---|
| `formats` | The formats each picture is written in, best first: `webp`, `png`, `jpeg` |
| `widths` | The widths written. A width larger than the original is skipped; the build never scales a picture up |
| `quality` | The quality of a lossy file. WebP and JPEG are written lossily at this quality; PNG is written losslessly |
| `pipeline` | `false` copies each file as it is, as `public/` files always were |

The build writes a file only when it saves something. A file that would be
no smaller than the original is not written: the original, which the
`<img>` falls back to, serves at that width instead. A format that has no
width left gets no `<source>`. A JPEG that was already saved small often
ends up this way.

The build caches its work in `.wf-cache/media/`, so a build that changes no
image does no image work. A PDF or a slide deck embeds the real picture
rather than drawing an empty box.

To see what the pipeline will make of a picture before it does it, run
`wf assets --plan`:

```bash
wf assets --plan /hero.jpg
```

It prints each width and format, what each file comes to, and the widths it
will not write because they save nothing. `wf assets` with no option lists
every file in `public/` with where it is used, and counts the files nothing
uses.

## Add video and audio

To show a video, use `Video(src:)`; to play sound, use `Audio(src:)`. Add
the `.controls` flag to show the browser's playback controls.

Give every video captions and every recording a transcript:

- `captions:` names a WebVTT file. The build writes it as a `<track>`, so
  a reader who cannot hear the video can follow it.
- `transcript:` names a text file. The build draws it as a link beneath the
  player.

The first example in this chapter shows both. A `Video` without `captions:`,
and an `Audio` without `transcript:`, each draw an `A09` warning.

## Show an icon

To draw an icon, write `Icon("name")`. The icon is an SVG drawn inline in
the current text colour, so it follows the theme. The `.sm` and `.lg` flags
size it, and a tone flag such as `.primary` colours it.

This page shows icons on their own, on an icon-only button, and in a
sidebar:

```wf
page Inbox(path: "/", title: "Inbox", description: "Messages, with icons.") {
    Heading("Inbox").h1
    Row(gap: .sm) {
        Icon("mail")
        Icon("bell").lg.primary
        IconButton(icon: "close", label: "Close the inbox")
    }
    Sidebar {
        Sidebar.Item(to: "/", icon: "home") { Text("Home") }
    }
}
```

`IconButton` is a button that shows only an icon. Its `label:` is its
accessible name for screen readers, and also its tooltip unless you give it
a `title:`. `Sidebar.Item` takes an `icon:` too.

These are the 83 built-in icons:

- **General**: `home`, `menu`, `search`, `close`, `user`, `settings`,
  `check`, `plus`, `minus`, `edit`, `trash`, `star`, `heart`, `mail`,
  `bell`, `download`, `upload`, `eye`, `link`, `calendar`, `filter`, `info`,
  `warning`, `arrow-left`, `arrow-right`, `chevron-down`, `chevron-right`,
  `chevron-left`, `logout`, `copy`, `sun`, `moon`.
- **For an editor or a tool**: `folder`, `folder-plus`, `folder-off`,
  `file`, `file-plus`, `refresh`, `alert-circle`, `clock`, `dot`, `diff`,
  `book`, `git-pull-request`, `pin`, `git-branch`, `more-horizontal`,
  `window-minimize`, `window-maximize`, `outline`, `help-circle`,
  `collapse-all`, `locate`, `chevron-up`, `globe`, `external-link`,
  `panel-down`, `panel-up`, `code`, `monitor`, `git-commit`, `play`,
  `layers`, `insert`, `grid`, `list`, `square`, `pill`, `image`, `sparkle`,
  `lock`, `type`, `pointer`, `reach`, `frame`, `line`,
  `pipette`, `comment`, `hand`, `zoom`, `ruler`, `eye-off`, `unlock`.

A name that is not in the list draws as the word itself, and the compiler
warns with `V08`.

> [!IMPORTANT]
> The build keeps only the glyphs your program names. If an icon's name
> arrives at run time, set `"build": { "runtime": "full" }` so every glyph
> ships.

For an icon that is not built in, put an SVG file in `public/` and show it
with `Image(src: "/icons/rocket.svg", alt: "")`.

## Show a carousel

To show slides one at a time, wrap each in a `Carousel.Slide` inside a
`Carousel`. This page shows three photographs that advance every five
seconds:

```wf
page Gallery(path: "/", title: "Gallery", description: "Three pictures.") {
    Heading("Gallery").h1
    Carousel(label: "Recent work", autoplay: true, interval: 5000) {
        Carousel.Slide { Image(src: "/one.jpg", alt: "A bridge at dusk") }
        Carousel.Slide { Image(src: "/two.jpg", alt: "The harbour from above") }
        Carousel.Slide { Image(src: "/three.jpg", alt: "Rain on the market") }
    }
}
```

A carousel has a named previous button, a named next button, and a dot per
slide. Its `label:` names the region for screen readers. It is a `region`
described as a carousel, unless you set its `role:` or
`aria-roledescription:` yourself.

With `autoplay`, the carousel also has a button that stops it. It pauses
while the pointer or the keyboard focus is on it, and it does not advance at
all for a reader who asked for reduced motion.

## Show another page in a frame

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
Content-Security-Policy refuses it ([Security](30-security.md)).

### Draw HTML you give the frame

To draw HTML instead of loading an address (an example a page renders, or a
preview of what a reader is writing), use `srcdoc:`. The frame draws it
sandboxed as `sandbox:` says, so `sandbox: ""` draws it with no script and
no reach into your page. A value that reads state draws the new HTML each
time it changes, and the pre-rendered page has the HTML before any script
runs.

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

## Troubleshooting

| What you see | Why, and what to do |
|---|---|
| The page jumps when a picture arrives | The picture is an SVG, a URL or a path built at run time, which the build cannot measure. Give it `width:` and `height:` |
| An icon shows as a word | The name is not one of the built-in icons (`V08`), or it arrives at run time and the build left its glyph out. Use a listed name, or set `build.runtime` to `"full"` |
| `A09` on a `Video` or `Audio` | Add `captions:` to the video or `transcript:` to the recording |
| `A17` on a `Frame` | Add a `title:` that says what the frame shows |
| A frame from another site stays blank | Its origin is not in `meta.frame`, so the policy blocks it |
| `wf assets --plan` lists widths it will not write | Those files would save nothing; the original serves at those widths |

## Next

[Accessibility](22-accessibility.md): build pages that work with a keyboard, a screen reader or voice control: what the built-ins do, what the compiler checks, and what is still yours to do.
