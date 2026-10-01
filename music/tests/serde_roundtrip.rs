#![cfg(feature = "serde")]

use music::fretboard::StringConvention;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::DurationKind;
use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::note::spelling::{Accidental, Spelling};

#[test]
fn note_roundtrips_each_spelling() {
    for n in [Note::C, Note::Cis, Note::Des, Note::Cisis, Note::Aeses] {
        let s = serde_json::to_string(&n).unwrap();
        let back: Note = serde_json::from_str(&s).unwrap();
        assert_eq!(n, back, "enharmonic spelling must round-trip");
    }
}

#[test]
fn pc_roundtrips() {
    for pc in [Pc::Pc0, Pc::Pc1, Pc::Pc6, Pc::Pc11] {
        let s = serde_json::to_string(&pc).unwrap();
        let back: Pc = serde_json::from_str(&s).unwrap();
        assert_eq!(pc, back);
    }
}

#[test]
fn spelling_roundtrips() {
    let sp = Spelling {
        letter: music::note::spelling::Letter::C,
        acc: Accidental::Sharp,
    };
    let s = serde_json::to_string(&sp).unwrap();
    let back: Spelling = serde_json::from_str(&s).unwrap();
    assert_eq!(sp, back);
}

#[test]
fn duration_kind_roundtrips() {
    for dk in [
        DurationKind::Whole,
        DurationKind::Qtr,
        DurationKind::OneTwentyEighth,
    ] {
        let s = serde_json::to_string(&dk).unwrap();
        let back: DurationKind = serde_json::from_str(&s).unwrap();
        assert_eq!(dk, back);
    }
}

#[test]
fn clef_roundtrips() {
    for c in [Clef::Treble, Clef::Bass, Clef::Treble8va, Clef::Treble8ba] {
        let s = serde_json::to_string(&c).unwrap();
        let back: Clef = serde_json::from_str(&s).unwrap();
        assert_eq!(c, back);
    }
}

#[test]
fn string_convention_roundtrips() {
    for sc in [
        StringConvention::OneIndexedFromHigh,
        StringConvention::OneIndexedFromLow,
        StringConvention::ZeroIndexedFromLow,
    ] {
        let s = serde_json::to_string(&sc).unwrap();
        let back: StringConvention = serde_json::from_str(&s).unwrap();
        assert_eq!(sc, back);
    }
}
