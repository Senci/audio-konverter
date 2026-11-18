# Bekannte Probleme

| Problem | Lösung |
|---|---|
| `cargo run` ist sehr langsam | `cargo run --release` verwenden |
| Erster Build dauert lange oder LAME-Fehler beim Bauen | C++ Build Tools installieren (bringen den C-Compiler mit), VS Code neu starten |
| Build-Fehler mit `convert.rs` oder `cover.rs` | Beide Dateien gibt es nicht mehr: aus `src/` löschen |
| „file not found for module“ | In `src/` müssen liegen: `main`, `app`, `gui`, `icons`, `texte`, `scan`, `auftraege`, `konverter`, `aiff`, `mp3`, `flac`, `tags`, `bericht` (je `.rs`), dazu `assets/icons/` im Projektordner |
| `cargo` nicht gefunden | Rust über rustup.rs installieren, VS Code neu starten |
| Protokoll-Knopf ist grau | Erst nach einer Umwandlung aktiv, wieder grau nach Änderung von Ordnern oder Zielformat |
| CSV in Excel: Umlaute falsch | Datei über „Daten → Aus Text/CSV“ öffnen, Trennzeichen „;“ |
| Fehler „Format nicht erkannt“ oder „Codec nicht unterstützt“ | Datei ist kein gültiges FLAC, WAV oder AIFF. Maus über **Fehler** halten |
| Fehler „Zieldatei existiert bereits“ (englisch: „Target file already exists“) | Ziel liegt schon im Ordner, z. B. weil `Track.wav` und `Track.flac` beide `Track.aiff` ergeben. Es wird nie überschrieben |
| Sprache gewechselt, alter Fehlertext bleibt | Fehlertexte entstehen beim Umwandeln und bleiben in der alten Sprache, bis die Datei neu umgewandelt wird |
| Sprachwahl wird nicht gemerkt | Das Programm schreibt `sprache.txt` nach `%APPDATA%\audio-konverter`; ohne Schreibrecht gilt beim nächsten Start wieder die Systemsprache |

Bei Fehlern: Maus über das rote **Fehler** halten. Immer mit Kopien testen.
