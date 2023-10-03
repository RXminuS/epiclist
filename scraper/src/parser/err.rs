use thiserror::Error;

use super::*;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("mismatched open and close tags")]
    MismatchedTags,
    #[error("unknown error: {0}")]
    Unknown(String),

    #[error(transparent)]
    MdHeadingBuilderError(#[from] MdHeadingBuilderError),
    #[error(transparent)]
    MdHeadingSectionBuilderError(#[from] MdHeadingSectionBuilderError),
    #[error(transparent)]
    MdParagraphBuilderError(#[from] MdParagraphBuilderError),
    #[error(transparent)]
    MdListItemBuilderError(#[from] MdListItemBuilderError),
    #[error(transparent)]
    MdListBuilderError(#[from] MdListBuilderError),
    #[error(transparent)]
    MdLinkBuilderError(#[from] MdLinkBuilderError),
}

pub type Result<T> = std::result::Result<T, ParserError>;
