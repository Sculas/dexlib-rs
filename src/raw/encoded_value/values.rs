use crate::raw::{method_handle::MethodHandle, simple::*, string::StringId, *};

macro_rules! value {
    ($name:ident($ty:ty), $prefix:ident) => {
        paste::paste! {
            #[derive(Debug, Clone, Copy)]
            pub struct [<Encoded $name>](pub(super) uint, pub(super) $ty);

            impl [<Encoded $name>] {
                pub fn [<$prefix _idx>](&self) -> uint {
                    self.0
                }
                pub fn [<$prefix _id>](&self) -> $ty {
                    self.1
                }
            }

            impl std::convert::AsRef<$ty> for [<Encoded $name>] {
                fn as_ref(&self) -> &$ty {
                    &self.1
                }
            }

            impl PartialEq for [<Encoded $name>] {
                fn eq(&self, other: &Self) -> bool {
                    self.0 == other.0
                }
            }
        }
    };
}

value!(MethodType(ProtoId), proto);
value!(MethodHandle(MethodHandle), method_handle);
value!(String(StringId), string);
value!(Type(TypeId), type);
value!(Field(FieldId), field);
value!(Method(MethodId), method);
value!(Enum(FieldId), field);
