use anyhow::{Context, Result};
use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::note_collections::chord_name::{ChordName, ChordNameDisplayConfig, MajNotation, TonalSpecification};

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

    // Pick the best spelling for each pc given the root context
    entries
        .iter()
        .map(|(_, pc)| pick_spelling(root, *pc))
        .collect()
}

/// Choose the most contextually appropriate Note spelling for a Pc,
/// preferring spellings consistent with the root's accidental tendency.
///
/// Priority:
///   1. A natural (no-accidental) spelling, if one exists.
///   2. A single-accidental spelling matching the root's flat/sharp tendency.
///   3. Any single-accidental spelling.
///   4. The first candidate (fallback; avoids double accidentals).
fn pick_spelling(root: Note, pc: Pc) -> Note {
    let candidates = pc.notes();
    if candidates.len() == 1 {
        return candidates[0];
    }

    if let Some(n) = candidates.iter().find(|n| is_natural(n)) {
        return *n;
    }

    let root_prefers_flats = matches!(
        root,
        Note::F | Note::Bes | Note::Ees | Note::Aes | Note::Des | Note::Ges | Note::Ces
    );

    let preferred = if root_prefers_flats {
        candidates.iter().find(|n| is_single_flat(n))
    } else {
        candidates.iter().find(|n| is_single_sharp(n))
    };

    preferred
        .or_else(|| candidates.iter().find(|n| is_single_flat(n) || is_single_sharp(n)))
        .copied()
        .unwrap_or(candidates[0])
}

fn is_natural(note: &Note) -> bool {
    matches!(
        note,
        Note::C | Note::D | Note::E | Note::F | Note::G | Note::A | Note::B
    )
}

fn is_single_flat(note: &Note) -> bool {
    matches!(
        note,
        Note::Des | Note::Ees | Note::Fes | Note::Ges | Note::Aes | Note::Bes | Note::Ces
    )
}

fn is_single_sharp(note: &Note) -> bool {
    matches!(
        note,
        Note::Cis | Note::Dis | Note::Eis | Note::Fis | Note::Gis | Note::Ais | Note::Bis
    )
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
            let int_strs: Vec<String> = intervals.iter().map(|i| interval_name(*i).to_string()).collect();
            println!("{}", int_strs.join(" "));
        }
        SpellFormat::All => {
            let note_strs: Vec<String> = notes.iter().map(|n| n.to_string()).collect();
            println!("Notes:     {}", note_strs.join(" "));

            let pc_strs: Vec<String> = intervals.iter().map(|i| i.to_string()).collect();
            println!("PCs:       {}", pc_strs.join(" "));

            let int_strs: Vec<String> = intervals.iter().map(|i| interval_name(*i).to_string()).collect();
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

    #[test]
    fn spell_format_parse() {
        assert!(matches!(SpellFormat::from_str_opt(None).unwrap(), SpellFormat::Notes));
        assert!(matches!(SpellFormat::from_str_opt(Some("pcs")).unwrap(), SpellFormat::Pcs));
        assert!(matches!(SpellFormat::from_str_opt(Some("intervals")).unwrap(), SpellFormat::Intervals));
        assert!(matches!(SpellFormat::from_str_opt(Some("all")).unwrap(), SpellFormat::All));
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
