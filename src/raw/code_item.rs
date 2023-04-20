use crate::raw::{encoded_value::EncodedCatchHandlerList, simple::TryItem, *};
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

type TriesPadding = ushort;

#[derive(Debug)]
pub struct CodeItem {
    pub registers_size: ushort,
    pub ins_size: ushort,
    pub outs_size: ushort,
    pub tries_size: ushort,
    pub debug_info_off: uint,
    pub insns: Vec<ushort>,
    pub tries: Option<Vec<TryItem>>,
    pub handlers: Option<EncodedCatchHandlerList>,
}

impl<'a> TryFromCtx<'a, scroll::Endian> for CodeItem {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let registers_size = src.gread_with(offset, ctx)?;
        let ins_size = src.gread_with(offset, ctx)?;
        let outs_size = src.gread_with(offset, ctx)?;
        let tries_size = src.gread_with(offset, ctx)?;
        let debug_info_off = src.gread_with(offset, ctx)?;
        let insns_size: uint = src.gread_with(offset, ctx)?;
        let insns = try_gread_vec_with!(src, offset, insns_size, ctx);
        // 2 bytes of padding to make `tries` four-byte aligned.
        // This element is only present if `tries_size` is non-zero and `insns_size` is odd.
        if insns_size % 2 != 0 && tries_size != 0 {
            src.gread_with::<TriesPadding>(offset, ctx)?;
        }
        let tries = if tries_size != 0 {
            Some(try_gread_vec_with!(src, offset, tries_size, ctx))
        } else {
            None
        };
        let handlers = if tries_size != 0 {
            Some(src.gread(offset)?)
        } else {
            None
        };
        Ok((
            Self {
                registers_size,
                ins_size,
                outs_size,
                tries_size,
                debug_info_off,
                insns,
                tries,
                handlers,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx<scroll::Endian> for CodeItem {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.registers_size, offset, ctx)?;
        dst.gwrite_with(self.ins_size, offset, ctx)?;
        dst.gwrite_with(self.outs_size, offset, ctx)?;
        dst.gwrite_with(self.tries_size, offset, ctx)?;
        dst.gwrite_with(self.debug_info_off, offset, ctx)?;
        dst.gwrite_with(self.insns.len() as uint, offset, ctx)?;
        try_gwrite_vec_with!(dst, offset, &self.insns, ctx);
        // 2 bytes of padding to make `tries` four-byte aligned.
        // This element is only present if `tries_size` is non-zero and `insns_size` is odd.
        if self.insns.len() % 2 != 0 && self.tries_size != 0 {
            dst.gwrite_with::<TriesPadding>(0, offset, ctx)?;
        }
        if let Some(tries) = self.tries {
            try_gwrite_vec_with!(dst, offset, tries, ctx);
        }
        if let Some(handlers) = self.handlers {
            dst.gwrite(handlers, offset)?;
        }
        Ok(*offset)
    }
}
