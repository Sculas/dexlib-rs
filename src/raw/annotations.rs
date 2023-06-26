use crate::raw::encoded_value::{EncodedAnnotation, EncodedValueError};
use crate::raw::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

#[derive(Debug, thiserror::Error)]
pub enum AnnotationError {
    #[error("invalid visibility byte in annotation: {0}")]
    InvalidVisibility(ubyte),
    #[error("error reading encoded annotation: {0}")]
    EncodedValue(#[from] EncodedValueError),
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}

/// See https://source.android.com/docs/core/runtime/dex-format#visibility
#[derive(FromPrimitive, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)] // ubyte
pub enum Visibility {
    Build = 0x00,
    Runtime = 0x01,
    System = 0x02,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub visibility: Visibility,
    pub annotation: EncodedAnnotation,
}

impl<'a> TryFromCtx<'a, scroll::Endian> for Annotation {
    type Error = AnnotationError;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let visibility_byte = src.gread_with(offset, ctx)?;
        let visibility = Visibility::from_u8(visibility_byte)
            .ok_or_else(|| AnnotationError::InvalidVisibility(visibility_byte))?;
        let annotation = src.gread(offset)?;
        Ok((
            Self {
                visibility,
                annotation,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx<scroll::Endian> for Annotation {
    type Error = AnnotationError;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.visibility as ubyte, offset, ctx)?;
        dst.gwrite(self.annotation, offset)?;
        Ok(*offset)
    }
}

#[derive(Debug, Clone)]
pub struct AnnotationsDirectory {
    /// Offset from the start of the file to the annotations made directly on the class, or 0 if the class has no direct annotations.
    /// The offset, if non-zero, should be to a location in the data section.
    /// The format of the data is specified by `annotation_set_item`.
    pub class_annotations_off: uint,
    /// List of associated field annotations.
    /// The elements of the list must be sorted in increasing order, by `field_idx`.
    pub field_annotations: Vec<FieldAnnotation>,
    /// List of associated method annotations.
    /// The elements of the list must be sorted in increasing order, by `method_idx`.
    pub method_annotations: Vec<MethodAnnotation>,
    /// List of associated method parameter annotations.
    /// The elements of the list must be sorted in increasing order, by `method_idx`.
    pub parameter_annotations: Vec<ParameterAnnotation>,
}

impl<'a> TryFromCtx<'a, scroll::Endian> for AnnotationsDirectory {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let class_annotations_off: uint = src.gread_with(offset, ctx)?;
        let fields_size: uint = src.gread_with(offset, ctx)?;
        let methods_size: uint = src.gread_with(offset, ctx)?;
        let parameters_size: uint = src.gread_with(offset, ctx)?;
        let field_annotations = try_gread_vec_with!(src, offset, fields_size, ctx);
        let method_annotations = try_gread_vec_with!(src, offset, methods_size, ctx);
        let parameter_annotations = try_gread_vec_with!(src, offset, parameters_size, ctx);
        Ok((
            Self {
                class_annotations_off,
                field_annotations,
                method_annotations,
                parameter_annotations,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx<scroll::Endian> for AnnotationsDirectory {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.class_annotations_off, offset, ctx)?;
        dst.gwrite_with(self.field_annotations.len() as uint, offset, ctx)?;
        dst.gwrite_with(self.method_annotations.len() as uint, offset, ctx)?;
        dst.gwrite_with(self.parameter_annotations.len() as uint, offset, ctx)?;
        try_gwrite_vec_with!(dst, offset, self.field_annotations, ctx);
        try_gwrite_vec_with!(dst, offset, self.method_annotations, ctx);
        try_gwrite_vec_with!(dst, offset, self.parameter_annotations, ctx);
        Ok(*offset)
    }
}

#[derive(Debug, Clone, Copy, Pread, Pwrite)]
pub struct FieldAnnotation {
    pub field_idx: uint,
    pub annotations_off: uint,
}

#[derive(Debug, Clone, Copy, Pread, Pwrite)]
pub struct MethodAnnotation {
    pub method_idx: uint,
    pub annotations_off: uint,
}

#[derive(Debug, Clone, Copy, Pread, Pwrite)]
pub struct ParameterAnnotation {
    pub method_idx: uint,
    pub annotations_off: uint,
}

#[derive(Debug, Default)]
pub struct AnnotationSetRefList(Vec<uint>); // inlined item into list

impl AnnotationSetRefList {
    pub(crate) fn into_inner(self) -> Vec<uint> {
        self.0
    }
}

impl<'a> TryFromCtx<'a, scroll::Endian> for AnnotationSetRefList {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let size: uint = src.gread_with(offset, ctx)?;
        let items = try_gread_vec_with!(src, offset, size, ctx);
        Ok((Self(items), *offset))
    }
}

impl TryIntoCtx<scroll::Endian> for AnnotationSetRefList {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.0.len() as uint, offset, ctx)?;
        try_gwrite_vec_with!(dst, offset, self.0, ctx);
        Ok(*offset)
    }
}

#[derive(Debug, Default)]
pub struct AnnotationSetItem(Vec<uint>); // inlined offsets into item

impl AnnotationSetItem {
    pub(crate) fn into_inner(self) -> Vec<uint> {
        self.0
    }
}

impl<'a> TryFromCtx<'a, scroll::Endian> for AnnotationSetItem {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let size: uint = src.gread_with(offset, ctx)?;
        let items = try_gread_vec_with!(src, offset, size, ctx);
        Ok((Self(items), *offset))
    }
}

impl TryIntoCtx<scroll::Endian> for AnnotationSetItem {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.0.len() as uint, offset, ctx)?;
        try_gwrite_vec_with!(dst, offset, self.0, ctx);
        Ok(*offset)
    }
}
