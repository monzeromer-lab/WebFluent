# Call a service from its OpenAPI spec

> Call a service from its OpenAPI spec: `wf api` writes the contract as files, checked in CI, and the program calls it typed.

The page: https://webfluent.monzeromer.dev/recipes/service-from-openapi

When the service publishes an OpenAPI spec (3.0 or 3.1, JSON or YAML), let
it write the contract: every endpoint, parameter, answer and error status,
typed.

**Needs:** the spec as a file in the project — the compiler never fetches,
so download it first:

```bash
curl -o openapi.json https://api.example.com/openapi.json
wf api add openapi.json --name Backend
```

`add` records the service in `webfluent.app.json`, writes
`src/api/backend/service.wf` once — yours, where the base and the
middleware go — and writes the contract into `src/api/backend/generated/`,
one file a tag and `types.wf`. Call it like any `api`:

```wf
// What `wf api sync` writes, for a spec with a `people` tag:
type Person { id: String, name: String }
type PersonPage { items: [Person], nextCursor: String? }

enum GetPersonError {
    notFound(problem: Problem) on 404
    http(error: HttpError)
}

api Backend(base: "/api") {
    // Yours: what a write makes stale and how a read pages, for the
    // generated endpoints.
    pagination {
        people.listPeople pages .cursor(param: after, next: p => p.nextCursor, items: p => p.items)
    }
}

api Backend.people {
    get listPeople(q: String?, after: String?) at "people" -> PersonPage!
    get getPerson(id: String) at "people/{id}" -> Person ! GetPersonError
}

page People(path: "/", title: "People", description: "Everyone on the team.") {
    resource people = Backend.people.listPeople()
    Heading("People").h1
    match people {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(list) { for p in list by p.id { Text(p.name) } }
    }
    if people.hasMore {
        Button("More") { on click { people.loadMore() } }
    }
}
```

When the spec changes, run `wf api sync` and read the diff: a field
renamed, a status added, is a build error where the program relies on the
old one. In CI, `wf api check` fails while the files are behind the spec,
and `wf check` warns `N07`.

```bash
wf api sync
wf api check
```

A generated file is never edited — `sync` rewrites the folder whole, and
refuses to touch a file in it without its marker. Customise beside it: a
`type` of the program's own wins over the spec's, an endpoint the spec
lacks goes in a group of your own, and `invalidations { }` and
`pagination { }` go in `service.wf`. More in
[Data and APIs](https://webfluent.monzeromer.dev/learn/15-data.md#read-the-service-from-an-openapi-file).
