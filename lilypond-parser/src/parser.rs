use crate::ast::{Event, Item};
use crate::error::{ParseError, ParseErrorKind};
use crate::lexer::tokenize;
use crate::token::{Span, Token, TokenKind};
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::{Note, Pitch};

/// Parse a LilyPond source string into a flat list of items.
///
/// The input may be either a bare sequence of events (no outer braces) or a
/// single outer `{ ... }` block. Nested blocks are returned as
/// [`Item::Block`]. See the crate docs for the supported subset.
pub fn parse(src: &str) -> Result<Vec<Item>, ParseError> {
    let tokens = tokenize(src)?;
    let mut p = Parser::new(tokens, src.len());
    p.parse_items(/*inside_braces=*/ false)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    eof: usize,
    last_duration: Option<Duration>,
}

impl Parser {
    fn new(tokens: Vec<Token>, eof: usize) -> Self {
        Self { tokens, pos: 0, eof, last_duration: None }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn eof_span(&self) -> Span {
        self.eof..self.eof
    }

    fn cur_span(&self) -> Span {
        self.peek().map(|t| t.span.clone()).unwrap_or_else(|| self.eof_span())
    }

    fn parse_items(&mut self, inside_braces: bool) -> Result<Vec<Item>, ParseError> {
        let mut items = Vec::new();
        loop {
            match self.peek_kind() {
                None => {
                    if inside_braces {
                        return Err(ParseError::new(
                            ParseErrorKind::UnexpectedEof("'}'"),
                            self.eof_span(),
                        ));
                    }
                    break;
                }
                Some(TokenKind::RBrace) => {
                    if inside_braces {
                        self.bump();
                        break;
                    }
                    return Err(ParseError::new(
                        ParseErrorKind::Expected {
                            expected: "top-level item",
                            found: "'}'".into(),
                        },
                        self.cur_span(),
                    ));
                }
                _ => {
                    let item = self.parse_item()?;
                    items.push(item);
                }
            }
        }
        Ok(items)
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        let tok = self.peek().cloned().ok_or_else(|| {
            ParseError::new(ParseErrorKind::UnexpectedEof("item"), self.eof_span())
        })?;
        match &tok.kind {
            TokenKind::LBrace => {
                self.bump();
                let items = self.parse_items(true)?;
                Ok(Item::Block(items))
            }
            TokenKind::Command(name) => {
                let name = name.clone();
                let span = tok.span.clone();
                self.bump();
                self.parse_command(&name, span)
            }
            TokenKind::LAngle => {
                let ev = self.parse_chord_event()?;
                Ok(Item::Event(ev))
            }
            TokenKind::Ident(_) => {
                let ev = self.parse_note_or_rest_event()?;
                Ok(Item::Event(ev))
            }
            other => Err(ParseError::new(
                ParseErrorKind::Expected {
                    expected: "pitch, chord, '{', or command",
                    found: format!("{:?}", other),
                },
                tok.span,
            )),
        }
    }

    fn parse_command(&mut self, name: &str, span: Span) -> Result<Item, ParseError> {
        match name {
            "clef" => {
                let ident = self.expect_ident("clef name")?;
                Ok(Item::Clef(ident))
            }
            "time" => {
                let num = self.expect_number("time signature numerator")?;
                self.expect_kind(&TokenKind::Slash, "'/'")?;
                let den = self.expect_number("time signature denominator")?;
                Ok(Item::Time(num, den))
            }
            "key" => {
                let tonic = self.expect_ident("key tonic")?;
                let mode_cmd = self.expect_command("mode command (e.g. \\major)")?;
                Ok(Item::Key(tonic, mode_cmd))
            }
            "tuplet" => {
                let num = self.expect_number("tuplet numerator")?;
                self.expect_kind(&TokenKind::Slash, "'/'")?;
                let den = self.expect_number("tuplet denominator")?;
                self.expect_kind(&TokenKind::LBrace, "'{'")?;
                let items = self.parse_items(true)?;
                Ok(Item::Tuplet { numerator: num, denominator: den, items })
            }
            other => Err(ParseError::new(
                ParseErrorKind::UnsupportedCommand(format!("\\{}", other)),
                span,
            )),
        }
    }

    fn parse_note_or_rest_event(&mut self) -> Result<Event, ParseError> {
        let tok = self.bump().unwrap();
        let (name, span) = match tok.kind {
            TokenKind::Ident(s) => (s, tok.span),
            _ => unreachable!("parse_note_or_rest_event called without ident"),
        };

        if name == "r" {
            let dur = self.parse_duration_sticky(span.clone())?;
            return Ok(Event::Rest(dur));
        }

        let note = parse_note_name(&name).ok_or_else(|| {
            ParseError::new(ParseErrorKind::UnknownPitch(name.clone()), span.clone())
        })?;
        let octave_delta = self.consume_octave_marks();
        let pitch = pitch_from_parsed(note, octave_delta, span.clone())?;
        let dur = self.parse_duration_sticky(span)?;
        Ok(Event::Note(pitch, dur))
    }

    fn parse_chord_event(&mut self) -> Result<Event, ParseError> {
        let langle = self.bump().unwrap();
        debug_assert!(matches!(langle.kind, TokenKind::LAngle));
        let mut pitches = Vec::new();
        loop {
            match self.peek_kind() {
                Some(TokenKind::RAngle) => {
                    self.bump();
                    break;
                }
                Some(TokenKind::Ident(_)) => {
                    let tok = self.bump().unwrap();
                    let (name, span) = match tok.kind {
                        TokenKind::Ident(s) => (s, tok.span),
                        _ => unreachable!(),
                    };
                    let note = parse_note_name(&name).ok_or_else(|| {
                        ParseError::new(ParseErrorKind::UnknownPitch(name.clone()), span.clone())
                    })?;
                    let octave_delta = self.consume_octave_marks();
                    let pitch = pitch_from_parsed(note, octave_delta, span)?;
                    pitches.push(pitch);
                }
                None => {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedEof("'>' or pitch"),
                        self.eof_span(),
                    ));
                }
                Some(other) => {
                    return Err(ParseError::new(
                        ParseErrorKind::Expected {
                            expected: "pitch or '>'",
                            found: format!("{:?}", other),
                        },
                        self.cur_span(),
                    ));
                }
            }
        }
        let dur = self.parse_duration_sticky(langle.span)?;
        Ok(Event::Chord(pitches, dur))
    }

    fn consume_octave_marks(&mut self) -> i32 {
        let mut delta = 0;
        loop {
            match self.peek_kind() {
                Some(TokenKind::Apostrophe) => {
                    self.bump();
                    delta += 1;
                }
                Some(TokenKind::Comma) => {
                    self.bump();
                    delta -= 1;
                }
                _ => return delta,
            }
        }
    }

    /// Parse an optional duration, carrying the last duration forward if absent.
    fn parse_duration_sticky(&mut self, anchor_span: Span) -> Result<Duration, ParseError> {
        if let Some(dur) = self.try_parse_duration()? {
            self.last_duration = Some(dur);
            return Ok(dur);
        }
        self.last_duration
            .ok_or_else(|| ParseError::new(ParseErrorKind::MissingDuration, anchor_span))
    }

    fn try_parse_duration(&mut self) -> Result<Option<Duration>, ParseError> {
        let kind = match self.peek_kind() {
            Some(TokenKind::Number(_)) => {
                let tok = self.bump().unwrap();
                let (n, span) = match tok.kind {
                    TokenKind::Number(n) => (n, tok.span),
                    _ => unreachable!(),
                };
                duration_kind_from_number(n)
                    .ok_or_else(|| ParseError::new(ParseErrorKind::InvalidDuration(n), span))?
            }
            Some(TokenKind::Command(name)) if name == "breve" => {
                self.bump();
                DurationKind::Breve
            }
            _ => return Ok(None),
        };
        let mut dots = 0u8;
        while let Some(TokenKind::Dot) = self.peek_kind() {
            self.bump();
            dots += 1;
        }
        Ok(Some(Duration::new(kind, dots)))
    }

    fn expect_ident(&mut self, what: &'static str) -> Result<String, ParseError> {
        let tok = self.bump().ok_or_else(|| {
            ParseError::new(ParseErrorKind::UnexpectedEof(what), self.eof_span())
        })?;
        match tok.kind {
            TokenKind::Ident(s) => Ok(s),
            other => Err(ParseError::new(
                ParseErrorKind::Expected { expected: what, found: format!("{:?}", other) },
                tok.span,
            )),
        }
    }

    fn expect_number(&mut self, what: &'static str) -> Result<u32, ParseError> {
        let tok = self.bump().ok_or_else(|| {
            ParseError::new(ParseErrorKind::UnexpectedEof(what), self.eof_span())
        })?;
        match tok.kind {
            TokenKind::Number(n) => Ok(n),
            other => Err(ParseError::new(
                ParseErrorKind::Expected { expected: what, found: format!("{:?}", other) },
                tok.span,
            )),
        }
    }

    fn expect_command(&mut self, what: &'static str) -> Result<String, ParseError> {
        let tok = self.bump().ok_or_else(|| {
            ParseError::new(ParseErrorKind::UnexpectedEof(what), self.eof_span())
        })?;
        match tok.kind {
            TokenKind::Command(s) => Ok(s),
            other => Err(ParseError::new(
                ParseErrorKind::Expected { expected: what, found: format!("{:?}", other) },
                tok.span,
            )),
        }
    }

    fn expect_kind(&mut self, want: &TokenKind, what: &'static str) -> Result<(), ParseError> {
        let tok = self.bump().ok_or_else(|| {
            ParseError::new(ParseErrorKind::UnexpectedEof(what), self.eof_span())
        })?;
        if &tok.kind == want {
            Ok(())
        } else {
            Err(ParseError::new(
                ParseErrorKind::Expected { expected: what, found: format!("{:?}", tok.kind) },
                tok.span,
            ))
        }
    }
}

fn duration_kind_from_number(n: u32) -> Option<DurationKind> {
    Some(match n {
        1 => DurationKind::Whole,
        2 => DurationKind::Half,
        4 => DurationKind::Qtr,
        8 => DurationKind::Eighth,
        16 => DurationKind::Sixteenth,
        32 => DurationKind::ThirtySecond,
        64 => DurationKind::SixtyFourth,
        128 => DurationKind::OneTwentyEighth,
        _ => return None,
    })
}

/// Invert `impl ToLilypondString for Note`: map a lowercase LilyPond name to a
/// [`Note`]. Returns None for unknown input.
fn parse_note_name(s: &str) -> Option<Note> {
    Some(match s {
        "c" => Note::C,
        "cis" => Note::Cis,
        "cisis" => Note::Cisis,
        "ces" => Note::Ces,
        "ceses" => return None, // not modeled (C double-flat disallowed)
        "d" => Note::D,
        "dis" => Note::Dis,
        "disis" => Note::Disis,
        "des" => Note::Des,
        "deses" => Note::Deses,
        "e" => Note::E,
        "eis" => Note::Eis,
        "ees" => Note::Ees,
        "eeses" => Note::Eeses,
        "f" => Note::F,
        "fis" => Note::Fis,
        "fisis" => Note::Fisis,
        "fes" => Note::Fes,
        "feses" => return None,
        "g" => Note::G,
        "gis" => Note::Gis,
        "gisis" => Note::Gisis,
        "ges" => Note::Ges,
        "geses" => Note::Geses,
        "a" => Note::A,
        "ais" => Note::Ais,
        "aisis" => Note::Aisis,
        "aes" => Note::Aes,
        "aeses" => Note::Aeses,
        "b" => Note::B,
        "bis" => Note::Bis,
        "bes" => Note::Bes,
        "beses" => Note::Beses,
        _ => return None,
    })
}

/// Inverts the octave-calculation quirk in `impl ToLilypondString for Pitch`.
///
/// Output code: for `Ces`, octave is bumped +1 before emitting marks; for
/// `Bis`, octave is bumped -1. Then `'`/`,` count encodes `octave - 3` /
/// `3 - octave`. We recover the original octave by undoing the bump.
fn pitch_from_parsed(note: Note, octave_delta: i32, span: Span) -> Result<Pitch, ParseError> {
    let rendered_octave = 3 + octave_delta;
    let original_octave = match note {
        Note::Ces => rendered_octave - 1,
        Note::Bis => rendered_octave + 1,
        _ => rendered_octave,
    };
    if original_octave < -1 || original_octave > 9 {
        return Err(ParseError::new(
            ParseErrorKind::OctaveOutOfRange(original_octave),
            span,
        ));
    }
    Pitch::try_new(note, original_octave as i8).map_err(|_| {
        ParseError::new(ParseErrorKind::OctaveOutOfRange(original_octave), span)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_note() {
        let items = parse("c'4").unwrap();
        assert_eq!(items.len(), 1);
        match &items[0] {
            Item::Event(Event::Note(p, d)) => {
                assert_eq!(p.note, Note::C);
                assert_eq!(p.octave, 4);
                assert_eq!(d.kind(), DurationKind::Qtr);
                assert_eq!(d.num_dots(), 0);
            }
            other => panic!("expected note, got {:?}", other),
        }
    }

    #[test]
    fn sticky_duration() {
        let items = parse("c'4 d' e'").unwrap();
        assert_eq!(items.len(), 3);
        for item in &items {
            match item {
                Item::Event(Event::Note(_, d)) => assert_eq!(d.kind(), DurationKind::Qtr),
                _ => panic!(),
            }
        }
    }

    #[test]
    fn missing_duration_errors() {
        let err = parse("c'").unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::MissingDuration);
    }

    #[test]
    fn chord_and_rest() {
        let items = parse("<c' e' g'>4 r8").unwrap();
        match &items[0] {
            Item::Event(Event::Chord(ps, d)) => {
                assert_eq!(ps.len(), 3);
                assert_eq!(d.kind(), DurationKind::Qtr);
            }
            other => panic!("{:?}", other),
        }
        match &items[1] {
            Item::Event(Event::Rest(d)) => assert_eq!(d.kind(), DurationKind::Eighth),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn breve_and_dots() {
        let items = parse("c'\\breve d'4.").unwrap();
        match &items[0] {
            Item::Event(Event::Note(_, d)) => assert_eq!(d.kind(), DurationKind::Breve),
            _ => panic!(),
        }
        match &items[1] {
            Item::Event(Event::Note(_, d)) => {
                assert_eq!(d.kind(), DurationKind::Qtr);
                assert_eq!(d.num_dots(), 1);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn commands() {
        let items = parse("\\clef treble \\time 4/4 \\key c \\major c'4").unwrap();
        assert!(matches!(items[0], Item::Clef(ref s) if s == "treble"));
        assert!(matches!(items[1], Item::Time(4, 4)));
        assert!(matches!(items[2], Item::Key(ref t, ref m) if t == "c" && m == "major"));
    }

    #[test]
    fn tuplet() {
        let items = parse("\\tuplet 3/2 { c'8 d' e' }").unwrap();
        match &items[0] {
            Item::Tuplet { numerator: 3, denominator: 2, items: inner } => {
                assert_eq!(inner.len(), 3);
            }
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn block() {
        let items = parse("{ c'4 d' }").unwrap();
        match &items[0] {
            Item::Block(inner) => assert_eq!(inner.len(), 2),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn octave_marks() {
        // Middle C = c' in LilyPond absolute mode (octave 4).
        let items = parse("c'4 c4 c,4 c''4").unwrap();
        let octaves: Vec<i8> = items.iter().map(|i| match i {
            Item::Event(Event::Note(p, _)) => p.octave,
            _ => panic!(),
        }).collect();
        assert_eq!(octaves, vec![4, 3, 2, 5]);
    }

    #[test]
    fn ces_and_bis_octave_quirk() {
        // `ces'` renders from Pitch{Ces, 4}? Check round-trip via outputs below.
        // Here we just confirm parsing produces a valid Pitch with the inverse
        // adjustment: `ces'` -> apostrophes=1 -> rendered_oct=4 -> note Ces -> original 3.
        let items = parse("ces'4 bis4").unwrap();
        match &items[0] {
            Item::Event(Event::Note(p, _)) => {
                assert_eq!(p.note, Note::Ces);
                assert_eq!(p.octave, 3);
            }
            _ => panic!(),
        }
        match &items[1] {
            Item::Event(Event::Note(p, _)) => {
                assert_eq!(p.note, Note::Bis);
                assert_eq!(p.octave, 4);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn unknown_pitch() {
        let err = parse("h4").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::UnknownPitch(ref s) if s == "h"));
    }

    #[test]
    fn unterminated_block() {
        let err = parse("{ c'4 ").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::UnexpectedEof(_)));
    }
}
