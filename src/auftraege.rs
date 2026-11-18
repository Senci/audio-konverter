// Aufträge im Hintergrund abarbeiten: mehrere Dateien gleichzeitig, ein Thread pro CPU-Kern.
// Die Oberfläche erfährt den Fortschritt über Nachrichten (Msg).

use crate::konverter::{self, Ziel, ABGEBROCHEN};
use crate::texte::t;
use eframe::egui;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

// Eine Datei, die umgewandelt werden soll
pub struct Auftrag {
    pub index: usize, // Zeile in der Dateiliste
    pub quelle: PathBuf,
    pub ziel_pfad: PathBuf,
    pub cover: Option<PathBuf>,
}

// Nachrichten an die Oberfläche
pub enum Msg {
    Laeuft(usize),
    Fertig(usize),
    Fehler(usize, String),
    Abgebrochen(usize),
    Ende,
}

// Aufträge parallel abarbeiten. Am Ende kommt immer genau ein Msg::Ende.
pub fn starte(
    auftraege: Vec<Auftrag>,
    ziel: Ziel,
    abbruch: Arc<AtomicBool>,
    tx: Sender<Msg>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || {
        let naechster = AtomicUsize::new(0);
        let threads = std::thread::available_parallelism()
            .map_or(2, |n| n.get())
            .min(auftraege.len())
            .max(1);
        std::thread::scope(|s| {
            for _ in 0..threads {
                let tx = tx.clone();
                let (auftraege, naechster, abbruch, ctx) = (&auftraege, &naechster, &abbruch, &ctx);
                s.spawn(move || {
                    let melde = |m: Msg| {
                        let _ = tx.send(m);
                        ctx.request_repaint();
                    };
                    while !abbruch.load(Ordering::Relaxed) {
                        let Some(a) = auftraege.get(naechster.fetch_add(1, Ordering::Relaxed))
                        else {
                            break;
                        };
                        melde(Msg::Laeuft(a.index));
                        // Ein Absturz im Konverter soll die Oberfläche nicht hängen lassen
                        let ergebnis = catch_unwind(AssertUnwindSafe(|| {
                            konverter::wandle(
                                &a.quelle,
                                &a.ziel_pfad,
                                ziel,
                                a.cover.as_deref(),
                                abbruch,
                            )
                        }))
                        .unwrap_or_else(|_| Err(t().interner_fehler.into()));
                        melde(match ergebnis {
                            Ok(()) => Msg::Fertig(a.index),
                            Err(e) if e == ABGEBROCHEN => Msg::Abgebrochen(a.index),
                            Err(e) => Msg::Fehler(a.index, e),
                        });
                    }
                });
            }
        });
        let _ = tx.send(Msg::Ende);
        ctx.request_repaint();
    });
}
