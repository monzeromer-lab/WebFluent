# wf verify

> Load every page of a built project in a real browser: errors, failed requests, paint timing, and which built-ins it drew

The page: https://webfluent.monzeromer.dev/reference/cli/verify

Load every page of a built project in a real browser: errors, failed requests, paint timing, and which built-ins it drew

```bash
wf verify [PATH] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[PATH]` | Project directory (default: current directory) |
| `--json` | Print JSON, for a pipeline |
| `--budget MS` | Fail a route whose first paint is slower than this, in milliseconds |
| `--returning-visitor` | Visit every route again as a reader whose storage holds what the previous build's pages kept |
| `--allow-output-outside` | Let `build.output` name a directory outside the project |
| `--fakes FILE` | Build the site with these fakes answering its endpoints, then load that: for a site whose API is elsewhere |

Loads every page of a built project in a real browser and reports what went wrong there.

```bash
wf build && wf verify
```

Chrome runs with its sandbox. Where it cannot start with one (a container that gives it none) it is started again without, and `wf verify` says so; `WF_CHROME_NO_SANDBOX=1` skips the first try. Its profile is a private directory with a name nobody can guess, and the browser and the profile go when `wf verify` or `wf test` ends — however it ends, a signal included.

`wf build` reports what the compiler wrote; `wf verify` reports what a browser does with it. It builds nothing: it starts a headless Chrome, serves what `wf build` wrote, and opens every route the project has — a `:param` route at each value its `paths:` names, or at a placeholder (`/user/1`) when it names none.

A route fails on any of these:

- an uncaught exception or a `console.error`;
- a file that did not arrive (a request the build has no file for is a `404`, not the page shell);
- an image that failed to load;
- a router that drew nothing into `<main>`;
- a page that rendered no text.

The report prints one line per route:

```text
  16 route(s) in http://127.0.0.1:39785
    ok   /                                    236ms     613 nodes    447.0 kB   11 req
    ok   /app/deployments                      68ms     657 nodes    360.6 kB   10 req

  16 page(s), 0 problem(s)
```

The columns are the first contentful paint, the number of elements in the document once it settled, the bytes the page fetched, and the requests it made. `--budget 400` fails a route that takes longer than 400 ms to paint.

`--returning-visitor` visits every route a second time as a reader who was here before. Their storage holds what the previous build's pages kept: each `persist` value's starting value as the build before this one wrote it, which a build saves in `.wf-cache/` when it changes. A persisted shape that changed without a `version:` then shows as the broken page it would be.

`--fakes tests/fakes.wf` is for a site whose API is elsewhere, or not running. It builds the project again, aside from `build/`, with the fakes that file declares answering every endpoint — the same `fake` declarations a test names ([Testing](https://webfluent.monzeromer.dev/learn/16-testing.md#answer-a-service-with-fakes)) — and loads every page of that build, which sends no request to a service. A fake that does not fit its contract stops the build (`N06`). The aside build is removed when the visits are done.

The report also names **the built-ins no page drew**. A component can be in the registry, the tests and the documentation and still be broken on a page. The classes the pages carried list what actually ran, so the difference is what nothing has exercised. One project is not expected to draw them all: the list is a fact about coverage, not a failure.

`wf verify` needs Chrome or Chromium on the machine, or `WF_CHROME` pointing at one, and says so when there is none.

See also: [Testing](https://webfluent.monzeromer.dev/learn/16-testing.md).
