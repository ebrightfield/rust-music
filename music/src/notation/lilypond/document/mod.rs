pub mod score;
pub mod staff;
pub mod tab_staff;

use crate::notation::lilypond::document::score::LilypondLayout;
use crate::notation::lilypond::error::LilypondError;
use crate::notation::lilypond::templates::TEMPLATE_ENGINE;
use crate::notation::lilypond::ToLilypondString;
use itertools::Itertools;
use once_cell::sync::Lazy;
use score::LilypondScore;
use std::path::PathBuf;
use tera::Context;

/// Either a pre-existing lilypond source file,
/// or one defined in Rust code with a [LilypondBuilder].
pub enum LilypondFile<'a> {
    Preexisting(PathBuf),
    Virtual(LilypondBuilder<'a>),
}

/// Builder for a Lilypond document.
pub struct LilypondBuilder<'a> {
    path: Option<PathBuf>,
    includes: Vec<LilypondInclude>,
    header: Option<LilypondHeader>,
    layout: Vec<LilypondLayout>,
    paper: Option<LilypondPaper>,
    score: Option<LilypondScore<'a>>,
    //version: String, // default "2.22.2"
}

impl<'a> LilypondBuilder<'a> {
    pub fn new() -> Self {
        Self {
            path: None,
            includes: vec![],
            header: None,
            layout: vec![],
            paper: None,
            score: None,
        }
    }

    pub fn include(mut self, include: LilypondInclude) -> Self {
        self.includes.push(include);
        self
    }

    pub fn header(mut self, header: Option<LilypondHeader>) -> Self {
        self.header = header;
        self
    }

    pub fn path(mut self, path: Option<PathBuf>) -> Self {
        self.path = path;
        self
    }

    pub fn get_path(&self) -> &Option<PathBuf> {
        &self.path
    }

    pub fn score(mut self, score: Option<LilypondScore<'a>>) -> Self {
        self.score = score;
        self
    }

    /// Set the paper block configuration.
    pub fn paper(mut self, paper: Option<LilypondPaper>) -> Self {
        self.paper = paper;
        self
    }

    pub fn write_to_file(&self) -> Result<(), LilypondError> {
        let path = self.path.as_ref().ok_or(LilypondError::DocumentHasNoPath)?;
        let path = path.to_str().unwrap();
        std::fs::write(&path, self.to_lilypond_string())
            .map_err(|e| LilypondError::DocumentWriteFailure(e))?;
        Ok(())
    }
}

impl<'a> ToLilypondString for LilypondBuilder<'a> {
    fn to_lilypond_string(&self) -> String {
        let mut content = self
            .includes
            .iter()
            .map(|include| include.to_lilypond_string())
            .join("\n");
        if let Some(header) = &self.header {
            content.push('\n');
            content = content + &header.to_lilypond_string();
        }
        for layout in &self.layout {
            content.push('\n');
            content = content + &layout.to_lilypond_string();
        }
        if let Some(paper) = &self.paper {
            content.push('\n');
            content = content + &paper.to_lilypond_string();
        }
        if let Some(score) = &self.score {
            content.push('\n');
            content = content + &score.to_lilypond_string();
        }
        content
    }
}

/// A top-level block that defines title, composer, and tagline.
pub struct LilypondHeader {
    title: Option<String>,
    composer: Option<String>,
    tagline: Option<String>,
}

impl LilypondHeader {
    pub fn new() -> Self {
        Self {
            title: None,
            composer: None,
            tagline: None,
        }
    }

    pub fn title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    pub fn composer(mut self, composer: Option<String>) -> Self {
        self.composer = composer;
        self
    }

    pub fn tagline(mut self, tagline: Option<String>) -> Self {
        self.tagline = tagline;
        self
    }
}

impl ToLilypondString for LilypondHeader {
    fn to_lilypond_string(&self) -> String {
        let mut content = "".to_string();
        if let Some(title) = &self.title {
            content = content + &format!("  title = {}", title);
        }
        if let Some(composer) = &self.composer {
            content = content + &format!("  composer = {}", composer);
        }
        if let Some(tagline) = &self.tagline {
            content = content + &format!("  tagline = {}", tagline);
        } else {
            content = content + "  tagline = \"\"";
        }
        let mut ctx = Context::new();
        ctx.insert("content", &content);
        (*TEMPLATE_ENGINE).render("header", &ctx).unwrap()
    }
}

/// An import statement at the top of a lilypond file.
pub struct LilypondInclude(PathBuf);

impl ToLilypondString for LilypondInclude {
    fn to_lilypond_string(&self) -> String {
        format!("\\include {}\n", &self.0.display())
    }
}

impl<'a> TryInto<LilypondInclude> for &LilypondBuilder<'a> {
    type Error = LilypondError;

    fn try_into(self) -> Result<LilypondInclude, LilypondError> {
        if self.path.is_none() {
            return Err(LilypondError::DocumentHasNoPath);
        }
        Ok(LilypondInclude(self.path.as_ref().unwrap().clone()))
    }
}

/// This allows for cropping of lilypond staff systems.
pub static LILYPOND_BOOK_PREAMBLE: Lazy<LilypondInclude> =
    Lazy::new(|| LilypondInclude(PathBuf::from("lilypond-book-preamble.ly")));

/// Paper block configuration for controlling page layout and spacing.
///
/// Example Lilypond output:
/// ```lilypond
/// \paper {
///   #(define fonts (set-global-fonts #:music "paganini"))
///   system-system-spacing = #'((padding . 4))
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct LilypondPaper {
    /// Music font name (e.g., "paganini", "emmentaler")
    music_font: Option<String>,
    /// Padding between systems (staff groups)
    system_system_padding: Option<f32>,
    /// Top margin in mm
    top_margin: Option<f32>,
    /// Bottom margin in mm
    bottom_margin: Option<f32>,
    /// Left margin in mm
    left_margin: Option<f32>,
    /// Right margin in mm
    right_margin: Option<f32>,
    /// Paper size (e.g., "a4", "letter")
    paper_size: Option<String>,
}

impl LilypondPaper {
    pub fn new() -> Self {
        Self {
            music_font: None,
            system_system_padding: None,
            top_margin: None,
            bottom_margin: None,
            left_margin: None,
            right_margin: None,
            paper_size: None,
        }
    }

    /// Set the music font.
    pub fn music_font(mut self, font: impl Into<String>) -> Self {
        self.music_font = Some(font.into());
        self
    }

    /// Set the padding between systems in staff-space units.
    pub fn system_system_padding(mut self, padding: f32) -> Self {
        self.system_system_padding = Some(padding);
        self
    }

    /// Set the top margin in mm.
    pub fn top_margin(mut self, margin: f32) -> Self {
        self.top_margin = Some(margin);
        self
    }

    /// Set the bottom margin in mm.
    pub fn bottom_margin(mut self, margin: f32) -> Self {
        self.bottom_margin = Some(margin);
        self
    }

    /// Set the left margin in mm.
    pub fn left_margin(mut self, margin: f32) -> Self {
        self.left_margin = Some(margin);
        self
    }

    /// Set the right margin in mm.
    pub fn right_margin(mut self, margin: f32) -> Self {
        self.right_margin = Some(margin);
        self
    }

    /// Set the paper size (e.g., "a4", "letter", "legal").
    pub fn paper_size(mut self, size: impl Into<String>) -> Self {
        self.paper_size = Some(size.into());
        self
    }
}

impl Default for LilypondPaper {
    fn default() -> Self {
        Self::new()
    }
}

impl ToLilypondString for LilypondPaper {
    fn to_lilypond_string(&self) -> String {
        let mut statements = Vec::new();

        if let Some(size) = &self.paper_size {
            statements.push(format!("  #(set-paper-size \"{}\")", size));
        }
        if let Some(font) = &self.music_font {
            statements.push(format!(
                "  #(define fonts (set-global-fonts #:music \"{}\"))",
                font
            ));
        }
        if let Some(padding) = self.system_system_padding {
            statements.push(format!(
                "  system-system-spacing = #'((padding . {}))",
                padding
            ));
        }
        if let Some(margin) = self.top_margin {
            statements.push(format!("  top-margin = {}\\mm", margin));
        }
        if let Some(margin) = self.bottom_margin {
            statements.push(format!("  bottom-margin = {}\\mm", margin));
        }
        if let Some(margin) = self.left_margin {
            statements.push(format!("  left-margin = {}\\mm", margin));
        }
        if let Some(margin) = self.right_margin {
            statements.push(format!("  right-margin = {}\\mm", margin));
        }

        if statements.is_empty() {
            return String::new();
        }

        format!("\\paper {{\n{}\n}}\n", statements.join("\n"))
    }
}
