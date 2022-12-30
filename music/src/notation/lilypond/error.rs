use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LilypondError {
    #[error("Document has no path")]
    DocumentHasNoPath,
    #[error("Failed to write Lilypond document to filesystem: {0}")]
    DocumentWriteFailure(io::Error),
    #[error("Failed to compile Lilypond document: {0}")]
    CompilationFailure(io::Error),
    #[error("Lilypond document does not exist: {0}")]
    DocumentDoesNotExist(String),
}
