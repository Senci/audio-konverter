# Audio Converter

Windows desktop app with a graphical interface (Rust, egui) that converts whole folders of audio files, for example for the Pioneer XDJ-700 (plays AIFF and MP3, no FLAC).

[Deutsch](README.de.md)

```mermaid
flowchart LR
    A["Select folder"] --> B["Find files"] --> C["Choose target format"] --> D["Start"] --> E["Convert per file"]
```

## Features

- Select whole folders, subfolders are scanned, multiple folders possible
- Sources: FLAC, WAV, AIFF, MP3
- Target via dropdown: **AIFF** (16 bit, 44.1 kHz) or **MP3** (320 kbit/s, 44.1 kHz)
- Table with number, file, path, conversion and status, same folder = same row color
- Checkbox per file to exclude it
- Locked: MP3 sources always, AIFF sources when the target is AIFF
- Output is saved in the source folder, originals and existing target files stay untouched
- Start, cancel and progress bar
- CSV log of all files (download button after a run), separated by semicolons
- Images are listed, never converted; button "Folders without image" shows folders without cover
- AIFF output gets the first image from the source folder as cover art

## Requirements

- Windows 11
- [Rust](https://rustup.rs) (on Windows with the C++ Build Tools, they include the C compiler needed for the MP3 encoder)

No ffmpeg needed.

## Run

```
git clone <repo-url>
cd audio-konverter
```

Then:

```
cargo run --release
```

Use `--release` for real conversions. The debug build (`cargo run`) is much slower.

Standalone build: `cargo build --release`. The result is `target\release\audio-konverter.exe`.

## Project structure

```
audio-konverter/
├── Cargo.toml
├── src/
│   ├── main.rs       Start
│   ├── app.rs        State, flow
│   ├── gui.rs        User interface
│   ├── scan.rs       Find audio and image files
│   ├── convert.rs    Background thread, status
│   ├── konverter.rs  Own converter: read, resample, 16 bit
│   ├── aiff.rs       AIFF writer
│   ├── mp3.rs        MP3 writer (LAME)
│   ├── tags.rs       Tags and cover as ID3
│   ├── cover.rs      Find cover in source folder
│   └── bericht.rs    CSV log
└── wiki/             features, decisions, diagrams (German)
```

## Planned

- Standalone file, later macOS and Linux
- DJ player selection with supported formats (Pioneer, Denon)

Details in the [wiki](wiki/index.md).

## Note

Always test with copies of your files.
