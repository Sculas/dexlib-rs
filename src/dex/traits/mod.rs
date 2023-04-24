use super::{annotations::Annotation, strings::DexString};
use crate::{
    raw::{encoded_value::EncodedValue, flags::AccessFlags},
    Result,
};

pub trait Class {
    fn descriptor(&self) -> Result<DexString>;
    fn superclass(&self) -> Result<Option<DexString>>;
    fn access_flags(&self) -> &AccessFlags;
    fn source_file(&self) -> Result<Option<DexString>>;
    fn interfaces(&self) -> Result<Vec<DexString>>;
    fn annotations(&self) -> Result<Vec<Annotation>>;
    fn static_fields(&self) -> Result<Vec<impl Field>>;
    fn instance_fields(&self) -> Result<Vec<impl Field>>;
    fn fields(&self) -> Result<Vec<impl Field>>;
    // fn direct_methods(&self) -> Result<Vec<impl Method>>;
    // fn virtual_methods(&self) -> Result<Vec<impl Method>>;
    // fn methods(&self) -> Result<Vec<impl Method>>;
}

pub trait Field {
    fn defining_class(&self) -> DexString;
    fn name(&self) -> Result<DexString>;
    fn descriptor(&self) -> Result<DexString>;
    fn access_flags(&self) -> &AccessFlags;
    fn initial_value(&self) -> Result<Option<EncodedValue>>; // TODO: use high level EncodedValue
    fn annotations(&self) -> Result<impl IntoIterator<Item = Annotation>>;
    // TODO: hidden api restrictions
}

pub trait Method {
    fn defining_class(&self) -> DexString;
    fn name(&self) -> Result<DexString>;
    fn parameters(&self) -> Result<DexString>;
    fn return_type(&self) -> Result<DexString>;
    fn access_flags(&self) -> &AccessFlags;
    fn annotations(&self) -> Result<impl IntoIterator<Item = Annotation>>;
    // TODO: hidden api restrictions
    // TODO: method implementation
}