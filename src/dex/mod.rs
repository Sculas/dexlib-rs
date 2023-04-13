use scroll::Pread;

use crate::error::Error;
use crate::raw::header::Header;
use crate::raw::map_list::MapList;
use crate::raw::tysize;

mod section;
#[macro_use]
mod utils;

pub struct DexFile<'a> {
    pub src: &'a [u8],
    pub header: Header<'a>,
    pub map_list: MapList,
}

// public api
impl<'a> DexFile<'a> {
    pub fn new(src: &'a [u8]) -> Result<Self, Error> {
        let header: Header = src.pread_with(0, scroll::LE)?;
        let map_list: MapList = src.pread_with(header.map_off as usize, scroll::LE)?;
        Ok(Self {
            src,
            header,
            map_list,
        })
    }
}

// sections
impl<'a> DexFile<'a> {
    section!(string_ids, tysize::STRING_ID);
    section!(type_ids, tysize::TYPE_ID);
    section!(proto_ids, tysize::PROTO_ID);
    section!(field_ids, tysize::FIELD_ID);
    section!(method_ids, tysize::METHOD_ID);
    section!(class_defs, tysize::CLASS_DEF);
}
