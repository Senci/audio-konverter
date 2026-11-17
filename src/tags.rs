// Tags der Quelldatei (Titel, Künstler ...) und Cover als ID3v2.3-Tag.
// Bei AIFF kommt der Tag als "ID3 "-Chunk in die Datei, bei MP3 an den Dateianfang.

use id3::frame::{Content, Picture, PictureType};
use id3::{Frame, Tag, TagLike, Version};
use std::path::Path;
use symphonia::core::meta::{StandardTagKey as K, Tag as QuellTag};

// Übernommene Felder: Schlüssel der Quelle -> ID3-Frame
fn frame_id(k: K) -> Option<&'static str> {
    Some(match k {
        K::TrackTitle => "TIT2",
        K::Artist => "TPE1",
        K::AlbumArtist => "TPE2",
        K::Album => "TALB",
        K::Genre => "TCON",
        K::Date => "TYER",
        K::TrackNumber => "TRCK",
        K::DiscNumber => "TPOS",
        K::Composer => "TCOM",
        K::Bpm => "TBPM",
        K::Label => "TPUB",
        _ => return None,
    })
}

// Fertigen ID3-Tag als Bytes erzeugen. Leer, wenn es nichts zu schreiben gibt.
pub fn erzeuge(quelle: &[QuellTag], bild: Option<&Path>) -> Vec<u8> {
    let mut tag = Tag::new();

    for t in quelle {
        let Some(id) = t.std_key.and_then(frame_id) else {
            continue;
        };
        let mut text = t.value.to_string();
        if id == "TYER" {
            text = text.chars().take(4).collect(); // ID3v2.3 kennt nur das Jahr
        }
        if !text.is_empty() {
            tag.set_text(id, text);
        }
    }

    // Cover einbetten (Bytes der Bilddatei unverändert)
    if let Some(pfad) = bild {
        if let Ok(daten) = std::fs::read(pfad) {
            let mime = match pfad
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_lowercase)
                .as_deref()
            {
                Some("png") => "image/png",
                Some("bmp") => "image/bmp",
                _ => "image/jpeg",
            };
            tag.add_frame(Frame::with_content(
                "APIC",
                Content::Picture(Picture {
                    mime_type: mime.to_string(),
                    picture_type: PictureType::CoverFront,
                    description: String::new(),
                    data: daten,
                }),
            ));
        }
    }

    let mut aus = Vec::new();
    if tag.frames().next().is_some() {
        let _ = tag.write_to(&mut aus, Version::Id3v23);
    }
    aus
}
