use anyhow::{Context, Result};
use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::note::spelling::{Accidental, Letter, Spelling};
use music::note_collections::chord_name::{
    ChordName, ChordNameDisplayConfig, MajNotation, TonalSpecification,
};

pub struct SpellArgs {
    pub symbol: String,
    pub format: SpellFormat,
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SpellFormat {
    /// Show spelled note names (default)
    #[default]
    Notes,
    /// Show pitch-class integers
    Pcs,
    /// Show intervals from root in semitones
    Intervals,
    /// Show all three
    All,
}

impl SpellFormat {
    pub fn from_str_opt(s: Option<&str>) -> Result<Self> {
        match s {
            None => Ok(SpellFormat::default()),
            Some("notes") => Ok(SpellFormat::Notes),
            Some("pcs") => Ok(SpellFormat::Pcs),
            Some("intervals") => Ok(SpellFormat::Intervals),
            Some("all") => Ok(SpellFormat::All),
            Some(other) => anyhow::bail!(
                "unknown format '{}'; expected notes, pcs, intervals, or all",
                other
            ),
        }
    }
}

/// Spell the notes of a chord from a root note, given the PcSet.
///
/// `pcs` is the chord's root-relative `PcShape` (zeroed so the root is `Pc0`),
/// so each `Pc`'s integer value is already the interval from the root.
/// Returns notes in root-based order (root first, ascending).
fn spell_from_root(root: Note, pcs: &[Pc]) -> Vec<Note> {
    let root_val = u8::from(&Pc::from(&root));

    // Build (interval_from_root, absolute_pc) pairs and sort by interval.
    // pcs are root-relative, so the pc value IS the interval; shift to absolute
    // before looking up enharmonic candidates.
    let mut entries: Vec<(u8, Pc)> = pcs
        .iter()
        .map(|&zeroed_pc| {
            let interval = u8::from(&zeroed_pc);
            let absolute_pc = Pc::from((interval + root_val) % 12);
            (interval, absolute_pc)
        })
        .collect();
    entries.sort_by_key(|(interval, _)| *interval);

    // Spell each chord tone as the interval it is above the root. The full
    // interval list is passed along so ambiguous intervals (the tritone, the
    // diminished seventh) can be resolved against the rest of the chord.
    let intervals: Vec<u8> = entries.iter().map(|(interval, _)| *interval).collect();
    entries
        .iter()
        .map(|(interval, pc)| pick_spelling(root, *pc, *interval, &intervals))
        .collect()
}

/// The diatonic letter-step for each chord interval, measured in letter names
/// above the root: a third is 2 letters up, a fifth 4, a seventh 6, etc.
///
/// Chromatic intervals map to the tertian degree they alter, so a b3 and a 3
/// share the third's letter and differ only in accidental. Two intervals are
/// ambiguous and are resolved from the rest of the chord (`others`):
///
/// - **6 semitones** is a b5 in diminished/altered chords but a #11 in lydian
///   ones. If the chord also has a perfect fifth, the tritone must be the
///   #11 (a fourth) — otherwise `Cmaj7#11` would spell `Gb G`.
/// - **9 semitones** is a 13th (sixth) normally, but the diminished seventh of
///   a fully-diminished chord. If the chord has a diminished fifth and no
///   seventh above it, spell it as a seventh so `Cdim7` gives `C Eb Gb Bbb`.
fn letter_steps_for_interval(semitones: u8, others: &[u8]) -> u8 {
    let has = |i: u8| others.contains(&i);
    match semitones {
        0 => 0,     // root
        1 | 2 => 1, // b9 / 9  → second
        3 | 4 => 2, // b3 / 3  → third
        5 => 3,     // 11      → fourth
        // Tritone: #11 (fourth) when a perfect fifth is present, else b5.
        6 => {
            if has(7) {
                3
            } else {
                4
            }
        }
        7 | 8 => 4, // 5 / #5  → fifth
        // Diminished seventh when sitting on a diminished fifth with no
        // other seventh; otherwise a 13th.
        9 => {
            if has(6) && !has(10) && !has(11) {
                6
            } else {
                5
            }
        }
        10 | 11 => 6, // b7 / 7  → seventh
        _ => 0,
    }
}

/// Semitones above C for a natural letter.
fn letter_semitones(letter: Letter) -> i32 {
    match letter {
        Letter::C => 0,
        Letter::D => 2,
        Letter::E => 4,
        Letter::F => 5,
        Letter::G => 7,
        Letter::A => 9,
        Letter::B => 11,
    }
}

/// Spell the note `interval` semitones above `root` as a proper tertian
/// interval: advance the root's letter by the interval's diatonic letter-step,
/// then pick the accidental (up to a double) that lands the letter on the
/// required pitch class.
///
/// This is what makes `Cm7`'s third come out as `Eb` rather than `D#`, and
/// `Gbm7`'s as `Bbb` rather than `A` — the letter is fixed by the interval, so
/// only the accidental is free.
///
/// Returns `None` when the required accidental would exceed a double
/// accidental, or when the letter/accidental pair has no `Note` (e.g. `Fbb`).
fn spell_interval(root: Note, interval: u8, others: &[u8]) -> Option<Note> {
    let root_spelling = Spelling::from(&root);

    // Walk the root's letter up by the interval's diatonic step.
    let steps = letter_steps_for_interval(interval, others);
    let mut letter = root_spelling.letter;
    for _ in 0..steps {
        letter = letter.next();
    }

    // Absolute semitone target, and what the bare letter gives us. Both are
    // reduced mod 12 so the comparison is octave-agnostic.
    let root_semis = letter_semitones(root_spelling.letter) + accidental_offset(root_spelling.acc);
    let target = (root_semis + i32::from(interval)).rem_euclid(12);
    let natural = letter_semitones(letter).rem_euclid(12);

    // Choose the accidental that closes the gap, taking the representative in
    // −6..=5 so we never pick a 10-semitone "correction" over a 2-semitone one.
    let mut delta = (target - natural).rem_euclid(12);
    if delta > 6 {
        delta -= 12;
    }

    let acc = match delta {
        0 => Accidental::Natural,
        1 => Accidental::Sharp,
        2 => Accidental::DoubleSharp,
        -1 => Accidental::Flat,
        -2 => Accidental::DoubleFlat,
        _ => return None,
    };

    Note::try_from(Spelling::new(letter, acc)).ok()
}

/// Choose the Note spelling for a chord tone.
///
/// Prefers the tertian spelling from letter arithmetic ([`spell_interval`]).
/// Falls back to an enharmonic of the correct pitch class only when the
/// tertian letter would need more than a double accidental (or names no
/// representable note), so a spelling is always produced.
fn pick_spelling(root: Note, pc: Pc, interval: u8, others: &[u8]) -> Note {
    if let Some(note) = spell_interval(root, interval, others) {
        return note;
    }

    // Fallback for spellings no `Note` can represent — e.g. the seventh of
    // `Gbdim7` is Fbb, a triple flat. Take an enharmonic of the right pitch
    // class, preferring a simple accidental and matching the root's flat/sharp
    // side so the result still reads in the same direction as the chord.
    let candidates = pc.notes();
    let root_is_flat = Spelling::from(&root).acc == Accidental::Flat
        || Spelling::from(&root).acc == Accidental::DoubleFlat;

    let simple: Vec<Note> = candidates
        .iter()
        .copied()
        .filter(|n| !Spelling::from(n).acc.is_double())
        .collect();
    let pool = if simple.is_empty() {
        &candidates
    } else {
        &simple
    };

    pool.iter()
        .find(|n| {
            let acc = Spelling::from(*n).acc;
            if root_is_flat {
                acc == Accidental::Flat
            } else {
                acc == Accidental::Sharp
            }
        })
        .or_else(|| {
            pool.iter()
                .find(|n| Spelling::from(*n).acc == Accidental::Natural)
        })
        .copied()
        .unwrap_or(pool[0])
}

/// Semitone offset contributed by an accidental.
fn accidental_offset(acc: Accidental) -> i32 {
    match acc {
        Accidental::Natural => 0,
        Accidental::Sharp => 1,
        Accidental::DoubleSharp => 2,
        Accidental::Flat => -1,
        Accidental::DoubleFlat => -2,
    }
}

/// Format interval in semitones as a conventional name.
fn interval_name(semitones: u8) -> &'static str {
    match semitones {
        0 => "R",
        1 => "b2",
        2 => "2",
        3 => "b3",
        4 => "3",
        5 => "4",
        6 => "b5",
        7 => "5",
        8 => "#5",
        9 => "6",
        10 => "b7",
        11 => "7",
        _ => "?",
    }
}

pub fn run(args: SpellArgs) -> Result<()> {
    let chord = ChordName::from_symbol(&args.symbol)
        .with_context(|| format!("failed to parse chord symbol '{}'", args.symbol))?;

    let root = match &chord.tonality {
        TonalSpecification::RootPosition(r) => *r,
        TonalSpecification::SlashChord { root, .. } => *root,
        TonalSpecification::None(_) => {
            anyhow::bail!("chord '{}' has no identifiable root", args.symbol);
        }
    };

    let pcs: Vec<Pc> = chord.pc_shape.iter().copied().collect();
    let notes = spell_from_root(root, &pcs);

    // pc_shape is root-relative, so each Pc value IS the interval from the root.
    let mut intervals: Vec<u8> = pcs.iter().map(|pc| u8::from(pc)).collect();
    intervals.sort();

    // Display config for the chord quality label in verbose mode
    let display_cfg = ChordNameDisplayConfig {
        maj_notation: MajNotation::Maj,
        utf8_accidentals: true,
        ..Default::default()
    };

    if args.verbose {
        let quality_str = chord.quality.to_string(&display_cfg);
        eprintln!("spell: {} → {}{}", args.symbol, root, quality_str);
    }

    match args.format {
        SpellFormat::Notes => {
            let note_strs: Vec<String> = notes.iter().map(|n| n.to_string()).collect();
            println!("{}", note_strs.join(" "));
        }
        SpellFormat::Pcs => {
            let pc_strs: Vec<String> = intervals.iter().map(|i| i.to_string()).collect();
            println!("{}", pc_strs.join(" "));
        }
        SpellFormat::Intervals => {
            let int_strs: Vec<String> = intervals
                .iter()
                .map(|i| interval_name(*i).to_string())
                .collect();
            println!("{}", int_strs.join(" "));
        }
        SpellFormat::All => {
            let note_strs: Vec<String> = notes.iter().map(|n| n.to_string()).collect();
            println!("Notes:     {}", note_strs.join(" "));

            let pc_strs: Vec<String> = intervals.iter().map(|i| i.to_string()).collect();
            println!("PCs:       {}", pc_strs.join(" "));

            let int_strs: Vec<String> = intervals
                .iter()
                .map(|i| interval_name(*i).to_string())
                .collect();
            println!("Intervals: {}", int_strs.join(" "));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_c_major_triad() {
        let args = SpellArgs {
            symbol: "C".into(),
            format: SpellFormat::Notes,
            verbose: false,
        };
        run(args).unwrap();
    }

    #[test]
    fn spell_cmaj7_notes() {
        let chord = ChordName::from_symbol("Cmaj7").unwrap();
        let root = match &chord.tonality {
            TonalSpecification::RootPosition(r) => *r,
            _ => panic!("expected root position"),
        };
        let pcs: Vec<Pc> = chord.pc_shape.iter().copied().collect();
        let notes = spell_from_root(root, &pcs);
        // Cmaj7 = C E G B — four notes, root is C
        assert_eq!(notes.len(), 4);
        assert_eq!(notes[0], Note::C);
    }

    #[test]
    fn spell_dm7_notes() {
        let chord = ChordName::from_symbol("Dm7").unwrap();
        let root = match &chord.tonality {
            TonalSpecification::RootPosition(r) => *r,
            _ => panic!("expected root position"),
        };
        assert_eq!(root, Note::D);
        let pcs: Vec<Pc> = chord.pc_shape.iter().copied().collect();
        let notes = spell_from_root(root, &pcs);
        // Dm7 = D F A C — four notes
        assert_eq!(notes.len(), 4);
        assert_eq!(notes, vec![Note::D, Note::F, Note::A, Note::C]);
    }

    #[test]
    fn spell_ebmaj7_notes_prefers_flats() {
        // Eb is in the flat-preferring list; verifies pick_spelling + the
        // zeroed-pc → absolute-pc shift land on Eb G Bb D rather than the
        // sharp enharmonics.
        let chord = ChordName::from_symbol("Ebmaj7").unwrap();
        let root = match &chord.tonality {
            TonalSpecification::RootPosition(r) => *r,
            _ => panic!("expected root position"),
        };
        assert_eq!(root, Note::Ees);
        let pcs: Vec<Pc> = chord.pc_shape.iter().copied().collect();
        let notes = spell_from_root(root, &pcs);
        assert_eq!(notes, vec![Note::Ees, Note::G, Note::Bes, Note::D]);
    }

    /// Spell a symbol end-to-end, as the CLI does.
    fn notes_of(symbol: &str) -> Vec<Note> {
        let chord = ChordName::from_symbol(symbol).unwrap();
        let root = match &chord.tonality {
            TonalSpecification::RootPosition(r) => *r,
            TonalSpecification::SlashChord { root, .. } => *root,
            _ => panic!("expected a root for {symbol}"),
        };
        let pcs: Vec<Pc> = chord.pc_shape.iter().copied().collect();
        spell_from_root(root, &pcs)
    }

    /// Chord tones must be spelled as the interval they are, so a minor third
    /// is always a third by letter — even when that needs a double flat.
    /// Regression for the `Cm7 → C D# G A#` family of bugs.
    #[test]
    fn minor_sevenths_spell_thirds_as_thirds() {
        use Note::*;
        assert_eq!(notes_of("Cm7"), vec![C, Ees, G, Bes]);
        assert_eq!(notes_of("Abm7"), vec![Aes, Ces, Ees, Ges]);
        assert_eq!(notes_of("Dbm7"), vec![Des, Fes, Aes, Ces]);
        // The third here is Bbb — a double flat is correct and preferred over A.
        assert_eq!(notes_of("Gbm7"), vec![Ges, Beses, Des, Fes]);
    }

    /// Roots that were already correct must stay correct.
    #[test]
    fn previously_correct_spellings_unchanged() {
        use Note::*;
        assert_eq!(notes_of("Bbmaj7"), vec![Bes, D, F, A]);
        assert_eq!(notes_of("Ebmaj7"), vec![Ees, G, Bes, D]);
        assert_eq!(notes_of("F7"), vec![F, A, C, Ees]);
        assert_eq!(notes_of("Ebm7"), vec![Ees, Ges, Bes, Des]);
        assert_eq!(notes_of("Fm7"), vec![F, Aes, C, Ees]);
    }

    /// Every letter of a seventh chord is distinct: root, third, fifth,
    /// seventh — never a repeated or skipped letter name.
    #[test]
    fn seventh_chords_use_four_distinct_letters() {
        for root in [
            "C", "C#", "Db", "D", "Eb", "E", "F", "F#", "Gb", "G", "Ab", "A", "Bb", "B",
        ] {
            for quality in ["m7", "maj7", "7", "m7b5"] {
                let symbol = format!("{root}{quality}");
                let Ok(chord) = ChordName::from_symbol(&symbol) else {
                    continue;
                };
                let TonalSpecification::RootPosition(r) = &chord.tonality else {
                    continue;
                };
                let pcs: Vec<Pc> = chord.pc_shape.iter().copied().collect();
                let notes = spell_from_root(*r, &pcs);
                let letters: std::collections::HashSet<Letter> =
                    notes.iter().map(|n| Spelling::from(n).letter).collect();
                assert_eq!(
                    letters.len(),
                    notes.len(),
                    "{symbol} reuses a letter name: {notes:?}"
                );
            }
        }
    }

    /// The tritone is a b5 in a diminished chord but a #11 when the chord also
    /// has a perfect fifth; the dim7's top note is a seventh, not a sixth.
    #[test]
    fn ambiguous_intervals_resolve_from_context() {
        use Note::*;
        assert_eq!(notes_of("Cdim7"), vec![C, Ees, Ges, Beses]);
        assert_eq!(notes_of("Cm7b5"), vec![C, Ees, Ges, Bes]);
        // Perfect fifth present → the tritone is F#, giving no absurd "Gb G".
        assert_eq!(notes_of("Cmaj7#11"), vec![C, E, Fis, G, B]);
    }

    /// Some tertian spellings need a triple accidental (`Gbdim7`'s seventh is
    /// Fbb) and no `Note` can hold one. Those fall back to an enharmonic on the
    /// root's accidental side rather than emitting a jarring sharp in a flat
    /// chord — and never panic or drop the note.
    #[test]
    fn unrepresentable_spellings_fall_back_on_the_root_side() {
        for symbol in ["Gbdim7", "Cbdim7", "Dbdim7"] {
            let notes = notes_of(symbol);
            assert_eq!(notes.len(), 4, "{symbol} should keep all four tones");
            assert!(
                notes.iter().all(|n| !matches!(
                    Spelling::from(n).acc,
                    Accidental::Sharp | Accidental::DoubleSharp
                )),
                "{symbol} is a flat-side chord but spelled with sharps: {notes:?}"
            );
        }
    }

    /// Spelling is driven by the root's letter, so enharmonic roots that sound
    /// alike still spell differently.
    #[test]
    fn enharmonic_roots_spell_differently() {
        use Note::*;
        assert_eq!(notes_of("C#m7"), vec![Cis, E, Gis, B]);
        assert_eq!(notes_of("Dbm7"), vec![Des, Fes, Aes, Ces]);
    }

    #[test]
    fn spell_format_parse() {
        assert!(matches!(
            SpellFormat::from_str_opt(None).unwrap(),
            SpellFormat::Notes
        ));
        assert!(matches!(
            SpellFormat::from_str_opt(Some("pcs")).unwrap(),
            SpellFormat::Pcs
        ));
        assert!(matches!(
            SpellFormat::from_str_opt(Some("intervals")).unwrap(),
            SpellFormat::Intervals
        ));
        assert!(matches!(
            SpellFormat::from_str_opt(Some("all")).unwrap(),
            SpellFormat::All
        ));
        assert!(SpellFormat::from_str_opt(Some("bad")).is_err());
    }

    #[test]
    fn interval_names_correct() {
        assert_eq!(interval_name(0), "R");
        assert_eq!(interval_name(3), "b3");
        assert_eq!(interval_name(4), "3");
        assert_eq!(interval_name(7), "5");
        assert_eq!(interval_name(10), "b7");
        assert_eq!(interval_name(11), "7");
    }

    #[test]
    fn spell_rejects_garbage() {
        let args = SpellArgs {
            symbol: "XYZ".into(),
            format: SpellFormat::Notes,
            verbose: false,
        };
        assert!(run(args).is_err());
    }
}
