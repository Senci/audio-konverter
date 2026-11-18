// MP3-Schreiber: LAME ist im Programm eingebaut (kein ffmpeg nötig).
// 320 kbit/s, 44,1 kHz. Ein ID3-Tag kommt vor die MP3-Daten.

use crate::texte::t;
use mp3lame_encoder::{
    max_required_buffer_size, Bitrate, Builder, Encoder, FlushNoGap, InterleavedPcm, MonoPcm,
    Quality,
};
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct Mp3Schreiber {
    datei: BufWriter<File>,
    enc: Encoder,
    kanaele: u8,
    puffer: Vec<u8>, // wiederverwendeter Ausgabepuffer
}

fn fehler(e: impl std::fmt::Debug) -> String {
    format!("{}: {e:?}", t().mp3_encoder)
}

impl Mp3Schreiber {
    pub fn neu(datei: File, kanaele: u8, id3: &[u8]) -> Result<Self, String> {
        let mut b = Builder::new().ok_or(t().mp3_encoder_start)?;
        b.set_num_channels(kanaele).map_err(fehler)?;
        b.set_sample_rate(44_100).map_err(fehler)?;
        b.set_brate(Bitrate::Kbps320).map_err(fehler)?;
        b.set_quality(Quality::NearBest).map_err(fehler)?;
        let enc = b.build().map_err(fehler)?;

        let mut datei = BufWriter::new(datei);
        datei.write_all(id3).map_err(|e| e.to_string())?; // leer = nichts
        Ok(Self {
            datei,
            enc,
            kanaele,
            puffer: Vec::new(),
        })
    }

    // Interleavte 16-Bit-Samples kodieren und schreiben
    pub fn schreibe(&mut self, pcm: &[i16]) -> Result<(), String> {
        self.puffer.clear();
        self.puffer.reserve(max_required_buffer_size(pcm.len()));
        let n = if self.kanaele == 1 {
            self.enc
                .encode(MonoPcm(pcm), self.puffer.spare_capacity_mut())
        } else {
            self.enc
                .encode(InterleavedPcm(pcm), self.puffer.spare_capacity_mut())
        }
        .map_err(fehler)?;
        // SAFETY: `puffer` ist leer und hat mindestens `max_required_buffer_size` freien Platz.
        // `encode` meldet in `n`, wie viele Bytes davon beschrieben wurden (n <= Kapazität);
        // genau diese Bytes sind initialisiert. Muster aus der Dokumentation der Crate.
        unsafe { self.puffer.set_len(n) };
        self.datei
            .write_all(&self.puffer)
            .map_err(|e| e.to_string())
    }

    // Reste aus dem Encoder holen und Datei abschließen
    pub fn fertig(mut self) -> Result<(), String> {
        self.puffer.clear();
        self.puffer.reserve(max_required_buffer_size(0));
        let n = self
            .enc
            .flush::<FlushNoGap>(self.puffer.spare_capacity_mut())
            .map_err(fehler)?;
        // SAFETY: wie in `schreibe`: leerer Puffer mit reservierter Kapazität, `flush` meldet
        // in `n` die beschriebenen Bytes.
        unsafe { self.puffer.set_len(n) };
        self.datei
            .write_all(&self.puffer)
            .map_err(|e| e.to_string())?;
        self.datei.flush().map_err(|e| e.to_string())
    }
}
