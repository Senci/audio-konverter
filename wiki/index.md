# Audio-Konverter – Wiki

Windows-11-Programm (Rust, egui): wandelt ganze Ordner mit FLAC, WAV, AIFF, MP3 in AIFF (16 Bit, 44,1 kHz) oder MP3 (320 kbit/s) um, für den Pioneer XDJ-700. Umwandlung aktuell per ffmpeg.

```mermaid
flowchart LR
    GUI["GUI (egui)"] -->|Start| T["Hintergrund-Thread"]
    T -->|startet pro Datei| F["ffmpeg.exe"]
    T -->|"Status (Channel)"| GUI
    F -.->|später ersetzt durch| E["eigener Konverter"]
```

## Seiten
- [funktionen.md](funktionen.md) – was es kann
- [offen.md](offen.md) – was noch fehlt
- [entscheidungen.md](entscheidungen.md) – warum so
- [bekannte-probleme.md](bekannte-probleme.md) – Fehler und Lösung
- [diagramme/](diagramme/) – Excalidraw-Dateien (`ablauf`, `architektur`)

## Start
`cargo run` im Projektordner, `ffmpeg.exe` liegt neben `Cargo.toml`.

## Regeln
KISS, YAGNI, Deutsch, kurz halten, bevorzugt Mermaid-Diagramme. Nicht coden, wenn „nichts coden“ gesagt wird.
