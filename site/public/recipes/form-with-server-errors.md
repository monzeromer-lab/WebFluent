# Show the server's field errors on a form

> Show the server's own field errors on a form: an error enum names the 422, the form takes its errors, and each lands under its field.

The page: https://webfluent.monzeromer.dev/recipes/form-with-server-errors

A form checks what it can before it sends (see [Show validation messages on a form](https://webfluent.monzeromer.dev/recipes/validation-messages.md)), but only the server knows that an address is taken. When it answers `422` with the fields that failed, put each message under its field.

**Needs:** a server that answers `POST /api/people` with the person it made, or `422` and `{ "errors": [{ "field": "email", "message": "…" }] }`. It claims `/`: paste it into a new project, or replace the starter's `src/pages/Home.wf` with it.

```wf
type Person { id: String, name: String, email: String }
type NewPerson { name: String, email: String }
type FieldError { field: String, message: String }
type ValidationProblem = Problem { errors: [FieldError] }

/// Why adding someone can be refused: each status the server answers with,
/// and `http` for the rest.
enum CreatePersonError {
    unprocessableContent(problem: ValidationProblem) on 422
    conflict(problem: Problem) on 409
    http(error: HttpError)
}

api Backend(base: "/api") {
    post createPerson(body: NewPerson) at "people" -> Person ! CreatePersonError
}

page Join(path: "/", title: "Join", description: "Add yourself to the team.") {
    state name = ""
    state email = ""
    state said = ""
    validate name { required }
    validate email { required  email }

    Heading("Join the team").h1
    Form(bind: form) {
        on submit {
            match await Backend.createPerson(body: NewPerson(name: name, email: email)) {
                .ok(p) {
                    said = "Welcome, {p.name}."
                    form.reset()
                }
                .err(.unprocessableContent(problem)) {
                    // Each field's message, by the field's name.
                    form.apply(problem.errors.reduce((byField: [String: String], e: FieldError) => byField.with(e.field, e.message), {}))
                }
                .err(.conflict(problem)) { said = problem.detail ?? "You are already on the team." }
                .err(.http(e)) { said = e.message }
            }
        }
        Input(bind: name, label: "Name")
        Input(bind: email, label: "Email").email
        Button("Join", type: .submit, disabled: !form.valid).primary
    }
    Text(said)
}
```

## How it works

`form.apply(errors)` takes each field's message by the field's name and shows it under the control whose `name:` (or whose bound state) is that field, with the same accessible plumbing a `validate` rule uses; the focus goes to the first. The `match` covers every case of `Person ! CreatePersonError`, so a status the server adds later is a build error here, not a silent nothing.
