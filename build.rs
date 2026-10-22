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
    guide();
}

// The guide, `md-docs/*.md`, listed for `include_str!`, so `wf` carries the
// chapters of exactly the compiler it was built from (`agent::guide`).
fn guide() {
    let dir = Path::new("md-docs");
    println!("cargo:rerun-if-changed=md-docs");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|r| {
            r.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.ends_with(".md"))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let mut out = String::from(
        "/// Every chapter of `md-docs/`, by file name, in name order.\npub static FILES: &[(&str, &str)] = &[\n",
    );
    for name in &names {
        let path = Path::new(&root).join("md-docs").join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        out.push_str(&format!(
            "    ({name:?}, include_str!({:?})),\n",
            path.display().to_string()
        ));
    }
    out.push_str("];\n");
    let target = Path::new(&std::env::var("OUT_DIR").unwrap_or_default()).join("guide_files.rs");
    std::fs::write(target, out).expect("write the guide's list");
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
