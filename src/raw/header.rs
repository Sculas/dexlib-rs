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

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Header<'a> {
    pub version: Version,
    pub checksum: uint,
    #[derivative(Debug = "ignore")]
    pub signature: &'a [ubyte],
    pub file_size: uint,
    pub header_size: uint,
    pub endian_tag: uint,
    pub link_size: uint,
    pub link_off: uint,
    pub map_off: uint,
    pub string_ids_size: uint,
    pub string_ids_off: uint,
    pub type_ids_size: uint,
    pub type_ids_off: uint,
    pub proto_ids_size: uint,
    pub proto_ids_off: uint,
    pub field_ids_size: uint,
    pub field_ids_off: uint,
    pub method_ids_size: uint,
    pub method_ids_off: uint,
    pub class_defs_size: uint,
    pub class_defs_off: uint,
    pub data_size: uint,
    pub data_off: uint,
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
