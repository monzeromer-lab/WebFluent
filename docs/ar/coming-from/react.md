# WebFluent لمطوّري React

> أفكار React في WebFluent: الحالة، والقيم المشتقة، والتأثيرات، والخصائص، والأبناء، والأحداث، والسياق، والمسارات، والقوائم، والجلب، جنبًا إلى جنب.

The page: https://webfluent.monzeromer.dev/ar/coming-from/react

معظم ما تعرفه من React ينتقل معك: مكوّنات لها خصائص (props)، وحالة،
وموجّه مسارات (router)، ومتاجر مشتركة. الفرق الذي عليك أن تتذكّره أن
WebFluent مترجم (compiler) بترميز ذي أنواع، وليس مكتبة: جسم المكوّن يعمل
مرة واحدة، ولا يتحدّث إلا النص أو السمة أو النمط الذي يقرأ قيمة تغيّرت.
لا خطافات (hooks)، ولا مصفوفات اعتماديات، ولا دوال ضبط (setters).

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
| خاصية دالة استدعاء راجع (callback) | `onChange={(v) => setLast(v)}` | `event change(value: Number)`, handled as `on change(value) { last = value }` |
| حقل إدخال متحكَّم به | `value={q} onChange={(e) => setQ(e.target.value)}` | `Input(bind: q, label: "Search")` |
| حالة مشتركة بين الصفحات | `createContext`, Redux, Zustand | `store Cart { … }`, read as `Cart.items` after `use Cart` |
| مسار | `<Route path="/users/:id" element={<User />} />` | `page User(path: "/users/:id", id: String) { … }` |
| إظهار شيء بشرط | `{open && <Panel />}` | `if open { Panel }` |
| قائمة | `{items.map((i) => <li key={i.id}>{i.name}</li>)}` | `for i in items by i.id { Text(i.name) }` |
| الجلب | `useQuery({ queryKey: ["users"], queryFn })` | `resource users = fetch("/api/users")`, read with `match users { … }` |
| التنسيق | `className`, CSS modules, Tailwind | flags (`Button("Save").primary`), `style { }` on the element, design tokens |
| اختبار | Testing Library: `render`, `fireEvent.click`, `getByText` | `test "…" { click "Save"  expect "Saved" }` |

## ما يختلف

**إنه مترجم، لا مكتبة.** لا يوجد DOM افتراضي ولا إعادة رسم. كل نص أو سمة
أو نمط يقرأ حالة يتبعها وحده، وجسم المكوّن يعمل مرة واحدة، حين يوضع. لهذا
لا خطافات، ولا قواعد عن ترتيب استدعائها.

**الترميز ذو أنواع.** يعرف المترجم خصائص كل عنصر مدمج وكل مكوّن تصرّح به،
وعلاماته (flags) وأحداثه وفتحاته (slots). `Button("Save").primray` لا
يُبنى، ولا يُبنى كذلك معالج لحدث لا يطلقه المكوّن أبدًا (`E103`). والخاصية
المجهولة على مكوّنك أنت هي `C02`، لا تمرير صامت.

**الصفحات تملك مساراتها.** لا يوجد جدول مسارات. كل `page` تسمّي `path:`
الخاص بها، ويبني المترجم الموجّه منها، مرتّبة حسب دقّتها، فلا يستطيع ترتيب
تصريحك بالصفحات أن يخفي مسارًا.

**لا استيراد ولا npm.** كل ملف تحت `src/` جزء من برنامج واحد، فالمكوّن
المصرَّح به في أي مكان يُستعمل في كل مكان. مكتبة JavaScript هي عنوان URL في
`meta.scripts`، وشيفرة JavaScript الخاصة بك ملف `.js` عادي تحت `src/`
تستدعي شيفرة `.wf` دوالّه بأسمائها
([استدعِ JavaScript الخاصة بك](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)).

**البناء هو الموقع.** العرض المسبق، وتقسيم الشيفرة، وتغيير مقاسات الصور،
وخريطة الموقع، ومعاينات الروابط، وسياسة Content-Security-Policy، كلها تأتي
من `wf build`، بلا إضافات تختار بينها. لا عرض على الخادم مع كل طلب: الموقع
ملفات ثابتة وبيئة تشغيل في المتصفح، تتحدّث إلى واجهتك البرمجية (API).

**النص نص.** الطريقة الوحيدة لوضع ترميز في صفحة هي `Unsafe.Html(…)`، التي
ينبّه عليها البناء في كل مرة، أما المعالجات المضمّنة وعناوين `javascript:`
فلا تُترجَم ([الأمان](https://webfluent.monzeromer.dev/about/security.md)).

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
- **لا `className`.** `Badge` يأتي مصمَّمًا، و`.info` علامة يفحصها
  المترجم.
- **الحدث مصرَّح به.** المستدعي الذي يعالج `on chnage` يحصل على `E103` وقت
  البناء، بدل معالج لا يعمل أبدًا.
