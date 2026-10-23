  // ─── i18n ────────────────────────────────────────────
  // The languages written right to left; a locale is one when its
  // language is (`ar-EG` as `ar`). The compiler's `i18n::table::RTL`.
  const RTL_LOCALES = new Set(["ar", "arc", "ckb", "dv", "fa", "he", "iw", "ks", "nqo", "ps", "sd", "syr", "ug", "ur", "yi"]);
  const isRtl = (l) => RTL_LOCALES.has(String(l).split(/[-_]/)[0].toLowerCase());

  function locales(defaultLocale, translations) {
    const locale = signal(defaultLocale);
    const dir = signal(isRtl(defaultLocale) ? "rtl" : "ltr");

    function t(key, params) {
      const currentLocale = locale();
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
      if (params && typeof params.count === "number") {
        if (params.count === 0) text = lookup(key + ".zero");
        if (text === undefined && typeof Intl !== "undefined" && Intl.PluralRules) {
          let category = "other";
          try { category = new Intl.PluralRules(currentLocale).select(params.count); } catch (e) { /* unknown locale */ }
          text = lookup(key + "." + category);
        }
        if (text === undefined) text = lookup(key + ".other");
      }
      if (text === undefined) text = lookup(key);
      // Fallback to key itself
      if (text === undefined) return key;
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
