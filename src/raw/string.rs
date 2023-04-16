use crate::raw::*;
use derivative::Derivative;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct StringId(/* offset into data section */ uint);

impl StringId {
    pub fn offset(&self) -> uint {
        self.0
    }
}

impl From<StringId> for uint {
    fn from(id: StringId) -> Self {
        id.0
    }
}

impl<'a> TryFromCtx<'a, scroll::Endian> for StringId {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let str_data_off: uint = src.gread_with(offset, ctx)?;
        Ok((Self(str_data_off), *offset))
    }
}

impl TryIntoCtx<scroll::Endian> for StringId {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.0, offset, ctx)?;
        Ok(*offset)
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct StringData<'a> {
    /// Size of this string in UTF-16 code units (which is the "string length" in many systems).
    /// That is, this is the decoded length of the string.
    /// The encoded length is implied by the position of the 0 byte.
    pub size: ulong,
    /// A series of MUTF-8 code units (a.k.a. octets, a.k.a. bytes) followed by a byte of value 0.
    /// See ["MUTF-8 (Modified UTF-8) Encoding"][1] for details and discussion about the data format.
    ///
    /// [1]: https://source.android.com/docs/core/runtime/dex-format#mutf-8
    #[derivative(Debug = "ignore")]
    pub data: &'a [ubyte],
}

impl<'a> TryFromCtx<'a, scroll::Endian> for StringData<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], _ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        // this is the decoded/actual length of the string
        // NOT the encoded length!
        let size = uleb128::read(src, offset)?;
        // for that, we read until we find a 0 byte
        let encoded_len = count_delim!(src, offset, b'\0');
        let data = src.gread_with::<&[ubyte]>(offset, encoded_len)?;
        Ok((Self { size, data }, *offset))
    }
}

impl<'a> TryIntoCtx<scroll::Endian> for StringData<'a> {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], _ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.size)?;
        dst.gwrite(self.data, offset)?;
        Ok(*offset)
    }
}
