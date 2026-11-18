// Eigener Konverter (ersetzt ffmpeg):
// Datei lesen (symphonia) -> auf 44,1 kHz umrechnen (rubato) -> auf 16 Bit runden
// -> als AIFF oder MP3 schreiben. Läuft blockweise, braucht also auch bei langen
// DJ-Mixen kaum Speicher.

use crate::aiff::AiffSchreiber;
use crate::mp3::Mp3Schreiber;
use crate::tags;
use rubato::{FftFixedIn, Resampler};
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymFehler;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

const ZIELRATE: usize = 44_100;
const BLOCK: usize = 4096; // Eingangsblock des Resamplers (Frames)

// Fehlertext bei Abbruch durch den Nutzer
pub const ABGEBROCHEN: &str = "abgebrochen";

// ---------------------------------------------------------------------------
// Zielformat und Sperrregel
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Ziel {
    #[default]
    Aiff,
    Mp3,
}

impl Ziel {
    pub fn name(self) -> &'static str {
        match self {
            Ziel::Aiff => "AIFF · 16 Bit · 44,1 kHz",
            Ziel::Mp3 => "MP3 · 320 kbit/s · 44,1 kHz",
        }
    }
    pub fn kurz(self) -> &'static str {
        match self {
            Ziel::Aiff => "AIFF",
            Ziel::Mp3 => "MP3",
        }
    }
    pub fn endung(self) -> &'static str {
        match self {
            Ziel::Aiff => "aiff",
            Ziel::Mp3 => "mp3",
        }
    }
}

// MP3 ist immer gesperrt, AIFF nur beim Ziel AIFF
pub fn ist_gesperrt(endung: &str, ziel: Ziel) -> bool {
    endung == "mp3" || (ziel == Ziel::Aiff && (endung == "aiff" || endung == "aif"))
}

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
            Senke::Aiff(a) => a.fertig(),
            Senke::Mp3(m) => m.fertig(),
        }
    }
}

// Letzte Stufe: Float -> 16 Bit (mit Dither) -> Datei
struct Ausgang {
    senke: Senke,
    dither: bool,     // nur bei AIFF und nur wenn wirklich Bits wegfallen
    rng: u32,         // Zufallszahlen für den Dither
    skip: usize,      // Verzögerung des Resamplers am Anfang, wird verworfen
    max: Option<u64>, // am Ende: nicht mehr als die erwartete Länge schreiben
    geschrieben: u64, // bisher geschriebene Frames
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
        let mut pcm = Vec::with_capacity(anzahl * kan.len());
        for i in start..start + anzahl {
            for k in kan {
                let s = self.quantisiere(k[i]);
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
            for (kanal, s) in self.pending.iter_mut().zip(frame) {
                kanal.push(*s);
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
                    for c in &mut self.pending {
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
                let rest: Vec<&[f32]> = pending.iter().map(Vec::as_slice).collect();
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

// Eine Datei umwandeln. `cover` ist ein Bild, das (wenn vorhanden) in die Metadaten kommt.
pub fn wandle(
    quelle: &Path,
    ziel_pfad: &Path,
    ziel: Ziel,
    cover: Option<&Path>,
    abbruch: &AtomicBool,
) -> Result<(), String> {
    // create_new: eine vorhandene Datei wird nie überschrieben, auch nicht wenn zwei
    // Aufträge gleichzeitig dasselbe Ziel haben (z. B. „Track.wav“ und „Track.flac“)
    let datei = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(ziel_pfad)
        .map_err(|e| match e.kind() {
            ErrorKind::AlreadyExists => "Zieldatei existiert bereits".to_string(),
            _ => format!("Zieldatei nicht anlegbar: {e}"),
        })?;
    let ergebnis = lauf(quelle, datei, ziel, cover, abbruch);
    if ergebnis.is_err() {
        // Nur die eben selbst angelegte, halbfertige Datei löschen
        let _ = std::fs::remove_file(ziel_pfad);
    }
    ergebnis
}

fn lauf(
    quelle: &Path,
    ziel_datei: File,
    ziel: Ziel,
    cover: Option<&Path>,
    abbruch: &AtomicBool,
) -> Result<(), String> {
    // 1. Quelle öffnen und das Format erkennen
    let datei = File::open(quelle).map_err(|e| format!("Quelle nicht lesbar: {e}"))?;
    let mss = MediaSourceStream::new(Box::new(datei), MediaSourceStreamOptions::default());
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
    let id3 = tags::erzeuge(&tag_liste, cover);
    let senke = match ziel {
        Ziel::Aiff => Senke::Aiff(
            AiffSchreiber::neu(ziel_datei, kanaele as u16, id3).map_err(|e| e.to_string())?,
        ),
        Ziel::Mp3 => Senke::Mp3(Mp3Schreiber::neu(ziel_datei, kanaele as u8, &id3)?),
    };
    let ausgang = Ausgang {
        senke,
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
            return Err(ABGEBROCHEN.into());
        }
        let paket = match leser.next_packet() {
            Ok(p) => p,
            Err(SymFehler::IoError(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
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
            Err(SymFehler::DecodeError(_)) => {} // defektes Paket überspringen
            Err(e) => return Err(format!("Dekodierfehler: {e}")),
        }
    }
    strom.abschliessen(frames_ein, rate)
}

// ---------------------------------------------------------------------------
// Tests: `cargo test --release`
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::path::PathBuf;

    // Testordner im Temp-Verzeichnis (bleibt zur Kontrolle liegen)
    pub fn ordner(name: &str) -> PathBuf {
        let o = std::env::temp_dir()
            .join("audio-konverter-tests")
            .join(name);
        let _ = std::fs::remove_dir_all(&o);
        std::fs::create_dir_all(&o).unwrap();
        o
    }

    // Einfache PCM-WAV-Datei schreiben: Sinus 440 Hz, Stereo
    pub fn wav(pfad: &Path, rate: u32, bits: u16, sekunden: f32) -> Vec<i32> {
        let frames = (rate as f32 * sekunden) as u32;
        let bytes = u32::from(bits / 8);
        let mut samples = Vec::new();
        let mut daten = Vec::new();
        let max = (1i64 << (bits - 1)) as f32 * 0.5;
        for i in 0..frames {
            let v = ((i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * max) as i32;
            for _ in 0..2 {
                samples.push(v);
                daten.extend_from_slice(&v.to_le_bytes()[..bytes as usize]);
            }
        }
        let mut f = Vec::new();
        f.extend_from_slice(b"RIFF");
        f.extend_from_slice(&(36 + daten.len() as u32).to_le_bytes());
        f.extend_from_slice(b"WAVEfmt ");
        f.extend_from_slice(&16u32.to_le_bytes());
        f.extend_from_slice(&1u16.to_le_bytes()); // PCM
        f.extend_from_slice(&2u16.to_le_bytes()); // Stereo
        f.extend_from_slice(&rate.to_le_bytes());
        f.extend_from_slice(&(rate * 2 * bytes).to_le_bytes());
        f.extend_from_slice(&(2 * bits / 8).to_le_bytes());
        f.extend_from_slice(&bits.to_le_bytes());
        f.extend_from_slice(b"data");
        f.extend_from_slice(&(daten.len() as u32).to_le_bytes());
        f.extend_from_slice(&daten);
        std::fs::write(pfad, f).unwrap();
        samples
    }

    // AIFF lesen: (Frames laut COMM, Audio-Samples aus SSND, Inhalt des ID3-Chunks)
    fn aiff_lesen(pfad: &Path) -> (u32, Vec<i16>, Vec<u8>) {
        let d = std::fs::read(pfad).unwrap();
        assert_eq!(&d[0..4], b"FORM");
        assert_eq!(&d[8..12], b"AIFF");
        assert_eq!(
            u32::from_be_bytes(d[4..8].try_into().unwrap()) as usize,
            d.len() - 8
        );
        let (mut frames, mut pcm, mut id3) = (0, Vec::new(), Vec::new());
        let mut p = 12;
        while p + 8 <= d.len() {
            let id = &d[p..p + 4];
            let groesse = u32::from_be_bytes(d[p + 4..p + 8].try_into().unwrap()) as usize;
            let inhalt = &d[p + 8..p + 8 + groesse];
            match id {
                b"COMM" => frames = u32::from_be_bytes(inhalt[2..6].try_into().unwrap()),
                b"SSND" => {
                    pcm = inhalt[8..]
                        .chunks_exact(2)
                        .map(|b| i16::from_be_bytes([b[0], b[1]]))
                        .collect()
                }
                b"ID3 " => id3 = inhalt.to_vec(),
                _ => {}
            }
            p += 8 + groesse + groesse % 2;
        }
        (frames, pcm, id3)
    }

    fn nie_abbrechen() -> AtomicBool {
        AtomicBool::new(false)
    }

    #[test]
    fn wav_16bit_44khz_wird_bitgenau_aiff() {
        let o = ordner("bitgenau");
        let quelle = o.join("a.wav");
        let original = wav(&quelle, 44_100, 16, 0.5);
        wandle(
            &quelle,
            &o.join("a.aiff"),
            Ziel::Aiff,
            None,
            &nie_abbrechen(),
        )
        .unwrap();
        let (frames, pcm, _) = aiff_lesen(&o.join("a.aiff"));
        assert_eq!(frames as usize * 2, original.len());
        assert!(pcm.iter().zip(&original).all(|(a, b)| i32::from(*a) == *b));
    }

    #[test]
    fn wav_24bit_96khz_wird_44khz_aiff_mit_richtiger_laenge() {
        let o = ordner("umrechnen");
        let quelle = o.join("a.wav");
        wav(&quelle, 96_000, 24, 1.0);
        wandle(
            &quelle,
            &o.join("a.aiff"),
            Ziel::Aiff,
            None,
            &nie_abbrechen(),
        )
        .unwrap();
        let (frames, pcm, _) = aiff_lesen(&o.join("a.aiff"));
        assert_eq!(frames, 44_100);
        assert_eq!(pcm.len(), 44_100 * 2);
        // Sinus mit halber Aussteuerung: Spitzen bei ca. 16384
        let spitze = pcm.iter().map(|s| i32::from(*s).abs()).max().unwrap();
        assert!((15_500..17_500).contains(&spitze), "Spitze {spitze}");
    }

    #[test]
    fn wav_wird_mp3() {
        let o = ordner("mp3");
        let quelle = o.join("a.wav");
        wav(&quelle, 48_000, 16, 1.0);
        wandle(&quelle, &o.join("a.mp3"), Ziel::Mp3, None, &nie_abbrechen()).unwrap();
        let d = std::fs::read(o.join("a.mp3")).unwrap();
        // 1 s bei 320 kbit/s = ca. 40 kB, erster Frame beginnt mit dem Sync-Wort
        assert!(d.len() > 35_000, "nur {} Bytes", d.len());
        assert_eq!(d[0], 0xFF);
        assert_eq!(d[1] & 0xE0, 0xE0);
    }

    #[test]
    fn cover_landet_im_id3_chunk() {
        let o = ordner("cover");
        let quelle = o.join("a.wav");
        wav(&quelle, 44_100, 16, 0.1);
        let bild = o.join("cover.jpg");
        std::fs::write(&bild, b"\xFF\xD8\xFF\xE0 kein echtes Bild").unwrap();
        wandle(
            &quelle,
            &o.join("a.aiff"),
            Ziel::Aiff,
            Some(&bild),
            &nie_abbrechen(),
        )
        .unwrap();
        let (_, _, id3) = aiff_lesen(&o.join("a.aiff"));
        assert_eq!(&id3[0..3], b"ID3");
        assert!(id3.windows(4).any(|w| w == b"APIC"));
    }

    #[test]
    fn cover_landet_im_mp3() {
        let o = ordner("cover-mp3");
        let quelle = o.join("a.wav");
        wav(&quelle, 44_100, 16, 0.1);
        let bild = o.join("cover.jpg");
        std::fs::write(&bild, b"\xFF\xD8\xFF\xE0 kein echtes Bild").unwrap();
        wandle(
            &quelle,
            &o.join("a.mp3"),
            Ziel::Mp3,
            Some(&bild),
            &nie_abbrechen(),
        )
        .unwrap();
        let d = std::fs::read(o.join("a.mp3")).unwrap();
        assert_eq!(&d[0..3], b"ID3");
        assert!(d.windows(4).any(|w| w == b"APIC"));
    }

    #[test]
    fn vorhandene_zieldatei_bleibt_unangetastet() {
        let o = ordner("vorhanden");
        let quelle = o.join("a.wav");
        wav(&quelle, 44_100, 16, 0.1);
        let ziel = o.join("a.aiff");
        std::fs::write(&ziel, b"nicht anfassen").unwrap();
        let fehler = wandle(&quelle, &ziel, Ziel::Aiff, None, &nie_abbrechen()).unwrap_err();
        assert_eq!(fehler, "Zieldatei existiert bereits");
        assert_eq!(std::fs::read(&ziel).unwrap(), b"nicht anfassen");
    }

    #[test]
    fn abbruch_loescht_halbfertige_datei() {
        let o = ordner("abbruch");
        let quelle = o.join("a.wav");
        wav(&quelle, 44_100, 16, 0.1);
        let ziel = o.join("a.aiff");
        let fehler = wandle(&quelle, &ziel, Ziel::Aiff, None, &AtomicBool::new(true)).unwrap_err();
        assert_eq!(fehler, ABGEBROCHEN);
        assert!(!ziel.exists());
    }

    #[test]
    fn kaputte_quelle_hinterlaesst_keine_datei() {
        let o = ordner("kaputt");
        let quelle = o.join("a.flac");
        std::fs::write(&quelle, b"kein Audio").unwrap();
        let ziel = o.join("a.aiff");
        assert!(wandle(&quelle, &ziel, Ziel::Aiff, None, &nie_abbrechen()).is_err());
        assert!(!ziel.exists());
    }

    #[test]
    fn sperrregel() {
        assert!(ist_gesperrt("mp3", Ziel::Aiff));
        assert!(ist_gesperrt("mp3", Ziel::Mp3));
        assert!(ist_gesperrt("aif", Ziel::Aiff));
        assert!(!ist_gesperrt("aiff", Ziel::Mp3));
        assert!(!ist_gesperrt("flac", Ziel::Aiff));
    }
}
