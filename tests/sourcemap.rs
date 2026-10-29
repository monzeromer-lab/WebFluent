//! `build.sourcemap`: each script the build writes has a map beside it that
//! reads a place in the running code back to the `.wf` line it came from,
//! and the script is otherwise what it is without one.

use std::path::{Path, PathBuf};

use webfluent::codegen::sourcemap::SourceMap;

fn project(name: &str, sourcemap: bool, minify: bool, split: bool) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-sourcemap-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/components")).unwrap();
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        format!(
            r#"{{ "name": "maps", "build": {{ "output": "build", "sourcemap": {sourcemap}, "minify": {minify}, "split": {split}, "compress": false }} }}"#
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("src/components/row.wf"),
        "// A note.\n\ncomponent NoteRow(label: String) {\n    Stack {\n        Text(\"row says {label}\")\n    }\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/pages/home.wf"),
        "page Home(path: \"/\", title: \"Home\", description: \"The home page.\") {\n    state n = 0\n    NoteRow(label: \"one\")\n    Button(\"Add\") {\n        on click { n = n + 1 }\n    }\n    Text(\"home counts {n}\")\n    Heading(\"Home\").h1\n}\n",
    )
    .unwrap();
    let options = webfluent::build::BuildOptions::new(&dir, &webfluent::vfs::FsVfs);
    webfluent::build::build(&options).expect("it builds");
    dir
}

/// Where `needle` is in `js`: its line and column, both from 1, the column
/// in UTF-16 units, as a stack trace gives them.
fn place(js: &str, needle: &str) -> (u32, u32) {
    let at = js
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} is not in the script"));
    let before = &js[..at];
    let line = before.matches('\n').count() as u32 + 1;
    let col = before
        .rsplit('\n')
        .next()
        .unwrap_or("")
        .encode_utf16()
        .count() as u32
        + 1;
    (line, col)
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join("build").join(file)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

fn origin(dir: &Path, file: &str, needle: &str) -> Option<String> {
    let js = read(dir, file);
    let map = SourceMap::parse(&read(dir, &format!("{file}.map"))).expect("a source map");
    let (line, col) = place(&js, needle);
    map.lookup(line, col)
        .map(|o| format!("{}:{}", o.source, o.line))
}

#[test]
fn a_place_in_the_bundle_or_a_page_s_chunk_is_read_back_to_its_wf_line() {
    for minify in [true, false] {
        let dir = project(&format!("on-{minify}"), true, minify, true);
        // A component's text, in the bundle; a page's, in its chunk.
        assert_eq!(
            origin(&dir, "app.js", "row says").as_deref(),
            Some("src/components/row.wf:5"),
            "minify {minify}"
        );
        assert_eq!(
            origin(&dir, "pages/Home.js", "home counts").as_deref(),
            Some("src/pages/home.wf:7"),
            "minify {minify}"
        );
        // A handler's body is its own line.
        let js = read(&dir, "pages/Home.js");
        let handler = "n.set(";
        assert!(js.contains(handler), "{js}");
        assert_eq!(
            origin(&dir, "pages/Home.js", handler).as_deref(),
            Some("src/pages/home.wf:5"),
            "minify {minify}"
        );
        // Each names its map, and none keeps a marker.
        assert!(read(&dir, "app.js").ends_with("//# sourceMappingURL=app.js.map\n"));
        assert!(read(&dir, "pages/Home.js").ends_with("//# sourceMappingURL=Home.js.map\n"));
        assert!(!read(&dir, "app.js").contains('\u{1}'));
        assert!(!read(&dir, "pages/Home.js").contains('\u{1}'));
        // The sources' text is in the map, for a browser's tools.
        let map: serde_json::Value = serde_json::from_str(&read(&dir, "app.js.map")).unwrap();
        let named: Vec<&str> = map["sources"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|s| s.as_str())
            .collect();
        let row = named
            .iter()
            .position(|s| *s == "src/components/row.wf")
            .expect("the component's file");
        assert!(
            map["sourcesContent"][row]
                .as_str()
                .is_some_and(|t| t.contains("component NoteRow")),
            "{map}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn one_bundle_is_mapped_too() {
    let dir = project("whole", true, true, false);
    assert_eq!(
        origin(&dir, "app.js", "home counts").as_deref(),
        Some("src/pages/home.wf:7")
    );
    assert_eq!(
        origin(&dir, "app.js", "row says").as_deref(),
        Some("src/components/row.wf:5")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn without_a_map_the_scripts_are_the_same_but_for_the_line_that_names_it() {
    for minify in [true, false] {
        let on = project(&format!("same-on-{minify}"), true, minify, true);
        let off = project(&format!("same-off-{minify}"), false, minify, true);
        for file in ["app.js", "pages/Home.js"] {
            let with = read(&on, file);
            let base = file.rsplit('/').next().unwrap();
            let without = with
                .strip_suffix(&format!("\n//# sourceMappingURL={base}.map\n"))
                .expect("the line that names the map");
            assert_eq!(without, read(&off, file), "{file}, minify {minify}");
        }
        assert!(!off.join("build/app.js.map").exists());
        let _ = std::fs::remove_dir_all(&on);
        let _ = std::fs::remove_dir_all(&off);
    }
}

#[test]
fn an_editor_s_preview_asks_for_maps_whatever_the_config_says() {
    let dir = project("asked", false, true, true);
    assert!(!dir.join("build/app.js.map").exists());
    let options = webfluent::build::BuildOptions {
        sourcemap: true,
        ..webfluent::build::BuildOptions::new(&dir, &webfluent::vfs::FsVfs)
    };
    webfluent::build::build(&options).expect("it builds");
    assert_eq!(
        origin(&dir, "pages/Home.js", "home counts").as_deref(),
        Some("src/pages/home.wf:7")
    );
    let _ = std::fs::remove_dir_all(&dir);
}
