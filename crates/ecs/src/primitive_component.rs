use std::{any::TypeId, collections::HashMap};

use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use once_cell::sync::Lazy;
use paste::paste;

use crate::{
    AttributeConstructor, AttributeStore, ComponentDesc, ComponentRegistry, ComponentVTable, Description, EntityId,
    ExternalComponentAttributes, Name,
};

use ambient_shared_types::primitive_component_definitions;

// implementation
macro_rules! build_attribute_registration {
    ($type:ty, $store:ident, $attributes:ident) => {{
        if let Some(name) = $attributes.name {
            <Name as AttributeConstructor<$type, _>>::construct(&mut $store, &name);
        }
        if let Some(description) = $attributes.description {
            <Description as AttributeConstructor<$type, _>>::construct(&mut $store, &description);
        }
        $attributes.flags.construct_for_store::<$type>(&mut $store);

        static VTABLE: &ComponentVTable<$type> = &ComponentVTable::construct_external();
        unsafe { VTABLE.erase() }
    }};
}

macro_rules! make_primitive_component {
    ($(($value:ident, $type:ty)),*) => {
        paste! {
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct PrimitiveComponent {
                pub ty: PrimitiveComponentType,
                pub desc: ComponentDesc,
            }

            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[derive(serde::Serialize, serde::Deserialize)]
            pub enum PrimitiveComponentContainerType {
                Vec,
                Option
            }
            impl PrimitiveComponentContainerType {
                pub fn as_str(&self) -> &'static str {
                    match self {
                        Self::Vec => "Vec",
                        Self::Option => "Option",
                    }
                }
            }

            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[derive(serde::Serialize, serde::Deserialize)]
            pub enum PrimitiveComponentType {
                $($value), *,
                $([< Vec $value >]), *,
                $([< Option$value >]), *,
            }

            impl TryFrom<&str> for PrimitiveComponentType {
                type Error = &'static str;

                fn try_from(value: &str) -> Result<Self, Self::Error> {
                    match value {
                        $(stringify!($value) => Ok(Self::$value),)*
                        "Vec" | "Option" => Err("The specified type is a container type, not primitive"),
                        _ => Err("Unsupported type")
                    }
                }
            }

            impl PrimitiveComponent {
                pub fn as_component(&self) -> ComponentDesc {
                    self.desc
                }
            }

            impl PrimitiveComponentType {
                /// Not defined for the container types; use [Self::decompose_container_type].
                pub fn as_str(&self) -> Option<&'static str> {
                    match self {
                        $(Self::$value => Some(stringify!($value)),)*
                        _ => None,
                    }
                }

                pub fn to_vec_type(&self) -> Option<Self> {
                    match self {
                        $(Self::$value => Some(Self::[<Vec $value>]),)*
                        _ => None
                    }
                }

                pub fn to_option_type(&self) -> Option<Self> {
                    match self {
                        $(Self::$value => Some(Self::[<Option $value>]),)*
                        _ => None
                    }
                }

                pub fn decompose_container_type(&self) -> Option<(PrimitiveComponentContainerType, Self)> {
                    match self {
                        $(Self::[<Vec $value>] => Some((PrimitiveComponentContainerType::Vec, Self::$value)),)*
                        $(Self::[<Option $value>] => Some((PrimitiveComponentContainerType::Option, Self::$value)),)*
                        _ => None
                    }
                }

                pub(crate) fn register(&self, reg: &mut ComponentRegistry, path: &str, attributes: ExternalComponentAttributes) {
                    let mut store = AttributeStore::new();
                    let vtable = match self {
                        $(
                            PrimitiveComponentType::$value => {
                                build_attribute_registration!($type, store, attributes)
                            },
                            PrimitiveComponentType::[< Vec $value >] => {
                                build_attribute_registration!(Vec<$type>, store, attributes)
                            },
                            PrimitiveComponentType::[< Option $value >] => {
                                build_attribute_registration!(Option<$type>, store, attributes)
                            },
                        )*
                    };

                    reg.register_external(path.into(), vtable, store);
                }
            }
            impl PartialEq<PrimitiveComponentType> for PrimitiveComponent {
                fn eq(&self, other: &PrimitiveComponentType) -> bool {
                    &self.ty == other
                }
            }

            pub static TYPE_ID_TO_PRIMITIVE_TYPE: Lazy<HashMap<TypeId, PrimitiveComponentType>> = Lazy::new(|| {
                HashMap::from_iter([
                    $((TypeId::of::<$type>(), PrimitiveComponentType::$value),)*
                    $((TypeId::of::<Vec<$type>>(), PrimitiveComponentType::[<Vec $value>]),)*
                    $((TypeId::of::<Option<$type>>(), PrimitiveComponentType::[<Option $value>]),)*
                ])
            });
        }
    }
}

primitive_component_definitions!(make_primitive_component);
