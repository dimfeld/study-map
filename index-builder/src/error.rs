use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Invalid query")]
  QueryParseError,

  #[error("Search Error: {0}")]
  TantivyError(tantivy::TantivyError),

  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

impl From<tantivy::TantivyError> for Error {
  fn from(t: tantivy::TantivyError) -> Error {
    Error::TantivyError(t)
  }
}
