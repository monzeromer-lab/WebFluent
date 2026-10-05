# Video 5 — one typo, one coded diagnostic

`src/Profile.wf` has one deliberate mistake: `user.nmae`. `wf check` stops on it with `error[T05]`, points at the line and column, and says `Did you mean `name`?` — the editor offers the same fix. Change `nmae` to `name` and the project checks clean.

    wf check            # fails: error[T05]: `User` has no field `nmae`

`tests/video_examples.rs` holds the compiler to the exact message, so what the video shows stays true.
