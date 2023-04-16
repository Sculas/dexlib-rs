use crate::raw::*;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use super::{EncodedValue, EncodedValueError};

#[derive(Debug, Clone, PartialEq)]
pub struct EncodedAnnotation {
    pub type_idx: ulong,
    pub size: ulong,
    pub elements: Vec<AnnotationElement>,
}

impl<'a> TryFromCtx<'a> for EncodedAnnotation {
    type Error = EncodedValueError;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let type_idx = uleb128::read(src, offset)?;
        let size = uleb128::read(src, offset)?;
        let elements = try_gread_vec_with!(src, offset, size, ());
        Ok((
            Self {
                type_idx,
                size,
                elements,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx for EncodedAnnotation {
    type Error = EncodedValueError;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.type_idx)?;
        uleb128::write(dst, offset, self.size)?;
        try_gwrite_vec_with!(dst, offset, self.elements, ());
        Ok(*offset)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationElement {
    pub name_idx: ulong,
    pub value: EncodedValue,
}

impl<'a> TryFromCtx<'a> for AnnotationElement {
    type Error = EncodedValueError;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let name_idx = uleb128::read(src, offset)?;
        let value = src.gread(offset)?;
        Ok((Self { name_idx, value }, *offset))
    }
}

impl TryIntoCtx for AnnotationElement {
    type Error = EncodedValueError;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.name_idx)?;
        dst.gwrite_with(self.value, offset, ())?;
        Ok(*offset)
    }
}
