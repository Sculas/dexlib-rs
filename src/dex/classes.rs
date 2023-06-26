use std::sync::Arc;

use super::{
    annotations::Annotation, fields::DexField, methods::DexMethod, strings::DexString, traits,
    DexFile,
};
use crate::{
    raw::{
        annotations::{AnnotationSetItem, AnnotationsDirectory},
        class_data::{ClassData, EncodedMethod},
        classdef::ClassDef,
        encoded_value::{EncodedArrayItem, EncodedValueError},
        flags::AccessFlags,
        string::StringId,
        type_list::TypeList,
        NO_INDEX, NO_OFFSET,
    },
    utils::IntoArc,
    Result,
};
use once_cell::unsync::OnceCell;
use scroll::Pread;

pub mod iter;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct DexClass<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    def: ClassDef,
    data: Option<ClassData>, // TODO: lazy load

    // internal
    descriptor_id: OnceCell<StringId>,
    annotations_dir: OnceCell<Option<Arc<AnnotationsDirectory>>>,
    static_values: OnceCell<Option<EncodedArrayItem>>,
}

impl<'a> DexClass<'a> {
    pub fn new(dex: &'a DexFile<'a>, def: ClassDef) -> Result<Self> {
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
            static_values: OnceCell::new(),
        })
    }
}

impl<'a> traits::Class for DexClass<'a> {
    fn descriptor(&self) -> Result<DexString> {
        let id = self
            .descriptor_id
            .get_or_try_init(|| self.dex.strings().id_at_type_idx(self.def.class_idx))?;
        Ok(self.dex.strings().get(id)?)
    }

    fn superclass(&self) -> Result<Option<DexString>> {
        if self.def.superclass_idx == NO_INDEX {
            return Ok(None);
        }
        let id = self.dex.strings().id_at_type_idx(self.def.superclass_idx)?;
        Ok(Some(self.dex.strings().get(&id)?))
    }

    fn access_flags(&self) -> &AccessFlags {
        &self.def.access_flags
    }

    fn source_file(&self) -> Result<Option<DexString>> {
        if self.def.source_file_idx == NO_INDEX {
            return Ok(None);
        }
        let id = self.dex.strings().id_at_idx(self.def.source_file_idx)?;
        Ok(Some(self.dex.strings().get(&id)?))
    }

    fn interfaces(&self) -> Result<Vec<DexString>> {
        let mut interfaces = Vec::new();
        if self.def.interfaces_off == 0 {
            return Ok(interfaces);
        }
        let ty_list = self
            .dex
            .src
            .pread_with::<TypeList>(self.def.interfaces_off as usize, scroll::LE)?
            .into_inner();
        interfaces.reserve(ty_list.len());
        for ty in ty_list {
            let id = self.dex.strings().id_at_type_idx(ty.type_idx as u32)?;
            interfaces.push(self.dex.strings().get(&id)?);
        }
        Ok(interfaces)
    }

    fn annotations(&self) -> Result<Vec<Annotation>> {
        let mut annotations = Vec::new();
        let offset = match self.annotations_dir()? {
            Some(dir) if dir.class_annotations_off != NO_OFFSET => dir.class_annotations_off,
            _ => return Ok(annotations),
        };
        let offsets = self
            .dex
            .src
            .pread_with::<AnnotationSetItem>(offset as usize, scroll::LE)?
            .into_inner();
        annotations.reserve(offsets.len());
        for offset in offsets {
            let raw = self.dex.src.pread_with(offset as usize, scroll::LE)?;
            annotations.push(Annotation::new(self.dex, raw));
        }
        Ok(annotations)
    }

    fn static_fields(&self) -> Result<Vec<DexField<'a>>> {
        let mut fields = Vec::new();
        let efs = match &self.data {
            Some(data) => &data.static_fields,
            _ => return Ok(Vec::new()),
        };
        let mut static_values = self.static_values()?.map(|v| v.iter());
        let annotations_dir = self.annotations_dir()?;
        let mut prev_idx = 0;
        for ef in efs {
            let initial_value = match static_values {
                Some(ref mut values) => values.next(),
                _ => None,
            };
            let field = DexField::new(
                self.dex,
                self,
                ef,
                prev_idx,
                initial_value.cloned(),
                annotations_dir.clone(),
            )?;
            prev_idx = field.idx;
            fields.push(field);
        }
        Ok(fields)
    }

    fn instance_fields(&self) -> Result<Vec<DexField<'a>>> {
        let mut fields = Vec::new();
        let efs = match &self.data {
            Some(data) => &data.instance_fields,
            _ => return Ok(Vec::new()),
        };
        let annotations_dir = self.annotations_dir()?;
        let mut prev_idx = 0;
        for ef in efs {
            let field = DexField::new(self.dex, self, ef, prev_idx, None, annotations_dir.clone())?;
            prev_idx = field.idx;
            fields.push(field);
        }
        Ok(fields)
    }

    fn fields(&self) -> Result<Vec<DexField<'a>>> {
        let mut fields = Vec::new();
        fields.extend(self.static_fields()?);
        fields.extend(self.instance_fields()?);
        Ok(fields)
    }

    fn direct_methods(&self) -> Result<Vec<DexMethod<'a>>> {
        self.parse_methods(match &self.data {
            Some(data) => &data.direct_methods,
            _ => return Ok(Vec::new()),
        })
    }

    fn virtual_methods(&self) -> Result<Vec<DexMethod<'a>>> {
        self.parse_methods(match &self.data {
            Some(data) => &data.virtual_methods,
            _ => return Ok(Vec::new()),
        })
    }

    fn methods(&self) -> Result<Vec<DexMethod<'a>>> {
        let mut methods = Vec::new();
        methods.extend(self.direct_methods()?);
        methods.extend(self.virtual_methods()?);
        Ok(methods)
    }
}

impl<'a> DexClass<'a> {
    fn annotations_dir(&self) -> Result<Option<Arc<AnnotationsDirectory>>> {
        Ok(self
            .annotations_dir
            .get_or_try_init(|| {
                if self.def.annotations_off == NO_OFFSET {
                    return Ok::<_, scroll::Error>(None);
                }
                let dir: AnnotationsDirectory = self
                    .dex
                    .src
                    .pread_with(self.def.annotations_off as usize, scroll::LE)?;
                Ok(Some(dir.into_arc()))
            })?
            .clone())
    }

    fn static_values(&self) -> Result<Option<&EncodedArrayItem>> {
        Ok(self
            .static_values
            .get_or_try_init(|| {
                if self.def.static_values_off == NO_OFFSET {
                    return Ok::<_, EncodedValueError>(None);
                }
                let item = self.dex.src.pread(self.def.static_values_off as usize)?;
                Ok(Some(item))
            })?
            .as_ref())
    }

    fn parse_methods(&self, raw_methods: &[EncodedMethod]) -> Result<Vec<DexMethod<'a>>> {
        let mut methods = Vec::new();
        let annotations_dir = self.annotations_dir()?;
        let mut prev_idx = 0;
        for raw in raw_methods {
            let method = DexMethod::new(self.dex, self, raw, prev_idx, annotations_dir.clone())?;
            prev_idx = method.idx;
            methods.push(method);
        }
        Ok(methods)
    }
}
