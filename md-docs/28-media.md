# 28. Media

<!--
route: guide/media
group: shipping
blurb: Pictures the build resizes for every screen, video with captions, audio with a transcript, icons and carousels.
description: Images from public/ and the image pipeline, Video with captions, Audio with a transcript, the 55 built-in icons, and Carousel.
-->

## Pictures from `public/`

```wf
page Media(path: "/") {
    Image(src: "/hero.jpg", alt: "The team at the launch", width: 1200, height: 630)
    Image(src: "/deco.png", alt: "")
    Video(src: "/demo.mp4", captions: "/demo.en.vtt").controls
    Audio(src: "/talk.mp3", transcript: "/talk.txt").controls
}
```

Give every `Image` an `alt:` — empty for a decorative one. Files in
`public/` are served from the root; the first image on a page loads
eagerly and later ones lazily.

A `src:` written as a path of `public/` that names a PNG, JPEG, GIF or
WebP is processed by the build, exactly as an `image` declaration (below)
is: written again at each width, given `width` and `height` from the file
so nothing shifts, and served as a `<picture>` — `sizes:` and
`placeholder:` work on it too. An SVG, a URL, or a path built at run time
is used as it is; give those a `width:` and `height:` where you know them.
`build.media.pipeline: false` turns it off.

A `Video` without `captions:` and an `Audio` without `transcript:` each
draw an `A09`: a video nobody can hear is a video nobody can follow.
`captions:` becomes a `<track>`; `transcript:` a link beneath the player.

## `image`: a picture the build processes

Naming a picture with `image` hands the build the file itself. The path is
looked up in the project's folder, then `src/`, then `public/`:

```wf
image hero = "hero.jpg"

page Home(path: "/", title: "Home", description: "The front page.") {
    Image(hero, alt: "The team at the launch",
          sizes: "(max-width: 768px) 100vw, 1200px",
          placeholder: .blur)
    Text("It is {hero.width} by {hero.height}, and mostly {hero.color}.")
}
```

The build reads its real size and its average colour, writes it again at
every width in `media.widths` smaller than the original, and gives the page
a `<picture>`:

- a `<source>` per format, each with a `srcset` of the widths, so the
  browser picks the one it needs for the space `sizes:` describes;
- `width` and `height` from the file, so the box is the right shape before
  a byte of the image has arrived and **nothing on the page moves**;
- a placeholder in the meantime — `.blur` inlines a sixteen-pixel-wide copy
  as a data URI, `.color` fills the box with the average colour, `.none`
  does neither;
- a content hash in every file's name, so a host can cache them forever.

The name is a value: `hero.src`, `.width`, `.height`, `.color`,
`.placeholder`, `.srcset`.

What any element takes — an `id:`, `aria-*`, `data-*`, a `ref:`, an `on
load` — is the `<img>` inside the `<picture>`, where its `alt:` is and its
`load` fires; the width, the height and the sources stay the build's.

```json
{ "build": { "media": {
    "formats": ["webp"],
    "widths": [480, 960, 1440, 1920],
    "quality": 78,
    "pipeline": true
} } }
```

`pipeline: false` copies the file as `public/` always did. The work is
cached in `.wf-cache/media/`, so a build that changes no image does no
image work. A PDF or a slide deck embeds the real picture rather than
drawing a box where one should be.

`wf assets --plan /hero.jpg` says what the pipeline will make of a picture
before it does: each width, each format, and what the files come to. `wf
assets` lists everything in `public/` with where it is used, and counts
what nothing uses.

## Icons

`Icon("name")` draws one of the built-in SVG icons inline, in the current
text colour, so it follows the theme; `.sm`, `.lg` and a tone flag size and
colour it. `IconButton(icon: "close", label: "Close")` is a button that is
only an icon, named for screen readers by its `label:`. `Sidebar.Item`
takes `icon:` too.

The 55 icons: `home`, `menu`, `search`, `close`, `user`, `settings`,
`check`, `plus`, `minus`, `edit`, `trash`, `star`, `heart`, `mail`, `bell`,
`download`, `upload`, `eye`, `link`, `calendar`, `filter`, `info`,
`warning`, `arrow-left`, `arrow-right`, `chevron-down`, `chevron-right`,
`chevron-left`, `logout`, `copy`, `sun`, `moon`; and what an editor or a
tool draws — `folder`, `folder-plus`, `file`, `refresh`, `alert-circle`, `clock`, `dot`, `diff`, `book`, `git-pull-request`, `pin`, `git-branch`, `more-horizontal`, `window-minimize`, `window-maximize`, `outline`, `help-circle`, `collapse-all`, `chevron-up`, `globe`, `external-link`, `panel-down`, `panel-up`.

A name that is not one of them draws as the word, and the build carries
only the glyphs the program names — so an icon whose name arrives at run
time needs `"build": { "runtime": "full" }`. For any other icon, put an SVG
file in `public/` and show it with `Image(src: "/icons/rocket.svg", alt: "")`.

## Carousel

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

A carousel has named previous and next buttons and a dot per slide; its
`label:` names the region for screen readers. It is a `region` called a
carousel unless its `role:` or `aria-roledescription:` says otherwise. With autoplay there is also a
button that stops it; it pauses on its own while the
pointer or focus is on it, and does not run at all for a reader who asked
for reduced motion.

## Frames

`Frame` shows another page inside this one — a video player, a map, a
preview an app builds. Its `title:` says what it shows: a screen reader
announces a frame by its title, and one without it draws an `A17` warning.

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

`on message(m)` hears what the framed page posts with
`window.parent.postMessage(…)` — `m.data` is what it sent — and nothing
another page posts. `sandbox:` limits what the framed page may do
(`"allow-scripts allow-forms"`; empty, nothing at all), and `allow:` names
the features it may use. A frame from another origin is named in
`meta.frame`, or the page's policy refuses it ([Security](23-security.md)).

A page of the site itself (`src: "/preview/"`) needs nothing named: the
pages are served refusing to be framed by anyone (`X-Frame-Options: DENY`),
and a project that places a `Frame` with a site-relative `src:` is served
`SAMEORIGIN` instead — by `wf serve` and in the `_headers` a build writes —
so it may frame its own pages and still no one else may.

## Next

[Deploying](29-deploying.md).
