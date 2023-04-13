use crate::{raw::*, utils::leb128::Uleb128};
use derivative::Derivative;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

#[derive(Debug, thiserror::Error)]
pub enum StringError {
    #[error("invalid size: {0}")]
    InvalidSize(usize),
    #[error("read error: {0}")]
    ScrollError(#[from] scroll::Error),
}

#[derive(Debug)]
pub struct StringId {
    pub offset: uint,
}

impl<'a> TryFromCtx<'a, scroll::Endian> for StringId {
    type Error = StringError;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let str_data_off: uint = src.gread_with(offset, ctx)?;
        Ok((
            Self {
                offset: str_data_off,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx<scroll::Endian> for StringId {
    type Error = StringError;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.offset, offset, ctx)?;
        Ok(*offset)
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct StringData<'a> {
    /// Size of this string in UTF-16 code units (which is the "string length" in many systems).
    /// That is, this is the decoded length of the string.
    /// The encoded length is implied by the position of the 0 byte.
    pub size: u64,
    /// a series of MUTF-8 code units (a.k.a. octets, a.k.a. bytes) followed by a byte of value 0.
    /// See ["MUTF-8 (Modified UTF-8) Encoding"](encoding) for details and discussion about the data format.
    ///
    /// [encoding]: https://source.android.com/docs/core/runtime/dex-format#mutf-8
    #[derivative(Debug = "ignore")]
    pub data: &'a [ubyte],
}

impl<'a> TryFromCtx<'a, scroll::Endian> for StringData<'a> {
    type Error = StringError;
    fn try_from_ctx(src: &'a [u8], _ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        // this is the decoded/actual length of the string
        // NOT the encoded length!
        let size = Uleb128::read(src, offset)?;
        // for that, we read until we find a 0 byte
        let encoded_len = count_delim!(src, offset, b'\0');
        let data = src.gread_with::<&[ubyte]>(offset, encoded_len)?;
        Ok((Self { size, data }, *offset))
    }
}

impl<'a> TryIntoCtx<scroll::Endian> for StringData<'a> {
    type Error = StringError;
    fn try_into_ctx(self, dst: &mut [u8], _ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        Uleb128::write(dst, offset, self.size)?;
        dst.gwrite(self.data, offset)?;
        Ok(*offset)
    }
}