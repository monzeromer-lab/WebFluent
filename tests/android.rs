//! What `output_type: "android"` makes a build write, keep, and refuse.
//!
//! The app itself — a WebView showing the build — needs the Android SDK to
//! compile and a phone to run, so this holds what the build decides: the
//! project it writes around the web build, which half of it is the
//! author's and which is refreshed on every build, what the web build
//! leaves out inside an app, the icons and colours, and the configurations
//! it refuses.

use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str, config: &str, app: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-android-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), config).unwrap();
    std::fs::write(dir.join("src/App.wf"), app).unwrap();
    dir
}

fn build(dir: &Path) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(dir)
        .output()
        .expect("run wf");
    (
        out.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn read(dir: &Path, path: &str) -> String {
    std::fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

const PAGES: &str = r#"
app { Router }
page Home(path: "/", title: "Home", description: "d") { Heading("Home").h1 }
page About(path: "/about", title: "About", description: "d") { Heading("About").h1 }
"#;

fn config(android: &str, rest: &str) -> String {
    format!(
        r#"{{ "name": "Ledger", "version": "1.4.2",
             "build": {{ "output": "build", "output_type": "android", "android": {android} }}{rest} }}"#
    )
}

#[test]
fn a_build_writes_an_android_project_around_the_web_build() {
    let dir = scratch(
        "project",
        &config(
            r#"{ "application_id": "app.ledger.books", "app_name": "Bob's Books" }"#,
            r#", "offline": { "precache": ["/"] }, "meta": { "title": "Ledger" }"#,
        ),
        PAGES,
    );
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    assert!(
        out.contains("Android: app.ledger.books 1.4.2 (version code 1004002), in android/"),
        "{out}"
    );
    assert!(out.contains("A new Android project"), "{out}");
    assert!(out.contains("`offline` is left out"), "{out}");

    // The author's half: Gradle, the manifest, and the activity in the
    // package the id names.
    for file in [
        "android/settings.gradle.kts",
        "android/build.gradle.kts",
        "android/gradle.properties",
        "android/gradle/wrapper/gradle-wrapper.properties",
        "android/.gitignore",
        "android/app/src/main/AndroidManifest.xml",
    ] {
        assert!(dir.join(file).is_file(), "{file} was not written");
    }
    let activity = read(
        &dir,
        "android/app/src/main/java/app/ledger/books/MainActivity.java",
    );
    assert!(
        activity.starts_with("package app.ledger.books;"),
        "{activity}"
    );
    assert!(
        !activity.contains("@@"),
        "a placeholder was left in the activity"
    );
    let gradle = read(&dir, "android/app/build.gradle.kts");
    assert!(
        gradle.contains(r#"namespace = "app.ledger.books""#),
        "{gradle}"
    );
    assert!(read(&dir, "android/settings.gradle.kts").contains(r#"rootProject.name = "Ledger""#));

    // The project's half: what Gradle reads, the name and colours.
    let properties = read(&dir, "android/app/webfluent.properties");
    for line in [
        "applicationId=app.ledger.books",
        "versionCode=1004002",
        "versionName=1.4.2",
        "minSdk=24",
        "targetSdk=36",
        "compileSdk=36",
    ] {
        assert!(
            properties.contains(line),
            "{line} missing from:\n{properties}"
        );
    }
    let values = read(&dir, "android/app/src/main/res/values/webfluent.xml");
    assert!(
        values.contains(r#"<string name="app_name">Bob\'s Books</string>"#),
        "{values}"
    );
    assert!(
        values.contains(r#"<color name="wf_background">#FFFFFFFF</color>"#),
        "{values}"
    );

    // The web build, inside the app — without what only a host needs.
    let www = dir.join("android/app/src/main/assets/www");
    assert!(www.join("index.html").is_file());
    assert!(www.join("app.js").is_file());
    assert!(www.join("styles.css").is_file());
    for host_only in [
        "sw.js",
        "robots.txt",
        "sitemap.xml",
        "index.html.gz",
        "app.js.gz",
    ] {
        assert!(!www.join(host_only).exists(), "{host_only} reached the app");
        assert!(
            !dir.join("build").join(host_only).exists(),
            "{host_only} was built"
        );
    }
    // What the app shows is what the web build wrote.
    assert_eq!(
        std::fs::read(www.join("app.js")).unwrap(),
        std::fs::read(dir.join("build/app.js")).unwrap()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_author_s_files_are_kept_and_the_project_s_are_refreshed() {
    let dir = scratch(
        "rebuild",
        &config(r#"{ "application_id": "app.ledger.books" }"#, ""),
        PAGES,
    );
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");

    let activity = "android/app/src/main/java/app/ledger/books/MainActivity.java";
    let manifest = "android/app/src/main/AndroidManifest.xml";
    let edited = format!("{}\n// The author's.\n", read(&dir, activity));
    std::fs::write(dir.join(activity), &edited).unwrap();
    std::fs::write(dir.join(manifest), "<!-- the author's manifest -->\n").unwrap();
    // A stale file where the web build goes, and a release.
    std::fs::write(dir.join("android/app/src/main/assets/www/gone.js"), "old").unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        config(r#"{ "application_id": "app.ledger.books" }"#, "").replace("1.4.2", "1.5.0"),
    )
    .unwrap();

    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    assert!(!out.contains("A new Android project"), "{out}");
    assert_eq!(
        read(&dir, activity),
        edited,
        "the activity was written over"
    );
    assert_eq!(read(&dir, manifest), "<!-- the author's manifest -->\n");
    let properties = read(&dir, "android/app/webfluent.properties");
    assert!(properties.contains("versionCode=1005000"), "{properties}");
    assert!(properties.contains("versionName=1.5.0"), "{properties}");
    assert!(
        !dir.join("android/app/src/main/assets/www/gone.js").exists(),
        "a file the build no longer writes stayed in the app"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_app_needs_an_id_it_can_keep_and_a_folder_of_its_own() {
    for (android, expected) in [
        (r#"{}"#, "`build.android.application_id` is not set"),
        (r#"{ "application_id": "ledger" }"#, "two or more parts"),
        (r#"{ "application_id": "com.ledger.new" }"#, "Java keyword"),
        (
            r#"{ "application_id": "app.ledger", "dir": "src/android" }"#,
            "rebuild forever",
        ),
        (
            r#"{ "application_id": "app.ledger", "dir": "build/android" }"#,
            "separate folders",
        ),
        (
            r#"{ "application_id": "app.ledger", "min_sdk": 21 }"#,
            "`build.android.min_sdk` is 21",
        ),
        (
            r#"{ "application_id": "app.ledger", "icon": "missing.png" }"#,
            "`build.android.icon` is `missing.png`",
        ),
    ] {
        let dir = scratch("refused", &config(android, ""), PAGES);
        let (ok, out) = build(&dir);
        assert!(!ok, "{android} built:\n{out}");
        assert!(
            out.contains(expected),
            "{android}: expected `{expected}` in:\n{out}"
        );
        assert!(!dir.join("android").exists(), "{android} wrote a project");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn a_placeholder_id_is_named_before_it_reaches_google_play() {
    let dir = scratch(
        "example",
        &config(r#"{ "application_id": "com.example.ledger" }"#, ""),
        PAGES,
    );
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    assert!(
        out.contains("Google Play refuses an id under `com.example`"),
        "{out}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_theme_colours_the_window_and_a_dark_theme_gets_night_resources() {
    let themes = r#"
theme Brand { color-background: #F5F5F4  color-primary: #0F766E }
theme Night { color-background: #0B1220 }
"#;
    let dir = scratch(
        "theme",
        &config(
            r#"{ "application_id": "app.ledger" }"#,
            r#", "theme": { "name": "Brand", "dark": "Night" }"#,
        ),
        &format!("{PAGES}{themes}"),
    );
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    let res = "android/app/src/main/res";
    let values = read(&dir, &format!("{res}/values/webfluent.xml"));
    assert!(
        values.contains(r#"<color name="wf_background">#FFF5F5F4</color>"#),
        "{values}"
    );
    assert!(
        values.contains(r#"<bool name="wf_light_bars">true</bool>"#),
        "{values}"
    );
    // With no icon named, the plain one is the primary colour, and so is
    // what surrounds it.
    assert!(
        values.contains(r#"<color name="wf_icon_background">#FF0F766E</color>"#),
        "{values}"
    );
    let night = read(&dir, &format!("{res}/values-night-v29/webfluent.xml"));
    assert!(
        night.contains(r#"<color name="wf_background">#FF0B1220</color>"#),
        "{night}"
    );
    assert!(
        night.contains(r#"<bool name="wf_light_bars">false</bool>"#),
        "{night}"
    );
    assert!(
        night.contains(r#"parent="android:Theme.Material.NoActionBar""#),
        "{night}"
    );

    // Without a dark theme the app stays light at night, as its pages do.
    std::fs::write(
        dir.join("webfluent.app.json"),
        config(
            r#"{ "application_id": "app.ledger" }"#,
            r#", "theme": { "name": "Brand" }"#,
        ),
    )
    .unwrap();
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    assert!(!dir.join(format!("{res}/values-night-v29")).exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_launcher_icon_is_made_from_the_picture_the_config_names() {
    let dir = scratch(
        "icon",
        &config(
            r#"{ "application_id": "app.ledger", "icon": "public/icon.png" }"#,
            "",
        ),
        PAGES,
    );
    // Wider than it is tall, and drawn on a solid red.
    std::fs::create_dir_all(dir.join("public")).unwrap();
    image::RgbaImage::from_pixel(600, 400, image::Rgba([200, 30, 40, 255]))
        .save(dir.join("public/icon.png"))
        .unwrap();
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    assert!(!out.contains("plain icon"), "{out}");

    let res = dir.join("android/app/src/main/res");
    for (density, legacy, layer) in [("mdpi", 48, 108), ("xxxhdpi", 192, 432)] {
        let icon = res.join(format!("mipmap-{density}/ic_launcher.png"));
        assert_eq!(image::image_dimensions(&icon).unwrap(), (legacy, legacy));
        let foreground = res.join(format!("mipmap-{density}/ic_launcher_foreground.png"));
        assert_eq!(
            image::image_dimensions(&foreground).unwrap(),
            (layer, layer)
        );
    }
    // The picture runs on into the launcher's shape in its own colour.
    let values = read(&dir, "android/app/src/main/res/values/webfluent.xml");
    assert!(
        values.contains(r#"<color name="wf_icon_background">#FFC81E28</color>"#),
        "{values}"
    );
    let adaptive = read(
        &dir,
        "android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml",
    );
    assert!(
        adaptive.contains("@mipmap/ic_launcher_foreground"),
        "{adaptive}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn every_android_setting_is_one_the_build_reads() {
    let dir = scratch(
        "keys",
        &config(
            r##"{ "application_id": "app.ledger", "app_name": "Ledger", "icon": "",
                 "icon_background": "#FFFFFF", "version_code": 7, "min_sdk": 26,
                 "target_sdk": 36, "dir": "native" }"##,
            "",
        ),
        PAGES,
    );
    let (ok, out) = build(&dir);
    assert!(ok, "{out}");
    assert!(!out.contains("is not a setting"), "{out}");
    let properties = read(&dir, "native/app/webfluent.properties");
    assert!(properties.contains("versionCode=7"), "{properties}");
    assert!(properties.contains("minSdk=26"), "{properties}");
    let _ = std::fs::remove_dir_all(&dir);
}
