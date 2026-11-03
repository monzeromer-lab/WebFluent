  // ─── i18n ────────────────────────────────────────────
  // The languages written right to left; a locale is one when its
  // language is (`ar-EG` as `ar`). The compiler's `i18n::table::RTL`.
  const RTL_LOCALES = new Set(["ar", "arc", "ckb", "dv", "fa", "he", "iw", "ks", "nqo", "ps", "sd", "syr", "ug", "ur", "yi"]);
  const isRtl = (l) => RTL_LOCALES.has(String(l).split(/[-_]/)[0].toLowerCase());

  /// Whether a table has plural forms of `key`.
  const PLURAL_FORMS = ["zero", "one", "two", "few", "many", "other"];
  const hasForms = (m, key) => {
    for (const f of PLURAL_FORMS) if (m[key + "." + f] !== undefined) return true;
    return false;
  };

  function locales(defaultLocale, translations) {
    const locale = signal(defaultLocale);
    const dir = signal(isRtl(defaultLocale) ? "rtl" : "ltr");

    function t(key, params) {
      const currentLocale = locale();
      // A plain key, the common case: one lookup, the reader's table then
      // the default's, else the key itself — no forms to try, nothing to fill.
      if (params === undefined) {
        const own = translations[currentLocale];
        if (own && typeof own[key] === "string") return own[key];
        const base = translations[defaultLocale];
        if (base && typeof base[key] === "string" && !(own && hasForms(own, key))) return base[key];
        if (!(own && hasForms(own, key)) && !(base && hasForms(base, key))) return key;
      }
      const messages = translations[currentLocale] || translations[defaultLocale] || {};
      const fallback = translations[defaultLocale] || {};
      const FORMS = ["zero", "one", "two", "few", "many", "other"];
      const holds = (m) => m[key] !== undefined || FORMS.some((f) => m[key + "." + f] !== undefined);
      // A key's forms all come from one language: the reader's when it
      // has the key at all, else the default's — never one form of each.
      const source = holds(messages) ? messages : fallback;
      const lookup = (k) => source[k];
      let text;
      // A `count` picks the plural form the locale's rules give it —
      // `key.one`, `key.few` … — and exactly none is `key.zero` wherever
      // the file has one, in any language; then `key.other`.
      // Literal text names its number after its expression, `{files}
      // files`: when the table has forms of the key and the call names no
      // `count`, its one number is the count, so Arabic's six forms pick by
      // it as a `t()` key's do.
      let count = params && typeof params.count === "number" ? params.count : undefined;
      if (count === undefined && params && PLURAL_FORMS.some((f) => source[key + "." + f] !== undefined)) {
        const numbers = Object.values(params).filter((v) => typeof v === "number");
        if (numbers.length === 1) count = numbers[0];
      }
      if (count !== undefined) {
        if (count === 0) text = lookup(key + ".zero");
        if (text === undefined && typeof Intl !== "undefined" && Intl.PluralRules) {
          let category = "other";
          try { category = new Intl.PluralRules(currentLocale).select(count); } catch (e) { /* unknown locale */ }
          text = lookup(key + "." + category);
        }
        if (text === undefined) text = lookup(key + ".other");
      }
      if (text === undefined) text = lookup(key);
      // No table has it: the key itself — under `i18n.literals`, the words
      // as written — its placeholders filled as a message's are.
      if (text === undefined) text = key;
      // Interpolate {placeholder} tokens
      if (params && text.includes("{")) {
        for (const [k, v] of Object.entries(params)) {
          text = text.replace(new RegExp("\\{" + k + "\\}", "g"), String(v));
        }
      }
      return text;
    }

    function setLocale(newLocale, remember) {
      locale.set(newLocale);
      // The reader's choice is kept, as `setTheme`'s is: a static site loads
      // each page afresh, and one that picked Arabic was handed the next
      // page in English. Only a choice is kept — a `?lang=` address opens
      // that language without changing what the reader picked.
      if (remember !== false) {
        try {
          if (newLocale === defaultLocale) window.localStorage.removeItem("wf:locale");
          else window.localStorage.setItem("wf:locale", newLocale);
        } catch (e) { /* no storage */ }
      }
      const newDir = isRtl(newLocale) ? "rtl" : "ltr";
      dir.set(newDir);
      document.documentElement.setAttribute("lang", newLocale);
      document.documentElement.setAttribute("dir", newDir);
    }

    i18nInstance = { t, locale, dir, setLocale };
    // `?lang=ar` opens the page in that locale. It is the address the
    // build's `hreflang` alternates give each language, so a reader who
    // follows one from a search result gets the page they were promised.
    try {
      const asked = typeof location !== "undefined" && location.search
        ? new URLSearchParams(location.search).get("lang")
        : null;
      if (asked && translations[asked]) {
        if (asked !== defaultLocale) setLocale(asked, false);
      } else {
        let kept = null;
        try { kept = window.localStorage.getItem("wf:locale"); } catch (e) { /* no storage */ }
        if (kept && kept !== defaultLocale && translations[kept]) setLocale(kept, false);
      }
    } catch (e) { /* no location to read */ }
    return i18nInstance;
  }
