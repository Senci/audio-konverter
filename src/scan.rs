// Ordner durchsuchen: findet Audio- und Bilddateien in einem Ordner samt Unterordnern.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Erkannte Quellformate
const ENDUNGEN: [&str; 5] = ["flac", "wav", "aiff", "aif", "mp3"];

// Erkannte Bildformate (für Cover, Punkt 3 in offen.md)
const BILD_ENDUNGEN: [&str; 4] = ["jpg", "jpeg", "png", "bmp"];

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

// Alle Bilddateien im Ordner und in allen Unterordnern.
// Werden nur gelistet, nie konvertiert (siehe offen.md Punkt 3).
pub fn bilddateien(wurzel: &Path) -> Vec<PathBuf> {
    WalkDir::new(wurzel)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| BILD_ENDUNGEN.contains(&endung_von(p).as_str()))
        .collect()
}
