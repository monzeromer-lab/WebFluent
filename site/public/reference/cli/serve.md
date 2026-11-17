# wf serve

> Start the development server

The page: https://webfluent.monzeromer.dev/reference/cli/serve

Start the development server

```bash
wf serve [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--host HOST` | The address to listen on: this machine only by default; `0.0.0.0` lets a phone on the same network reach it |
| `--allow-output-outside` | Let `build.output` name a directory outside the project |

**Writes:** the site, in the output folder, on every save

Runs the development server: it builds once, serves the output, and
rebuilds and reloads on every save.

```bash
wf serve
wf serve --host 0.0.0.0     # a phone on this network, at the address it prints
```

The server listens on `dev.port` (3000), on `127.0.0.1`: nothing else on the
network can reach it unless `--host` says so. It watches `src/`, `public/`
and the config, rebuilds on a change, and reloads every open tab. Each
rebuild is a `wf build --cache`, so only the pages the change reached are
rendered again.

It serves the build and nothing beside it: an address that climbs out
(`/../`, written plainly or as `%2e%2e`), holds a backslash or a NUL, or
reaches a link out of the build is a `404`. It answers only a request whose
`Host` is a name it is reached by — `localhost`, an address written as
numbers, the `--host` it was given, and with `--host 0.0.0.0` a `.local`
name — so a page on another site cannot point a name of its own at this
machine and read the build (DNS rebinding); any other gets a `421`. It
answers several requests at once, streams files rather than reading them
whole, and compresses a file once until it changes.

When a build fails, the last good page stays up and the error is drawn over
it — file, line, message and hint — until a save fixes it. The overlay is
served at `/__wf/dev.js`, and the build's state, as JSON, at `/__wf/status`.

Every page the server sends also carries a **stores** button. It shows what
each store holds, a log of every action with the state on each side of it,
and lets you put a store back to before an action ran
([Stores](../learn/11-stores.md#looking-at-one-while-it-runs)). The service worker of
an offline site is removed under `wf serve`, so an edit is never hidden
behind its cache.

A request that is not a navigation — an `http` or `api` call, a script, an
image — for a file the build did not write gets a `404`, as a host would
send, and a line in the terminal naming the file. Only a navigation gets the single-page
shell. (Answering a missing API with the shell's HTML makes the page fail to
parse it, far from the cause.)

A build under `wf serve` is a *development build*, and it adds these checks
to the running page:

- The body of each status an endpoint's error enum names is held to the
  type its case carries. (A response's declared type — `resource users:
  [User] = …`, an endpoint's `-> User!` — is held in every build, served or
  deployed: one that does not match fails as `.decode(at, message)`, naming
  the place, `$[1].name is missing`.)
- A route no page answers (and no `path: "*"` page catches) shows a `404`
  box saying so. A deployed page shows nothing.
- An item of a `for` that throws shows where it is and what it threw, and
  the rest of the list draws. On a deployed page the item leaves a gap and
  the console says why.
- An effect that changes what it reads every time it runs is stopped after
  a hundred runs, with a warning, instead of overflowing the stack. A
  deployed page stops it too, silently.

See also: [`dev` in Configuration](configuration.md#dev).
