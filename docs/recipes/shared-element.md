# Carry an element across a route change

> Carry one element from one page to the next, such as a cover picture that grows into the article's header, by giving it one shared: name on both.

The page: https://webfluent.monzeromer.dev/recipes/shared-element

Have the browser carry one element from one page to the next — a cover
picture in a list of posts that grows into the article's header — instead
of fading it out and back in. Give it the same `shared:` name on both
pages:

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

## How it works

- The `Blog` page draws each cover with `shared: "cover-{post.id}"`, and
  the `Article` page draws the same post's cover under the same name.
- When the route changes from one to the other, the browser moves the
  element from its old place and size to its new one.
- Both elements must carry the same name, and only one element on a page
  may carry each name — which is why the name here includes the post's
  `id`. If the two pages give it different names, or two elements on one
  page carry the same name, the element crossfades instead of moving.
