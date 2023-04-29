use crate::Result;
use crate::{
    dex::{annotations::Annotation, traits, DexFile},
    raw::code_item::{CodeItem, DebugInfoItem},
};
use scroll::Pread;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct DexMethodImplementation<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    midx: usize,
    raw: CodeItem,
}

impl<'a> DexMethodImplementation<'a> {
    pub fn new(dex: &'a DexFile<'a>, midx: usize, raw: CodeItem) -> Self {
        Self { dex, midx, raw }
    }
}

impl<'a> traits::MethodImplementation for DexMethodImplementation<'a> {
    fn registers(&self) -> u16 {
        self.raw.registers_size
    }

    fn instructions(&self) -> Result<Vec<u16>> {
        todo!()
    }

    fn try_blocks(&self) -> Result<Vec<Annotation>> {
        todo!()
    }

    fn debug_info(&self) -> Result<DebugInfoItem> {
        Ok(self.dex.src.pread(self.raw.debug_info_off as usize)?)
    }
}
