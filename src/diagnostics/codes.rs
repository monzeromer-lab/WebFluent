//! Every code a finding can carry: its severity, a title, and what it means.
//!
//! The one list the build, the language server, `wf explain`, the JSON
//! output and the guide's diagnostics chapter read. A finding with a code
//! that is not here is a compiler bug, and a test says so; so is a code here
//! that the guide does not document.
//!
//! The families:
//!
//! | | |
//! |---|---|
//! | `E` | syntax and structure — what cannot be read, or refers to nothing |
//! | `T` | types |
//! | `C` | components and their props |
//! | `R` | routes and navigation |
//! | `F` | forms and bindings |
//! | `X` | state and reactivity |
//! | `I` | translations |
//! | `D` | data, assets and what is kept in the browser |
//! | `A` | accessibility |
//! | `S` | search and sharing |
//! | `P` | persisted values |
//! | `U` | declared and never used |
//! | `V` | vocabulary and styles |

use super::Severity;

/// One code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeInfo {
    pub code: &'static str,
    pub severity: Severity,
    /// A few words: "A field or method that does not exist".
    pub title: &'static str,
    /// What it means and why it matters, in a sentence or three.
    pub summary: &'static str,
    /// Whether what it points at is code nothing needs, which an editor
    /// greys out.
    pub unnecessary: bool,
}

const fn code(
    code: &'static str,
    severity: Severity,
    title: &'static str,
    summary: &'static str,
) -> CodeInfo {
    CodeInfo {
        code,
        severity,
        title,
        summary,
        unnecessary: false,
    }
}

const fn unused(code: &'static str, title: &'static str, summary: &'static str) -> CodeInfo {
    CodeInfo {
        code,
        severity: Severity::Warning,
        title,
        summary,
        unnecessary: true,
    }
}

use Severity::{Error, Warning};

/// Every code, in the order the guide lists them.
pub static CODES: &[CodeInfo] = &[
    // ── E: syntax and structure ──────────────────────────────────────
    code(
        "E001",
        Error,
        "Text the compiler cannot read",
        "A character that begins nothing the language has, or a string, comment or splice that never closes. The file is not read past it.",
    ),
    code(
        "E002",
        Error,
        "A syntax error",
        "The text reads, but not as the language: a missing brace, a stray word, an argument list that does not close. The message names what was expected and what was found.",
    ),
    code(
        "E003",
        Error,
        "Indentation that lines up with no block",
        "In a `.wfx` file a line opens a block by being followed by a deeper one, and closes blocks by coming back out. A line that comes back to an indentation no enclosing block has is ambiguous.",
    ),
    code(
        "E004",
        Error,
        "A file in the WebFluent 2 grammar",
        "WebFluent 3 reads one grammar. `wf migrate` rewrites a WebFluent 2 project in place.",
    ),
    code(
        "E005",
        Error,
        "A file the formatter will not change",
        "`wf fmt` holds its result to the file's tokens; when formatting would change what a file says, it leaves the file alone and says so.",
    ),
    code(
        "E101",
        Error,
        "A component nothing declares",
        "An element, a `layout:` or a part names a component that no `component` declares and no built-in has. Often a misspelling.",
    ),
    code(
        "E102",
        Error,
        "A name declared twice",
        "Two declarations with one name in one scope: two stores, two constants, a state and a derived of one name. It compiled to a script the browser refused, or let the second quietly replace the first.",
    ),
    code(
        "E103",
        Error,
        "A flag, case, part, event or slot the component does not have",
        "What a component takes comes from one registry for the built-ins and from the declaration for your own; a word that resolves to nothing there does nothing on screen, so it is refused.",
    ),
    code(
        "E104",
        Error,
        "A layout with nowhere to put the page",
        "A page's `layout:` is a component the page is placed in, at its default slot. A component that declares no `slot` has nowhere to put it.",
    ),
    code(
        "E105",
        Error,
        "A statement a render block cannot hold",
        "A page's or component's body draws; an assignment or a call that does something belongs in a handler, an `action` or an `effect`.",
    ),
    code(
        "E106",
        Error,
        "Script in an attribute",
        "`onclick:` and the other `on*` attributes are script in a string, which the content security policy refuses and nothing checks. A handler is `on click { … }`.",
    ),
    code(
        "E107",
        Error,
        "A URL a browser would run",
        "`javascript:`, `data:`, `vbscript:`, `blob:` and `file:` URLs run or load something the page did not write. A link takes `http`, `https`, `mailto`, `tel`, `sms`, `ftp` or a relative URL.",
    ),
    code(
        "E108",
        Error,
        "An `env` name that is not public",
        "Whatever a page reads from `env` is in the bundle, where anyone can read it. A name a page reads must begin `PUBLIC_` or be listed in `public_env`.",
    ),
    code(
        "E109",
        Error,
        "An element this output cannot draw",
        "A PDF or a slide deck has no buttons, inputs, handlers or routes; an interactive element in one would be drawn as nothing.",
    ),
    code(
        "E110",
        Error,
        "A project script that cannot be a plain script",
        "A `.js` file under `src/` is linked as a plain browser script: one with `import` or `export`, one a file in `public/` would replace, or one whose names clash with the program's, cannot be.",
    ),
    code(
        "E111",
        Error,
        "A setting that cannot work",
        "A value in `webfluent.app.json` that names something that does not exist or asks for something the build cannot do.",
    ),
    code(
        "E112",
        Warning,
        "A setting nothing reads",
        "A key in `webfluent.app.json` that is not a setting. Usually a misspelling, so the setting it meant keeps its default.",
    ),
    code(
        "E113",
        Error,
        "An argument that cannot be positional",
        "Only the first argument of a call may be written without its name, and only when the component declares a positional prop (`_ name`). Name the others.",
    ),
    code(
        "E114",
        Error,
        "`emit` outside a component",
        "An event belongs to a component, which declares it and fires it with `emit`; a page has none.",
    ),
    code(
        "E115",
        Warning,
        "A project script the compiler could not read",
        "The file is still linked and may run fine in a browser, but the compiler could not read what it declares, so its names are not in scope for the program.",
    ),
    code(
        "E116",
        Error,
        "A tag that cannot be a custom element's",
        "`Element` names a custom element by its tag, first and as a string: lower case, with a hyphen (`\"stripe-pricing-table\"`).",
    ),
    code(
        "E117",
        Error,
        "A key combination no keyboard sends",
        "`on key(…)` names modifiers (`ctrl`, `shift`, `alt`, `meta`) and one key; a modifier it does not know, no key, or a key no keyboard has would never match.",
    ),
    code(
        "E901",
        Error,
        "The compiler wrote JavaScript a browser would refuse",
        "The build reads back every script it writes, and this one would not run. It is a bug in WebFluent, not in the program; please report it with the source that produced it.",
    ),
    code(
        "E902",
        Error,
        "Output its own security policy refuses",
        "The build holds every page to the content security policy it ships beside it — as it wrote the page, and as the page's own `head { }` tags would add to it; something there would be blocked.",
    ),
    // ── T: types ─────────────────────────────────────────────────────
    code(
        "T01",
        Error,
        "A value of the wrong type",
        "A value given to a prop, a state, a field, a parameter or an assignment that is not of the type it is declared to hold.",
    ),
    code(
        "T02",
        Error,
        "A case the enum does not have",
        "`.loud` given where an enum is wanted that has no case `loud`.",
    ),
    code(
        "T04",
        Error,
        "A value that may be null, read as if it were not",
        "A field or method read from a value that may be `null`. `if let`, `??`, `?.` or a `!= null` check narrows it.",
    ),
    code(
        "T05",
        Error,
        "A field or method that does not exist",
        "A record, a map literal or a built-in value read for a field or method it does not have.",
    ),
    code(
        "T06",
        Error,
        "A member a store or service does not have",
        "A store's state, derived value or action, or a service's endpoint, that it does not declare.",
    ),
    code(
        "T07",
        Error,
        "A condition that is always true",
        "A list, a record or an action used as a condition: it is never `false`, so the branch always runs.",
    ),
    code(
        "T08",
        Error,
        "A loop over something that is not a list",
        "A `for` over a number, a string or a record.",
    ),
    code(
        "T09",
        Error,
        "An `emit` that does not match its event",
        "The arguments an `emit` passes are not the ones the event declares.",
    ),
    code(
        "T10",
        Error,
        "A call with the wrong arguments",
        "Too many or too few arguments, a name the callee does not take, or one of the wrong kind.",
    ),
    code(
        "T11",
        Error,
        "A `match` on something that cannot be matched",
        "A `match` is over a resource (`loading`, `error`, `ready`) or an enum (`.case`); its arms are of that kind.",
    ),
    code(
        "T12",
        Error,
        "A secret where it would escape",
        "A `Secret` shown by an element, spliced into text, logged or kept with `persist`.",
    ),
    code(
        "T13",
        Error,
        "A name nothing declares",
        "A name, or a function called, that no declaration, parameter, script or browser global provides — a ReferenceError the first time the page reads it.",
    ),
    code(
        "T16",
        Error,
        "A method a number, a string or a list does not have",
        "In the browser the call is `x.method is not a function`, the first time the line runs. The message suggests the nearest method there is.",
    ),
    code(
        "T14",
        Error,
        "A comparison that is always the same",
        "`==` or `!=` between values that can never be equal — a string and a number, a record and a string, an enum and a case it does not have — so the answer never changes.",
    ),
    code(
        "T17",
        Error,
        "An async result used before it is awaited",
        "An action that awaits hands back a promise; read as its result without `await` it is not one. `await` is written in an action, a handler, a timer or a hook — not in a `derived` value or an effect, which run at once — and `.pending` is for an action that awaits.",
    ),
    code(
        "T19",
        Warning,
        "A value that may be null, shown as text",
        "Spliced into text, `null` shows as `null` (and a missing value as `undefined`). Say what to show instead with `??`.",
    ),
    code(
        "T20",
        Warning,
        "A list or a record shown as text",
        "A list in text runs its items together with commas, and a map or a record shows as `[object Object]`. Show a field, or join a list of text.",
    ),
    code(
        "T18",
        Error,
        "Arithmetic on something that is not a number",
        "`-`, `*`, `/` and `%` take numbers; on a string or a list the result is `NaN`. `+` joins text when one side is a string, and takes numbers otherwise.",
    ),
    code(
        "T15",
        Error,
        "A `match` that misses a case, or has one twice",
        "A `match` with no `else` must give every case of the enum an arm — a value with a case it misses would match nothing. A case with two arms reaches only the first. A `match` expression that covers every case needs no `else`.",
    ),
    code(
        "T21",
        Warning,
        "A resource `match` with no `error` arm",
        "When the request fails, a `match` with no `error` arm and no `else` shows nothing at all, and the reader is not told.",
    ),
    // ── C: components ────────────────────────────────────────────────
    code(
        "C01",
        Error,
        "A required prop or field left out",
        "A component's prop, or a record's field, that has no default is not given. The value would be `undefined` where the declaration promises one.",
    ),
    code(
        "C02",
        Error,
        "A prop the component does not declare",
        "Your own component is called with a prop it does not declare. Nothing in it reads the value, so the call does not do what it says; often a misspelling.",
    ),
    code(
        "C03",
        Error,
        "A part outside the component it belongs to",
        "`Select.Option`, `Table.Row`, `Card.Header` — a part is placed inside its owner. Outside it, it draws a stray element with nothing around it.",
    ),
    code(
        "C04",
        Warning,
        "An attribute a built-in does not declare",
        "A named argument a built-in has no prop for is written to its element as an HTML attribute. `aria-*`, `data-*` and the global attributes are expected; anything else is usually a misspelling.",
    ),
    code(
        "C05",
        Warning,
        "A positional argument a built-in does not take",
        "The built-in takes its arguments by name; one written without a name is not given to anything.",
    ),
    code(
        "C06",
        Warning,
        "A positional argument bound by order",
        "Your component declares no positional prop (`_ name`), so the argument is bound to its first prop. Mark that prop with `_` to say so.",
    ),
    // ── R: routes ────────────────────────────────────────────────────
    code(
        "R01",
        Error,
        "A route to nothing",
        "A link, a `navigate()` or a `Route` names a page or a path that no page has.",
    ),
    code(
        "R02",
        Error,
        "A route parameter and a page parameter that do not match",
        "A route's `:name` fills the page's parameter of that name; a `:name` with no parameter is read as nothing, and a parameter with no `:name` is never filled.",
    ),
    code(
        "R03",
        Error,
        "An `app` with no `Router`, or with two",
        "The current page is drawn where `app` places its `Router`. With none, no page is ever drawn; with two, the page has two places.",
    ),
    code(
        "R04",
        Warning,
        "A relative URL on a nested route",
        "`\"images/x.png\"` is fetched relative to the page's address, so on `/blog/post` it is `/blog/images/x.png`. Written from the root, `\"/images/x.png\"`, it is the same file on every page.",
    ),
    // ── X: state and reactivity ──────────────────────────────────────
    code(
        "X01",
        Error,
        "An assignment to something that cannot change",
        "A `const`, a `derived` value, a prop, a route parameter, a loop variable or an action is assigned to. At run time the write throws, or changes a copy nothing reads.",
    ),
    code(
        "X02",
        Error,
        "An effect that feeds itself",
        "An effect re-runs when what it reads changes. One that writes, every run, something it reads starts the next run itself: the page never settles, and the browser overflows its stack.",
    ),
    code(
        "X04",
        Error,
        "A derived value that changes something",
        "A `derived` value is worked out whenever what it reads changes, as often as the page needs it; one that calls an action that assigns changes state each time it is read.",
    ),
    code(
        "X05",
        Error,
        "`use` of a store nothing declares",
        "`use` names a store the page depends on; one that is not declared is a misspelling, and every read of it is nothing.",
    ),
    // ── F: forms and bindings ────────────────────────────────────────
    code(
        "F01",
        Error,
        "A `bind:` to something that cannot be written",
        "`bind:` writes what the control holds back to what it names: a state, a store's state, or a field of a loop's item. A constant, a derived value, a prop or a value worked out on the spot cannot take it.",
    ),
    code(
        "F02",
        Error,
        "A `bind:` to a state the control cannot hold",
        "A text field holds text, a number field (`.number`) a number, a checkbox a `Bool`. Bound to a state of another type, it writes the wrong kind of value into it.",
    ),
    code(
        "F03",
        Error,
        "A `validate` block no control can satisfy",
        "The rules of a `validate` block are shown on the control bound to the state, and a form is valid when they pass. On a state no control binds, or a derived value, they can never pass, and the submit stays disabled.",
    ),
    code(
        "F04",
        Warning,
        "A checkbox that shows a state and never changes it",
        "`Checkbox(checked: x)` with no `bind:` and no `on change` draws `x`, and a click toggles the box while `x` stays as it was.",
    ),
    code(
        "F05",
        Warning,
        "A select whose value is none of its options",
        "The bound state starts as a value no option has, so the select shows its first option while the state holds something else.",
    ),
    // ── D: data and assets ───────────────────────────────────────────
    code(
        "D01",
        Warning,
        "A file the project does not have",
        "A site-relative `src:`, `poster:`, `captions:` or `transcript:` names a file that is not in `public/`; the page asks for it and gets nothing.",
    ),
    code(
        "D02",
        Error,
        "A persisted value storage cannot keep",
        "`persist` writes its value to the browser's storage as JSON; a function, a file or a promise does not survive it.",
    ),
    code(
        "D03",
        Warning,
        "A persisted value every instance shares",
        "A `persist` in a component placed more than once, or in a loop, is one stored value for all of them. `key:` gives each its own.",
    ),
    code(
        "D04",
        Warning,
        "A persisted shape changed with no new version",
        "What a returning reader's browser holds is the old shape; the page reads it as the new one. Raising `version:`, with a `migrate` step, brings it forward.",
    ),
    code(
        "D05",
        Warning,
        "An asset from another origin with no integrity hash",
        "A stylesheet, font or script loaded from another origin is code that origin can change after you read it. A `meta.integrity` hash is how the browser checks it has not.",
    ),
    // ── I: translations ──────────────────────────────────────────────
    code(
        "I01",
        Error,
        "A message no translation has",
        "`t(\"key\")` names a key no locale's file holds, so the page shows the key itself.",
    ),
    code(
        "I02",
        Error,
        "A message and its call that disagree",
        "The call passes a value the message does not use, or the message shows a placeholder the call does not pass, which then shows as written.",
    ),
    code(
        "I03",
        Warning,
        "A message one locale has and another does not",
        "A reader in the locale that lacks it sees the key instead of the words.",
    ),
    code(
        "I04",
        Error,
        "A locale the project does not have",
        "`setLocale(\"xx\")` names a locale `i18n.locales` does not list.",
    ),
    // ── A: accessibility ─────────────────────────────────────────────
    code(
        "A01",
        Warning,
        "An image with no alt text",
        "An `Image` with no `alt`. A screen reader reads the file name; a decorative image takes `alt: \"\"`.",
    ),
    code(
        "A02",
        Warning,
        "An icon button with no name",
        "An `IconButton` with no `label:`, which is its accessible name.",
    ),
    code(
        "A03",
        Warning,
        "An input with no label",
        "An `Input` with neither `label:` nor `placeholder:`.",
    ),
    code(
        "A04",
        Warning,
        "A control with no label",
        "A `Checkbox`, `Radio`, `Switch`, `Slider` or `Textarea` with no `label:`.",
    ),
    code(
        "A05",
        Warning,
        "A button with no text",
        "A `Button` with no text.",
    ),
    code(
        "A06",
        Warning,
        "A link with no text",
        "A `Link` with no text.",
    ),
    code(
        "A07",
        Warning,
        "An empty heading",
        "A `Heading` with no text.",
    ),
    code(
        "A08",
        Warning,
        "A dialog with no title",
        "A `Modal` or `Dialog` with no `title:`, which is its accessible name.",
    ),
    code(
        "A09",
        Warning,
        "Media nobody can follow without sound",
        "A `Video` with no `controls`, or a `Video` or `Audio` with no `captions:` or `transcript:`.",
    ),
    code(
        "A10",
        Warning,
        "A table with no header row",
        "A `Table` with no `Table.Head`.",
    ),
    code(
        "A11",
        Warning,
        "A skipped heading level",
        "A heading level skipped — `h2` straight to `h4`.",
    ),
    code(
        "A12",
        Warning,
        "A page without exactly one h1",
        "A page with no `h1`, or more than one.",
    ),
    code(
        "A13",
        Warning,
        "Colours without enough contrast",
        "A theme's colour pairing falls below the WCAG AA contrast ratio.",
    ),
    code(
        "A14",
        Warning,
        "A role whose children are the wrong kind",
        "A `role:` that requires particular children holds a control that is not one.",
    ),
    code(
        "A15",
        Warning,
        "An `aria-label` that does not contain the visible text",
        "A control's `aria-label` does not contain the text it shows, so what a voice-control user says does not match what they see.",
    ),
    code(
        "A16",
        Warning,
        "One id on several elements",
        "A literal `id:` inside a `for`, or in a component placed more than once, gives several elements one id; a label's `for` and `aria-*` find only the first.",
    ),
    // ── S: search and sharing ────────────────────────────────────────
    code(
        "S01",
        Warning,
        "A page with no title",
        "A page has no `title:`.",
    ),
    code(
        "S02",
        Warning,
        "A page with no description",
        "A page has no `description:`, so its search snippet is written for it.",
    ),
    code(
        "S03",
        Warning,
        "A description too long for a search result",
        "A description longer than about 160 characters, which a search result truncates.",
    ),
    code(
        "S04",
        Error,
        "Two pages on one route",
        "Two pages claim one path, and only one of them can ever be reached.",
    ),
    // ── P: persisted values ──────────────────────────────────────────
    code(
        "P01",
        Warning,
        "A persisted value with no way forward",
        "A `persist` at `version: n` with nothing to bring an older version forward, so what a returning reader had is discarded.",
    ),
    code(
        "P02",
        Warning,
        "A migration that never runs",
        "A `migrate` step above the declared `version:`.",
    ),
    code(
        "P03",
        Warning,
        "A persisted value in a route-scoped store",
        "The route change drops the store, and the next read builds it again from storage, so the value comes straight back.",
    ),
    // ── U: declared and never used ───────────────────────────────────
    unused(
        "U01",
        "A state nothing reads",
        "A `state` nothing in its page or component reads; an assignment alone does not read it.",
    ),
    unused(
        "U02",
        "A derived value nothing reads",
        "A `derived` nothing reads.",
    ),
    unused(
        "U03",
        "A component nothing places",
        "A component nothing places, names as a layout, or reaches as a part.",
    ),
    unused(
        "U04",
        "A store member nothing reads",
        "A store's state, derived value or action that nothing reads, inside the store or as `Store.member`.",
    ),
    unused(
        "U05",
        "An action nothing calls",
        "An action nothing calls; a name that starts with `_` is understood to be unused on purpose.",
    ),
    unused(
        "U06",
        "Code after return",
        "What follows a `return` in the same block never runs.",
    ),
    unused(
        "U07",
        "An allow that allows nothing",
        "A `// wf-allow(CODE)` comment covers the next line of code (or its own, at the end of one); when nothing it names is reported there, or it names an error no allow may silence, it is reported itself — so allows cannot outlive what they were written for.",
    ),
    unused(
        "U08",
        "A condition that is always the same",
        "A literal condition, or a value compared with itself, decides the same way every time; one branch never runs.",
    ),
    code(
        "U09",
        Warning,
        "An unkeyed loop whose items hold state",
        "With no `by`, a change to the list redraws every item, and what an item holds — a field being typed in, a component's own state — starts again.",
    ),
    unused(
        "U10",
        "An `else` no value reaches",
        "Every case of the enum has its own arm, so the `match`'s `else` never runs.",
    ),
    // ── V: vocabulary ────────────────────────────────────────────────
    code(
        "V01",
        Warning,
        "A bare word that means nothing",
        "A bare word in an argument that nothing in scope declares — a misspelled name, or a flag written without its dot.",
    ),
    code(
        "V02",
        Warning,
        "A flag no stylesheet styles",
        "A flag whose class no stylesheet — the engine's or one of the project's — defines.",
    ),
    code(
        "V03",
        Warning,
        "Markup put in as markup",
        "An `Unsafe.Html`, and whether what it puts in went through `sanitize`.",
    ),
    code(
        "V04",
        Warning,
        "A class that is the engine's",
        "A `class:` naming one of the built-ins' `wf-*` classes, which brings that built-in's rules with it.",
    ),
    code(
        "V05",
        Warning,
        "A CSS property no browser knows",
        "A misspelled property — `colr:` — is ignored by the browser; nothing tells you why the style did not take.",
    ),
    code(
        "V06",
        Error,
        "A design token the theme does not declare",
        "`$name` compiles to `var(--name)`; a token the theme does not declare is never set, so the property falls back to nothing.",
    ),
    code(
        "V07",
        Warning,
        "A number where CSS wants a length",
        "`width: {pct}` with a number is a length with no unit, which the browser drops. Give it one: `{pct}%`.",
    ),
    code(
        "V08",
        Warning,
        "An icon the runtime does not draw",
        "`Icon` and `IconButton` draw the built-in icons; any other name shows as the word.",
    ),
    code(
        "V09",
        Warning,
        "An event the element does not fire",
        "A handler names neither a DOM event nor one the element declares, so it never runs; often a misspelling (`on clik`).",
    ),
    code(
        "V10",
        Warning,
        "A `Host` tag it does not make",
        "A `Host` is made of one of a few elements — `div`, `span`, `canvas`, `svg`, `section`, `figure`, `pre`, `p`, `ul`, `table`; any other tag is a `div`.",
    ),
];

/// The errors a project may lower with `lints`: each is a mistake, and none
/// ships a page that breaks — a link to a route that is missing, a prop
/// nothing reads, a page no route reaches, a message that shows its key.
const LOWERABLE_ERRORS: &[&str] = &["R01", "C02", "S04", "I01", "I02", "I04"];

/// Whether `lints` may lower a code: any warning, and the errors that do
/// not ship a broken page.
pub fn lowerable(code: &str) -> bool {
    match info(code) {
        Some(c) => c.severity != Severity::Error || LOWERABLE_ERRORS.contains(&code),
        None => false,
    }
}

/// The entry for a code.
pub fn info(code: &str) -> Option<&'static CodeInfo> {
    CODES.iter().find(|c| c.code == code)
}

/// The family a code belongs to: its letter.
pub fn family(code: &str) -> &str {
    &code[..code.len().min(1)]
}

/// Where the guide explains a code: the anchor the site gives the
/// chapter's `### CODE — Title` heading.
pub fn docs_url(code: &str) -> String {
    let anchor = match info(code) {
        Some(c) => slug(&format!("{} — {}", c.code, c.title)),
        None => code.to_lowercase(),
    };
    format!("https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#{anchor}")
}

/// A heading's anchor, as the site writes it (`scripts/site-from-guide.py`):
/// backticks off, lower case, every run of anything but `a–z` and `0–9` a
/// hyphen, none at either end.
pub fn slug(heading: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in heading.replace('`', "").to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.push(c);
        } else {
            gap = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_is_listed_once_and_well_formed() {
        let mut seen = std::collections::HashSet::new();
        for c in CODES {
            assert!(seen.insert(c.code), "{} is listed twice", c.code);
            assert!((3..=4).contains(&c.code.len()), "{}", c.code);
            assert!(
                "ETCRFXIDASPUV".contains(family(c.code)),
                "{} is in no family",
                c.code
            );
            assert!(
                c.code[1..].chars().all(|ch| ch.is_ascii_digit()),
                "{}",
                c.code
            );
            assert!(!c.title.is_empty() && !c.summary.is_empty(), "{}", c.code);
        }
    }

    #[test]
    fn every_title_is_its_heading_in_the_guide() {
        // The heading is the anchor `docs_url` links to.
        let chapter = include_str!("../../md-docs/39-diagnostics.md");
        for c in CODES {
            let heading = format!("### {} — {}\n", c.code, c.title);
            assert!(
                chapter.contains(&heading),
                "the guide's heading for {} is not `{}`",
                c.code,
                heading.trim()
            );
        }
        assert_eq!(
            docs_url("T05"),
            "https://monzeromer-lab.github.io/WebFluent/docs/guide/diagnostics#t05-a-field-or-method-that-does-not-exist"
        );
    }
}
