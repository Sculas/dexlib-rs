use crate::{
    dex::strings::StringReadError,
    raw::{header::HeaderError, map_list::MapListError},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error parsing header: {0}")]
    Header(#[from] HeaderError),
    #[error("error parsing map_list: {0}")]
    MapList(#[from] MapListError),
    #[error("error reading string: {0}")]
    StringRead(#[from] StringReadError),
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}
