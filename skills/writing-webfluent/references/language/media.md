### Media

| Component | Usage |
|-----------|-------|
| `Image` | `Image(src: "/photo.jpg", alt: "Description")`, or `Image(hero, alt: "…", sizes: "…", placeholder: .blur)` with an `image` the program declares; a literal `src:` naming a PNG, JPEG, GIF or WebP in `public/` is processed by the build just as an `image` is — decoded asynchronously; the first image on a page loads eagerly at high priority (it is usually the largest paint), the rest lazily. `loading:` overrides |
| `Video` | `Video(src: "/video.mp4", captions: "/clip.en.vtt", poster: "/frame.jpg").controls` — `muted`, `loop`, `playsinline`; without `captions` it draws an `A09` |
| `Audio` | `Audio(src: "/talk.mp3", transcript: "/talk.txt").controls` — the transcript is a link beneath the player |
| `Frame` | `Frame(src: "/preview/", title: "Preview", sandbox: "allow-scripts") { on message(m) { heard = m.data } }` — another page inside this one; `on message` hears only what that page posts; without `title:` it draws an `A17`. `Frame(srcdoc: html, title: "Example", sandbox: "")` draws the HTML it is given instead of an address, sandboxed as `sandbox:` says (HTML from a value with no `sandbox:` gets `sandbox=""` and a `V03`), and draws the new HTML when a value that reads state changes; `on message` checks the sender's origin too, so a page the frame navigated to is not heard |
| `Icon` | `Icon("home")` or `Icon("search").lg.primary` — 83 built-in SVG icons rendered inline (see [Icon System](#icon-system)) |
| `Carousel` | `Carousel(autoplay: true, interval: 5000) { Carousel.Slide { Image(src: "...") } }` — slide track with dots and autoplay |
| `Lightbox` | `Lightbox(label: "Trip") { Image(hero, alt: "The harbour")  Image(src: "/hills.jpg", alt: "The hills", data-full: "/hills-large.jpg") }` — every picture in its block a button that opens it full screen: the arrow keys (the other way round right to left) and a swipe go between them, Home and End to the ends, a double press or `+` zooms and `-` comes back, Escape closes and the focus goes back to the picture; it opens a picture's `data-full:`, else its largest size; `previous:`, `next:`, `close:`, `zoom:` and `of:` say it in your words |
| `Gallery` | `Gallery(columns: 3, ratio: 4 / 3) { for p in photos by p.id { Image(src: p.src, alt: p.alt) } }` — pictures in a grid, each cropped to one shape (unset, square), as many across as fit unless `columns:` says; in a `Lightbox`, each opens full screen; `columns:` takes one per breakpoint too, `{ base: 2, md: 4 }` |

