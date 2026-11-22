# WebFluent for React developers

> React ideas in WebFluent: state, derived values, effects, props, children, events, context, routing, lists and fetching, side by side.

The page: https://webfluent.monzeromer.dev/coming-from/react

Most of what you know from React carries over: components with props, state, a router and shared stores. The difference to hold on to is that WebFluent is a compiler with typed markup, not a library: a component's body runs once, and only the text, attribute or style that reads a changed value updates. There are no hooks, no dependency arrays and no setters.

## The map

| Concept | React | WebFluent |
|---|---|---|
| A value that changes | `const [n, setN] = useState(0)` | `state n = 0` |
| Changing it | `setN(n + 1)` | `n = n + 1` |
| A value worked out from others | `const total = useMemo(() => sum(items), [items])` | `derived total = items.sum()` |
| A side effect | `useEffect(() => { start(); return () => stop() }, [id])` | `effect { start(id)  cleanup { stop() } }` |
| A handle on an element | `const box = useRef(null) … box.current.focus()` | `Input(bind: q, ref: box) … box.focus()` |
| A component | `function Card({ title }) { … }` | `component Card(_ title: String) { … }` |
| What the caller puts inside | `{props.children}` | `children` |
| A callback prop | `onChange={(v) => setLast(v)}` | `event change(value: Number), handled as on change(value) { last = value }` |
| A controlled input | `value={q} onChange={(e) => setQ(e.target.value)}` | `Input(bind: q, label: "Search")` |
| State shared across pages | `createContext, Redux, Zustand` | `store Cart { … }, read as Cart.items after use Cart` |
| A route | `<Route path="/users/:id" element={<User />} />` | `page User(path: "/users/:id", id: String) { … }` |
| Showing something conditionally | `{open && <Panel />}` | `if open { Panel }` |
| A list | `{items.map((i) => <li key={i.id}>{i.name}</li>)}` | `for i in items by i.id { Text(i.name) }` |
| Fetching | `TanStack Query's useQuery({ queryKey: ["users"], queryFn }), or useEffect + fetch` | `resource users: [User] = http.get("/api/users"), read with match users { … }` |
| Styling | `className, CSS modules, Tailwind` | `flags (Button("Save").primary), style { } on the element, design tokens` |
| A test | `Testing Library: render, fireEvent.click, getByText` | `test "…" { click "Save"  expect "Saved" }` |

## Things that work differently

**It is a compiler, not a library.** There is no virtual DOM and no re-render. Each text, attribute or style that reads a state follows it on its own, and a component's body runs once, when it is placed. That is why there are no hooks and no rules about calling them in order.

**The markup is typed.** The compiler knows the props, flags, events and slots of every built-in and of every component you declare. `Button("Save").primray` does not build, and neither does a handler for an event the component never fires (`E103`). An unknown prop on your own component is `C02`, not a silent pass-through.

**Typed without a second language.** There is no TypeScript beside the markup and no `tsc` step: a `.wf` file is typed by the compiler that builds it. The checker infers what it can and asks you to write the rest: a state whose first value says nothing (`state items: [Todo] = []`), an action that returns (`-> Number`), a parameter. There is no `any`: JSON whose shape is open is `Json`, matched or decoded (`User.decode(raw)`) before it is read, and no setting turns the checks off.

**Pages own their routes.** There is no route table. Each `page` names its `path:`, and the compiler builds the router from them, ordered by specificity, so the order you declare pages in cannot hide a route.

**No imports, no npm.** Every file under `src/` is part of one program, so a component declared anywhere is usable everywhere. A JavaScript library is a URL in `meta.scripts`, and your own JavaScript is a plain `.js` file under `src/` whose functions `.wf` code calls by name ([Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)).

**The build is the site.** Pre-rendering, code splitting, image resizing, a sitemap, link previews and a Content-Security-Policy come from `wf build`, with no plugins to choose. There is no server rendering on each request: a site is static files and a browser runtime, talking to your API.

**Text is text.** The only way to put markup on a page is `Unsafe.Html(…)`, which the build flags every time, and inline handlers and `javascript:` URLs do not compile ([Security](https://webfluent.monzeromer.dev/about/security.md)).

**Using it inside a React app.** Publish components as custom elements and place their tags in JSX ([Publish components as custom elements](https://webfluent.monzeromer.dev/recipes/publish-custom-elements.md)).

## A component, ported

The same counter, in React and in WebFluent. It starts at a number the caller passes, counts up on a click, shows a badge when the count is even, and tells the caller about each change.

```jsx
function Counter({ start = 0, onChange }) {
  const [n, setN] = useState(start);
  const even = useMemo(() => n % 2 === 0, [n]);
  return (
    <div className="row">
      <button onClick={() => { setN(n + 1); onChange?.(n + 1); }}>+</button>
      <span>{n}</span>
      {even && <span className="badge">even</span>}
    </div>
  );
}
```

```wf
page Scores(path: "/", title: "Scores", description: "A counter that reports each change.") {
    state last = 0
    Heading("Scores").h1
    Counter(start: 2) { on change(value) { last = value } }
    Text("Last change: {last}")
}

component Counter(start: Number = 0) {
    event change(value: Number)
    state n = start
    derived even = n % 2 == 0
    Row(gap: .sm, align: .center) {
        Button("+") { on click { n = n + 1  emit change(n) } }
        Text("{n}")
        if even { Badge("even").info }
    }
}
```

- **No hooks and no dependency arrays.** `derived` knows what it reads.
- **No setter.** You assign: `n = n + 1`.
- **No `className`.** `Badge` comes designed, and `.info` is a flag the compiler checks.
- **The event is declared.** A caller that handles `on chnage` gets `E103` at build time, instead of a handler that never runs.

## Data, as TanStack Query does it

TanStack Query is a cache of requests a component asks for. In WebFluent the same jobs are split by where they belong: what the service has is a contract, how it is reached is the service, and what a page reads is a `resource` — one value with three cases.

| TanStack Query | WebFluent |
|---|---|
| `useQuery({ queryKey: ["users", page], queryFn })` | `resource users = Backend.users.list(page: page)`: the arguments are followed, so a change of `page` asks again |
| `isPending`, `isError`, `data`, `error` | one value: `match users { .loading … .failed(e) … .loaded(list) … }`; a `match` that leaves out `.failed` does not build |
| `isFetching` with data on screen | `.loaded(list, refreshing, stale)`: the list stays, `refreshing` is true |
| `staleTime`, `gcTime` | what the server's `Cache-Control` says (RFC 9111), or `cache: .swr(60.seconds)` on the endpoint |
| `refetchOnWindowFocus`, `refetchInterval` | `refresh: .onFocus`, `refresh: .every(30.seconds)` |
| `useMutation({ mutationFn })` | an action that calls the endpoint and `match`es its result |
| `queryClient.invalidateQueries(["users"])` after a mutation | `invalidates [list]` on the write, in the contract: every read of it asks again, with no call written |
| `onMutate` with a snapshot and `onError` rolling back | `let undo = Backend.users.list.update(list => …)`, and `undo()` when the write fails |
| `useInfiniteQuery` with `getNextPageParam` | `pages .cursor(param: after, next: p => p.next)` on the endpoint; `users.loadMore()`, `users.hasMore` |
| `retry: 3` | `retry: .backoff(times: 3)` on the service, for what is safe to send twice; a `POST` only when its endpoint is `idempotent` |
| `error` typed `unknown` or `Error` | `-> User ! CreateUserError`: an error enum with a case for each status the server answers with, matched case by case |
| `openapi-typescript`, `orval` | `wf api add openapi.json`: the contract as files, checked in CI with `wf api check` |
| Mock Service Worker in tests | a `fake` of the contract, held to it by the compiler, named in the test's `fakes:` |

The same list, with a create that refreshes it and a field error shown:

```jsx
function People() {
  const qc = useQueryClient();
  const { data, isPending, error } = useQuery({ queryKey: ["people"], queryFn: listPeople });
  const add = useMutation({
    mutationFn: createPerson,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["people"] }),
  });
  if (isPending) return <Spinner />;
  if (error) return <Alert>{error.message}</Alert>;
  return (
    <>
      {data.map((p) => <p key={p.id}>{p.name}</p>)}
      <button onClick={() => add.mutate({ name: "Grace" })}>Add</button>
      {add.error?.status === 422 && <p>{add.error.body.errors[0].message}</p>}
    </>
  );
}
```

```wf
type Person { id: String, name: String }
type NewPerson { name: String }
type FieldError { field: String, message: String }
type ValidationProblem = Problem { errors: [FieldError] }

enum CreatePersonError {
    unprocessableContent(problem: ValidationProblem) on 422
    http(error: HttpError)
}

api Backend(base: "/api") { }

api Backend.people {
    get list() at "people" -> [Person]!
    post create(body: NewPerson) at "people" -> Person ! CreatePersonError
        invalidates [list]
}

page People(path: "/", title: "People", description: "The team.") {
    state said = ""
    resource people = Backend.people.list()
    Heading("People").h1
    match people {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(list) { for p in list by p.id { Text(p.name) } }
    }
    Button("Add") {
        on click {
            match await Backend.people.create(body: NewPerson(name: "Grace")) {
                .ok(_) { said = "" }
                .err(.unprocessableContent(p)) { said = p.errors.first()?.message ?? "Check the name" }
                .err(.http(e)) { said = e.message }
            }
        }
    }
    Text(said)
}
```

- **No query keys.** A read is known by its endpoint and its arguments; two readers of one address make one request.
- **Invalidation is part of the contract**, said once beside the write, and checked: a read the group does not have is `N05`.
- **The error is typed.** Every status the server declares is a case, and a `match` that misses one does not build.
