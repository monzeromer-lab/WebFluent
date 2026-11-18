# Video

> Video player. Without captions it draws an A09 warning: a video nobody can hear is a video nobody can follow.

The page: https://webfluent.monzeromer.dev/reference/video

Video player. Without captions it draws an A09 warning: a video nobody can hear is a video nobody can follow.

```wf
Video(src: String, controls: Bool, autoplay: Bool, …)
```

## Example

```wf
Video(src: "/media/launch.mp4", poster: "/og.png", captions: "/media/launch.en.vtt",
      width: 1280, height: 720).controls.playsinline
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The video's address |
| `controls:` | `Bool` | The browser's playback controls; without them, `A09`. Written as a flag, `.controls`, or `controls: true`. |
| `autoplay:` | `Bool` | Start playing when shown; browsers allow it only for a `.muted` video. Written as a flag, `.autoplay`, or `autoplay: true`. |
| `poster:` | `String` | The picture shown before it plays |
| `muted:` | `Bool` | Start with the sound off. Written as a flag, `.muted`, or `muted: true`. |
| `loop:` | `Bool` | Start again when it ends. Written as a flag, `.loop`, or `loop: true`. |
| `playsinline:` | `Bool` | Play where it sits on a phone, rather than full screen. Written as a flag, `.playsinline`, or `playsinline: true`. |
| `captions:` | `String` | A WebVTT file of captions, which becomes a default `<track kind="captions">`; without it, `A09` |
| `width:` | `Number` | The player's `width` in px |
| `height:` | `Number` | The player's `height` in px |

## Flags

`.controls` `.autoplay` `.muted` `.loop` `.playsinline`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on play`: A video or audio started playing
- `on pause`: A video or audio paused
- `on ended`: A video or audio reached its end
- `on timeupdate`: A playing video or audio moved on

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [D01](https://webfluent.monzeromer.dev/errors/D01.md): A file the project does not have
- [A09](https://webfluent.monzeromer.dev/errors/A09.md): Media nobody can follow without sound

Before WebFluent 5.0.
