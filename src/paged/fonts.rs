//! The faces a document can be set in, and how a style picks one.
//!
//! Three sources, in order: the project's own fonts (files under `fonts/`,
//! the `@font-face` rules of its sheets, `pdf.fonts`), the Liberation family
//! bundled inside `wf` for every generic and standard name, and — only for
//! a family nothing else has, or a character no named face covers — the
//! fonts installed on the machine, each use of which is noted so a
//! document can be made reproducible.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use ttf_parser::{Face as TtfFace, name_id};

/// One face: a family at one weight and slant (or a variable range).
#[derive(Clone)]
pub struct Face {
    pub family: String,
    /// The lightest and heaviest weight it draws; equal for a static face.
    pub weights: (u16, u16),
    pub italic: bool,
    pub data: Arc<Vec<u8>>,
    pub index: u32,
    pub source: Source,
    /// Font units per em, and the vertical metrics in those units.
    pub units_per_em: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub variable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Bundled,
    Project(PathBuf),
    System(PathBuf),
}

/// A face chosen for a run: which one, at which weight, and what the
/// engine must fake because the family has no such face.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pick {
    pub face: usize,
    /// The weight a variable face is drawn at; the face's own for a static one.
    pub weight: u16,
    pub synthetic_bold: bool,
    pub synthetic_italic: bool,
}

pub struct FontDb {
    pub faces: Vec<Face>,
    /// Family (lower case) → faces.
    by_family: HashMap<String, Vec<usize>>,
    system_loaded: bool,
    system_paths: Vec<(String, PathBuf, u32)>,
    /// Families that were found only on this machine.
    pub system_used: Vec<String>,
    /// Characters → the face that covers them, once looked up.
    coverage_cache: HashMap<char, Option<usize>>,
    allow_system: bool,
}

// A reference to the bytes, never a copy: as a constant expression the
// compiler would evaluate two megabytes of data element by element.
macro_rules! bundled {
    ($name:literal) => {{
        static BYTES: &[u8] = include_bytes!(concat!("fonts/", $name, ".ttf.z"));
        BYTES
    }};
}

/// The bundled faces: (family, weight, italic, compressed bytes).
static BUNDLED: &[(&str, u16, bool, &[u8])] = &[
    ("Liberation Sans", 400, false, bundled!("LiberationSans-Regular")),
    ("Liberation Sans", 700, false, bundled!("LiberationSans-Bold")),
    ("Liberation Sans", 400, true, bundled!("LiberationSans-Italic")),
    ("Liberation Sans", 700, true, bundled!("LiberationSans-BoldItalic")),
    ("Liberation Serif", 400, false, bundled!("LiberationSerif-Regular")),
    ("Liberation Serif", 700, false, bundled!("LiberationSerif-Bold")),
    ("Liberation Serif", 400, true, bundled!("LiberationSerif-Italic")),
    ("Liberation Serif", 700, true, bundled!("LiberationSerif-BoldItalic")),
    ("Liberation Mono", 400, false, bundled!("LiberationMono-Regular")),
    ("Liberation Mono", 700, false, bundled!("LiberationMono-Bold")),
    ("Liberation Mono", 400, true, bundled!("LiberationMono-Italic")),
    ("Liberation Mono", 700, true, bundled!("LiberationMono-BoldItalic")),
];

/// The bundled faces, inflated once per process.
fn bundled_faces() -> &'static Vec<Face> {
    static FACES: OnceLock<Vec<Face>> = OnceLock::new();
    FACES.get_or_init(|| {
        BUNDLED
            .iter()
            .filter_map(|(family, weight, italic, z)| {
                let data = miniz_oxide::inflate::decompress_to_vec_zlib(z).ok()?;
                let mut face = read_face(Arc::new(data), 0, Source::Bundled)?;
                face.family = family.to_string();
                face.weights = (*weight, *weight);
                face.italic = *italic;
                Some(face)
            })
            .collect()
    })
}

/// The fonts an SVG's text is set in: the bundled faces, and the machine's.
/// Read once per process; an icon or a chart's labels need no more.
pub fn svg_fontdb() -> Arc<fontdb::Database> {
    static DB: OnceLock<Arc<fontdb::Database>> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = fontdb::Database::new();
        for face in bundled_faces() {
            db.load_font_data(face.data.as_ref().clone());
        }
        db.load_system_fonts();
        db.set_sans_serif_family("Liberation Sans");
        db.set_serif_family("Liberation Serif");
        db.set_monospace_family("Liberation Mono");
        Arc::new(db)
    })
    .clone()
}

/// The bundled family a generic or standard name stands for.
pub fn alias(family: &str) -> Option<&'static str> {
    Some(match family.to_ascii_lowercase().as_str() {
        "sans-serif" | "system-ui" | "-apple-system" | "blinkmacsystemfont" | "segoe ui"
        | "roboto" | "helvetica" | "helvetica neue" | "arial" | "ui-sans-serif" | "liberation sans"
        | "arimo" | "noto sans" | "ubuntu" | "cantarell" | "open sans" => "Liberation Sans",
        "serif" | "times" | "times new roman" | "ui-serif" | "liberation serif" | "tinos"
        | "georgia" => "Liberation Serif",
        "monospace" | "ui-monospace" | "sfmono-regular" | "sf mono" | "menlo" | "consolas"
        | "monaco" | "courier" | "courier new" | "liberation mono" | "cousine"
        | "lucida console" => "Liberation Mono",
        _ => return None,
    })
}

/// Read a face's name, weight, slant and metrics from its own tables.
fn read_face(data: Arc<Vec<u8>>, index: u32, source: Source) -> Option<Face> {
    let ttf = TtfFace::parse(&data, index).ok()?;
    let name = |id: u16| {
        ttf.names()
            .into_iter()
            .filter(|n| n.name_id == id && n.is_unicode())
            .find_map(|n| n.to_string())
    };
    let family = name(name_id::TYPOGRAPHIC_FAMILY).or_else(|| name(name_id::FAMILY))?;
    let weight = ttf.weight().to_number();
    let variable = ttf.is_variable();
    let weights = if variable {
        ttf.variation_axes()
            .into_iter()
            .find(|a| a.tag == ttf_parser::Tag::from_bytes(b"wght"))
            .map(|a| (a.min_value as u16, a.max_value as u16))
            .unwrap_or((weight, weight))
    } else {
        (weight, weight)
    };
    let (ascent, descent, line_gap) = (
        ttf.ascender() as f32,
        -(ttf.descender() as f32),
        ttf.line_gap() as f32,
    );
    Some(Face {
        family,
        weights,
        italic: ttf.is_italic() || ttf.is_oblique(),
        units_per_em: ttf.units_per_em() as f32,
        ascent,
        descent,
        line_gap,
        variable,
        data: data.clone(),
        index,
        source,
    })
}

impl FontDb {
    /// The bundled faces, and whether the machine's may be consulted.
    pub fn new(allow_system: bool) -> FontDb {
        let mut db = FontDb {
            faces: Vec::new(),
            by_family: HashMap::new(),
            system_loaded: false,
            system_paths: Vec::new(),
            system_used: Vec::new(),
            coverage_cache: HashMap::new(),
            allow_system,
        };
        for face in bundled_faces() {
            db.add(face.clone());
        }
        db
    }

    fn add(&mut self, face: Face) -> usize {
        let ix = self.faces.len();
        self.by_family
            .entry(face.family.to_ascii_lowercase())
            .or_default()
            .push(ix);
        self.faces.push(face);
        ix
    }

    /// Every face in a font file (a collection has several), under the
    /// family its tables name, or under `family` when one is given.
    pub fn add_file(&mut self, path: &Path, family: Option<&str>) -> usize {
        let Ok(bytes) = std::fs::read(path) else {
            return 0;
        };
        let data = Arc::new(bytes);
        let count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
        let mut added = 0;
        for index in 0..count {
            if let Some(mut face) = read_face(data.clone(), index, Source::Project(path.to_path_buf())) {
                if let Some(f) = family {
                    face.family = f.to_string();
                }
                // A family registered twice from the same file is one face.
                if self.faces.iter().any(|f| f.source == face.source && f.index == index && f.family == face.family) {
                    continue;
                }
                self.add(face);
                added += 1;
            }
        }
        added
    }

    /// Every `.ttf`, `.otf` and `.ttc` under a directory.
    pub fn add_dir(&mut self, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                self.add_dir(&path);
            } else if is_font_file(&path) {
                self.add_file(&path, None);
            }
        }
    }

    fn load_system(&mut self) {
        if self.system_loaded || !self.allow_system {
            return;
        }
        self.system_loaded = true;
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        for face in db.faces() {
            if let fontdb::Source::File(path) = &face.source {
                for (family, _) in &face.families {
                    self.system_paths.push((family.to_ascii_lowercase(), path.clone(), face.index));
                }
            }
        }
    }

    /// The faces of a family found on this machine, added on first use.
    fn system_family(&mut self, family: &str) -> Vec<usize> {
        self.load_system();
        let lower = family.to_ascii_lowercase();
        let paths: Vec<(PathBuf, u32)> = self
            .system_paths
            .iter()
            .filter(|(f, _, _)| *f == lower)
            .map(|(_, p, i)| (p.clone(), *i))
            .collect();
        let mut out = Vec::new();
        for (path, index) in paths {
            if let Some(existing) = self.faces.iter().position(|f| f.source == Source::System(path.clone()) && f.index == index) {
                out.push(existing);
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else { continue };
            if let Some(face) = read_face(Arc::new(bytes), index, Source::System(path.clone())) {
                let mut face = face;
                face.family = family.to_string();
                out.push(self.add(face));
            }
        }
        if !out.is_empty() && !self.system_used.iter().any(|f| f.eq_ignore_ascii_case(family)) {
            self.system_used.push(family.to_string());
        }
        out
    }

    /// The faces of the first family in the list that something has.
    pub fn family_faces(&mut self, families: &[String]) -> Vec<usize> {
        for family in families {
            if let Some(v) = self.by_family.get(&family.to_ascii_lowercase()) {
                if self.faces[v[0]].source != Source::Bundled || alias(family).is_some() {
                    return v.clone();
                }
            }
            if let Some(bundled) = alias(family) {
                return self.by_family[&bundled.to_ascii_lowercase()].clone();
            }
            let system = self.system_family(family);
            if !system.is_empty() {
                return system;
            }
        }
        self.by_family["liberation sans"].clone()
    }

    /// The face of `families` that best matches a weight and slant, as CSS
    /// matches one.
    pub fn pick(&mut self, families: &[String], weight: u16, italic: bool) -> Pick {
        let candidates = self.family_faces(families);
        self.best_of(&candidates, weight, italic)
    }

    fn best_of(&self, candidates: &[usize], weight: u16, italic: bool) -> Pick {
        let slanted: Vec<usize> = candidates.iter().copied().filter(|&c| self.faces[c].italic == italic).collect();
        let pool = if slanted.is_empty() { candidates.to_vec() } else { slanted.clone() };
        let distance = |c: usize| -> i32 {
            let (lo, hi) = self.faces[c].weights;
            let w = weight as i32;
            if w >= lo as i32 && w <= hi as i32 {
                0
            } else if w < lo as i32 {
                // Heavier than asked: fine for a bold request, less so for
                // a light one.
                (lo as i32 - w) * if weight >= 500 { 1 } else { 2 }
            } else {
                (w - hi as i32) * if weight >= 500 { 2 } else { 1 }
            }
        };
        let face = pool.iter().copied().min_by_key(|&c| distance(c)).unwrap_or(0);
        let f = &self.faces[face];
        let drawn = weight.clamp(f.weights.0, f.weights.1);
        Pick {
            face,
            weight: if f.variable { drawn } else { f.weights.0 },
            synthetic_bold: weight >= 600 && f.weights.1 < 550,
            synthetic_italic: italic && !f.italic,
        }
    }

    pub fn covers(&self, face: usize, ch: char) -> bool {
        let f = &self.faces[face];
        TtfFace::parse(&f.data, f.index)
            .ok()
            .and_then(|t| t.glyph_index(ch))
            .is_some()
    }

    /// A face, from anywhere, that has a glyph for `ch` — at the asked
    /// weight and slant when its family has one.
    pub fn fallback(&mut self, ch: char, weight: u16, italic: bool) -> Option<Pick> {
        let face = match self.coverage_cache.get(&ch) {
            Some(f) => *f,
            None => {
                let found = self.find_covering(ch);
                self.coverage_cache.insert(ch, found);
                found
            }
        }?;
        let family = self.faces[face].family.to_ascii_lowercase();
        let siblings = self.by_family.get(&family).cloned().unwrap_or_else(|| vec![face]);
        let covering: Vec<usize> = siblings.into_iter().filter(|&s| self.covers(s, ch)).collect();
        Some(self.best_of(if covering.is_empty() { std::slice::from_ref(&face) } else { &covering }, weight, italic))
    }

    fn find_covering(&mut self, ch: char) -> Option<usize> {
        // The project's and the bundled faces first.
        if let Some(f) = (0..self.faces.len()).find(|&f| self.covers(f, ch)) {
            return Some(f);
        }
        self.load_system();
        // Then the machine's, a family at a time, the plainest first: a
        // Noto Sans for the script before a calligraphic, monospaced or
        // emoji face that happens to cover it too.
        let mut tried: Vec<String> = Vec::new();
        let mut families: Vec<(String, PathBuf, u32)> = self.system_paths.clone();
        families.sort_by_key(|(family, path, _)| (fallback_rank(family), path.clone()));
        for (family, path, index) in families {
            if tried.contains(&family) {
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else { continue };
            let Ok(ttf) = TtfFace::parse(&bytes, index) else { continue };
            if ttf.glyph_index(ch).is_none() {
                continue;
            }
            tried.push(family.clone());
            let name = ttf
                .names()
                .into_iter()
                .filter(|n| (n.name_id == name_id::TYPOGRAPHIC_FAMILY || n.name_id == name_id::FAMILY) && n.is_unicode())
                .find_map(|n| n.to_string())
                .unwrap_or(family.clone());
            let faces = self.system_family(&name);
            if let Some(&f) = faces.iter().find(|&&f| self.covers(f, ch)) {
                return Some(f);
            }
        }
        None
    }

    pub fn face(&self, pick: Pick) -> &Face {
        &self.faces[pick.face]
    }
}

/// How plain a family is as a stand-in, lower first.
fn fallback_rank(family: &str) -> u32 {
    let f = family.to_ascii_lowercase();
    let mut rank = if f.starts_with("noto sans") {
        0
    } else if f.starts_with("noto naskh") || f.starts_with("noto serif") || f.starts_with("noto kufi") {
        2
    } else if f.starts_with("dejavu sans") || f.starts_with("freesans") || f.starts_with("liberation") {
        3
    } else {
        5
    };
    for (word, penalty) in [
        ("mono", 4),
        ("nastaliq", 6),
        ("fallback", 3),
        ("emoji", 8),
        ("color", 6),
        ("display", 2),
        ("ui", 1),
        ("condensed", 2),
        ("narrow", 2),
        ("symbols", 1),
        ("math", 3),
    ] {
        if f.split(|c: char| !c.is_alphanumeric()).any(|w| w == word) {
            rank += penalty;
        }
    }
    rank
}

pub fn is_font_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(),
        Some("ttf" | "otf" | "ttc" | "otc")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fam(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_bundled_family_stands_for_every_generic_name() {
        let mut db = FontDb::new(false);
        for name in ["sans-serif", "system-ui", "Helvetica", "Arial"] {
            let p = db.pick(&fam(&[name]), 400, false);
            assert_eq!(db.face(p).family, "Liberation Sans", "{name}");
        }
        let p = db.pick(&fam(&["Georgia", "serif"]), 400, false);
        assert_eq!(db.face(p).family, "Liberation Serif");
        let p = db.pick(&fam(&["ui-monospace", "monospace"]), 400, false);
        assert_eq!(db.face(p).family, "Liberation Mono");
        // The default theme's stack.
        let stack = crate::paged::style::parse_family_list(
            "system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
        );
        let p = db.pick(&stack, 400, false);
        assert_eq!(db.face(p).family, "Liberation Sans");
        // A name nothing has falls to the default when the machine may not
        // be asked.
        let p = db.pick(&fam(&["Nonexistent Grotesk"]), 400, false);
        assert_eq!(db.face(p).family, "Liberation Sans");
    }

    #[test]
    fn weight_and_slant_pick_the_face_or_fake_it() {
        let mut db = FontDb::new(false);
        let bold = db.pick(&fam(&["sans-serif"]), 700, false);
        assert_eq!(db.face(bold).weights.0, 700);
        assert!(!bold.synthetic_bold);
        let semibold = db.pick(&fam(&["sans-serif"]), 600, false);
        assert_eq!(db.face(semibold).weights.0, 700);
        let light = db.pick(&fam(&["sans-serif"]), 300, false);
        assert_eq!(db.face(light).weights.0, 400);
        let it = db.pick(&fam(&["serif"]), 400, true);
        assert!(db.face(it).italic && !it.synthetic_italic);
    }

    #[test]
    fn coverage_finds_a_face_with_the_glyph() {
        let mut db = FontDb::new(false);
        // Latin, Greek and Cyrillic are in the bundled faces.
        for ch in ['é', 'ż', 'Ω', 'Ж', '€', '—'] {
            let p = db.fallback(ch, 400, false).expect("covered");
            assert_eq!(db.face(p).source, Source::Bundled, "{ch}");
        }
        // Without the machine's fonts, Arabic has no face.
        assert!(db.fallback('م', 400, false).is_none());
    }
}
