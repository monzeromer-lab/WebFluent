# ConsentBanner

> The reader's say over the scripts that track them: shown until they choose, Decline as easy as Accept, and the choice kept. Accepting loads the meta.scripts entries marked "load": "consent", then and on every later visit; nothing loads before.

The page: https://webfluent.monzeromer.dev/reference/consentbanner

The reader's say over the scripts that track them: shown until they choose, Decline as easy as Accept, and the choice kept. Accepting loads the meta.scripts entries marked "load": "consent", then and on every later visit; nothing loads before.

```wf
ConsentBanner(_ text: String, accept: String, decline: String, label: String) { … }
```

## Example

```wf
ConsentBanner("We would like to count visits to improve the site. Nothing is shared.", accept: "Allow", decline: "No thanks") {
    Link("How we count", to: "/learn")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String (positional)` | What is asked, drawn above the block and the two buttons |
| `accept:` | `String` | The yes button's words; pressing it loads the `"load": "consent"` scripts and remembers the answer |
| `decline:` | `String` | The no button's words, drawn first and as Accept is; the answer is remembered and nothing loads |
| `label:` | `String` | The banner's accessible name (`aria-label`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
