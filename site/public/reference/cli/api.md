# wf api

> A service's contract as files, from its OpenAPI spec: add one, sync it, check it is current, turn `api … from` into files, list them

The page: https://webfluent.monzeromer.dev/reference/cli/api

A service's contract as files, from its OpenAPI spec: add one, sync it, check it is current, turn `api … from` into files, list them

```bash
wf api <add|sync|check|eject|list>
```

## wf api add

Keep a service read from SPEC: record it in `apis`, write its `service.wf` once, and its `generated/` files

```bash
wf api add SPEC [options]
```

| Option | Meaning |
|---|---|
| `SPEC` | The OpenAPI spec, JSON or YAML, a file in the project |
| `--name NAME` | The service's name, as the program calls it (default `Backend`) |
| `--out OUT` | The folder its files go in (default: `src/api/<name>`) |
| `--dir, -d DIR` | The project's folder (default `.`) |

## wf api sync

Rewrite each service's `generated/` from its spec

```bash
wf api sync [NAME] [options]
```

| Option | Meaning |
|---|---|
| `[NAME]` | One service (default: every one `apis` keeps) |
| `--skip SKIP` | Leave an operation out, by its name or `METHOD /path`; kept in the config, so the next sync and `check` leave it out too |
| `--dir, -d DIR` | The project's folder (default `.`) |

## wf api check

Fail when `sync` would change anything: for CI

```bash
wf api check [NAME] [options]
```

| Option | Meaning |
|---|---|
| `[NAME]` | One service (default: every one `apis` keeps) |
| `--dir, -d DIR` | The project's folder (default `.`) |

## wf api eject

Turn `api NAME from "openapi.json"` into a kept service's files

```bash
wf api eject NAME [options]
```

| Option | Meaning |
|---|---|
| `NAME` | The service |
| `--out OUT` | The folder its files go in (default: `src/api/<name>`) |
| `--dir, -d DIR` | The project's folder (default `.`) |

## wf api list

Every service read from a spec: its spec, its folder, its groups and their endpoints

```bash
wf api list [options]
```

| Option | Meaning |
|---|---|
| `--json` | Print JSON, for tools |
| `--dir, -d DIR` | The project's folder (default `.`) |

The [full entry for wf api](https://webfluent.monzeromer.dev/reference/cli.md#wf-api) has its examples and the details this page leaves out.
