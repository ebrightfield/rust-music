//! Chord name parsing - converts string chord symbols into (Note, PcShape) pairs.
//!
//! This module provides the reverse operation of chord naming heuristics:
//! given a chord symbol string like "CMaj7" or "F#m7b5", it produces the root
//! note and corresponding pitch class shape (interval template rooted at Pc0).

use crate::error::MusicSemanticsError;
use crate::note::note::Note;
use crate::note::pitch_class::Pc;
use crate::note_collections::pc_set::PcShape;
use std::str::FromStr;

/// Parse a chord name into a root note and interval-template shape.
///
/// The returned `PcShape` contains the chord's interval template rooted at `Pc0`
/// (REQ-O22): e.g. `F#m7b5` → `[Pc0, Pc3, Pc6, Pc10]`, not the sounding pcs
/// `[Pc0, Pc4, Pc6, Pc9]`.
///
/// # Supported formats
/// - Major: C, CMaj, Cmajor
/// - Minor: Cm, Cmin, Cminor, C-
/// - Dominant 7th: C7
/// - Major 7th: CMaj7, Cmaj7, CM7, CΔ7
/// - Minor 7th: Cm7, Cmin7, C-7
/// - Diminished: Cdim, C°
/// - Diminished 7th: Cdim7, C°7
/// - Half-diminished: Cm7b5, Cø, Cø7
/// - Augmented: Caug, C+
/// - Suspended: Csus2, Csus4, Csus, C7sus4
///
/// # Examples
/// ```
/// use music::note_collections::chord_name::parsing::parse_chord_name;
/// use music::note::note::Note;
///
/// let (root, shape) = parse_chord_name("CMaj7").unwrap();
/// assert_eq!(root, Note::C);
/// assert_eq!(shape.len(), 4); // C, E, G, B intervals: [0,4,7,11]
/// ```
pub fn parse_chord_name(name: &str) -> Result<(Note, PcShape), MusicSemanticsError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(MusicSemanticsError::EmptySetOfNotes);
    }

    // Parse root note first
    let (root, quality_str) = parse_root(name)?;

    // Parse quality to get intervals from root (these are already root-relative, i.e. the
    // interval template). We do NOT add the root_pc offset — the intervals ARE the shape.
    // REQ-O22: PcShape = interval template rooted at Pc0.
    let intervals = parse_quality(quality_str)?;

    // Build PcShape directly from the intervals (which are already relative to root=0)
    let pcs: Vec<Pc> = intervals.iter().map(Pc::from).collect();

    Ok((root, PcShape::new(pcs)))
}

/// Parse the root note from the beginning of a chord name.
/// Returns the note and the remaining string (the quality portion).
fn parse_root(name: &str) -> Result<(Note, &str), MusicSemanticsError> {
    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() {
        return Err(MusicSemanticsError::EmptySetOfNotes);
    }

    // First char must be a letter A-G
    let letter = chars[0].to_ascii_uppercase();
    if !('A'..='G').contains(&letter) {
        return Err(MusicSemanticsError::InvalidNoteLetter(chars[0].to_string()));
    }

    // Check for accidentals
    let mut pos = 1;
    let mut root_str = letter.to_string();

    if pos < chars.len() {
        let next = chars[pos];
        // Handle various accidental notations
        match next {
            '#' | '♯' => {
                root_str.push('#');
                pos += 1;
                // Check for double sharp
                if pos < chars.len() && (chars[pos] == '#' || chars[pos] == '♯') {
                    root_str.push('#');
                    pos += 1;
                }
            }
            'b' | '♭' => {
                // Be careful: 'b' could be the note B, or a flat
                // If followed by another 'b', it's definitely a double flat
                // If followed by lowercase or quality indicators, it's a flat
                if pos + 1 < chars.len() {
                    let following = chars[pos + 1];
                    if following == 'b' || following == '♭' {
                        // Double flat
                        root_str.push('b');
                        root_str.push('b');
                        pos += 2;
                    } else if following.is_lowercase()
                        || following.is_numeric()
                        || following == '+'
                        || following == '-'
                        || following == '°'
                        || following == 'ø'
                    {
                        // Single flat followed by quality
                        root_str.push('b');
                        pos += 1;
                    }
                    // Otherwise, assume 'b' is part of quality (like "Bb" note but "Bbm" chord)
                } else {
                    // End of string after 'b', it's a flat
                    root_str.push('b');
                    pos += 1;
                }
            }
            '𝄪' => {
                // Unicode double sharp
                root_str.push_str("##");
                pos += 1;
            }
            '𝄫' => {
                // Unicode double flat
                root_str.push_str("bb");
                pos += 1;
            }
            _ => {}
        }
    }

    let note = Note::from_str(&root_str)?;
    let remaining = &name[pos..];

    Ok((note, remaining))
}

/// Parse a chord quality string into a set of intervals from the root.
/// The root (interval 0) is always included.
fn parse_quality(quality: &str) -> Result<Vec<u8>, MusicSemanticsError> {
    let q = quality.trim().to_lowercase();

    // Handle empty string = major triad
    if q.is_empty() {
        return Ok(vec![0, 4, 7]); // Major triad
    }

    // Try to match known patterns
    // Order matters - check longer patterns first

    // Major variants
    if q == "maj" || q == "major" || q == "M" {
        return Ok(vec![0, 4, 7]);
    }
    if q.starts_with("maj7") || q.starts_with("Δ7") || q == "M7" {
        let base = vec![0, 4, 7, 11];
        return parse_extensions(&q[4..], base);
    }
    if q.starts_with("maj9") {
        return Ok(vec![0, 4, 7, 11, 14 % 12]); // 14 % 12 = 2 (9th)
    }
    if q.starts_with("maj11") {
        return Ok(vec![0, 4, 7, 11, 2, 5]);
    }
    if q.starts_with("maj13") {
        return Ok(vec![0, 4, 7, 11, 2, 5, 9]);
    }

    // Dominant 7th variants
    if q == "7" {
        return Ok(vec![0, 4, 7, 10]);
    }
    if q == "9" {
        return Ok(vec![0, 4, 7, 10, 2]);
    }
    if q == "11" {
        return Ok(vec![0, 4, 7, 10, 2, 5]);
    }
    if q == "13" {
        return Ok(vec![0, 4, 7, 10, 2, 5, 9]);
    }

    // Minor variants
    if q == "m" || q == "min" || q == "minor" || q == "-" {
        return Ok(vec![0, 3, 7]);
    }
    if q == "m7" || q == "min7" || q == "-7" {
        return Ok(vec![0, 3, 7, 10]);
    }
    if q == "m9" || q == "min9" || q == "-9" {
        return Ok(vec![0, 3, 7, 10, 2]);
    }
    if q == "m11" || q == "min11" || q == "-11" {
        return Ok(vec![0, 3, 7, 10, 2, 5]);
    }
    if q == "m13" || q == "min13" || q == "-13" {
        return Ok(vec![0, 3, 7, 10, 2, 5, 9]);
    }

    // Minor-major 7th
    if q.starts_with("mmaj7") || q.starts_with("minmaj7") || q.starts_with("m(maj7)") {
        return Ok(vec![0, 3, 7, 11]);
    }

    // Diminished variants
    if q == "dim" || q == "°" || q == "o" {
        return Ok(vec![0, 3, 6]);
    }
    if q == "dim7" || q == "°7" || q == "o7" {
        return Ok(vec![0, 3, 6, 9]);
    }

    // Half-diminished
    if q == "m7b5" || q == "min7b5" || q == "ø" || q == "ø7" || q == "-7b5" {
        return Ok(vec![0, 3, 6, 10]);
    }

    // Augmented variants
    if q == "aug" || q == "+" {
        return Ok(vec![0, 4, 8]);
    }
    if q == "aug7" || q == "+7" {
        return Ok(vec![0, 4, 8, 10]);
    }
    if q == "augmaj7" || q == "+maj7" {
        return Ok(vec![0, 4, 8, 11]);
    }

    // Suspended chords
    if q == "sus" || q == "sus4" {
        return Ok(vec![0, 5, 7]);
    }
    if q == "sus2" {
        return Ok(vec![0, 2, 7]);
    }
    if q == "7sus" || q == "7sus4" {
        return Ok(vec![0, 5, 7, 10]);
    }
    if q == "9sus" || q == "9sus4" {
        return Ok(vec![0, 5, 7, 10, 2]);
    }

    // 6th chords
    if q == "6" {
        return Ok(vec![0, 4, 7, 9]);
    }
    if q == "m6" || q == "min6" || q == "-6" {
        return Ok(vec![0, 3, 7, 9]);
    }
    if q == "6/9" || q == "69" {
        return Ok(vec![0, 4, 7, 9, 2]);
    }

    // Add chords
    if q == "add9" || q == "add2" {
        return Ok(vec![0, 4, 7, 2]);
    }
    if q == "madd9" || q == "madd2" {
        return Ok(vec![0, 3, 7, 2]);
    }
    if q == "add11" || q == "add4" {
        return Ok(vec![0, 4, 7, 5]);
    }

    // Power chord
    if q == "5" {
        return Ok(vec![0, 7]);
    }

    // Handle alterations like 7b9, 7#11, etc.
    if q.starts_with("7") && q.len() > 1 {
        let mut pcs = vec![0, 4, 7, 10];
        parse_alterations(&q[1..], &mut pcs);
        return Ok(pcs);
    }

    // If we can't parse it, return an error
    Err(MusicSemanticsError::InvalidChordQuality(
        quality.to_string(),
    ))
}

/// Parse extensions that follow a base quality (e.g., "(#11)" after "Maj7")
fn parse_extensions(ext: &str, mut base: Vec<u8>) -> Result<Vec<u8>, MusicSemanticsError> {
    if ext.is_empty() {
        return Ok(base);
    }

    parse_alterations(ext, &mut base);
    Ok(base)
}

/// Parse alteration symbols and add/modify pitch classes
fn parse_alterations(alt_str: &str, pcs: &mut Vec<u8>) {
    let s = alt_str.to_lowercase();

    // Common alterations
    if s.contains("b9") || s.contains("♭9") {
        pcs.push(1); // flat 9 = 1 semitone
    }
    if s.contains("#9") || s.contains("♯9") {
        pcs.push(3); // sharp 9 = 3 semitones
    }
    if s.contains("9")
        && !s.contains("b9")
        && !s.contains("#9")
        && !s.contains("♭9")
        && !s.contains("♯9")
    {
        pcs.push(2); // natural 9
    }
    if s.contains("b11") || s.contains("♭11") {
        pcs.push(4); // flat 11 = 4 semitones
    }
    if s.contains("#11") || s.contains("♯11") {
        pcs.push(6); // sharp 11 = 6 semitones
    }
    if s.contains("11")
        && !s.contains("b11")
        && !s.contains("#11")
        && !s.contains("♭11")
        && !s.contains("♯11")
    {
        pcs.push(5); // natural 11
    }
    if s.contains("b13") || s.contains("♭13") {
        pcs.push(8); // flat 13 = 8 semitones
    }
    if s.contains("#13") || s.contains("♯13") {
        pcs.push(10); // sharp 13 = 10 semitones (same as b7, but contextually different)
    }
    if s.contains("13")
        && !s.contains("b13")
        && !s.contains("#13")
        && !s.contains("♭13")
        && !s.contains("♯13")
    {
        pcs.push(9); // natural 13
    }
    if s.contains("b5") || s.contains("♭5") {
        // Replace natural 5 with flat 5
        pcs.retain(|&x| x != 7);
        pcs.push(6);
    }
    if s.contains("#5") || s.contains("♯5") {
        // Replace natural 5 with sharp 5
        pcs.retain(|&x| x != 7);
        pcs.push(8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: since parse_chord_name now returns interval templates (PcShape rooted at Pc0),
    // all checks use interval-relative pitch classes (not absolute sounding pcs).

    #[test]
    fn test_parse_major_chords() {
        // C major: interval template [0, 4, 7]
        let (root, shape) = parse_chord_name("C").unwrap();
        assert_eq!(root, Note::C);
        assert!(shape.contains(&Pc::Pc0)); // root
        assert!(shape.contains(&Pc::Pc4)); // major 3rd
        assert!(shape.contains(&Pc::Pc7)); // perfect 5th

        // G major: same interval template (rooted at Pc0)
        let (root, shape) = parse_chord_name("G").unwrap();
        assert_eq!(root, Note::G);
        assert!(shape.contains(&Pc::Pc0));
        assert!(shape.contains(&Pc::Pc4));
        assert!(shape.contains(&Pc::Pc7));
    }

    #[test]
    fn test_parse_minor_chords() {
        // Am: interval template [0, 3, 7]
        let (root, shape) = parse_chord_name("Am").unwrap();
        assert_eq!(root, Note::A);
        assert!(shape.contains(&Pc::Pc0)); // root (interval 0)
        assert!(shape.contains(&Pc::Pc3)); // minor 3rd
        assert!(shape.contains(&Pc::Pc7)); // perfect 5th

        let (root, _) = parse_chord_name("Dm").unwrap();
        assert_eq!(root, Note::D);

        let (root, _) = parse_chord_name("Emin").unwrap();
        assert_eq!(root, Note::E);
    }

    #[test]
    fn test_parse_seventh_chords() {
        // Dominant 7th: interval template [0,4,7,10]
        let (root, shape) = parse_chord_name("G7").unwrap();
        assert_eq!(root, Note::G);
        assert_eq!(shape.len(), 4);

        // Major 7th: interval template [0,4,7,11]
        let (root, shape) = parse_chord_name("CMaj7").unwrap();
        assert_eq!(root, Note::C);
        assert!(shape.contains(&Pc::Pc11)); // major 7th interval

        // Minor 7th: interval template [0,3,7,10]
        let (root, shape) = parse_chord_name("Am7").unwrap();
        assert_eq!(root, Note::A);
        assert_eq!(shape.len(), 4);
    }

    #[test]
    fn test_parse_with_accidentals() {
        let (root, _) = parse_chord_name("F#m").unwrap();
        assert_eq!(root, Note::Fis);

        let (root, _) = parse_chord_name("Bb7").unwrap();
        assert_eq!(root, Note::Bes);

        let (root, _) = parse_chord_name("Ebmaj7").unwrap();
        assert_eq!(root, Note::Ees);
    }

    #[test]
    fn test_parse_diminished() {
        let (root, shape) = parse_chord_name("Bdim").unwrap();
        assert_eq!(root, Note::B);
        assert_eq!(shape.len(), 3);

        let (root, shape) = parse_chord_name("Cdim7").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(shape.len(), 4);
    }

    #[test]
    fn test_parse_augmented() {
        // Augmented: interval template [0, 4, 8]
        let (root, shape) = parse_chord_name("C+").unwrap();
        assert_eq!(root, Note::C);
        assert!(shape.contains(&Pc::Pc0)); // root
        assert!(shape.contains(&Pc::Pc4)); // major 3rd
        assert!(shape.contains(&Pc::Pc8)); // augmented 5th
    }

    #[test]
    fn test_parse_half_diminished() {
        // Half-diminished interval template: [0, 3, 6, 10]
        let (root, shape) = parse_chord_name("Cm7b5").unwrap();
        assert_eq!(root, Note::C);
        assert_eq!(shape.len(), 4, "shape = {:?}", shape);
        assert!(
            shape.contains(&Pc::Pc0),
            "Missing root, shape = {:?}",
            shape
        );
        assert!(shape.contains(&Pc::Pc3), "Missing m3, shape = {:?}", shape);
        assert!(shape.contains(&Pc::Pc6), "Missing b5, shape = {:?}", shape);
        assert!(shape.contains(&Pc::Pc10), "Missing m7, shape = {:?}", shape);
    }

    #[test]
    fn test_parse_suspended() {
        // Sus4 interval template: [0, 5, 7]
        let (root, shape) = parse_chord_name("Csus4").unwrap();
        assert_eq!(root, Note::C);
        assert!(
            shape.contains(&Pc::Pc0),
            "Missing root, shape = {:?}",
            shape
        );
        assert!(shape.contains(&Pc::Pc5), "Missing 4th, shape = {:?}", shape);
        assert!(shape.contains(&Pc::Pc7), "Missing 5th, shape = {:?}", shape);

        // Sus2 interval template: [0, 2, 7]
        let (root, shape) = parse_chord_name("Csus2").unwrap();
        assert_eq!(root, Note::C);
        assert!(
            shape.contains(&Pc::Pc0),
            "Missing root, shape = {:?}",
            shape
        );
        assert!(shape.contains(&Pc::Pc2), "Missing 2nd, shape = {:?}", shape);
        assert!(shape.contains(&Pc::Pc7), "Missing 5th, shape = {:?}", shape);
    }

    #[test]
    fn test_parse_power_chord() {
        let (root, shape) = parse_chord_name("E5").unwrap();
        assert_eq!(root, Note::E);
        assert_eq!(shape.len(), 2);
    }

    #[test]
    fn test_parse_sixth_chords() {
        // C6 interval template: [0, 4, 7, 9]
        let (root, shape) = parse_chord_name("C6").unwrap();
        assert_eq!(root, Note::C);
        assert!(shape.contains(&Pc::Pc9)); // 6th interval

        let (root, shape) = parse_chord_name("Am6").unwrap();
        assert_eq!(root, Note::A);
        assert_eq!(shape.len(), 4);
    }

    #[test]
    fn test_invalid_chord_names() {
        assert!(parse_chord_name("").is_err());
        assert!(parse_chord_name("H").is_err()); // Invalid note letter
        assert!(parse_chord_name("Cxyz").is_err()); // Invalid quality
    }
}
