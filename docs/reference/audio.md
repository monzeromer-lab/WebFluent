# Audio

> Audio player. transcript links what was said, which is what a reader who cannot hear it needs.

The page: https://webfluent.monzeromer.dev/reference/audio

Audio player. transcript links what was said, which is what a reader who cannot hear it needs.

```wf
Audio(src: String, controls: Bool, autoplay: Bool, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | Audio URL |
| `controls:` | `Bool` | Show the browser's playback controls |
| `autoplay:` | `Bool` | Start playing when shown |
| `loop:` | `Bool` | Start again when it ends |
| `transcript:` | `String` | A link to what was said, shown beneath the player |

## Flags

`.controls` `.autoplay` `.loop`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on play`: A video or audio started playing
- `on pause`: A video or audio paused
- `on ended`: A video or audio reached its end
- `on timeupdate`: A playing video or audio moved on

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
Audio(src: "/episode-1.mp3", transcript: "/episode-1.txt").controls
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [D01](https://webfluent.monzeromer.dev/errors/D01.md): A file the project does not have
- [A09](https://webfluent.monzeromer.dev/errors/A09.md): Media nobody can follow without sound

Before WebFluent 5.0.
