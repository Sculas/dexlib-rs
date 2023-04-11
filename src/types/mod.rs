#![allow(non_camel_case_types, dead_code)] // TODO: remove dead_code

pub mod header;

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
