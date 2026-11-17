// Umwandlung starten: Aufträge nacheinander im Hintergrund-Thread abarbeiten.
// Die eigentliche Arbeit macht der eigene Konverter (konverter.rs), ffmpeg wird nicht mehr gebraucht.

use crate::{cover, konverter};
use eframe::egui;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

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

// Aufträge (Index, Quelle, Ziel) nacheinander in einem Hintergrund-Thread abarbeiten
// und Status-Nachrichten an die Oberfläche schicken
pub fn starte(
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
            // Bei AIFF: Cover aus dem Quellordner suchen und einbetten
            let bild = if ziel == Ziel::Aiff {
                quelle.parent().and_then(cover::suche)
            } else {
                None
            };
            // Ein Absturz im Konverter soll die Oberfläche nicht hängen lassen
            let ergebnis = catch_unwind(AssertUnwindSafe(|| {
                konverter::wandle(&quelle, &ziel_pfad, ziel, bild.as_deref(), &abbruch)
            }))
            .unwrap_or_else(|_| Err("Interner Fehler im Konverter".to_string()));
            let msg = match ergebnis {
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
