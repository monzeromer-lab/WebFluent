# LocaleSwitch

> One button for each of i18n.locales, each written in its own language (English, العربية), the current one pressed; a press is setLocale.

The page: https://webfluent.monzeromer.dev/reference/localeswitch

One button for each of i18n.locales, each written in its own language (English, العربية), the current one pressed; a press is setLocale.

```wf
LocaleSwitch(label: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The group's name for a screen reader; unset, `Language` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
LocaleSwitch(label: "Language")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
