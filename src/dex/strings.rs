use super::section::Section;
use crate::{
    raw::{
        header::Header,
        string::{StringData, StringId},
        uint, RawTypeIndex,
    },
    utils::{nohash::BuildNoHashHasher, IntoArc},
};
use cesu8::{from_java_cesu8, to_java_cesu8, Cesu8DecodingError};
use scroll::Pread;
use std::sync::Arc;

/// This is the same as [`StringId`]'s data offset, but it's a
/// direct typedef to [`uint`] instead of a newtype struct.
type RawStringId = uint;
type Result<T> = std::result::Result<T, StringReadError>;
pub type DexString = Arc<String>;

#[derive(Debug, thiserror::Error)]
pub enum StringReadError {
    #[error("string not found")]
    StringNotFound,
    #[error("string index {0} is out of bounds")]
    IndexOutOfBounds(uint),
    #[error("string data offset {0} is out of bounds")]
    OffsetOutOfBounds(RawStringId),
    #[error("string at offset {0} is malformed")]
    Malformed(RawStringId, #[source] Cesu8DecodingError),
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}

pub struct Strings<'a> {
    src: &'a [u8],
    header: Header<'a>,
    // string id section
    string_section: Section<'a>,
    // type id section
    type_section: Section<'a>,
    read_cache: dashmap::DashMap<RawStringId, DexString, BuildNoHashHasher<RawStringId>>,
}

impl<'a> Strings<'a> {
    pub fn new(
        src: &'a [u8],
        header: Header<'a>,
        string_section: Section<'a>,
        type_section: Section<'a>,
    ) -> Self {
        Self {
            src,
            header,
            string_section,
            type_section,
            read_cache: Default::default(),
        }
    }

    #[allow(clippy::len_without_is_empty)] // no need for that here
    pub fn len(&self) -> uint {
        self.header.string_ids_size
    }

    pub fn id_at_idx(&self, index: uint) -> Result<StringId> {
        if index >= self.len() {
            return Err(StringReadError::IndexOutOfBounds(index));
        }
        let id = self.string_section.index(index as usize, scroll::LE)?;
        Ok(id)
    }

    pub fn get(&self, id: &StringId) -> Result<DexString> {
        let data_offset = id.offset();
        match self.read_cache.get(&data_offset) {
            Some(v) => Ok(v.value().clone()),
            None => {
                if !self.header.in_data_section(data_offset) {
                    return Err(StringReadError::OffsetOutOfBounds(data_offset));
                }
                let data: StringData = self.src.pread_with(data_offset as usize, scroll::LE)?;
                let str = from_java_cesu8(data.data)
                    .map_err(|e| StringReadError::Malformed(data_offset, e))?
                    .into_owned()
                    .into_arc();
                self.read_cache.insert(data_offset, str.clone());
                Ok(str)
            }
        }
    }

    pub fn id_at_type_idx(&self, type_idx: RawTypeIndex) -> Result<StringId> {
        let id: crate::raw::simple::TypeId =
            self.type_section.index(type_idx as usize, scroll::LE)?;
        self.id_at_idx(id.descriptor_idx)
    }

    pub fn find(&self, query: &str) -> Result<StringId> {
        let element = to_java_cesu8(query);
        let index = self
            .string_section
            .binary_search(&element, scroll::LE, move |offset: &uint, element: _| {
                let data: StringData = self.src.pread_with(*offset as usize, scroll::LE)?;
                Ok::<_, StringReadError>((**element).cmp(data.data))
            })?
            .ok_or_else(|| StringReadError::StringNotFound)?;
        self.id_at_idx(index.try_into().expect("bad string idx?"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test() {
        let dex = crate::t::dex!();
        let sidx = dex.strings().len() / 2;
        let sid_1 = dex.strings().id_at_idx(sidx).unwrap();
        let str = dex.strings().get(&sid_1).unwrap();
        let sid_2 = dex.strings().find(&str).unwrap();
        assert_eq!(sid_1, sid_2);
    }
}
