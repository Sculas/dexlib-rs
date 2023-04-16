use crate::raw::*;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use super::flags::AccessFlags;

#[derive(Debug, thiserror::Error)]
pub enum ClassDataError {
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}

#[derive(Debug)]
pub struct ClassData {
    /// The number of static fields defined in this item.
    pub static_fields_size: ulong,
    /// The number of instance fields defined in this item.
    pub instance_fields_size: ulong,
    /// The number of direct methods defined in this item.
    pub direct_methods_size: ulong,
    /// The number of virtual methods defined in this item.
    pub virtual_methods_size: ulong,
    /// The defined static fields, represented as a sequence of encoded elements.
    /// The fields must be sorted by `field_idx` in increasing order.
    pub static_fields: Vec<EncodedField>,
    /// The defined instance fields, represented as a sequence of encoded elements.
    /// The fields must be sorted by `field_idx` in increasing order.
    pub instance_fields: Vec<EncodedField>,
    /// The defined direct (any of static, private, or constructor) methods, represented as a sequence of encoded elements.
    /// The methods must be sorted by `method_idx` in increasing order.
    pub direct_methods: Vec<EncodedMethod>,
    /// The defined virtual (none of static, private, or constructor) methods, represented as a sequence of encoded elements.
    /// This list should not include inherited methods unless overridden by the class that this item represents.
    /// The methods must be sorted by `method_idx` in increasing order.
    /// The `method_idx` of a virtual method must not be the same as any direct method.
    pub virtual_methods: Vec<EncodedMethod>,
}

impl<'a> TryFromCtx<'a> for ClassData {
    type Error = ClassDataError;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let static_fields_size = uleb128::read(src, offset)?;
        let instance_fields_size = uleb128::read(src, offset)?;
        let direct_methods_size = uleb128::read(src, offset)?;
        let virtual_methods_size = uleb128::read(src, offset)?;
        let static_fields = try_gread_vec_with!(src, offset, static_fields_size, ());
        let instance_fields = try_gread_vec_with!(src, offset, instance_fields_size, ());
        let direct_methods = try_gread_vec_with!(src, offset, direct_methods_size, ());
        let virtual_methods = try_gread_vec_with!(src, offset, virtual_methods_size, ());
        Ok((
            Self {
                static_fields_size,
                instance_fields_size,
                direct_methods_size,
                virtual_methods_size,
                static_fields,
                instance_fields,
                direct_methods,
                virtual_methods,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx for ClassData {
    type Error = ClassDataError;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.static_fields_size)?;
        uleb128::write(dst, offset, self.instance_fields_size)?;
        uleb128::write(dst, offset, self.direct_methods_size)?;
        uleb128::write(dst, offset, self.virtual_methods_size)?;
        try_gwrite_vec_with!(dst, offset, self.static_fields, ());
        try_gwrite_vec_with!(dst, offset, self.instance_fields, ());
        try_gwrite_vec_with!(dst, offset, self.direct_methods, ());
        try_gwrite_vec_with!(dst, offset, self.virtual_methods, ());
        Ok(*offset)
    }
}

#[derive(Debug)]
pub struct EncodedField {
    /// Index into the `field_ids` list for the identity of this field (includes the name and descriptor),
    /// represented as a difference from the index of previous element in the list.
    /// The index of the first element in a list is represented directly.
    pub field_idx_diff: ulong,
    /// Access flags for the field (`public`, `final`, etc.).
    /// See [`AccessFlags`] for details.
    pub access_flags: AccessFlags,
}

impl<'a> TryFromCtx<'a> for EncodedField {
    type Error = ClassDataError;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let field_idx_diff = uleb128::read(src, offset)?;
        let access_flags = AccessFlags::try_from_uleb128(src, offset)?;
        Ok((
            Self {
                field_idx_diff,
                access_flags,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx for EncodedField {
    type Error = ClassDataError;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.field_idx_diff)?;
        self.access_flags.try_into_uleb128(dst, offset)?;
        Ok(*offset)
    }
}

#[derive(Debug)]
pub struct EncodedMethod {
    /// Index into the `method_ids` list for the identity of this field (includes the name and descriptor),
    /// represented as a difference from the index of previous element in the list.
    /// The index of the first element in a list is represented directly.
    pub method_idx_diff: ulong,
    /// Access flags for the method (`public`, `final`, etc.).
    /// See [`AccessFlags`] for details.
    pub access_flags: AccessFlags,
    /// Offset from the start of the file to the code structure for this method,
    /// or 0 if this method is either `abstract` or `native`.
    /// The offset should be to a location in the `data` section.
    pub code_off: ulong,
}

impl<'a> TryFromCtx<'a> for EncodedMethod {
    type Error = ClassDataError;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let method_idx_diff = uleb128::read(src, offset)?;
        let access_flags = AccessFlags::try_from_uleb128(src, offset)?;
        let code_off = uleb128::read(src, offset)?;
        Ok((
            Self {
                method_idx_diff,
                access_flags,
                code_off,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx for EncodedMethod {
    type Error = ClassDataError;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.method_idx_diff)?;
        self.access_flags.try_into_uleb128(dst, offset)?;
        uleb128::write(dst, offset, self.code_off)?;
        Ok(*offset)
    }
}
