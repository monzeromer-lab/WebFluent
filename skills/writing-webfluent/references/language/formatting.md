### Formatting

```wf
Text(format(price, .currency))                 // $1,234.50 — the page's locale
Text(format(price, .currency, "EUR"))          // €1,234.50
Text(format(count, .integer))                  // 1,235
Text(format(ratio, .percent, 1))               // 25.6%
Text(format(views, .compact))                  // 1.2K
Text(format(when, .date, "long"))              // March 5, 2024
Text(format(when, "yyyy-MM-dd HH:mm"))         // a pattern: yyyy yy MMMM MMM MM M dd d EEEE EEE HH H hh h mm ss a
Text(ago(when))                                // 3 minutes ago · yesterday · in 2 weeks
```

`format(value, .style, option)` and `ago(date)` speak the i18n locale when
the project has one (a change of locale redraws them) and the document's
language otherwise. Styles: `.number` (the default), `.integer`,
`.decimal` (option: places, 2), `.currency` (option: the code, `USD`),
`.percent` (of a fraction; option: places, 0), `.compact`, `.date`,
`.time`, `.datetime` (option: `short`, `medium`, `long`, `full`),
`.relative`. The static paint formats with a table of the common locales
and currencies and English names; the browser's `Intl` takes over on
hydration.

<!-- core -->
