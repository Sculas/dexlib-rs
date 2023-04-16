#![allow(non_camel_case_types, dead_code)] // TODO: remove dead_code

pub mod classdef;
pub mod encoded_value;
pub mod flags;
pub mod header;
pub mod map_list;
pub mod method_handle;
/// Simple, small types that don't need their own module.
pub mod simple;
pub mod string;

pub(crate) type byte = i8;
pub(crate) type ubyte = u8;
/// little endian
pub(crate) type short = i16;
/// little endian
pub(crate) type ushort = u16;
/// little endian
pub(crate) type int = i32;
/// little endian
pub(crate) type uint = u32;
/// little endian
pub(crate) type long = i64;
/// little endian
pub(crate) type ulong = u64;
pub(crate) type sleb128 = crate::utils::leb128::Sleb128;
pub(crate) type uleb128 = crate::utils::leb128::Uleb128;

pub(crate) const NO_INDEX: uint = 0xffffffff;
pub(crate) const RESERVED_VALUE: usize = 0;

pub(crate) type RawStringIndex = uint;
pub(crate) type RawTypeIndex = uint;
pub(crate) type RawProtoIndex = uint;
pub(crate) type RawFieldIndex = uint;
pub(crate) type RawMethodIndex = uint;
pub(crate) type RawMethodHandleIndex = uint;

pub mod tysize {
    pub const STRING_ID: usize = 0x04;
    pub const TYPE_ID: usize = 0x04;
    pub const PROTO_ID: usize = 0x0c;
    pub const FIELD_ID: usize = 0x08;
    pub const METHOD_ID: usize = 0x08;
    pub const CLASS_DEF: usize = 0x20;
    pub const CALL_SITE_ID: usize = 0x04;
    pub const METHOD_HANDLE: usize = 0x08;

    #[cfg(debug_assertions)]
    use super::*;
    #[cfg(debug_assertions)]
    assert_sz!(
        STRING_ID; string::StringId
        TYPE_ID; simple::TypeId
        PROTO_ID; simple::ProtoId
        FIELD_ID; simple::FieldId
        METHOD_ID; simple::MethodId
        CLASS_DEF; classdef::ClassDef
        CALL_SITE_ID; simple::CallSiteId
    );
}
