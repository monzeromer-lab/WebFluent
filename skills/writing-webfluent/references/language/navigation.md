### Navigation

`navigate()` changes the route from code; `Link` does it from markup.

```wf
navigate("/path")                    // Programmatic; an address on another origin is refused
Link("About", to: "/about")          // Declarative
```

