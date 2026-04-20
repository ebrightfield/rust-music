//! SF2 SoundFont loading, caching, and verification.
//!
//! Requires the `render` feature (`music-midi = { features = ["render"] }`).
//!
//! This module provides [`SoundFont`] — a loaded SF2 file — plus optional
//! network download of the bundled GeneralUser GS SoundFont.
//!
//! # Security
//! Every download is verified against a pinned SHA-256 digest ([`SF_SHA256_HEX`]).
//! Downloads are capped at [`MAX_BYTES`] (256 MiB). Only HTTPS URLs are accepted.
// REQ-O13, X4, X5, X10: SoundFont loader.
// Phase 5-pre has pinned the canonical download source and SHA-256; constants below.
#![cfg(feature = "render")]

use crate::error::MidiConversionError;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

// [AMEND-W1 superseded]: the original canonical URL
// `https://www.schristiancollins.com/generaluser/GeneralUser-GS-v1.471.sf2`
// no longer serves raw SF2 bytes (the upstream site migrated to an SPA that
// routes every path to a landing page). Phase 5-pre re-pinned the canonical
// source to the maintainer's own GitHub repository, which README explicitly
// endorses as the preferred source for automated packaging. Pinned to an
// exact commit SHA so the URL is stable even if `main` advances.
//
// Version: GeneralUser GS v2.0.3 (commit 9704918364, 2026-02-23)
// Size: 32,319,396 bytes (~30.8 MiB), well under the 256 MiB download cap.
// Magic header verified: RIFF + 4-byte length + sfbk.
pub const SF_URL: &str =
    "https://raw.githubusercontent.com/mrbumpy409/GeneralUser-GS/9704918364/GeneralUser-GS.sf2";

// [AMEND-D resolved]: 64-char hex digest pinned by Phase 5-pre.
pub const SF_SHA256_HEX: &str =
    "9575028c7a1f589f5770fccc8cff2734566af40cd26ed836944e9a5152688cfe";

// Compile-time assertion: the pinned hash constant must be exactly 64 hex chars.
// If the constant gets accidentally reverted to a placeholder, this fails to compile.
const _: [(); 64] = [(); SF_SHA256_HEX.len()];

// 256 MiB hard cap on the download (per threat model: bounded disk use).
pub const MAX_BYTES: u64 = 256 * 1024 * 1024;

// Cache layout under the user's XDG data dir (no user-supplied path segments).
pub const CACHE_SUBDIR: &str = "music-midi/soundfonts";
pub const CACHE_FILE: &str = "GeneralUser-GS-v2.0.3.sf2";

// ---------------------------------------------------------------------------
// Seam traits (5a) — allow test injection of HTTP / FS / synth / WAV impls
// ---------------------------------------------------------------------------

/// Filesystem cache seam. Implementations must be injected; production code
/// uses `RealFs`. Tests can substitute a `TempDirFs` or a `MockFs`.
pub trait FsCache {
    fn read(&self, path: &Path) -> Result<Vec<u8>, MidiConversionError>;
    fn write_atomic(&mut self, path: &Path, bytes: &[u8]) -> Result<(), MidiConversionError>;
    fn exists(&self, path: &Path) -> bool;
}

/// HTTP fetch seam. Only HTTPS URLs are accepted; implementations must enforce
/// the `max_bytes` cap and return `SoundFontDownload` on failure.
pub trait HttpFetch {
    fn get(&mut self, url: &str, max_bytes: u64) -> Result<Vec<u8>, MidiConversionError>;
}

/// Synthesizer output seam. Callers push MIDI events and pull audio blocks.
pub trait Synthesizer {
    fn handle_event(&mut self, event: crate::MidiEvent);
    fn write_block(&mut self, buf: &mut [f32]);
}

/// WAV sink seam. Callers push stereo i16 frames; call `finalize` when done.
pub trait WavSink {
    fn write_frame(&mut self, l: i16, r: i16);
    fn finalize(self: Box<Self>);
}

// ---------------------------------------------------------------------------
// Default implementations
// ---------------------------------------------------------------------------

/// Production filesystem implementation using `std::fs`.
pub struct RealFs;

impl FsCache for RealFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>, MidiConversionError> {
        Ok(fs::read(path)?)
    }

    fn write_atomic(&mut self, path: &Path, bytes: &[u8]) -> Result<(), MidiConversionError> {
        write_atomic_impl(path, bytes)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

/// Production HTTP implementation using `ureq` (TLS only; no bypass of certificate validation).
pub struct UreqFetch;

impl HttpFetch for UreqFetch {
    fn get(&mut self, url: &str, max_bytes: u64) -> Result<Vec<u8>, MidiConversionError> {
        download_impl(url, max_bytes)
    }
}

/// Production oxisynth adapter.
pub struct OxiSynthAdapter {
    inner: oxisynth::Synth,
    #[allow(dead_code)]
    sample_rate: f32,
}

impl OxiSynthAdapter {
    /// Create a new synth loaded with the given SF2 bytes at `sample_rate` Hz.
    ///
    /// Returns [`MidiConversionError::SoundFont`] if `oxisynth` cannot parse the SF2 data.
    pub fn new(sf_bytes: &[u8], sample_rate: f32) -> Result<Self, MidiConversionError> {
        let mut synth = oxisynth::Synth::default();
        synth.set_sample_rate(sample_rate);
        let font = oxisynth::SoundFont::load(&mut std::io::Cursor::new(sf_bytes))
            .map_err(|_| MidiConversionError::SoundFont(
                "oxisynth failed to parse SF2 data".into(),
            ))?;
        synth.add_font(font, true);
        Ok(Self { inner: synth, sample_rate })
    }
}

impl Synthesizer for OxiSynthAdapter {
    fn handle_event(&mut self, event: crate::MidiEvent) {
        use crate::MidiMessage;
        let oxi_event = match event.message {
            MidiMessage::NoteOn { key, velocity } => Some(oxisynth::MidiEvent::NoteOn {
                channel: event.channel,
                key,
                vel: velocity,
            }),
            MidiMessage::NoteOff { key, .. } => Some(oxisynth::MidiEvent::NoteOff {
                channel: event.channel,
                key,
            }),
            MidiMessage::ProgramChange(p) => Some(oxisynth::MidiEvent::ProgramChange {
                channel: event.channel,
                program_id: p,
            }),
            MidiMessage::ControlChange { controller, value } => {
                Some(oxisynth::MidiEvent::ControlChange {
                    channel: event.channel,
                    ctrl: controller,
                    value,
                })
            }
            // Tempo, TimeSignature, TrackName, EndOfTrack have no synth equivalent
            _ => None,
        };
        if let Some(ev) = oxi_event {
            let _ = self.inner.send_event(ev);
        }
    }

    fn write_block(&mut self, buf: &mut [f32]) {
        // buf is interleaved L R L R ... (len = 2 * frames)
        self.inner.write(buf.as_mut());
    }
}

/// Production WAV sink using `hound`.
pub struct HoundWav {
    writer: hound::WavWriter<std::io::BufWriter<std::fs::File>>,
}

impl HoundWav {
    /// Create (or truncate) a WAV file at `path` with the given sample rate.
    ///
    /// The file is written as 16-bit stereo PCM. Call [`WavSink::finalize`] to flush
    /// the header after all frames have been written.
    pub fn create(path: &Path, sample_rate: u32) -> Result<Self, MidiConversionError> {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let writer = hound::WavWriter::create(path, spec)
            .map_err(|e| MidiConversionError::SoundFont(e.to_string()))?;
        Ok(Self { writer })
    }
}

impl WavSink for HoundWav {
    fn write_frame(&mut self, l: i16, r: i16) {
        let _ = self.writer.write_sample(l);
        let _ = self.writer.write_sample(r);
    }

    fn finalize(self: Box<Self>) {
        let _ = self.writer.finalize();
    }
}

// ---------------------------------------------------------------------------
// SoundFont
// ---------------------------------------------------------------------------

/// A loaded SoundFont (SF2) blob. Construct via one of the `from_*` or
/// `general_user_gs*` constructors; pass to `AudioRenderer::new`.
#[derive(Debug)]
pub struct SoundFont {
    pub(crate) bytes: Vec<u8>,
}

impl SoundFont {
    /// Load a SoundFont from an arbitrary file path. No SHA-256 verification
    /// is performed; the caller is responsible for provenance.
    pub fn from_path(p: impl AsRef<Path>) -> Result<Self, MidiConversionError> {
        let bytes = fs::read(p.as_ref())?;
        Ok(Self { bytes })
    }

    /// Wrap raw bytes as a SoundFont. No SHA-256 verification.
    pub fn from_bytes(b: &[u8]) -> Result<Self, MidiConversionError> {
        Ok(Self { bytes: b.to_vec() })
    }

    /// Load GeneralUser GS v2.0.3 from cache, or download it if not cached.
    ///
    /// REQ-O13, X4, X5, X10: HTTPS-only; SHA-256 verified on every load;
    /// 256 MiB download cap; no user-supplied path segments.
    pub fn general_user_gs() -> Result<Self, MidiConversionError> {
        Self::general_user_gs_impl(&mut RealFs, &mut UreqFetch)
    }

    /// Load GeneralUser GS v2.0.3 from cache only; never downloads.
    ///
    /// Returns `SoundFontDownload` if the cache file is absent.
    pub fn general_user_gs_offline() -> Result<Self, MidiConversionError> {
        Self::general_user_gs_offline_impl(&RealFs)
    }

    // --- seam-injectable helpers for testing ---

    /// Like `general_user_gs()` but accepts injected seam implementations.
    pub fn general_user_gs_impl(
        fs: &mut impl FsCache,
        http: &mut impl HttpFetch,
    ) -> Result<Self, MidiConversionError> {
        let cache = cache_path()?;
        if fs.exists(&cache) {
            match load_and_verify_impl(fs, &cache) {
                Ok(bytes) => return Ok(Self { bytes }),
                Err(MidiConversionError::SoundFontChecksum { .. }) => {
                    // REQ-X10: refuse silent reuse; delete and re-download
                    let _ = std::fs::remove_file(&cache);
                }
                Err(e) => return Err(e),
            }
        }
        let bytes = http.get(SF_URL, MAX_BYTES)?;
        verify_sha256(&bytes)?;
        fs.write_atomic(&cache, &bytes)?;
        Ok(Self { bytes })
    }

    /// Like `general_user_gs_offline()` but accepts an injected `FsCache`.
    pub fn general_user_gs_offline_impl(
        fs: &impl FsCache,
    ) -> Result<Self, MidiConversionError> {
        let cache = cache_path()?;
        if !fs.exists(&cache) {
            return Err(MidiConversionError::SoundFontDownload(
                "offline mode requested but no cached GeneralUser GS v2.0.3 found".into(),
            ));
        }
        let bytes = load_and_verify_impl(fs, &cache)?;
        Ok(Self { bytes })
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Construct the cache path from const literals only. No user-supplied segments.
/// REQ anti-pattern: path-traversal prevention.
fn cache_path() -> Result<PathBuf, MidiConversionError> {
    let root = dirs::data_dir().ok_or_else(|| {
        MidiConversionError::SoundFontDownload("no XDG data dir available".into())
    })?;
    // Every push uses a const literal — CACHE_SUBDIR and CACHE_FILE are defined above.
    let mut p = root;
    p.push(CACHE_SUBDIR);
    p.push(CACHE_FILE);
    Ok(p)
}

fn download_impl(url: &str, max_bytes: u64) -> Result<Vec<u8>, MidiConversionError> {
    // REQ-X4: HTTPS-only gate.
    if !url.starts_with("https://") {
        return Err(MidiConversionError::SoundFontDownload(format!(
            "non-HTTPS URL rejected: {url}"
        )));
    }
    let resp = ureq::get(url)
        .call()
        .map_err(|e| MidiConversionError::SoundFontDownload(e.to_string()))?;
    // Limit the reader to max_bytes + 1 so we can detect over-limit downloads.
    let mut reader = resp.into_reader().take(max_bytes + 1);
    let mut bytes: Vec<u8> = Vec::new();
    std::io::copy(&mut reader, &mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(MidiConversionError::SoundFontDownload(format!(
            "download exceeded {max_bytes}-byte cap"
        )));
    }
    Ok(bytes)
}

fn verify_sha256(bytes: &[u8]) -> Result<(), MidiConversionError> {
    let digest = Sha256::digest(bytes);
    let got_hex = format!("{:x}", digest);
    if got_hex != SF_SHA256_HEX {
        return Err(MidiConversionError::SoundFontChecksum {
            expected: SF_SHA256_HEX.into(),
            actual: got_hex,
        });
    }
    Ok(())
}

fn load_and_verify_impl(
    fs: &impl FsCache,
    path: &Path,
) -> Result<Vec<u8>, MidiConversionError> {
    let bytes = fs.read(path)?;
    verify_sha256(&bytes)?;
    Ok(bytes)
}

/// Write `bytes` to `path` atomically: write to `*.partial`, fsync, rename.
/// The parent directory is created if absent.
fn write_atomic_impl(path: &Path, bytes: &[u8]) -> Result<(), MidiConversionError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("partial");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}
