use super::{
    annotations::Annotation,
    classes::DexClass,
    strings::DexString,
    traits::{self, Class},
    DexFile,
};
use crate::{
    raw::{
        annotations::AnnotationsDirectory, class_data::EncodedField, encoded_value::EncodedValue,
        flags::AccessFlags, simple::FieldId,
    },
    Result,
};
use scroll::Pread;
use std::sync::Arc;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct DexField<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    pub(crate) idx: usize,
    fid: FieldId,
    defining_class: DexString,
    access_flags: AccessFlags,
    initial_value: Option<EncodedValue>,
    annotations_dir: Option<Arc<AnnotationsDirectory>>,
}

impl<'a> DexField<'a> {
    pub fn new(
        dex: &'a DexFile<'a>,
        class: &DexClass<'a>,
        raw: &EncodedField,
        prev_idx: usize,
        initial_value: Option<EncodedValue>,
        annotations_dir: Option<Arc<AnnotationsDirectory>>,
    ) -> Result<Self> {
        let idx = raw.field_idx_diff as usize + prev_idx;
        let fid = dex.field_ids_section().index(idx, scroll::LE)?;
        Ok(Self {
            dex,
            idx,
            fid,
            defining_class: class.descriptor()?,
            access_flags: raw.access_flags,
            initial_value,
            annotations_dir,
        })
    }
}

impl<'a> traits::Field for DexField<'a> {
    fn defining_class(&self) -> DexString {
        self.defining_class.clone()
    }

    fn name(&self) -> Result<DexString> {
        let id = self.dex.strings().id_at_idx(self.fid.name_idx)?;
        Ok(self.dex.strings().get(&id)?)
    }

    fn descriptor(&self) -> Result<DexString> {
        let id = self
            .dex
            .strings()
            .id_at_type_idx(self.fid.type_idx as u32)?;
        Ok(self.dex.strings().get(&id)?)
    }

    fn access_flags(&self) -> &AccessFlags {
        &self.access_flags
    }

    fn initial_value(&self) -> Option<&EncodedValue> {
        self.initial_value.as_ref()
    }

    fn annotations(&self) -> Result<Vec<Annotation>> {
        let mut annotations = Vec::new();
        let fas = match &self.annotations_dir {
            Some(dir) => &dir.field_annotations,
            _ => return Ok(Vec::new()),
        };
        for fa in fas {
            if fa.field_idx as usize != self.idx {
                continue;
            }
            let raw = self
                .dex
                .src
                .pread_with(fa.annotations_off as usize, scroll::LE)?;
            annotations.push(Annotation::new(self.dex, raw));
        }
        Ok(annotations)
    }
}
