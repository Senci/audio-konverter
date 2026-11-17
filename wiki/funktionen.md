# Funktionen

- Ganze Ordner wählen, Unterordner werden durchsucht, mehrere Ordner möglich
- Quellen: FLAC, WAV, AIFF, MP3
- Tabelle: Nr, Datei, Pfad, Von, Nach, Status; gleicher Ordner = gleiche Zeilenfarbe
- Nr zählt ab 1, die Gesamtzahl steht oben
- Häkchen pro Datei zum Ausnehmen
- Zielformat per Dropdown: AIFF 16 Bit 44,1 kHz oder MP3 320 kbit/s 44,1 kHz
- Ziel liegt im Ordner der Quelle, Originale bleiben unverändert
- Start, Abbrechen, Fortschrittsbalken; Fehlertext per Mouseover
- Protokoll als CSV (Knopf nach einem Lauf): `nr;datei;Ordner/Pfad;von;nach;status`
- Gefundene Bilder werden aufgelistet (einklappbar), nie konvertiert
- Knopf „Ordner ohne Bild“ öffnet ein Fenster mit Ordnern ohne Cover
- Bei AIFF wird das erste Bild im Quellordner als Cover eingebettet

## Ablauf

```mermaid
flowchart LR
    A["Ordner wählen<br/>gui.rs"] --> B["Dateien und Bilder suchen<br/>scan.rs"] --> C["Sperren und Status<br/>app.rs"] --> D["Start<br/>app.rs"] --> E["Konverter pro Datei<br/>convert.rs, konverter.rs"] --> F["Status und Fortschritt<br/>app.rs, gui.rs"] --> G["CSV-Protokoll<br/>bericht.rs"]
```

## Sperren

```mermaid
flowchart TD
    A["Datei"] --> B{"MP3?"}
    B -->|ja| X["gesperrt"]
    B -->|nein| C{"AIFF und Ziel AIFF?"}
    C -->|ja| X
    C -->|nein| D{"Zieldatei existiert?"}
    D -->|ja| Y["existiert bereits (gelb)"]
    D -->|nein| Z["bereit"]
```

Die Regel steht in `convert.rs` (`ist_gesperrt`), die Statusvergabe in `app.rs`.

## Umwandlung

```mermaid
flowchart LR
    Q["Quelle<br/>FLAC, WAV, AIFF"] --> L["lesen<br/>symphonia"] --> R["auf 44,1 kHz<br/>rubato"] --> D["auf 16 Bit<br/>Dither nur bei AIFF"] --> Z{"Ziel"}
    Z -->|AIFF| A["aiff.rs"]
    Z -->|MP3| M["mp3.rs, LAME 320 kbit/s"]
    T["tags.rs: Tags + Cover"] --> A
    T --> M
```
