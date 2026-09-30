//! `output_type: "android"` — the web build, as an Android app.
//!
//! The app is one WebView showing the build, which travels inside it. The
//! pages are served from the app's own files on a host Android keeps for
//! them, `appassets.androidplatform.net`, so they have a real `https` origin
//! — storage, `fetch` and the Content-Security-Policy behave as they do on
//! the web — and the app works with no network from its first launch. The
//! WebView is Chromium, the engine `wf verify` and `wf test` already drive,
//! so what they check is what the app shows.
//!
//! What is written falls in two halves. The Gradle files, the manifest and
//! the activity are written once, when they are missing, and are the
//! author's from then on: signing, a permission, a deep link or a screen of
//! native code goes there. What the WebFluent project decides is written on
//! every build, and only where it changed, so Gradle's incremental build
//! has nothing to redo: the web build under `assets/www`, the launcher
//! icons, the app's name and colours in `webfluent.xml`, and its id, version
//! and Android levels in `app/webfluent.properties`, which the Gradle file
//! reads.
//!
//! The activity uses the platform's own classes and nothing else, so a
//! project depends on one thing, the Android Gradle plugin.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};

use image::imageops::{self, FilterType};
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};

use crate::config::ProjectConfig;
use crate::error::{Result, WebFluentError};
use crate::parser::Program;

/// The Android Gradle plugin a new project is written with.
const AGP_VERSION: &str = "8.13.0";
/// The Gradle that plugin runs on, for Android Studio to fetch.
const GRADLE_VERSION: &str = "8.14.3";
/// The oldest Android the activity runs on: a `WebResourceRequest` reaches
/// `shouldOverrideUrlLoading` from Android 7.0.
const MIN_SDK: u32 = 24;
/// The least the activity is compiled against: it names classes from
/// Android 13 (predictive back) and constants from Android 15.
const COMPILE_SDK: u32 = 36;
/// The largest version code Google Play accepts.
const MAX_VERSION_CODE: u32 = 2_100_000_000;

const SETTINGS: &str = include_str!("template/settings.gradle.kts");
const ROOT_BUILD: &str = include_str!("template/build.gradle.kts");
const APP_BUILD: &str = include_str!("template/app.build.gradle.kts");
const GRADLE_PROPERTIES: &str = include_str!("template/gradle.properties");
const WRAPPER: &str = include_str!("template/gradle-wrapper.properties");
const GITIGNORE: &str = include_str!("template/gitignore");
const MANIFEST: &str = include_str!("template/AndroidManifest.xml");
const ACTIVITY: &str = include_str!("template/MainActivity.java");

/// Each screen density a launcher asks for: the legacy icon's size, and the
/// adaptive icon's layer size, in pixels.
const DENSITIES: [(&str, u32, u32); 5] = [
    ("mdpi", 48, 108),
    ("hdpi", 72, 162),
    ("xhdpi", 96, 216),
    ("xxhdpi", 144, 324),
    ("xxxhdpi", 192, 432),
];

/// What a build wrote into the Android project.
pub struct Written {
    /// The project's directory, as the config names it.
    pub dir: String,
    /// Whether the project was new, so its Gradle files were written.
    pub created: bool,
    /// The id and version the app was built with, for the build to print.
    pub summary: String,
    /// What the build noticed about the app, for the author.
    pub notes: Vec<String>,
}

/// What `build.android` asks for, refused where it cannot mean it. Checked
/// before anything is built, so a mistake costs nothing.
pub fn check(config: &ProjectConfig, project_dir: &Path) -> Vec<String> {
    let android = &config.build.android;
    let mut problems = Vec::new();
    let id = android.application_id.trim();
    if id.is_empty() {
        problems.push(
            "`build.android.application_id` is not set. It is the id the app is known by on a \
             phone and in Google Play — `com.yourname.ledger` — and it can never change once the \
             app is published, so the build does not guess one"
                .to_string(),
        );
    } else if let Err(why) = check_application_id(id) {
        problems.push(format!("`build.android.application_id` is `{id}`: {why}"));
    }
    if android.min_sdk < MIN_SDK {
        problems.push(format!(
            "`build.android.min_sdk` is {}; the app needs Android 7.0, which is {MIN_SDK}",
            android.min_sdk
        ));
    }
    if android.target_sdk < android.min_sdk {
        problems.push(format!(
            "`build.android.target_sdk` is {}, below `min_sdk`, which is {}",
            android.target_sdk, android.min_sdk
        ));
    }
    if let Some(code) = android.version_code
        && !(1..=MAX_VERSION_CODE).contains(&code)
    {
        problems.push(format!(
            "`build.android.version_code` is {code}; Google Play takes 1 to {MAX_VERSION_CODE}"
        ));
    }
    if let Err(why) = project_folder(&android.dir, &config.build.output) {
        problems.push(format!("`build.android.dir` is `{}`: {why}", android.dir));
    }
    if !android.icon.is_empty() && icon_file(project_dir, &android.icon).is_none() {
        problems.push(format!(
            "`build.android.icon` is `{}`, which is no file in the project or in `public/`",
            android.icon
        ));
    }
    if !android.icon_background.is_empty()
        && android_color(&android.icon_background, &HashMap::new()).is_none()
    {
        problems.push(format!(
            "`build.android.icon_background` is `{}`; a colour is `#RRGGBB`, `#RGB`, \
             `#RRGGBBAA` or `rgb(…)`",
            android.icon_background
        ));
    }
    problems
}

/// The web build, fitted to living inside an app.
///
/// What a host does for a site — a sub-path, a service worker, `.gz` twins,
/// a sitemap — means nothing to pages that are already on the phone, so it
/// is left out, with a note where the config asked for it.
pub fn prepare(config: &mut ProjectConfig) -> Vec<String> {
    let mut notes = Vec::new();
    if !config.build.base_path.is_empty() {
        notes.push(format!(
            "`build.base_path` (`{}`) is for a site on a sub-path; the app serves its pages from \
             its own root, so it is left out",
            config.build.base_path
        ));
        config.build.base_path.clear();
    }
    if config.offline.take().is_some() {
        notes.push(
            "`offline` is left out: the app's pages are on the phone, so it works with no \
             network already"
                .to_string(),
        );
    }
    config.build.compress = false;
    config.meta.sitemap = false;
    let id = config.build.android.application_id.trim();
    if id == "com.example" || id.starts_with("com.example.") {
        notes.push(
            "Google Play refuses an id under `com.example`: give \
             `build.android.application_id` one of your own before you publish"
                .to_string(),
        );
    }
    notes
}

/// Write the Android project around the web build in `output_dir`.
pub fn write_project(
    project_dir: &Path,
    output_dir: &Path,
    config: &ProjectConfig,
    program: &Program,
) -> Result<Written> {
    let android = &config.build.android;
    let root = project_dir.join(&android.dir);
    let id = android.application_id.trim();
    let created = !root.join("app/build.gradle.kts").exists();
    let mut notes = Vec::new();

    // The author's half: written when missing, and never again.
    let fill = |text: &str| {
        text.replace("@@PACKAGE@@", id)
            .replace("@@PROJECT@@", &gradle_project_name(&config.name))
            .replace("@@AGP@@", AGP_VERSION)
            .replace("@@GRADLE@@", GRADLE_VERSION)
    };
    for (path, text) in [
        ("settings.gradle.kts", SETTINGS),
        ("build.gradle.kts", ROOT_BUILD),
        ("gradle.properties", GRADLE_PROPERTIES),
        ("gradle/wrapper/gradle-wrapper.properties", WRAPPER),
        (".gitignore", GITIGNORE),
        ("app/build.gradle.kts", APP_BUILD),
        ("app/src/main/AndroidManifest.xml", MANIFEST),
    ] {
        write_once(&root.join(path), &fill(text))?;
    }
    // The activity lives in the package the project was made with. A later
    // change of id moves the app's identity, not its code: the namespace in
    // `app/build.gradle.kts` still names this package.
    let sources = root.join("app/src/main/java");
    if !sources.exists() {
        let file = sources.join(id.replace('.', "/")).join("MainActivity.java");
        write_once(&file, &fill(ACTIVITY))?;
    }

    // The project's half: written on every build, where it changed.
    let version_code = android
        .version_code
        .unwrap_or_else(|| version_code(&config.version));
    write_if_changed(
        &root.join("app/webfluent.properties"),
        properties(config, version_code).as_bytes(),
    )?;

    let tokens = crate::themes::resolve_tokens(program, &config.theme)?;
    let light = android_color_token("color-background", &tokens).unwrap_or(WHITE);
    let dark = crate::themes::resolve_dark_tokens(program, &config.theme)?.map(|overrides| {
        let mut merged = tokens.clone();
        merged.extend(overrides);
        android_color_token("color-background", &merged).unwrap_or(light)
    });
    let res = root.join("app/src/main/res");
    let icon = launcher_icon(project_dir, config, &tokens, &mut notes)?;
    let app_name = if android.app_name.trim().is_empty() {
        config.name.as_str()
    } else {
        android.app_name.trim()
    };
    write_if_changed(
        &res.join("values/webfluent.xml"),
        values(app_name, light, icon.background).as_bytes(),
    )?;
    write_if_changed(
        &res.join("values-v27/webfluent.xml"),
        theme_file(None, "android:Theme.Material.Light.NoActionBar", true).as_bytes(),
    )?;
    // Night mode is Android 10's; a project with no dark theme stays light
    // in it, as its pages do.
    let night = res.join("values-night-v29/webfluent.xml");
    match dark {
        Some(color) => {
            write_if_changed(
                &night,
                theme_file(Some(color), "android:Theme.Material.NoActionBar", true).as_bytes(),
            )?;
        }
        None => remove_generated(&night)?,
    }
    write_icons(&res, &icon)?;

    sync(output_dir, &root.join("app/src/main/assets/www"))?;

    Ok(Written {
        dir: normal(&android.dir)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| android.dir.clone()),
        created,
        summary: format!("{id} {} (version code {version_code})", config.version),
        notes,
    })
}

// ─── The app's identity ─────────────────────────────────────────────

/// Why `id` cannot be an application id, if it cannot.
///
/// Android asks for two or more parts separated by dots, each starting with
/// a letter; the id is also the activity's Java package, so no part may be
/// a Java keyword.
pub fn check_application_id(id: &str) -> std::result::Result<(), String> {
    const KEYWORDS: &[&str] = &[
        "abstract",
        "assert",
        "boolean",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "class",
        "const",
        "continue",
        "default",
        "do",
        "double",
        "else",
        "enum",
        "extends",
        "false",
        "final",
        "finally",
        "float",
        "for",
        "goto",
        "if",
        "implements",
        "import",
        "instanceof",
        "int",
        "interface",
        "long",
        "native",
        "new",
        "null",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "short",
        "static",
        "strictfp",
        "super",
        "switch",
        "synchronized",
        "this",
        "throw",
        "throws",
        "transient",
        "true",
        "try",
        "void",
        "volatile",
        "while",
    ];
    let parts: Vec<&str> = id.split('.').collect();
    if parts.len() < 2 {
        return Err(
            "an id is two or more parts separated by dots, like `com.yourname.ledger`".into(),
        );
    }
    for part in parts {
        let mut chars = part.chars();
        let starts_with_letter = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
        if !starts_with_letter || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(format!(
                "`{part}` is not a part an id can have: each starts with a letter and holds \
                 only letters, digits and `_`"
            ));
        }
        if KEYWORDS.contains(&part) {
            return Err(format!(
                "`{part}` is a Java keyword, and the id is also the app's code package"
            ));
        }
    }
    Ok(())
}

/// The version code a version stands for: `1.4.2` is `1004002`.
///
/// A later version is a larger number, which is all Google Play asks of it,
/// so raising `version` is the one step a release takes. A version that is
/// not numbers is `1`, and `version_code` says what it should be.
pub fn version_code(version: &str) -> u32 {
    let core = version
        .trim()
        .trim_start_matches(['v', 'V'])
        .split(['-', '+'])
        .next()
        .unwrap_or("");
    let parts: Option<Vec<u32>> = core.split('.').map(|p| p.parse().ok()).collect();
    let Some(parts) = parts.filter(|p| (1..=3).contains(&p.len())) else {
        return 1;
    };
    let at = |i: usize| parts.get(i).copied().unwrap_or(0);
    let code = at(0).min(2099) * 1_000_000 + at(1).min(999) * 1_000 + at(2).min(999);
    code.max(1)
}

/// Why `dir` cannot hold the Android project, if it cannot.
fn project_folder(dir: &str, output: &str) -> std::result::Result<(), String> {
    let folder = normal(dir).ok_or("it must be a folder inside the project")?;
    if folder.as_os_str().is_empty() {
        return Err("the Android project needs a folder of its own".into());
    }
    // The dev server rebuilds when anything under these changes, and the
    // build writes the Android project: it would rebuild forever.
    if folder.starts_with("src") || folder.starts_with("public") {
        return Err(
            "the dev server rebuilds on every change under `src/` and `public/`, and a build \
             writes the Android project, so it would rebuild forever"
                .into(),
        );
    }
    if let Some(output) = normal(output)
        && (folder.starts_with(&output) || output.starts_with(&folder))
    {
        return Err(format!(
            "the web build goes to `{}`, and the two must be separate folders, since the one is \
             copied into the other",
            output.display()
        ));
    }
    Ok(())
}

/// `path` within the project, without `.` and with no way out through `..`.
fn normal(path: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in Path::new(path).components() {
        match part {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

/// The Gradle project's name: Gradle refuses a space, a slash and a few
/// other characters in one.
fn gradle_project_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect();
    let clean = clean.trim_matches('-');
    if clean.is_empty() {
        "app".into()
    } else {
        clean.into()
    }
}

/// `app/webfluent.properties`: what `app/build.gradle.kts` reads.
fn properties(config: &ProjectConfig, version_code: u32) -> String {
    let android = &config.build.android;
    format!(
        "# Written by `wf build` on every build, from `build.android` in\n\
         # webfluent.app.json. Change them there.\n\
         applicationId={}\n\
         versionCode={version_code}\n\
         versionName={}\n\
         minSdk={}\n\
         targetSdk={}\n\
         compileSdk={}\n",
        android.application_id.trim(),
        property_value(&config.version),
        android.min_sdk,
        android.target_sdk,
        android.target_sdk.max(COMPILE_SDK),
    )
}

/// A value as a `.properties` file holds it: read as ISO-8859-1, so
/// anything beyond ASCII is written as its `\u` escape.
fn property_value(value: &str) -> String {
    let mut out = String::new();
    for unit in value.encode_utf16() {
        match unit {
            0x5C => out.push_str("\\\\"),
            0x0A => out.push_str("\\n"),
            0x20..=0x7E => out.push(unit as u8 as char),
            _ => out.push_str(&format!("\\u{unit:04X}")),
        }
    }
    out
}

// ─── Resources ──────────────────────────────────────────────────────

/// A colour as Android writes one: `#AARRGGBB`.
type Argb = [u8; 4];

const WHITE: Argb = [0xFF, 0xFF, 0xFF, 0xFF];

fn argb_hex([a, r, g, b]: Argb) -> String {
    format!("#{a:02X}{r:02X}{g:02X}{b:02X}")
}

/// Whether the system bars over `color` want dark icons: whichever of black
/// and white contrasts more with it.
fn wants_dark_icons([_, r, g, b]: Argb) -> bool {
    let channel = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
    (luminance + 0.05) / 0.05 > 1.05 / (luminance + 0.05)
}

/// `res/values/webfluent.xml`: the app's name, its colours, and the theme
/// the window is drawn with until the first page paints.
fn values(app_name: &str, background: Argb, icon_background: Argb) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <!-- Written by `wf build` on every build, from webfluent.app.json and the\n     \
         theme. Change them there. -->\n\
         <resources>\n    \
         <string name=\"app_name\">{}</string>\n    \
         <color name=\"wf_icon_background\">{}</color>\n\
         {}\
         </resources>\n",
        android_string(app_name),
        argb_hex(icon_background),
        theme_body(
            Some(background),
            "android:Theme.Material.Light.NoActionBar",
            false
        ),
    )
}

/// A `webfluent.xml` that only restyles the window: the navigation bar's
/// colours are Android 8.1's, and night mode is Android 10's.
fn theme_file(background: Option<Argb>, parent: &str, navigation_bar: bool) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <!-- Written by `wf build` on every build, from the theme. -->\n\
         <resources>\n\
         {}\
         </resources>\n",
        theme_body(background, parent, navigation_bar)
    )
}

/// The colours and the window style: the page's background behind the
/// system bars, with icons that can be read on it.
fn theme_body(background: Option<Argb>, parent: &str, navigation_bar: bool) -> String {
    let mut out = String::new();
    if let Some(color) = background {
        out.push_str(&format!(
            "    <color name=\"wf_background\">{}</color>\n    \
             <bool name=\"wf_light_bars\">{}</bool>\n",
            argb_hex(color),
            wants_dark_icons(color)
        ));
    }
    out.push_str(&format!(
        "    <style name=\"Theme.App\" parent=\"{parent}\">\n"
    ));
    let mut items = vec![
        ("android:windowBackground", "@color/wf_background"),
        ("android:statusBarColor", "@color/wf_background"),
        ("android:windowLightStatusBar", "@bool/wf_light_bars"),
    ];
    if navigation_bar {
        items.push(("android:navigationBarColor", "@color/wf_background"));
        items.push(("android:windowLightNavigationBar", "@bool/wf_light_bars"));
    }
    for (name, value) in items {
        out.push_str(&format!("        <item name=\"{name}\">{value}</item>\n"));
    }
    out.push_str("    </style>\n");
    out
}

/// Text as a string resource holds it: escaped for XML, and for the
/// resource compiler, which reads `\`, quotes, and a leading `@` or `?`.
fn android_string(text: &str) -> String {
    let mut out = String::new();
    for (i, c) in text.chars().enumerate() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '@' | '?' if i == 0 => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// A token's colour, as Android writes one.
fn android_color_token(name: &str, tokens: &HashMap<String, String>) -> Option<Argb> {
    android_color(tokens.get(name)?, tokens)
}

/// A CSS colour as Android writes one — `#RGB`, `#RGBA`, `#RRGGBB`,
/// `#RRGGBBAA`, `rgb()` and `rgba()`, `white` and `black`, and a
/// `var(--token)` that leads to one of those.
fn android_color(value: &str, tokens: &HashMap<String, String>) -> Option<Argb> {
    color_within(value, tokens, 0)
}

fn color_within(value: &str, tokens: &HashMap<String, String>, depth: usize) -> Option<Argb> {
    let value = value.trim();
    if let Some(inner) = value.strip_prefix("var(").and_then(|v| v.strip_suffix(')')) {
        if depth > 8 {
            return None;
        }
        let (name, fallback) = match inner.split_once(',') {
            Some((name, fallback)) => (name.trim(), Some(fallback)),
            None => (inner.trim(), None),
        };
        let named = tokens
            .get(name.trim_start_matches("--"))
            .and_then(|v| color_within(v, tokens, depth + 1));
        return named.or_else(|| fallback.and_then(|f| color_within(f, tokens, depth + 1)));
    }
    if let Some(hex) = value.strip_prefix('#') {
        let digit = |i: usize| u8::from_str_radix(hex.get(i..i + 1)?, 16).ok();
        let pair = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
        return match hex.len() {
            3 | 4 => {
                let alpha = if hex.len() == 4 { digit(3)? * 17 } else { 255 };
                Some([alpha, digit(0)? * 17, digit(1)? * 17, digit(2)? * 17])
            }
            6 | 8 => {
                let alpha = if hex.len() == 8 { pair(6)? } else { 255 };
                Some([alpha, pair(0)?, pair(2)?, pair(4)?])
            }
            _ => None,
        };
    }
    let lower = value.to_ascii_lowercase();
    if let Some(inner) = lower
        .strip_prefix("rgba(")
        .or_else(|| lower.strip_prefix("rgb("))
        .and_then(|v| v.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner
            .split([',', ' ', '/'])
            .filter(|p| !p.is_empty())
            .collect();
        if !(3..=4).contains(&parts.len()) {
            return None;
        }
        let channel = |p: &str| -> Option<u8> {
            let n = match p.strip_suffix('%') {
                Some(pct) => pct.parse::<f64>().ok()? * 2.55,
                None => p.parse::<f64>().ok()?,
            };
            Some(n.round().clamp(0.0, 255.0) as u8)
        };
        let alpha = match parts.get(3) {
            Some(p) => {
                let a = match p.strip_suffix('%') {
                    Some(pct) => pct.parse::<f64>().ok()? / 100.0,
                    None => p.parse::<f64>().ok()?,
                };
                (a.clamp(0.0, 1.0) * 255.0).round() as u8
            }
            None => 255,
        };
        return Some([
            alpha,
            channel(parts[0])?,
            channel(parts[1])?,
            channel(parts[2])?,
        ]);
    }
    match lower.as_str() {
        "white" => Some(WHITE),
        "black" => Some([0xFF, 0, 0, 0]),
        _ => None,
    }
}

// ─── The launcher icon ──────────────────────────────────────────────

/// The picture the launcher icons are made from, and what fills an
/// adaptive icon's shape behind it.
struct Icon {
    image: DynamicImage,
    background: Argb,
}

/// Where the icon a config names is: in the project, or in `public/`
/// under the site path a page would name it by.
fn icon_file(project_dir: &Path, path: &str) -> Option<PathBuf> {
    let relative = path.trim_start_matches('/');
    [
        project_dir.join(relative),
        project_dir.join("public").join(relative),
    ]
    .into_iter()
    .find(|p| p.is_file())
}

/// The icon: `build.android.icon`, else `meta.touch_icon`, else a plain one
/// in the theme's primary colour.
fn launcher_icon(
    project_dir: &Path,
    config: &ProjectConfig,
    tokens: &HashMap<String, String>,
    notes: &mut Vec<String>,
) -> Result<Icon> {
    let android = &config.build.android;
    let named = if !android.icon.is_empty() {
        icon_file(project_dir, &android.icon).map(|p| (p, "build.android.icon"))
    } else if !config.meta.touch_icon.is_empty() {
        icon_file(project_dir, &config.meta.touch_icon).map(|p| (p, "meta.touch_icon"))
    } else {
        None
    };
    let image = match named {
        Some((path, key)) => {
            let bytes = fs::read(&path)?;
            let image = image::load_from_memory(&bytes).map_err(|e| {
                WebFluentError::IoError(format!(
                    "`{key}` ({}) could not be read as an image: {e}",
                    path.display()
                ))
            })?;
            // The largest launcher icon holds the picture 264 pixels across.
            let (width, height) = image.dimensions();
            if width.min(height) < 264 {
                notes.push(format!(
                    "the launcher icon is made from `{key}`, which is {width}×{height}; a \
                     512×512 PNG in `build.android.icon` keeps it sharp on every screen"
                ));
            }
            image
        }
        None => {
            notes.push(
                "no `build.android.icon`: the launcher shows a plain icon in the theme's \
                 primary colour until you name a 512×512 PNG"
                    .to_string(),
            );
            let primary = android_color_token("color-primary", tokens).unwrap_or(WHITE);
            DynamicImage::ImageRgba8(placeholder_icon(512, primary))
        }
    };

    let background = if android.icon_background.is_empty() {
        // An icon drawn on a solid colour runs on into the shape; one drawn
        // on nothing sits on the page's own background.
        let corner = image.get_pixel(0, 0).0;
        if corner[3] == 255 {
            [255, corner[0], corner[1], corner[2]]
        } else {
            android_color_token("color-background", tokens).unwrap_or(WHITE)
        }
    } else {
        android_color(&android.icon_background, tokens).unwrap_or(WHITE)
    };
    Ok(Icon { image, background })
}

/// A plain icon, for a project that names none: its primary colour, and a
/// window drawn on it in white.
pub fn placeholder_icon(size: u32, [_, r, g, b]: Argb) -> RgbaImage {
    let s = f64::from(size);
    // The window: its half-size, the rounding of its corners, the width of
    // its frame, and the depth of its title bar.
    let (half_w, half_h) = (0.30 * s, 0.24 * s);
    let (radius, stroke, bar) = (0.075 * s, 0.055 * s, 0.12 * s);
    let top = s / 2.0 - half_h;
    RgbaImage::from_fn(size, size, |x, y| {
        let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
        // The signed distance to the window's edge, negative inside it.
        let qx = (px - s / 2.0).abs() - half_w + radius;
        let qy = (py - s / 2.0).abs() - half_h + radius;
        let outside = qx.max(0.0).hypot(qy.max(0.0));
        let d = outside + qx.max(qy).min(0.0) - radius;
        let frame = (0.5 - ((d + stroke / 2.0).abs() - stroke / 2.0)).clamp(0.0, 1.0);
        let title = (0.5 - d).clamp(0.0, 1.0) * (0.5 - (py - (top + bar))).clamp(0.0, 1.0);
        let white = frame.max(title);
        let mix = |c: u8| (f64::from(c) + (255.0 - f64::from(c)) * white).round() as u8;
        Rgba([mix(r), mix(g), mix(b), 255])
    })
}

/// `image`, scaled to fit a `size` square and centred on a clear one.
fn fit_square(image: &DynamicImage, size: u32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let scale = f64::from(size) / f64::from(width.max(height));
    let w = ((f64::from(width) * scale).round() as u32).clamp(1, size);
    let h = ((f64::from(height) * scale).round() as u32).clamp(1, size);
    let scaled = image.resize_exact(w, h, FilterType::Lanczos3).to_rgba8();
    let mut square = RgbaImage::new(size, size);
    imageops::overlay(
        &mut square,
        &scaled,
        i64::from((size - w) / 2),
        i64::from((size - h) / 2),
    );
    square
}

/// The icon at every density: the picture itself for a launcher that takes
/// it as it is, and an adaptive icon — the picture on the colour behind it,
/// in a shape the launcher chooses — for Android 8 and later.
fn write_icons(res: &Path, icon: &Icon) -> Result<()> {
    for (density, legacy, layer) in DENSITIES {
        let dir = res.join(format!("mipmap-{density}"));
        write_if_changed(
            &dir.join("ic_launcher.png"),
            &png(fit_square(&icon.image, legacy))?,
        )?;
        // A launcher may cut anything outside the middle 66 of the layer's
        // 108 units, so the picture fits there.
        let inner = layer * 66 / 108;
        let mut foreground = RgbaImage::new(layer, layer);
        let offset = i64::from((layer - inner) / 2);
        imageops::overlay(
            &mut foreground,
            &fit_square(&icon.image, inner),
            offset,
            offset,
        );
        write_if_changed(&dir.join("ic_launcher_foreground.png"), &png(foreground)?)?;
    }
    write_if_changed(
        &res.join("mipmap-anydpi-v26/ic_launcher.xml"),
        b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
          <!-- Written by `wf build` on every build, from `build.android.icon`. -->\n\
          <adaptive-icon xmlns:android=\"http://schemas.android.com/apk/res/android\">\n    \
          <background android:drawable=\"@color/wf_icon_background\" />\n    \
          <foreground android:drawable=\"@mipmap/ic_launcher_foreground\" />\n\
          </adaptive-icon>\n",
    )?;
    Ok(())
}

fn png(image: RgbaImage) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    DynamicImage::ImageRgba8(image)
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|e| {
            WebFluentError::IoError(format!("a launcher icon could not be written: {e}"))
        })?;
    Ok(out)
}

// ─── Files ──────────────────────────────────────────────────────────

/// Write `text` to `path` if nothing is there: a file the author owns.
fn write_once(path: &Path, text: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)?;
    Ok(())
}

/// Write `bytes` to `path` unless it already holds them, so a build that
/// changed nothing leaves Gradle nothing to redo.
fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<()> {
    if fs::read(path).is_ok_and(|held| held == bytes) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

/// Remove a file the build wrote and no longer writes.
fn remove_generated(path: &Path) -> Result<()> {
    if path.is_file() {
        fs::remove_file(path)?;
        if let Some(parent) = path.parent() {
            // Only if nothing else is there: the folder may hold the author's.
            let _ = fs::remove_dir(parent);
        }
    }
    Ok(())
}

/// What a host serves beside a site, and an app has no use for.
fn host_only(relative: &Path) -> bool {
    if relative.extension().is_some_and(|e| e == "gz") {
        return true;
    }
    relative.parent() == Some(Path::new(""))
        && relative
            .to_str()
            .is_some_and(|name| matches!(name, "sw.js" | "_headers" | "sitemap.xml" | "robots.txt"))
}

/// Mirror the web build into the app: what changed is written, and what
/// the build no longer writes is removed.
fn sync(from: &Path, to: &Path) -> Result<()> {
    let mut kept = BTreeSet::new();
    let mut walk = vec![from.to_path_buf()];
    while let Some(dir) = walk.pop() {
        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk.push(path);
                continue;
            }
            let Ok(relative) = path.strip_prefix(from) else {
                continue;
            };
            if host_only(relative) {
                continue;
            }
            write_if_changed(&to.join(relative), &fs::read(&path)?)?;
            kept.insert(relative.to_path_buf());
        }
    }
    if to.is_dir() {
        prune(to, to, &kept)?;
    }
    Ok(())
}

/// Remove from `dir` every file `kept` does not name, and the folders that
/// leaves empty.
fn prune(root: &Path, dir: &Path, kept: &BTreeSet<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            prune(root, &path, kept)?;
            if fs::read_dir(&path)?.next().is_none() {
                fs::remove_dir(&path)?;
            }
        } else if path
            .strip_prefix(root)
            .is_ok_and(|relative| !kept.contains(relative))
        {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_id_is_dotted_parts_that_are_not_keywords() {
        assert!(check_application_id("com.yourname.ledger").is_ok());
        assert!(check_application_id("app.ledger_2").is_ok());
        assert!(check_application_id("ledger").is_err());
        assert!(check_application_id("com.2ledger").is_err());
        assert!(check_application_id("com.your-name.ledger").is_err());
        assert!(check_application_id("com..ledger").is_err());
        let keyword = check_application_id("com.example.new").unwrap_err();
        assert!(keyword.contains("keyword"), "{keyword}");
    }

    #[test]
    fn a_version_is_a_code_that_grows_with_it() {
        assert_eq!(version_code("1.4.2"), 1_004_002);
        assert_eq!(version_code("v2.0"), 2_000_000);
        assert_eq!(version_code("3"), 3_000_000);
        assert_eq!(version_code("1.10.0-beta.1"), 1_010_000);
        assert!(version_code("1.10.0") > version_code("1.9.99"));
        assert_eq!(version_code("0.0.0"), 1);
        assert_eq!(version_code("next"), 1);
        assert_eq!(version_code("1.2.3.4"), 1);
    }

    #[test]
    fn the_project_folder_stays_out_of_the_sources_and_the_output() {
        assert!(project_folder("android", "./build").is_ok());
        assert!(project_folder("./apps/android", "build").is_ok());
        assert!(project_folder("src/android", "build").is_err());
        assert!(project_folder("public", "build").is_err());
        assert!(project_folder("build/android", "./build").is_err());
        assert!(project_folder("android", "android/web").is_err());
        assert!(project_folder("../android", "build").is_err());
        assert!(project_folder(".", "build").is_err());
    }

    #[test]
    fn colours_come_out_as_android_writes_them() {
        let tokens: HashMap<String, String> = [
            ("color-background".to_string(), "var(--surface)".to_string()),
            ("surface".to_string(), "#0B1220".to_string()),
        ]
        .into();
        assert_eq!(android_color("#fff", &tokens), Some(WHITE));
        assert_eq!(
            android_color("#3B82F680", &tokens),
            Some([0x80, 0x3B, 0x82, 0xF6])
        );
        assert_eq!(
            android_color("rgb(59, 130, 246)", &tokens),
            Some([0xFF, 59, 130, 246])
        );
        assert_eq!(
            android_color("rgba(0 0 0 / 50%)", &tokens),
            Some([128, 0, 0, 0])
        );
        assert_eq!(
            android_color_token("color-background", &tokens),
            Some([0xFF, 0x0B, 0x12, 0x20])
        );
        assert_eq!(android_color("hsl(0 0% 0%)", &tokens), None);
        assert_eq!(argb_hex([0xFF, 0x0B, 0x12, 0x20]), "#FF0B1220");
    }

    #[test]
    fn bars_over_a_light_page_get_dark_icons() {
        assert!(wants_dark_icons(WHITE));
        assert!(wants_dark_icons([0xFF, 0xF5, 0xF5, 0xF4]));
        assert!(!wants_dark_icons([0xFF, 0x0B, 0x12, 0x20]));
        assert!(!wants_dark_icons([0xFF, 0x1E, 0x3A, 0x8A]));
        // A mid blue reads better with black on it than with white.
        assert!(wants_dark_icons([0xFF, 0x3B, 0x82, 0xF6]));
    }

    #[test]
    fn a_name_is_escaped_for_the_resource_compiler() {
        assert_eq!(
            android_string("Bob's <Shop> & \"Co\""),
            "Bob\\'s &lt;Shop&gt; &amp; \\\"Co\\\""
        );
        assert_eq!(android_string("@home"), "\\@home");
        assert_eq!(android_string("?"), "\\?");
        assert_eq!(android_string("Q&A?"), "Q&amp;A?");
    }

    #[test]
    fn a_properties_value_is_ascii() {
        assert_eq!(property_value("1.0.0"), "1.0.0");
        assert_eq!(property_value("2.0 é"), "2.0 \\u00E9");
        assert_eq!(property_value("a\\b"), "a\\\\b");
    }

    #[test]
    fn the_gradle_project_name_is_one_gradle_takes() {
        assert_eq!(gradle_project_name("my-app"), "my-app");
        assert_eq!(gradle_project_name("My App: 2"), "My-App--2");
        assert_eq!(gradle_project_name("???"), "app");
    }

    #[test]
    fn the_placeholder_is_the_colour_with_a_white_window_on_it() {
        let icon = placeholder_icon(512, [0xFF, 0x3B, 0x82, 0xF6]);
        assert_eq!(icon.get_pixel(0, 0).0, [0x3B, 0x82, 0xF6, 255]);
        assert_eq!(icon.get_pixel(256, 256).0, [0x3B, 0x82, 0xF6, 255]);
        // The title bar, just inside the top of the window.
        assert_eq!(icon.get_pixel(256, 256 - 115).0, [255, 255, 255, 255]);
        // The frame, on the window's left edge.
        assert_eq!(icon.get_pixel(256 - 150, 256).0, [255, 255, 255, 255]);
    }

    #[test]
    fn a_host_s_files_stay_out_of_the_app() {
        assert!(host_only(Path::new("app.js.gz")));
        assert!(host_only(Path::new("sw.js")));
        assert!(host_only(Path::new("_headers")));
        assert!(host_only(Path::new("robots.txt")));
        assert!(!host_only(Path::new("app.js")));
        assert!(!host_only(Path::new("docs/robots.txt")));
    }
}
