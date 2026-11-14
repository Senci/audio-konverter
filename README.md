# Audio Converter

Windows desktop app with a graphical interface (Rust, egui) that converts whole folders of audio files, for example for the Pioneer XDJ-700 (plays AIFF and MP3, no FLAC).

[Deutsch](README.de.md)

```mermaid
flowchart LR
    A["Select folder"] --> B["Find files"] --> C["Choose target format"] --> D["Start"] --> E["ffmpeg per file"]
```

## Features

- Select whole folders, subfolders are scanned, multiple folders possible
- Sources: FLAC, WAV, AIFF, MP3
- Target via dropdown: **AIFF** (16 bit, 44.1 kHz) or **MP3** (320 kbit/s, 44.1 kHz)
- Table with path, conversion and status, same folder = same row color
- Checkbox per file to exclude it
- Locked: MP3 sources always, AIFF sources when the target is AIFF
- Output is saved in the source folder, originals and existing target files stay untouched
- Start, cancel and progress bar

## Requirements

- Windows 11
- [Rust](https://rustup.rs) (on Windows with the C++ Build Tools)
- `ffmpeg.exe` from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) (`ffmpeg-release-essentials.zip`, inside it `bin\ffmpeg.exe`)

## Run

```
git clone <repo-url>
cd audio-konverter
```

Put `ffmpeg.exe` next to `Cargo.toml`, then:

```
cargo run
```

Release build: `cargo build --release`. The result is `target\release\audio-konverter.exe`, with `ffmpeg.exe` next to it.

## Project structure

```
audio-konverter/
├── Cargo.toml
├── ffmpeg.exe        (not in Git)
├── src/main.rs
└── wiki/             features, decisions, diagrams (German)
```

## Planned

- Log of converted files
- Image search and cover art in AIFF metadata, window for folders without an image
- Built-in converter instead of ffmpeg
- Standalone file, later macOS and Linux

Details in the [wiki](wiki/index.md).

## Note

Always test with copies of your files.
