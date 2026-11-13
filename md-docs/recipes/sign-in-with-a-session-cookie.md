---
kind: "recipe"
route: "/recipes/sign-in-with-a-session-cookie"
group: "data"
description: "To sign a reader in without holding a token in the page, describe the service with `credentials: .sameOrigin` and keep the user in a store:"
from: ["/docs/guide/recipes#sign-in-with-a-session-cookie"]
---
# Sign in with a session cookie

To sign a reader in without holding a token in the page, describe the
service with `credentials: .sameOrigin` and keep the user in a store:

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

page Account(path: "/account", title: "Your account", description: "Your account.", guard: Session.signedIn, redirect: "/sign-in") {
    use Session
    Heading("Your account").h1
    if let u = Session.user { Text("Signed in as {u.name}") }
    Button("Sign out") { on click { Session.signOut() } }
}
```

The session is an httpOnly cookie the server sets, so the page never holds
a token. `guard:` hides the page from a reader who is signed out, and the
server still checks every request
([Security](../about/security.md#keep-the-session-in-a-cookie)). Call `Session.load()`
from the app's set-up, so a returning reader is signed in.
