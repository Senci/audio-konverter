// Eigener Konverter (ersetzt ffmpeg):
// Datei lesen (symphonia) -> auf 44,1 kHz umrechnen (rubato) -> auf 16 Bit runden
// -> als AIFF oder MP3 schreiben. Läuft blockweise, braucht also auch bei langen
// DJ-Mixen kaum Speicher.

use crate::aiff::AiffSchreiber;
use crate::convert::Ziel;
use crate::mp3::Mp3Schreiber;
use crate::tags;
use rubato::{FftFixedIn, Resampler};
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymFehler;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

const ZIELRATE: usize = 44_100;
const BLOCK: usize = 4096; // Eingangsblock des Resamplers (Frames)

// ---------------------------------------------------------------------------
// Ausgabe: AIFF- oder MP3-Datei
// ---------------------------------------------------------------------------

enum Senke {
    Aiff(AiffSchreiber),
    Mp3(Mp3Schreiber),
}

impl Senke {
    fn schreibe(&mut self, pcm: &[i16]) -> Result<(), String> {
        match self {
            Senke::Aiff(a) => a.schreibe(pcm).map_err(|e| e.to_string()),
            Senke::Mp3(m) => m.schreibe(pcm),
        }
    }
    fn fertig(self) -> Result<(), String> {
        match self {
            Senke::Aiff(a) => a.fertig().map_err(|e| e.to_string()),
            Senke::Mp3(m) => m.fertig(),
        }
    }
}

// Letzte Stufe: Float -> 16 Bit (mit Dither) -> Datei
struct Ausgang {
    senke: Senke,
    kanaele: usize,
    dither: bool,       // nur bei AIFF und nur wenn wirklich Bits wegfallen
    rng: u32,           // Zufallszahlen für den Dither
    skip: usize,        // Verzögerung des Resamplers am Anfang, wird verworfen
    max: Option<u64>,   // am Ende: nicht mehr als die erwartete Länge schreiben
    geschrieben: u64,   // bisher geschriebene Frames
}

impl Ausgang {
    // Zufallszahl in [0, 1) (xorshift, reicht für Dither völlig)
    fn zufall(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / 16_777_216.0
    }

    // Ein Sample auf 16 Bit runden. TPDF-Dither = Summe zweier Zufallszahlen minus 1 LSB.
    fn quantisiere(&mut self, x: f32) -> i16 {
        let mut v = x * 32768.0;
        if self.dither {
            v += self.zufall() - self.zufall();
        }
        v.round().clamp(-32768.0, 32767.0) as i16
    }

    // Einen Block (ein Vec pro Kanal) schreiben
    fn ausgeben(&mut self, kan: &[Vec<f32>]) -> Result<(), String> {
        let n = kan[0].len();
        let start = self.skip.min(n);
        self.skip -= start;
        let mut anzahl = n - start;
        if let Some(max) = self.max {
            anzahl = anzahl.min(max.saturating_sub(self.geschrieben) as usize);
        }
        if anzahl == 0 {
            return Ok(());
        }
        let mut pcm = Vec::with_capacity(anzahl * self.kanaele);
        for i in start..start + anzahl {
            for c in 0..self.kanaele {
                let s = self.quantisiere(kan[c][i]);
                pcm.push(s);
            }
        }
        self.geschrieben += anzahl as u64;
        self.senke.schreibe(&pcm)
    }
}

// ---------------------------------------------------------------------------
// Strom: sammelt Samples, rechnet blockweise auf 44,1 kHz um
// ---------------------------------------------------------------------------

struct Strom {
    resampler: Option<FftFixedIn<f32>>, // None = Quelle hat schon 44,1 kHz
    pending: Vec<Vec<f32>>,             // noch nicht verarbeitete Samples je Kanal
    ausgang: Ausgang,
}

impl Strom {
    fn neu(rate: usize, kanaele: usize, mut ausgang: Ausgang) -> Result<Self, String> {
        let resampler = if rate == ZIELRATE {
            None
        } else {
            let rs = FftFixedIn::<f32>::new(rate, ZIELRATE, BLOCK, 2, kanaele)
                .map_err(|e| e.to_string())?;
            ausgang.skip = rs.output_delay();
            Some(rs)
        };
        Ok(Self {
            resampler,
            pending: vec![Vec::new(); kanaele],
            ausgang,
        })
    }

    // Interleavte Samples entgegennehmen und alles verarbeiten, was vollständig ist
    fn zufuehren(&mut self, interleaved: &[f32]) -> Result<(), String> {
        let k = self.pending.len();
        for frame in interleaved.chunks_exact(k) {
            for (c, s) in frame.iter().enumerate() {
                self.pending[c].push(*s);
            }
        }
        match self.resampler.as_mut() {
            // Gleiche Rate: direkt ausgeben
            None => {
                let alle = std::mem::replace(&mut self.pending, vec![Vec::new(); k]);
                self.ausgang.ausgeben(&alle)
            }
            // Andere Rate: in festen Blöcken umrechnen
            Some(rs) => {
                let n = rs.input_frames_next();
                while self.pending[0].len() >= n {
                    let aus = {
                        let teil: Vec<&[f32]> = self.pending.iter().map(|c| &c[..n]).collect();
                        rs.process(&teil, None).map_err(|e| e.to_string())?
                    };
                    for c in self.pending.iter_mut() {
                        c.drain(..n);
                    }
                    self.ausgang.ausgeben(&aus)?;
                }
                Ok(())
            }
        }
    }

    // Reste verarbeiten, Datei abschließen
    fn abschliessen(self, frames_ein: u64, rate: usize) -> Result<(), String> {
        let Strom {
            mut resampler,
            pending,
            mut ausgang,
        } = self;
        if let Some(rs) = resampler.as_mut() {
            // Erwartete Länge: danach angehängte Auffüll-Stille wird abgeschnitten
            ausgang.max = Some((frames_ein * ZIELRATE as u64).div_ceil(rate as u64));
            if !pending[0].is_empty() {
                let rest: Vec<&[f32]> = pending.iter().map(|c| c.as_slice()).collect();
                let aus = rs
                    .process_partial(Some(rest.as_slice()), None)
                    .map_err(|e| e.to_string())?;
                ausgang.ausgeben(&aus)?;
            }
            // Ein Block Stille hinterher, damit der Rest des Filters herauskommt
            let aus = rs
                .process_partial(None::<&[Vec<f32>]>, None)
                .map_err(|e| e.to_string())?;
            ausgang.ausgeben(&aus)?;
        }
        ausgang.senke.fertig()
    }
}

// ---------------------------------------------------------------------------
// Einstieg
// ---------------------------------------------------------------------------

// Eine Datei umwandeln. `bild` ist ein Cover, das (wenn vorhanden) in die Metadaten kommt.
pub fn wandle(
    quelle: &Path,
    ziel_pfad: &Path,
    ziel: Ziel,
    bild: Option<&Path>,
    abbruch: &AtomicBool,
) -> Result<(), String> {
    // Vorhandene Dateien niemals anfassen
    if ziel_pfad.exists() {
        return Err("Zieldatei existiert bereits".into());
    }
    let ergebnis = lauf(quelle, ziel_pfad, ziel, bild, abbruch);
    if ergebnis.is_err() {
        let _ = std::fs::remove_file(ziel_pfad); // halbfertige Datei löschen
    }
    ergebnis
}

fn lauf(
    quelle: &Path,
    ziel_pfad: &Path,
    ziel: Ziel,
    bild: Option<&Path>,
    abbruch: &AtomicBool,
) -> Result<(), String> {
    // 1. Quelle öffnen und das Format erkennen
    let datei = File::open(quelle).map_err(|e| format!("Quelle nicht lesbar: {e}"))?;
    let mss = MediaSourceStream::new(Box::new(datei), Default::default());
    let mut hint = Hint::new();
    if let Some(e) = quelle.extension().and_then(|e| e.to_str()) {
        hint.with_extension(e);
    }
    let mut probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Format nicht erkannt: {e}"))?;

    // Tags einsammeln (ID3 vor dem Container und Tags im Container)
    let mut tag_liste = Vec::new();
    if let Some(m) = probed.metadata.get() {
        if let Some(r) = m.current() {
            tag_liste.extend(r.tags().iter().cloned());
        }
    }
    let mut leser = probed.format;
    if let Some(r) = leser.metadata().current() {
        tag_liste.extend(r.tags().iter().cloned());
    }

    // 2. Audiospur und Decoder
    let (spur, params) = {
        let t = leser.default_track().ok_or("Keine Audiospur gefunden")?;
        (t.id, t.codec_params.clone())
    };
    let rate = params.sample_rate.ok_or("Samplerate unbekannt")? as usize;
    let kanaele = params.channels.ok_or("Kanalzahl unbekannt")?.count();
    if kanaele == 0 || kanaele > 2 {
        return Err(format!(
            "{kanaele} Kanäle werden nicht unterstützt (nur Mono und Stereo)"
        ));
    }
    let mut decoder = symphonia::default::get_codecs()
        .make(&params, &DecoderOptions::default())
        .map_err(|e| format!("Codec nicht unterstützt: {e}"))?;

    // Dither nur bei AIFF und nur, wenn Bits wegfallen oder umgerechnet wird.
    // 16-Bit-Quellen mit 44,1 kHz werden dadurch bitgenau übernommen.
    let dither = ziel == Ziel::Aiff && !(rate == ZIELRATE && params.bits_per_sample == Some(16));

    // 3. Ausgabe vorbereiten
    let id3 = tags::erzeuge(&tag_liste, bild);
    let senke = match ziel {
        Ziel::Aiff => Senke::Aiff(
            AiffSchreiber::neu(ziel_pfad, kanaele as u16, id3).map_err(|e| e.to_string())?,
        ),
        Ziel::Mp3 => Senke::Mp3(Mp3Schreiber::neu(ziel_pfad, kanaele, &id3)?),
    };
    let ausgang = Ausgang {
        senke,
        kanaele,
        dither,
        rng: 0x1234_5678,
        skip: 0,
        max: None,
        geschrieben: 0,
    };
    let mut strom = Strom::neu(rate, kanaele, ausgang)?;

    // 4. Paket für Paket lesen, dekodieren und weiterreichen
    let mut frames_ein: u64 = 0;
    loop {
        if abbruch.load(Ordering::Relaxed) {
            return Err("abgebrochen".into());
        }
        let paket = match leser.next_packet() {
            Ok(p) => p,
            Err(SymFehler::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SymFehler::ResetRequired) => break,
            Err(e) => return Err(format!("Lesefehler: {e}")),
        };
        if paket.track_id() != spur {
            continue;
        }
        match decoder.decode(&paket) {
            Ok(puffer) => {
                if puffer.spec().channels.count() != kanaele {
                    return Err("Kanalzahl ändert sich mitten in der Datei".into());
                }
                let mut sb = SampleBuffer::<f32>::new(puffer.capacity() as u64, *puffer.spec());
                sb.copy_interleaved_ref(puffer);
                frames_ein += (sb.samples().len() / kanaele) as u64;
                strom.zufuehren(sb.samples())?;
            }
            Err(SymFehler::DecodeError(_)) => continue, // defektes Paket überspringen
            Err(e) => return Err(format!("Dekodierfehler: {e}")),
        }
    }
    strom.abschliessen(frames_ein, rate)
}
