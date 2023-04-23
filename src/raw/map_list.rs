use crate::raw::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

#[derive(Debug, thiserror::Error)]
pub enum MapListError {
    #[error("invalid item type in map_list: {0}")]
    InvalidTypeId(u16),
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}

/// List of the entire contents of a file, in order. A given type must appear at most
/// once in a map, entries must be ordered by initial offset and must not overlap.
#[derive(Debug)]
pub struct MapList(Vec<MapItem>);

impl<'a> TryFromCtx<'a, scroll::Endian> for MapList {
    type Error = MapListError;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let size: uint = src.gread_with(offset, ctx)?;
        let map_items = try_gread_vec_with!(src, offset, size, ctx);
        Ok((Self(map_items), *offset))
    }
}

impl TryIntoCtx<scroll::Endian> for MapList {
    type Error = MapListError;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.0.len() as uint, offset, ctx)?;
        try_gwrite_vec_with!(dst, offset, self.0, ctx);
        Ok(*offset)
    }
}

impl MapList {
    /// Returns the `MapItem` corresponding to the [`ItemType`].
    pub fn get(&self, item_type: ItemType) -> Option<&MapItem> {
        self.0
            .iter()
            .find(|map_item| map_item.item_type == item_type)
    }

    /// Returns the offset of the item corresponding to the [`ItemType`].
    pub fn get_offset(&self, item_type: ItemType) -> Option<uint> {
        self.get(item_type).map(|map_item| map_item.offset)
    }

    /// Returns the length of the item corresponding to the [`ItemType`].
    pub fn get_len(&self, item_type: ItemType) -> Option<uint> {
        self.get(item_type).map(|map_item| map_item.size)
    }
}

/// Items that can be found in the MapList.
#[derive(FromPrimitive, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u16)] // ushort
pub enum ItemType {
    HeaderItem = 0x0,
    StringIdItem = 0x1,
    TypeIdItem = 0x2,
    ProtoIdItem = 0x3,
    FieldIdItem = 0x4,
    MethodIdItem = 0x5,
    ClassDefItem = 0x6,
    CallSiteIdItem = 0x7,
    MethodHandleItem = 0x8,
    MapList = 0x1000,
    TypeList = 0x1001,
    AnnotationSetRefList = 0x1002,
    AnnotationSetItem = 0x1003,
    ClassDataItem = 0x2000,
    CodeItem = 0x2001,
    StringDataItem = 0x2002,
    DebugInfoItem = 0x2003,
    AnnotationItem = 0x2004,
    EncodedArrayItem = 0x2005,
    AnnotationsDirectoryItem = 0x2006,
    HiddenapiClassDataItem = 0xF000,
}

/// Single item of the MapList.
#[derive(Debug, Clone, Copy)]
pub struct MapItem {
    /// Type of the current item.
    pub item_type: ItemType,
    /// Count of the number of items to be found at the indicated offset.
    pub size: uint,
    /// Offset from the start of the file to the current item type.
    pub offset: uint,
}

impl<'a> TryFromCtx<'a, scroll::Endian> for MapItem {
    type Error = MapListError;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let ty: ushort = src.gread_with(offset, ctx)?;
        let item_type = ItemType::from_u16(ty).ok_or_else(|| MapListError::InvalidTypeId(ty))?;
        let __reserved: ushort = src.gread_with(offset, ctx)?;
        debug_assert_eq!(__reserved, RESERVED_VALUE as ushort);
        let size: uint = src.gread_with(offset, ctx)?;
        let item_offset: uint = src.gread_with(offset, ctx)?;
        Ok((
            Self {
                item_type,
                size,
                offset: item_offset,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx<scroll::Endian> for MapItem {
    type Error = MapListError;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.item_type as ushort, offset, ctx)?;
        dst.gwrite_with(RESERVED_VALUE as ushort, offset, ctx)?;
        dst.gwrite_with(self.size, offset, ctx)?;
        dst.gwrite_with(self.offset, offset, ctx)?;
        Ok(*offset)
    }
}
