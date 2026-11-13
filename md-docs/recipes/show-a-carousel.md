---
kind: "recipe"
route: "/recipes/show-a-carousel"
group: "look"
description: "To show slides one at a time, wrap each in a `Carousel.Slide` inside a `Carousel`."
from: ["/docs/guide/media#show-a-carousel"]
---
# Show a carousel

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
