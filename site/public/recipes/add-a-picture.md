# Add a picture, sized for every screen

> Show a picture from public/ or declared with image: the build writes it at every width, sets its size, and gives it a placeholder while it loads.

The page: https://webfluent.monzeromer.dev/recipes/add-a-picture

Put a picture in `public/` and show it with `Image`, and the build writes it
again at every width a screen may ask for, sets its size so nothing on the
page shifts, and fills its box while it loads. Use this for a photograph, a
hero picture or a decorative image.

This page declares a hero picture with `image` and shows it with a blurred
placeholder, prints its real size and colour, and shows a second picture
and a decorative one straight from `public/`:

**Needs:** `hero.jpg`, `office.jpg` and `deco.png` in `public/` (any
pictures will do); without them the build stops at `image hero`. It claims
`/`: paste it into a new project, or replace the starter's
`src/pages/Home.wf` with it.

```wf
image hero = "hero.jpg"

page Home(path: "/", title: "Home", description: "The front page.") {
    Heading("Home").h1
    Image(hero, alt: "The team at the launch",
          sizes: "(max-width: 768px) 100vw, 1200px",
          placeholder: .blur)
    Text("It is {hero.width} by {hero.height}, and mostly {hero.color}.")
    Image(src: "/office.jpg", alt: "The office on a quiet morning")
    Image(src: "/deco.png", alt: "")
}
```

## How it works

### A picture from `public/`

Files in `public/` are served from the site's root, so `public/office.jpg`
is `/office.jpg`. Name it in `Image(src:)` with that site-relative path.

Give every `Image` an `alt:` that says what the picture shows. A decorative
picture takes `alt: ""`, so a screen reader skips it.

When `src:` is a literal path into `public/` that names a PNG, JPEG, GIF or
WebP file, the build processes the file the same way it processes an
[`image` declaration](#a-picture-declared-with-image):

- It writes the picture again at each width the page may ask for.
- It sets `width` and `height` from the file, so nothing on the page shifts
  when the picture arrives.
- It serves the picture as a `<picture>` element. `sizes:` and
  `placeholder:` work here too.

The build uses an SVG file, a URL, or a path built at run time as it is
written. Give those a `width:` and `height:` when you know them, as in
`Image(src: "/hero.jpg", alt: "The team at the launch", width: 1200, height: 630)`.
To turn processing off, set `build.media.pipeline` to `false`.

The first image on a page loads eagerly at high priority, because it is
usually the largest thing the reader sees first. Every later image loads
lazily. Set `loading:` on an `Image` to choose for yourself.

### A picture declared with `image`

To use a picture's real size and colour in your page, declare it with
`image`, as `hero` is above. The declaration hands the build the file
itself. The build looks for the path in the project's folder, then in
`src/`, then in `public/`.

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

A PDF or a slide deck embeds the real picture rather than drawing an empty
box.

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
image does no image work.

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
| `A01` on an `Image` | Add an `alt:` that says what it shows, or `alt: ""` for a decorative picture |
| `wf assets --plan` lists widths it will not write | Those files would save nothing; the original serves at those widths |
