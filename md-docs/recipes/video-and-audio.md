---
kind: "recipe"
route: "/recipes/video-and-audio"
group: "look"
description: "To show a video, use `Video(src:)`; to play sound, use `Audio(src:)`."
from: ["/docs/guide/media#add-video-and-audio"]
---
# Add video and audio

To show a video, use `Video(src:)`; to play sound, use `Audio(src:)`. Add
the `.controls` flag to show the browser's playback controls.

Give every video captions and every recording a transcript:

- `captions:` names a WebVTT file. The build writes it as a `<track>`, so
  a reader who cannot hear the video can follow it.
- `transcript:` names a text file. The build draws it as a link beneath the
  player.

The first example in this chapter shows both. A `Video` without `captions:`,
and an `Audio` without `transcript:`, each draw an `A09` warning.
