// Release-Build ohne zusätzliches Konsolenfenster
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;
use walkdir::WalkDir;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// ---------------------------------------------------------------------------
// Konstanten
// ---------------------------------------------------------------------------

// Erkannte Quellformate
const ENDUNGEN: [&str; 5] = ["flac", "wav", "aiff", "aif", "mp3"];

// Spalten der Tabelle: (Titel, Breite)
const SPALTEN: [(&str, f32); 4] = [
    ("Datei", 260.0),
    ("Ordner / Pfad", 420.0),
    ("Von nach", 110.0),
    ("Status", 130.0),
];

// Zeilenfarben: gleicher Ordner = gleiche Farbe, nächster Ordner wechselt
const FARBE_A: Color32 = Color32::from_rgba_premultiplied(4, 14, 30, 30);
const FARBE_B: Color32 = Color32::from_rgba_premultiplied(3, 20, 9, 30);

// ---------------------------------------------------------------------------
// Datentypen
// ---------------------------------------------------------------------------

// Zielformat (Dropdown)
#[derive(Clone, Copy, PartialEq)]
enum Ziel {
    Aiff,
    Mp3,
}

impl Ziel {
    fn name(self) -> &'static str {
        match self {
            Ziel::Aiff => "AIFF · 16 Bit · 44,1 kHz",
            Ziel::Mp3 => "MP3 · 320 kbit/s · 44,1 kHz",
        }
    }
    fn kurz(self) -> &'static str {
        match self {
            Ziel::Aiff => "AIFF",
            Ziel::Mp3 => "MP3",
        }
    }
    fn endung(self) -> &'static str {
        match self {
            Ziel::Aiff => "aiff",
            Ziel::Mp3 => "mp3",
        }
    }
}

// Status einer Datei in der Tabelle
#[derive(Clone, PartialEq)]
enum Status {
    Bereit,
    Gesperrt,
    Existiert,
    Wartet,
    Laeuft,
    Fertig,
    Fehler(String),
}

impl Status {
    fn text(&self) -> &'static str {
        match self {
            Status::Bereit => "bereit",
            Status::Gesperrt => "gesperrt",
            Status::Existiert => "existiert bereits",
            Status::Wartet => "wartet",
            Status::Laeuft => "läuft ...",
            Status::Fertig => "fertig",
            Status::Fehler(_) => "Fehler",
        }
    }
    fn farbe(&self) -> Color32 {
        match self {
            Status::Existiert => Color32::from_rgb(230, 160, 0),
            Status::Laeuft => Color32::from_rgb(80, 140, 255),
            Status::Fertig => Color32::from_rgb(40, 170, 80),
            Status::Fehler(_) => Color32::from_rgb(220, 60, 60),
            _ => Color32::GRAY,
        }
    }
}

// Eine gefundene Audiodatei
struct Datei {
    pfad: PathBuf,   // vollständiger Pfad der Quelldatei
    wurzel: PathBuf, // vom Nutzer gewählter Quellordner
    ordner: PathBuf, // Ordner, in dem die Datei liegt (Anzeige + Farbe)
    name: String,
    endung: String, // kleingeschrieben, z. B. "flac"
    gewaehlt: bool, // Häkchen
    status: Status,
    farbe: usize, // 0 oder 1 (Zeilenfarbe)
}

// Nachrichten vom Umwandlungs-Thread an die Oberfläche
enum Msg {
    Laeuft(usize),
    Fertig(usize),
    Fehler(usize, String),
    Ende,
}

// ---------------------------------------------------------------------------
// Hilfsfunktionen
// ---------------------------------------------------------------------------

// MP3 ist immer gesperrt, AIFF nur beim Ziel AIFF
fn ist_gesperrt(endung: &str, ziel: Ziel) -> bool {
    endung == "mp3" || (ziel == Ziel::Aiff && (endung == "aiff" || endung == "aif"))
}

// ffmpeg suchen: erst neben der Programmdatei, dann im Arbeitsordner
// (beim Start mit `cargo run` ist das der Projektordner)
fn finde_ffmpeg() -> Option<PathBuf> {
    let name = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
    let mut kandidaten = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            kandidaten.push(dir.join(name));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        kandidaten.push(cwd.join(name));
    }
    kandidaten.into_iter().find(|p| p.exists())
}

// Eine Datei mit ffmpeg umwandeln (läuft im Hintergrund-Thread)
fn konvertiere(
    ffmpeg: &Path,
    quelle: &Path,
    ziel: &Path,
    format: Ziel,
    abbruch: &AtomicBool,
) -> Result<(), String> {
    // Vorhandene Dateien niemals anfassen
    if ziel.exists() {
        return Err("Zieldatei existiert bereits".into());
    }

    let mut cmd = Command::new(ffmpeg);
    cmd.args(["-n", "-nostdin", "-loglevel", "error", "-i"])
        .arg(quelle)
        .arg("-vn"); // Cover-Bilder weglassen

    match format {
        // AIFF 16 Bit, 44,1 kHz (mit Dither beim Runterrechnen von 24 Bit)
        Ziel::Aiff => cmd.args([
            "-af",
            "aresample=osr=44100:osf=s16:dither_method=triangular",
            "-c:a",
            "pcm_s16be",
            "-write_id3v2",
            "1",
        ]),
        // MP3 320 kbit/s, 44,1 kHz, ID3v2.3 (am besten kompatibel)
        Ziel::Mp3 => cmd.args([
            "-ar",
            "44100",
            "-c:a",
            "libmp3lame",
            "-b:a",
            "320k",
            "-id3v2_version",
            "3",
        ]),
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

// Eine Zeile mit Hintergrundfarbe
fn zeile(ui: &mut egui::Ui, farbe: Color32, inhalt: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(farbe)
        .inner_margin(egui::Margin::symmetric(4.0, 3.0))
        .show(ui, |ui| {
            ui.horizontal(inhalt);
        });
}

// Eine Tabellenzelle mit fester Breite
fn zelle(ui: &mut egui::Ui, breite: f32, inhalt: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(breite, 20.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(breite);
            inhalt(ui);
        },
    );
}

// ---------------------------------------------------------------------------
// Programm
// ---------------------------------------------------------------------------

struct App {
    ordner: Vec<PathBuf>, // gewählte Quellordner
    dateien: Vec<Datei>,  // alle gefundenen Dateien
    ziel: Ziel,
    ffmpeg: Option<PathBuf>,
    laeuft: bool,
    gesamt: usize,   // Anzahl Aufträge im aktuellen Lauf
    erledigt: usize, // davon abgeschlossen
    rx: Option<Receiver<Msg>>,
    abbruch: Arc<AtomicBool>,
}

impl App {
    fn new() -> Self {
        Self {
            ordner: Vec::new(),
            dateien: Vec::new(),
            ziel: Ziel::Aiff,
            ffmpeg: finde_ffmpeg(),
            laeuft: false,
            gesamt: 0,
            erledigt: 0,
            rx: None,
            abbruch: Arc::new(AtomicBool::new(false)),
        }
    }

    // Ordner samt Unterordnern nach Audiodateien durchsuchen
    fn ordner_hinzufuegen(&mut self, wurzel: PathBuf) {
        if self.ordner.contains(&wurzel) {
            return;
        }
        for eintrag in WalkDir::new(&wurzel)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let pfad = eintrag.path().to_path_buf();
            let endung = pfad
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if !ENDUNGEN.contains(&endung.as_str()) {
                continue;
            }
            // Datei nicht doppelt aufnehmen (z. B. bei verschachtelten Ordnern)
            if self.dateien.iter().any(|d| d.pfad == pfad) {
                continue;
            }
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
    fn ordner_entfernen(&mut self, index: usize) {
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
    fn aktualisiere_status(&mut self) {
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

    // Umwandlung im Hintergrund-Thread starten
    fn starten(&mut self, ctx: &egui::Context, ffmpeg: PathBuf) {
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
        self.abbruch = Arc::new(AtomicBool::new(false));

        let (tx, rx) = channel();
        self.rx = Some(rx);
        let abbruch = self.abbruch.clone();
        let ctx = ctx.clone();

        std::thread::spawn(move || {
            for (i, quelle, ziel_pfad) in auftraege {
                if abbruch.load(Ordering::Relaxed) {
                    break;
                }
                let _ = tx.send(Msg::Laeuft(i));
                ctx.request_repaint();
                let msg = match konvertiere(&ffmpeg, &quelle, &ziel_pfad, ziel, &abbruch) {
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

    // Nachrichten des Threads abholen und Tabelle aktualisieren
    fn nachrichten_verarbeiten(&mut self) {
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

    // Linke Seite: Quellordner und Zielformat
    fn seitenleiste(&mut self, ctx: &egui::Context) {
        let laeuft = self.laeuft;
        egui::SidePanel::left("links")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.add_enabled_ui(!laeuft, |ui| {
                    ui.add_space(6.0);
                    ui.strong("Quellordner");
                    if ui.button("+ Ordner hinzufügen …").clicked() {
                        if let Some(p) = rfd::FileDialog::new().pick_folder() {
                            self.ordner_hinzufuegen(p);
                        }
                    }

                    let mut entfernen = None;
                    for (i, o) in self.ordner.iter().enumerate() {
                        let anzahl = self.dateien.iter().filter(|d| &d.wurzel == o).count();
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.add(
                                        egui::Label::new(o.display().to_string()).truncate(),
                                    );
                                    ui.weak(format!("inkl. Unterordner · {anzahl} Dateien"));
                                });
                                if ui.button("x").clicked() {
                                    entfernen = Some(i);
                                }
                            });
                        });
                    }
                    if let Some(i) = entfernen {
                        self.ordner_entfernen(i);
                    }

                    ui.add_space(12.0);
                    ui.strong("Zielformat");
                    let alt = self.ziel;
                    egui::ComboBox::from_label("")
                        .selected_text(self.ziel.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.ziel, Ziel::Aiff, Ziel::Aiff.name());
                            ui.selectable_value(&mut self.ziel, Ziel::Mp3, Ziel::Mp3.name());
                        });
                    if self.ziel != alt {
                        self.aktualisiere_status();
                    }
                    ui.weak("Speicherort: Ordner der Quelldatei.");
                    ui.weak("MP3-Quellen sind immer gesperrt, AIFF-Quellen bei Ziel AIFF.");
                });
            });
    }

    // Unten: Start, Abbrechen, Fortschritt
    fn fussleiste(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("unten").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let kann_starten = self.ffmpeg.is_some()
                    && !self.laeuft
                    && self
                        .dateien
                        .iter()
                        .any(|d| d.gewaehlt && d.status == Status::Bereit);
                if ui
                    .add_enabled(kann_starten, egui::Button::new("Konvertierung starten"))
                    .clicked()
                {
                    if let Some(f) = self.ffmpeg.clone() {
                        self.starten(ctx, f);
                    }
                }
                if ui
                    .add_enabled(self.laeuft, egui::Button::new("Abbrechen"))
                    .clicked()
                {
                    self.abbruch.store(true, Ordering::Relaxed);
                }
                let anteil = if self.gesamt == 0 {
                    0.0
                } else {
                    self.erledigt as f32 / self.gesamt as f32
                };
                ui.add(egui::ProgressBar::new(anteil).desired_width(300.0));
                ui.label(format!("{} / {}", self.erledigt, self.gesamt));
            });
            if self.ffmpeg.is_none() {
                ui.colored_label(
                    Color32::from_rgb(220, 60, 60),
                    "ffmpeg nicht gefunden: bitte ffmpeg.exe in den Projektordner (neben Cargo.toml) \
                     oder neben die Programmdatei legen.",
                );
            }
            ui.add_space(6.0);
        });
    }

    // Mitte: Tabelle der Dateien
    fn tabelle(&mut self, ctx: &egui::Context) {
        let laeuft = self.laeuft;
        let ziel = self.ziel;
        egui::CentralPanel::default().show(ctx, |ui| {
            let gewaehlt = self.dateien.iter().filter(|d| d.gewaehlt).count();
            ui.horizontal(|ui| {
                ui.strong("Zu konvertierende Dateien");
                ui.label(format!(
                    "{} von {} ausgewählt",
                    gewaehlt,
                    self.dateien.len()
                ));
            });
            ui.separator();

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    // Kopfzeile
                    zeile(ui, Color32::TRANSPARENT, |ui| {
                        for (titel, breite) in SPALTEN {
                            zelle(ui, breite, |ui| {
                                ui.strong(titel);
                            });
                        }
                    });

                    // Datenzeilen
                    for d in self.dateien.iter_mut() {
                        let farbe = if d.farbe == 0 { FARBE_A } else { FARBE_B };
                        let aktiv = d.status == Status::Bereit && !laeuft;
                        let von_nach = if d.endung == "mp3" {
                            "MP3 nach -".to_string()
                        } else {
                            format!("{} nach {}", d.endung.to_uppercase(), ziel.kurz())
                        };
                        zeile(ui, farbe, |ui| {
                            // Gesperrte Zeilen komplett ausgegraut
                            ui.add_enabled_ui(d.status != Status::Gesperrt, |ui| {
                                // Spalte 1: Häkchen + Dateiname
                                zelle(ui, SPALTEN[0].1, |ui| {
                                    ui.add_enabled(
                                        aktiv,
                                        egui::Checkbox::without_text(&mut d.gewaehlt),
                                    );
                                    ui.add(egui::Label::new(d.name.as_str()).truncate());
                                });
                                // Spalte 2: Ordner / Pfad
                                zelle(ui, SPALTEN[1].1, |ui| {
                                    let pfad = d.ordner.display().to_string();
                                    ui.add(egui::Label::new(pfad.as_str()).truncate())
                                        .on_hover_text(pfad.as_str());
                                });
                                // Spalte 3: Von nach
                                zelle(ui, SPALTEN[2].1, |ui| {
                                    ui.label(von_nach.as_str());
                                });
                                // Spalte 4: Status
                                zelle(ui, SPALTEN[3].1, |ui| {
                                    let r = ui.label(
                                        egui::RichText::new(d.status.text())
                                            .color(d.status.farbe()),
                                    );
                                    if let Status::Fehler(e) = &d.status {
                                        r.on_hover_text(e.as_str());
                                    }
                                });
                            });
                        });
                    }
                });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.nachrichten_verarbeiten();
        self.seitenleiste(ctx);
        self.fussleiste(ctx);
        self.tabelle(ctx);
    }
}

fn main() -> eframe::Result<()> {
    let optionen = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Audio-Konverter",
        optionen,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
