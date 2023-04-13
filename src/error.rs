use crate::raw::{header::HeaderError, map_list::MapListError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error parsing header: {0}")]
    HeaderError(#[from] HeaderError),
    #[error("error parsing map_list: {0}")]
    MapListError(#[from] MapListError),
    #[error("read error: {0}")]
    ScrollError(#[from] scroll::Error),
}
