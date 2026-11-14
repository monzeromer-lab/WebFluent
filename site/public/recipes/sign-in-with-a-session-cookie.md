# Sign in with a session cookie

> Sign a reader in with an httpOnly session cookie: an api with credentials: .sameOrigin, a Session store, and a guarded account page.

The page: https://webfluent.monzeromer.dev/recipes/sign-in-with-a-session-cookie

A token kept in the page can be read by any script that reaches it. To sign
a reader in without holding a token at all, let the server set a session
cookie, describe the service with `credentials: .sameOrigin`, and keep the
signed-in user in a store:

**Needs:** a server that answers the `/api` endpoints the `api` block
names and sets the session cookie. The code declares `type User` and an
`app`, as the starter does: paste it into a new project, replacing
`src/App.wf` and the starter's `src/stores/auth.wf`.

```wf
type User { id: String, name: String }

api Auth(base: "/api") {
    credentials: .sameOrigin
    get me() -> User
    post signIn(body: Map) -> User
        errors { 401 -> Map }
    post signOut()
}

store Session {
    state user: User? = null
    state checked = false
    derived signedIn = user != null
    action load() {
        try { user = await Auth.me() } catch e { user = null }
        checked = true
    }
    action signIn(email: String, password: String) {
        user = await Auth.signIn(body: { email: email, password: password })
        navigate("/account")
    }
    action signOut() {
        await Auth.signOut()
        user = null
        Auth.me.invalidate()
        navigate("/")
    }
}

page SignIn(path: "/sign-in", title: "Sign in", description: "Sign in to your account.", noindex: true) {
    state email = ""
    state password = ""
    state problem = ""
    validate email { required  email }
    validate password { required }
    action send() {
        problem = ""
        try { await Session.signIn(email, password) } catch e { problem = "That email and password do not match." }
    }
    Heading("Sign in").h1
    if problem != "" { Alert(problem).danger }
    Form(bind: form) {
        on submit { send() }
        Input(bind: email, label: "Email").email
        Input(bind: password, label: "Password").password
        Button("Sign in", type: .submit, disabled: send.pending).primary
    }
}

page Account(path: "/account", title: "Your account", description: "Your account.", guard: !Session.checked || Session.signedIn, redirect: "/sign-in") {
    use Session
    // The guard is read when the route opens, which on a reload is before
    // `load` has answered; this sends the reader on once it has.
    effect { if Session.checked && !Session.signedIn { navigate("/sign-in") } }
    Heading("Your account").h1
    if !Session.checked { Spinner }
    if let u = Session.user {
        Text("Signed in as {u.name}")
        Button("Sign out") { on click { Session.signOut() } }
    }
}
```

The app asks the server who is signed in once, when the site opens, in
`src/App.wf`:

```wf
app {
    use Session
    Session.load()
    Router
}
```

## How it works

- `Auth` [describes the service](https://webfluent.monzeromer.dev/learn/15-data.md#describe-a-service-with-api).
  `credentials: .sameOrigin` sends the session cookie with every call, so
  the page holds no token. `errors { 401 -> Map }` types what a refused
  sign-in answers.
- The `Session` [store](https://webfluent.monzeromer.dev/learn/11-stores.md#a-first-store) holds the
  user, or `null`, and `signedIn` is derived from it. `load` asks the
  server who is signed in, `signIn` and `signOut` call the service and
  move the reader on with `navigate`.
- The server sets the cookie when a sign-in succeeds, with a header such as
  `Set-Cookie: session=…; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=1209600`.
  `HttpOnly` keeps it from every script, `SameSite=Lax`
  from another site's form, and `Secure` off plaintext connections.
- `Auth.me.invalidate()` on sign-out drops what the endpoint cached, so
  the next reader of the tab is not shown the last one's. Do the same for
  every endpoint whose answer is one reader's: an answer cached under one
  `Authorization` header is never served for another, but a cookie cannot
  be read, so the cache cannot tell one cookie's reader from the next.
- The `SignIn` page validates its two fields and calls `Session.signIn`
  in a `try`, so a wrong password shows an `Alert` instead of throwing.
  `send.pending` disables the button while the request runs, and
  `noindex: true` keeps the page out of search results.
- `Session.load()` at the top of `app` runs once, when the site opens: a
  reader who comes back with a session cookie is signed in again, and
  `checked` turns true when the server has answered either way.
- The `Account` page has a [guard](https://webfluent.monzeromer.dev/learn/05-pages-and-routing.md#guards).
  A guard is read once, when the route opens, so it lets the page through
  while `checked` is still false, and the `effect` sends a signed-out
  reader to `/sign-in` as soon as the answer arrives. Until then the page
  shows a `Spinner`.

The session is an httpOnly cookie the server sets, so the page never holds
a token. `guard:` hides the page from a reader who is signed out, and the
server still checks every request
([Security](https://webfluent.monzeromer.dev/about/security.md#where-a-session-belongs)).
