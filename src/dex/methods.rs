use std::sync::Arc;

use self::code::DexMethodImplementation;

use super::{
    annotations::Annotation,
    classes::DexClass,
    strings::DexString,
    traits::{self, Class, MethodImplementation},
    DexFile,
};
use crate::{
    raw::{
        annotations::AnnotationsDirectory,
        class_data::EncodedMethod,
        encoded_value::EncodedValue,
        flags::AccessFlags,
        simple::{MethodId, ProtoId},
        type_list::TypeList,
    },
    Result,
};
use once_cell::unsync::OnceCell;
use scroll::Pread;

pub mod code;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct DexMethod<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    pub(crate) idx: usize,
    mid: MethodId,
    pid: ProtoId,
    defining_class: DexString,
    access_flags: AccessFlags,
    annotations_dir: Option<Arc<AnnotationsDirectory>>,
    code_offset: u64,
    implementation: OnceCell<DexMethodImplementation<'a>>,
}

impl<'a> DexMethod<'a> {
    pub fn new(
        dex: &'a DexFile<'a>,
        class: &DexClass<'a>,
        raw: &EncodedMethod,
        prev_idx: usize,
        annotations_dir: Option<Arc<AnnotationsDirectory>>,
    ) -> Result<Self> {
        let idx = raw.method_idx_diff as usize + prev_idx;
        let mid = dex.method_ids_section().index(idx, scroll::LE)?;
        let pid = dex.proto_ids_section().index(idx, scroll::LE)?;
        Ok(Self {
            dex,
            idx,
            mid,
            pid,
            defining_class: class.descriptor()?,
            access_flags: raw.access_flags,
            annotations_dir,
            code_offset: raw.code_off,
            implementation: OnceCell::new(),
        })
    }
}

impl<'a> traits::Method for DexMethod<'a> {
    fn defining_class(&self) -> DexString {
        self.defining_class.clone()
    }

    fn name(&self) -> Result<DexString> {
        let id = self.dex.strings().id_at_idx(self.mid.name_idx)?;
        Ok(self.dex.strings().get(&id)?)
    }

    fn parameters(&self) -> Result<Vec<DexMethodParameter<'a>>> {
        let mut params = Vec::new();
        let debug_info = self.implementation()?.debug_info()?;
        let param_info = self
            .dex
            .src
            .pread_with::<TypeList>(self.pid.parameters_off as usize, scroll::LE)?
            .into_inner()
            .into_iter()
            .zip(debug_info.parameter_names.iter());
        for (ty, name) in param_info {
            let param = DexMethodParameter::new(
                self.dex,
                self.idx,
                name.clone(),
                ty.type_idx as u32,
                self.annotations_dir.clone(),
            );
            params.push(param);
        }
        Ok(params)
    }

    fn return_type(&self) -> Result<DexString> {
        let id = self
            .dex
            .strings()
            .id_at_type_idx(self.pid.return_type_idx)?;
        Ok(self.dex.strings().get(&id)?)
    }

    fn access_flags(&self) -> &AccessFlags {
        &self.access_flags
    }

    fn annotations(&self) -> Result<Vec<Annotation>> {
        let mut annotations = Vec::new();
        let mas = match &self.annotations_dir {
            Some(dir) => &dir.method_annotations,
            _ => return Ok(Vec::new()),
        };
        for ma in mas {
            if ma.method_idx as usize != self.idx {
                continue;
            }
            let raw = self
                .dex
                .src
                .pread_with(ma.annotations_off as usize, scroll::LE)?;
            annotations.push(Annotation::new(self.dex, raw));
        }
        Ok(annotations)
    }

    fn implementation(&self) -> Result<&DexMethodImplementation<'a>> {
        Ok(self.implementation.get_or_try_init(|| {
            let offset = self.code_offset as usize;
            let raw = self.dex.src.pread_with(offset, scroll::LE)?;
            Ok::<_, scroll::Error>(DexMethodImplementation::new(self.dex, self.idx, raw))
        })?)
    }
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct DexMethodParameter<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    midx: usize,
    name_idx: Option<u64>,
    type_idx: u32,
    annotations_dir: Option<Arc<AnnotationsDirectory>>,
}

impl<'a> DexMethodParameter<'a> {
    pub fn new(
        dex: &'a DexFile<'a>,
        midx: usize,
        name_idx: Option<u64>,
        type_idx: u32,
        annotations_dir: Option<Arc<AnnotationsDirectory>>,
    ) -> Self {
        Self {
            dex,
            midx,
            name_idx,
            type_idx,
            annotations_dir,
        }
    }
}

impl<'a> traits::MethodParameter for DexMethodParameter<'a> {
    fn name(&self) -> Result<Option<DexString>> {
        match self.name_idx {
            Some(idx) => {
                let id = self.dex.strings().id_at_idx(idx as u32)?;
                Ok(Some(self.dex.strings().get(&id)?))
            }
            _ => Ok(None),
        }
    }

    fn descriptor(&self) -> Result<DexString> {
        let id = self.dex.strings().id_at_type_idx(self.type_idx)?;
        Ok(self.dex.strings().get(&id)?)
    }

    fn annotations(&self) -> Result<Vec<Annotation>> {
        let mut annotations = Vec::new();
        let pas = match &self.annotations_dir {
            Some(dir) => &dir.parameter_annotations,
            _ => return Ok(Vec::new()),
        };
        for pa in pas {
            if pa.method_idx as usize != self.midx {
                continue;
            }
            let raw = self
                .dex
                .src
                .pread_with(pa.annotations_off as usize, scroll::LE)?;
            annotations.push(Annotation::new(self.dex, raw));
        }
        Ok(annotations)
    }

    fn signature(&self) -> Result<Option<String>> {
        let annotations: Vec<Annotation> = self.annotations()?;
        for a in annotations {
            if *a.descriptor()? != "Ldalvik/annotation/Signature;" {
                continue;
            }
            for el in a.elements() {
                if *el.name()? != "value" {
                    continue;
                }
                if let EncodedValue::Array(v) = el.value() {
                    let mut sig = String::new();
                    for el in v {
                        if let EncodedValue::String(idx) = el {
                            let sidx = self.dex.strings().id_at_idx(*idx)?;
                            let str = self.dex.strings().get(&sidx)?;
                            sig.push_str(&str);
                        }
                    }
                    return Ok(Some(sig));
                }
            }
        }
        Ok(None)
    }
}
