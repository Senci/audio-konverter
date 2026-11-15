# Bekannte Probleme

| Problem | Lösung |
|---|---|
| „ffmpeg konnte nicht gestartet werden: program not found“ | In `finde_ffmpeg()` (`convert.rs`) den absoluten Pfad `current_dir().join(name)` verwenden statt `PathBuf::from(name)` (im Code bereits behoben) |
| `cargo run` bricht ab, „file not found for module“ | Alle Dateien müssen in `src/` liegen: `main.rs`, `app.rs`, `gui.rs`, `scan.rs`, `convert.rs`, `bericht.rs` |
| `cargo run` bricht sofort ab | `main.rs` muss in `src/` liegen, nicht im Projektordner |
| `cargo` nicht gefunden | Rust über rustup.rs installieren, VS Code neu starten |
| `.gitignore` wirkt nicht | Datei muss exakt `.gitignore` heißen |
| Rote Meldung „ffmpeg nicht gefunden“ | `ffmpeg.exe` neben `Cargo.toml` oder neben die Programmdatei legen |

Bei Fehlern: Maus über das rote **Fehler** halten, dort steht der ffmpeg-Text. Immer mit Kopien testen.
