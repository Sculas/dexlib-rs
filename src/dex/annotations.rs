use super::{strings::DexString, DexFile};
use crate::raw::{
    annotations::{Annotation as RawAnnotation, Visibility},
    encoded_value::EncodedAnnotation,
};

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct Annotation<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    visibility: Visibility,
    value: EncodedAnnotation,
}

impl<'a> Annotation<'a> {
    pub fn new(dex: &'a DexFile<'a>, raw: RawAnnotation) -> Self {
        Self {
            dex,
            visibility: raw.visibility,
            value: raw.annotation,
        }
    }

    pub fn descriptor(&self) -> crate::Result<DexString> {
        let id = self
            .dex
            .strings()
            .id_at_type_idx(self.value.type_idx as u32)?;
        Ok(self.dex.strings().get(&id)?)
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }
}
