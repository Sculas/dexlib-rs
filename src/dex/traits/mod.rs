use super::{annotations::Annotation, strings::DexString};
use crate::{
    raw::{code_item::DebugInfoItem, encoded_value::EncodedValue, flags::AccessFlags},
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
    fn direct_methods(&self) -> Result<Vec<impl Method>>;
    fn virtual_methods(&self) -> Result<Vec<impl Method>>;
    fn methods(&self) -> Result<Vec<impl Method>>;
}

pub trait Field {
    fn defining_class(&self) -> DexString;
    fn name(&self) -> Result<DexString>;
    fn descriptor(&self) -> Result<DexString>;
    fn access_flags(&self) -> &AccessFlags;
    fn initial_value(&self) -> Option<&EncodedValue>; // TODO: use high level EncodedValue
    fn annotations(&self) -> Result<impl IntoIterator<Item = Annotation>>;
    // TODO: hidden api restrictions
}

pub trait Method {
    fn defining_class(&self) -> DexString;
    fn name(&self) -> Result<DexString>;
    fn parameters(&self) -> Result<Vec<impl MethodParameter>>;
    fn return_type(&self) -> Result<DexString>;
    fn access_flags(&self) -> &AccessFlags;
    fn annotations(&self) -> Result<impl IntoIterator<Item = Annotation>>;
    fn implementation(&self) -> Result<&impl MethodImplementation>;
    // TODO: hidden api restrictions
}

pub trait MethodParameter {
    fn name(&self) -> Result<Option<DexString>>;
    fn descriptor(&self) -> Result<DexString>;
    fn annotations(&self) -> Result<impl IntoIterator<Item = Annotation>>;
    fn signature(&self) -> Result<Option<String>>;
}

pub trait MethodImplementation {
    fn registers(&self) -> u16;
    fn instructions(&self) -> Result<impl IntoIterator<Item = u16>>;
    fn try_blocks(&self) -> Result<impl IntoIterator<Item = Annotation>>;
    fn debug_info(&self) -> Result<DebugInfoItem>; // TODO: use high level DebugInfo
}
