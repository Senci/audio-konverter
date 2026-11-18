// FLAC-Schreiber (flacenc, reines Rust): 16 Bit, 44,1 kHz wie bei AIFF. Es wird
// blockweise geschrieben, auch lange Mixe brauchen kaum Speicher. Tags und Cover stehen
// als Metadatenblöcke am Dateianfang; der Kopf wird am Ende noch einmal mit der fertigen
// Länge und der MD5-Prüfsumme überschrieben.

use crate::texte::t;
use flacenc::bitsink::ByteSink;
use flacenc::component::{BitRepr, MetadataBlockData, Stream, StreamInfo};
use flacenc::config::Encoder;
use flacenc::encode_fixed_size_frame;
use flacenc::error::{Verified, Verify};
use flacenc::source::{Context, Fill, FrameBuf};
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};

const BLOCK: usize = 4096; // Frames pro FLAC-Block
const RATE: usize = 44_100;
const BITS: usize = 16;

pub struct FlacSchreiber {
    datei: BufWriter<File>,
    config: Verified<Encoder>,
    info: StreamInfo,
    bloecke: Vec<(u8, Vec<u8>)>, // Metadatenblöcke (Typ, Inhalt): Vorbis-Kommentar, Bild
    kanaele: usize,
    sammel: Vec<i32>, // interleavte Samples des angefangenen Blocks
    fb: FrameBuf,
    ctx: Context, // zählt Samples und rechnet die MD5-Prüfsumme
    frames: usize,
    sink: ByteSink,
}

fn fehler(e: impl std::fmt::Display) -> String {
    format!("{}: {e}", t().flac_encoder)
}

// Dateikopf: "fLaC", STREAMINFO und die weiteren Metadatenblöcke
fn kopf(info: &StreamInfo, bloecke: &[(u8, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut stream = Stream::with_stream_info(info.clone());
    for (typ, daten) in bloecke {
        stream.add_metadata_block(MetadataBlockData::new_unknown(*typ, daten).map_err(fehler)?);
    }
    let mut sink = ByteSink::new();
    stream.write(&mut sink).map_err(fehler)?;
    Ok(sink.as_slice().to_vec())
}

impl FlacSchreiber {
    pub fn neu(datei: File, kanaele: usize, bloecke: Vec<(u8, Vec<u8>)>) -> Result<Self, String> {
        let config = Encoder::default()
            .into_verified()
            .map_err(|(_, e)| fehler(e))?;
        let mut info = StreamInfo::new(RATE, kanaele, BITS).map_err(fehler)?;
        info.set_block_sizes(BLOCK, BLOCK).map_err(fehler)?;
        let mut datei = BufWriter::new(datei);
        datei
            .write_all(&kopf(&info, &bloecke)?)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            datei,
            config,
            info,
            bloecke,
            kanaele,
            sammel: Vec::new(),
            fb: FrameBuf::with_size(kanaele, BLOCK).map_err(fehler)?,
            ctx: Context::new(BITS, kanaele),
            frames: 0,
            sink: ByteSink::new(),
        })
    }

    // Interleavte 16-Bit-Samples entgegennehmen und fertige Blöcke schreiben
    pub fn schreibe(&mut self, samples: &[i16]) -> Result<(), String> {
        self.sammel.extend(samples.iter().map(|&s| i32::from(s)));
        let voll = BLOCK * self.kanaele;
        let n = self.sammel.len() / voll * voll;
        if n > 0 {
            let fertig: Vec<i32> = self.sammel.drain(..n).collect();
            for block in fertig.chunks(voll) {
                self.block(block)?;
            }
        }
        Ok(())
    }

    // Einen Block kodieren und schreiben
    fn block(&mut self, samples: &[i32]) -> Result<(), String> {
        let mut paar = (&mut self.fb, &mut self.ctx);
        paar.fill_interleaved(samples).map_err(fehler)?;
        let nr = self.ctx.current_frame_number().unwrap_or(0);
        let frame =
            encode_fixed_size_frame(&self.config, &self.fb, nr, &self.info).map_err(fehler)?;
        self.info.update_frame_info(&frame);
        self.sink.clear();
        frame.write(&mut self.sink).map_err(fehler)?;
        self.frames += 1;
        self.datei
            .write_all(self.sink.as_slice())
            .map_err(|e| e.to_string())
    }

    // Rest schreiben, Kopf mit Länge und Prüfsumme nachtragen
    pub fn fertig(mut self) -> Result<(), String> {
        if !self.sammel.is_empty() {
            let rest = std::mem::take(&mut self.sammel);
            self.block(&rest)?;
        }
        if self.frames == 0 {
            self.info.set_frame_sizes(0, 0).map_err(fehler)?; // leere Datei
        }
        // Blockgröße wie beim Referenz-Encoder: min = max = Blockgröße, der letzte kürzere
        // Block zählt nicht mit. Sonst hält z. B. symphonia den Strom für variabel.
        let gesamt = self.ctx.total_samples();
        let block = if gesamt == 0 || gesamt >= BLOCK {
            BLOCK
        } else {
            gesamt.max(16) // nur ein kurzer Block; FLAC verlangt mindestens 16
        };
        self.info.set_block_sizes(block, block).map_err(fehler)?;
        self.info.set_total_samples(gesamt);
        self.info.set_md5_digest(&self.ctx.md5_digest());
        let neu = kopf(&self.info, &self.bloecke)?;
        self.datei
            .seek(SeekFrom::Start(0))
            .map_err(|e| e.to_string())?;
        self.datei.write_all(&neu).map_err(|e| e.to_string())?;
        self.datei.flush().map_err(|e| e.to_string())
    }
}
