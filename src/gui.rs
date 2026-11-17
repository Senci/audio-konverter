// Oberfläche: Seitenleiste, Tabelle, Fußleiste, Bilderliste, Fenster „Ordner ohne Bild“.

use crate::app::{App, Status};
use crate::bericht;
use crate::convert::Ziel;
use eframe::egui::{self, Color32};

// Spalten der Tabelle: (Titel, Breite)
const SPALTEN: [(&str, f32); 6] = [
    ("Nr", 50.0),
    ("Datei", 250.0),
    ("Ordner / Pfad", 380.0),
    ("Von", 70.0),
    ("Nach", 70.0),
    ("Status", 130.0),
];

// Zeilenfarben: gleicher Ordner = gleiche Farbe, nächster Ordner wechselt
const FARBE_A: Color32 = Color32::from_rgba_premultiplied(4, 14, 30, 30);
const FARBE_B: Color32 = Color32::from_rgba_premultiplied(3, 20, 9, 30);

// Anzeigetext und Farbe eines Status (der Text wird auch im Protokoll verwendet)
pub fn status_text(s: &Status) -> &'static str {
    match s {
        Status::Bereit => "bereit",
        Status::Gesperrt => "gesperrt",
        Status::Existiert => "existiert bereits",
        Status::Wartet => "wartet",
        Status::Laeuft => "läuft ...",
        Status::Fertig => "fertig",
        Status::Fehler(_) => "Fehler",
    }
}

fn status_farbe(s: &Status) -> Color32 {
    match s {
        Status::Existiert => Color32::from_rgb(230, 160, 0),
        Status::Laeuft => Color32::from_rgb(80, 140, 255),
        Status::Fertig => Color32::from_rgb(40, 170, 80),
        Status::Fehler(_) => Color32::from_rgb(220, 60, 60),
        _ => Color32::GRAY,
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

impl App {
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

    // Protokoll als CSV speichern (Dialog „Speichern unter“)
    fn protokoll_speichern(&mut self) {
        let Some(pfad) = rfd::FileDialog::new()
            .set_file_name("protokoll.csv")
            .add_filter("CSV", &["csv"])
            .save_file()
        else {
            return;
        };
        self.meldung = Some(match bericht::speichern(&self.dateien, self.ziel, &pfad) {
            Ok(()) => format!("Protokoll gespeichert: {}", pfad.display()),
            Err(e) => format!("Protokoll konnte nicht gespeichert werden: {e}"),
        });
    }

    // Unten: Start, Abbrechen, Protokoll, Fortschritt
    fn fussleiste(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("unten").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let kann_starten = self.kann_starten();
                if ui
                    .add_enabled(kann_starten, egui::Button::new("Konvertierung starten"))
                    .clicked()
                {
                    self.starten(ctx);
                }
                if ui
                    .add_enabled(self.laeuft, egui::Button::new("Abbrechen"))
                    .clicked()
                {
                    self.abbrechen();
                }
                // Protokoll erst nach einem Lauf möglich
                if ui
                    .add_enabled(
                        self.protokoll_bereit,
                        egui::Button::new("Protokoll herunterladen (CSV)"),
                    )
                    .clicked()
                {
                    self.protokoll_speichern();
                }
                // Fenster mit Ordnern ohne Bild (Punkt 4)
                let anzahl_ohne_bild = self.ordner_ohne_bild().len();
                if ui
                    .button(format!("Ordner ohne Bild · {anzahl_ohne_bild}"))
                    .clicked()
                {
                    self.zeige_ordner_ohne_bild = true;
                }
                let anteil = if self.gesamt == 0 {
                    0.0
                } else {
                    self.erledigt as f32 / self.gesamt as f32
                };
                ui.add(egui::ProgressBar::new(anteil).desired_width(200.0));
                ui.label(format!("{} / {}", self.erledigt, self.gesamt));
            });
            if let Some(m) = &self.meldung {
                ui.label(m.as_str());
            }
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

    // Extra-Fenster: Ordner, die Audiodateien aber kein Bild (Cover) enthalten
    fn fenster_ordner_ohne_bild(&mut self, ctx: &egui::Context) {
        if !self.zeige_ordner_ohne_bild {
            return;
        }
        let ordner = self.ordner_ohne_bild();
        let mut offen = true;
        egui::Window::new("Ordner ohne Bild")
            .open(&mut offen)
            .default_width(420.0)
            .show(ctx, |ui| {
                if ordner.is_empty() {
                    ui.label("Alle Ordner mit Audiodateien enthalten ein Bild.");
                } else {
                    ui.label(format!("{} Ordner ohne Bild (Cover):", ordner.len()));
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for o in ordner {
                            ui.label(o.display().to_string());
                        }
                    });
                }
            });
        self.zeige_ordner_ohne_bild = offen;
    }

    // Mitte: Tabelle der Dateien
    fn tabelle(&mut self, ctx: &egui::Context) {
        let laeuft = self.laeuft;
        let ziel = self.ziel;
        egui::CentralPanel::default().show(ctx, |ui| {
            let gewaehlt = self.dateien.iter().filter(|d| d.gewaehlt).count();
            ui.horizontal(|ui| {
                ui.strong("Zu konvertierende Dateien");
                // Gesamtzahl = höchste Nr in der Tabelle
                ui.label(format!(
                    "{} Dateien · {} ausgewählt",
                    self.dateien.len(),
                    gewaehlt
                ));
            });
            ui.separator();

            if !self.bilder.is_empty() {
                egui::CollapsingHeader::new(format!(
                    "Gefundene Bilder · {} (werden nie konvertiert)",
                    self.bilder.len()
                ))
                .default_open(false)
                .show(ui, |ui| {
                    for (i, b) in self.bilder.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.weak(format!("{}.", i + 1));
                            ui.label(b.name.as_str());
                            ui.weak(b.ordner.display().to_string());
                        });
                    }
                });
                ui.separator();
            }

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

                    // Datenzeilen, Nr zählt ab 1
                    for (i, d) in self.dateien.iter_mut().enumerate() {
                        let farbe = if d.farbe == 0 { FARBE_A } else { FARBE_B };
                        let aktiv = d.status == Status::Bereit && !laeuft;
                        let von = d.von();
                        let nach = d.nach(ziel);
                        zeile(ui, farbe, |ui| {
                            // Gesperrte Zeilen komplett ausgegraut
                            ui.add_enabled_ui(d.status != Status::Gesperrt, |ui| {
                                // Spalte 1: Nr
                                zelle(ui, SPALTEN[0].1, |ui| {
                                    ui.weak((i + 1).to_string());
                                });
                                // Spalte 2: Häkchen + Dateiname
                                zelle(ui, SPALTEN[1].1, |ui| {
                                    ui.add_enabled(
                                        aktiv,
                                        egui::Checkbox::without_text(&mut d.gewaehlt),
                                    );
                                    ui.add(egui::Label::new(d.name.as_str()).truncate());
                                });
                                // Spalte 3: Ordner / Pfad
                                zelle(ui, SPALTEN[2].1, |ui| {
                                    let pfad = d.ordner.display().to_string();
                                    ui.add(egui::Label::new(pfad.as_str()).truncate())
                                        .on_hover_text(pfad.as_str());
                                });
                                // Spalte 4: Von
                                zelle(ui, SPALTEN[3].1, |ui| {
                                    ui.label(von.as_str());
                                });
                                // Spalte 5: Nach
                                zelle(ui, SPALTEN[4].1, |ui| {
                                    ui.label(nach);
                                });
                                // Spalte 6: Status
                                zelle(ui, SPALTEN[5].1, |ui| {
                                    let r = ui.label(
                                        egui::RichText::new(status_text(&d.status))
                                            .color(status_farbe(&d.status)),
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
        self.fenster_ordner_ohne_bild(ctx);
    }
}
