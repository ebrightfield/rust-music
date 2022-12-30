use itertools::Itertools;
use tera::Context;
use crate::notation::lilypond::document::staff::LilypondStaff;
use crate::notation::lilypond::ToLilypondString;
use crate::notation::lilypond::templates::TEMPLATE_ENGINE;

pub struct LilypondScore {
    staff_groups: Vec<LilypondStaffGroup>,
    layout: Option<LilypondLayout>,
    // TODO midi block
}

impl LilypondScore {
    pub fn new() -> Self {
        Self {
            staff_groups: vec![],
            layout: None,
        }
    }

    pub fn layout(mut self, layout: Option<LilypondLayout>) -> Self {
        self.layout = layout;
        self
    }

    pub fn staff_group(mut self, staff_group: LilypondStaffGroup) -> Self {
        self.staff_groups.push(staff_group);
        self
    }
}

impl ToLilypondString for LilypondScore {
    fn to_lilypond_string(&self) -> String {
        let mut score_block = self.staff_groups.iter()
            .map(|group| group.to_lilypond_string())
            .join("\n");
        if let Some(layout) = &self.layout {
            score_block.push('\n');
            score_block = score_block + &layout.to_lilypond_string();
        }
        let mut ctx = Context::new();
        ctx.insert("content", &score_block);
        (*TEMPLATE_ENGINE).render("score", &ctx).unwrap()
    }
}

// TODO bracketed: bool (`\new StaffGroup`)
pub struct LilypondStaffGroup(Vec<LilypondStaff>);

impl LilypondStaffGroup {
    pub fn new(groups: Vec<LilypondStaff>) -> Self {
        Self(groups)
    }
}

impl ToLilypondString for LilypondStaffGroup {
    fn to_lilypond_string(&self) -> String {
        let staves = self.0.iter()
            .map(|staff| staff.to_lilypond_string())
            .join("\n");
        format!("<<\n{}\n>>\n", staves)
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
        }.to_string();
        statements.push(ragged_right);
        self.contexts.iter().for_each(|ctx| {
            statements.push(ctx.to_lilypond_string());
        });
        let mut ctx = Context::new();
        ctx.insert("statements", &statements);
        (*TEMPLATE_ENGINE).render("layout", &ctx).unwrap()
    }
}

// TODO More types, Staff, StaffGroup, TabStaff, TabVoice, I think there's more...
pub enum LayoutContextTy {
    Voice,
}

impl ToLilypondString for LayoutContextTy {
    fn to_lilypond_string(&self) -> String {
        match &self {
            LayoutContextTy::Voice => "\\Voice\n"
        }.to_string()
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