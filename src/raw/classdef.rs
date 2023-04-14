use crate::raw::*;
use scroll::{Pread, Pwrite};

use super::flags::AccessFlags;

#[derive(Debug, Pread, Pwrite)]
pub struct ClassDef {
    /// Index into the `type_ids` list for this class.
    /// This must be a class type, and not an array or primitive type.
    pub class_idx: uint,
    /// Access flags for the class (`public`, `final`, etc.).
    /// See [`AccessFlags`] for details.
    pub access_flags: AccessFlags,
    /// Index into the `type_ids` list for the superclass,
    /// or the constant value [`NO_INDEX`] if this class has no superclass (i.e., it is a root class such as Object).
    /// If present, this must be a class type, and not an array or primitive type.
    pub superclass_idx: uint,
    /// Offset from the start of the file to the list of interfaces, or 0 if there are none.
    /// This offset should be in the `data` section, and the data there should be in the format specified by `type_list`.
    /// Each of the elements of the list must be a class type (not an array or primitive type), and there must not be any duplicates.
    pub interfaces_off: uint,
    /// Index into the `string_ids` list for the name of the file containing the original source for (at least most of) this class,
    /// or the special value [`NO_INDEX`] to represent a lack of this information.
    /// The `debug_info_item` of any given method may override this source file,
    /// but the expectation is that most classes will only come from one source file.
    pub source_file_idx: uint,
    /// Offset from the start of the file to the annotations structure for this class, or 0 if there are no annotations on this class.
    /// This offset, if non-zero, should be in the `data` section, and the data there should be in the format specified by `annotations_directory_item`,
    /// with all items referring to this class as the definer.
    pub annotations_off: uint,
    /// Offset from the start of the file to the associated class data for this item, or 0 if there is no class data for this class
    /// (this may be the case, for example, if this class is a marker interface).
    /// The offset, if non-zero, should be in the `data` section, and the data there should be in the format specified by `class_data_item`,
    /// with all items referring to this class as the definer.
    pub class_data_off: uint,
    /// Offset from the start of the file to the list of initial values for `static` fields, or 0 if there are none
    /// (and all `static` fields are to be initialized with 0 or null).
    /// This offset should be in the `data` section, and the data there should be in the format specified by `encoded_array_item`.
    /// The size of the array must be no larger than the number of `static` fields declared by this class,
    /// and the elements correspond to the `static` fields in the same order as declared in the corresponding `field_list`.
    /// The type of each array element must match the declared type of its corresponding field.
    /// If there are fewer elements in the array than there are `static` fields,
    /// then the leftover fields are initialized with a type-appropriate 0 or `null`.
    pub static_values_off: uint,
}
