# Noch zu integrieren

```mermaid
flowchart LR
    S["6 Standalone-Datei (offen)"]
```

| Nr. | Punkt | Modul | Stand |
|---|---|---|---|
| 6 | Standalone-Datei ohne Rust und ohne Installation, muss wie `cargo run --release` schnell laufen | Build | offen |

## Erledigt

| Nr. | Punkt | Modul |
|---|---|---|
| 1 | Protokoll als CSV | `bericht.rs` |
| 2 | Cover in AIFF-Metadaten | `app.rs`, `tags.rs` |
| 3 | Bildersuche | `scan.rs` |
| 4 | Fenster „Cover“ (vorher „Ordner ohne Bild“) | `gui.rs` |
| 5 | Eigener Konverter | `konverter.rs` |
| 7 | Aufteilung in Module | `main.rs` u. a. |
| 8 | DJ-Player-Auswahl: nicht gebaut, siehe `entscheidungen.md` | – |
| 9 | Cover auch in MP3-Metadaten | `app.rs` |
| 10 | MP3-Umwandlung beschleunigen (60 Tracks WAV/FLAC nach MP3: 34 s) | `auftraege.rs` |
| 11 | Umwandlung nach FLAC (16 Bit, 44,1 kHz) | `konverter.rs`, `flac.rs`, `tags.rs` |
| 12 | Sprachauswahl Deutsch / Englisch | `texte.rs`, `gui.rs` |

Später: Builds für macOS und Linux Mint über GitHub Actions.
