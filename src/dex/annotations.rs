use super::{strings::DexString, DexFile};
use crate::raw::{
    annotations::{Annotation as RawAnnotation, Visibility},
    encoded_value::{AnnotationElement as RawAnnotationElement, EncodedValue},
};

// TODO: write traits or use once_cells here

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct Annotation<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    type_idx: u32,
    visibility: Visibility,
    elements: Vec<AnnotationElement<'a>>,
}

impl<'a> Annotation<'a> {
    pub fn new(dex: &'a DexFile<'a>, raw: RawAnnotation) -> Self {
        Self {
            dex,
            type_idx: raw.annotation.type_idx as u32,
            visibility: raw.visibility,
            elements: raw
                .annotation
                .elements
                .into_iter()
                .map(|e| AnnotationElement::new(dex, e))
                .collect(),
        }
    }

    pub fn descriptor(&self) -> crate::Result<DexString> {
        let id = self.dex.strings().id_at_type_idx(self.type_idx)?;
        Ok(self.dex.strings().get(&id)?)
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn elements(&self) -> &[AnnotationElement<'a>] {
        &self.elements
    }
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct AnnotationElement<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    name_idx: u64,
    value: EncodedValue, // TODO: use high level EncodedValue
}

impl<'a> AnnotationElement<'a> {
    pub fn new(dex: &'a DexFile<'a>, raw: RawAnnotationElement) -> Self {
        Self {
            dex,
            name_idx: raw.name_idx,
            value: raw.value,
        }
    }

    pub fn name(&self) -> crate::Result<DexString> {
        let id = self
            .dex
            .strings()
            .id_at_idx(self.name_idx.try_into().expect("bad name idx?"))?;
        Ok(self.dex.strings().get(&id)?)
    }

    pub fn value(&self) -> &EncodedValue {
        &self.value
    }
}
