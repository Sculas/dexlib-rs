use crate::raw::*;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread,
};

// TODO: move stubs and write serialization impls

// TODO: STUB!
#[derive(Debug, Clone)]
pub enum ExceptionType {
    /// The `Exception` class.
    BaseException,
    /// Sub-types of the `Exception` class.
    Type(RawTypeId),
}

#[derive(Debug, Clone)]
pub struct CatchHandler {
    /// Type of the exception handled by this handler.
    pub exception: ExceptionType,
    /// Start address of the catch handler.
    pub addr: ulong,
}
// TODO: STUB!

#[derive(Debug, Clone)]
pub(crate) struct EncodedCatchHandlers {
    inner: Vec<EncodedCatchHandler>,
}

impl EncodedCatchHandlers {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &EncodedCatchHandler> {
        self.inner.iter()
    }

    pub(crate) fn find(&self, offset: usize) -> Option<&EncodedCatchHandler> {
        self.iter().find(|p| p.offset == offset)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EncodedCatchHandler {
    offset: usize,
    handlers: Vec<CatchHandler>,
}

impl EncodedCatchHandler {
    pub(crate) fn handlers(&self) -> Vec<CatchHandler> {
        self.handlers.to_vec()
    }
}

impl<'a> TryFromCtx<'a, usize> for EncodedCatchHandler {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], handler_offset: usize) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let size = sleb128::read(src, offset)?;
        let type_addr_pairs: Vec<EncodedTypeAddrPair> =
            try_gread_vec_with!(src, offset, size.abs(), ());
        let mut handlers = type_addr_pairs
            .into_iter()
            .map(|type_addr_pair| CatchHandler {
                exception: ExceptionType::Type(type_addr_pair.type_id),
                addr: type_addr_pair.addr,
            })
            .collect::<Vec<_>>();
        if size <= 0 {
            let all_handler_addr = uleb128::read(src, offset)?;
            handlers.push(CatchHandler {
                exception: ExceptionType::BaseException,
                addr: all_handler_addr,
            });
        }
        Ok((
            Self {
                handlers,
                offset: handler_offset,
            },
            *offset,
        ))
    }
}

impl<'a> TryFromCtx<'a> for EncodedCatchHandlers {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let encoded_handler_size = uleb128::read(src, offset)?;
        let handlers = try_gread_vec_with!(src, offset, encoded_handler_size; ctx = offset);
        Ok((Self { inner: handlers }, *offset))
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
