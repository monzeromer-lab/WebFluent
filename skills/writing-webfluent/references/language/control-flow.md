### Control Flow

```wf
// Conditional
if isLoggedIn {
    Text("Welcome!")
} else {
    Button("Log In") { on click { navigate("/login") } }
}

// A value that may be null, bound when it is not. The binding follows the
// value: another user is drawn in place, and the branch is drawn again only
// when the value goes to null and back
if let user = session.user {
    Text(user.name)
}

// Loop, keyed: an item keeps its nodes for as long as its key is in the
// list — across inserts, removals, moves, and a new value under the same
// key, which its body reads live. The key may read the index too
// (`by "{item.id}-{i}"`); the index alone moves with the position, so key
// by what identifies the item
for item in items by item.id {
    Card { Text(item.name) }
}

// Loop with index
for item, index in items {
    Text("{index}. {item.name}")
}

// Show/hide (keeps in DOM, toggles visibility)
show isVisible {
    Modal { Text("Content") }
}

// One arm by a value: a resource's states, or an enum's cases. An arm is
// drawn again when the arm changes; its binding follows the value in place
match tone {
    .calm { Text("Calm") }
    else { Text("Loud") }
}
```

