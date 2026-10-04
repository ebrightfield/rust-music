//! `rhythm-drill` — generate rhythmic-dictation / clapping exercises.
//!
//! Produces a sequence of measures of rhythm in a chosen time signature, with a
//! style feel (straight / swing / latin) and a syncopation level. Output is a
//! text rhythm map by default, or staff notation (SVG/PNG) when `-o` is given.
//! Notation renders the rhythm on a single repeated pitch (the convention for
//! rhythm-only drills).

use anyhow::Result;

use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;

use super::notation_out::{render_events_to_file, ClefChoice, NotationEvent};
use music_engraver::layout::key_signature::KeySignature;

pub struct RhythmDrillArgs {
    pub time_sig: Option<String>,
    pub style: Option<String>,
    pub syncopation: u8,
    pub measures: usize,
    pub clef: Option<String>,
    pub seed: Option<u64>,
    pub output: Option<String>,
    pub verbose: bool,
}

/// Rhythmic feel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    Straight,
    Swing,
    Latin,
}

impl Style {
    fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s.unwrap_or("straight").to_lowercase().as_str() {
            "straight" => Ok(Style::Straight),
            "swing" => Ok(Style::Swing),
            "latin" => Ok(Style::Latin),
            other => anyhow::bail!("unknown style: '{other}' (options: straight, swing, latin)"),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Style::Straight => "straight",
            Style::Swing => "swing",
            Style::Latin => "latin",
        }
    }
}

/// Parse "N/D" into (numerator, denominator). Defaults to 4/4.
fn parse_time_sig(s: Option<&str>) -> Result<(u8, u8)> {
    let s = s.unwrap_or("4/4");
    let (n, d) = s
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!("time signature must be N/D, got '{s}'"))?;
    let num: u8 = n
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid numerator in time signature '{s}'"))?;
    let den: u8 = d
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid denominator in time signature '{s}'"))?;
    anyhow::ensure!(num >= 1, "time signature numerator must be ≥ 1");
    anyhow::ensure!(
        matches!(den, 1 | 2 | 4 | 8 | 16),
        "time signature denominator must be one of 1, 2, 4, 8, 16 (got {den})"
    );
    Ok((num, den))
}

/// A small deterministic PRNG (SplitMix64) so drills are reproducible by seed
/// without depending on the `rand` crate in the default build.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        // Mix the seed so seed=0 isn't degenerate.
        Rng(seed.wrapping_add(0x9E37_79B9_7F4A_7C15))
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Choose an index in `0..n` weighted by `weights` (len must equal `n`).
    fn weighted(&mut self, weights: &[u32]) -> usize {
        let total: u32 = weights.iter().sum();
        if total == 0 {
            return 0;
        }
        let mut pick = (self.next_u64() % u64::from(total)) as u32;
        for (i, &w) in weights.iter().enumerate() {
            if pick < w {
                return i;
            }
            pick -= w;
        }
        weights.len() - 1
    }
}

/// A rhythm cell filling one quarter-note beat. Each variant is a fixed list of
/// (duration, is_rest) summing to exactly one quarter (32 ticks).
#[derive(Clone, Copy)]
enum BeatCell {
    /// Quarter note.
    Quarter,
    /// Two eighth notes.
    TwoEighths,
    /// Eighth note + eighth rest (on the beat).
    EighthThenRest,
    /// Eighth rest + eighth note (off the beat — syncopated).
    RestThenEighth,
    /// Dotted-eighth + sixteenth.
    DottedEighthSixteenth,
    /// Four sixteenths.
    FourSixteenths,
}

impl BeatCell {
    /// (duration, is_rest) events that fill the beat.
    fn events(self) -> Vec<(Duration, bool)> {
        let eighth = Duration::EIGHTH;
        let sixteenth = Duration::SIXTEENTH;
        let dotted_eighth = Duration::new(DurationKind::Eighth, 1);
        match self {
            BeatCell::Quarter => vec![(Duration::QTR, false)],
            BeatCell::TwoEighths => vec![(eighth, false), (eighth, false)],
            BeatCell::EighthThenRest => vec![(eighth, false), (eighth, true)],
            BeatCell::RestThenEighth => vec![(eighth, true), (eighth, false)],
            BeatCell::DottedEighthSixteenth => vec![(dotted_eighth, false), (sixteenth, false)],
            BeatCell::FourSixteenths => vec![
                (sixteenth, false),
                (sixteenth, false),
                (sixteenth, false),
                (sixteenth, false),
            ],
        }
    }

    fn symbol(self) -> &'static str {
        match self {
            BeatCell::Quarter => "♩",
            BeatCell::TwoEighths => "♫",
            BeatCell::EighthThenRest => "♪𝄾",
            BeatCell::RestThenEighth => "𝄾♪",
            BeatCell::DottedEighthSixteenth => "♪.♬",
            BeatCell::FourSixteenths => "♬♬",
        }
    }
}

/// All beat cells in palette order, with selection weights for a syncopation
/// level (0 = none, 3 = heavy). Index aligns with `PALETTE`.
const PALETTE: [BeatCell; 6] = [
    BeatCell::Quarter,
    BeatCell::TwoEighths,
    BeatCell::EighthThenRest,
    BeatCell::RestThenEighth,
    BeatCell::DottedEighthSixteenth,
    BeatCell::FourSixteenths,
];

/// Weights per syncopation level. Higher syncopation favors off-beat and
/// dotted/sixteenth cells.
fn weights_for(syncopation: u8, style: Style) -> [u32; 6] {
    //                 Qtr  2x8  8+r  r+8  8.x16  4x16
    let base = match syncopation {
        0 => [60, 30, 5, 0, 3, 2],
        1 => [35, 30, 8, 10, 10, 7],
        2 => [20, 25, 10, 22, 13, 10],
        _ => [10, 20, 12, 30, 16, 12],
    };
    // Latin leans on sixteenth-driven and off-beat figures; swing leans on
    // eighth pairs (which the swing feel will later inflect).
    let mut w = base;
    match style {
        Style::Latin => {
            w[3] += 8; // rest-then-eighth
            w[5] += 8; // four sixteenths
        }
        Style::Swing => {
            w[1] += 10; // two eighths
        }
        Style::Straight => {}
    }
    w
}

/// One measure: the per-beat cells chosen (for text display) drive the flattened
/// (Duration, is_rest) event stream (for notation).
struct Measure {
    cells: Vec<BeatCell>,
}

impl Measure {
    /// Flattened (duration, is_rest) events across all beats.
    fn events(&self) -> Vec<(Duration, bool)> {
        self.cells.iter().flat_map(|c| c.events()).collect()
    }
}

/// Build one measure by choosing a weighted cell for each quarter-note beat.
fn build_measure(beats: usize, weights: &[u32; 6], rng: &mut Rng) -> Measure {
    let cells = (0..beats).map(|_| PALETTE[rng.weighted(weights)]).collect();
    Measure { cells }
}

/// Number of quarter-note beats in a measure for a given time signature.
/// E.g. 4/4 → 4, 3/4 → 3, 6/8 → 3 (dotted-quarter beats counted as ×2 eighths
/// ≈ we still fill in quarter cells for simplicity), 2/2 → 4.
fn quarter_beats(num: u8, den: u8) -> usize {
    // Total measure value in quarters = num * (4 / den).
    // Use rational arithmetic and floor to at least 1.
    let quarters = (num as usize * 4) / den as usize;
    quarters.max(1)
}

pub fn run(args: RhythmDrillArgs) -> Result<()> {
    let (num, den) = parse_time_sig(args.time_sig.as_deref())?;
    let style = Style::from_str_opt(args.style.as_deref())?;
    anyhow::ensure!(
        args.syncopation <= 3,
        "syncopation must be 0–3 (got {})",
        args.syncopation
    );
    anyhow::ensure!(args.measures >= 1, "measures must be ≥ 1");

    // Validate every argument before generating anything, so a bad `--clef`
    // fails whether or not `-o` was passed.
    let clef = ClefChoice::from_str_opt(args.clef.as_deref())?;

    let beats = quarter_beats(num, den);
    let weights = weights_for(args.syncopation, style);
    let mut rng = Rng::new(args.seed.unwrap_or(0));

    // Generate all measures.
    let measures: Vec<Measure> = (0..args.measures)
        .map(|_| build_measure(beats, &weights, &mut rng))
        .collect();

    // Output dispatch.
    if let Some(ref path) = args.output {
        // Render rhythm on a single repeated pitch on the staff's middle line,
        // the convention for rhythm-only drills. Rests are emitted as rests so
        // the notation matches the text output for the same seed.
        let drum_pitch = match clef {
            ClefChoice::Bass => Pitch::new(Note::D, 3),
            ClefChoice::Alto => Pitch::new(Note::C, 4),
            ClefChoice::Tenor => Pitch::new(Note::A, 3),
            ClefChoice::Treble | ClefChoice::Treble8 => Pitch::new(Note::B, 4),
        };
        let events = rhythm_to_events(&measures, drum_pitch);
        let n = render_events_to_file(&events, clef, KeySignature::Open, (num, den), path)?;
        if args.verbose {
            eprintln!("wrote {path} ({n} bytes)");
        } else {
            println!("Wrote {path}");
        }
        return Ok(());
    }

    print_text(&measures, num, den, style, args.syncopation);
    Ok(())
}

/// Convert generated rhythm measures into a notation event stream on a single
/// pitch. Rests are carried through as rests so the engraved rhythm matches the
/// text output — dropping them would shorten every measure containing one.
fn rhythm_to_events(measures: &[Measure], pitch: Pitch) -> Vec<NotationEvent> {
    let mut events = Vec::new();
    for measure in measures {
        for (dur, is_rest) in measure.events() {
            events.push(if is_rest {
                NotationEvent::rest(dur)
            } else {
                NotationEvent::note(pitch, dur)
            });
        }
    }
    events
}

fn print_text(measures: &[Measure], num: u8, den: u8, style: Style, syncopation: u8) {
    println!("=== Rhythm Drill ===");
    println!(
        "Time: {num}/{den}   Style: {}   Syncopation: {}/3",
        style.label(),
        syncopation
    );
    println!("Measures: {}", measures.len());
    println!();
    for (i, measure) in measures.iter().enumerate() {
        let symbols: Vec<&str> = measure.cells.iter().map(|c| c.symbol()).collect();
        println!("  m{}: {}", i + 1, symbols.join("  "));
    }
    if style == Style::Swing {
        println!();
        println!("(swing feel: play eighth-note pairs as long-short)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_time_sig_default_and_values() {
        assert_eq!(parse_time_sig(None).unwrap(), (4, 4));
        assert_eq!(parse_time_sig(Some("3/4")).unwrap(), (3, 4));
        assert_eq!(parse_time_sig(Some("6/8")).unwrap(), (6, 8));
        assert_eq!(parse_time_sig(Some("2/2")).unwrap(), (2, 2));
        assert!(parse_time_sig(Some("4")).is_err());
        assert!(parse_time_sig(Some("4/3")).is_err()); // unsupported denominator
        assert!(parse_time_sig(Some("x/4")).is_err());
    }

    #[test]
    fn style_parsing() {
        assert_eq!(Style::from_str_opt(None).unwrap(), Style::Straight);
        assert_eq!(Style::from_str_opt(Some("swing")).unwrap(), Style::Swing);
        assert_eq!(Style::from_str_opt(Some("latin")).unwrap(), Style::Latin);
        assert!(Style::from_str_opt(Some("polka")).is_err());
    }

    #[test]
    fn quarter_beats_math() {
        assert_eq!(quarter_beats(4, 4), 4);
        assert_eq!(quarter_beats(3, 4), 3);
        assert_eq!(quarter_beats(2, 2), 4);
        assert_eq!(quarter_beats(6, 8), 3);
        assert_eq!(quarter_beats(1, 4), 1);
    }

    #[test]
    fn each_beat_cell_sums_to_a_quarter() {
        for cell in PALETTE {
            let total: usize = cell.events().iter().map(|(d, _)| d.ticks()).sum();
            assert_eq!(
                total,
                Duration::QTR.ticks(),
                "cell must fill exactly one beat"
            );
        }
    }

    #[test]
    fn measure_fills_exactly() {
        let weights = weights_for(2, Style::Straight);
        let mut rng = Rng::new(42);
        let measure = build_measure(4, &weights, &mut rng);
        let total: usize = measure.events().iter().map(|(d, _)| d.ticks()).sum();
        assert_eq!(
            total,
            4 * Duration::QTR.ticks(),
            "4/4 measure must sum to 4 quarters"
        );
    }

    #[test]
    fn deterministic_by_seed() {
        let weights = weights_for(2, Style::Latin);
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        let ma = build_measure(4, &weights, &mut a).events();
        let mb = build_measure(4, &weights, &mut b).events();
        assert_eq!(ma.len(), mb.len());
        for (x, y) in ma.iter().zip(mb.iter()) {
            assert_eq!(x.0.ticks(), y.0.ticks());
            assert_eq!(x.1, y.1);
        }
    }

    #[test]
    fn run_text_output() {
        let args = RhythmDrillArgs {
            time_sig: Some("4/4".into()),
            style: Some("swing".into()),
            syncopation: 2,
            measures: 4,
            clef: None,
            seed: Some(1),
            output: None,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn run_svg_output() {
        let tmp = tempfile::TempDir::new().unwrap();
        let out = tmp.path().join("drill.svg");
        let args = RhythmDrillArgs {
            time_sig: Some("3/4".into()),
            style: Some("latin".into()),
            syncopation: 3,
            measures: 2,
            clef: Some("treble-8".into()),
            seed: Some(99),
            output: Some(out.to_string_lossy().into_owned()),
            verbose: false,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.contains("<svg"));
        assert!(content.contains("</svg>"));
    }

    #[test]
    fn rejects_bad_syncopation() {
        let args = RhythmDrillArgs {
            time_sig: None,
            style: None,
            syncopation: 9,
            measures: 4,
            clef: None,
            seed: None,
            output: None,
            verbose: false,
        };
        assert!(run(args).is_err());
    }
}
