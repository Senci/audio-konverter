// Protokoll der Dateien als CSV: nr, datei, Ordner/Pfad, von, nach, status.

use crate::app::{Datei, Status};
use crate::convert::Ziel;
use crate::gui::status_text;
use std::path::Path;

// Trennzeichen zwischen den Feldern
const TRENNER: &str = ";";

// Ein Feld für die CSV aufbereiten: Zeilenumbrüche und Semikolons entschärfen,
// ohne Anführungszeichen
fn feld(text: &str) -> String {
    text.replace(['\r', '\n'], " ").replace(';', ",")
}

// Statuswert für das Protokoll
fn status_fuer_protokoll(d: &Datei) -> String {
    match &d.status {
        Status::Bereit if !d.gewaehlt => "nicht gewählt".to_string(),
        Status::Fehler(e) => format!("Fehler: {e}"),
        s => status_text(s).to_string(),
    }
}

// CSV-Text erzeugen, eine Zeile pro Datei in der Reihenfolge der Tabelle
pub fn erstelle(dateien: &[Datei], ziel: Ziel) -> String {
    // BOM am Anfang, damit Excel Umlaute richtig anzeigt
    let mut csv = String::from("\u{FEFF}");
    csv.push_str(&["nr", "datei", "Ordner/Pfad", "von", "nach", "status"].join(TRENNER));
    csv.push_str("\r\n");
    for (i, d) in dateien.iter().enumerate() {
        let felder = [
            (i + 1).to_string(),
            d.name.clone(),
            d.ordner.display().to_string(),
            d.von(),
            d.nach(ziel).to_string(),
            status_fuer_protokoll(d),
        ];
        let felder: Vec<String> = felder.iter().map(|f| feld(f)).collect();
        csv.push_str(&felder.join(TRENNER));
        csv.push_str("\r\n");
    }
    csv
}

// CSV-Datei speichern
pub fn speichern(dateien: &[Datei], ziel: Ziel, pfad: &Path) -> Result<(), String> {
    std::fs::write(pfad, erstelle(dateien, ziel)).map_err(|e| e.to_string())
}
