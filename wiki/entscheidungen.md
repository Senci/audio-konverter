# Entscheidungen

| Entscheidung | Grund |
|---|---|
| Rust + egui | kleine, schnelle Einzeldatei |
| ffmpeg statt Audacity | Audacity nur per mod-script-pipe, instabil bei vielen Dateien |
| Eigener Konverter später | ffmpeg nicht mehr mitliefern |
| Zuerst nur Windows 11 | macOS/Linux später per GitHub Actions |
| Ein Zielformat-Dropdown für alle | einfacher als je Datei |
| MP3-Quellen immer gesperrt | MP3 nicht erneut umwandeln |
| Zieldateien nie überschreiben | Datenverlust vermeiden |
| MP3 mit ID3v2.3 | beste Player-Kompatibilität |
| Dither beim Runterrechnen auf 16 Bit | saubereres Ergebnis |
| Code in Module aufgeteilt | eine `main.rs` wäre mit den offenen Punkten zu groß geworden |
| Nr zählt alle Zeilen | gesperrte Dateien bleiben sichtbar, Nr passt zur Gesamtzahl |
| CSV mit Semikolon, ohne Anführungszeichen | deutsches Excel öffnet das direkt in Spalten |
| Semikolon im Text wird zu Komma | sonst verrutschen die Spalten |
| BOM am Dateianfang | Umlaute in Excel korrekt |
| Protokoll-Knopf erst nach einem Lauf | ein Protokoll ohne Lauf wäre leer |

## Module

```mermaid
flowchart LR
    M["main.rs"] --> A["app.rs"]
    A --> G["gui.rs"]
    A --> S["scan.rs"]
    A --> C["convert.rs"]
    G --> B["bericht.rs"]
```

| Modul | Inhalt |
|---|---|
| `main.rs` | nur Start |
| `app.rs` | `App`, `Datei`, `Status`, Ordner, Start, Abbruch, Nachrichten |
| `gui.rs` | Seitenleiste, Tabelle, Fußleiste, Farben, Statustexte |
| `scan.rs` | liefert nur Pfade, `app.rs` baut daraus `Datei` |
| `convert.rs` | `Ziel`, `Msg`, Sperrregel, ffmpeg-Suche, Thread, Aufruf |
| `bericht.rs` | CSV-Protokoll erzeugen und speichern |

`convert.rs` ist die Stelle, an der ffmpeg später durch den eigenen Konverter getauscht wird, ohne die Oberfläche anzufassen.
