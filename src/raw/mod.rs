#![allow(non_camel_case_types, dead_code)] // TODO: remove dead_code

pub mod header;
pub mod map_list;
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
pub(crate) type sleb128 = scroll::Sleb128;
pub(crate) type uleb128 = scroll::Uleb128;

pub(crate) const NO_INDEX: uint = 0xffffffff;

pub mod tysize {
    pub const STRING_ID: usize = 0x04;
    pub const TYPE_ID: usize = 0x04;
    pub const PROTO_ID: usize = 0x0c;
    pub const FIELD_ID: usize = 0x08;
    pub const METHOD_ID: usize = 0x08;
    pub const CLASS_DEF: usize = 0x20;
    pub const CALL_SITE_ID: usize = 0x04;
    pub const METHOD_HANDLE: usize = 0x08;
}
