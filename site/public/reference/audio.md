# Audio

> Audio player. transcript links what was said, which is what a reader who cannot hear it needs.

The page: https://webfluent.monzeromer.dev/reference/audio

Audio player. transcript links what was said, which is what a reader who cannot hear it needs.

```wf
Audio(src: String, controls: Bool, autoplay: Bool, …)
```

## Example

```wf
Stack(gap: .sm) {
    Heading("Episode 12: Shipping 7.0").h3
    Text("Forty minutes on types, results and what changed.").muted
    Audio(src: "/media/episode-12.mp3", transcript: "/media/episode-12.txt").controls
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The audio's address |
| `controls:` | `Bool` | The browser's playback controls; without them the player shows nothing. Written as a flag, `.controls`, or `controls: true`. |
| `autoplay:` | `Bool` | Start playing when shown; most browsers refuse until the reader has interacted with the page. Written as a flag, `.autoplay`, or `autoplay: true`. |
| `loop:` | `Bool` | Start again when it ends. Written as a flag, `.loop`, or `loop: true`. |
| `transcript:` | `String` | The address of what was said, linked as `Read the transcript` beneath the player; without it, `A09` |

## Flags

`.controls` `.autoplay` `.loop`

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
