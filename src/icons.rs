// Icons aus dem Iconify-Paket Lucide (ISC-Lizenz, siehe assets/icons/README.md).
// Die SVGs sind weiß und werden beim Zeichnen in der passenden Farbe eingefärbt.

use eframe::egui::{self, include_image, Color32, ImageSource};

pub const ORDNER_PLUS: ImageSource<'static> = include_image!("../assets/icons/folder-plus.svg");
pub const ORDNER: ImageSource<'static> = include_image!("../assets/icons/folder.svg");
pub const ORDNER_OFFEN: ImageSource<'static> = include_image!("../assets/icons/folder-open.svg");
pub const ENTFERNEN: ImageSource<'static> = include_image!("../assets/icons/x.svg");
pub const START: ImageSource<'static> = include_image!("../assets/icons/play.svg");
pub const STOPP: ImageSource<'static> = include_image!("../assets/icons/square.svg");
pub const PROTOKOLL: ImageSource<'static> = include_image!("../assets/icons/file-down.svg");
pub const OHNE_COVER: ImageSource<'static> = include_image!("../assets/icons/image-off.svg");
pub const PFEIL: ImageSource<'static> = include_image!("../assets/icons/arrow-right.svg");
pub const BILD: ImageSource<'static> = include_image!("../assets/icons/image.svg");
pub const SPRACHE: ImageSource<'static> = include_image!("../assets/icons/languages.svg");
pub const BEREIT: ImageSource<'static> = include_image!("../assets/icons/circle.svg");
pub const GESPERRT: ImageSource<'static> = include_image!("../assets/icons/lock.svg");
pub const EXISTIERT: ImageSource<'static> = include_image!("../assets/icons/file-check.svg");
pub const WARTET: ImageSource<'static> = include_image!("../assets/icons/clock.svg");
pub const FERTIG: ImageSource<'static> = include_image!("../assets/icons/circle-check.svg");
pub const FEHLER: ImageSource<'static> = include_image!("../assets/icons/circle-alert.svg");

// Icon in Größe `groesse` und Farbe `farbe`
pub fn bild(quelle: ImageSource<'static>, groesse: f32, farbe: Color32) -> egui::Image<'static> {
    egui::Image::new(quelle)
        .fit_to_exact_size(egui::vec2(groesse, groesse))
        .tint(farbe)
}
