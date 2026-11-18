// Oberfläche: Werkzeugleiste oben, Quellordner links, Dateiliste in der Mitte,
// Statusleiste mit Hauptaktion unten, Fenster „Cover“ bei Bedarf.

use crate::app::{App, Datei, Status};
use crate::konverter::Ziel;
use crate::{bericht, icons};
use eframe::egui::{
    self, vec2, Align, Align2, Checkbox, Color32, FontId, ImageSource, Label, Layout, Margin,
    RichText, Sense, Ui, UiBuilder,
};

const ZEILE: f32 = 22.0; // Zeilenhöhe der Tabelle
const ICON: f32 = 14.0; // Icon-Größe in Knöpfen
const GRUEN: Color32 = Color32::from_rgb(60, 170, 100);
const AKZENT: Color32 = Color32::from_rgb(37, 99, 235); // kräftig genug für Weiß in hell und dunkel

// Feste Spaltenbreiten; Datei und Ordner teilen sich den Rest
const B_HAKEN: f32 = 28.0;
const B_NR: f32 = 44.0;
const B_FORMAT: f32 = 104.0;
const B_STATUS: f32 = 140.0;

// ---------------------------------------------------------------------------
// Stil: kompakt, ruhig, ohne Schatten. Gilt für helles und dunkles Systemthema.
// ---------------------------------------------------------------------------

pub fn stil(ctx: &egui::Context) {
    ctx.all_styles_mut(|s| {
        use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
        s.text_styles = [
            (Heading, FontId::proportional(15.0)),
            (Body, FontId::proportional(13.0)),
            (Button, FontId::proportional(13.0)),
            (Small, FontId::proportional(11.0)),
            (Monospace, FontId::monospace(12.0)),
        ]
        .into();
        s.spacing.item_spacing = vec2(6.0, 4.0);
        s.spacing.button_padding = vec2(8.0, 3.0);
        s.spacing.interact_size.y = ZEILE;
        s.visuals.window_shadow = egui::Shadow::NONE;
        s.visuals.popup_shadow = egui::Shadow::NONE;
        s.visuals.window_rounding = 6.0.into();
        s.visuals.menu_rounding = 4.0.into();
        let w = &mut s.visuals.widgets;
        for z in [
            &mut w.noninteractive,
            &mut w.inactive,
            &mut w.hovered,
            &mut w.active,
            &mut w.open,
        ] {
            z.rounding = 4.0.into();
        }
    });
}

// ---------------------------------------------------------------------------
// Kleine Bausteine
// ---------------------------------------------------------------------------

// Knopf mit Icon und Text in Textfarbe
fn knopf<'a>(ui: &Ui, icon: ImageSource<'static>, text: &'a str) -> egui::Button<'a> {
    egui::Button::image_and_text(icons::bild(icon, ICON, ui.visuals().text_color()), text)
}

// Hauptaktion: gefüllt in Akzentfarbe
fn haupt_knopf<'a>(icon: ImageSource<'static>, text: &'a str) -> egui::Button<'a> {
    egui::Button::image_and_text(
        icons::bild(icon, ICON, Color32::WHITE),
        RichText::new(text).color(Color32::WHITE),
    )
    .fill(AKZENT)
    .min_size(vec2(112.0, ZEILE + 2.0))
}

// Kleine Überschrift, z. B. „QUELLEN“
fn ueberschrift(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text.to_uppercase()).small().strong().weak());
}

// Eine Tabellenzeile über die volle Breite, optional mit Hintergrund
fn zeile(ui: &mut Ui, hintergrund: Option<Color32>, inhalt: impl FnOnce(&mut Ui)) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), ZEILE), Sense::hover());
    if let Some(f) = hintergrund {
        ui.painter().rect_filled(rect, 0.0, f);
    }
    if ui.rect_contains_pointer(rect) {
        let hover = ui
            .visuals()
            .widgets
            .hovered
            .weak_bg_fill
            .gamma_multiply(0.4);
        ui.painter().rect_filled(rect, 0.0, hover);
    }
    let mut kind = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    kind.spacing_mut().item_spacing.x = 0.0;
    inhalt(&mut kind);
}

// Eine Zelle mit fester Breite und kleinem linken Innenabstand
fn zelle(ui: &mut Ui, breite: f32, inhalt: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        vec2(breite, ZEILE),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_width(breite);
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.add_space(6.0);
            inhalt(ui);
        },
    );
}

// Status als Icon und Text in passender Farbe
fn status_anzeige(ui: &mut Ui, status: &Status) {
    let v = ui.visuals();
    let (icon, farbe) = match status {
        Status::Bereit => (Some(icons::BEREIT), v.weak_text_color()),
        Status::Gesperrt => (Some(icons::GESPERRT), v.weak_text_color()),
        Status::Existiert => (Some(icons::EXISTIERT), v.warn_fg_color),
        Status::Wartet => (Some(icons::WARTET), v.weak_text_color()),
        Status::Laeuft => (None, AKZENT),
        Status::Fertig => (Some(icons::FERTIG), GRUEN),
        Status::Fehler(_) => (Some(icons::FEHLER), v.error_fg_color),
    };
    match icon {
        Some(i) => ui.add(icons::bild(i, 13.0, farbe)),
        None => ui.add(egui::Spinner::new().size(12.0).color(farbe)),
    };
    let antwort = ui.label(RichText::new(status.text()).color(farbe));
    if let Status::Fehler(e) = status {
        antwort.on_hover_text(e);
    }
}

// Eine Zeile der Dateiliste
fn dateizeile(ui: &mut Ui, d: &mut Datei, nr: usize, spalten: &[f32; 6], ziel: Ziel, laeuft: bool) {
    let hintergrund = d.gruppe_ungerade.then(|| ui.visuals().faint_bg_color);
    zeile(ui, hintergrund, |ui| {
        if d.status == Status::Gesperrt {
            ui.disable(); // ganze Zeile ausgegraut
        }
        let waehlbar = d.waehlbar() && !laeuft;
        zelle(ui, spalten[0], |ui| {
            ui.add_enabled(waehlbar, Checkbox::without_text(&mut d.gewaehlt));
        });
        zelle(ui, spalten[1], |ui| {
            ui.label(RichText::new(nr.to_string()).weak());
        });
        zelle(ui, spalten[2], |ui| {
            ui.add(Label::new(d.name.as_str()).truncate())
                .on_hover_text(d.pfad.display().to_string());
        });
        zelle(ui, spalten[3], |ui| {
            ui.add(Label::new(RichText::new(d.ordner_anzeige.as_str()).weak()).truncate())
                .on_hover_text(d.ordner.display().to_string());
        });
        zelle(ui, spalten[4], |ui| {
            let schwach = ui.visuals().weak_text_color();
            ui.label(RichText::new(d.von()).weak());
            if d.nach(ziel) != "-" {
                ui.add(icons::bild(icons::PFEIL, 11.0, schwach));
                ui.label(RichText::new(d.nach(ziel)).weak());
            }
        });
        zelle(ui, spalten[5], |ui| status_anzeige(ui, &d.status));
    });
}

// ---------------------------------------------------------------------------
// Bereiche des Fensters
// ---------------------------------------------------------------------------

impl App {
    // Oben: Ordner hinzufügen, Zielformat, Cover, Protokoll
    fn werkzeugleiste(&mut self, ctx: &egui::Context) {
        let rahmen =
            egui::Frame::side_top_panel(&ctx.style()).inner_margin(Margin::symmetric(10.0, 6.0));
        egui::TopBottomPanel::top("werkzeuge")
            .frame(rahmen)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!self.laeuft, |ui| {
                        if ui
                            .add(knopf(ui, icons::ORDNER_PLUS, "Ordner hinzufügen"))
                            .clicked()
                        {
                            for p in rfd::FileDialog::new().pick_folders().unwrap_or_default() {
                                self.ordner_hinzufuegen(p);
                            }
                        }
                        ui.separator();
                        ui.label(RichText::new("Ziel").weak());
                        let mut ziel = self.ziel;
                        egui::ComboBox::from_id_salt("ziel")
                            .selected_text(ziel.name())
                            .width(190.0)
                            .show_ui(ui, |ui| {
                                for z in [Ziel::Aiff, Ziel::Mp3] {
                                    ui.selectable_value(&mut ziel, z, z.name());
                                }
                            });
                        self.ziel_setzen(ziel);
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                self.protokoll_bereit,
                                knopf(ui, icons::PROTOKOLL, "Protokoll"),
                            )
                            .on_hover_text("Protokoll als CSV speichern")
                            .on_disabled_hover_text("Verfügbar nach einer Umwandlung")
                            .clicked()
                        {
                            self.protokoll_speichern();
                        }
                        let fehlen = self.ohne_cover.len();
                        let text = if fehlen == 0 {
                            "Cover".to_string()
                        } else {
                            format!("Cover · {fehlen} fehlen")
                        };
                        if ui
                            .add_enabled(
                                !self.ordner.is_empty(),
                                knopf(ui, icons::OHNE_COVER, &text),
                            )
                            .on_hover_text("Ordner ohne Cover und gefundene Bilder")
                            .clicked()
                        {
                            self.zeige_cover = !self.zeige_cover;
                        }
                    });
                });
            });
    }

    // Links: gewählte Quellordner
    fn quellen(&mut self, ctx: &egui::Context) {
        let rahmen = egui::Frame::side_top_panel(&ctx.style()).inner_margin(Margin::same(10.0));
        egui::SidePanel::left("quellen")
            .frame(rahmen)
            .default_width(220.0)
            .width_range(160.0..=420.0)
            .show(ctx, |ui| {
                ueberschrift(ui, "Quellen");
                let schwach = ui.visuals().weak_text_color();
                let mut entfernen = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, o) in self.ordner.iter().enumerate() {
                        let anzahl = self.dateien.iter().filter(|d| &d.wurzel == o).count();
                        let name = o.file_name().map_or_else(
                            || o.display().to_string(),
                            |n| n.to_string_lossy().to_string(),
                        );
                        ui.horizontal(|ui| {
                            ui.add(icons::bild(icons::ORDNER, ICON, schwach));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let x = egui::Button::image(icons::bild(
                                    icons::ENTFERNEN,
                                    12.0,
                                    schwach,
                                ))
                                .frame(false);
                                if ui
                                    .add_enabled(!self.laeuft, x)
                                    .on_hover_text("Ordner entfernen")
                                    .clicked()
                                {
                                    entfernen = Some(i);
                                }
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    ui.add(Label::new(name).truncate())
                                        .on_hover_text(o.display().to_string());
                                    ui.label(
                                        RichText::new(format!("{anzahl} Dateien")).small().weak(),
                                    );
                                });
                            });
                        });
                        ui.add_space(4.0);
                    }
                });
                if let Some(i) = entfernen {
                    self.ordner_entfernen(i);
                }
            });
    }

    // Unten: Zusammenfassung, Fortschritt, Hauptaktion
    fn statusleiste(&mut self, ctx: &egui::Context) {
        let rahmen =
            egui::Frame::side_top_panel(&ctx.style()).inner_margin(Margin::symmetric(10.0, 6.0));
        egui::TopBottomPanel::bottom("status")
            .frame(rahmen)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let mut info = format!("{} Dateien", self.dateien.len());
                    let gewaehlt = self.anzahl_gewaehlt();
                    if !self.laeuft && gewaehlt > 0 {
                        info = format!("{info} · {gewaehlt} ausgewählt");
                    }
                    if let Some(m) = &self.meldung {
                        info = format!("{info} · {m}");
                    }
                    ui.label(RichText::new(info).weak());

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.laeuft {
                            if ui.add(knopf(ui, icons::STOPP, "Abbrechen")).clicked() {
                                self.abbrechen();
                            }
                            let anteil = self.erledigt as f32 / self.gesamt.max(1) as f32;
                            ui.label(
                                RichText::new(format!("{} / {}", self.erledigt, self.gesamt))
                                    .weak(),
                            );
                            ui.add(
                                egui::ProgressBar::new(anteil)
                                    .desired_width(160.0)
                                    .desired_height(6.0),
                            );
                        } else if ui
                            .add_enabled(
                                self.kann_starten(),
                                haupt_knopf(icons::START, "Umwandeln"),
                            )
                            .on_disabled_hover_text("Keine Datei ausgewählt")
                            .clicked()
                        {
                            self.starten(ctx);
                        }
                    });
                });
            });
    }

    // Mitte: Dateiliste oder Hinweis, wenn noch nichts gewählt ist
    fn liste(&mut self, ctx: &egui::Context) {
        let rahmen =
            egui::Frame::central_panel(&ctx.style()).inner_margin(Margin::symmetric(8.0, 6.0));
        egui::CentralPanel::default().frame(rahmen).show(ctx, |ui| {
            if self.dateien.is_empty() {
                leer(ui);
                return;
            }
            let laeuft = self.laeuft;
            let ziel = self.ziel;
            let rest = (ui.available_width() - B_HAKEN - B_NR - B_FORMAT - B_STATUS).max(200.0);
            let spalten = [B_HAKEN, B_NR, rest * 0.55, rest * 0.45, B_FORMAT, B_STATUS];

            // Kopfzeile mit „alle an/aus“
            let waehlbar = self.anzahl_waehlbar();
            let gewaehlt = self.anzahl_gewaehlt();
            zeile(ui, None, |ui| {
                zelle(ui, spalten[0], |ui| {
                    let mut alle = waehlbar > 0 && gewaehlt == waehlbar;
                    let kasten = Checkbox::without_text(&mut alle)
                        .indeterminate(gewaehlt > 0 && gewaehlt < waehlbar);
                    if ui
                        .add_enabled(waehlbar > 0 && !laeuft, kasten)
                        .on_hover_text("Alle an- oder abwählen")
                        .changed()
                    {
                        self.alle_waehlen(alle);
                    }
                });
                for (titel, breite) in ["Nr", "Datei", "Ordner", "Format", "Status"]
                    .into_iter()
                    .zip(&spalten[1..])
                {
                    zelle(ui, *breite, |ui| {
                        ui.label(RichText::new(titel).small().strong().weak());
                    });
                }
            });
            ui.add(egui::Separator::default().spacing(2.0));

            // Nur sichtbare Zeilen zeichnen, auch bei tausenden Dateien flüssig
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::ScrollArea::vertical().auto_shrink(false).show_rows(
                ui,
                ZEILE,
                self.dateien.len(),
                |ui, bereich| {
                    for i in bereich {
                        dateizeile(ui, &mut self.dateien[i], i + 1, &spalten, ziel, laeuft);
                    }
                },
            );
        });
    }

    // Fenster „Cover“: Ordner ohne Bild und alle gefundenen Bilder
    fn cover_fenster(&mut self, ctx: &egui::Context) {
        if !self.zeige_cover {
            return;
        }
        let mut offen = true;
        egui::Window::new("Cover")
            .open(&mut offen)
            .collapsible(false)
            .default_size([480.0, 360.0])
            .show(ctx, |ui| {
                let v = ui.visuals();
                let (warn, schwach) = (v.warn_fg_color, v.weak_text_color());
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ueberschrift(ui, &format!("Ordner ohne Cover · {}", self.ohne_cover.len()));
                    if self.ohne_cover.is_empty() {
                        ui.label(RichText::new("Alle Ordner mit Audiodateien haben ein Bild.").weak());
                    }
                    for (pfad, anzeige) in &self.ohne_cover {
                        ui.horizontal(|ui| {
                            ui.add(icons::bild(icons::OHNE_COVER, ICON, warn));
                            ui.add(Label::new(anzeige.as_str()).truncate())
                                .on_hover_text(pfad.display().to_string());
                        });
                    }
                    ui.add_space(10.0);
                    ueberschrift(ui, &format!("Gefundene Bilder · {}", self.bilder.len()));
                    ui.label(
                        RichText::new(
                            "Werden nie umgewandelt. Bei AIFF wird das erste Bild eines Ordners als Cover eingebettet.",
                        )
                        .small()
                        .weak(),
                    );
                    for b in &self.bilder {
                        ui.horizontal(|ui| {
                            ui.add(icons::bild(icons::BILD, ICON, schwach));
                            ui.label(b.name.as_str());
                            ui.add(Label::new(RichText::new(b.ordner_anzeige.as_str()).weak()).truncate())
                                .on_hover_text(b.pfad.display().to_string());
                        });
                    }
                });
            });
        self.zeige_cover = offen;
    }

    // Ordner per Drag & Drop aufnehmen; beim Ziehen Hinweis über dem Fenster
    fn ablegen(&mut self, ctx: &egui::Context) {
        let (abgelegt, schwebt): (Vec<_>, bool) = ctx.input(|i| {
            (
                i.raw
                    .dropped_files
                    .iter()
                    .filter_map(|f| f.path.clone())
                    .collect(),
                !i.raw.hovered_files.is_empty(),
            )
        });
        for p in abgelegt {
            if p.is_dir() {
                self.ordner_hinzufuegen(p);
            } else {
                self.meldung = Some("Nur Ordner können abgelegt werden".into());
            }
        }
        if schwebt && !self.laeuft {
            let maler = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("ablegen"),
            ));
            let rect = ctx.screen_rect();
            maler.rect_filled(rect, 0.0, Color32::from_black_alpha(150));
            maler.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Ordner hier ablegen",
                FontId::proportional(18.0),
                Color32::WHITE,
            );
        }
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
            Err(e) => format!("Protokoll nicht gespeichert: {e}"),
        });
    }
}

// Hinweis bei leerer Liste
fn leer(ui: &mut Ui) {
    let schwach = ui.visuals().weak_text_color();
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.3);
        ui.add(icons::bild(icons::ORDNER_OFFEN, 40.0, schwach));
        ui.add_space(8.0);
        ui.label(RichText::new("Ordner hinzufügen oder hierher ziehen").weak());
        ui.label(
            RichText::new("FLAC, WAV und AIFF werden zu AIFF oder MP3")
                .small()
                .weak(),
        );
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.nachrichten_verarbeiten();
        self.ablegen(ctx);
        self.werkzeugleiste(ctx);
        self.statusleiste(ctx);
        self.quellen(ctx);
        self.liste(ctx);
        self.cover_fenster(ctx);
    }
}
