// Ordner durchsuchen: findet Audiodateien in einem Ordner samt Unterordnern.
// Später kommt hier die Suche nach Bilddateien dazu.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Erkannte Quellformate
const ENDUNGEN: [&str; 5] = ["flac", "wav", "aiff", "aif", "mp3"];

// Dateiendung in Kleinbuchstaben, z. B. "flac"
pub fn endung_von(pfad: &Path) -> String {
    pfad.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

// Alle Audiodateien im Ordner und in allen Unterordnern
pub fn audiodateien(wurzel: &Path) -> Vec<PathBuf> {
    WalkDir::new(wurzel)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| ENDUNGEN.contains(&endung_von(p).as_str()))
        .collect()
}
