use scroll::Pread;

use crate::error::Error;
use crate::types::header::Header;

pub struct DexFile<'a> {
    pub src: &'a [u8],
    pub header: Header<'a>,
}

impl<'a> DexFile<'a> {
    pub fn new(src: &'a [u8]) -> Result<Self, Error> {
        let header = src.pread_with(0, scroll::LE)?;
        Ok(Self { src, header })
    }
}
