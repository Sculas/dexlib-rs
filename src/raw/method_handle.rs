use crate::raw::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use scroll::{Pread, Pwrite};

#[derive(Debug, thiserror::Error)]
pub enum MethodHandleError {
    #[error("invalid handle type in method handle: {0}")]
    InvalidType(u16),
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}

#[derive(Debug, Clone, Copy)]
pub struct MethodHandle {
    pub ty: MethodHandleType,
    pub field_or_method_id: ushort,
}

impl MethodHandle {
    pub fn is_accessor(&self) -> bool {
        matches!(
            self.ty,
            MethodHandleType::StaticPut
                | MethodHandleType::StaticGet
                | MethodHandleType::InstancePut
                | MethodHandleType::InstanceGet
        )
    }
}

impl<'a> scroll::ctx::TryFromCtx<'a, scroll::Endian> for MethodHandle {
    type Error = MethodHandleError;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let ty: ushort = src.gread_with(offset, ctx)?;
        let ty =
            MethodHandleType::from_u16(ty).ok_or_else(|| MethodHandleError::InvalidType(ty))?;
        let __unused: ushort = src.gread_with(offset, ctx)?;
        debug_assert_eq!(__unused, RESERVED_VALUE as ushort);
        let field_or_method_id: ushort = src.gread_with(offset, ctx)?;
        let __unused: ushort = src.gread_with(offset, ctx)?;
        debug_assert_eq!(__unused, RESERVED_VALUE as ushort);
        Ok((
            Self {
                ty,
                field_or_method_id,
            },
            *offset,
        ))
    }
}

impl scroll::ctx::TryIntoCtx<scroll::Endian> for MethodHandle {
    type Error = MethodHandleError;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.ty as ushort, offset, ctx)?;
        dst.gwrite_with(RESERVED_VALUE as ushort, offset, ctx)?;
        dst.gwrite_with(self.field_or_method_id, offset, ctx)?;
        dst.gwrite_with(RESERVED_VALUE as ushort, offset, ctx)?;
        Ok(*offset)
    }
}

#[derive(FromPrimitive, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u16)] // ushort
pub enum MethodHandleType {
    /// Method handle is a static field setter (accessor)
    StaticPut = 0x00,
    /// Method handle is a static field getter (accessor)
    StaticGet = 0x01,
    /// Method handle is an instance field setter (accessor)
    InstancePut = 0x02,
    /// Method handle is an instance field getter (accessor)
    InstanceGet = 0x03,
    /// Method handle is a static method invoker
    InvokeStatic = 0x04,
    /// Method handle is an instance method invoker
    InvokeInstance = 0x05,
    /// Method handle is a constructor method invoker
    InvokeConstructor = 0x06,
    /// Method handle is a direct method invoker
    InvokeDirect = 0x07,
    /// Method handle is an interface method invoker
    InvokeInterface = 0x08,
}
