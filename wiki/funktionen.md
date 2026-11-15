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

## Ablauf (mit Modulen)

```mermaid
flowchart LR
    A["Ordner wählen<br/>gui.rs"] --> B["Dateien suchen<br/>scan.rs"] --> C["Sperren und Status<br/>app.rs"] --> D["Start<br/>app.rs"] --> E["ffmpeg pro Datei<br/>convert.rs"] --> F["Status und Fortschritt<br/>app.rs, gui.rs"] --> G["CSV-Protokoll<br/>bericht.rs"]
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

## ffmpeg-Befehle

| Ziel | Optionen |
|---|---|
| AIFF | `-af aresample=osr=44100:osf=s16:dither_method=triangular -c:a pcm_s16be -write_id3v2 1` |
| MP3 | `-ar 44100 -c:a libmp3lame -b:a 320k -id3v2_version 3` |
| beide | `-n -nostdin -loglevel error -i quelle -vn ziel` |
