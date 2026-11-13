# Show a carousel

> Show slides one at a time with Carousel and Carousel.Slide: previous and next buttons, a dot per slide, and autoplay that pauses.

The page: https://webfluent.monzeromer.dev/recipes/show-a-carousel

Show pictures or panels one at a time, with buttons to move between them
and, if you want, advancing on their own. Wrap each slide in a
`Carousel.Slide` inside a `Carousel`.

This page shows three photographs that advance every five seconds:

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

## How it works

- **Each `Carousel.Slide` is one slide**, shown one at a time.
- **The controls come with it.** A carousel has a named previous button, a
  named next button, and a dot per slide.
- **`label:` names it.** Its `label:` names the region for screen readers.
  It is a `region` described as a carousel, unless you set its `role:` or
  `aria-roledescription:` yourself.
- **`autoplay` advances it** every `interval:` milliseconds. With
  `autoplay`, the carousel also has a button that stops it. It pauses while
  the pointer or the keyboard focus is on it, and it does not advance at
  all for a reader who asked for reduced motion.
