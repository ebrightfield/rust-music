pub mod score;
pub mod staff;
pub mod tab_staff;

use std::path::PathBuf;
use itertools::Itertools;
use once_cell::sync::Lazy;
use tera::Context;
use score::LilypondScore;
use crate::notation::lilypond::document::score::LilypondLayout;
use crate::notation::lilypond::error::LilypondError;
use crate::notation::lilypond::templates::TEMPLATE_ENGINE;
use crate::notation::lilypond::ToLilypondString;

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
    score: Option<LilypondScore<'a>>,
    //version: String, // default "2.22.2"
    // TODO page and paper block
}

impl<'a> LilypondBuilder<'a> {
    pub fn new() -> Self {
        Self {
            path: None,
            includes: vec![],
            header: None,
            layout: vec![],
            score: None,
        }
    }

    pub fn include(mut self, include: LilypondInclude) -> Self {
        self.includes.push(include);
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
        let mut content = self.includes.iter()
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
            tagline: None
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
pub static LILYPOND_BOOK_PREAMBLE: Lazy<LilypondInclude> = Lazy::new(|| {
    LilypondInclude(PathBuf::from("lilypond-book-preamble.ly"))
});
