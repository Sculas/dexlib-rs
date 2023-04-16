use crate::raw::uleb128;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

bitflags::bitflags! {
  /// For more information, click [here][1].
  ///
  /// [1]: https://source.android.com/docs/core/runtime/dex-format#access-flags
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct AccessFlags: u32 {
      const Public = 0x1;
      const Private = 0x2;
      const Protected = 0x4;
      const Static = 0x8;
      const Final = 0x10;
      const Synchronized = 0x20;
      const Volatile = 0x40;
      const Bridge = 0x40;
      const Transient = 0x80;
      const Varargs = 0x80;
      const Native = 0x100;
      const Interface = 0x200;
      const Abstract = 0x400;
      const Strict = 0x800;
      const Synthetic = 0x1000;
      const Annotation = 0x2000;
      const Enum = 0x4000;
      const Constructor = 0x10000;
      const DeclaredSynchronized = 0x20000;
  }
}

impl AccessFlags {
    pub fn try_from_uleb128(src: &[u8], offset: &mut usize) -> Result<Self, scroll::Error> {
        let flags = uleb128::read(src, offset)?;
        Ok(AccessFlags::from_bits_truncate(flags as u32))
    }

    pub fn try_into_uleb128(
        &self,
        dst: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), scroll::Error> {
        uleb128::write(dst, offset, self.bits() as u64)?;
        Ok(())
    }
}

impl<'a> TryFromCtx<'a, scroll::Endian> for AccessFlags {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let flags = src.gread_with(offset, ctx)?;
        Ok((AccessFlags::from_bits_truncate(flags), *offset))
    }
}

impl<'a> TryIntoCtx<scroll::Endian> for &'a AccessFlags {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.bits(), offset, ctx)?;
        Ok(*offset)
    }
}

impl TryIntoCtx<scroll::Endian> for AccessFlags {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        (&self).try_into_ctx(dst, ctx)
    }
}
