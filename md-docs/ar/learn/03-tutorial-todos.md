---
kind: "tutorial"
lang: "ar"
translates: "learn/03-tutorial-todos.md"
route: "/ar/learn/03-tutorial-todos"
blurb: "ابنِ قائمة مهام تبقى في متصفح القارئ: أنواع، ومخزن يتذكّر، ومكوّن، ومرشّح في العنوان، واختبارات تنقر."
description: "درس WebFluent للمبتدئين: تطبيق مهام بمخزن محفوظ ومكوّن وقائمة بمفاتيح ومرشّح في العنوان واختبارات، في 20 دقيقة."
learned: ["صِف البيانات بـ`type` و`enum`", "احفظ قائمة عبر إعادة التحميل في مخزن بـ`persist`، لا تغيّرها إلا إجراءاته", "أعطِ كل صف من القائمة `state` خاصة به في مكوّن", "اربط القائمة بمفتاح بـ`for … by`، واحفظ المرشّح في العنوان بـ`query`", "اختبر صفحة بالكتابة والنقر، بـ`wf test`"]
---
# 3. درس: قائمة المهام

يبني هذا الدرس قائمة مهام تبقى في متصفح القارئ، فيها مخزن (store)
ومكوّن ومرشّح في العنوان واختبارات تنقر.

هو أول ثلاثة دروس يبني كل منها مشروعًا كاملًا، خطوة بخطوة. كل مشروع موجود كاملًا في المستودع تحت `examples/tutorials/`، ولا يحتاج أي منها إلى خادم: ما يحفظه المشروع يحفظه في متصفح القارئ.

| الدرس | ما تبنيه | ما يعلّمه | الوقت |
|---|---|---|---|
| [قائمة المهام](03-tutorial-todos.md) | قائمة بما عليك فعله: أضف، وعلّم، وأعد التسمية، ورشّح | الأنواع، ومخزن بـ`persist`، ومكوّن، وقائمة بمفاتيح، ومرشّح في العنوان، واختبارات تنقر | 20 دقيقة |
| [Field Notes](../../recipes/build-a-notebook-site.md) | موقع دفتر ملاحظات، مبني بناءً ثابتًا، بصفحة لكل ملاحظة | ملفات `data`، وتخطيط، وصفحات `:slug` مع `paths:`، وصفحات Markdown، والبحث والوسوم، وما يمكن لصفحة مُصيَّرة مسبقًا أن تعرفه | ساعة |
| [Pocket](../../recipes/build-a-budget-app.md) | متتبّع ميزانية بلوحة معلومات ونماذج ولغتين | `Money` و`Date`، والمجاميع المشتقة، وترحيل `persist`، و`validate`، ونافذة حوار واختصارات، والترجمة (i18n) والنص من اليمين إلى اليسار، وسكربت من كتابتك، والعمل دون اتصال | بضع ساعات |

إن لم تكتب WebFluent من قبل، فابدأ بـ[درس Shelf](02-tutorial-shelf.md): نصف ساعته الأولى ألطف.

**ما ستبنيه.** قائمة بما عليك فعله، تبقى في متصفح القارئ: أضف مهمة،
وعلّمها منجزة، وأعد تسميتها، واحذفها، واعرض المفتوحة منها فقط. هي صفحة
واحدة ومخزن واحد ومكوّن واحد. المشروع المكتمل هو
[`examples/tutorials/easy-todos`](https://github.com/monzeromer-lab/WebFluent/tree/master/examples/tutorials/easy-todos).

**الوقت:** نحو 20 دقيقة.

## قبل أن تبدأ

تحتاج إلى:

- `wf` مثبّتًا وفي `PATH` لديك — يطبع `wf --version` القيمة `wf 6.0.0` أو
  أحدث ([البدء](01-getting-started.md)).
- Chrome أو Chromium على الجهاز، للاختبارات التي تنقر ولـ`wf verify`.
  اضبط `WF_CHROME` على مساره إن لم يكن في مكان معتاد.
- Git، لجلب المشاريع المكتملة.

كل كتلة شيفرة يسمّي سطرها الأول ملفًا — `// src/stores/Todos.wf` — هي ذلك
الملف، أو مجموعة متتالية من أسطره، كما هي في المشروع تمامًا. يُلزم اختبارٌ
هذا الفصل بالمشاريع، فالشيفرة التي تقرؤها هنا هي الشيفرة التي تُبنى.

لتبدأ من مشروع مكتمل بدلًا من ذلك، انسخ المستودع وشغّل أحد المشاريع:

```bash
git clone https://github.com/monzeromer-lab/WebFluent
cd WebFluent/examples/tutorials/easy-todos
wf serve
```

**تحقّق:** تطبع الطرفية `Starting dev server at
http://localhost:3000/`، ويعرض ذلك العنوان المشروع.

لا تحتاج في هذا الدرس إلا إلى `wf` وChrome. تكتب كل ملف في هذا المشروع
بنفسك، والكتل أدناه تحمل كل ملف كاملًا، إلا الاختبارات، إذ يظهر منها واحد
من ستة.

## الخطوة 1: أنشئ المشروع

أنشئ مشروع صفحة واحدة وادخل إليه:

```bash
wf init todos -t spa
cd todos
```

احذف كل ما وضعه القالب في `src/` و`tests/`: ستكتب كل ملف بنفسك، واختبارات
القالب ستفشل دون صفحاته. ثم استبدل `webfluent.app.json` بهذا:

```json
{
  "name": "todos",
  "version": "1.0.0",
  "theme": { "name": "Paper", "dark": "Night" },
  "build": { "output": "./build", "output_type": "spa", "csp": true },
  "meta": {
    "title": "Todos",
    "description": "A small list of things to do, kept in your browser.",
    "lang": "en",
    "favicon": "/favicon.svg"
  }
}
```

- `"output_type": "spa"` يصنع تطبيق صفحة واحدة: غلاف HTML واحد، تُبدَّل فيه
  الصفحات دون إعادة تحميل
  ([المواقع الثابتة وتطبيقات الصفحة الواحدة](../../about/static-and-spa.md)).
- `"csp": true` يشحن سياسة أمان المحتوى ويُلزم البناء بها
  ([الأمان](../../about/security.md)).
- يسمّي `theme` سمتين (themes)، `Paper` و`Night`، تكتبهما في الخطوة التالية.

**تحقّق:** يطبع `wf check` الرسالة `IO Error: No .wf files found in src/`.
هذا متوقّع: يبقى `src/` فارغًا حتى الخطوة التالية.

## الخطوة 2: أعطه مظهرًا خاصًا به

الإعدادات تسمّي سمة، فصرّح بها أولًا. لا تسمّي السمة إلا رموز التصميم
(design tokens) التي تغيّرها؛ ويبقى كل رمز آخر على قيمته الافتراضية
([التنسيق](../../learn/13-styling.md)). `Night` هي السمة المسمّاة `"dark"` في
الإعدادات، وتستعملها الصفحة حين يكون نظام القارئ على الوضع الداكن:

```wf
// src/theme.wf
/// Warm paper and ink, with one teal for what you can act on.
theme Paper {
    color-primary: #0F766E
    color-background: #F6F4EF
    color-surface: #FFFFFF
    color-text: #1F2328
    color-text-muted: #5E6670
    color-border: #E4E0D8
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif
    radius-md: 10px
}

/// The same page after dark.
theme Night {
    color-primary: #2DD4BF
    color-background: #14171A
    color-surface: #1C2024
    color-text: #ECEEF0
    color-text-muted: #A3ABB3
    color-border: #2C3238
}
```

الصفحة التي تكتبها في الخطوة 6 تسمّي أصنافًا (classes) — `card` و`todo`
و`is-done` و`filters` — ويُنسّقها `src/todos.css`. يجمع البناء كل ملف `.css`
تحت `src/`، فلا يحتاج الملف إلى رابط. هذه أربع من قواعده؛ انسخ الملف كاملًا
من [المشروع المكتمل](https://github.com/monzeromer-lab/WebFluent/blob/master/examples/tutorials/easy-todos/src/todos.css):

```css
.todo { flex-wrap: nowrap; padding: 0.6rem 0.25rem; border-top: 1px solid var(--color-border); }
.todo__title { flex: 1; }
.todo.is-done .todo__title { text-decoration: line-through; color: var(--color-text-muted); }
.filters a.is-on { background: var(--color-background); color: var(--color-text); font-weight: 600; }
```

كل لون رمز، `var(--color-border)`، فتعيد السمة الداكنة تلوين الصفحة دون
قاعدة خاصة بها.

**تحقّق:** يطبع `wf check` الرسالة `No problems in todos.`

## الخطوة 3: صِف المهمة

يسمّي `type` الحقول التي تملكها القيمة، ويُلزم المدقّق كل استعمال بها
([الأنواع](../../learn/12-types.md)). و`enum` مجموعة ثابتة من الحالات:

```wf
// src/types.wf
/// One thing to do.
type Todo {
    id: Uuid
    title: String
    done: Bool = false
}

/// Which todos the list shows.
enum Filter { all, open, done }
```

- `Uuid` من الأنواع التي تأتي مع اللغة؛ و`uuid()` تصنع قيمة منه.
- للحقل `done` قيمة افتراضية، فيمكن للمهمة الجديدة أن تُغفله.
- تُكتب الحالة بنقطة، `.open`، حيثما يُطلب `Filter`.

لأن المدقّق يعرف الحقول، فإن `todo.titel` خطأ في الموضع الذي كُتب فيه،
والرسالة تسمّي الحقل الذي قصدته.

**تحقّق:** يطبع `wf check` الرسالة `No problems in todos.`

## الخطوة 4: اكتب مخزنًا يتذكّر

يحمل المخزن حالة يمكن لأي صفحة أو مكوّن قراءتها
([المخازن](../../learn/11-stores.md)). هذا المخزن يحفظ القائمة:

```wf
// src/stores/Todos.wf
/// The list, kept in the browser between visits.
store Todos {
    persist items: [Todo] = []

    derived open = items.filter(todo => !todo.done)
    derived done = items.filter(todo => todo.done)

    action add(title: String) {
        let text = title.trim()
        if text == "" { return }
        items = items.concat([Todo(id: uuid(), title: text)])
    }

    action toggle(id: Uuid) {
        items = items.map(todo => if todo.id == id { { ...todo, done: !todo.done } } else { todo })
    }

    action rename(id: Uuid, title: String) {
        let text = title.trim()
        if text == "" { return }
        items = items.map(todo => if todo.id == id { { ...todo, title: text } } else { todo })
    }

    action remove(id: Uuid) {
        items = items.filter(todo => todo.id != id)
    }

    action clearDone() {
        items = items.filter(todo => !todo.done)
    }
}
```

- `persist` حالة (`state`) تُحفظ في تخزين المتصفح. تبقى القائمة بعد إعادة
  التحميل، ويتبع التبويب الثاني الأول، دون أن تكتب شيفرة تخزين.
- القيمة المشتقة (`derived`) تتبع ما تقرؤه: `open` هي دائمًا المهام التي لم
  تُنجز بعد.
- الإجراء (action) هو المكان الوحيد الذي تتغيّر فيه الحالة.
  `{ ...todo, done: !todo.done }` هي المهمة نفسها مع تغيير حقل واحد.
- يغيّر `rename` و`remove` و`clearDone` القائمة بالطريقة نفسها.

> [!NOTE]
> **قادم من Vue؟** القيمة `derived` هي `computed` بلا دالة تكتبها ولا
> `.value` تقرؤه: `Todos.open` هي القائمة نفسها. انظر
> [القادمون من Vue](../../coming-from/vue.md).

**تحقّق:** يطبع `wf check` سبعة تحذيرات `U04`، مثل ``warning[U04]:
`Todos.open` is declared but never read``. هذا متوقّع: لا شيء يقرأ المخزن
بعد. الصفحة في الخطوة 6 تقرأ كل عضو فيه، فتزول التحذيرات.

## الخطوة 5: ابنِ صفًا واحدًا

المكوّن عنصر من صنعك ([المكوّنات](../../learn/10-components.md)). هذا المكوّن
يرسم مهمة واحدة، ويتيح للقارئ إعادة تسميتها في مكانها:

```wf
// src/components/TodoRow.wf
/// One row of the list: a box to tick, the title, and buttons to rename and
/// remove it.
component TodoRow(_ todo: Todo) {
    use Todos
    state editing = false
    state draft = todo.title

    action save() {
        Todos.rename(todo.id, draft)
        editing = false
    }

    Row(class: ["todo", if todo.done { "is-done" }], align: .center, gap: .sm) {
        Checkbox(checked: todo.done, aria-label: "Done: {todo.title}") {
            on change { Todos.toggle(todo.id) }
        }
        if editing {
            // `mount:` runs when the input is made: the cursor goes straight in.
            Input(bind: draft, aria-label: "New title for {todo.title}", class: "todo__edit", mount: (n) => n.focus()) {
                on key("Enter") { save() }
                on key("Escape") { draft = todo.title  editing = false }
                on blur { save() }
            }
        } else {
            Text(todo.title, class: "todo__title")
            IconButton(icon: "edit", label: "Rename {todo.title}", class: "todo__action").sm {
                on click { draft = todo.title  editing = true }
            }
        }
        IconButton(icon: "trash", label: "Remove {todo.title}", class: "todo__action").sm {
            on click { Todos.remove(todo.id) }
        }
    }
}
```

- `_ todo` هي الخاصية (prop) الموضعية الوحيدة للمكوّن، فيُكتب الاستدعاء
  `TodoRow(todo)`.
- حالته (`state`) ملك لكل صف: إعادة تسمية صف لا تضع الصفوف الأخرى في وضع
  التحرير.
- يضيف `class:` أصنافًا إلى أصناف العنصر نفسه. ويمكن لصيغة القائمة أن تحمل
  `if`، فلا تضيف `is-done` إلا ما دامت المهمة منجزة.
- لكل عنصر تحكّم اسم يُتاح لتقنيات المساعدة. مربع الاختيار وأزرار الأيقونات
  تقول أي مهمة تعمل عليها، فيمكن لقارئ الشاشة — ولاختبارات الخطوة 7 — أن
  يميّز بينها ([إمكانية الوصول](../../about/accessibility.md)).

**تحقّق:** يطبع `wf check` التحذير `U03` لـ`TodoRow` (لا شيء يضعه بعد)
وأربعة تحذيرات `U04`. لم تعد `toggle` و`rename` و`remove` الخاصة بالمخزن
تظهر: الصف يستدعيها.

## الخطوة 6: اكتب الصفحة

تحمل الصفحة مربع المهمة الجديدة والمرشّحات والقائمة. هذا نصفها الأول:

```wf
// src/pages/Home.wf
page Home(path: "/", title: "Todos", description: "A small list of things to do, kept in your browser.") {
    use Todos
    state draft = ""

    // The filter lives in the address — `/?show=open` — so a reload or a
    // shared link keeps it.
    derived filter: Filter = if query.show == "open" { .open } else if query.show == "done" { .done } else { .all }
    derived shown = match filter {
        .open { Todos.open }
        .done { Todos.done }
        .all { Todos.items }
    }

    action add() {
        Todos.add(draft)
        draft = ""
    }

    Stack(class: "card", gap: .md) {
        Row(justify: .between, align: .baseline) {
            Heading("Todos").h1
            Text("{Todos.open.length} left", class: "count")
        }

        Row(class: "new", gap: .sm) {
            Input(bind: draft, placeholder: "What needs doing?", aria-label: "New todo") {
                on key("Enter") { add() }
            }
            Button("Add", disabled: draft.trim() == "").primary {
                on click { add() }
            }
        }
```

- `bind: draft` يربط حقل الإدخال بالحالة في الاتجاهين: الكتابة تغيّر
  `draft`، و`draft = ""` تفرغ المربع.
- `query` هي سلسلة الاستعلام في العنوان بوصفها قيمة، تبقى محدّثة. لذا يتبع
  `filter` العنوان، ويتبع `shown` المرشّح `filter`.
- يجب أن يغطي `match` على `enum` كل حالة. إن أضفت حالة رابعة إلى `Filter`،
  يصبح هذا الـ`match` خطأ حتى يعالجها.

يرسم النصف الثاني المرشّحات والقائمة، ويغلق الصفحة. أضفه تحت النصف الأول،
بعد سطر فارغ:

```wf
// src/pages/Home.wf
        Row(class: "filters", gap: .xs, align: .center) {
            Link("All", to: "/?show=all", class: { "is-on": filter == .all })
            Link("Open", to: "/?show=open", class: { "is-on": filter == .open })
            Link("Done", to: "/?show=done", class: { "is-on": filter == .done })
            Spacer
            if Todos.done.length > 0 {
                Button("Clear done").sm {
                    on click { Todos.clearDone() }
                }
            }
        }

        if shown.length == 0 {
            Text(if Todos.items.length == 0 { "Nothing yet — add your first todo above." } else { "Nothing here." }, class: "empty")
        }
        Stack(class: "list") {
            for todo in shown by todo.id {
                TodoRow(todo, animate: .fadeIn, exit: .fadeOut)
            }
        }
    }
    Text("Your list stays in this browser, and nowhere else.", class: "hint")
}
```

- `for … by todo.id` قائمة بمفاتيح. حين تُعاد تسمية مهمة، أو يخفيها المرشّح،
  لا يتغيّر إلا صفها؛ وتحتفظ الصفوف الأخرى بعناصرها، ويحتفظ الصف قيد التحرير
  بمؤشّره ([التحكم في التدفق](../../learn/09-control-flow.md)).
- `animate:` و`exit:` يُظهران الصف ويُخفيانه بتلاشٍ.
- خريطة الأصناف، `{ "is-on": filter == .all }`، تفعّل الصنف ما دام شرطه
  قائمًا.

> [!NOTE]
> **قادم من React؟** يوضع المفتاح على الحلقة، `by todo.id`، لا بوصفه خاصية
> `key` على الصف. انظر [القادمون من React](../coming-from/react.md).

أخيرًا، التصريح `app` هو الإطار الذي تجلس فيه كل صفحة. يحدّد `Router` موضع
رسم الصفحة الحالية:

```wf
// src/App.wf
// The frame every page sits in: a narrow column, centred.
app {
    Container(class: "shell") {
        Router
    }
}
```

الآن شغّل خادم التطوير في طرفية ثانية، واتركه يعمل. كل حفظ يعيد بناء المشروع
ويعيد تحميل الصفحة:

```bash
wf serve
```

**تحقّق:** يطبع `wf check` الرسالة `No problems in todos.`، ويعرض
<http://localhost:3000/> بطاقة عنوانها **Todos** فيها `0 left`
و*Nothing yet — add your first todo above.* اكتب مهمة واضغط Enter: تظهر،
ويصبح العدّاد `1 left`. أعد تحميل الصفحة، فتجد المهمة ما زالت هناك. انقر
**Open**، فيصبح العنوان `/?show=open`.

إن طبع الخادم `No theme Paper is declared in this project`، فالأرجح أنك
تخطّيت `src/theme.wf` في الخطوة 2. وإن ظهر عضو في المخزن بالتحذير `U04`
هنا، فالأرجح أنك أغفلت جزءًا من الصفحة.

## الخطوة 7: اكتب الاختبارات

يرسم الاختبار ما يصرّح به، ثم ينفّذ خطواته بالترتيب. يكتب في عنصر التحكم
المسمّى **New todo** وينقر الزر المسمّى **Add**، كما يفعل القارئ
([الاختبار](../../learn/16-testing.md)). أنشئ `tests/todos.wf` بهذا الاختبار:

```wf
// tests/todos.wf
test "adding a todo puts it in the list" {
    use Todos
    state draft = ""
    Input(bind: draft, aria-label: "New todo")
    Button("Add") { on click { Todos.add(draft)  draft = "" } }
    for todo in Todos.items by todo.id { TodoRow(todo) }
    Text("{Todos.open.length} left")

    expect "0 left"
    type "Buy bread" into "New todo"
    click "Add"
    expect "Buy bread"
    expect "1 left"
}
```

الاختبار الذي ينقر أو يكتب يعمل في Chrome بلا واجهة (headless). في المشروع
المكتمل ستة اختبارات في
[`tests/todos.wf`](https://github.com/monzeromer-lab/WebFluent/blob/master/examples/tutorials/easy-todos/tests/todos.wf):
اثنان ينظران إلى صف فقط، وأربعة تنقر — إضافة مهمة، ورفض مهمة فارغة،
والتعليم ثم المسح، وإعادة التسمية مع بقاء صف واحد لكل مهمة. انسخ الملف لتحصل
عليها كلها، ثم شغّلها:

```bash
wf test
```

**تحقّق:** مع الملف كاملًا، ينتهي `wf test` بـ`6 passed, 0
failed, 2 snapshot(s) written`. الاختباران اللذان ينظران فقط يكتبان لقطة
(snapshot) لما رسماه تحت `tests/__snapshots__/`، وتُقارَن بها التشغيلات
اللاحقة.

إن أفاد اختبار ينقر بأنه لم يجد متصفحًا، فإن Chrome ليس حيث يبحث `wf`: اضبط
`WF_CHROME` على مساره.

## الخطوة 8: ابنِه

افحص المشروع واختبره وابنِه:

```bash
wf check
wf test
wf build
```

يبلّغ `wf check` عن كل نتيجة ولا يكتب شيئًا. ويكتب `wf build` المجلد
`build/`، الذي يقدّمه أي مضيف ثابت — يغطي [النشر](../../recipes/index.md)
GitHub Pages وNetlify وغيرهما. لتحمّل الصفحة المبنية في متصفح حقيقي قبل
النشر، شغّل `wf verify`: يفشل عند خطأ، أو ملف مفقود، أو صفحة لا ترسم شيئًا.

**تحقّق:** ينتهي `wf build` بـ`Build complete.`، ويحتوي `build/` على
`index.html` و`app.js` و`styles.css`.

## ما بنيته

- `type` و`enum`، يُلزَم بهما كل استعمال ([الأنواع](../../learn/12-types.md)).
- مخزن تبقى قائمته بعد إعادة التحميل بفضل `persist`، وقيم `derived` تتبعها
  ([المخازن](../../learn/11-stores.md)).
- مكوّن له حالته الخاصة، وأسماء متاحة لتقنيات المساعدة على كل عنصر تحكم
  ([المكوّنات](../../learn/10-components.md)).
- قائمة بمفاتيح، ومرشّح محفوظ في العنوان بـ`query`
  ([التحكم في التدفق](../../learn/09-control-flow.md)).
- سمة بنسخة داكنة، وورقة أنماط خاصة بالمشروع
  ([التنسيق](../../learn/13-styling.md)).
- اختبارات تنظر واختبارات تنقر ([الاختبار](../../learn/16-testing.md)).

## أخطاء شائعة

`match` يُغفل حالة. يجب أن يسمّي `match` على `enum` كل حالة، لذا فإن إغفال
`.all` خطأ:

```wf expect=T15
enum Filter { all, open, done }

page Home(path: "/", title: "Todos", description: "A small list of things to do.") {
    state filter: Filter = .all
    derived label = match filter {
        .open { "Open" }
        .done { "Done" }
    }
    Heading("Todos").h1
    Text(label)
}
```

أضف الفرع الناقص، `.all { "All" }`، أو فرع `else`
([T15](../../errors/T15.md)).

قائمة بلا `by`. دون مفتاح، يُعاد رسم الصف الذي يحمل عنصر تحكم كلما تغيّرت
القائمة، فيفقد المربع الذي كان القارئ يستعمله موضعه:

```wf expect=U09
type Todo { id: String, title: String }

page Home(path: "/", title: "Todos", description: "A small list of things to do.") {
    state items: [Todo] = [Todo(id: "1", title: "Buy bread")]
    Heading("Todos").h1
    for todo in items {
        Checkbox(label: todo.title)
    }
}
```

اجعل مفتاح القائمة ما يميّز كل عنصر: `for todo in items by todo.id`
([U09](../../errors/U09.md)).

عنصر تحكم مربوط بقيمة `derived`. يكتب `bind:` إلى ما يسمّيه، والقيمة
المشتقة تُحسَب ولا يُسنَد إليها أبدًا:

```wf expect=F01
store Todos {
    state items: [String] = []
    derived open = items.length
}

page Home(path: "/", title: "Todos", description: "A small list of things to do.") {
    use Todos
    Heading("Todos").h1
    Input(bind: Todos.open, label: "Open")
}
```

اربط `state`، كما تربط الصفحة `draft`، وغيّر المخزن عبر إجراءاته
([F01](../../errors/F01.md)).

## التالي

[درس: Field Notes](../../recipes/build-a-notebook-site.md) يبني موقعًا مولَّدًا بناءً ثابتًا على ما علّمه هذا الدرس.
