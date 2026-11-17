# Bekannte Probleme

| Problem | Lösung |
|---|---|
| `cargo run` ist sehr langsam (mehrere Minuten statt Sekunden) | `cargo run --release` verwenden. Der Debug-Build ist für die Umwandlung nicht geeignet |
| Erster Build dauert lange, oder LAME-Fehler beim Bauen (C-Compiler fehlt) | C++ Build Tools installieren (sie bringen den C-Compiler mit), VS Code neu starten |
| `cargo run` bricht ab, „file not found for module“ | Alle Dateien müssen in `src/` liegen: `main.rs`, `app.rs`, `gui.rs`, `scan.rs`, `convert.rs`, `konverter.rs`, `aiff.rs`, `mp3.rs`, `tags.rs`, `cover.rs`, `bericht.rs` |
| `cargo` nicht gefunden | Rust über rustup.rs installieren, VS Code neu starten |
| `.gitignore` wirkt nicht | Datei muss exakt `.gitignore` heißen |
| Protokoll-Knopf ist grau | Erst nach einem Umwandlungslauf aktiv, und wieder grau nach Änderung von Ordnern oder Zielformat |
| CSV in Excel: Umlaute falsch | BOM ist gesetzt. Datei mit Excel über „Daten → Aus Text/CSV“ öffnen und Trennzeichen „;“ wählen |
| CSV in Excel: Spalten verrutschen | Ein Semikolon im Dateinamen oder Fehlertext wird zu Komma ersetzt. Sonst Trennzeichen prüfen |
| Fehler „Format nicht erkannt“ oder „Codec nicht unterstützt“ | Datei ist kein FLAC, WAV, AIFF oder MP3, oder hat ein ungewöhnliches Format. Maus über das rote **Fehler** halten |
| Fehler „Zieldatei existiert bereits“ | Die Zieldatei liegt schon im Ordner und wird nie überschrieben. Datei umbenennen oder löschen |

Bei Fehlern: Maus über das rote **Fehler** halten, dort steht die Fehlermeldung. Immer mit Kopien testen.
