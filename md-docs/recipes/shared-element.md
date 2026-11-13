---
kind: "recipe"
route: "/recipes/shared-element"
group: "look"
description: "To have the browser carry one element from one page to the next — a cover picture that grows into the article's header — give it the same `shared:`…"
from: ["/docs/guide/motion#carry-an-element-across-a-route-change"]
---
# Carry an element across a route change

To have the browser carry one element from one page to the next — a cover
picture that grows into the article's header — give it the same `shared:`
name on both pages. The browser moves it instead of fading it out and back
in:

```wf
type Post { id: String, slug: String, title: String, cover: String }

const posts: [Post] = [
    Post(id: "1", slug: "slow-roasting", title: "Slow roasting", cover: "/roast.jpg"),
]

page Blog(path: "/", title: "Blog", description: "Every post.") {
    Heading("Blog").h1
    for post in posts by post.id {
        Link(to: "/p/{post.slug}") {
            Image(src: post.cover, alt: post.title, shared: "cover-{post.id}")
            Text(post.title)
        }
    }
}

page Article(path: "/p/:slug", title: "Post", description: "One post.", slug: String) {
    if let post = posts.find(p => p.slug == slug) {
        Heading(post.title).h1
        Image(src: post.cover, alt: post.title, shared: "cover-{post.id}")
    } else {
        Heading("Not found").h1
    }
}
```

Both elements must carry the same name, and only one element on a page may
carry each name — which is why the name here includes the post's `id`.
