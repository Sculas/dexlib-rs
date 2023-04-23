use scroll::Pread;

use crate::{
    error::Error,
    raw::{
        header::Header,
        map_list::{ItemType, MapList},
        tysize,
    },
};
use strings::Strings;

pub mod annotations;
pub mod classes;
pub(crate) mod internal;
pub(crate) mod section;
pub mod strings;
#[macro_use]
mod utils;

pub struct DexFile<'a> {
    src: &'a [u8],
    header: Header<'a>,
    map_list: MapList,
    strings: Strings<'a>,

    // internal
    classdef_info: utils::OffsetInfo,
}

impl<'a> DexFile<'a> {
    pub fn open(src: &'a [u8]) -> crate::Result<Self> {
        let header: Header = src.pread_with(0, scroll::LE)?;
        let map_list: MapList = src.pread_with(header.map_off as usize, scroll::LE)?;
        let strings = Strings::new(
            src,
            /* shallow clone */ header.clone(),
            raw_string_ids_section(src, &header),
            raw_type_ids_section(src, &header),
        );
        Ok(Self {
            src,
            header,
            strings,

            // internal
            classdef_info: get_offset_info!(map_list, ItemType::ClassDefItem),

            // map_list last to prevent move errors
            map_list,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn map_list(&self) -> &MapList {
        &self.map_list
    }

    pub fn strings(&self) -> &Strings {
        &self.strings
    }

    pub fn classes(&self) -> classes::iter::ClassIterator {
        classes::iter::ClassIterator::new(self, self.classdef_info.size)
    }
}

// sections
section!(DexFile, string_ids, tysize::STRING_ID);
section!(DexFile, type_ids, tysize::TYPE_ID);
section!(DexFile, proto_ids, tysize::PROTO_ID);
section!(DexFile, field_ids, tysize::FIELD_ID);
section!(DexFile, method_ids, tysize::METHOD_ID);
section!(DexFile, class_defs, tysize::CLASS_DEF);
section!(
    map(CallSiteIdItem): DexFile,
    call_site_ids,
    tysize::CALL_SITE_ID
);
section!(
    map(MethodHandleItem): DexFile,
    method_handles,
    tysize::METHOD_HANDLE
);

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "debug"]
    pub fn header() {
        let dex = crate::t::dex!();
        println!("{:#?}", dex.header());
    }

    #[test]
    #[ignore = "debug"]
    pub fn map_list() {
        let dex = crate::t::dex!();
        println!("{:#?}", dex.map_list());
    }
}
