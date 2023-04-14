use std::sync::Arc;

use cesu8::{from_java_cesu8, to_java_cesu8, Cesu8DecodingError};
use scroll::Pread;

use crate::{
    raw::{
        header::Header,
        string::{StringData, StringError, StringId},
        tysize, uint,
    },
    utils::{nohash::BuildNoHashHasher, IntoArc},
};

use super::section::Section;

/// This is the same as [`StringId`]'s data offset, but it's a
/// direct typedef to [`uint`] instead of a newtype struct.
type RawStringId = uint;
type Result<T> = std::result::Result<T, StringCacheError>;
pub type DexString = Arc<String>;

#[derive(Debug, thiserror::Error)]
pub enum StringCacheError {
    #[error("string not found")]
    StringNotFound,
    #[error("string index {0} is out of bounds")]
    IndexOutOfBounds(uint),
    #[error("string data offset {0} is out of bounds")]
    OffsetOutOfBounds(RawStringId),
    #[error("string at offset {0} is malformed")]
    Malformed(RawStringId, #[source] Cesu8DecodingError),
    #[error("read error: {0}")]
    StringError(#[from] StringError),
    #[error("read error: {0}")]
    ScrollError(#[from] scroll::Error),
}

pub struct StringCache<'a> {
    src: &'a [u8],
    header: Header<'a>,
    // string id section
    section: Section<'a>,
    read_cache: dashmap::DashMap<RawStringId, DexString, BuildNoHashHasher<RawStringId>>,
    // a list of custom strings that need to be written to the dex file
    added_strings: Vec<Vec<u8>>,
}

impl<'a> StringCache<'a> {
    pub fn new(src: &'a [u8], header: Header<'a>, section: Section<'a>) -> Self {
        Self {
            src,
            header,
            section,
            read_cache: Default::default(),
            added_strings: Vec::new(),
        }
    }

    #[allow(clippy::len_without_is_empty)] // no need for that here
    pub fn len(&self) -> uint {
        self.header.string_ids_size
    }

    pub fn id_at(&self, index: uint) -> Result<StringId> {
        if index >= self.header.string_ids_size {
            return Err(StringCacheError::IndexOutOfBounds(index));
        }
        let offset = self.header.string_ids_off as usize + index as usize * tysize::STRING_ID;
        let id = self.src.pread_with(offset, scroll::LE)?;
        Ok(id)
    }

    pub fn get(&self, id: &StringId) -> Result<DexString> {
        let data_offset = id.offset();
        match self.read_cache.get(&data_offset) {
            Some(v) => Ok(v.value().clone()),
            None => {
                if !self.header.in_data_section(data_offset) {
                    return Err(StringCacheError::OffsetOutOfBounds(data_offset));
                }
                let data: StringData = self.src.pread_with(data_offset as usize, scroll::LE)?;
                let str = from_java_cesu8(data.data)
                    .map_err(|e| StringCacheError::Malformed(data_offset, e))?
                    .into_owned()
                    .into_arc();
                self.read_cache.insert(data_offset, str.clone());
                Ok(str)
            }
        }
    }

    pub fn find(&self, query: &str) -> Result<StringId> {
        let element = to_java_cesu8(query);
        let index = self
            .section
            .binary_search(&element, scroll::LE, move |offset: &uint, element: _| {
                let data: StringData = self.src.pread_with(*offset as usize, scroll::LE)?;
                Ok::<_, StringCacheError>((**element).cmp(data.data))
            })?
            .ok_or_else(|| StringCacheError::StringNotFound)?;
        self.id_at(index as uint)
    }

    // TODO: does this need to be parallelized?
    #[allow(dead_code)] // TODO: remove
    pub(crate) fn add(&mut self, string: String) {
        self.added_strings.push(to_java_cesu8(&string).into_owned());
    }
}
