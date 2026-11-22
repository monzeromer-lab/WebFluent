# wf start

> Serve a built project as a host would: `$PORT`, the build's headers and compressed files, a single-page build's shell for every route, and `--proxy` for an API elsewhere

The page: https://webfluent.monzeromer.dev/reference/cli/start

Serve a built project as a host would: `$PORT`, the build's headers and compressed files, a single-page build's shell for every route, and `--proxy` for an API elsewhere

```bash
wf start [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--port PORT` | The port (default: `$PORT`, else 3000) |
| `--host HOST` | The address to listen on (default: every interface) |
| `--proxy PREFIX=URL` | Forward requests under a path to another server, the path replaced by the URL: `--proxy /api=https://api.example.com` (repeatable; http or https) |
| `--allow-output-outside` | Let `build.output` name a directory outside the project |

Serves what `wf build` wrote, as a host would. It builds nothing: run `wf build` first.

```bash
wf build && wf start
PORT=8080 wf start --proxy /api=http://localhost:9000/api
```

What it sends:

- **The files of the build,** each with the `.gz` beside it when the client takes gzip, and the headers the build's `_headers` file names for its path (the content security policy, the frame rule). A page is `no-cache`; a file whose name carries its hash is cached for a year.
- **A navigation the build wrote no file for** gets a static build's `404.html` with a `404`, or a single-page build's shell, which draws the route itself. A script, a picture or an API call for a file the build did not write gets a `404`, never the shell.
- **A request under a `--proxy` prefix** goes to the server behind it, its method, headers and body as sent, and its answer comes back as that server gave it. A page calls its own origin (`api Backend(base: "/api")`), so the policy needs nothing added and there is no CORS, and the API's address is chosen where the site starts, not when it was built:

  ```json
  { "scripts": { "build": "wf build", "start": "wf start --proxy /api=$API_BASE_URL/api" } }
  ```

  It speaks HTTP/1.1 to the server behind it, over TLS for an `https://` URL, and trusts what this machine trusts: its certificate store, and the certificates `SSL_CERT_FILE` or `NODE_EXTRA_CA_CERTS` names — a company's own authority, or a development one such as Caddy's or mkcert's. A certificate nothing vouches for is refused, with a `502` that says so.

It is a small server for a container, a preview or an app that serves itself. Behind a CDN or a static host, upload `build/` instead ([Deploy to GitHub Pages](https://webfluent.monzeromer.dev/recipes/deploy-to-github-pages.md)).
