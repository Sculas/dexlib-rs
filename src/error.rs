use crate::{
    dex::{section::Error as SectionError, strings::StringReadError},
    raw::{
        annotations::AnnotationError,
        class_data::ClassDataError,
        code_item::DebugInfoError,
        encoded_value::EncodedValueError,
        header::HeaderError,
        map_list::{ItemType, MapListError},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error parsing header: {0}")]
    Header(#[from] HeaderError),
    #[error("error parsing map_list: {0}")]
    MapList(#[from] MapListError),
    #[error("error reading string: {0}")]
    StringRead(#[from] StringReadError),
    #[error("error reading from section: {0}")]
    Section(#[from] SectionError),
    #[error("map list does not contain required item type: {0:?}")]
    InvalidMapList(ItemType),
    #[error("class index {0} out of bounds, classes count: {1}")]
    ClassIndexOutOfBounds(usize, usize),
    #[error("error reading class data: {0}")]
    ClassData(#[from] ClassDataError),
    #[error("error reading annotation: {0}")]
    Annotation(#[from] AnnotationError),
    #[error("error reading encoded value: {0}")]
    EncodedValue(#[from] EncodedValueError),
    #[error("error reading debug info: {0}")]
    DebugInfo(#[from] DebugInfoError),
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}
