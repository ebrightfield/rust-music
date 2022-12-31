use std::path::PathBuf;
use std::process::Stdio;
use crate::notation::lilypond::document::{LilypondBuilder, LilypondFile};
use crate::notation::lilypond::error::LilypondError;

#[derive(Debug, PartialEq)]
pub enum LilypondOutput {
    Pdf,
    Svg,
    Png,
}

pub struct LilypondCmdBuilder<'a> {
    formats: Vec<LilypondOutput>,
    output: Option<PathBuf>,
    files: Vec<LilypondFile<'a>>,
}

impl<'a> LilypondCmdBuilder<'a> {
    pub fn new() -> Self {
        Self {
            formats: vec![],
            output: None,
            files: vec![],
        }
    }

    pub fn formats(mut self, formats: Vec<LilypondOutput>) -> Self {
        self.formats = formats;
        self
    }

    pub fn output(mut self, output: Option<PathBuf>) -> Self {
        self.output = output;
        self
    }

    pub fn builder(mut self, builder: LilypondBuilder<'a>) -> Self {
        self.files.push(LilypondFile::Virtual(builder));
        self
    }

    pub fn preexisting(mut self, path: PathBuf) -> Self {
        self.files.push(LilypondFile::Preexisting(path));
        self
    }

    pub fn build_files(&self) -> Result<(), LilypondError> {
        for file in &self.files {
            match file {
                LilypondFile::Preexisting(path) => {
                    if !path.exists() {
                        return Err(LilypondError::DocumentDoesNotExist(
                            path.to_str().unwrap_or("").to_string())
                        );
                    }
                },
                LilypondFile::Virtual(builder) => {
                    builder.write_to_file()?;
                }
            }
        }
        Ok(())
    }

    pub fn compile(&self) -> Result<(), LilypondError> {
        let mut cmd = std::process::Command::new("lilypond");
        cmd.stderr(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        if self.formats.contains(&LilypondOutput::Pdf) {
            cmd.arg("--pdf");
        }
        if self.formats.contains(&LilypondOutput::Svg) {
            cmd.arg("--svg");
        }
        if self.formats.contains(&LilypondOutput::Png) {
            cmd.arg("--png");
        }
        if let Some(path) = &self.output {
            cmd.arg("--output");
            cmd.arg(path);
        }
        for file in &self.files {
            match file {
                LilypondFile::Preexisting(path) => {
                    cmd.arg(path);
                }
                LilypondFile::Virtual(builder) => {
                    let path = builder.get_path().as_ref().unwrap();
                    let path = path.to_str().unwrap();
                    cmd.arg(path);
                }
            }
        }
        let mut child_process = cmd.spawn().map_err(|e|
            LilypondError::CompilationFailure(e))?;
        let _ = child_process.wait()
            .map_err(|e| LilypondError::CompilationFailure(e))?;
        Ok(())
    }

    pub fn build_and_compile(&self) -> Result<(), LilypondError> {
        self.build_files()?;
        self.compile()
    }
}