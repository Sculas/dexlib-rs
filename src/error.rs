use crate::{
    dex::strings::StringCacheError,
    raw::{header::HeaderError, map_list::MapListError},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error parsing header: {0}")]
    Header(#[from] HeaderError),
    #[error("error parsing map_list: {0}")]
    MapList(#[from] MapListError),
    #[error("error in string cache: {0}")]
    StringCache(#[from] StringCacheError),
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}
