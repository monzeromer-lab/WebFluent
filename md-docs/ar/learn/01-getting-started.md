---
kind: "tutorial"
lang: "ar"
translates: "learn/01-getting-started.md"
route: "/ar/learn/01-getting-started"
blurb: "ثبّت المُصرِّف، وأنشئ مشروعًا، وشغّله، وغيّره وشاهد التغيير — ثم جهّز محررك أو WebFluent Studio، وتعرّف على ما يكتبه البناء."
description: "ثبّت wf وتحقّق من التثبيت، وأنشئ أول مشروع، وشغّل خادم التطوير وأجرِ تغييرًا. ثم جهّز خادم اللغة واقرأ ناتج البناء."
learned: ["تثبيت `wf` والتحقق منه بالأمر `wf --version`", "إنشاء مشروع بالأمر `wf init` وتشغيله بالأمر `wf serve`", "تغيير صفحة وقراءة الخطأ الذي يرسمه خادم التطوير فوقها", "كتابة موقع من صفحتين: `app` فيه `Router`، وصفحات تسمّي كلٌّ منها `path:` الخاص بها", "معرفة ما يكتبه `wf build` في `build/`"]
---
# 1. البدء

يثبّت هذا الفصل `wf`، مُصرِّف WebFluent، ويشغّل أول مشروع في متصفحك. في نحو
عشر دقائق تنشئ المشروع، وتغيّر سطرًا واحدًا، وترى الصفحة تتبعه. بقية الفصل
إعداد اختياري: محررك، وقوالب المشاريع الأخرى، وموقع تكتبه من الصفر، وما
يكتبه البناء.

## قبل أن تبدأ

تحتاج إلى:

- طرفية: macOS أو Linux مع `curl` و`bash`، أو Windows مع PowerShell (أو
  `cargo` الخاص بـRust على أي نظام)؛
- متصفح ويب؛
- محرر نصوص. أي محرر يصلح؛ يضيف [إعداد المحرر](#إعداد-المحرر) فحوصًا
  أثناء الكتابة، و[WebFluent Studio](#webfluent-studio) تطبيق مصنوع لـWebFluent.

## التثبيت

WebFluent ملف تنفيذي واحد هو `wf`. شغّل السطر الخاص بجهازك. كلٌّ منها يضع
`wf` على `PATH` لديك.

### macOS وLinux

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
```

يجلب السكربت الملف التنفيذي المناسب لمنصتك من أحدث إصدار. لا تحتاج إلى Rust.
يغطي أجهزة Mac بمعالجات Apple Silicon وIntel، وLinux على x86_64.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.ps1 | iex
```

> [!IMPORTANT]
> على Windows، اكتب `webfluent` حيثما يكتب هذا الدليل `wf`.

في Windows أمر اسمه `wf` أصلًا: `C:\Windows\System32\WF.msc`، وهو وحدة تحكم
جدار حماية Windows. يأتي System32 أولًا في `PATH`، و`.MSC` من الامتدادات التي
يجرّبها Windows للاسم المجرّد، فكتابة `wf` تفتح جدار الحماية بدل WebFluent.
لهذا يثبّت ملف zip الخاص بالإصدار و`install.ps1` وملف `.msi` المُصرِّفَ باسم
`webfluent.exe` بجانب `wf.exe`، ويضع `.msi` مجلدهما على `PATH`. تعمل
`webfluent build` و`webfluent serve` و`wf.exe build` كلها. بعد
`cargo install webfluent`، استخدم `wf.exe`.

### باستخدام Rust

```bash
cargo install webfluent
```

يبني هذا `wf` من crates.io. يعمل حيثما تعمل Rust، وهو طريقة التثبيت على
Linux arm64، الذي لا ملف تنفيذي جاهزًا له بعد.

### الحزم

يحمل كل [إصدار](https://github.com/monzeromer-lab/WebFluent/releases) أيضًا
ملف `.deb` وملف `.msi` وأرشيفات tar عادية، لجهاز لا يُسمح فيه بتمرير سكربت
إلى الصدفة.

## التحقق من التثبيت

اسأل `wf` عن إصداره:

```bash
wf --version
```

**ترى** الإصدار الذي ثبّته:

```text
wf 6.0.0
```

إن قالت الصدفة إنها لا تجد `wf`، فافتح طرفية جديدة. يضيف المثبّت مجلده إلى
`PATH` في ملف بدء تشغيل الصدفة، والطرفية التي شغّلته فيها قرأت ذلك الملف قبل
التغيير.

## إنشاء مشروع

أنشئ مشروعًا اسمه `hello` وانتقل إليه:

```bash
wf init hello
cd hello
```

**ترى:**

```text
Created new WebFluent project: hello (template: spa)
  AGENTS.md is the language reference, for you and your coding agent
```

يكتب `wf init` تطبيقًا صغيرًا لتجد شيئًا حقيقيًا تقرؤه: أربع صفحات (لوحة
معلومات، وقائمة مهام، وملف شخصي، وإعدادات)، وثلاثة مكوّنات، ومخزنين، مع
اختبارات تحت `tests/`.

## تشغيل خادم التطوير

شغّل خادم التطوير:

```bash
wf serve
```

**ترى:**

```text
Building hello...
  4 pages, 3 components, 2 stores
  Output: ./build/
Build complete.
Starting dev server at http://localhost:3000/
```

افتح `http://localhost:3000` في متصفحك. **ترى** صفحة **Dashboard** في
التطبيق: شريط تنقل، وثلاثة أعداد للمهام، وقائمة بالمهام الأخيرة.

اترك الخادم يعمل. يعيد البناء حين تحفظ ملفًا، ويعيد تحميل الصفحة حين يكتمل
بناء، وحين يفشل بناء يرسم الخطأ فوق الصفحة إلى أن يصلحه حفظ لاحق.

## إجراء تغيير

1. افتح `src/pages/Home.wf` في محررك.
2. جد السطر `Text("Welcome to your dashboard.").muted` وغيّر النص إلى
   `Text("Welcome back.").muted`.
3. احفظ الملف.

**ترى** المتصفح يعيد التحميل من تلقاء نفسه، ولوحة المعلومات تعرض الآن
**Welcome back.** تحت عنوانها.

لترى كيف يبدو الخطأ، غيّر `.muted` إلى `.mutd` واحفظ. تعرض الصفحة الخطأ
`E103` فوقها، مع الملف والسطر والتلميح
“Did you mean `.muted`?”.
أعده كما كان واحفظ، فتعود الصفحة.

لديك الآن مشروع يعمل. أوقف الخادم بـCtrl+C حين تنتهي. شغّل `wf build` لتكتب
الموقع النهائي في `build/`، جاهزًا لأي استضافة ثابتة
([النشر](../../recipes/index.md)).

بقية هذا الفصل إعداد اختياري. لتواصل التعلّم، انتقل إلى
[الدرس التطبيقي](02-tutorial-shelf.md).

## إعداد المحرر

يمنح خادم اللغة (language server)، `wf-lsp`، محررك الفحوص نفسها التي يجريها
`wf build`، أثناء الكتابة: التشخيصات، وإكمال الخصائص والأعلام والحالات،
والتلميح عند المرور بالأنواع والتوثيق، ومساعدة التواقيع، والانتقال إلى
التعريف، والمراجع، وإعادة التسمية، والتنسيق، والأنواع المضمّنة في السطر،
والتلوين الدلالي، والإصلاحات السريعة.

`wf-lsp` ملف تنفيذي ثانٍ. احصل عليه من
[الإصدار](https://github.com/monzeromer-lab/WebFluent/releases) نفسه الذي فيه
`wf` — `wf-lsp-<version>-<arch>-<os>.tar.gz` — وضعه على `PATH`، أو ابنه من
نسخة من المستودع:

```bash
cargo install --path crates/wf-lsp
```

ثم اربط محررك:

- **VS Code** — الإضافة [webfluent-vscode](https://github.com/monzeromer-lab/webfluent-vscode)
  تجلب التلوين وخادم اللغة. إلى أن تصل إلى Marketplace، ثبّت ملف `.vsix` من
  أحدث إصداراتها بالأمر **Extensions: Install from VSIX**.
- **Zed** — الإضافة [webfluent-zed](https://github.com/monzeromer-lab/webfluent-zed)
  تجلب تلوين Tree-sitter، والمخطط، والمقتطفات، وخادم اللغة (يُعثر عليه في
  `PATH`، أو يُنزَّل من أحدث إصدار). ثبّتها بالأمر
  **zed: install dev extension**، موجّهًا إلى نسخة منها.
- **Neovim وHelix وEmacs، أو أي محرر آخر يدعم LSP** — شغّل `wf-lsp` عبر stdio
  لنوع الملفات `webfluent` (`*.wf` و`*.wfx`).
  في [المحررات](../../reference/cli.md#editors) إعداد كلٍّ منها.

### WebFluent Studio

[WebFluent Studio](https://github.com/monzeromer-lab/webfluent-studio) تطبيق
مصنوع لكتابة WebFluent. يجمع في نافذة واحدة محرر شيفرة، ومعاينة حية، ومصممًا
مرئيًا، وGit وGitHub، وهذا الدليل، ووكيلًا. لا يستخدم `wf-lsp`: المُصرِّف يعمل
داخل التطبيق، فلا يحتاج إلى `wf` على `PATH`، ولا إلى طرفية، ولا إلى خادم
تطوير. المشروع المبني في Studio مطابق بايتًا ببايت للمشروع نفسه مبنيًا
بـ`wf build`، فتستطيع التنقل بين Studio وسطر الأوامر متى شئت.

- **المعاينة** تُعاد بناءً أثناء الكتابة، وسجل البناء ووحدة التحكم يربطان
  بالمصدر.
- **التشخيصات** فيها رسالة المُصرِّف نفسها وإصلاحها، على بعد نقرة واحدة؛ ويفتح
  **Explain** فصل هذا الدليل الخاص بذلك الرمز.
- **وضع التصميم** يحرّر الصفحة على لوحة رسم ويكتب ملف `.wf` الذي كنت ستكتبه.
- **الوكيل** يبني تغييره قبل أن تراه، ثم تقبله أو ترفضه جزءًا جزءًا.

Studio تطبيق منفصل مغلق المصدر في الإصدار 0.1، لـLinux وmacOS وWindows؛
ويليها Android. تُنشر نسخه على
[صفحة إصداراته](https://github.com/monzeromer-lab/webfluent-studio/releases).
لا تحتاج إليه لاستخدام WebFluent: `wf` وأي محرر هما سلسلة الأدوات كلها.

## تغيير طريقة تثبيت `wf`

يقرأ كلا المثبّتين متغيّري بيئة:

| المتغيّر | أثره | مثال |
|---|---|---|
| `WF_INSTALL_DIR` | يثبّت الملف التنفيذي في مكان غير `~/.webfluent/bin` | `WF_INSTALL_DIR=/opt/wf/bin` |
| `WF_VERSION` | يثبّت ذلك الإصدار بدل الأحدث | `WF_VERSION=v6.0.0` |

تشغيل المثبّت مرة أخرى يرقّي `wf` في مكانه.

للبناء من نسخة من المستودع بدلًا من ذلك — لتغيير لم يُصدَر بعد، أو للعمل على
المُصرِّف نفسه — شغّل:

```bash
git clone https://github.com/monzeromer-lab/WebFluent.git
cd WebFluent && cargo install --path .
```

## اختيار قالب

يأخذ `wf init` قالبًا بالخيار `-t`:

| القالب | ما يصنعه |
|---|---|
| `-t spa` (الافتراضي) | تطبيقًا تفاعليًا، توجيهه في المتصفح |
| `-t static` | موقعًا تسويقيًا، كل صفحة فيه مُصيَّرة مسبقًا إلى HTML |
| `-t pdf` | مستند PDF |
| `-t slides` | عرض شرائح PDF |

يبدأ `--locales en,ar` مشروع `spa` أو `static` مترجَمًا إلى أكثر من لغة: نصه في
`src/translations/`، ملف لكل لغة، واللغة الأولى هي الافتراضية
([الترجمة](../../recipes/translate-a-site.md#start-a-project-translated)).

إن لم تكن متأكدًا بين `spa` و`static`، فابدأ بـ`static`. كل صفحة HTML حقيقية
قبل أن يعمل أي سكربت، وهذا أفضل للبحث وللرسم الأول، والصفحة تبقى تفاعلية
بالكامل بعد تحميل السكربت. يشرح [ثابت أم صفحة واحدة](../../about/static-and-spa.md)
الفرق.

## كتابة موقع من الصفر

لترى أصغر موقع كامل، استبدل محتويات `src/` بملفين، واحذف اختبارات القالب
تحت `tests/`، فهي تختبر الصفحات التي أزلتها.

`src/App.wf` هو الهيكل الذي تتشاركه كل الصفحات، وفيه الموجّه:

```wf
app {
    Navbar {
        Navbar.Brand { Text("Hello").heading }
        Navbar.Links {
            Link("Home", to: "/")
            Link("About", to: "/about")
        }
    }
    Router
}
```

`src/pages/Home.wf` يحمل الصفحات، وكل صفحة تسمّي مسارها:

```wf
page Home(path: "/", title: "Hello", description: "A first WebFluent site.") {
    state count = 0
    Container {
        Heading("Hello, world").h1
        Text("You have clicked {count} times.")
        Button("Click me").primary { on click { count = count + 1 } }
    }
}

page About(path: "/about", title: "About", description: "What this site is for.") {
    Container {
        Heading("About").h1
        Paragraph("Two pages, one shell, no framework.")
    }
}
```

احفظ، فيعرض خادم التطوير شريط تنقل فيه **Home** و**About**. ويعدّ
**Click me** نقراتك. هذا ما يفعله كل جزء:

- `app { … }` هو الهيكل: ما تقع داخله كل صفحة. و`Router` هو المكان الذي تُرسم
  فيه الصفحة الحالية. لا جدول مسارات؛ كل `page` تسمّي `path:` الخاص بها.
- `page Home(path: "/", title: …, description: …)` يعلن صفحة. يصير `title`
  و`description` وسم `<title>`، ووصف meta، وبطاقات Open Graph وTwitter،
  وJSON-LD. إن أغفلتهما، ينبّهك البناء.
- `state count = 0` إشارة (signal). يقرؤها `Text("… {count} …")`، فيتحدّث
  النص حين تتغيّر.
- في `Button("Click me").primary { on click { … } }`، التسمية هي الوسيط الموضعي
  الوحيد، و`.primary` علم (حالة من حالات الخاصية `tone` للزر)، والكتلة تحمل
  معالج النقر.
- `Container` و`Heading` و`Text` و`Paragraph` و`Navbar` و`Link` عناصر مدمجة.
  يسردها كلها [مرجع المكوّنات](../../reference/components.md).

> [!NOTE]
> **قادم من React؟** لا جدول مسارات ولا `import`. كل `page` تسمّي `path:`
> الخاص بها، وكل ملف تحت `src/` جزء من برنامج واحد، فيضع `App.wf` صفحات
> ومكوّنات لا يستوردها أبدًا. راجع [قادم من React](../coming-from/react.md).

## ما ينتجه البناء

يكتب `wf build` الموقع النهائي في `build/`. بناء ثابت فيه صفحتان وصفحة
`path: "*"` يكتب ملفات مثل هذه:

```text
build/
  index.html          the page, pre-rendered (static) or the app shell (spa)
  about/index.html    one directory per route in a static build
  404.html            the `path: "*"` page, which a host serves for unknown paths
  app.js              the runtime this program needs, its stores and components
  pages/Home.js       each page in its own chunk, loaded when its route shows
  pages/Home.css      the styles only that page reaches
  styles.css          tokens, the built-ins you used, your stylesheets
  sitemap.xml         every indexable route (needs meta.site_url)
  robots.txt
  _headers            the security headers (build.csp), for hosts that read the file
  sw.js               the service worker, when the config names "offline"
  *.gz                pre-compressed copies a host can serve as they are
```

لا تعدّل شيئًا في `build/`: البناء التالي يكتبه من جديد. أبقه خارج نظام
التحكم بالإصدارات؛ ملف `.gitignore` الذي يكتبه `wf init` يفعل ذلك أصلًا.
وينسخ البناء `public/` إلى جذر `build/` كما هو.

لا يحمل `app.js` إلا أجزاء وقت التشغيل (runtime) التي يصل إليها البرنامج.
وقت التشغيل مجموعة من وحدات الميزات — `each` لحلقة `for`، و`router` لصفحة
ثانية، و`net` لـ`resource`، و`icons` لـ`Icon` — ويستنتج البناء أيّها تستدعيه
الشيفرة التي كتبها، ثم لا يُبقي إلا الدوال التي تصل إليها تلك الاستدعاءات،
بأسماء مختصرة. صفحة من نص ثابت تعطي `app.js` حجمه 1.9 kB مضغوطًا بـgzip.
يعرض `wf build --stats` وزن البناء، ويشرح [الأداء](../../about/performance.md)
كيف تقرؤه.

## المشروع باختصار

تُرتَّب ملفات المشروع هكذا:

```text
AGENTS.md             the language reference, for you and a coding agent
webfluent.app.json    the config
src/
  App.wf              the app shell (optional)
  pages/*.wf          pages
  components/*.wf     components
  stores/*.wf         stores
  *.css               your own stylesheets, bundled into styles.css
  translations/*.json translation files, with "i18n" in the config
public/               copied to the build's root (images, fonts, favicon)
tests/*.wf            `test "…" { }` declarations for `wf test`
```

يقرأ المُصرِّف كل ملف `.wf` أو `.wfx` تحت `src/` ويدمجها في برنامج واحد، فتستخدم
صفحة في ملف مكوّنًا من ملف آخر دون استيراده. المجلدات عُرف، لا قاعدة.
يغطي [هيكل المشروع](../../reference/project-structure.md) التسمية والتنظيم
والإعدادات، ويسرد [مرجع الإعدادات](../../reference/configuration.md) كل مفتاح.

لا تحتاج الإعدادات إلا إلى اسم:

```json
{ "name": "hello" }
```

## أخطاء شائعة

علم فيه خطأ إملائي. يُفحص العلم مقابل العنصر الذي يأخذه، فيكون `.mutd` خطأً،
لا نمطًا لا يفعل شيئًا بصمت:

```wf expect=E103
page Home(path: "/", title: "Hello", description: "A first site.") {
    Heading("Hello").h1
    Text("Welcome back.").mutd
}
```

استخدم أحد الأعلام التي تسردها الرسالة، وهنا `.muted`
([E103](../../errors/E103.md)).

`app` بلا `Router`. يُرسم الهيكل، لكن لا تُرسم أي صفحة أبدًا:

```wf expect=R03
app {
    Navbar {
        Navbar.Brand { Text("Hello").heading }
    }
}

page Home(path: "/", title: "Hello", description: "A first site.") {
    Heading("Hello, world").h1
}
```

ضع `Router` في `app` حيث يجب أن تظهر الصفحة الحالية
([R03](../../errors/R03.md)).

رابط إلى صفحة لم تكتبها بعد. يفحص المُصرِّف كل `to:` مقابل مسارات الصفحات:

```wf expect=R01
app {
    Navbar {
        Navbar.Links {
            Link("Home", to: "/")
            Link("About", to: "/about")
        }
    }
    Router
}

page Home(path: "/", title: "Hello", description: "A first site.") {
    Heading("Hello, world").h1
}
```

أضف الصفحة، `page About(path: "/about", …)`، أو صحّح العنوان
([R01](../../errors/R01.md)).

## التالي

[درس تطبيقي: Shelf](02-tutorial-shelf.md): ابنِ موقع قائمة قراءة من البداية إلى النهاية.
