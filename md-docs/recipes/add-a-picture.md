---
kind: "recipe"
route: "/recipes/add-a-picture"
group: "look"
description: "To show a picture, put the file in `public/` and name it in `Image(src:)` with a site-relative path."
from: ["/docs/guide/media#add-a-picture-from-public", "/docs/guide/media#process-a-picture-with-image", "/docs/guide/media#tune-the-image-pipeline", "/docs/guide/media#troubleshooting"]
---
# Add a picture, sized for every screen

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

## Troubleshooting

| What you see | Why, and what to do |
|---|---|
| The page jumps when a picture arrives | The picture is an SVG, a URL or a path built at run time, which the build cannot measure. Give it `width:` and `height:` |
| An icon shows as a word | The name is not one of the built-in icons (`V08`), or it arrives at run time and the build left its glyph out. Use a listed name, or set `build.runtime` to `"full"` |
| `A09` on a `Video` or `Audio` | Add `captions:` to the video or `transcript:` to the recording |
| `A17` on a `Frame` | Add a `title:` that says what the frame shows |
| A frame from another site stays blank | Its origin is not in `meta.frame`, so the policy blocks it |
| `wf assets --plan` lists widths it will not write | Those files would save nothing; the original serves at those widths |
