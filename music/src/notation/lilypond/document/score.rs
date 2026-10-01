use crate::notation::lilypond::document::staff::LilypondStaff;
use crate::notation::lilypond::templates::TEMPLATE_ENGINE;
use crate::notation::lilypond::ToLilypondString;
use itertools::Itertools;
use tera::Context;

pub struct LilypondScore<'a> {
    staff_groups: Vec<LilypondStaffGroup<'a>>,
    layout: Option<LilypondLayout>,
    midi: Option<LilypondMidi>,
}

/// MIDI output configuration block.
/// When included in a score, Lilypond will generate a MIDI file alongside the PDF.
#[derive(Debug, Clone, PartialEq)]
pub struct LilypondMidi {
    /// Tempo in quarter notes per minute (e.g., 120 = 120 bpm)
    tempo: Option<u16>,
    /// MIDI instrument name (e.g., "acoustic grand", "electric guitar")
    instrument: Option<String>,
}

impl LilypondMidi {
    pub fn new() -> Self {
        Self {
            tempo: None,
            instrument: None,
        }
    }

    /// Set the tempo in quarter notes per minute.
    pub fn tempo(mut self, bpm: u16) -> Self {
        self.tempo = Some(bpm);
        self
    }

    /// Set the MIDI instrument name.
    pub fn instrument(mut self, instrument: impl Into<String>) -> Self {
        self.instrument = Some(instrument.into());
        self
    }
}

impl Default for LilypondMidi {
    fn default() -> Self {
        Self::new()
    }
}

impl ToLilypondString for LilypondMidi {
    fn to_lilypond_string(&self) -> String {
        let mut content = String::new();
        if let Some(tempo) = self.tempo {
            content.push_str(&format!("    \\tempo 4 = {}\n", tempo));
        }
        if let Some(instrument) = &self.instrument {
            content.push_str(&format!(
                "    \\set Staff.midiInstrument = #\"{}\"\n",
                instrument
            ));
        }
        format!("  \\midi {{\n{}}}\n", content)
    }
}

impl<'a> LilypondScore<'a> {
    pub fn new() -> Self {
        Self {
            staff_groups: vec![],
            layout: None,
            midi: None,
        }
    }

    pub fn layout(mut self, layout: Option<LilypondLayout>) -> Self {
        self.layout = layout;
        self
    }

    /// Add MIDI output configuration to this score.
    pub fn midi(mut self, midi: Option<LilypondMidi>) -> Self {
        self.midi = midi;
        self
    }

    pub fn staff_group(mut self, staff_group: LilypondStaffGroup<'a>) -> Self {
        self.staff_groups.push(staff_group);
        self
    }

    /// Returns all staff groups in this score.
    /// Used by `music::notation::rhythm::flatten::iter_events` (REQ-O16).
    pub fn staff_groups(&self) -> &[LilypondStaffGroup<'_>] {
        &self.staff_groups
    }
}

impl<'a> ToLilypondString for LilypondScore<'a> {
    fn to_lilypond_string(&self) -> String {
        let mut score_block = self
            .staff_groups
            .iter()
            .map(|group| group.to_lilypond_string())
            .join("\n");
        if let Some(layout) = &self.layout {
            score_block.push('\n');
            score_block = score_block + &layout.to_lilypond_string();
        }
        if let Some(midi) = &self.midi {
            score_block.push('\n');
            score_block = score_block + &midi.to_lilypond_string();
        }
        let mut ctx = Context::new();
        ctx.insert("content", &score_block);
        (*TEMPLATE_ENGINE).render("score", &ctx).unwrap()
    }
}

/// A group of staves that can optionally be bracketed together.
/// When bracketed, produces `\new StaffGroup << ... >>` in Lilypond output.
pub struct LilypondStaffGroup<'a> {
    staves: Vec<LilypondStaff<'a>>,
    /// When true, wraps staves in `\new StaffGroup` which adds a bracket
    bracketed: bool,
}

impl<'a> LilypondStaffGroup<'a> {
    pub fn new(staves: Vec<LilypondStaff<'a>>) -> Self {
        Self {
            staves,
            bracketed: false,
        }
    }

    /// Create a bracketed staff group (with connecting bracket on the left).
    pub fn bracketed(staves: Vec<LilypondStaff<'a>>) -> Self {
        Self {
            staves,
            bracketed: true,
        }
    }

    /// Set whether this group should have a bracket.
    pub fn set_bracketed(mut self, bracketed: bool) -> Self {
        self.bracketed = bracketed;
        self
    }

    /// Add a staff to this group.
    pub fn add_staff(mut self, staff: LilypondStaff<'a>) -> Self {
        self.staves.push(staff);
        self
    }

    /// Returns all staves in this staff group.
    /// Used by `music::notation::rhythm::flatten::iter_events` (REQ-O16).
    pub fn staves(&self) -> &[LilypondStaff<'_>] {
        &self.staves
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::lilypond::document::staff::LilypondStaff;

    #[test]
    fn staff_groups_returns_slice() {
        let staff = LilypondStaff::new();
        let group = LilypondStaffGroup::new(vec![staff]);
        let score = LilypondScore::new().staff_group(group);
        assert_eq!(score.staff_groups().len(), 1);
    }

    #[test]
    fn staves_returns_slice() {
        let staff_a = LilypondStaff::new();
        let staff_b = LilypondStaff::new();
        let group = LilypondStaffGroup::new(vec![staff_a, staff_b]);
        assert_eq!(group.staves().len(), 2);
    }
}

impl<'a> ToLilypondString for LilypondStaffGroup<'a> {
    fn to_lilypond_string(&self) -> String {
        let staves = self
            .staves
            .iter()
            .map(|staff| staff.to_lilypond_string())
            .join("\n");
        if self.bracketed {
            format!("\\new StaffGroup <<\n{}\n  >>", staves)
        } else {
            format!("<<{}  >>", staves)
        }
    }
}

pub struct LilypondLayout {
    ragged_right: bool,
    contexts: Vec<LilypondLayoutContext>,
}

impl LilypondLayout {
    pub fn new() -> Self {
        Self {
            ragged_right: false,
            contexts: vec![],
        }
    }

    pub fn ragged_right(mut self, ragged_right: bool) -> Self {
        self.ragged_right = ragged_right;
        self
    }

    pub fn add_context(mut self, context: LilypondLayoutContext) -> Self {
        self.contexts.push(context);
        self
    }
}

impl ToLilypondString for LilypondLayout {
    fn to_lilypond_string(&self) -> String {
        let mut statements: Vec<String> = vec![];
        let ragged_right = if self.ragged_right {
            "ragged-right = ##t"
        } else {
            "ragged-right = ##f"
        }
        .to_string();
        statements.push(ragged_right);
        self.contexts.iter().for_each(|ctx| {
            statements.push(ctx.to_lilypond_string());
        });
        let mut ctx = Context::new();
        ctx.insert("statements", &statements);
        (*TEMPLATE_ENGINE).render("layout", &ctx).unwrap()
    }
}

/// Context types used within `\layout` blocks to modify engraving behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutContextTy {
    /// Voice context - individual melodic line
    Voice,
    /// TabVoice context - voice within a TabStaff
    TabVoice,
    /// Staff context - standard notation staff
    Staff,
    /// TabStaff context - tablature staff
    TabStaff,
    /// StaffGroup context - group of staves with bracket
    StaffGroup,
    /// Score context - top-level score settings
    Score,
}

impl ToLilypondString for LayoutContextTy {
    fn to_lilypond_string(&self) -> String {
        match &self {
            LayoutContextTy::Voice => "\\Voice\n",
            LayoutContextTy::TabVoice => "\\TabVoice\n",
            LayoutContextTy::Staff => "\\Staff\n",
            LayoutContextTy::TabStaff => "\\TabStaff\n",
            LayoutContextTy::StaffGroup => "\\StaffGroup\n",
            LayoutContextTy::Score => "\\Score\n",
        }
        .to_string()
    }
}

pub struct LilypondLayoutContext {
    ty: Option<LayoutContextTy>,
    statements: Vec<String>, // TODO Make an enum for various kinds of statements
}

impl LilypondLayoutContext {
    pub fn new() -> Self {
        Self {
            ty: None,
            statements: vec![],
        }
    }

    pub fn layout_type(mut self, ty: Option<LayoutContextTy>) -> Self {
        self.ty = ty;
        self
    }

    pub fn add_statement(mut self, statement: String) -> Self {
        self.statements.push(statement);
        self
    }
}

impl ToLilypondString for LilypondLayoutContext {
    fn to_lilypond_string(&self) -> String {
        let mut statements: Vec<String> = vec![];
        if let Some(ty) = &self.ty {
            statements.push(ty.to_lilypond_string())
        }
        self.statements.iter().for_each(|st| {
            let mut st = st.clone();
            if !st.ends_with("\n") {
                st.push('\n');
            }
            statements.push(st);
        });
        let mut ctx = Context::new();
        ctx.insert("statements", &statements);
        (*TEMPLATE_ENGINE).render("layout_context", &ctx).unwrap()
    }
}

// TODO \set Score.markFormatter = #format-mark-box-alphabet
/* This
\layout {
  \context { \Voice
  \remove "New_fingering_engraver"
  }
}

\paper {
  #(define fonts (set-global-fonts #:music "paganini"))
  system-system-spacing = #'((padding . 4))
}

 */
