use once_cell::unsync::OnceCell;
use scroll::Pread;

use super::{annotations::Annotation, strings::DexString, DexFile};
use crate::{
    raw::{
        annotations::{AnnotationSetItem, AnnotationsDirectory},
        class_data::ClassData,
        classdef::ClassDef,
        flags::AccessFlags,
        string::StringId,
        type_list::TypeList,
        NO_INDEX,
    },
    utils::set::LazySet,
};

pub mod iter;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct Class<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    def: ClassDef,
    data: Option<ClassData>,

    // internal
    descriptor_id: OnceCell<StringId>,
    annotations_dir: OnceCell<Option<AnnotationsDirectory>>,
}

impl<'a> Class<'a> {
    pub fn new(dex: &'a DexFile<'a>, def: ClassDef) -> crate::Result<Self> {
        let data = if def.class_data_off > 0 {
            Some(dex.src.pread(def.class_data_off as usize)?)
        } else {
            None
        };
        Ok(Self {
            dex,
            def,
            data,

            // internal
            descriptor_id: OnceCell::new(),
            annotations_dir: OnceCell::new(),
        })
    }

    pub fn descriptor(&self) -> crate::Result<DexString> {
        let id = self
            .descriptor_id
            .get_or_try_init(|| self.dex.strings().id_at_type_idx(self.def.class_idx))?;
        Ok(self.dex.strings().get(id)?)
    }

    pub fn superclass(&self) -> crate::Result<Option<DexString>> {
        if self.def.superclass_idx == NO_INDEX {
            return Ok(None);
        }
        let id = self.dex.strings().id_at_type_idx(self.def.superclass_idx)?;
        Ok(Some(self.dex.strings().get(&id)?))
    }

    pub fn access_flags(&self) -> &AccessFlags {
        &self.def.access_flags
    }

    pub fn source_file(&self) -> crate::Result<Option<DexString>> {
        if self.def.source_file_idx == NO_INDEX {
            return Ok(None);
        }
        let id = self.dex.strings().id_at_idx(self.def.source_file_idx)?;
        Ok(Some(self.dex.strings().get(&id)?))
    }

    pub fn interfaces(&self) -> crate::Result<LazySet<crate::Result<DexString>>> {
        if self.def.interfaces_off == 0 {
            return Ok(LazySet::empty());
        }
        let ty_list = self
            .dex
            .src
            .pread_with::<TypeList>(self.def.interfaces_off as usize, scroll::LE)?
            .into_inner();
        let strings = self.dex.strings();
        let interfaces = LazySet::new(ty_list.len(), move |idx| {
            let id = strings.id_at_type_idx(ty_list[idx].type_idx as u32)?;
            Ok(strings.get(&id)?)
        });
        Ok(interfaces)
    }

    pub fn annotations(&self) -> crate::Result<LazySet<crate::Result<Annotation>>> {
        let offset = match self.annotations_dir()? {
            Some(dir) if dir.class_annotations_off != 0 => dir.class_annotations_off,
            _ => return Ok(LazySet::empty()),
        };
        let set = self
            .dex
            .src
            .pread_with::<AnnotationSetItem>(offset as usize, scroll::LE)?
            .into_inner();
        let annotations = LazySet::new(set.len(), move |idx| {
            let annotation = self.dex.src.pread_with(set[idx] as usize, scroll::LE)?;
            Ok(Annotation::new(self.dex, annotation))
        });
        Ok(annotations)
    }
}

impl<'a> Class<'a> {
    fn annotations_dir(&self) -> crate::Result<Option<&AnnotationsDirectory>> {
        Ok(self
            .annotations_dir
            .get_or_try_init(|| {
                let offset = self.def.annotations_off as usize;
                if offset == 0 {
                    return Ok::<_, scroll::Error>(None);
                }
                Ok(Some(self.dex.src.pread_with(offset, scroll::LE)?))
            })?
            .as_ref())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn annotations() {
        let dex = crate::t::dex!();
        for class in dex.classes() {
            let class = class.unwrap();
            let annotations = class
                .annotations()
                .unwrap()
                .into_iter()
                .map(|x| x.unwrap())
                .collect::<Vec<_>>();
            if !annotations.is_empty() {
                println!("class {} => {annotations:?}", class.descriptor().unwrap());
            }
        }
    }
}
