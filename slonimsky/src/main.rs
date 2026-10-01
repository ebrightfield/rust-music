use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd;

/// Note shown in `--help` so the feature-gated subcommand isn't mistaken for
/// missing when it is simply absent from a default build.
#[cfg(not(feature = "midi"))]
const FEATURE_HELP: &str = "MIDI capabilities require `--features midi`:\n  \
    ear-training              Generate MIDI ear-training quizzes\n  \
    sequence --format midi    Write Standard MIDI Files\n  \
    sequence --format wav     Render audio through an SF2 SoundFont\n  \
    sequence --format play    Play through the default MIDI output";

#[cfg(feature = "midi")]
const FEATURE_HELP: &str =
    "Built with `midi`: ear training plus sequence MIDI, WAV, and playback output are available.";

#[derive(Parser)]
#[command(
    name = "slonimsky",
    version,
    about = "Music-theory CLI",
    after_help = FEATURE_HELP
)]
struct Cli {
    /// Output file path (format inferred from extension)
    #[arg(short, long, global = true)]
    output: Option<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// SVG color theme (default, dark, print, colorful)
    #[arg(short, long, global = true)]
    theme: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Draw a pitch-class circle diagram
    PitchCircle {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Root pitch class (default: first in input)
        #[arg(long)]
        root: Option<String>,

        /// Draw lines between adjacent pitch classes
        #[arg(long)]
        show_intervals: bool,

        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Draw a fretboard shape diagram
    Fretboard {
        /// Fret notation string (e.g. x-3-2-0-1-0)
        frets: String,

        /// Tuning name, comma-separated pitches/MIDI values, or @path
        #[arg(long, default_value = "standard")]
        tuning: String,

        /// Output format: svg, positions, or fret-spec (otherwise inferred from --output)
        #[arg(long)]
        format: Option<String>,

        /// Include spelled notes in ASCII output
        #[arg(long)]
        notes: bool,

        /// Render ASCII strings from highest to lowest
        #[arg(long)]
        high_to_low: bool,

        /// Orientation: vertical (default) or horizontal
        #[arg(long)]
        orientation: Option<String>,

        /// Diagram title
        #[arg(long)]
        title: Option<String>,

        /// Number of frets to display
        #[arg(long)]
        num_frets: Option<u8>,
    },
    /// Generate, rank, filter, and render melodic fretboard realizations
    MelodicShapes {
        /// Notes or pitch classes, individually or comma-separated (e.g. B,D#,F#,A#)
        #[arg(required = true)]
        input: Vec<String>,

        /// Output format override: text, json, svg, png, or pdf (otherwise inferred from --output)
        #[arg(long)]
        format: Option<String>,

        /// Shape categories, repeatable/comma-separated: all, open, simple, 2nps, 2-3nps, 3-2nps, 3nps, exhaustive
        #[arg(long = "category", value_delimiter = ',')]
        categories: Vec<String>,

        /// Restrict starting notes, repeatable/comma-separated (default: every input note)
        #[arg(long = "starting-note", value_delimiter = ',')]
        starting_notes: Vec<String>,

        /// Tuning name, comma-separated pitches/MIDI values, or @path
        #[arg(long, default_value = "standard")]
        tuning: String,

        /// Root note to highlight (default: first input note)
        #[arg(long)]
        root: Option<String>,

        /// Keep only shapes with this playability cost or lower
        #[arg(long)]
        max_score: Option<usize>,

        /// Keep only shapes spanning at most this many frets
        #[arg(long)]
        max_span: Option<u8>,

        /// Maximum total shapes after filtering and sorting
        #[arg(long)]
        limit: Option<usize>,

        /// Sort by category, score, span, or position
        #[arg(long, default_value = "category")]
        sort: String,

        /// Remove identical fingerboard paths appearing in multiple categories
        #[arg(long)]
        deduplicate: bool,

        /// Diagram orientation: horizontal or vertical
        #[arg(long, default_value = "horizontal")]
        orientation: String,

        /// Number of diagram columns in rendered output
        #[arg(long, default_value = "3")]
        columns: usize,

        /// Width of each rendered diagram tile
        #[arg(long, default_value = "320")]
        tile_width: u32,

        /// Height of each rendered diagram tile
        #[arg(long, default_value = "220")]
        tile_height: u32,

        /// Gap between rendered diagram tiles
        #[arg(long, default_value = "20")]
        gap: u32,

        /// Force the first displayed fret instead of choosing it from each shape
        #[arg(long)]
        start_fret: Option<u8>,

        /// Force the number of displayed frets instead of fitting each shape
        #[arg(long)]
        num_frets: Option<u8>,

        /// Automatic fret-window padding
        #[arg(long, default_value = "1")]
        fret_padding: u8,

        /// Omit per-shape titles from rendered output
        #[arg(long)]
        no_titles: bool,

        /// Do not highlight occurrences of the root
        #[arg(long)]
        no_root_markers: bool,

        /// Hide fret numbers
        #[arg(long)]
        no_fret_numbers: bool,

        /// Hide open-string names
        #[arg(long)]
        no_string_names: bool,

        /// PNG resolution in dots per inch
        #[arg(long, default_value = "144")]
        dpi: f32,

        /// Document title for rendered output
        #[arg(long)]
        title: Option<String>,
    },
    /// Name a chord from pitch classes with configurable inference and display policies
    Name {
        /// Pitch classes (integer or note name)
        #[arg(required = true)]
        pcs: Vec<String>,

        /// Explicit chord root; disables automatic root inference
        #[arg(long)]
        root: Option<String>,

        /// Explicit bass note; enables bass-aware inversion inference
        #[arg(long)]
        bass: Option<String>,

        /// Minimum distinct pitch classes required for inferred slash chords
        #[arg(long)]
        slash_threshold: Option<usize>,

        /// Inference preset: default, strict, jazz, or pop
        #[arg(long, default_value = "default")]
        naming_style: String,

        /// Prefer add9/add11/add13 over extension alterations
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        prefer_add: Option<bool>,

        /// Include omitted tones such as no5
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        show_omissions: Option<bool>,

        /// Distinguish sixth chords from thirteenth chords without a seventh
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        distinguish_sixth: Option<bool>,

        /// Detect and report naming ambiguities
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        report_ambiguities: Option<bool>,

        /// Extension rendering: none, strict, highest, or highest-unless-one
        #[arg(long, default_value = "none")]
        extension_style: String,

        /// Major-quality symbol: delta, maj, capital-m, or lower-maj
        #[arg(long, default_value = "maj")]
        major_symbol: String,

        /// Accidental rendering: unicode or ascii
        #[arg(long, default_value = "unicode")]
        accidentals: String,

        /// Render suspended-fourth qualities explicitly as sus4
        #[arg(long, num_args = 0..=1, default_missing_value = "true")]
        explicit_sus4: Option<bool>,

        /// Spaces between the root and chord quality
        #[arg(long, default_value = "0")]
        root_spacing: usize,

        /// Spaces between the chord quality and slash
        #[arg(long, default_value = "0")]
        quality_slash_spacing: usize,

        /// Spaces after the slash
        #[arg(long, default_value = "0")]
        slash_spacing: usize,

        /// Output format: text or json (otherwise inferred from --output)
        #[arg(long)]
        format: Option<String>,
    },
    /// Spell a chord symbol into notes
    Spell {
        /// Chord symbol (e.g. Cmaj7, Dm9)
        symbol: String,

        /// Output format: notes (default), pcs, intervals, or all
        #[arg(long, default_value = None)]
        format: Option<String>,
    },
    /// Print or draw an interval matrix for a PcSet
    IntervalMatrix {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Diagram title
        #[arg(long)]
        title: Option<String>,

        /// Show full 12-element interval vector instead of reduced 6-element
        #[arg(long)]
        full: bool,
    },
    /// Print or draw the interval-class vector for a PcSet
    IntervalVector {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Diagram title
        #[arg(long)]
        title: Option<String>,

        /// Show full 12-element vector instead of reduced 6-element
        #[arg(long)]
        full: bool,
    },
    /// Draw pitches on a line with arcs showing adjacent intervals
    IntervalLinear {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Diagram title
        #[arg(long)]
        title: Option<String>,
    },
    /// Query directed intervals between selected pitch-class pairs
    IntervalPairs {
        /// Pitch classes defining the query set
        #[arg(required = true)]
        input: Vec<String>,

        /// Pair to query as FROM,TO; repeatable (default: every unordered pair)
        #[arg(long = "pair")]
        pairs: Vec<String>,

        /// Output format: text or json (otherwise inferred from --output)
        #[arg(long)]
        format: Option<String>,
    },
    /// Find known chords/scales that contain a given PcSet
    Superchords {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Minimum size of superchords to find (default: input size + 1)
        #[arg(long)]
        min_size: Option<u8>,

        /// Maximum size of superchords to find (default: 12)
        #[arg(long)]
        max_size: Option<u8>,
    },
    /// Compute transpositional and inversional symmetry orbits of a PcSet
    Orbits {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Symmetry type: transpositional, inversional, or both (default)
        #[arg(long = "type")]
        sym_type: Option<String>,
    },
    /// Compute the prime form of a PcSet (Rahn's algorithm)
    PrimeForm {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,
    },
    /// Classify a PcSet by Forte number (prime form + table lookup)
    Forte {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,
    },
    /// Find common pitch classes between two or more sets
    CommonTones {
        /// Sets as comma-separated groups (e.g. C,E,G D,F,A)
        #[arg(required = true)]
        sets: Vec<String>,
    },
    /// Query containment: which scales contain this chord, or which chords are in this scale
    Contains {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Pool to search: chords, scales, or both (default)
        #[arg(long = "in")]
        pool: Option<String>,

        /// Direction: super (what contains input) or sub (what input contains). Auto-detected from input size if omitted.
        #[arg(long)]
        direction: Option<String>,

        /// Maximum number of results
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Rank known chords/scales by distance to a given PcSet
    Closest {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Distance metric: symmetric-diff (default)
        #[arg(long)]
        metric: Option<String>,

        /// Pool to search: chords, scales, or both (default)
        #[arg(long)]
        pool: Option<String>,

        /// Maximum number of results (default: 20)
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Search every chord-shape classification and filter the results
    ChordDictionary {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Output format: text, json, or svg (otherwise inferred from --output)
        #[arg(long)]
        format: Option<String>,

        /// Tuning name, comma-separated pitches/MIDI values, or @path
        #[arg(long, default_value = "standard")]
        tuning: String,

        /// Classifications, repeatable/comma-separated: all, playable, wide, nontransposable, high-fret, unplayable
        #[arg(long = "classification", value_delimiter = ',')]
        classifications: Vec<String>,

        /// Exact low-to-high pitch voicing, including octaves (for example C4,E4,G4)
        #[arg(long)]
        voicing: Option<String>,

        /// Low-to-high pitch-class family, ignoring octaves (for example E,G,C)
        #[arg(long)]
        family: Option<String>,

        /// Required bass note
        #[arg(long)]
        bass: Option<String>,

        /// Open-string policy: any, required, or excluded
        #[arg(long, default_value = "any")]
        open_strings: String,

        /// Lowest allowed fret, including open strings as fret 0
        #[arg(long)]
        min_fret: Option<u8>,

        /// Highest allowed fret
        #[arg(long)]
        max_fret: Option<u8>,

        /// Maximum fret span
        #[arg(long, default_value = "4")]
        max_span: Option<u8>,

        /// Maximum number of results after filtering
        #[arg(long, default_value = "20")]
        max_results: Option<usize>,
    },
    /// Find voice-leadings between two voicings
    VoiceLeading {
        /// Starting voicing as comma-separated pitches (e.g. C4,E4,G4)
        #[arg(long)]
        from: String,

        /// Target notes as comma-separated note names (e.g. F,A,C)
        #[arg(long)]
        to: String,

        /// Maximum number of results to show
        #[arg(long)]
        limit: Option<usize>,

        /// Forbid voice crossings
        #[arg(long)]
        no_crossings: bool,

        /// Distance metric: l1, linf, or weighted
        #[arg(long, default_value = "l1")]
        metric: String,

        /// Comma-separated lowest-to-highest voice weights for the weighted metric
        #[arg(long)]
        weights: Option<String>,
    },
    /// Generate a bounded melodic sequence from interval, harmony, and rhythm patterns
    Sequence {
        /// Harmonic contexts as comma-separated note groups (e.g. C,D,E,F,G,A,B or C,E,G)
        #[arg(required = true)]
        harmony: Vec<String>,

        /// Duration of each harmony: one value for all or comma-separated values per harmony
        #[arg(long, default_value = "w")]
        chord_durations: String,

        /// Nested interval levels; commas separate intervals and slashes separate levels
        #[arg(long, default_value = "1")]
        pattern: String,

        /// Interval used after all pattern levels complete a cycle
        #[arg(long, default_value = "1", allow_hyphen_values = true)]
        master_step: i8,

        /// Comma-separated rhythm cycle: w, h, q, 8, 16, 32, 64, 128; dots allowed
        #[arg(long, default_value = "8")]
        rhythm: String,

        /// Starting pitch with octave
        #[arg(long, default_value = "C4")]
        start: String,

        /// Lowest allowed pitch
        #[arg(long, default_value = "C3")]
        low: String,

        /// Highest allowed pitch
        #[arg(long, default_value = "C6")]
        high: String,

        /// Initial direction: up or down
        #[arg(long, default_value = "up")]
        direction: String,

        /// Boundary behavior: reflect, ricochet, start-over, wrap, or stop
        #[arg(long, default_value = "reflect")]
        turnaround: String,

        /// Number of events to generate
        #[arg(long, default_value = "16")]
        length: usize,

        /// Output format override: text, json, midi, wav, or play (otherwise inferred from --output)
        #[arg(long)]
        format: Option<String>,

        /// Tempo for MIDI, WAV, and playback output
        #[arg(long, default_value = "120")]
        bpm: f32,

        /// Pulses per quarter note for MIDI output (non-zero multiple of 32)
        #[arg(long, default_value = "480")]
        ppq: u16,

        /// SF2 SoundFont path for WAV rendering (default: cached/downloaded GeneralUser GS)
        #[arg(long)]
        soundfont: Option<String>,

        /// Do not download the default SoundFont when it is absent
        #[arg(long)]
        offline: bool,

        /// WAV sample rate in hertz
        #[arg(long, default_value = "48000")]
        sample_rate: u32,

        /// WAV decay tail in seconds
        #[arg(long, default_value = "1")]
        tail: f32,
    },
    /// Validate and render a declarative music-ron document
    Render {
        /// Input .ron path, or - to read from stdin
        input: String,

        /// Output format override: text, json, svg, png, or pdf
        #[arg(long)]
        format: Option<String>,

        /// PNG resolution in dots per inch
        #[arg(long, default_value = "144")]
        dpi: f32,
    },
    /// Plan smoothest voice-leadings through a chord sequence
    Progression {
        /// Chords as comma-separated groups (e.g. C,E,G F,A,C G,B,D)
        #[arg(required = true)]
        chords: Vec<String>,

        /// Forbid voice crossings
        #[arg(long)]
        no_crossings: bool,

        /// Distance metric for greedy step selection: l1, linf, or weighted
        #[arg(long, default_value = "l1")]
        metric: String,

        /// Comma-separated lowest-to-highest voice weights for the weighted metric
        #[arg(long)]
        weights: Option<String>,
    },
    /// Enumerate canonical voicings of a 3- or 4-note chord
    Voicings {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Inclusive pitch range as LOW..HIGH (for example, C3..C6)
        #[arg(long)]
        range: Option<String>,

        /// Minimum adjacent-voice spacing in semitones
        #[arg(long)]
        min_spacing: Option<u8>,

        /// Maximum adjacent-voice spacing in semitones
        #[arg(long)]
        max_spacing: Option<u8>,

        /// Number of tuning strings that must sound
        #[arg(long)]
        strings: Option<usize>,

        /// Named tuning, comma-separated open pitches/MIDI values, or @path
        #[arg(long)]
        tuning: Option<String>,

        /// Chord-tone doubling policy: allow, forbid, or require
        #[arg(long, default_value = "forbid")]
        doubling: String,

        /// Maximum number of voicings to show
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Generate per-key arpeggio sheets across fretboard positions
    ArpeggioDictionary {
        /// Pitch classes, note names, or comma-separated list (chord type)
        #[arg(required = true)]
        input: Vec<String>,

        /// Tuning name, comma-separated pitches/MIDI values, or @path
        #[arg(long, default_value = "standard")]
        tuning: String,

        /// Keys to include: "all" (default) or comma-separated (e.g. C,G,D)
        #[arg(long)]
        keys: Option<String>,

        /// Number of fretboard positions per key (default: 5)
        #[arg(long, default_value = "5")]
        positions: usize,

        /// Maximum fret span for playable shapes (default: 4)
        #[arg(long, default_value = "4")]
        max_span: u8,
    },
    /// Generate a scale book: all modes of a parent scale across keys
    ScaleBook {
        /// Seven-note family slug; run `scale-catalog` to list all 22
        scale: String,

        /// Keys to include: "all" (default) or comma-separated (e.g. C,G,D)
        #[arg(long)]
        keys: Option<String>,
    },
    /// List, expand, or identify the complete seven-note scale catalog
    ScaleCatalog {
        /// Scale family to inspect (default: all 22 families)
        scale: Option<String>,

        /// Identify a tonic-first seven-note scale from pitches or pitch classes
        #[arg(long, num_args = 1.., conflicts_with = "scale")]
        identify: Vec<String>,

        /// Include all seven rotations of each selected family
        #[arg(long, conflicts_with = "identify")]
        modes: bool,

        /// Output format: text or json (otherwise inferred from --output)
        #[arg(long)]
        format: Option<String>,
    },
    /// Analyze, transform, and compare melodic contours
    Contour {
        /// Pitches with octaves or MIDI values, separated by spaces or commas
        #[arg(required = true)]
        input: Vec<String>,

        /// A second pitch sequence to compare against
        #[arg(long, num_args = 1..)]
        compare: Vec<String>,

        /// Applied contour transformation: original, retrograde, inversion, or retrograde-inversion
        #[arg(long, default_value = "original")]
        transform: String,

        /// Output format: text or json (otherwise inferred from --output)
        #[arg(long)]
        format: Option<String>,
    },
    /// Rotate a pitch-class scale directly to another modal tonic
    ScaleRotate {
        /// Scale pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Signed number of scale degrees to rotate forward
        #[arg(long, default_value = "1", allow_hyphen_values = true)]
        steps: isize,

        /// Output format: text or json (otherwise inferred from --output)
        #[arg(long)]
        format: Option<String>,
    },
    /// Generate a practice sheet for a key and scale
    PracticeSheet {
        /// Key (default: C). A note name like C, G, Bb, F#
        #[arg(long)]
        key: Option<String>,

        /// Scale: major, melodic-minor, harmonic-minor, harmonic-major
        #[arg(long)]
        scale: Option<String>,
    },
    /// Generate MIDI ear-training quizzes (intervals, chords, progressions)
    #[cfg(feature = "midi")]
    EarTraining {
        /// Quiz type: intervals (default)
        #[arg(long = "type")]
        quiz_type: Option<String>,

        /// Number of quiz items (default: 10)
        #[arg(long, default_value = "10")]
        count: usize,

        /// Random seed for reproducible quizzes
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Generate sight-reading melodic exercises
    SightReading {
        /// Key (default: C). A note name like C, G, Bb, F#
        #[arg(long)]
        key: Option<String>,

        /// Scale: major, melodic-minor, harmonic-minor, harmonic-major
        #[arg(long)]
        scale: Option<String>,

        /// Difficulty level 1–5 (default: 2)
        #[arg(long, default_value = "2")]
        difficulty: u8,

        /// Number of measures to generate (default: 4)
        #[arg(long, default_value = "4")]
        measures: usize,

        /// Random seed for reproducible exercises
        #[arg(long)]
        seed: Option<u64>,

        /// Clef for notation output: treble (default), treble-8 (guitar), bass
        #[arg(long)]
        clef: Option<String>,
    },
    /// Generate a rhythm-dictation / clapping drill (text or staff notation)
    RhythmDrill {
        /// Time signature as N/D (default: 4/4)
        #[arg(long)]
        time_sig: Option<String>,

        /// Feel: straight (default), swing, latin
        #[arg(long)]
        style: Option<String>,

        /// Syncopation level 0–3 (default: 1)
        #[arg(long, default_value = "1")]
        syncopation: u8,

        /// Number of measures to generate (default: 4)
        #[arg(long, default_value = "4")]
        measures: usize,

        /// Clef for notation output: treble (default), treble-8 (guitar), bass
        #[arg(long)]
        clef: Option<String>,

        /// Random seed for reproducible drills
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Analyze a chord progression: key estimation, Roman numerals, common tones, voice-leading cost
    Analyze {
        /// Chords as comma-separated groups (e.g. C,E,G F,A,C G,B,D)
        #[arg(required = true)]
        chords: Vec<String>,

        /// Key (e.g. C, G, Bb). If omitted, key is estimated from the progression.
        #[arg(long)]
        key: Option<String>,

        /// Scale: major, natural-minor, harmonic-minor, melodic-minor, harmonic-major (default: major)
        #[arg(long)]
        scale: Option<String>,

        /// Output format: text (default) or json
        #[arg(long)]
        format: Option<String>,
    },
    /// Annotate a voiced progression with voice-leading quality metrics
    Annotate {
        /// Chords as comma-separated pitched groups (e.g. C4,E4,G4 C4,F4,A4)
        #[arg(required = true)]
        chords: Vec<String>,

        /// Key (e.g. C, G, Bb) for context
        #[arg(long)]
        key: Option<String>,

        /// Scale for context (e.g. major, natural-minor)
        #[arg(long)]
        scale: Option<String>,

        /// Flag voice crossings as warnings
        #[arg(long)]
        no_crossings: bool,

        /// Output format: text (default) or json
        #[arg(long)]
        format: Option<String>,
    },
    /// Find all N-note subsets of a chord or scale
    Subchords {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Size of subchords to enumerate (default: 3)
        #[arg(long, default_value = "3")]
        size: u8,

        /// Try to name each subchord
        #[arg(long)]
        name: bool,

        /// Report sets and names relative to prime form (rooted on pitch-class
        /// 0) instead of in the key of the queried input
        #[arg(long)]
        relative: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::PitchCircle {
            input,
            root,
            show_intervals,
            title,
        } => {
            cmd::pitch_circle::run(cmd::pitch_circle::PitchCircleArgs {
                input,
                output: cli.output,
                theme: cli.theme,
                root,
                show_intervals,
                title,
                verbose: cli.verbose,
            })?;
        }
        Commands::Fretboard {
            frets,
            tuning,
            format,
            notes,
            high_to_low,
            orientation,
            title,
            num_frets,
        } => {
            cmd::fretboard::run(cmd::fretboard::FretboardArgs {
                frets,
                output: cli.output,
                theme: cli.theme,
                tuning,
                format,
                notes,
                high_to_low,
                orientation,
                title,
                num_frets,
                verbose: cli.verbose,
            })?;
        }
        Commands::MelodicShapes {
            input,
            format,
            categories,
            starting_notes,
            tuning,
            root,
            max_score,
            max_span,
            limit,
            sort,
            deduplicate,
            orientation,
            columns,
            tile_width,
            tile_height,
            gap,
            start_fret,
            num_frets,
            fret_padding,
            no_titles,
            no_root_markers,
            no_fret_numbers,
            no_string_names,
            dpi,
            title,
        } => {
            cmd::melodic_shapes::run(cmd::melodic_shapes::MelodicShapesArgs {
                input,
                output: cli.output,
                format,
                categories,
                starting_notes,
                tuning,
                root,
                max_score,
                max_span,
                limit,
                sort,
                deduplicate,
                orientation,
                columns,
                tile_width,
                tile_height,
                gap,
                start_fret,
                num_frets,
                fret_padding,
                no_titles,
                no_root_markers,
                no_fret_numbers,
                no_string_names,
                dpi,
                theme: cli.theme,
                title,
                verbose: cli.verbose,
            })?;
        }
        Commands::Name {
            pcs,
            root,
            bass,
            slash_threshold,
            naming_style,
            prefer_add,
            show_omissions,
            distinguish_sixth,
            report_ambiguities,
            extension_style,
            major_symbol,
            accidentals,
            explicit_sus4,
            root_spacing,
            quality_slash_spacing,
            slash_spacing,
            format,
        } => {
            cmd::name::run(cmd::name::NameArgs {
                pcs,
                root,
                bass,
                slash_threshold,
                naming_style,
                prefer_add,
                show_omissions,
                distinguish_sixth,
                report_ambiguities,
                extension_style,
                major_symbol,
                accidentals,
                explicit_sus4,
                root_spacing,
                quality_slash_spacing,
                slash_spacing,
                format,
                output: cli.output,
                verbose: cli.verbose,
            })?;
        }
        Commands::Spell { symbol, format } => {
            let fmt = cmd::spell::SpellFormat::from_str_opt(format.as_deref())?;
            cmd::spell::run(cmd::spell::SpellArgs {
                symbol,
                format: fmt,
                verbose: cli.verbose,
            })?;
        }
        Commands::IntervalMatrix { input, title, full } => {
            cmd::interval_matrix::run(cmd::interval_matrix::IntervalMatrixArgs {
                input,
                output: cli.output,
                theme: cli.theme,
                title,
                verbose: cli.verbose,
                full,
            })?;
        }
        Commands::IntervalVector { input, title, full } => {
            cmd::interval_vector::run(cmd::interval_vector::IntervalVectorArgs {
                input,
                output: cli.output,
                theme: cli.theme,
                title,
                verbose: cli.verbose,
                full,
            })?;
        }
        Commands::IntervalLinear { input, title } => {
            cmd::intervals::run_linear(cmd::intervals::LinearIntervalArgs {
                input,
                output: cli.output,
                theme: cli.theme,
                title,
                verbose: cli.verbose,
            })?;
        }
        Commands::IntervalPairs {
            input,
            pairs,
            format,
        } => {
            cmd::intervals::run_pairs(cmd::intervals::IntervalPairsArgs {
                input,
                pairs,
                output: cli.output,
                format,
                verbose: cli.verbose,
            })?;
        }
        Commands::Superchords {
            input,
            min_size,
            max_size,
        } => {
            cmd::superchords::run(cmd::superchords::SuperchordsArgs {
                input,
                min_size,
                max_size,
                verbose: cli.verbose,
            })?;
        }
        Commands::Orbits { input, sym_type } => {
            let st = cmd::orbits::SymmetryType::from_str_opt(sym_type.as_deref())?;
            cmd::orbits::run(cmd::orbits::OrbitsArgs {
                input,
                sym_type: st,
                verbose: cli.verbose,
            })?;
        }
        Commands::PrimeForm { input } => {
            cmd::prime_form::run(cmd::prime_form::PrimeFormArgs {
                input,
                verbose: cli.verbose,
            })?;
        }
        Commands::Forte { input } => {
            cmd::forte::run(cmd::forte::ForteArgs {
                input,
                verbose: cli.verbose,
            })?;
        }
        Commands::CommonTones { sets } => {
            cmd::common_tones::run(cmd::common_tones::CommonTonesArgs {
                sets,
                verbose: cli.verbose,
            })?;
        }
        Commands::Contains {
            input,
            pool,
            direction,
            limit,
        } => {
            let pool = cmd::contains::Pool::from_str_opt(pool.as_deref())?;
            let direction = if let Some(d) = direction.as_deref() {
                Some(cmd::contains::Direction::from_str_opt(Some(d))?)
            } else {
                None
            };
            cmd::contains::run(cmd::contains::ContainsArgs {
                input,
                pool,
                direction,
                limit,
                verbose: cli.verbose,
            })?;
        }
        Commands::Closest {
            input,
            metric,
            pool,
            limit,
        } => {
            let metric = cmd::closest::Metric::from_str_opt(metric.as_deref())?;
            let pool = cmd::closest::Pool::from_str_opt(pool.as_deref())?;
            cmd::closest::run(cmd::closest::ClosestArgs {
                input,
                metric,
                pool,
                limit,
                verbose: cli.verbose,
            })?;
        }
        Commands::ChordDictionary {
            input,
            format,
            tuning,
            classifications,
            voicing,
            family,
            bass,
            open_strings,
            min_fret,
            max_fret,
            max_span,
            max_results,
        } => {
            cmd::chord_dictionary::run(cmd::chord_dictionary::ChordDictionaryArgs {
                input,
                output: cli.output,
                format,
                theme: cli.theme,
                tuning,
                classifications,
                voicing,
                family,
                bass,
                open_strings,
                min_fret,
                max_fret,
                max_span,
                max_results,
                verbose: cli.verbose,
            })?;
        }
        Commands::VoiceLeading {
            from,
            to,
            limit,
            no_crossings,
            metric,
            weights,
        } => {
            cmd::voice_leading::run(cmd::voice_leading::VoiceLeadingArgs {
                from,
                to,
                limit,
                no_crossings,
                metric,
                weights,
                verbose: cli.verbose,
            })?;
        }
        Commands::Sequence {
            harmony,
            chord_durations,
            pattern,
            master_step,
            rhythm,
            start,
            low,
            high,
            direction,
            turnaround,
            length,
            format,
            bpm,
            ppq,
            soundfont,
            offline,
            sample_rate,
            tail,
        } => {
            cmd::sequence::run(cmd::sequence::SequenceArgs {
                harmony,
                chord_durations,
                pattern,
                master_step,
                rhythm,
                start,
                low,
                high,
                direction,
                turnaround,
                length,
                format,
                output: cli.output,
                verbose: cli.verbose,
                bpm,
                ppq,
                soundfont,
                offline,
                sample_rate,
                tail,
            })?;
        }
        Commands::Render { input, format, dpi } => {
            cmd::render::run(cmd::render::RenderArgs {
                input,
                format,
                output: cli.output,
                dpi,
                verbose: cli.verbose,
            })?;
        }
        Commands::Progression {
            chords,
            no_crossings,
            metric,
            weights,
        } => {
            cmd::progression::run(cmd::progression::ProgressionArgs {
                chords,
                no_crossings,
                metric,
                weights,
                output: cli.output,
                verbose: cli.verbose,
            })?;
        }
        Commands::Voicings {
            input,
            range,
            min_spacing,
            max_spacing,
            strings,
            tuning,
            doubling,
            limit,
        } => {
            cmd::voicings::run(cmd::voicings::VoicingsArgs {
                input,
                limit,
                range,
                min_spacing,
                max_spacing,
                strings,
                tuning,
                doubling,
                verbose: cli.verbose,
            })?;
        }
        Commands::ArpeggioDictionary {
            input,
            tuning,
            keys,
            positions,
            max_span,
        } => {
            cmd::arpeggio_dictionary::run(cmd::arpeggio_dictionary::ArpeggioDictionaryArgs {
                input,
                output: cli.output,
                theme: cli.theme,
                tuning,
                keys,
                positions,
                max_span,
                verbose: cli.verbose,
            })?;
        }
        Commands::ScaleBook { scale, keys } => {
            cmd::scale_book::run(cmd::scale_book::ScaleBookArgs {
                scale,
                keys,
                output: cli.output,
                theme: cli.theme,
                verbose: cli.verbose,
            })?;
        }
        Commands::ScaleCatalog {
            scale,
            identify,
            modes,
            format,
        } => {
            cmd::scale_catalog::run(cmd::scale_catalog::ScaleCatalogArgs {
                scale,
                identify,
                modes,
                format,
                output: cli.output,
                verbose: cli.verbose,
            })?;
        }
        Commands::Contour {
            input,
            compare,
            transform,
            format,
        } => {
            cmd::contour::run(cmd::contour::ContourArgs {
                input,
                compare,
                transform,
                output: cli.output,
                format,
                verbose: cli.verbose,
            })?;
        }
        Commands::ScaleRotate {
            input,
            steps,
            format,
        } => {
            cmd::scale_rotate::run(cmd::scale_rotate::ScaleRotateArgs {
                input,
                steps,
                output: cli.output,
                format,
                verbose: cli.verbose,
            })?;
        }
        Commands::PracticeSheet { key, scale } => {
            cmd::practice_sheet::run(cmd::practice_sheet::PracticeSheetArgs {
                key,
                scale,
                output: cli.output,
                theme: cli.theme,
                verbose: cli.verbose,
            })?;
        }
        #[cfg(feature = "midi")]
        Commands::EarTraining {
            quiz_type,
            count,
            seed,
        } => {
            let qt = cmd::ear_training::QuizType::from_str_opt(quiz_type.as_deref())?;
            cmd::ear_training::run(cmd::ear_training::EarTrainingArgs {
                quiz_type: qt,
                count,
                output: cli.output,
                verbose: cli.verbose,
                seed,
            })?;
        }
        Commands::SightReading {
            key,
            scale,
            difficulty,
            measures,
            seed,
            clef,
        } => {
            cmd::sight_reading::run(cmd::sight_reading::SightReadingArgs {
                key,
                scale,
                difficulty,
                measures,
                seed,
                clef,
                output: cli.output,
                verbose: cli.verbose,
            })?;
        }
        Commands::RhythmDrill {
            time_sig,
            style,
            syncopation,
            measures,
            clef,
            seed,
        } => {
            cmd::rhythm_drill::run(cmd::rhythm_drill::RhythmDrillArgs {
                time_sig,
                style,
                syncopation,
                measures,
                clef,
                seed,
                output: cli.output,
                verbose: cli.verbose,
            })?;
        }
        Commands::Analyze {
            chords,
            key,
            scale,
            format,
        } => {
            let fmt = cmd::analyze::OutputFormat::from_str_opt(format.as_deref())?;
            cmd::analyze::run(cmd::analyze::AnalyzeArgs {
                chords,
                key,
                scale,
                format: fmt,
                output: cli.output,
                verbose: cli.verbose,
            })?;
        }
        Commands::Annotate {
            chords,
            key,
            scale,
            no_crossings,
            format,
        } => {
            let fmt = cmd::annotate::OutputFormat::from_str_opt(format.as_deref())?;
            cmd::annotate::run(cmd::annotate::AnnotateArgs {
                chords,
                key,
                scale,
                no_crossings,
                format: fmt,
                output: cli.output,
                verbose: cli.verbose,
            })?;
        }
        Commands::Subchords {
            input,
            size,
            name,
            relative,
        } => {
            cmd::subchords::run(cmd::subchords::SubchordsArgs {
                input,
                size,
                name,
                relative,
                verbose: cli.verbose,
            })?;
        }
    }

    Ok(())
}
