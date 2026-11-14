# Video

> Video player. Without captions it draws an A09 warning: a video nobody can hear is a video nobody can follow.

The page: https://webfluent.monzeromer.dev/reference/video

Video player. Without captions it draws an A09 warning: a video nobody can hear is a video nobody can follow.

```wf
Video(src: String, controls: Bool, autoplay: Bool, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Video URL |
| `controls:` | `Bool` | Show the browser's playback controls |
| `autoplay:` | `Bool` | Start playing when shown |
| `poster:` | `String` | The frame shown before it plays |
| `muted:` | `Bool` | Start with the sound off |
| `loop:` | `Bool` | Start again when it ends |
| `playsinline:` | `Bool` | Play where it sits, rather than full screen |
| `captions:` | `String` | A WebVTT file of captions, which becomes a `<track>` |
| `width:` | `Number` | Intrinsic width |
| `height:` | `Number` | Intrinsic height |

## Flags

`.controls` `.autoplay` `.muted` `.loop` `.playsinline`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on play`: A video or audio started playing
- `on pause`: A video or audio paused
- `on ended`: A video or audio reached its end
- `on timeupdate`: A playing video or audio moved on

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
Video(src: "/demo.mp4", captions: "/demo.en.vtt", poster: "/demo.jpg").controls
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [D01](https://webfluent.monzeromer.dev/errors/D01.md): A file the project does not have
- [A09](https://webfluent.monzeromer.dev/errors/A09.md): Media nobody can follow without sound

Before WebFluent 5.0.
