// A digest of the compiler's own sources, as `WF_SOURCE_FINGERPRINT`: the
// build cache keys what it keeps by it (src/build/cache.rs), so a page
// rendered by one compiler is never reused by another, even between two
// builds of the same version.

use std::path::Path;

fn main() {
    let mut files = Vec::new();
    walk(Path::new("src"), &mut files);
    files.sort();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for b in bytes {
            hash ^= u64::from(*b);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for file in &files {
        eat(file.to_string_lossy().as_bytes());
        eat(&std::fs::read(file).unwrap_or_default());
    }
    println!("cargo:rustc-env=WF_SOURCE_FINGERPRINT={hash:016x}");
    println!("cargo:rerun-if-changed=src");
    explanations();
    capabilities();
}

// What this build of the compiler can do. A native build does everything; a
// WebAssembly build (which `crates/wf-wasm` makes) carries each of these
// only when its feature asks for it, since each costs a download:
//
//   wf_webp            lossy WebP through libwebp       feature `webp`
//   wf_pdf             the PDF and slide-deck renderer  feature `pdf`
//   wf_bundled_fonts   the Liberation faces, built in   feature `bundled-fonts`
fn capabilities() {
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32");
    for (cfg, feature) in [
        ("wf_webp", "WEBP"),
        ("wf_pdf", "PDF"),
        ("wf_bundled_fonts", "BUNDLED_FONTS"),
    ] {
        println!("cargo::rustc-check-cfg=cfg({cfg})");
        if !wasm || std::env::var_os(format!("CARGO_FEATURE_{feature}")).is_some() {
            println!("cargo::rustc-cfg={cfg}");
        }
    }
}

// Each code's explanation, `src/diagnostics/explain/T05.md`, for
// `wf explain` and the editor (`diagnostics::codes::guide_section`).
fn explanations() {
    let dir = Path::new("src/diagnostics/explain");
    println!("cargo:rerun-if-changed=src/diagnostics/explain");
    let mut paths = Vec::new();
    walk(dir, &mut paths);
    let mut codes: Vec<String> = paths
        .iter()
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .filter(|code| {
            code.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && code[1..].chars().all(|c| c.is_ascii_digit())
        })
        .collect();
    codes.sort();
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let mut pages = String::from(
        "/// Every code's explanation under `src/diagnostics/explain/`, by code.\npub(crate) static ERROR_PAGES: &[(&str, &str)] = &[\n",
    );
    for code in &codes {
        let path = Path::new(&root).join(dir).join(format!("{code}.md"));
        println!("cargo:rerun-if-changed={}", path.display());
        pages.push_str(&format!(
            "    ({code:?}, include_str!({:?})),\n",
            path.display().to_string()
        ));
    }
    pages.push_str("];\n");
    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    std::fs::write(Path::new(&out_dir).join("error_pages.rs"), pages)
        .expect("write the explanations' list");
}

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}
