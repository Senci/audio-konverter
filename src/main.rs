// Release-Build ohne zusätzliches Konsolenfenster
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app; // Zustand und Ablaufsteuerung
mod bericht; // Protokoll (noch leer)
mod convert; // Umwandlung (ffmpeg)
mod gui; // Oberfläche
mod scan; // Ordner durchsuchen

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
