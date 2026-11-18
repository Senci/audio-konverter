# Audio-Konverter

[English](README.md)

Windows-Programm (Rust, egui), das ganze Ordner mit Audiodateien umwandelt, zum Beispiel für den Pioneer XDJ-700 (spielt AIFF und MP3, kein FLAC). Eigener Konverter eingebaut, kein ffmpeg nötig.

```mermaid
flowchart LR
    A["Ordner hinzufügen"] --> B["Dateien suchen"] --> C["Ziel wählen"] --> D["Parallel umwandeln"] --> E["Zusammenfassung und CSV-Protokoll"]
```

## Funktionen

- Ordner per Knopf (mehrere auf einmal), Drag & Drop oder Kommandozeile; Unterordner werden durchsucht
- Quellen: FLAC, WAV, AIFF; Ziel **AIFF** (16 Bit, 44,1 kHz) oder **MP3** (320 kbit/s, 44,1 kHz)
- Kompakte Tabelle: Häkchen, Nr, Datei, kurzer Ordnerpfad, Format, Status mit Icon; alle an/aus im Kopf
- Mehrere Dateien gleichzeitig (ein Thread pro CPU-Kern), Abbrechen jederzeit
- Ergebnis neben der Quelle; Originale und vorhandene Dateien bleiben unverändert
- Tags werden übernommen; das erste Bild eines Ordners wird als Cover eingebettet (AIFF und MP3)
- Fenster „Cover“: Ordner ohne Cover und alle gefundenen Bilder
- CSV-Protokoll nach dem Lauf, helles und dunkles Thema, Lucide-Icons

## Voraussetzungen

- Windows 11
- [Rust](https://rustup.rs) mit den C++ Build Tools (C-Compiler für den eingebauten MP3-Encoder)

## Starten

```
git clone <repo-url>
cd audio-konverter
cargo run --release
```

Für echte Umwandlungen immer `--release`. Tests: `cargo test --release`. Fertige Datei: `cargo build --release`, Ergebnis `target\release\audio-konverter.exe`.

## Projektstruktur

```
audio-konverter/
├── Cargo.toml, Cargo.lock
├── assets/icons/     Lucide-Icons (SVG, ISC)
├── src/
│   ├── main.rs       Start
│   ├── app.rs        Zustand, Ablauf
│   ├── gui.rs        Oberfläche
│   ├── icons.rs      Icons
│   ├── scan.rs       Audio- und Bilddateien finden
│   ├── auftraege.rs  Parallele Aufträge im Hintergrund
│   ├── konverter.rs  Eigener Konverter: lesen, umrechnen, 16 Bit
│   ├── aiff.rs       AIFF schreiben
│   ├── mp3.rs        MP3 schreiben (LAME)
│   ├── tags.rs       Tags und Cover als ID3
│   └── bericht.rs    CSV-Protokoll
└── wiki/             Funktionen, Entscheidungen, Review, Diagramme
```

## Geplant

- Standalone-Datei, später macOS und Linux
- DJ-Player-Auswahl mit unterstützten Formaten (Pioneer, Denon)

Details im [Wiki](wiki/index.md). Immer mit Kopien der Dateien testen.
