// Cover für AIFF-Metadaten: sucht ein Bild im selben Ordner wie die Quelldatei
// (Punkt 2 in offen.md). Eingebettet wird es von tags.rs.

use std::path::{Path, PathBuf};

const BILD_ENDUNGEN: [&str; 4] = ["jpg", "jpeg", "png", "bmp"];

// Erstes Bild im Ordner der Quelldatei finden (nicht rekursiv, nur dieser Ordner).
// Mehrere Bilder im Ordner: das alphabetisch erste wird verwendet.
pub fn suche(ordner: &Path) -> Option<PathBuf> {
    let mut treffer: Vec<PathBuf> = std::fs::read_dir(ordner)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            p.extension()
                .map(|e| BILD_ENDUNGEN.contains(&e.to_string_lossy().to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    treffer.sort();
    treffer.into_iter().next()
}
