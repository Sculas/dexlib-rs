use scroll::Pread;

use crate::raw::{header::Header, map_list::MapList, tysize};
use strings::StringCache;

mod section;
pub mod strings;
#[macro_use]
mod utils;

pub struct DexFile<'a> {
    src: &'a [u8],
    header: Header<'a>,
    map_list: MapList,
    strings: StringCache<'a>,
}

impl<'a> DexFile<'a> {
    pub fn new(src: &'a [u8]) -> crate::Result<Self> {
        let header: Header = src.pread_with(0, scroll::LE)?;
        let map_list: MapList = src.pread_with(header.map_off as usize, scroll::LE)?;
        let strings = StringCache::new(
            src,
            /* shallow clone */ header.clone(),
            raw_string_ids_section(src, &header),
        );
        Ok(Self {
            src,
            header,
            map_list,
            strings,
        })
    }
    pub fn header(&self) -> &Header {
        &self.header
    }
    pub fn map_list(&self) -> &MapList {
        &self.map_list
    }
    pub fn strings(&self) -> &StringCache {
        &self.strings
    }
}

// sections
section!(DexFile, string_ids, tysize::STRING_ID);
section!(DexFile, type_ids, tysize::TYPE_ID);
section!(DexFile, proto_ids, tysize::PROTO_ID);
section!(DexFile, field_ids, tysize::FIELD_ID);
section!(DexFile, method_ids, tysize::METHOD_ID);
section!(DexFile, class_defs, tysize::CLASS_DEF);
