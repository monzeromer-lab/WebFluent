# 30. Environments

<!--
route: guide/environments
group: shipping
blurb: Set values that change per build (an API address, a feature switch, a public key) from the config, a .env file or the shell, and choose which a page may read.
description: Set env values from the config, a .env file or the shell; learn which wins; keep private names out of the bundle; and build once per environment.
-->

This chapter shows you how to give a site values that differ between your
machine, staging and production: the API's address, an analytics key, a
feature switch. A page reads each one as `env.NAME`. The build fixes the
value **at build time**: it writes the value into the output, so there is
nothing to configure on the host.

In this chapter:

- Set a value in the config, a `.env` file or the shell.
- Choose which names a page may read, and keep the rest out of the bundle.
- Build the same source once per environment.

## Read a value in a page

To read a value, write `env.NAME`. This page reads the API's address and a
feature switch, and falls back to defaults when nothing sets them:

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

A name that nothing sets reads as nothing (`null`), so `??` supplies the
default. Keep the values in `const`s at the top of the file, as above, so
every page reads them from one place.

## Set a value

You can set a value in three places. When two set the same name, the later
one in this list wins:

1. **The config's `env` map**, committed with the project:

   ```json
   { "env": { "PUBLIC_API": "/api", "PUBLIC_BILLING": "off" } }
   ```

2. **A `.env` file** beside `webfluent.app.json`, which you do not commit.
   `wf init` adds `.env` to `.gitignore`. Each line is `NAME=value`. A line
   that starts with `#` is a comment, a line may start with `export `, and a
   value may be in single or double quotes:

   ```bash
   PUBLIC_API=http://localhost:8080/api
   PUBLIC_BILLING=on
   ```

3. **The shell** that runs the build, for CI and one-off builds:

   ```bash
   PUBLIC_API=https://api.example.com wf build
   ```

   The shell supplies a name that the config or `.env` already declares, or
   any name that is public (see the next section). It does not bring in
   every variable the build happens to run with, such as `HOME`.

> [!NOTE]
> Versions before 4.1 read only the config's `env` map.

## Keep a private value out of the bundle

The build replaces `env.NAME` with its value in the JavaScript and HTML
that every reader downloads. That is right for an API address and wrong for
an API key. So a page, a component, a store or an `api` block may read only
a name that says it is public:

- a name that begins with **`PUBLIC_`**, or
- a name listed in the config's **`public_env`**.

In this config, `env.PUBLIC_API` and `env.ANALYTICS_ID` are public, and
`env.STRIPE_SECRET` is private:

```json
{
  "env": { "PUBLIC_API": "/api/v1", "ANALYTICS_ID": "UA-1", "STRIPE_SECRET": "sk_…" },
  "public_env": ["ANALYTICS_ID"]
}
```

A page may read `env.PUBLIC_API` and `env.ANALYTICS_ID`. Reading
`env.STRIPE_SECRET` from a page is a compile error
([`E108`](40-diagnostics.md#e108-an-env-name-that-is-not-public)) that names the file and the line.

A private value is still available to [server rendering](35-server-rendering.md),
which runs on a server, where a secret stays secret. It never reaches the
bundle: the build writes only public values into it.

> [!CAUTION]
> Adding a name to `public_env` publishes its value to every reader of the
> site. A secret that a third party needs belongs in a request your server
> makes, not in a page ([Security](23-security.md#keep-secrets-out-of-the-bundle)).

## Name your own values something other than `env`

`env` is the language's name for these settings, and nothing else may use
it. A `state`, a prop, a route parameter, a loop item or a top-level `const`
called `env` is an error
([`E118`](40-diagnostics.md#e118-a-name-the-language-already-has)). For a
value that says which stage a page is about, use a name such as `target` or
`stage`.

## Build once per environment

To deploy the same source to several environments, run one build per
environment, each with its own values:

```bash
wf build                                              # .env on your machine
PUBLIC_API=https://staging.example.com/api wf build   # staging
PUBLIC_API=https://api.example.com wf build           # production
```

In CI, set the variables on the job's build step:

```yaml
- run: ~/.webfluent/bin/wf build
  env:
    PUBLIC_API: https://api.example.com
    PUBLIC_ANALYTICS_ID: ${{ vars.ANALYTICS_ID }}
```

Each build writes its own `build/` folder. Deploy each one to its own host
([Deploying](29-deploying.md)).

## Change other settings per deploy

A few settings outside `env` also change between deploys:

| Setting | What it is |
|---|---|
| `build.base_path` | The sub-path the site is served under. The build prefixes every link, asset and route with it ([Deploying](29-deploying.md)) |
| `meta.site_url` | The site's absolute address, for canonical links, the sitemap and sharing tags |
| `theme.tokens` | Design tokens a pipeline supplies, on top of the theme: `{ "theme": { "tokens": { "color-primary": "#8B5CF6" } } }` |

These settings live in `webfluent.app.json`. There is no separate config
file per environment. A pipeline that needs different settings writes the
file before it builds; `wf config set` edits it from a script:

```bash
wf config set build.base_path /staging
```

## Audit what the project reads

To list every `env` name the project has, run `wf audit`:

```bash
wf audit
```

It prints each name, where it is read, and whether it is public, beside
everything else the site trusts ([Security](23-security.md#audit-the-project)).
Run it before a release, and when you review a change that adds a name.

## Next

[Performance](31-performance.md).
