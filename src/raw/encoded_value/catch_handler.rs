use crate::raw::*;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

/// An array of [`EncodedCatchHandler`]s.
///
/// Contains the offset of each handler, which is only
/// used for decoding and is ignored when encoding.
#[derive(Debug, Default)]
pub struct EncodedCatchHandlerList(pub(crate) Vec<(usize, EncodedCatchHandler)>);

impl EncodedCatchHandlerList {
    pub(crate) fn into_inner(self) -> Vec<(usize, EncodedCatchHandler)> {
        self.0
    }

    /// Used for decoding purposes only.
    pub(crate) fn find(&self, offset: ushort) -> Option<&EncodedCatchHandler> {
        self.0
            .iter()
            .find(|(off, _)| *off == offset as usize)
            .map(|(_, eh)| eh)
    }
}

impl<'a> TryFromCtx<'a> for EncodedCatchHandlerList {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let size = uleb128::read(src, offset)?;
        let mut handlers = Vec::with_capacity(size as usize);
        for _ in 0..size {
            handlers.push((*offset, src.gread_with(offset, ())?));
        }
        Ok((Self(handlers), *offset))
    }
}

impl TryIntoCtx for EncodedCatchHandlerList {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.0.len() as u64)?;
        for (_, eh) in self.0 {
            dst.gwrite_with(eh, offset, ())?;
        }
        Ok(*offset)
    }
}

#[derive(Debug, Clone)]
pub struct EncodedCatchHandler {
    pub size: i64,
    pub handlers: Vec<EncodedTypeAddrPair>,
    pub catch_all_addr: Option<ulong>,
}

impl<'a> TryFromCtx<'a> for EncodedCatchHandler {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let size = sleb128::read(src, offset)?;
        let handlers = try_gread_vec_with!(src, offset, size.abs(), ());
        let catch_all_addr = if size <= 0 {
            Some(uleb128::read(src, offset)?)
        } else {
            None
        };
        Ok((
            Self {
                size,
                handlers,
                catch_all_addr,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx for EncodedCatchHandler {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        sleb128::write(dst, offset, self.size)?;
        try_gwrite_vec_with!(dst, offset, self.handlers, ());
        if let Some(addr) = self.catch_all_addr {
            // TODO: check if size is <= 0
            uleb128::write(dst, offset, addr)?;
        }
        Ok(*offset)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EncodedTypeAddrPair {
    pub type_id: ulong,
    pub addr: ulong,
}

impl<'a> TryFromCtx<'a> for EncodedTypeAddrPair {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let type_id = uleb128::read(src, offset)?;
        let addr = uleb128::read(src, offset)?;
        Ok((Self { type_id, addr }, *offset))
    }
}

impl TryIntoCtx for EncodedTypeAddrPair {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.type_id)?;
        uleb128::write(dst, offset, self.addr)?;
        Ok(*offset)
    }
}
