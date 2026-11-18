# Funktionen

- Ordner per Knopf (mehrere auf einmal), per Drag & Drop oder als Programm-Argument hinzufügen; Unterordner werden durchsucht
- Quellen: FLAC, WAV, AIFF (MP3 wird gelistet, aber nie umgewandelt)
- Ziel per Dropdown: AIFF 16 Bit 44,1 kHz oder MP3 320 kbit/s 44,1 kHz
- Tabelle: Häkchen, Nr, Datei, Ordner (kurz, ab Quellordner), Format (z. B. FLAC → AIFF), Status mit Icon
- Kopf-Häkchen wählt alle wählbaren Dateien an oder ab
- Gleicher Ordner = gleiche Zeilenfarbe; flüssig auch bei tausenden Dateien
- Mehrere Dateien gleichzeitig (ein Thread pro CPU-Kern), Abbrechen jederzeit
- Ziel liegt im Ordner der Quelle; Originale und vorhandene Dateien werden nie verändert
- Nach dem Lauf: Zusammenfassung in der Statusleiste, Protokoll als CSV (`nr;datei;Ordner/Pfad;von;nach;status`)
- Fenster „Cover“: Ordner ohne Bild und alle gefundenen Bilder; das erste Bild eines Ordners wird in AIFF und MP3 eingebettet
- Tags (Titel, Künstler, Album, Album-Künstler, Genre, Jahr, Track, Disc, Komponist, BPM, Label) werden übernommen
- Helles und dunkles Systemthema, Lucide-Icons

## Ablauf

```mermaid
flowchart LR
    A["Ordner wählen<br/>gui.rs"] --> B["Dateien und Bilder suchen<br/>scan.rs"] --> C["Sperren und Status<br/>app.rs"] --> D["Umwandeln<br/>auftraege.rs"] --> E["parallel pro Datei<br/>konverter.rs"] --> F["Zusammenfassung<br/>app.rs"] --> G["CSV-Protokoll<br/>bericht.rs"]
```

## Sperren

```mermaid
flowchart TD
    A["Datei"] --> B{"MP3?"}
    B -->|ja| X["gesperrt"]
    B -->|nein| C{"AIFF und Ziel AIFF?"}
    C -->|ja| X
    C -->|nein| D{"Zieldatei existiert?"}
    D -->|ja| Y["existiert bereits"]
    D -->|nein| Z["bereit"]
```

## Umwandlung

```mermaid
flowchart LR
    Q["Quelle<br/>FLAC, WAV, AIFF"] --> L["lesen<br/>symphonia"] --> R["auf 44,1 kHz<br/>rubato"] --> D["auf 16 Bit<br/>Dither nur bei AIFF"] --> Z{"Ziel"}
    Z -->|AIFF| A["aiff.rs"]
    Z -->|MP3| M["mp3.rs, LAME 320 kbit/s"]
    T["tags.rs: Tags + Cover"] --> A
    T --> M
```
