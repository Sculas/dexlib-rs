use scroll::Pread;

use super::DexFile;
use crate::{
    error::Error,
    raw::{classdef::ClassDef, tysize},
};

impl<'a> DexFile<'a> {
    pub(crate) fn class_def(&self, idx: usize) -> crate::Result<ClassDef> {
        if idx >= self.classdef_info.size {
            return Err(Error::ClassIndexOutOfBounds(idx, self.classdef_info.size));
        }
        Ok(self.src.pread_with(
            self.classdef_info.offset + (idx * tysize::CLASS_DEF),
            scroll::LE,
        )?)
    }
}
