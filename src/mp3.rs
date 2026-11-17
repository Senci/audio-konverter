// MP3-Schreiber: LAME ist im Programm eingebaut (kein ffmpeg nötig).
// 320 kbit/s, 44,1 kHz. Ein ID3-Tag kommt vor die MP3-Daten.

use mp3lame_encoder::{
    max_required_buffer_size, Bitrate, Builder, Encoder, FlushNoGap, InterleavedPcm, MonoPcm,
    Quality,
};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct Mp3Schreiber {
    datei: BufWriter<File>,
    enc: Encoder,
    kanaele: usize,
}

impl Mp3Schreiber {
    pub fn neu(pfad: &Path, kanaele: usize, id3: &[u8]) -> Result<Self, String> {
        let fehler = |e: &dyn std::fmt::Debug| format!("MP3-Encoder: {e:?}");

        let mut b = Builder::new().ok_or("MP3-Encoder konnte nicht gestartet werden")?;
        b.set_num_channels(kanaele as u8).map_err(|e| fehler(&e))?;
        b.set_sample_rate(44_100).map_err(|e| fehler(&e))?;
        b.set_brate(Bitrate::Kbps320).map_err(|e| fehler(&e))?;
        b.set_quality(Quality::NearBest).map_err(|e| fehler(&e))?;
        let enc = b.build().map_err(|e| fehler(&e))?;

        // create_new: eine vorhandene Datei wird nie überschrieben
        let datei = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(pfad)
            .map_err(|e| e.to_string())?;
        let mut datei = BufWriter::new(datei);
        datei.write_all(id3).map_err(|e| e.to_string())?; // leer = nichts
        Ok(Self { datei, enc, kanaele })
    }

    // Interleavte 16-Bit-Samples kodieren und schreiben
    pub fn schreibe(&mut self, pcm: &[i16]) -> Result<(), String> {
        let mut aus: Vec<u8> = Vec::new();
        aus.reserve(max_required_buffer_size(pcm.len()));
        let n = if self.kanaele == 1 {
            self.enc.encode(MonoPcm(pcm), aus.spare_capacity_mut())
        } else {
            self.enc.encode(InterleavedPcm(pcm), aus.spare_capacity_mut())
        }
        .map_err(|e| format!("MP3-Encoder: {e:?}"))?;
        // LAME hat n Bytes in den freien Platz geschrieben (so sieht es die Doku der Crate vor)
        unsafe { aus.set_len(n) };
        self.datei.write_all(&aus).map_err(|e| e.to_string())
    }

    // Reste aus dem Encoder holen und Datei abschließen
    pub fn fertig(mut self) -> Result<(), String> {
        let mut aus: Vec<u8> = Vec::new();
        aus.reserve(max_required_buffer_size(0));
        let n = self
            .enc
            .flush::<FlushNoGap>(aus.spare_capacity_mut())
            .map_err(|e| format!("MP3-Encoder: {e:?}"))?;
        unsafe { aus.set_len(n) };
        self.datei.write_all(&aus).map_err(|e| e.to_string())?;
        self.datei.flush().map_err(|e| e.to_string())
    }
}
