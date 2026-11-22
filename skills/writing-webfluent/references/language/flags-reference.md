## Flags Reference

A flag is written tight after the element, its arguments or another flag:
`Button("Save").primary.lg`. It sets a `Bool` prop (`.required`,
`.outlined`, `.pill`) or picks the case of an enum prop (`.lg` is `size:
.lg`, `.primary` is `tone: .primary`); a case that more than one prop has
is named — `tone: .info`. Each component's flags are listed in its entry
above; the language server offers them after `.`, and a flag the
component does not take is an error. A `Bool` prop may be given a
condition instead — `Button("Light", outlined: theme != "light")` — and
the flag's look follows it.

**Sizes**: `.sm`, `.md`, `.lg` (`Spacer` and `gap:` also take `.xs`, `.xl`)
**Tones**: `.primary`, `.secondary`, `.success`, `.danger`, `.warning`, `.info`
**Shape**: `.rounded`, `.pill`
**Surface**: `.flat`, `.elevated`, `.outlined`
**Width**: `.full`, `.fluid`
**Text**: `.bold`, `.italic`, `.underline`, `.uppercase`, `.lowercase`, `.left`, `.center`, `.right`, `.muted`, `.heading`, `.subtitle`
**Heading level**: `.h1` … `.h6`
**Input type**: `.text`, `.email`, `.password`, `.number`, `.search`, `.tel`, `.url`, `.date`, `.time`, `.datetime`, `.color`
**Animation** (on every element): `.fadeIn`, `.fadeOut`, `.slideUp`, `.slideDown`, `.slideLeft`, `.slideRight`, `.scaleIn`, `.scaleOut`, `.bounce`, `.shake`, `.pulse`, `.spin`, `.expand`, `.collapse`
**Speed** (on every element): `.fast` (150ms), `.normal` (the default), `.slow` (500ms)

