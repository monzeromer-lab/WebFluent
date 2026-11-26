## Strings

Strings come in three forms: plain `"…"` with escapes and splices, raw `#"…"#`, and indented block `"""…"""`.

A plain `"…"` string reads the escapes `\n \t \r \\ \" \{ \}` and `{…}`
splices. Two other forms exist for text a plain string makes you spell
twice:

```wf
// Raw: no escapes, no splices — the text is what is written. As many `#`
// as it takes for the text not to close it.
const SAMPLE = #"page Home(path: "/") { state n = 0 }"#
const TRICKY = ##"a raw string ends with "# — like that"##

// Block: over as many lines as it likes, escapes and splices on, and the
// indentation the source gave it removed. The closing `"""` says how far
// the text was indented, and that much comes off every line.
const LETTER = """
    Dear {name},
      thank you.
    """                        // "Dear Ada,\n  thank you."
```

A splice may say how to show its value — `{value:.style}` or
`{value:.style(option)}`, which is `format(value, .style, option)` written
where it is read:

```wf
Text("Total: {total:.currency}")          // Total: $1,234.50
Text("{count:.integer} items")            // 1,235 items
Text("Shipped {when:.date(long)}")        // Shipped March 5, 2026
```

A splice opens with a name, a `(` or a `[`, and closes on the same line;
anything else — `"{key: value}"` in prose, `"{"` alone — is text.

