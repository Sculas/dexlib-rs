use crate::types::header::HeaderError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error parsing header: {0}")]
    HeaderError(#[from] HeaderError),
}
