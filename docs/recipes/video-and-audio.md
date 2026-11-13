# Add video and audio

> Play a video with Video(src:) and sound with Audio(src:), with the browser's controls, captions for the video and a transcript for the recording.

The page: https://webfluent.monzeromer.dev/recipes/video-and-audio

Show a video with `Video(src:)` and play sound with `Audio(src:)`, with the
browser's own playback controls, captions for the video and a transcript for
the recording.

This page plays a demo with captions and a talk with its transcript, both
from `public/`:

```wf
page Media(path: "/", title: "Media", description: "The demo and the launch talk.") {
    Heading("Media").h1
    Video(src: "/demo.mp4", captions: "/demo.en.vtt", poster: "/demo.jpg").controls
    Audio(src: "/talk.mp3", transcript: "/talk.txt").controls
}
```

## How it works

- The `.controls` flag shows the browser's playback controls.
- `captions:` names a WebVTT file. The build writes it as a `<track>`, so a
  reader who cannot hear the video can follow it.
- `transcript:` names a text file. The build draws it as a link beneath the
  player.
- `poster:` names the picture a video shows before it plays.

Give every video captions and every recording a transcript. A `Video`
without `captions:`, and an `Audio` without `transcript:`, each draw an
`A09` warning.

## Troubleshooting

| What you see | Why, and what to do |
|---|---|
| `A09` on a `Video` or `Audio` | Add `captions:` to the video or `transcript:` to the recording |
