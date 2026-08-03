use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd;

/// Note shown in `--help` so the feature-gated subcommand isn't mistaken for
/// missing when it is simply absent from a default build.
#[cfg(not(feature = "midi"))]
const FEATURE_HELP: &str = "Additional subcommands:\n  \
    ear-training  Generate MIDI ear-training quizzes — requires the `midi` \
    feature: cargo build -p slonimsky --features midi";

#[cfg(feature = "midi")]
const FEATURE_HELP: &str = "Built with the `midi` feature: `ear-training` is available.";

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

        /// Tuning: standard, drop-d, dadgad, open-g, 7-string, bass-4, bass-5
        #[arg(long, default_value = "standard")]
        tuning: String,

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
    /// Name a chord from pitch classes
    Name {
        /// Pitch classes (integer or note name)
        #[arg(required = true)]
        pcs: Vec<String>,

        /// Root pitch class (default: first in input)
        #[arg(long)]
        root: Option<String>,
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
    /// Enumerate chord shapes for a given chord across the fretboard
    ChordDictionary {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Tuning: standard, drop-d, dadgad, open-g, 7-string, bass-4, bass-5
        #[arg(long, default_value = "standard")]
        tuning: String,

        /// Maximum fret span for playable shapes (default: 4)
        #[arg(long, default_value = "4")]
        max_span: u8,

        /// Maximum number of results to show (default: 20)
        #[arg(long, default_value = "20")]
        max_results: usize,
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

        /// Distance metric: l1 (sum of absolute semitone motions, default) or linf (max single-voice motion)
        #[arg(long, default_value = "l1")]
        metric: String,
    },
    /// Plan smoothest voice-leadings through a chord sequence
    Progression {
        /// Chords as comma-separated groups (e.g. C,E,G F,A,C G,B,D)
        #[arg(required = true)]
        chords: Vec<String>,

        /// Forbid voice crossings
        #[arg(long)]
        no_crossings: bool,

        /// Distance metric for greedy step selection: l1 (default) or linf
        #[arg(long, default_value = "l1")]
        metric: String,
    },
    /// Enumerate canonical voicings of a 3- or 4-note chord
    Voicings {
        /// Pitch classes, note names, or comma-separated list
        #[arg(required = true)]
        input: Vec<String>,

        /// Maximum number of voicings to show
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Generate per-key arpeggio sheets across fretboard positions
    ArpeggioDictionary {
        /// Pitch classes, note names, or comma-separated list (chord type)
        #[arg(required = true)]
        input: Vec<String>,

        /// Tuning: standard, drop-d, dadgad, open-g, 7-string, bass-4, bass-5
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
        /// Parent scale: major, melodic-minor, harmonic-minor, harmonic-major
        scale: String,

        /// Keys to include: "all" (default) or comma-separated (e.g. C,G,D)
        #[arg(long)]
        keys: Option<String>,
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
            orientation,
            title,
            num_frets,
        } => {
            cmd::fretboard::run(cmd::fretboard::FretboardArgs {
                frets,
                output: cli.output,
                theme: cli.theme,
                tuning,
                orientation,
                title,
                num_frets,
                verbose: cli.verbose,
            })?;
        }
        Commands::Name { pcs, root } => {
            cmd::name::run(cmd::name::NameArgs {
                pcs,
                root,
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
        Commands::IntervalMatrix {
            input,
            title,
            full,
        } => {
            cmd::interval_matrix::run(cmd::interval_matrix::IntervalMatrixArgs {
                input,
                output: cli.output,
                theme: cli.theme,
                title,
                verbose: cli.verbose,
                full,
            })?;
        }
        Commands::IntervalVector {
            input,
            title,
            full,
        } => {
            cmd::interval_vector::run(cmd::interval_vector::IntervalVectorArgs {
                input,
                output: cli.output,
                theme: cli.theme,
                title,
                verbose: cli.verbose,
                full,
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
            tuning,
            max_span,
            max_results,
        } => {
            cmd::chord_dictionary::run(cmd::chord_dictionary::ChordDictionaryArgs {
                input,
                output: cli.output,
                theme: cli.theme,
                tuning,
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
        } => {
            cmd::voice_leading::run(cmd::voice_leading::VoiceLeadingArgs {
                from,
                to,
                limit,
                no_crossings,
                metric,
                verbose: cli.verbose,
            })?;
        }
        Commands::Progression {
            chords,
            no_crossings,
            metric,
        } => {
            cmd::progression::run(cmd::progression::ProgressionArgs {
                chords,
                no_crossings,
                metric,
                output: cli.output,
                verbose: cli.verbose,
            })?;
        }
        Commands::Voicings { input, limit } => {
            cmd::voicings::run(cmd::voicings::VoicingsArgs {
                input,
                limit,
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
        Commands::Subchords { input, size, name, relative } => {
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
