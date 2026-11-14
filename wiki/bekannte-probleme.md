# Bekannte Probleme

| Problem | Lösung |
|---|---|
| „ffmpeg konnte nicht gestartet werden: program not found“ | In `finde_ffmpeg()` den absoluten Pfad `current_dir().join(name)` statt `PathBuf::from(name)` eintragen |
| `cargo run` bricht sofort ab | `main.rs` muss in `src/` liegen |
| `cargo` nicht gefunden | Rust über rustup.rs installieren, VS Code neu starten |
| `.gitignore` wirkt nicht | Datei muss exakt `.gitignore` heißen |
| Rote Meldung „ffmpeg nicht gefunden“ | `ffmpeg.exe` neben `Cargo.toml` oder neben die Programmdatei legen |

Bei Fehlern: Maus über das rote **Fehler** halten, dort steht der ffmpeg-Text. Immer mit Kopien testen.
