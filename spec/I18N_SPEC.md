# WebFluent Internationalization (i18n) Specification

> Version 1.0 — Draft
> Author: Monzer Omer
> Date: 2026-03-24

---

## Overview

WebFluent provides **built-in internationalization** for building multilingual single-page applications. Translations are defined as JSON files, one per locale. A single `t()` function handles all text lookups with reactive locale switching — changing the language instantly updates every translated string in the UI.

---

## 1. Project Setup

### Translation Files

Place one JSON file per locale in the translations directory:

```
src/
  translations/
    en.json
    ar.json
    es.json
    fr.json
```

Each file is a flat or dot-keyed map of translation keys to strings:

```json
// src/translations/en.json
{
    "app.name": "TaskFlow",
    "nav.home": "Home",
    "nav.about": "About",
    "nav.settings": "Settings",
    "home.title": "Welcome",
    "home.description": "Manage your tasks with ease.",
    "home.greeting": "Hello, {name}!",
    "tasks.add": "Add Task",
    "tasks.empty": "No tasks yet.",
    "tasks.remaining": "{count} tasks remaining",
    "actions.save": "Save",
    "actions.cancel": "Cancel",
    "actions.delete": "Delete"
}
```

```json
// src/translations/ar.json
{
    "app.name": "تاسك فلو",
    "nav.home": "الرئيسية",
    "nav.about": "حول",
    "nav.settings": "الإعدادات",
    "home.title": "مرحباً",
    "home.description": "أدر مهامك بسهولة.",
    "home.greeting": "أهلاً، {name}!",
    "tasks.add": "أضف مهمة",
    "tasks.empty": "لا توجد مهام بعد.",
    "tasks.remaining": "{count} مهام متبقية",
    "actions.save": "حفظ",
    "actions.cancel": "إلغاء",
    "actions.delete": "حذف"
}
```

### Configuration

Add i18n config to `webfluent.app.json`:

```json
{
    "name": "My App",
    "i18n": {
        "defaultLocale": "en",
        "locales": ["en", "ar", "es"],
        "dir": "src/translations"
    }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `defaultLocale` | String | `"en"` | Initial locale on app load |
| `locales` | String[] | `["en"]` | Supported locale codes |
| `dir` | String | `"src/translations"` | Directory containing translation JSON files |

---

## 2. The `t()` Function

### Basic Usage

```wf
Text(t("home.title"))
Heading(t("app.name"), h1)
Button(t("actions.save"), primary)
```

`t("key")` returns the translated string for the current locale. It is **reactive** — when the locale changes, all `t()` calls automatically re-evaluate.

### Interpolation

Translation strings can contain `{placeholder}` tokens. Pass values as named arguments:

```wf
// Translation: "Hello, {name}!"
Text(t("home.greeting", name: user.name))

// Translation: "{count} tasks remaining"
Text(t("tasks.remaining", count: tasks.length))
```

### Fallback Behavior

If a key is missing in the current locale:
1. Fall back to the `defaultLocale`
2. If still missing, return the key itself (e.g., `"home.title"`)

---

## 3. Locale Switching

### `setLocale(code)`

Switch the active locale at runtime:

```wf
Button("English") { setLocale("en") }
Button("العربية") { setLocale("ar") }
Button("Español") { setLocale("es") }
```

When `setLocale()` is called:
- All `t()` calls reactively update
- `<html lang>` attribute updates
- `<html dir>` attribute updates (`rtl` for Arabic, Hebrew, Farsi, Urdu; `ltr` for all others)

### `locale` Variable

Access the current locale code:

```wf
Text("Language: {locale}")

if locale == "ar" {
    Text("You are viewing the Arabic version")
}
```

### `dir` Variable

Access the current text direction:

```wf
// "ltr" or "rtl"
Text("Direction: {dir}")
```

---

## 4. RTL Support

WebFluent automatically detects RTL locales and sets the document direction. RTL locales:
- `ar` — Arabic
- `he` — Hebrew
- `fa` — Farsi/Persian
- `ur` — Urdu

When an RTL locale is active:
- `<html dir="rtl">` is set
- CSS logical properties in the design system handle layout mirroring
- No manual RTL handling needed in components

---

## 5. Full Example

### Translation Files

```json
// src/translations/en.json
{
    "nav.home": "Home",
    "nav.about": "About",
    "welcome": "Welcome to our app!",
    "greeting": "Hello, {name}!",
    "lang.switch": "Switch Language"
}
```

```json
// src/translations/ar.json
{
    "nav.home": "الرئيسية",
    "nav.about": "حول",
    "welcome": "!مرحباً بك في تطبيقنا",
    "greeting": "!أهلاً، {name}",
    "lang.switch": "تغيير اللغة"
}
```

### App Code

```wf
App {
    Navbar {
        Navbar.Brand {
            Text(t("nav.home"), heading)
        }
        Navbar.Links {
            Link(to: "/") { Text(t("nav.home")) }
            Link(to: "/about") { Text(t("nav.about")) }
        }
        Navbar.Actions {
            Dropdown(label: t("lang.switch")) {
                Dropdown.Item { Button("English") { setLocale("en") } }
                Dropdown.Item { Button("العربية") { setLocale("ar") } }
            }
        }
    }

    Router {
        Route(path: "/", page: Home)
        Route(path: "/about", page: About)
    }
}
```

```wf
Page Home (path: "/") {
    Container {
        Heading(t("welcome"), h1, fadeIn)
        Text(t("greeting", name: "Monzer"))
    }
}
```

### Config

```json
{
    "name": "My App",
    "i18n": {
        "defaultLocale": "en",
        "locales": ["en", "ar"],
        "dir": "src/translations"
    }
}
```

---

## 6. Compilation Output

### What the compiler does

1. Reads all JSON files from the translations directory
2. Bakes all translations into the generated `app.js` as a JavaScript object
3. Creates the i18n runtime at the top of `app.js`
4. Compiles `t("key")` → `WF.i18n.t("key")`
5. Compiles `t("key", name: val)` → `WF.i18n.t("key", { name: val })`
6. Compiles `setLocale("ar")` → `WF.i18n.setLocale("ar")`
7. Compiles `locale` → `WF.i18n.locale()`

### Generated JavaScript (conceptual)

```javascript
// i18n setup with baked-in translations
WF.i18n = WF.createI18n("en", {
    "en": { "nav.home": "Home", "greeting": "Hello, {name}!", ... },
    "ar": { "nav.home": "الرئيسية", "greeting": "!أهلاً، {name}", ... }
});

// Usage in generated page function
const _e0 = WF.h("p", { className: "wf-text" }, () => WF.i18n.t("greeting", { name: _name() }));
```

### Why `t()` is reactive

`t()` reads from a signal (`locale`). Any `effect()` or DOM binding that calls `t()` automatically subscribes to locale changes. When `setLocale()` updates the signal, all subscribers re-render.

---

## 7. Grammar

No new grammar rules needed. `t()` is parsed as a regular function call:

```ebnf
// Already covered by existing grammar:
FunctionCall = IDENT "(" ArgList? ")" ;
```

The compiler recognizes `t` and `setLocale` as special built-in functions during code generation.
