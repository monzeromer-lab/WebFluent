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
