// AIFF-Schreiber: 16 Bit, 44,1 kHz. Der ID3-Tag (mit Cover) kommt als "ID3 "-Chunk ans Ende.

use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

pub struct AiffSchreiber {
    datei: BufWriter<File>,
    kanaele: u16,
    daten_bytes: u64, // Anzahl geschriebener Audio-Bytes
    id3: Vec<u8>,     // fertiger ID3v2-Tag, leer = keiner
}

impl AiffSchreiber {
    // Datei anlegen und Kopf schreiben. Größen werden erst in fertig() eingetragen.
    pub fn neu(pfad: &Path, kanaele: u16, id3: Vec<u8>) -> io::Result<Self> {
        // create_new: eine vorhandene Datei wird nie überschrieben
        let datei = OpenOptions::new().write(true).create_new(true).open(pfad)?;
        let mut w = BufWriter::new(datei);
        w.write_all(b"FORM")?;
        w.write_all(&0u32.to_be_bytes())?; // Dateigröße (Offset 4), später
        w.write_all(b"AIFF")?;
        w.write_all(b"COMM")?;
        w.write_all(&18u32.to_be_bytes())?;
        w.write_all(&kanaele.to_be_bytes())?;
        w.write_all(&0u32.to_be_bytes())?; // Anzahl Frames (Offset 22), später
        w.write_all(&16u16.to_be_bytes())?; // 16 Bit
        w.write_all(&[0x40, 0x0E, 0xAC, 0x44, 0, 0, 0, 0, 0, 0])?; // 44100 Hz als 80-Bit-Float
        w.write_all(b"SSND")?;
        w.write_all(&0u32.to_be_bytes())?; // Größe des Audio-Chunks (Offset 42), später
        w.write_all(&0u32.to_be_bytes())?; // offset
        w.write_all(&0u32.to_be_bytes())?; // blockSize
        Ok(Self { datei: w, kanaele, daten_bytes: 0, id3 })
    }

    // Interleavte 16-Bit-Samples anhängen (AIFF ist Big Endian)
    pub fn schreibe(&mut self, pcm: &[i16]) -> io::Result<()> {
        let bytes: Vec<u8> = pcm.iter().flat_map(|s| s.to_be_bytes()).collect();
        self.daten_bytes += bytes.len() as u64;
        self.datei.write_all(&bytes)
    }

    // ID3-Chunk anhängen und die Größen im Kopf nachtragen
    pub fn fertig(mut self) -> io::Result<()> {
        if !self.id3.is_empty() {
            self.datei.write_all(b"ID3 ")?;
            self.datei.write_all(&(self.id3.len() as u32).to_be_bytes())?;
            self.datei.write_all(&self.id3)?;
            if self.id3.len() % 2 == 1 {
                self.datei.write_all(&[0])?; // Chunks haben gerade Länge
            }
        }
        let ende = self.datei.stream_position()?;
        let frames = self.daten_bytes / (2 * self.kanaele as u64);
        self.datei.seek(SeekFrom::Start(4))?;
        self.datei.write_all(&((ende - 8) as u32).to_be_bytes())?;
        self.datei.seek(SeekFrom::Start(22))?;
        self.datei.write_all(&(frames as u32).to_be_bytes())?;
        self.datei.seek(SeekFrom::Start(42))?;
        self.datei.write_all(&((self.daten_bytes + 8) as u32).to_be_bytes())?;
        self.datei.flush()
    }
}
