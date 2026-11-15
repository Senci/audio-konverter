# Audio-Konverter – Wiki

Windows-11-Programm (Rust, egui): wandelt ganze Ordner mit FLAC, WAV, AIFF, MP3 in AIFF (16 Bit, 44,1 kHz) oder MP3 (320 kbit/s) um, für den Pioneer XDJ-700. Umwandlung aktuell per ffmpeg. Protokoll als CSV.

```mermaid
flowchart TD
    main["main.rs"] --> app["app.rs"]
    app --> gui["gui.rs"]
    app --> scan["scan.rs"]
    app --> convert["convert.rs"]
    gui --> bericht["bericht.rs (CSV)"]
    convert --> ffmpeg["ffmpeg.exe"]
    ffmpeg -.->|später ersetzt durch| eigen["eigener Konverter"]
```

## Seiten
- [funktionen.md](funktionen.md) – was es kann
- [offen.md](offen.md) – was noch fehlt
- [entscheidungen.md](entscheidungen.md) – warum so
- [bekannte-probleme.md](bekannte-probleme.md) – Fehler und Lösung
- [diagramme/](diagramme/) – Excalidraw-Dateien (`ablauf`, `architektur`)

## Projektstruktur

```
audio-konverter/
├── Cargo.toml
├── ffmpeg.exe        (nicht im Git)
├── src/              main, app, gui, scan, convert, bericht
└── wiki/
```

## Start
`cargo run` im Projektordner, `ffmpeg.exe` liegt neben `Cargo.toml`.

## Regeln
KISS, YAGNI, Deutsch, kurz halten, bevorzugt Mermaid-Diagramme. Nicht coden, wenn „nichts coden“ gesagt wird.
