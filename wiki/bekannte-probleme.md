# Bekannte Probleme

| Problem | Lösung |
|---|---|
| „ffmpeg konnte nicht gestartet werden: program not found“ | Im Code behoben: `finde_ffmpeg()` nutzt den absoluten Pfad (`current_dir().join(name)`) |
| `cargo run` bricht ab, „file not found for module“ | Alle Dateien müssen in `src/` liegen: `main.rs`, `app.rs`, `gui.rs`, `scan.rs`, `convert.rs`, `bericht.rs` |
| `cargo` nicht gefunden | Rust über rustup.rs installieren, VS Code neu starten |
| `.gitignore` wirkt nicht | Datei muss exakt `.gitignore` heißen |
| Rote Meldung „ffmpeg nicht gefunden“ | `ffmpeg.exe` neben `Cargo.toml` oder neben die Programmdatei legen |
| Protokoll-Knopf ist grau | Erst nach einem Umwandlungslauf aktiv, und wieder grau nach Änderung von Ordnern oder Zielformat |
| CSV in Excel: Umlaute falsch | BOM ist gesetzt. Datei mit Excel über „Daten → Aus Text/CSV“ öffnen und Trennzeichen „;“ wählen |
| CSV in Excel: Spalten verrutschen | Ein Semikolon im Dateinamen oder Fehlertext wird zu Komma ersetzt. Sonst Trennzeichen prüfen |

Bei Fehlern: Maus über das rote **Fehler** halten, dort steht der ffmpeg-Text. Immer mit Kopien testen.
