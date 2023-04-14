use crate::raw::*;
use scroll::{Pread, Pwrite};

#[derive(Debug, Pread, Pwrite)]
pub struct TypeId {
    /// Index into the `string_ids` list for the descriptor string of this type.
    /// The string must conform to the syntax for `TypeDescriptor`.
    pub descriptor_idx: uint,
}

#[derive(Debug, Pread, Pwrite)]
pub struct ProtoId {
    /// Index into the `string_ids` list for the short-form descriptor string of this prototype.
    /// The string must conform to the syntax for `ShortyDescriptor`
    /// and must correspond to the return type and parameters of this item.
    pub shorty_idx: uint,
    /// Index into the `type_ids` list for the return type of this prototype.
    pub return_type_idx: uint,
    /// Offset from the start of the file to the list of parameter types for this prototype,
    /// or 0 if this prototype has no parameters.
    /// This offset, if non-zero, should be in the data section,
    /// and the data there should be in the format specified by `type_list`.
    /// Additionally, there should be no reference to the type void in the list.
    pub parameters_off: uint,
}

#[derive(Debug, Pread, Pwrite)]
pub struct FieldId {
    /// Index into the `type_ids` list for the definer of this field.
    /// This must be a class type, and not an array or primitive type.
    pub class_idx: ushort,
    /// Index into the `type_ids` list for the type of this field.
    pub type_idx: ushort,
    /// Index into the `string_ids` list for the name of this field.
    /// The string must conform to the syntax for `MemberName`.
    pub name_idx: uint,
}

#[derive(Debug, Pread, Pwrite)]
pub struct MethodId {
    /// Index into the `type_ids` list for the definer of this field.
    /// This must be a class type, and not an array or primitive type.
    pub class_idx: ushort,
    /// Index into the `proto_ids` list for the type of this method.
    pub proto_idx: ushort,
    /// Index into the `string_ids` list for the name of this field.
    /// The string must conform to the syntax for `MemberName`.
    pub name_idx: uint,
}

#[derive(Debug, Pread, Pwrite)]
pub struct CallSiteId {
    /// Offset from the start of the file to call site definition.
    /// The offset should be in the `data` section,
    /// and the data there should be in the format specified by `call_site_item`.
    pub call_site_off: uint,
}
