use anyhow::{bail, Context, Result};
use music::melody::MelodicEvent;
use music_midi::melody_to_smf_bytes;
use std::fs;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MidiOutputFormat {
    Midi,
    Wav,
    Play,
}

impl MidiOutputFormat {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "midi" | "mid" => Some(Self::Midi),
            "wav" | "wave" => Some(Self::Wav),
            "play" => Some(Self::Play),
            _ => None,
        }
    }
}

pub struct MidiOutputArgs<'a> {
    pub format: MidiOutputFormat,
    pub output: Option<&'a str>,
    pub bpm: f32,
    pub ppq: u16,
    pub soundfont: Option<&'a str>,
    pub offline: bool,
    pub sample_rate: u32,
    pub tail: f32,
}

pub fn render_melody(events: &[MelodicEvent], args: MidiOutputArgs<'_>) -> Result<()> {
    anyhow::ensure!(
        args.bpm.is_finite() && args.bpm > 0.0,
        "--bpm must be positive"
    );
    anyhow::ensure!(args.sample_rate > 0, "--sample-rate must be positive");
    anyhow::ensure!(
        args.tail.is_finite() && args.tail >= 0.0,
        "--tail must be non-negative"
    );

    let bytes = melody_to_smf_bytes(events, args.bpm, args.ppq)
        .context("converting melody to Standard MIDI File")?;
    match args.format {
        MidiOutputFormat::Midi => {
            let path = args
                .output
                .context("MIDI output requires --output <path>")?;
            fs::write(path, bytes).with_context(|| format!("writing {path}"))?;
        }
        MidiOutputFormat::Wav => {
            let path = args.output.context("WAV output requires --output <path>")?;
            let soundfont = load_soundfont(args.soundfont, args.offline)?;
            music_midi::render::AudioRenderer::new(soundfont)?
                .sample_rate(args.sample_rate)
                .tail_seconds(args.tail)
                .render_bytes_to_wav(&bytes, path)
                .with_context(|| format!("rendering {path}"))?;
        }
        MidiOutputFormat::Play => {
            if args.output.is_some() {
                bail!("--output cannot be used with --format play");
            }
            let smf = midly::Smf::parse(&bytes).context("parsing generated MIDI")?;
            let mut player = music_midi::playback::MidiPlayer::connect_default()
                .context("opening the default MIDI output")?;
            player.play_blocking(&smf).context("playing MIDI")?;
        }
    }
    Ok(())
}

fn load_soundfont(path: Option<&str>, offline: bool) -> Result<music_midi::soundfont::SoundFont> {
    match (path, offline) {
        (Some(_), true) => bail!("--offline cannot be combined with --soundfont"),
        (Some(path), false) => music_midi::soundfont::SoundFont::from_path(path)
            .with_context(|| format!("loading SoundFont {path}")),
        (None, true) => music_midi::soundfont::SoundFont::general_user_gs_offline()
            .context("loading cached GeneralUser GS SoundFont"),
        (None, false) => music_midi::soundfont::SoundFont::general_user_gs()
            .context("loading GeneralUser GS SoundFont"),
    }
}
