use music::notation::rhythm::duration::Duration;
use music::Pitch;

/// A single rhythmic event inside a voice block.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Note(Pitch, Duration),
    Chord(Vec<Pitch>, Duration),
    Rest(Duration),
}

/// A top-level item inside a `{ ... }` block.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Event(Event),
    /// `\clef treble` etc. — kept as the argument ident for now; no semantic
    /// conversion until a consumer needs it.
    Clef(String),
    /// `\time 4/4`
    Time(u32, u32),
    /// `\key c \major` — key note name and mode ident.
    Key(String, String),
    /// Nested `{ ... }`.
    Block(Vec<Item>),
    /// `\tuplet n/d { ... }`.
    Tuplet {
        numerator: u32,
        denominator: u32,
        items: Vec<Item>,
    },
}
