# 30. Environments

<!--
route: guide/environments
group: shipping
blurb: Values fixed per build — an API address, a feature switch, a public key — from the config, a .env file or the shell, and which of them a page may read.
description: env values from the config, a .env file and the shell; their precedence; public and private names; and a build per environment.
-->

A site usually needs a few values that differ between your machine, staging
and production: the API's address, an analytics key, a feature switch. In
WebFluent they are read as `env.NAME` and fixed **at build time** — the
build writes the value into the output, so there is nothing to configure on
the host.

```wf
const API = env.PUBLIC_API ?? "/api"
const FEATURES = { billing: env.PUBLIC_BILLING == "on" }

page Rows(path: "/", title: "Rows", description: "Rows from the API.") {
    resource rows = fetch("{API}/rows")
    Heading("Rows").h1
    if FEATURES.billing { Link("Billing", to: "/billing") }
    match rows {
        loading { Spinner }
        error(e) { Text(e.message).danger }
        ready(r) { Text("{r.length} rows") }
    }
}
```

A name nothing sets reads as nothing, so `??` supplies a default.

## Where values come from

Three places, later ones winning:

1. **The config's `env` map**, committed with the project:

   ```json
   { "env": { "PUBLIC_API": "/api", "PUBLIC_BILLING": "off" } }
   ```

2. **A `.env` file** beside the config — `NAME=value` lines, `#` comments,
   an optional `export ` and quotes. Not committed (`wf init` puts it in
   `.gitignore`):

   ```bash
   PUBLIC_API=http://localhost:8080/api
   PUBLIC_BILLING=on
   ```

3. **The shell** the build runs in — for CI and one-off builds:

   ```bash
   PUBLIC_API=https://api.example.com wf build
   ```

   The shell supplies a name the config or `.env` already declares, or any
   name that is public (below). It does not bring in every variable the build
   happens to run with.

**4.1.** Earlier versions read only the config's `env` map.

## Public and private names

`env.NAME` is replaced by its value in the JavaScript and HTML every reader
downloads. That is right for an API address and wrong for an API key — so a
page, a component, a store or an `api` block may only read a name that says
it is public:

- a name beginning **`PUBLIC_`**, or
- a name listed in the config's **`public_env`**:

```json
{
  "env": { "PUBLIC_API": "/api/v1", "ANALYTICS_ID": "UA-1", "STRIPE_SECRET": "sk_…" },
  "public_env": ["ANALYTICS_ID"]
}
```

`env.PUBLIC_API` and `env.ANALYTICS_ID` may be read from a page;
`env.STRIPE_SECRET` read from one is a **compile error**, naming the file and
the line. A private value is still there for [server rendering](34-server-rendering.md),
which runs where a secret stays secret — and it never reaches the bundle:
the build writes only public values into it.

A secret a third party needs belongs in a request your server makes, not in a
page ([Security](23-security.md#sessions-and-tokens)).

## A build per environment

The same source, different values:

```bash
wf build                                              # .env on your machine
PUBLIC_API=https://staging.example.com/api wf build   # staging
PUBLIC_API=https://api.example.com wf build           # production
```

In CI, set the variables in the job:

```yaml
- run: ~/.webfluent/bin/wf build
  env:
    PUBLIC_API: https://api.example.com
    PUBLIC_ANALYTICS_ID: ${{ vars.ANALYTICS_ID }}
```

## Other settings that change per deploy

- **`build.base_path`** — the sub-path the site is served under. Every link,
  asset and route is prefixed ([Deploying](29-deploying.md)).
- **`meta.site_url`** — the absolute address, for canonical links, the
  sitemap and sharing tags.
- **`theme.tokens`** — design tokens a pipeline supplies, on top of the
  theme: `{ "theme": { "tokens": { "color-primary": "#8B5CF6" } } }`.

These live in `webfluent.app.json`. There is no separate config per
environment; a pipeline that needs one writes the file before building.

## What a review asks

```bash
wf audit
```

lists every `env` name the project has, where it is read, and whether it is
public — beside every other thing the site trusts ([Security](23-security.md#wf-audit)).

## Next

[Performance](31-performance.md).
