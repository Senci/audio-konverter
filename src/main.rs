// Release-Build ohne zusätzliches Konsolenfenster
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod aiff; // AIFF-Datei schreiben
mod app; // Zustand und Ablaufsteuerung
mod auftraege; // Umwandlungen parallel im Hintergrund
mod bericht; // CSV-Protokoll
mod gui; // Oberfläche
mod icons; // Lucide-Icons
mod konverter; // eigener Konverter: lesen, umrechnen, runden
mod mp3; // MP3-Datei schreiben (LAME)
mod scan; // Ordner durchsuchen
mod tags; // Tags und Cover als ID3

use eframe::egui;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    // Ordner aus der Kommandozeile übernehmen (z. B. Verknüpfung oder „Öffnen mit“)
    let ordner: Vec<PathBuf> = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .collect();

    let optionen = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_min_inner_size([720.0, 400.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Audio-Konverter",
        optionen,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            gui::stil(&cc.egui_ctx);
            let mut app = app::App::default();
            for o in ordner {
                app.ordner_hinzufuegen(o);
            }
            Ok(Box::new(app))
        }),
    )
}
