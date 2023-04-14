use crate::raw::*;
use adler32::adler32;
use derivative::Derivative;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

const MAGIC_LEN: usize = 8;
const SIG_LEN: usize = 20;
const ENDIAN_CONSTANT: uint = 0x12345678;

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error("invalid magic: {0}")]
    InvalidMagic(#[from] VersionError),
    #[error("invalid endian tag {0:#x}, only LE is supported for now")]
    InvalidEndianTag(u32),
    #[error("invalid checksum")]
    InvalidChecksum,
    #[error("read error: {0}")]
    ScrollError(#[from] scroll::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct Header<'a> {
    /// The version of the dex file.
    /// [`Version`] is syntactic sugar for the `magic` value which would normally be here.
    /// Any future reference to `magic` refers to this field.
    pub version: Version,
    /// Adler32 checksum of the rest of the file (everything but `magic` and this field).
    pub checksum: uint,
    /// SHA-1 signature (hash) of the rest of the file (everything but `magic`, `checksum`, and this field).
    #[derivative(Debug = "ignore")]
    pub signature: &'a [ubyte],
    /// Size of the entire file (including the header), in bytes.
    pub file_size: uint,
    /// Size of the header (this entire section), in bytes.
    pub header_size: uint,
    /// Specifies the endianness of the dex file.
    /// Currently, only little-endian is supported.
    /// Click [here](endian-constant) for more information.
    ///
    /// [endian-constant]: https://source.android.com/devices/tech/dalvik/dex-format#endian-constant
    pub endian_tag: uint,
    /// Size of the link section, or 0 if this file isn't statically linked.
    pub link_size: uint,
    /// Offset from the start of the file to the link section, or 0 if `link_size == 0`.
    /// The offset, if non-zero, should be to an offset into the `link_data` section.
    /// The format of the data pointed at is left unspecified by this document;
    /// this header field (and the previous) are left as hooks for use by runtime implementations.
    pub link_off: uint,
    /// Offset from the start of the file to the map item.
    /// The offset, which must be non-zero, should be to an offset into the `data` section,
    /// and the data should be in the format specified by "map_list" below.
    ///
    /// Implemented as [`MapList`][super::map_list::MapList].
    pub map_off: uint,
    /// Count of strings in the string identifiers list.
    ///
    /// Implemented in the [`string`][super::string] module.
    pub string_ids_size: uint,
    /// Offset from the start of the file to the string identifiers list,
    /// or 0 if `string_ids_size == 0` (admittedly a strange edge case).
    /// The offset, if non-zero, should be to the start of the `string_ids` section.
    ///
    /// Implemented in the [`string`][super::string] module.
    pub string_ids_off: uint,
    /// Count of elements in the type identifiers list, at most 65535.
    pub type_ids_size: uint,
    /// Offset from the start of the file to the type identifiers list,
    /// or 0 if `type_ids_size == 0` (admittedly a strange edge case).
    /// The offset, if non-zero, should be to the start of the `type_ids` section.
    ///
    /// Implemented as [`TypeId`][super::simple::TypeId].
    pub type_ids_off: uint,
    /// Count of elements in the prototype identifiers list, at most 65535.
    pub proto_ids_size: uint,
    /// Offset from the start of the file to the prototype identifiers list,
    /// or 0 if `proto_ids_size == 0` (admittedly a strange edge case).
    /// The offset, if non-zero, should be to the start of the `proto_ids` section.
    ///
    /// Implemented as [`ProtoId`][super::simple::ProtoId].
    pub proto_ids_off: uint,
    /// Count of elements in the field identifiers list.
    pub field_ids_size: uint,
    /// Offset from the start of the file to the field identifiers list,
    /// or 0 if `field_ids_size == 0`.
    /// The offset, if non-zero, should be to the start of the `field_ids` section.
    ///
    /// Implemented as [`FieldId`][super::simple::FieldId].
    pub field_ids_off: uint,
    /// Count of elements in the method identifiers list (undocumented: at most 65535, requires multidex).
    pub method_ids_size: uint,
    /// Offset from the start of the file to the method identifiers list,
    /// or 0 if `method_ids_size == 0`.
    /// The offset, if non-zero, should be to the start of the `method_ids` section.
    ///
    /// Implemented as [`MethodId`][super::simple::MethodId].
    pub method_ids_off: uint,
    /// Count of elements in the class definitions list.
    pub class_defs_size: uint,
    /// Offset from the start of the file to the class definitions list,
    /// or 0 if `class_defs_size == 0` (admittedly a strange edge case).
    /// The offset, if non-zero, should be to the start of the `class_defs` section.
    ///
    /// TODO: Implement this.
    pub class_defs_off: uint,
    /// Size of `data` section in bytes. Must be an even multiple of [`sizeof(uint)`][std::mem::size_of].
    pub data_size: uint,
    /// Offset from the start of the file to the start of the `data` section.
    pub data_off: uint,
}

impl<'a> Header<'a> {
    pub fn data_section(&self) -> std::ops::Range<uint> {
        self.data_off..self.data_off + self.data_size
    }

    pub fn in_data_section(&self, offset: uint) -> bool {
        self.data_section().contains(&offset)
    }
}

impl<'a> Clone for Header<'a> {
    /// The [`Clone`] implementation for [`Header`] is a shallow clone,
    /// and only clones offsets and sizes.
    fn clone(&self) -> Self {
        Self {
            link_size: self.link_size,
            link_off: self.link_off,
            map_off: self.map_off,
            string_ids_size: self.string_ids_size,
            string_ids_off: self.string_ids_off,
            type_ids_size: self.type_ids_size,
            type_ids_off: self.type_ids_off,
            proto_ids_size: self.proto_ids_size,
            proto_ids_off: self.proto_ids_off,
            field_ids_size: self.field_ids_size,
            field_ids_off: self.field_ids_off,
            method_ids_size: self.method_ids_size,
            method_ids_off: self.method_ids_off,
            class_defs_size: self.class_defs_size,
            class_defs_off: self.class_defs_off,
            data_size: self.data_size,
            data_off: self.data_off,
            ..Default::default()
        }
    }
}

impl<'a> TryFromCtx<'a, scroll::Endian> for Header<'a> {
    type Error = HeaderError;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let version = src.gread_with::<&[ubyte]>(offset, MAGIC_LEN)?.try_into()?;
        let checksum = src.gread_with(offset, ctx)?;
        if checksum != adler32(&src[*offset..])? {
            return Err(HeaderError::InvalidChecksum);
        }

        let signature = src.gread_with::<&[ubyte]>(offset, SIG_LEN)?;
        let file_size = src.gread_with(offset, ctx)?;
        let header_size = src.gread_with(offset, ctx)?;

        let endian_tag = src.gread_with(offset, ctx)?;
        match endian_tag {
            ENDIAN_CONSTANT => {}                                 // LE, ok
            _ => Err(HeaderError::InvalidEndianTag(endian_tag))?, // BE or unknown, not ok
        }

        let link_size = src.gread_with(offset, ctx)?;
        let link_off = src.gread_with(offset, ctx)?;
        let map_off = src.gread_with(offset, ctx)?;
        let string_ids_size = src.gread_with(offset, ctx)?;
        let string_ids_off = src.gread_with(offset, ctx)?;
        let type_ids_size = src.gread_with(offset, ctx)?;
        let type_ids_off = src.gread_with(offset, ctx)?;
        let proto_ids_size = src.gread_with(offset, ctx)?;
        let proto_ids_off = src.gread_with(offset, ctx)?;
        let field_ids_size = src.gread_with(offset, ctx)?;
        let field_ids_off = src.gread_with(offset, ctx)?;
        let method_ids_size = src.gread_with(offset, ctx)?;
        let method_ids_off = src.gread_with(offset, ctx)?;
        let class_defs_size = src.gread_with(offset, ctx)?;
        let class_defs_off = src.gread_with(offset, ctx)?;
        let data_size = src.gread_with(offset, ctx)?;
        let data_off = src.gread_with(offset, ctx)?;

        Ok((
            Header {
                version,
                checksum,
                signature,
                file_size,
                header_size,
                endian_tag,
                link_size,
                link_off,
                map_off,
                string_ids_size,
                string_ids_off,
                type_ids_size,
                type_ids_off,
                proto_ids_size,
                proto_ids_off,
                field_ids_size,
                field_ids_off,
                method_ids_size,
                method_ids_off,
                class_defs_size,
                class_defs_off,
                data_size,
                data_off,
            },
            *offset,
        ))
    }
}

impl<'a> TryIntoCtx<scroll::Endian> for Header<'a> {
    type Error = HeaderError;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.version, offset, ctx)?;
        dst.gwrite_with(self.checksum, offset, ctx)?;
        dst.gwrite_with(self.signature, offset, ())?;
        dst.gwrite_with(self.file_size, offset, ctx)?;
        dst.gwrite_with(self.header_size, offset, ctx)?;
        dst.gwrite_with(self.endian_tag, offset, ctx)?;
        dst.gwrite_with(self.link_size, offset, ctx)?;
        dst.gwrite_with(self.link_off, offset, ctx)?;
        dst.gwrite_with(self.map_off, offset, ctx)?;
        dst.gwrite_with(self.string_ids_size, offset, ctx)?;
        dst.gwrite_with(self.string_ids_off, offset, ctx)?;
        dst.gwrite_with(self.type_ids_size, offset, ctx)?;
        dst.gwrite_with(self.type_ids_off, offset, ctx)?;
        dst.gwrite_with(self.proto_ids_size, offset, ctx)?;
        dst.gwrite_with(self.proto_ids_off, offset, ctx)?;
        dst.gwrite_with(self.field_ids_size, offset, ctx)?;
        dst.gwrite_with(self.field_ids_off, offset, ctx)?;
        dst.gwrite_with(self.method_ids_size, offset, ctx)?;
        dst.gwrite_with(self.method_ids_off, offset, ctx)?;
        dst.gwrite_with(self.class_defs_size, offset, ctx)?;
        dst.gwrite_with(self.class_defs_off, offset, ctx)?;
        dst.gwrite_with(self.data_size, offset, ctx)?;
        dst.gwrite_with(self.data_off, offset, ctx)?;
        Ok(*offset)
    }
}

#[derive(Default)]
pub struct Version(uint, uint, uint);

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}{}", self.0, self.1, self.2)
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("invalid length while parsing: got {0} bytes, expected {MAGIC_LEN} bytes")]
    InvalidLength(usize),
    #[error("invalid magic bytes while parsing")]
    InvalidMagic,
    #[error("invalid version: {0}")]
    InvalidVersion(uint),
    #[error("read error: {0}")]
    ScrollError(#[from] scroll::Error),
}

// dex\nXXX\0
const MAGIC_START: &[ubyte; 4] = b"dex\n";
const MAGIC_END: ubyte = 0;

impl TryFrom<&[ubyte]> for Version {
    type Error = VersionError;
    fn try_from(magic: &[ubyte]) -> Result<Self, Self::Error> {
        if magic.len() != MAGIC_LEN {
            return Err(VersionError::InvalidLength(magic.len()));
        }
        if magic[..4] != *MAGIC_START {
            return Err(VersionError::InvalidMagic);
        }
        if magic[MAGIC_LEN - 1] != MAGIC_END {
            return Err(VersionError::InvalidMagic);
        }
        // Maybe not the most optimal solution, but it'll do for now.
        let mut version = std::str::from_utf8(&magic[4..=6])
            .map_err(|_| VersionError::InvalidMagic)?
            .chars()
            .map(|c| c.to_digit(10).unwrap());
        Ok(Version(
            version.next().unwrap(),
            version.next().unwrap(),
            version.next().unwrap(),
        ))
    }
}

impl TryIntoCtx<scroll::Endian> for Version {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(MAGIC_START.as_slice(), offset, ())?;
        dst.gwrite_with(self.0, offset, ctx)?;
        dst.gwrite_with(self.1, offset, ctx)?;
        dst.gwrite_with(self.2, offset, ctx)?;
        dst.gwrite_with(MAGIC_END, offset, ctx)?;
        Ok(*offset)
    }
}
