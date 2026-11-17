// Umwandlung: aktuell mit ffmpeg. Später kann hier der eigene Konverter
// ffmpeg ersetzen, ohne dass die Oberfläche angepasst werden muss.

use crate::cover;
use eframe::egui;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// Zielformat (Dropdown)
#[derive(Clone, Copy, PartialEq)]
pub enum Ziel {
    Aiff,
    Mp3,
}

impl Ziel {
    pub fn name(self) -> &'static str {
        match self {
            Ziel::Aiff => "AIFF · 16 Bit · 44,1 kHz",
            Ziel::Mp3 => "MP3 · 320 kbit/s · 44,1 kHz",
        }
    }
    pub fn kurz(self) -> &'static str {
        match self {
            Ziel::Aiff => "AIFF",
            Ziel::Mp3 => "MP3",
        }
    }
    pub fn endung(self) -> &'static str {
        match self {
            Ziel::Aiff => "aiff",
            Ziel::Mp3 => "mp3",
        }
    }
}

// Nachrichten vom Umwandlungs-Thread an die Oberfläche
pub enum Msg {
    Laeuft(usize),
    Fertig(usize),
    Fehler(usize, String),
    Ende,
}

// MP3 ist immer gesperrt, AIFF nur beim Ziel AIFF
pub fn ist_gesperrt(endung: &str, ziel: Ziel) -> bool {
    endung == "mp3" || (ziel == Ziel::Aiff && (endung == "aiff" || endung == "aif"))
}

// ffmpeg suchen: erst neben der Programmdatei, dann im Arbeitsordner
// (beim Start mit `cargo run` ist das der Projektordner)
pub fn finde_ffmpeg() -> Option<PathBuf> {
    let name = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
    let mut kandidaten = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            kandidaten.push(dir.join(name));
        }
    }
    // Absoluter Pfad nötig: ein relativer Name wird sonst nur im PATH gesucht
    if let Ok(cwd) = std::env::current_dir() {
        kandidaten.push(cwd.join(name));
    }
    kandidaten.into_iter().find(|p| p.exists())
}

// Aufträge (Index, Quelle, Ziel) nacheinander in einem Hintergrund-Thread abarbeiten
// und Status-Nachrichten an die Oberfläche schicken
pub fn starte(
    ffmpeg: PathBuf,
    auftraege: Vec<(usize, PathBuf, PathBuf)>,
    ziel: Ziel,
    abbruch: Arc<AtomicBool>,
    tx: Sender<Msg>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        for (i, quelle, ziel_pfad) in auftraege {
            if abbruch.load(Ordering::Relaxed) {
                break;
            }
            let _ = tx.send(Msg::Laeuft(i));
            ctx.request_repaint();
            // Bei AIFF: Cover aus dem Quellordner suchen und einbetten (Punkt 2)
            let bild = if ziel == Ziel::Aiff {
                quelle.parent().and_then(cover::suche)
            } else {
                None
            };
            let msg = match konvertiere(&ffmpeg, &quelle, &ziel_pfad, ziel, bild.as_deref(), &abbruch) {
                Ok(()) => Msg::Fertig(i),
                Err(e) => Msg::Fehler(i, e),
            };
            let _ = tx.send(msg);
            ctx.request_repaint();
        }
        let _ = tx.send(Msg::Ende);
        ctx.request_repaint();
    });
}

// Eine Datei mit ffmpeg umwandeln. `bild` ist ein Cover, das bei AIFF
// in die Metadaten eingebettet wird (Punkt 2 in offen.md); bei MP3 ungenutzt.
fn konvertiere(
    ffmpeg: &Path,
    quelle: &Path,
    ziel: &Path,
    format: Ziel,
    bild: Option<&Path>,
    abbruch: &AtomicBool,
) -> Result<(), String> {
    // Vorhandene Dateien niemals anfassen
    if ziel.exists() {
        return Err("Zieldatei existiert bereits".into());
    }

    let hat_cover = format == Ziel::Aiff && bild.is_some();

    let mut cmd = Command::new(ffmpeg);
    cmd.args(["-n", "-nostdin", "-loglevel", "error", "-i"]).arg(quelle);
    if let Some(b) = bild.filter(|_| hat_cover) {
        cmd.arg("-i").arg(b); // zweiter Input: das Cover-Bild
    } else {
        cmd.arg("-vn"); // keine Cover-Bilder aus der Quelle übernehmen
    }

    match format {
        // AIFF 16 Bit, 44,1 kHz (mit Dither beim Runterrechnen von 24 Bit)
        Ziel::Aiff => {
            cmd.args([
                "-map",
                "0:a",
                "-af",
                "aresample=osr=44100:osf=s16:dither_method=triangular",
                "-c:a",
                "pcm_s16be",
                "-write_id3v2",
                "1",
            ]);
            if hat_cover {
                // Cover als angehängtes Bild in die ID3-Metadaten einbetten
                cmd.args([
                    "-map",
                    "1:v",
                    "-c:v",
                    "mjpeg",
                    "-disposition:v:0",
                    "attached_pic",
                ]);
            }
        }
        // MP3 320 kbit/s, 44,1 kHz, ID3v2.3 (am besten kompatibel)
        Ziel::Mp3 => {
            cmd.args([
                "-ar",
                "44100",
                "-c:a",
                "libmp3lame",
                "-b:a",
                "320k",
                "-id3v2_version",
                "3",
            ]);
        }
    };
    cmd.arg(ziel).stdout(Stdio::null()).stderr(Stdio::piped());

    // Unter Windows kein Konsolenfenster aufblitzen lassen
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000);

    let mut kind = cmd
        .spawn()
        .map_err(|e| format!("ffmpeg konnte nicht gestartet werden: {e}"))?;

    // Auf das Ende warten, dabei auf Abbruch prüfen
    loop {
        if abbruch.load(Ordering::Relaxed) {
            let _ = kind.kill();
            let _ = kind.wait();
            let _ = std::fs::remove_file(ziel); // halbfertige Datei löschen
            return Err("abgebrochen".into());
        }
        match kind.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }
                let mut fehler = String::new();
                if let Some(mut s) = kind.stderr.take() {
                    let _ = s.read_to_string(&mut fehler);
                }
                let _ = std::fs::remove_file(ziel);
                return Err(fehler.trim().to_string());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(e.to_string()),
        }
    }
}
