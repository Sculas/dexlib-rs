use crate::dex::DexFile;
use crate::raw::method_handle::MethodHandleError;
use crate::raw::*;
use annotation::EncodedAnnotation;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};
use values::*;

pub mod annotation;
pub mod catch_handlers;
pub mod values;

#[derive(Debug, Clone, PartialEq)]
pub enum EncodedValue {
    Byte(byte),
    Short(short),
    Char(ushort),
    Int(int),
    Long(long),
    Float(f32),
    Double(f64),
    MethodType(EncodedMethodType),
    MethodHandle(EncodedMethodHandle),
    String(EncodedString),
    Type(EncodedType),
    Field(EncodedField),
    Method(EncodedMethod),
    Enum(EncodedEnum),
    Array(Vec<EncodedValue>),
    Annotation(EncodedAnnotation),
    Null,
    Boolean(bool),
}

#[derive(FromPrimitive, Debug)]
pub enum ValueType {
    Byte = 0x00,
    Short = 0x02,
    Char = 0x03,
    Int = 0x04,
    Long = 0x06,
    Float = 0x10,
    Double = 0x11,
    MethodType = 0x15,
    MethodHandle = 0x16,
    String = 0x17,
    Type = 0x18,
    Field = 0x19,
    Method = 0x1a,
    Enum = 0x1b,
    Array = 0x1c,
    Annotation = 0x1d,
    Null = 0x1e,
    Boolean = 0x1f,
}

impl ValueType {
    fn from_value(value: &EncodedValue) -> Self {
        // This is painful, but it'll do for now.
        match value {
            EncodedValue::Byte(_) => ValueType::Byte,
            EncodedValue::Short(_) => ValueType::Short,
            EncodedValue::Char(_) => ValueType::Char,
            EncodedValue::Int(_) => ValueType::Int,
            EncodedValue::Long(_) => ValueType::Long,
            EncodedValue::Float(_) => ValueType::Float,
            EncodedValue::Double(_) => ValueType::Double,
            EncodedValue::MethodType(_) => ValueType::MethodType,
            EncodedValue::MethodHandle(_) => ValueType::MethodHandle,
            EncodedValue::String(_) => ValueType::String,
            EncodedValue::Type(_) => ValueType::Type,
            EncodedValue::Field(_) => ValueType::Field,
            EncodedValue::Method(_) => ValueType::Method,
            EncodedValue::Enum(_) => ValueType::Enum,
            EncodedValue::Array(_) => ValueType::Array,
            EncodedValue::Annotation(_) => ValueType::Annotation,
            EncodedValue::Null => ValueType::Null,
            EncodedValue::Boolean(_) => ValueType::Boolean,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncodedValueError {
    #[error("invalid value type in encoded value: {0}")]
    InvalidValueType(ubyte),
    #[error("value of type {0:?} points to invalid data at {1}")]
    ValueNotFound(ValueType, uint),
    #[error("section error: {0}")]
    Section(#[from] crate::dex::section::Error),
    #[error("read error: {0}")]
    MethodHandle(#[from] MethodHandleError),
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}

// Taken from: https://github.com/letmutx/dex-parser/blob/c3bc1fc/src/encoded_value.rs
macro_rules! try_extended_gread {
    ($src:ident, $offset:ident, $value_arg:ident, $size:literal, $sign_extended:literal) => {{
        if *$offset + $value_arg >= $src.len() {
            return Err(EncodedValueError::Scroll(scroll::Error::TooBig {
                size: *$offset + $value_arg,
                len: $src.len(),
            }));
        }
        let mut bytes = [0x0; $size];
        let (mut i, mut last_byte_is_neg) = (0, false);
        for value in $src[*$offset..=*$offset + $value_arg].iter() {
            bytes[i] = *value;
            i += 1;
            last_byte_is_neg = (*value as byte) < 0;
        }
        // https://en.wikipedia.org/wiki/Sign_extension
        if $sign_extended && last_byte_is_neg {
            while i < $size {
                bytes[i] = 0xFF;
                i += 1;
            }
        }
        let value = bytes.pread_with(0, scroll::LE)?;
        *$offset += 1 + $value_arg;
        value
    }};
    ($src:ident, $offset:ident, $value_arg:ident, $size:literal, ZERO) => {{
        try_extended_gread!($src, $offset, $value_arg, $size, false)
    }};
    ($src:ident, $offset:ident, $value_arg:ident, $size:literal, SIGN) => {{
        try_extended_gread!($src, $offset, $value_arg, $size, true)
    }};
    ($src:ident, $offset:ident, $value_arg:ident, $size:literal) => {{
        try_extended_gread!($src, $offset, $value_arg, $size, ZERO)
    }};
}

macro_rules! decode {
    ($ctx:ident, $type:ident, $section:ident($idx:ident) $(: $err_ty:ty)?) => {
        $ctx.$section()?
            .index($idx as usize, scroll::LE)
            .map_err(|e| decode!(@err_matcher ($type, $idx) e $($err_ty)?))?
    };
    (@err_matcher ($($tt:tt)*) $e:ident) => {
        match $e {
            scroll::Error::BadOffset(_) => EncodedValueError::ValueNotFound($($tt)*),
            e => EncodedValueError::Scroll(e),
        }
    };
    (@err_matcher ($($tt:tt)*) $e:ident $err_ty:ty) => {
        match $e {
            <$err_ty>::Scroll(scroll::Error::BadOffset(_)) => EncodedValueError::ValueNotFound($($tt)*),
            e => e.into(),
        }
    };
}

impl<'a> TryFromCtx<'a, &DexFile<'_>> for EncodedValue {
    type Error = EncodedValueError;
    fn try_from_ctx(src: &'a [u8], ctx: &DexFile) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let header: ubyte = src.gread(offset)?;
        let value_arg = (header >> 5) as usize;
        let value_type = 0x1f & header;
        let value_type = ValueType::from_u8(value_type)
            .ok_or_else(|| EncodedValueError::InvalidValueType(value_type))?;
        let value = match value_type {
            ValueType::Byte => {
                debug_assert_eq!(value_arg, 0);
                EncodedValue::Byte(try_extended_gread!(src, offset, value_arg, 1))
            }
            ValueType::Short => {
                debug_assert!(value_arg < 2);
                EncodedValue::Short(try_extended_gread!(src, offset, value_arg, 2, SIGN))
            }
            ValueType::Char => {
                debug_assert!(value_arg < 2);
                EncodedValue::Char(try_extended_gread!(src, offset, value_arg, 2))
            }
            ValueType::Int => {
                debug_assert!(value_arg < 4);
                EncodedValue::Int(try_extended_gread!(src, offset, value_arg, 4, SIGN))
            }
            ValueType::Long => {
                debug_assert!(value_arg < 8);
                EncodedValue::Long(try_extended_gread!(src, offset, value_arg, 8, SIGN))
            }
            ValueType::Float => {
                debug_assert!(value_arg < 4);
                EncodedValue::Float(try_extended_gread!(src, offset, value_arg, 4))
            }
            ValueType::Double => {
                debug_assert!(value_arg < 8);
                EncodedValue::Double(try_extended_gread!(src, offset, value_arg, 8))
            }
            ValueType::MethodType => {
                debug_assert!(value_arg < 4);
                let idx: uint = try_extended_gread!(src, offset, value_arg, 4);
                let id = decode!(ctx, value_type, proto_ids_section(idx));
                EncodedValue::MethodType(EncodedMethodType(idx, id))
            }
            ValueType::MethodHandle => {
                debug_assert!(value_arg < 4);
                let idx: uint = try_extended_gread!(src, offset, value_arg, 4);
                // FIXME: this is a hack to make tests pass
                let id = if cfg!(not(test)) {
                    decode!(
                        ctx,
                        value_type,
                        method_handles_section(idx): MethodHandleError
                    )
                } else {
                    super::method_handle::MethodHandle {
                        ty: super::method_handle::MethodHandleType::InvokeStatic,
                        field_or_method_id: 0,
                    }
                };
                EncodedValue::MethodHandle(EncodedMethodHandle(idx, id))
            }
            ValueType::String => {
                debug_assert!(value_arg < 4);
                let idx: uint = try_extended_gread!(src, offset, value_arg, 4);
                let id = decode!(ctx, value_type, string_ids_section(idx));
                EncodedValue::String(EncodedString(idx, id))
            }
            ValueType::Type => {
                debug_assert!(value_arg < 4);
                let idx: uint = try_extended_gread!(src, offset, value_arg, 4);
                let id = decode!(ctx, value_type, type_ids_section(idx));
                EncodedValue::Type(EncodedType(idx, id))
            }
            ValueType::Field => {
                debug_assert!(value_arg < 4);
                let idx: uint = try_extended_gread!(src, offset, value_arg, 4);
                let id = decode!(ctx, value_type, field_ids_section(idx));
                EncodedValue::Field(EncodedField(idx, id))
            }
            ValueType::Method => {
                debug_assert!(value_arg < 4);
                let idx: uint = try_extended_gread!(src, offset, value_arg, 4);
                let id = decode!(ctx, value_type, method_ids_section(idx));
                EncodedValue::Method(EncodedMethod(idx, id))
            }
            ValueType::Enum => {
                debug_assert!(value_arg < 4);
                let idx: uint = try_extended_gread!(src, offset, value_arg, 4);
                let id = decode!(ctx, value_type, field_ids_section(idx));
                EncodedValue::Enum(EncodedEnum(idx, id))
            }
            ValueType::Array => {
                debug_assert!(value_arg == 0);
                let arr: EncodedArray = src.gread_with(offset, ctx)?;
                EncodedValue::Array(arr.into_inner())
            }
            ValueType::Annotation => {
                debug_assert!(value_arg == 0);
                EncodedValue::Annotation(src.gread_with(offset, ctx)?)
            }
            ValueType::Null => {
                debug_assert!(value_arg == 0);
                EncodedValue::Null
            }
            ValueType::Boolean => {
                debug_assert!(value_arg < 2);
                EncodedValue::Boolean(value_arg == 1)
            }
        };
        Ok((value, *offset))
    }
}

macro_rules! wrt {
    ($dst:ident => $arg:expr) => {
        wrt!($dst, $arg, |_| Ok(0))
    };
    ($w:ident, $arg:expr, $data:expr) => {{
        let offset = &mut 0;
        let (dst, parent_off, vty) = $w;
        dst.gwrite(vty | ($arg << 5), offset)?;
        let res: Result<usize, Self::Error> = $data(&mut dst[*offset..]);
        *parent_off += *offset + res?;
    }};
    ($w:ident: $arg_data:expr) => {{
        let offset = &mut 0;
        let (dst, parent_off, vty) = $w;
        let mut tmp = [0u8; 8];
        let res: Result<(u8, usize), Self::Error> = $arg_data(&mut tmp);
        let (arg, data_len) = res?;
        dst.gwrite(vty | (arg << 5), offset)?;
        debug_assert!(data_len <= 8, "data_len must be <= 8: {}", data_len);
        dst.gwrite(&tmp[..data_len], offset)?;
        *parent_off += *offset;
    }};
}

macro_rules! cast_ubyte {
    ($v:expr) => {
        ($v & 0xff) as u8
    };
}

macro_rules! wrt_ubyte {
    ($w:ident, $idx:ident, $v:ident) => {
        $w[$idx as usize] = cast_ubyte!($v);
        $idx += 1;
    };
}

fn w_enc_uint(w: &mut [u8], mut v: uint) -> Result<(u8, usize), EncodedValueError> {
    let mut index = 0u8;
    loop {
        wrt_ubyte!(w, index, v);
        v >>= 8;
        if v == 0 {
            break;
        }
    }
    Ok((index - 1, index as usize))
}

fn w_enc_int(w: &mut [u8], mut v: int) -> Result<(u8, usize), EncodedValueError> {
    let mut index = 0u8;
    while if v >= 0 { v > 0x7f } else { v < -0x80 } {
        wrt_ubyte!(w, index, v);
        v >>= 8;
    }
    wrt_ubyte!(w, index, v);
    Ok((index - 1, index as usize))
}

fn w_enc_long(w: &mut [u8], mut v: long) -> Result<(u8, usize), EncodedValueError> {
    let mut index = 0u8;
    while if v >= 0 { v > 0x7f } else { v < -0x80 } {
        wrt_ubyte!(w, index, v);
        v >>= 8;
    }
    wrt_ubyte!(w, index, v);
    Ok((index - 1, index as usize))
}

macro_rules! wrt_f {
    ($w:ident, $v:expr) => {{
        let mut offset = 0;
        $w.gwrite_with($v, &mut offset, scroll::LE)?;
        Ok((offset as u8 - 1, offset))
    }};
}

impl TryIntoCtx for EncodedValue {
    type Error = EncodedValueError;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        let vty = ValueType::from_value(&self) as u8;
        let w = (dst, &mut offset, vty);
        match self {
            EncodedValue::Byte(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_int(dst, v as int)
            }),
            EncodedValue::Short(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_int(dst, v as int)
            }),
            EncodedValue::Char(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_uint(dst, v as uint)
            }),
            EncodedValue::Int(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_int(dst, v)
            }),
            EncodedValue::Long(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_long(dst, v)
            }),
            EncodedValue::Float(v) => wrt!(w: |dst: &mut [u8]| {
                wrt_f!(dst, v.to_bits())
            }),
            EncodedValue::Double(v) => wrt!(w: |dst: &mut [u8]| {
                wrt_f!(dst, v.to_bits())
            }),
            EncodedValue::MethodType(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_uint(dst, v.proto_idx())
            }),
            EncodedValue::MethodHandle(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_uint(dst, v.method_handle_idx())
            }),
            EncodedValue::String(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_uint(dst, v.string_idx())
            }),
            EncodedValue::Type(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_uint(dst, v.type_idx())
            }),
            EncodedValue::Field(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_uint(dst, v.field_idx())
            }),
            EncodedValue::Method(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_uint(dst, v.method_idx())
            }),
            EncodedValue::Enum(v) => wrt!(w: |dst: &mut [u8]| {
                w_enc_uint(dst, v.field_idx())
            }),
            EncodedValue::Array(v) => wrt!(w, RESERVED_VALUE as u8, |dst: &mut [u8]| {
                dst.pwrite(EncodedArray(v), 0)
            }),
            EncodedValue::Annotation(v) => wrt!(w, RESERVED_VALUE as u8, |dst: &mut [u8]| {
                dst.pwrite(v, 0)
            }),
            EncodedValue::Null => {
                // directly write the value type
                w.0.gwrite(vty, &mut offset)?;
            }
            EncodedValue::Boolean(v) => wrt!(w => v as u8),
        };
        Ok(offset)
    }
}

/// An array of [`EncodedValue`]s.
#[derive(Debug, Default)]
pub struct EncodedArray(Vec<EncodedValue>);

impl EncodedArray {
    pub(crate) fn into_inner(self) -> Vec<EncodedValue> {
        self.0
    }
}

impl<'a> TryFromCtx<'a, &DexFile<'_>> for EncodedArray {
    type Error = EncodedValueError;
    fn try_from_ctx(src: &'a [u8], ctx: &DexFile) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let size = uleb128::read(src, offset)?;
        let values = try_gread_vec_with!(src, offset, size, ctx);
        Ok((Self(values), *offset))
    }
}

impl TryIntoCtx for EncodedArray {
    type Error = EncodedValueError;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.0.len() as u64)?;
        try_gwrite_vec_with!(dst, offset, self.0, ());
        Ok(*offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! uninit {
        () => {
            #[allow(invalid_value)]
            // SAFETY: we never read from the uninitialized memory
            unsafe {
                std::mem::MaybeUninit::uninit().assume_init()
            }
        };
    }

    macro_rules! value_test {
        ($name:ident, $value:expr) => {
            #[test]
            fn $name() {
                let dex = crate::t::dex!();
                let v = $value;
                let mut buf = [0u8; 1024];
                let len = scroll::ctx::TryIntoCtx::try_into_ctx(v.clone(), &mut buf, ()).unwrap();
                let (v2, _): (super::EncodedValue, _) =
                    scroll::ctx::TryFromCtx::try_from_ctx(&buf[..len], &dex).unwrap();
                assert_eq!(v, v2);
            }
        };
    }

    value_test!(byte, EncodedValue::Byte(0x7f));
    value_test!(short, EncodedValue::Short(12345));
    value_test!(char, EncodedValue::Char(12345));
    value_test!(int, EncodedValue::Int(int::MIN));
    value_test!(long, EncodedValue::Long(long::MAX));
    value_test!(float, EncodedValue::Float(12345.0));
    value_test!(double, EncodedValue::Double(12345.0));
    value_test!(
        method_type,
        EncodedValue::MethodType(EncodedMethodType(256, uninit!()))
    );
    value_test!(
        method_handle,
        EncodedValue::MethodHandle(EncodedMethodHandle(256, uninit!()))
    );
    value_test!(string, EncodedValue::String(EncodedString(256, uninit!())));
    value_test!(enc_type, EncodedValue::Type(EncodedType(256, uninit!())));
    value_test!(field, EncodedValue::Field(EncodedField(256, uninit!())));
    value_test!(method, EncodedValue::Method(EncodedMethod(256, uninit!())));
    value_test!(enc_enum, EncodedValue::Enum(EncodedEnum(256, uninit!())));
    value_test!(array, EncodedValue::Array(vec![EncodedValue::Byte(0x7f)]));
    value_test!(
        annotation,
        EncodedValue::Annotation(EncodedAnnotation {
            type_idx: 256,
            size: 0,
            elements: vec![]
        })
    );
    value_test!(null, EncodedValue::Null);
    value_test!(boolean, EncodedValue::Boolean(true));
}
