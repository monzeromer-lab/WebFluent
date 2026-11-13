---
kind: "tutorial"
lang: "ar"
translates: "learn/02-tutorial-shelf.md"
route: "/ar/learn/02-tutorial-shelf"
blurb: "ابنِ موقعًا صغيرًا كاملًا — بيانات ومكوّنات ومخزن ونموذج وسمة واختبار — وانشره على الويب."
description: "درس خطوة بخطوة: ابنِ موقع قائمة قراءة بـ WebFluent، واختبره، وانشره على GitHub Pages."
learned: ["اقرأ ملف JSON وقت البناء بيانات ذات أنواع، بـ `type` و`data`", "احفظ ما يختاره القارئ عبر الزيارات في `store` مع `persist`", "اكتب مكوّنًا وضعه داخل `for … by` ذي مفاتيح", "اعرض صفحة لكل عنصر بمسار `:slug` و`paths:`", "تحقّق من نموذج، وأضف سمة، واختبر بالنقرات، وانشر الموقع"]
---
# 2. درس: Shelf

يبني هذا الدرس **Shelf**، موقعًا صغيرًا يسرد كتبًا تستحق القراءة،
ويتيح للقارئ أن يحتفظ بما يريد منها. كل خطوة تضيف قطعة واحدة،
وتنتهي بما يجب أن تراه.

## ما ستبنيه

في Shelf صفحة رئيسية تسرد الكتب مع مربع بحث، وصفحة لكل كتاب، وصفحة
**Your shelf** فيها الكتب التي حفظها القارئ، ونموذج لاقتراح كتاب، وسمة
بوضع داكن، واختبار ينقر، ونشر على GitHub Pages.

في الطريق تستخدم معظم ما يحتاجه موقع حقيقي: بيانات ذات أنواع تُقرأ وقت
البناء، ومكوّنًا يُعاد استخدامه، وصفحة لكل كتاب، ومخزنًا (store) يحفظ
اختيارات القارئ، ونموذجًا يُتحقَّق منه، وسمة، واختبارًا، ونشرًا. كل خطوة
تربط الفصل الذي يشرح فكرتها بالتفصيل.

يستغرق نحو نصف ساعة، إضافة إلى النشر.

## قبل أن تبدأ

تحتاج إلى:

- `wf` مثبّتًا وموجودًا في `PATH` ([البدء](01-getting-started.md)).
  على Windows، اكتب `webfluent` حيث يكتب هذا الدرس `wf`.
- Chrome أو Chromium على الجهاز، للاختبار الذي ينقر
  ([الخطوة 9](#الخطوة-9-اختبر-بطاقة-الكتاب)) ولـ `wf verify`
  ([الخطوة 10](#الخطوة-10-ابن-الموقع-وتحقق-منه)). إن كان مثبّتًا في
  مكان غير معتاد، فاجعل متغير البيئة `WF_CHROME` يشير إليه.
- للنشر: حساب على GitHub ومستودع اسمه `shelf`.

لا تحتاج إلى معرفة WebFluent بعد.

## الخطوة 1: أنشئ المشروع

ابدأ من القالب `static`، الذي يُصيِّر كل صفحة مسبقًا إلى HTML —
وهذا مناسب لموقع يصل إليه الناس عبر البحث:

```bash
wf init shelf -t static
cd shelf
```

ستكتب كل ملف بنفسك، لذا احذف ما وضعه القالب في `src/`
واختباراته في `tests/` (فهي تختبر صفحات القالب). ثم استبدل
`webfluent.app.json` بما يلي:

```json
{
  "name": "shelf",
  "build": { "ssg": true, "csp": true },
  "meta": {
    "title": "Shelf",
    "description": "Books worth reading, and the ones you want to.",
    "lang": "en"
  }
}
```

يُصيِّر `ssg` كل صفحة مسبقًا. ويُرفق `csp` سياسة Content-Security-Policy،
ويجعل البناء يفحص مخرجاته بها. يسرد [الإعدادات](../../reference/configuration.md)
كل مفتاح.

**تحقّق:** شغّل `wf check`. يطبع `IO Error: No .wf files found in src/` —
وهذا صحيح، لأن `src/` فارغ حتى الخطوة التالية.

## الخطوة 2: صِف الكتب

الكتب بيانات، لا شيفرة. ضعها في `src/books.json`:

```json
[
  { "slug": "notes-on-the-analytical-engine", "title": "Notes on the Analytical Engine",
    "author": "Ada Lovelace", "year": 1843, "pages": 66,
    "summary": "The first published algorithm, and a view of what a machine might do beyond arithmetic.",
    "tags": ["computing", "history"] },
  { "slug": "the-design-of-everyday-things", "title": "The Design of Everyday Things",
    "author": "Don Norman", "year": 1988, "pages": 368,
    "summary": "Why doors confuse us, and what that says about everything else we make.",
    "tags": ["design"] },
  { "slug": "a-pattern-language", "title": "A Pattern Language",
    "author": "Christopher Alexander", "year": 1977, "pages": 1171,
    "summary": "Two hundred and fifty-three patterns for towns, buildings and rooms people want to live in.",
    "tags": ["design", "architecture"] }
]
```

ثم صِف شكل البيانات واقرأ الملف، في `src/types.wf`:

```wf
/// One book on the list.
type Book {
    slug: String
    title: String
    author: String
    year: Number
    pages: Number
    summary: String
    tags: [String] = []
}

data books: [Book] = "books.json"
```

`type Book` سجلّ يُلزم مدقق الأنواع كل استخدام له: كتابة
`book.titel` خطأ في الموضع الذي تكتبه فيه. يقرأ `data books` الملف **وقت
البناء** ويجعله ثابتًا تقرؤه كل صفحة، فلا يُجلب شيء عند تحميل
الصفحة. ([الأنواع](../../learn/12-types.md)،
[جلب البيانات](../../learn/15-data.md#read-a-file-at-build-time-with-data).)

**تحقّق:** شغّل `wf check`. يطبع `No problems in shelf.`

## الخطوة 3: احفظ رفّ القارئ في مخزن

ما يحتفظ به القارئ يجب أن يبقى بعد إعادة التحميل وأن يكون هو نفسه في كل صفحة،
لذا يوضع في مخزن، وتحفظ `persist` قيمته. أنشئ
`src/stores/Shelf.wf`:

```wf
/// The books the reader wants to read, kept across visits.
store Shelf {
    persist saved: [String] = []
    derived count = saved.length

    action toggle(slug: String) {
        if saved.includes(slug) {
            saved = saved.filter(s => s != slug)
        } else {
            saved = [...saved, slug]
        }
    }
}
```

`persist` حالة (`state`) تُحفظ في تخزين المتصفح وتُقرأ من جديد في
الزيارة التالية؛ والتغيير الذي يحدث في تبويب آخر يصل إلى هنا أيضًا. ويُعاد
حساب `derived count` كلما تغيّر `saved`. ([المخازن](../../learn/11-stores.md).)

> [!NOTE]
> **قادم من React؟** الحالة المشتركة ليست مزوّد سياق (context provider) يلتفّ
> حول التطبيق، وحفظها ليس `useEffect` يكتب في
> `localStorage`. يُقرأ الـ `store` باسمه حيثما استُخدم، و
> `persist` تتولّى الحفظ. انظر [قادم من React](../../coming-from/react.md).

**تحقّق:** شغّل `wf check`. يطبع تحذيرَي `U04`:
`` `Shelf.count` is declared but never read `` ومثله لـ
`Shelf.toggle`. لا شيء يستخدم المخزن بعد؛ الخطوات التالية تستخدمه، فيزول
التحذيران.

## الخطوة 4: اعرض كتابًا بمكوّن

كل كتاب يُعرض بالطريقة نفسها، لذا هو مكوّن. أنشئ
`src/components/BookCard.wf`:

```wf
// … reads what the other steps declare
/// A book at a glance, with the button that puts it on the shelf.
component BookCard(_ book: Book) {
    use Shelf
    Card.outlined {
        Stack(gap: .sm) {
            Link(book.title, to: "/books/{book.slug}")
            Text("{book.author} · {book.year}").muted.sm
            Text(book.summary)
            Row(gap: .xs) {
                for tag in book.tags { Tag(tag) }
            }
            Button(if Shelf.saved.includes(book.slug) { "On your shelf" } else { "Want to read" }).sm {
                on click { Shelf.toggle(book.slug) }
            }
        }
    }
}
```

- `_ book: Book` هي الخاصية الوحيدة التي يمرّرها المستدعي دون اسمها:
  `BookCard(book)`.
- `Card.outlined` و`Stack(gap: .sm)` و`Text(…).muted.sm` عناصر مدمجة
  بعلامات. يعرف المترجم خصائص كل منها، ويرفض علامة لا
  يقبلها.
- يقرأ `use Shelf` المخزن من الخطوة 3. نص الزر تعبير `if`
  مستخدم قيمةً، وهو يتبع المخزن: حين ينقر القارئ، يتغيّر
  النص ولا يُعاد رسم شيء آخر.

([المكوّنات](../../learn/10-components.md)، [العناصر](../../learn/06-elements.md).)

**تحقّق:** شغّل `wf check`. يُبلغ عن خطأ واحد:
`` error[R01]: `/books/{…}` is not a route ``. تربط البطاقة بصفحة
الكتاب، ولا صفحة تجيب عن هذا العنوان بعد. يفحص المترجم كل رابط
مقابل الصفحات التي تعلنها؛ والخطوة التالية تضيف الصفحة.

## الخطوة 5: أضف الصفحة الرئيسية وصفحة لكل كتاب

تسرد الصفحة الرئيسية الكتب مع مربع بحث. أنشئ
`src/pages/Home.wf`:

```wf
// … reads what the other steps declare
page Home(path: "/", title: "Shelf", description: "Books worth reading, and the ones you want to.") {
    state q = ""
    derived needle = q.trim().toLowerCase()
    derived found = books.filter(b => b.title.toLowerCase().includes(needle) || b.author.toLowerCase().includes(needle))

    Container {
        Stack(gap: .lg) {
            Heading("Books worth reading").h1
            Input(bind: q, label: "Search by title or author").search
            Text("{found.length} of {books.length} books").muted.sm
            Grid(columns: { base: 1, md: 2 }, gap: .md) {
                for book in found by book.slug { BookCard(book) }
            }
            if found.length == 0 { Text("Nothing matches “{q}”.").muted }
        }
    }
}
```

- يُبقي `bind: q` الحقل والحالة متطابقين، في الاتجاهين.
- `found` مشتقة من `q`، فالكتابة تضيّق القائمة وأنت تكتب.
- `for … by book.slug` حلقة ذات مفاتيح: البطاقة التي تبقى في القائمة تحتفظ
  بعنصرها بدل أن يُعاد بناؤه.
- `Grid(columns: { base: 1, md: 2 })` عمود واحد على الهاتف وعمودان ابتداءً من
  نقطة التوقف `md`. يحمل HTML المُصيَّر مسبقًا هذا التخطيط أصلًا، فلا حاجة
  إلى سكربت.

> [!NOTE]
> **قادم من React؟** الحقل المتحكَّم فيه لا يحتاج إلى `value` ولا إلى
> `onChange` ينسخ `e.target.value` عائدًا: `bind: q` يفعل الأمرين. انظر
> [قادم من React](../../coming-from/react.md).

ثم أضف صفحة لكل كتاب، `src/pages/BookPage.wf`:

```wf
// … reads what the other steps declare
page BookPage(path: "/books/:slug", slug: String, title: "Book", description: "One book on the shelf.", type: "article", paths: books.map(b => b.slug)) {
    derived book = books.find(b => b.slug == slug)
    head {
        meta(property: "og:title", content: book?.title ?? "Book")
    }
    Container {
        if let b = book {
            Breadcrumb {
                Breadcrumb.Item(to: "/") { Text("Books") }
                Breadcrumb.Item { Text(b.title) }
            }
            Heading(b.title).h1
            Text("{b.author}, {b.year} · {b.pages} pages").muted
            Paragraph(b.summary)
        } else {
            Heading("No such book").h1
            Link("Back to the list", to: "/")
        }
    }
}
```

- `:slug` في المسار معامل ذو نوع للصفحة.
- يخبر `paths:` البناءَ الثابت بالقيم التي يُصيِّرها: ملف HTML لكل
  كتاب، `books/a-pattern-language/index.html` وهكذا.
- قد لا يجد `books.find(…)` شيئًا، لذا `book` من النوع `Book?` (إما `Book` أو
  `null`). يربطه `if let b = book` فقط حين يكون موجودًا. وقراءة
  `book.title` دون هذا الفحص خطأ ترجمة (`T04`).
- يضيف `head { … }` وسومًا إلى `<head>` هذه الصفحة — هنا، العنوان الذي تظهر به
  معاينة الرابط حين يُشارَك.

([الصفحات والتوجيه](../../learn/05-pages-and-routing.md).)

**تحقّق:** شغّل `wf serve` وافتح `http://localhost:3000`. ترى
**Books worth reading** و`3 of 3 books` وثلاث بطاقات. اكتب `norman` في
مربع البحث: تبقى بطاقة واحدة، ويصير العدد `1 of 3 books`. انقر **Want to read** على بطاقة فيتغيّر
نصها إلى **On your shelf**. انقر عنوان كتاب فتُفتح صفحته،
مع مسار تنقّل (breadcrumb) يعود إلى القائمة. لا يُبلغ `wf check` الآن إلا عن تحذير `U04`
الخاص بـ `Shelf.count`؛ والخطوة 6 تستخدمه.

## الخطوة 6: أضف صفحة الرفّ وإطار الموقع

تعرض صفحة الرفّ الكتب التي حفظها القارئ. أنشئ
`src/pages/ShelfPage.wf`:

```wf
// … reads what the other steps declare
page ShelfPage(path: "/shelf", title: "Your shelf", description: "The books you want to read.") {
    use Shelf
    derived mine = books.filter(b => Shelf.saved.includes(b.slug))
    Container {
        Stack(gap: .lg) {
            Heading("Your shelf").h1
            if mine.length == 0 {
                Text("Nothing yet. Pick something from the list.").muted
                Link("Browse the books", to: "/")
            } else {
                for book in mine by book.slug { BookCard(book) }
            }
        }
    }
}
```

أضف صفحة للعناوين التي لا تطالب بها صفحة أخرى، `src/pages/NotFound.wf`.
يطابق `path: "*"` أي شيء، ويكتبها البناء الثابت باسم `404.html`:

```wf
page NotFound(path: "*", title: "Not found", description: "Nothing lives at this address.", noindex: true) {
    Container {
        Heading("Nothing lives here").h1
        Link("Back to the books", to: "/")
    }
}
```

أخيرًا، الإطار الذي تتشاركه كل الصفحات، `src/App.wf`. `Router` هو حيث تُرسم
الصفحة الحالية:

```wf
// … reads what the other steps declare
app {
    Navbar {
        Navbar.Brand { Link("Shelf", to: "/") }
        Navbar.Links {
            Link("Books", to: "/")
            Link("Your shelf ({Shelf.count})", to: "/shelf")
        }
    }
    Router
    Footer { Text("Built with WebFluent").muted.sm }
}
```

**تحقّق:** يطبع `wf check` الرسالة `No problems in shelf.` صار لكل صفحة الآن
شريط تنقّل. ضع كتابين على رفّك: يصير الرابط
**Your shelf (2)**. افتحه، ثم أعد تحميل الصفحة — لا يزال الكتابان
هناك. افتح `http://localhost:3000/nowhere` فترى **Nothing lives
here**.

## الخطوة 7: أضف نموذجًا لاقتراح كتاب

يستطيع القرّاء اقتراح كتب. أنشئ `src/pages/Suggest.wf`:

```wf
page Suggest(path: "/suggest", title: "Suggest a book", description: "Tell us what belongs on the shelf.") {
    state title = ""
    state author = ""
    state why = ""
    state sent = false

    validate title { required "Which book?" }
    validate author { required "Who wrote it?" }
    validate why { maxLength(280) "Keep it under 280 characters" }

    Container {
        Stack(gap: .lg) {
            Heading("Suggest a book").h1
            if sent { Alert("Thank you — we will take a look.").success }
            Form(bind: form) {
                on submit {
                    sent = true
                    form.reset()
                }
                Input(bind: title, label: "Title")
                Input(bind: author, label: "Author")
                Textarea(bind: why, label: "Why it belongs here", rows: 4, maxLength: 280)
                Button("Send the suggestion", type: .submit).primary
            }
        }
    }
}
```

تقع كتلة `validate` بجانب الحالة التي تحرسها، ويعرض عنصر التحكم المربوط بتلك
الحالة رسائلها بطريقة يسهل الوصول إليها: يأخذ الحقل `aria-invalid`، و
تُعلَن الرسالة. والإرسال الذي يفشل ينقل التركيز إلى أول حقل
فيه مشكلة ولا يشغّل `on submit`. ([النماذج](../../learn/14-forms.md).)

ثم اربط الصفحة من الإطار. أضف الرابط الثالث إلى `src/App.wf`:

```wf
// … reads what the other steps declare
app {
    Navbar {
        Navbar.Brand { Link("Shelf", to: "/") }
        Navbar.Links {
            Link("Books", to: "/")
            Link("Your shelf ({Shelf.count})", to: "/shelf")
            Link("Suggest a book", to: "/suggest")
        }
    }
    Router
    Footer { Text("Built with WebFluent").muted.sm }
}
```

**تحقّق:** انقر **Suggest a book**، ثم **Send the suggestion** والنموذج
فارغ. لا يُرسَل النموذج: تظهر **Which book?** تحت
**Title**، و**Who wrote it?** تحت **Author**. املأ الحقلين وأرسل
مرة أخرى. ترى **Thank you — we will take a look.**، ويصير النموذج فارغًا.

الموقع الحقيقي يرسل الاقتراح إلى مكان ما. لا تحتاج إلى خادم خلفي في
هذا الدرس، لذا هذا الجزء لما بعد: صِف الخدمة مرة واحدة، في
ملف خاص بها،

```wf
// … in src/api.wf, beside the pages
api Suggestions(base: env.PUBLIC_API ?? "/api") {
    post suggest(body: Map)
}
```

واستدعها من معالج النموذج:

```wf
// … in src/pages/Suggest.wf; the rest of the page is as above
on submit {
    await Suggestions.suggest(body: { title: title, author: author, why: why })
    sent = true
    form.reset()
}
```

([البيانات وواجهات API](../../learn/15-data.md)، [البيئات](../../recipes/index.md).)

## الخطوة 8: امنح الموقع سمة

تغيّر السمة رموز التصميم (design tokens) التي يُرسم بها كل عنصر مدمج. أنشئ
`src/theme.wf`:

```wf
theme Paper {
    color-primary: #6D28D9
    font-family: Charter, Georgia, serif
    radius-md: 10px
}

theme Ink {
    color-background: #111014
    color-surface: #1A1820
    color-text: #ECEAF2
    color-text-muted: #A8A3B5
    color-border: #2B2833
}
```

ثم سمِّ السمتين في `webfluent.app.json` — `Paper` سمةً و
`Ink` سمةً داكنة:

```json
{
  "name": "shelf",
  "build": { "ssg": true, "csp": true },
  "theme": { "name": "Paper", "dark": "Ink" },
  "meta": {
    "title": "Shelf",
    "description": "Books worth reading, and the ones you want to.",
    "lang": "en"
  }
}
```

القارئ الذي ضُبط نظامه على الوضع الداكن يحصل على `Ink`، قبل الرسم الأول
ودون وميض للسمة الفاتحة. والرمز الذي لا تسمّيه يحتفظ بقيمته
الأساسية. ([التنسيق](../../learn/13-styling.md).)

**تحقّق:** الروابط وزر **Send the suggestion** بنفسجية،
والنص بخط ذي زوائد (serif). حوّل
نظامك إلى الوضع الداكن فتصير الصفحة داكنة. إن توقّف البناء برسالة
``No `theme Paper` is declared in this project``، فالإعدادات تسمّي سمة
لا يعلنها `src/theme.wf`: تحقّق من اسم الملف ومن التهجئة.

## الخطوة 9: اختبر بطاقة الكتاب

تعيش الاختبارات في `tests/`. أنشئ `tests/BookCard.wf`:

```wf
// … reads what the other steps declare
test "a card shows who wrote the book, and when" {
    BookCard(Book(slug: "x", title: "X", author: "Ada Lovelace", year: 1843, pages: 66, summary: "Notes."))
    expect "Ada Lovelace · 1843"
    expect "Want to read"
}

test "a book goes on the shelf, and comes off again" {
    BookCard(Book(slug: "x", title: "X", author: "Ada Lovelace", year: 1843, pages: 66, summary: "Notes."))
    click "Want to read"
    expect "On your shelf"
    click "On your shelf"
    expect "Want to read"
}
```

شغّلها:

```bash
wf test
```

يُصيِّر الاختبار الأول البطاقة ويقرأ ما تعرضه. ويكتب أيضًا
لقطة (snapshot) تقارن بها المرة التالية. أما الاختبار الثاني **فينقر**، لذا
يعمل في Chrome بلا واجهة (headless)، بالمترجم ووقت التشغيل نفسيهما اللذين يحصل
عليهما القارئ. تُوجد الأزرار بنصها، كما يجدها القارئ.
([الاختبار](../../learn/16-testing.md).)

**تحقّق:** ينتهي التشغيل الأول بما يلي:

```text
  new   tests/BookCard.wf — a card shows who wrote the book, and when (snapshot written)
  ok    tests/BookCard.wf — a book goes on the shelf, and comes off again (in a browser)
2 passed, 0 failed, 1 snapshot(s) written
```

شغّل `wf test` مرة أخرى فيصير السطر الأول `ok`: يطابق التصيير
لقطته.

إن قال الاختبار الثاني إنه لا يوجد متصفح، فثبّت Chrome أو Chromium، أو
اضبط `WF_CHROME` على مساره.

## الخطوة 10: ابنِ الموقع وتحقّق منه

أوقف خادم التطوير، ثم ابنِ الموقع وحمّل كل صفحة في متصفح
حقيقي:

```bash
wf build
wf verify
```

يكتب `wf build` المجلد `build/`، وهو مجلد HTML وCSS وJavaScript يستطيع أي
مضيف ثابت تقديمه، ويطبع كل تحذير، مثل `alt` مفقود، أو
صفحة بلا وصف، أو حالة لا يقرؤها شيء. ويحمّل `wf verify` كل
صفحة في متصفح حقيقي ويفشل عند خطأ سكربت، أو طلب فاشل، أو
صفحة فارغة.

**تحقّق:** ينتهي `wf build` بـ `Build complete.`، ويحوي `build/`
`index.html` و`shelf/index.html` و`suggest/index.html` و`404.html` وملف
`books/<slug>/index.html` لكل كتاب. يسرد `wf verify` ستة مسارات، كلٌّ منها `ok`،
وينتهي بـ `6 page(s), 0 problem(s)`.

## الخطوة 11: انشر على GitHub Pages

أي مضيف ثابت يقدّم `build/`. على GitHub Pages، يعيش موقع المشروع تحت
`https://<you>.github.io/shelf/`، لذا أخبر البناء بمساره الفرعي
وعنوانه. أضف هذه المفاتيح إلى `webfluent.app.json`، مع إبقاء البقية:

```json
{
  "build": { "ssg": true, "csp": true, "base_path": "/shelf" },
  "meta": { "site_url": "https://you.github.io/shelf" }
}
```

يضيف `base_path` بادئة إلى كل رابط وملف. ويفعّل `site_url` الروابط
القانونية (canonical)، وخريطة الموقع، وعناوين Open Graph المطلقة.

ثم أضف سير عمل (workflow) يبني الموقع وينشره مع كل دفع (push) إلى
`main`، `.github/workflows/pages.yml`:

```yaml
name: Deploy
on:
  push:
    branches: [main]
permissions:
  contents: read
  pages: write
  id-token: write
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/checkout@v4
      - run: curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
      - run: ~/.webfluent/bin/wf build
      - uses: actions/upload-pages-artifact@v3
        with:
          path: build
      - id: deployment
        uses: actions/deploy-pages@v4
```

ادفع المشروع إلى مستودع `shelf`. في
**Settings → Pages** الخاصة بالمستودع، اختر **GitHub Actions** مصدرًا.

**تحقّق:** يُظهر تبويب **Actions** سير العمل **Deploy** ناجحًا، و
يعرض `https://<you>.github.io/shelf/` العنوان **Books worth reading**.
يغطي [النشر](../../recipes/index.md) Netlify وVercel وCloudflare Pages
وnginx ومضيفين آخرين.

## البرنامج كاملًا

كل ملف `.wf` مما سبق، في قائمة واحدة. هذا هو البرنامج الذي بنته الخطوات،
مع أول الاختبارين:

```wf
type Book {
    slug: String
    title: String
    author: String
    year: Number
    pages: Number
    summary: String
    tags: [String] = []
}

data books: [Book] = "books.json"

store Shelf {
    persist saved: [String] = []
    derived count = saved.length

    action toggle(slug: String) {
        if saved.includes(slug) {
            saved = saved.filter(s => s != slug)
        } else {
            saved = [...saved, slug]
        }
    }
}

component BookCard(_ book: Book) {
    use Shelf
    Card.outlined {
        Stack(gap: .sm) {
            Link(book.title, to: "/books/{book.slug}")
            Text("{book.author} · {book.year}").muted.sm
            Text(book.summary)
            Row(gap: .xs) {
                for tag in book.tags { Tag(tag) }
            }
            Button(if Shelf.saved.includes(book.slug) { "On your shelf" } else { "Want to read" }).sm {
                on click { Shelf.toggle(book.slug) }
            }
        }
    }
}

app {
    Navbar {
        Navbar.Brand { Link("Shelf", to: "/") }
        Navbar.Links {
            Link("Books", to: "/")
            Link("Your shelf ({Shelf.count})", to: "/shelf")
            Link("Suggest a book", to: "/suggest")
        }
    }
    Router
    Footer { Text("Built with WebFluent").muted.sm }
}

page Home(path: "/", title: "Shelf", description: "Books worth reading, and the ones you want to.") {
    state q = ""
    derived needle = q.trim().toLowerCase()
    derived found = books.filter(b => b.title.toLowerCase().includes(needle) || b.author.toLowerCase().includes(needle))

    Container {
        Stack(gap: .lg) {
            Heading("Books worth reading").h1
            Input(bind: q, label: "Search by title or author").search
            Text("{found.length} of {books.length} books").muted.sm
            Grid(columns: { base: 1, md: 2 }, gap: .md) {
                for book in found by book.slug { BookCard(book) }
            }
            if found.length == 0 { Text("Nothing matches “{q}”.").muted }
        }
    }
}

page BookPage(path: "/books/:slug", slug: String, title: "Book", description: "One book on the shelf.", type: "article", paths: books.map(b => b.slug)) {
    derived book = books.find(b => b.slug == slug)
    head {
        meta(property: "og:title", content: book?.title ?? "Book")
    }
    Container {
        if let b = book {
            Breadcrumb {
                Breadcrumb.Item(to: "/") { Text("Books") }
                Breadcrumb.Item { Text(b.title) }
            }
            Heading(b.title).h1
            Text("{b.author}, {b.year} · {b.pages} pages").muted
            Paragraph(b.summary)
        } else {
            Heading("No such book").h1
            Link("Back to the list", to: "/")
        }
    }
}

page ShelfPage(path: "/shelf", title: "Your shelf", description: "The books you want to read.") {
    use Shelf
    derived mine = books.filter(b => Shelf.saved.includes(b.slug))
    Container {
        Stack(gap: .lg) {
            Heading("Your shelf").h1
            if mine.length == 0 {
                Text("Nothing yet. Pick something from the list.").muted
                Link("Browse the books", to: "/")
            } else {
                for book in mine by book.slug { BookCard(book) }
            }
        }
    }
}

page Suggest(path: "/suggest", title: "Suggest a book", description: "Tell us what belongs on the shelf.") {
    state title = ""
    state author = ""
    state why = ""
    state sent = false

    validate title { required "Which book?" }
    validate author { required "Who wrote it?" }
    validate why { maxLength(280) "Keep it under 280 characters" }

    Container {
        Stack(gap: .lg) {
            Heading("Suggest a book").h1
            if sent { Alert("Thank you — we will take a look.").success }
            Form(bind: form) {
                on submit {
                    sent = true
                    form.reset()
                }
                Input(bind: title, label: "Title")
                Input(bind: author, label: "Author")
                Textarea(bind: why, label: "Why it belongs here", rows: 4, maxLength: 280)
                Button("Send the suggestion", type: .submit).primary
            }
        }
    }
}

page NotFound(path: "*", title: "Not found", description: "Nothing lives at this address.", noindex: true) {
    Container {
        Heading("Nothing lives here").h1
        Link("Back to the books", to: "/")
    }
}

theme Paper {
    color-primary: #6D28D9
    font-family: Charter, Georgia, serif
    radius-md: 10px
}

theme Ink {
    color-background: #111014
    color-surface: #1A1820
    color-text: #ECEAF2
    color-text-muted: #A8A3B5
    color-border: #2B2833
}

test "a card shows who wrote the book, and when" {
    BookCard(Book(slug: "x", title: "X", author: "Ada Lovelace", year: 1843, pages: 66, summary: "Notes."))
    expect "Ada Lovelace · 1843"
    expect "Want to read"
}
```

## ما بنيته

Shelf موقع كامل، وكل جزء منه يستخدم فكرة واحدة من اللغة:

| ما كتبته | الفكرة | اقرأ المزيد |
|---|---|---|
| `type Book` و`data books` | بيانات ذات أنواع تُقرأ وقت البناء | [الأنواع](../../learn/12-types.md)، [جلب البيانات](../../learn/15-data.md#read-a-file-at-build-time-with-data) |
| `store Shelf` مع `persist` | حالة مشتركة تُحفظ عبر الزيارات | [المخازن](../../learn/11-stores.md) |
| `component BookCard` | مكوّن يُعاد استخدامه بخاصية موضعية | [المكوّنات](../../learn/10-components.md) |
| `state` و`derived` و`bind:` و`for … by` في الصفحة الرئيسية | التفاعلية والقوائم ذات المفاتيح | [الحالة والتفاعلية](../../learn/07-state-and-reactivity.md)، [التحكم في التدفق](../../learn/09-control-flow.md) |
| `path: "/books/:slug"` مع `paths:`، و`if let` | معاملات المسار، والمسارات الثابتة، والقيم الاختيارية | [الصفحات والتوجيه](../../learn/05-pages-and-routing.md) |
| `app { … Router … }` | الإطار الذي تتشاركه كل الصفحات | [الصفحات والتوجيه](../../learn/05-pages-and-routing.md) |
| `validate` و`Form(bind: form)` | نماذج يُتحقَّق منها | [النماذج](../../learn/14-forms.md) |
| `theme Paper` و`theme Ink` | رموز التصميم والوضع الداكن | [التنسيق](../../learn/13-styling.md) |
| `test "…" { … click … expect … }` | اختبارات تُصيِّر وتنقر | [الاختبار](../../learn/16-testing.md) |
| `wf build` و`wf verify` وسير عمل | البناء والفحص والنشر | [النشر](../../recipes/index.md) |

لتذهب أبعد:

- اقرأ اللغة كما ينبغي، بدءًا من [أساسيات اللغة](../../learn/04-language-basics.md).
- تحدّث مع خادم حقيقي: [البيانات وواجهات API](../../learn/15-data.md).
- اجعل الموقع يعمل دون اتصال: [العمل دون اتصال](../../recipes/index.md).
- ترجمه: [التدويل](../../recipes/index.md).
- ابنِ مشروعًا أكبر: [الدروس](03-tutorial-todos.md).

## أخطاء شائعة

قراءة قيمة قد تكون مفقودة. لا يجد `books.find(…)` شيئًا حين لا يحمل أي
كتاب هذا الـ slug، لذا `book` من النوع `Book?`:

```wf expect=T04
type Book { slug: String, title: String }

const books: [Book] = [Book(slug: "a-pattern-language", title: "A Pattern Language")]

page BookPage(path: "/books/:slug", slug: String, title: "Book", description: "One book.") {
    derived book = books.find(b => b.slug == slug)
    Heading(book.title).h1
}
```

اربطه بـ `if let b = book { Heading(b.title).h1 }`، كما تفعل الخطوة 5، أو
اقرأه بـ `book?.title ?? "Book"` ([T04](../../errors/T04.md)).

حقل لا يملكه النوع. يسمّي `type Book` كل حقل، فالخطأ المطبعي
خطأ في الموضع الذي يُكتب فيه:

```wf expect=T05
type Book { slug: String, title: String }

const books: [Book] = [Book(slug: "a-pattern-language", title: "A Pattern Language")]

page Home(path: "/", title: "Shelf", description: "Books worth reading.") {
    Heading("Books").h1
    for book in books by book.slug {
        Text(book.titel)
    }
}
```

استخدم الاسم الذي تقترحه الرسالة، `book.title` ([T05](../../errors/T05.md)).

عضو مخزن لا يعلنه المخزن. لدى `Shelf` الأعضاء `saved` و`count`
و`toggle`، ولا شيء غيرها:

```wf expect=T06
store Shelf {
    persist saved: [String] = []
    action toggle(slug: String) {
        saved = if saved.includes(slug) { saved.filter(s => s != slug) } else { [...saved, slug] }
    }
}

page Home(path: "/", title: "Shelf", description: "Books worth reading.") {
    use Shelf
    Heading("Books").h1
    Text("{Shelf.total} saved")
    Button("Keep it") { on click { Shelf.toggle("a-pattern-language") } }
}
```

اقرأ عضوًا يعلنه المخزن، أو أعلن العضو الذي تحتاجه:
`derived count = saved.length` ([T06](../../errors/T06.md)).

## التالي

[قادم من React أو Vue أو Svelte](../../coming-from/react.md).
