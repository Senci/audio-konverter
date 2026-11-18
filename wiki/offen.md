# Noch zu integrieren

```mermaid
flowchart LR
    S["6 Standalone-Datei (erledigt)"]
```

Keine offenen Punkte.

## Erledigt

| Nr. | Punkt | Modul |
|---|---|---|
| 1 | Protokoll als CSV | `bericht.rs` |
| 2 | Cover in AIFF-Metadaten | `app.rs`, `tags.rs` |
| 3 | Bildersuche | `scan.rs` |
| 4 | Fenster „Cover“ (vorher „Ordner ohne Bild“) | `gui.rs` |
| 5 | Eigener Konverter | `konverter.rs` |
| 6 | Standalone-Datei (`cargo build --release`, Release mit LTO, C-Laufzeit eingebaut, Build über GitHub Actions) | `Cargo.toml`, `.cargo/`, `.github/workflows/` |
| 7 | Aufteilung in Module | `main.rs` u. a. |
| 8 | DJ-Player-Auswahl: nicht gebaut, siehe `entscheidungen.md` | – |
| 9 | Cover auch in MP3-Metadaten | `app.rs` |
| 10 | MP3-Umwandlung beschleunigen (60 Tracks WAV/FLAC nach MP3: 34 s) | `auftraege.rs` |
| 11 | Umwandlung nach FLAC (16 Bit, 44,1 kHz) | `konverter.rs`, `flac.rs`, `tags.rs` |
| 12 | Sprachauswahl Deutsch / Englisch | `texte.rs`, `gui.rs` |

Zu Punkt 6: `Cargo.toml` (Release mit `lto`, `codegen-units = 1`, `strip`), `.cargo/config.toml` (C-Laufzeit fest eingebaut, keine Visual-C++-Runtime nötig), `.github/workflows/windows.yml` (baut und testet auf GitHub, `.exe` als Download, bei Tag `v*` als Release). Im Sandbox-Test (Linux, 12 Dateien à 30 s, 96 kHz/24 Bit): AIFF 0,3 s, FLAC 0,8 s, MP3 2,3 s; alle 18 Tests grün. Die Windows-`.exe` wurde auf deinem PC gebaut und von dir als erledigt bestätigt.

Später: Builds für macOS und Linux Mint über GitHub Actions.
