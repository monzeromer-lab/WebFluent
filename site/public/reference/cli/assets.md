# wf assets

> What `public/` holds — each file's kind, size and dimensions, and where the project names it — or what the media pipeline will make of one image

The page: https://webfluent.monzeromer.dev/reference/cli/assets

What `public/` holds — each file's kind, size and dimensions, and where the project names it — or what the media pipeline will make of one image

```bash
wf assets [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--plan PATH` | What the pipeline will make of this image of `public/`: `/hero.png` |
| `--unused` | Only the files nothing names |
| `--json` | Print JSON, for tools |

Lists every file of `public/` and where the project uses it, or what the image pipeline makes of one picture.

```bash
wf assets
wf assets --unused
wf assets --plan /hero.png
wf assets --json
```

For each file it prints its kind (image, font, video, audio, document, code, data), its size, a picture's dimensions, and every place the project names it: a prop the registry types as a file (`Image`'s `src`, `Video`'s `poster` and `captions`), a page's `image:`, a `const`, an `image` declaration, a `data` file's strings, a value in `webfluent.app.json`, a stylesheet's `url(…)`, a project script, or a manifest. A file a browser or a host asks for by name — `favicon.ico`, `robots.txt`, `.well-known/…` — counts as used. Anything else nothing names counts as unused.

`--plan PATH` applies the project's `build.media` to one picture: each width it writes, in each format, with the file's name and size — for example *3 widths × 1 format · 3 files · about 84 kB*. A size is exact once `wf build` has written the file (it is read from `.wf-cache/media/`), or when the picture is small enough to encode on the spot. Otherwise it is estimated from strips of the picture encoded at that width, and says `about`. The plan names a format the build cannot write, with the reason, and a width it skips because the file would be no smaller than the original. With `--json`, the same sentences are in the `said` field.

The library has the same as `webfluent::build::assets::{list, plan}`. It reads through a `Vfs`, so an editor's unsaved buffers count.

See also: [Add a picture](https://webfluent.monzeromer.dev/recipes/add-a-picture.md).
