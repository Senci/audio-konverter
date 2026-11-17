// Release-Build ohne zusätzliches Konsolenfenster
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod aiff; // AIFF-Datei schreiben
mod app; // Zustand und Ablaufsteuerung
mod bericht; // CSV-Protokoll
mod convert; // Umwandlung starten (Thread, Status)
mod konverter; // eigener Konverter: lesen, umrechnen, runden
mod cover; // Cover-Suche für AIFF-Metadaten
mod gui; // Oberfläche
mod mp3; // MP3-Datei schreiben (LAME)
mod scan; // Ordner durchsuchen
mod tags; // Tags und Cover als ID3

use app::App;
use eframe::egui;

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
