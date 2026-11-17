# WebFluent لمطوّري React

> أفكار React في WebFluent: الحالة، والقيم المشتقة، والتأثيرات، والخصائص، والأبناء، والأحداث، والسياق، والمسارات، والقوائم، والجلب، جنبًا إلى جنب.

The page: https://webfluent.monzeromer.dev/ar/coming-from/react

معظم ما تعرفه من React ينتقل معك: مكوّنات لها خصائص (props)، وحالة،
وموجّه مسارات (router)، ومخازن (stores) مشتركة. الفرق الذي عليك أن تتذكّره أن
WebFluent مُصرِّف (compiler) بترميز ذي أنواع، وليس مكتبة: جسم المكوّن يعمل
مرة واحدة، ولا يتحدّث إلا النص أو الصفة أو النمط الذي يقرأ قيمة تغيّرت.
لا خطافات (hooks)، ولا مصفوفات اعتماديات، ولا دوال ضبط (setters).

> [!NOTE]
> لم تُترجم كل فصول الدليل بعد: الروابط إلى فصل غير مترجم تفتح نسخته
> الإنجليزية.

## الخريطة

| المفهوم | React | WebFluent |
|---|---|---|
| قيمة تتغيّر | `const [n, setN] = useState(0)` | `state n = 0` |
| تغييرها | `setN(n + 1)` | `n = n + 1` |
| قيمة تُحسب من قيم أخرى | `const total = useMemo(() => sum(items), [items])` | `derived total = items.sum()` |
| تأثير جانبي | `useEffect(() => { start(); return () => stop() }, [id])` | `effect { start(id)  cleanup { stop() } }` |
| مقبض على عنصر | `const box = useRef(null)` … `box.current.focus()` | `Input(bind: q, ref: box)` … `box.focus()` |
| مكوّن | `function Card({ title }) { … }` | `component Card(_ title: String) { … }` |
| ما يضعه المستدعي في الداخل | `{props.children}` | `children` |
| خاصية دالة استدعاء راجع (callback) | `onChange={(v) => setLast(v)}` | `event change(value: Number)`، تُعالَج بـ`on change(value) { last = value }` |
| حقل إدخال متحكَّم به | `value={q} onChange={(e) => setQ(e.target.value)}` | `Input(bind: q, label: "Search")` |
| حالة مشتركة بين الصفحات | `createContext`, Redux, Zustand | `store Cart { … }`، تُقرأ `Cart.items` بعد `use Cart` |
| مسار | `<Route path="/users/:id" element={<User />} />` | `page User(path: "/users/:id", id: String) { … }` |
| إظهار شيء بشرط | `{open && <Panel />}` | `if open { Panel }` |
| قائمة | `{items.map((i) => <li key={i.id}>{i.name}</li>)}` | `for i in items by i.id { Text(i.name) }` |
| الجلب | `useQuery({ queryKey: ["users"], queryFn })` من TanStack Query، أو `useEffect` مع `fetch` | `resource users: [User] = http.get("/api/users")`، تُقرأ بـ`match users { … }` |
| التنسيق | `className`, CSS modules, Tailwind | أعلام (`Button("Save").primary`)، و`style { }` على العنصر، ورموز التصميم |
| اختبار | Testing Library: `render`, `fireEvent.click`, `getByText` | `test "…" { click "Save"  expect "Saved" }` |

## ما يختلف

**إنه مُصرِّف، لا مكتبة.** لا يوجد DOM افتراضي ولا إعادة رسم. كل نص أو صفة
أو نمط يقرأ حالة يتبعها وحده، وجسم المكوّن يعمل مرة واحدة، حين يوضع. لهذا
لا خطافات، ولا قواعد عن ترتيب استدعائها.

**الترميز ذو أنواع.** يعرف المُصرِّف خصائص كل عنصر مدمج وكل مكوّن تصرّح به،
وأعلامه (flags) وأحداثه وفتحاته (slots). `Button("Save").primray` لا
يُبنى، ولا يُبنى كذلك معالج لحدث لا يطلقه المكوّن أبدًا (`E103`). والخاصية
المجهولة على مكوّنك أنت هي `C02`، لا تمرير صامت.

**الأنواع بلا لغة ثانية.** لا TypeScript بجانب الترميز ولا خطوة `tsc`:
ملف `.wf` يفحص أنواعه المُصرِّف نفسه الذي يبنيه. يستنتج الفاحص ما يستطيع،
ويطلب منك كتابة الباقي: حالة لا تقول قيمتها الأولى شيئًا (`state items:
[Todo] = []`)، وإجراء يُرجع قيمة (`-> Number`)، ومعامل. لا `any`: JSON الذي
لم يُحدَّد شكله هو `Json`، يُطابَق أو يُفكّ (`User.decode(raw)`) قبل أن
يُقرأ، ولا إعداد يُطفئ الفحوص.

**الصفحات تملك مساراتها.** لا يوجد جدول مسارات. كل `page` تسمّي `path:`
الخاص بها، ويبني المُصرِّف الموجّه منها، مرتّبة حسب دقّتها، فلا يستطيع ترتيب
تصريحك بالصفحات أن يخفي مسارًا.

**لا استيراد ولا npm.** كل ملف تحت `src/` جزء من برنامج واحد، فالمكوّن
المصرَّح به في أي مكان يُستعمل في كل مكان. مكتبة JavaScript هي عنوان URL في
`meta.scripts`، وشيفرة JavaScript الخاصة بك ملف `.js` عادي تحت `src/`
تستدعي شيفرة `.wf` دوالّه بأسمائها
([استدعِ JavaScript الخاصة بك](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)).

**البناء هو الموقع.** الصفحات المُصيَّرة مسبقًا، وتقسيم الشيفرة، وتغيير مقاسات الصور،
وخريطة الموقع، ومعاينات الروابط، وسياسة Content-Security-Policy، كلها تأتي
من `wf build`، بلا إضافات تختار بينها. لا تصيير على الخادم مع كل طلب: الموقع
ملفات ثابتة ووقت تشغيل (runtime) في المتصفح، تتحدّث إلى واجهتك البرمجية (API).

**النص نص.** الطريقة الوحيدة لوضع ترميز في صفحة هي `Unsafe.Html(…)`، التي
ينبّه عليها البناء في كل مرة، أما المعالجات المضمّنة وعناوين `javascript:`
فلا تُصرَّف ([الأمان](https://webfluent.monzeromer.dev/about/security.md)).

**استعماله داخل تطبيق React.** انشر المكوّنات عناصرَ مخصّصة (custom
elements) وضع وسومها في JSX
([انشر المكوّنات عناصرَ مخصّصة](https://webfluent.monzeromer.dev/recipes/publish-custom-elements.md)).

## مكوّن، منقولًا

العدّاد نفسه، في React وفي WebFluent. يبدأ من رقم يمرّره المستدعي، ويزيد
عند النقر، ويُظهر شارة حين يكون العدد زوجيًا، ويُخبر المستدعي بكل تغيير.

```jsx
function Counter({ start = 0, onChange }) {
  const [n, setN] = useState(start);
  const even = useMemo(() => n % 2 === 0, [n]);
  return (
    <div className="row">
      <button onClick={() => { setN(n + 1); onChange?.(n + 1); }}>+</button>
      <span>{n}</span>
      {even && <span className="badge">even</span>}
    </div>
  );
}
```

```wf
page Scores(path: "/", title: "Scores", description: "A counter that reports each change.") {
    state last = 0
    Heading("Scores").h1
    Counter(start: 2) { on change(value) { last = value } }
    Text("Last change: {last}")
}

component Counter(start: Number = 0) {
    event change(value: Number)
    state n = start
    derived even = n % 2 == 0
    Row(gap: .sm, align: .center) {
        Button("+") { on click { n = n + 1  emit change(n) } }
        Text("{n}")
        if even { Badge("even").info }
    }
}
```

- **لا خطافات ولا مصفوفات اعتماديات.** `derived` يعرف ما يقرؤه.
- **لا دالة ضبط.** أنت تُسند القيمة: `n = n + 1`.
- **لا `className`.** `Badge` يأتي مصمَّمًا، و`.info` علم يفحصه
  المُصرِّف.
- **الحدث مصرَّح به.** المستدعي الذي يعالج `on chnage` يحصل على `E103` وقت
  البناء، بدل معالج لا يعمل أبدًا.

## البيانات كما يفعلها TanStack Query

TanStack Query ذاكرةٌ للطلبات التي يطلبها المكوّن. في WebFluent تتوزّع
المهام نفسها كلٌّ في مكانه: ما تملكه الخدمة عقدٌ، وكيف تُبلَغ هو الخدمة،
وما تقرؤه الصفحة `resource` — قيمةٌ واحدة لها ثلاث حالات.

| TanStack Query | WebFluent |
|---|---|
| `useQuery({ queryKey: ["users", page], queryFn })` | `resource users = Backend.users.list(page: page)`: الوسائط متتبَّعة، فتغيّر `page` يعيد الطلب |
| `isPending` و`isError` و`data` و`error` | قيمةٌ واحدة: `match users { .loading … .failed(e) … .loaded(list) … }`؛ و`match` بلا `.failed` لا يُبنى |
| `isFetching` والبيانات على الشاشة | `.loaded(list, refreshing, stale)`: القائمة باقية و`refreshing` صحيحة |
| `staleTime` و`gcTime` | ما يقوله `Cache-Control` من الخادم (RFC 9111)، أو `cache: .swr(60.seconds)` على نقطة النهاية |
| `refetchOnWindowFocus` و`refetchInterval` | `refresh: .onFocus` و`refresh: .every(30.seconds)` |
| `useMutation({ mutationFn })` | إجراءٌ يستدعي نقطة النهاية ويطابق نتيجتها بـ`match` |
| `queryClient.invalidateQueries(["users"])` بعد التعديل | `invalidates [list]` على الكتابة في العقد: كل قراءةٍ لها تُعاد، بلا استدعاءٍ يُكتب |
| `onMutate` بلقطةٍ و`onError` يتراجع | `let undo = Backend.users.list.update(list => …)` ثم `undo()` إن فشلت الكتابة |
| `useInfiniteQuery` مع `getNextPageParam` | `pages .cursor(param: after, next: p => p.next)` على نقطة النهاية؛ و`users.loadMore()` و`users.hasMore` |
| `retry: 3` | `retry: .backoff(times: 3)` على الخدمة لما يصحّ إرساله مرتين؛ و`POST` فقط إن كانت نقطة النهاية `idempotent` |
| `error` من نوع `unknown` أو `Error` | `-> User ! CreateUserError`: تعدادُ أخطاءٍ فيه حالةٌ لكل حالةٍ يجيب بها الخادم، تُطابَق واحدةً واحدة |
| `openapi-typescript` و`orval` | `wf api add openapi.json`: العقد ملفاتٍ، يُفحص في CI بـ`wf api check` |
| Mock Service Worker في الاختبارات | `fake` للعقد يُلزمه المُصرِّف به، ويُسمّى في `fakes:` الاختبار |

القائمة نفسها، مع إضافة تُحدّثها وخطأ حقل يُعرض:

```jsx
function People() {
  const qc = useQueryClient();
  const { data, isPending, error } = useQuery({ queryKey: ["people"], queryFn: listPeople });
  const add = useMutation({
    mutationFn: createPerson,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["people"] }),
  });
  if (isPending) return <Spinner />;
  if (error) return <Alert>{error.message}</Alert>;
  return (
    <>
      {data.map((p) => <p key={p.id}>{p.name}</p>)}
      <button onClick={() => add.mutate({ name: "Grace" })}>Add</button>
      {add.error?.status === 422 && <p>{add.error.body.errors[0].message}</p>}
    </>
  );
}
```

```wf
type Person { id: String, name: String }
type NewPerson { name: String }
type FieldError { field: String, message: String }
type ValidationProblem = Problem { errors: [FieldError] }

enum CreatePersonError {
    unprocessableContent(problem: ValidationProblem) on 422
    http(error: HttpError)
}

api Backend(base: "/api") { }

api Backend.people {
    get list() at "people" -> [Person]!
    post create(body: NewPerson) at "people" -> Person ! CreatePersonError
        invalidates [list]
}

page People(path: "/", title: "People", description: "The team.") {
    state said = ""
    resource people = Backend.people.list()
    Heading("People").h1
    match people {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(list) { for p in list by p.id { Text(p.name) } }
    }
    Button("Add") {
        on click {
            match await Backend.people.create(body: NewPerson(name: "Grace")) {
                .ok(_) { said = "" }
                .err(.unprocessableContent(p)) { said = p.errors.first()?.message ?? "Check the name" }
                .err(.http(e)) { said = e.message }
            }
        }
    }
    Text(said)
}
```

- **لا مفاتيح للاستعلامات.** القراءة تُعرف بنقطة نهايتها ووسائطها، وقارئان
  للعنوان نفسه يرسلان طلبًا واحدًا.
- **الإبطال جزءٌ من العقد**، يُقال مرةً بجانب الكتابة ويُفحص: قراءةٌ ليست
  في المجموعة خطأ `N05`.
- **الخطأ مكتوب النوع.** كل حالةٍ يعلنها الخادم حالةٌ في التعداد، و`match`
  يُسقط واحدةً لا يُبنى.
