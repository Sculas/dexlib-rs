use crate::raw::*;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

#[derive(Debug)]
pub struct TypeList(Vec<TypeItem>);

impl<'a> TryFromCtx<'a, scroll::Endian> for TypeList {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let size: uint = src.gread_with(offset, ctx)?;
        let type_items = try_gread_vec_with!(src, offset, size, ctx);
        Ok((Self(type_items), *offset))
    }
}

impl TryIntoCtx<scroll::Endian> for TypeList {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.0.len() as uint, offset, ctx)?;
        try_gwrite_vec_with!(dst, offset, self.0, ctx);
        Ok(*offset)
    }
}

#[derive(Debug, Clone, Copy, Pread, Pwrite)]
pub struct TypeItem {
    /// Index into the `type_ids` list.
    type_idx: ushort,
}
