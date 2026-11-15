// Zustand und Ablaufsteuerung: Ordner, Dateiliste, Start, Abbruch.

use crate::convert::{self, finde_ffmpeg, ist_gesperrt, Msg, Ziel};
use crate::scan;
use eframe::egui;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

// Status einer Datei in der Tabelle
#[derive(Clone, PartialEq)]
pub enum Status {
    Bereit,
    Gesperrt,
    Existiert,
    Wartet,
    Laeuft,
    Fertig,
    Fehler(String),
}

// Eine gefundene Audiodatei
pub struct Datei {
    pub pfad: PathBuf,   // vollständiger Pfad der Quelldatei
    pub wurzel: PathBuf, // vom Nutzer gewählter Quellordner
    pub ordner: PathBuf, // Ordner, in dem die Datei liegt (Anzeige + Farbe)
    pub name: String,
    pub endung: String, // kleingeschrieben, z. B. "flac"
    pub gewaehlt: bool, // Häkchen
    pub status: Status,
    pub farbe: usize, // 0 oder 1 (Zeilenfarbe)
}

impl Datei {
    // Quellformat für Tabelle und Protokoll, z. B. "FLAC"
    pub fn von(&self) -> String {
        self.endung.to_uppercase()
    }
    // Zielformat für Tabelle und Protokoll ("-" bei MP3-Quellen, die nie umgewandelt werden)
    pub fn nach(&self, ziel: Ziel) -> &'static str {
        if self.endung == "mp3" {
            "-"
        } else {
            ziel.kurz()
        }
    }
}

pub struct App {
    pub ordner: Vec<PathBuf>, // gewählte Quellordner
    pub dateien: Vec<Datei>,  // alle gefundenen Dateien
    pub ziel: Ziel,
    pub ffmpeg: Option<PathBuf>,
    pub laeuft: bool,
    pub gesamt: usize,   // Anzahl Aufträge im aktuellen Lauf
    pub erledigt: usize, // davon abgeschlossen
    pub protokoll_bereit: bool, // Protokoll gibt es erst nach einem Lauf
    pub meldung: Option<String>, // Hinweis unten, z. B. „Protokoll gespeichert“
    rx: Option<Receiver<Msg>>,
    abbruch: Arc<AtomicBool>,
}

impl App {
    pub fn new() -> Self {
        Self {
            ordner: Vec::new(),
            dateien: Vec::new(),
            ziel: Ziel::Aiff,
            ffmpeg: finde_ffmpeg(),
            laeuft: false,
            gesamt: 0,
            erledigt: 0,
            protokoll_bereit: false,
            meldung: None,
            rx: None,
            abbruch: Arc::new(AtomicBool::new(false)),
        }
    }

    // Ordner samt Unterordnern nach Audiodateien durchsuchen
    pub fn ordner_hinzufuegen(&mut self, wurzel: PathBuf) {
        if self.ordner.contains(&wurzel) {
            return;
        }
        for pfad in scan::audiodateien(&wurzel) {
            // Datei nicht doppelt aufnehmen (z. B. bei verschachtelten Ordnern)
            if self.dateien.iter().any(|d| d.pfad == pfad) {
                continue;
            }
            let endung = scan::endung_von(&pfad);
            self.dateien.push(Datei {
                ordner: pfad.parent().unwrap_or(&wurzel).to_path_buf(),
                name: pfad
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                wurzel: wurzel.clone(),
                pfad,
                endung,
                gewaehlt: false,
                status: Status::Wartet, // wird gleich neu berechnet
                farbe: 0,
            });
        }
        self.ordner.push(wurzel);
        self.neu_ordnen();
    }

    // Quellordner samt seinen Dateien entfernen
    pub fn ordner_entfernen(&mut self, index: usize) {
        let wurzel = self.ordner.remove(index);
        self.dateien.retain(|d| d.wurzel != wurzel);
        self.neu_ordnen();
    }

    // Nach Ordner sortieren und Zeilenfarben vergeben
    fn neu_ordnen(&mut self) {
        self.dateien
            .sort_by(|a, b| (&a.ordner, &a.name).cmp(&(&b.ordner, &b.name)));
        let mut gruppe = 0;
        let mut letzter: Option<PathBuf> = None;
        for d in self.dateien.iter_mut() {
            if letzter.as_ref() != Some(&d.ordner) {
                if letzter.is_some() {
                    gruppe += 1;
                }
                letzter = Some(d.ordner.clone());
            }
            d.farbe = gruppe % 2;
        }
        self.aktualisiere_status();
    }

    // Status bestimmen: gesperrt / existiert bereits / bereit
    pub fn aktualisiere_status(&mut self) {
        self.protokoll_bereit = false; // Protokoll passt nicht mehr zur Liste
        let ziel = self.ziel;
        for d in self.dateien.iter_mut() {
            let neu = if ist_gesperrt(&d.endung, ziel) {
                Status::Gesperrt
            } else if d.pfad.with_extension(ziel.endung()).exists() {
                Status::Existiert
            } else {
                Status::Bereit
            };
            // Häkchen nur zurücksetzen, wenn sich der Status ändert
            if neu != d.status {
                d.gewaehlt = neu == Status::Bereit;
                d.status = neu;
            }
        }
    }

    // Darf der Start-Button aktiv sein?
    pub fn kann_starten(&self) -> bool {
        self.ffmpeg.is_some()
            && !self.laeuft
            && self
                .dateien
                .iter()
                .any(|d| d.gewaehlt && d.status == Status::Bereit)
    }

    // Umwandlung der angehakten Dateien starten
    pub fn starten(&mut self, ctx: &egui::Context) {
        let Some(ffmpeg) = self.ffmpeg.clone() else {
            return;
        };
        let ziel = self.ziel;
        let mut auftraege = Vec::new();
        for (i, d) in self.dateien.iter_mut().enumerate() {
            if d.gewaehlt && d.status == Status::Bereit {
                d.status = Status::Wartet;
                auftraege.push((i, d.pfad.clone(), d.pfad.with_extension(ziel.endung())));
            }
        }
        if auftraege.is_empty() {
            return;
        }

        self.gesamt = auftraege.len();
        self.erledigt = 0;
        self.laeuft = true;
        self.protokoll_bereit = false;
        self.meldung = None;
        self.abbruch = Arc::new(AtomicBool::new(false));

        let (tx, rx) = channel();
        self.rx = Some(rx);
        convert::starte(ffmpeg, auftraege, ziel, self.abbruch.clone(), tx, ctx.clone());
    }

    // Laufende Umwandlung abbrechen
    pub fn abbrechen(&mut self) {
        self.abbruch.store(true, Ordering::Relaxed);
    }

    // Nachrichten des Threads abholen und Tabelle aktualisieren
    pub fn nachrichten_verarbeiten(&mut self) {
        let Some(rx) = &self.rx else { return };
        let nachrichten: Vec<Msg> = rx.try_iter().collect();
        for m in nachrichten {
            match m {
                Msg::Laeuft(i) => self.dateien[i].status = Status::Laeuft,
                Msg::Fertig(i) => {
                    self.dateien[i].status = Status::Fertig;
                    self.erledigt += 1;
                }
                Msg::Fehler(i, e) => {
                    self.dateien[i].status = Status::Fehler(e);
                    self.erledigt += 1;
                }
                Msg::Ende => {
                    self.laeuft = false;
                    self.protokoll_bereit = true;
                    self.rx = None;
                    // Nicht gestartete Dateien (nach Abbruch) wieder freigeben
                    for d in self.dateien.iter_mut() {
                        if d.status == Status::Wartet {
                            d.status = Status::Bereit;
                        }
                    }
                }
            }
        }
    }
}
