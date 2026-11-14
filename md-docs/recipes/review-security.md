---
kind: "recipe"
route: "/recipes/review-security"
group: "ship"
description: "Review what a WebFluent site trusts before a release: read wf audit, then go through the security checklist for the build, the host and the server."
outputs: ["website", "app"]
---
# Review a site's security before you deploy

The compiler refuses most of what would make a page unsafe on every build
([Security](../about/security.md)). What it cannot see — what the project
chose to trust, the host's headers, the server's checks — is a review, and
`wf audit` prints what that review asks for:

```bash
wf audit
```

## How it works

`wf audit` lists:

- every `Unsafe.*`, and whether it is sanitised;
- the project's own scripts, and the names each declares;
- every origin the pages load from;
- everything written to the reader's machine, and where;
- every `env` name, and whether it is public;
- the policy this build ships;
- what the compiler itself depends on.

Add `--json` to get the same report for a tool. `wf audit` only reports; it
never fails a build.

## Before you deploy

- Run `wf audit`, and read all of it.
- Set `"csp": true`, and make sure the build passes its own check.
- Check that `_headers` reaches your host, or set the same headers
  another way ([Set cache and security headers](cache-and-security-headers.md)).
- Serve over HTTPS, with HSTS.
- Keep the session in an httpOnly, Secure, SameSite cookie — not in
  `persist` ([Sign in with a session cookie](sign-in-with-a-session-cookie.md)).
- Read no non-public `env` name in a page (the build refuses one).
- Make sure every `Unsafe.Html` is sanitised, or provably your own
  markup ([Show HTML from outside the project](show-html-from-outside.md)).
- Check authorisation on the server for every request, not with
  `guard:`.
- Have the server check uploads: the type from the file's bytes, not its
  name; a size limit it enforces itself; a stored name it generates, never
  the one it was given; and storage nothing is served from.

After the deploy, [check the headers it sends](check-a-deploy.md).
