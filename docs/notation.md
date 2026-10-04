# Notation Output: Lilypond and VexTab

The notation module provides tools for converting musical data to notation output formats.

**Location:** `music::notation`

## Module Structure

```
notation/
├── lilypond/       - Lilypond source generation (feature-gated)
├── vextab.rs       - VexTab source generation
├── rhythm/         - Duration and meter types
└── clef.rs         - Clef definitions
```

## Enabling Lilypond Support

Lilypond output is behind a feature flag. Enable it in `Cargo.toml`:

```toml
[dependencies]
music = { path = "path/to/music", features = ["lilypond"] }
```

## Rhythm Types

**Location:** `music::notation::rhythm`

### Duration

Rhythmic duration values:

```rust
use music::notation::rhythm::duration::{Duration, DurationKind};

// Common durations
let whole = Duration::WHOLE;
let half = Duration::HALF;
let quarter = Duration::QTR;
let eighth = Duration::EIGHTH;
let sixteenth = Duration::SIXTEENTH;

// Duration kinds
pub enum DurationKind {
    Whole,
    Half,
    Qtr,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
}
```

### RhythmicNotatedEvent

Combines pitch content with duration:

```rust
use music::notation::rhythm::{RhythmicNotatedEvent, Duration};
use music::{pitch, voicing, Voicing};

// Single pitch with duration
let note = RhythmicNotatedEvent::pitch(pitch!(c, 4), Duration::QTR);

// Voicing (chord) with duration
let chord = RhythmicNotatedEvent::voicing(
    voicing!(pitch!(c, 4), pitch!(e, 4), pitch!(g, 4)),
    Duration::HALF
);

// Rest
let rest = RhythmicNotatedEvent::rest(Duration::EIGHTH);

// Tied notes
let tied = RhythmicNotatedEvent::pitch_tied(pitch!(c, 4), Duration::QTR);

// Fretted notes
let fretted = RhythmicNotatedEvent::fretted(sounded_note, Duration::QTR);
let fretted_many = RhythmicNotatedEvent::fretted_many(vec![note1, note2], Duration::QTR);
```

### SingleEvent

The pitch content portion of a rhythmic event:

```rust
pub enum SingleEvent<'a> {
    Pitch(Pitch),                   // Single note
    Voicing(Voicing),               // Chord
    Fretted(SoundedNote<'a>),       // Single fretted note
    FrettedMany(Vec<SoundedNote<'a>>), // Multiple fretted notes
    Rest,                           // Silence
}
```

### Tuplets

For rhythmic groupings like triplets:

```rust
use music::notation::rhythm::{RhythmicNotatedEvent, Tuplet, Duration};
use music::notation::rhythm::duration::DurationKind;

// Create an eighth-note triplet
let triplet = Tuplet::new(
    vec![
        RhythmicNotatedEvent::pitch(pitch!(c, 4), Duration::EIGHTH),
        RhythmicNotatedEvent::pitch(pitch!(d, 4), Duration::EIGHTH),
        RhythmicNotatedEvent::pitch(pitch!(e, 4), Duration::EIGHTH),
    ],
    3,  // numerator: 3 virtual beats
    2,  // denominator: in the space of 2 real beats
    DurationKind::Eighth  // base unit
);

// Convert to RhythmicNotatedEvent
let event: RhythmicNotatedEvent = triplet.into();

// Tuplet properties
assert!(triplet.is_complete());
let virtual_dur = triplet.virtual_duration();  // 3 eighths
let real_dur = triplet.real_duration();        // 2 eighths
```

## VexTab Output

**Location:** `music::notation::vextab`

VexTab is a text-based music notation format for web applications.

### ToVexTab Trait

```rust
pub trait ToVexTab {
    fn to_vextab(&self) -> String;
}
```

Implemented for:
- `Pitch`
- `Voicing`
- `SoundedNote`
- `FrettedNote`
- `FretboardShape`
- `Duration`
- `RhythmicNotatedEvent`

### Usage

```rust
use music::notation::vextab::ToVexTab;
use music::{pitch, voicing};

// Single pitch
let vex = pitch!(c, 4).to_vextab();
assert_eq!(vex, "C/4");

// Accidentals use @ instead of b
let vex = pitch!(bes, 4).to_vextab();
assert_eq!(vex, "B@/4");

// Voicing (chord)
let vex = voicing!(pitch!(c, 4), pitch!(e, 4), pitch!(g, 4)).to_vextab();
assert_eq!(vex, "(C/4.E/4.G/4)");

// Fretted note: fret/string
let vex = sounded_note.to_vextab();
// e.g., "5/3" = 5th fret, 3rd string (from top in VexTab)

// Duration
let vex = Duration::QTR.to_vextab();
assert_eq!(vex, ":q");
```

### Duration Mappings

| Duration | VexTab |
|----------|--------|
| Whole | `:w` |
| Half | `:h` |
| Quarter | `:q` |
| Eighth | `:8` |
| Sixteenth | `:16` |
| Thirty-second | `:32` |

Dots are appended with `d`: `:qd` = dotted quarter

### Barlines

```rust
use music::notation::vextab::barline;

barline::BAR           // "|"
barline::DOUBLE_BAR    // "=||"
barline::REPEAT_BEGIN  // "=|:"
barline::REPEAT_END    // "=:|"
barline::DOUBLE_REPEAT // "=::"
barline::END_BAR       // "=|="
```

## Lilypond Output

**Location:** `music::notation::lilypond` (requires `lilypond` feature)

### Document Building

```rust
use music::notation::lilypond::document::LilypondBuilder;
use music::notation::lilypond::document::score::{LilypondScore, LilypondStaffGroup};
use music::notation::lilypond::document::staff::LilypondStaff;
use std::path::PathBuf;

// Create musical events
let events: Vec<_> = musical_events
    .into_iter()
    .map(|e| e.into())  // Convert to Lilypond event type
    .collect();

// Create a staff with one or more voices
let staff = LilypondStaff::new()
    .add_voice(events)
    .add_voice(another_voice);

// Create a score with staff groups
let score = LilypondScore::new()
    .staff_group(LilypondStaffGroup::new(vec![staff]));

// Build the document
let doc = LilypondBuilder::new()
    .path(Some(PathBuf::from("output.ly")))
    .score(Some(score));
```

### Compiling Lilypond

Requires `lilypond` to be installed on your system:

```rust
use music::notation::lilypond::command::LilypondCmdBuilder;

LilypondCmdBuilder::new()
    .builder(doc)
    .output(Some(PathBuf::from("output/")))
    .build_and_compile()
    .unwrap();
```

### Score Functions

```rust
use music::notation::lilypond::scoring;

// Wrap content in a score block
let score_block = scoring::score(content, true);  // true = ragged-right

// Wrap content in a staff block
let staff_block = scoring::staff(content, Some("4/4".to_string()));

// Wrap content in a tab staff
let tab_block = scoring::tab_staff(content, None);  // No time signature

// Create a markup block
let markup = scoring::markup("Some text".to_string());
```

### ToLilypondString Trait

```rust
pub trait ToLilypondString {
    fn to_lilypond_string(&self) -> String;
}
```

## Clef

**Location:** `music::notation::clef`

```rust
use music::notation::clef::Clef;

pub enum Clef {
    Treble,
    Treble8va,
    Treble8ba,
    Bass,
    Alto,  // C clef, C4 on the middle line (LilyPond `\clef alto`)
    Tenor, // C clef, C4 on the fourth line (LilyPond `\clef tenor`)
}

// Get the pitch bounds for a clef
let (bottom, top) = Clef::Treble.bounds();
```

## Complete Lilypond Example

```rust
use music::{pitch, voicing, Voicing};
use music::notation::rhythm::{RhythmicNotatedEvent, Tuplet, Duration};
use music::notation::rhythm::duration::DurationKind;
use music::notation::lilypond::command::LilypondCmdBuilder;
use music::notation::lilypond::document::LilypondBuilder;
use music::notation::lilypond::document::score::{LilypondScore, LilypondStaffGroup};
use music::notation::lilypond::document::staff::LilypondStaff;
use std::path::PathBuf;

// 1. Create musical content
let events = vec![
    RhythmicNotatedEvent::voicing(
        voicing![pitch!(c, 3), pitch!(e, 3), pitch!(bes, 3)],
        Duration::EIGHTH
    ),
    RhythmicNotatedEvent::pitch(pitch!(a, 3), Duration::EIGHTH),
    RhythmicNotatedEvent::pitch(pitch!(g, 3), Duration::QTR),
    Tuplet::new(
        vec![
            RhythmicNotatedEvent::pitch(pitch!(c, 4), Duration::EIGHTH),
            RhythmicNotatedEvent::pitch(pitch!(dis, 4), Duration::EIGHTH),
            RhythmicNotatedEvent::pitch(pitch!(e, 4), Duration::EIGHTH),
        ],
        3, 2, DurationKind::Eighth
    ).into(),
];

// 2. Convert to Lilypond events
let ly_events: Vec<_> = events.into_iter().map(|e| e.into()).collect();

// 3. Build staff and score
let staff = LilypondStaff::new().add_voice(ly_events);
let score = LilypondScore::new()
    .staff_group(LilypondStaffGroup::new(vec![staff]));

// 4. Create and compile document
let doc = LilypondBuilder::new()
    .path(Some(PathBuf::from("target/output.ly")))
    .score(Some(score));

LilypondCmdBuilder::new()
    .builder(doc)
    .output(Some(PathBuf::from("target/")))
    .build_and_compile()
    .unwrap();
```

## Tab Staff Support

For guitar tablature:

```rust
use music::notation::lilypond::document::tab_staff::LilypondTabStaff;

// Tab staves work with fretted notes
let tab = LilypondTabStaff::new()
    .add_voice(fretted_events);
```

## Fretboard Diagrams

```rust
use music::notation::lilypond::fretboard_diagram;

// Generate chord diagram markup
```
