use std::{any::TypeId, collections::HashMap};

use elements_std::asset_url::ObjectRef;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use paste::paste;

use crate::{
    ComponentDesc, ComponentRegistry, ComponentVTable, Debuggable, EntityId, EntityUid, Networked, PrimitiveAttributeRegistry, Store
};

pub static PRIMITIVE_ATTRIBUTE_REGISTRY: Lazy<RwLock<PrimitiveAttributeRegistry>> = Lazy::new(Default::default);

macro_rules! make_primitive_component_with_attrs {
    ($(($value:ident, $type:ty, [$($attr: ty),*])),*) => { paste! {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct PrimitiveComponent {
            pub ty: PrimitiveComponentType,
            pub desc: ComponentDesc,
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
            /// Register dynamic attributes for primitive component types
            pub fn register_attributes() {
                let mut reg = PRIMITIVE_ATTRIBUTE_REGISTRY.write();

                $(
                    {
                        #[allow(unused_mut)]
                        let mut store = $crate::AttributeStore::new();


                        $(
                            <$attr as $crate::AttributeConstructor::<$type, _>>::construct(
                                &mut store,
                                ()
                            );
                        )*

                        reg.set(PrimitiveComponentType::$value, store);
                    }
                    {
                        #[allow(unused_mut)]
                        let mut store = $crate::AttributeStore::new();


                        $(
                            <$attr as $crate::AttributeConstructor::<Vec<$type>, _>>::construct(
                                &mut store,
                                ()
                            );
                        )*

                        reg.set(PrimitiveComponentType::[<Vec $value >], store);
                    }
                    {
                        #[allow(unused_mut)]
                        let mut store = $crate::AttributeStore::new();


                        $(
                            <$attr as $crate::AttributeConstructor::<Option<$type>, _>>::construct(
                                &mut store,
                                ()
                            );
                        )*

                        reg.set(PrimitiveComponentType::[<Option $value >], store);
                    }
                )*
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

            pub(crate) fn register(&self, reg: &mut ComponentRegistry, path: &str) -> Option<PrimitiveComponent> {
                match self {
                    $(
                        PrimitiveComponentType::$value =>
                        {

                            static VTABLE: &ComponentVTable<$type> = &ComponentVTable::construct_external() ;
                            reg.register_external(path.into(), unsafe { VTABLE.erase() }, Default::default());

                            reg.set_primitive_component(path.into(), self.clone())
                        },
                        PrimitiveComponentType::[< Vec $value >] =>

                        {
                            static VTABLE: &ComponentVTable<Vec<$type>> = &ComponentVTable::construct_external() ;
                            reg.register_external(path.into(), unsafe { VTABLE.erase() }, Default::default());

                            reg.set_primitive_component(path.into(), self.clone())
                        },
                        PrimitiveComponentType::[< Option $value >] =>
                        {
                            static VTABLE: &ComponentVTable<Option<$type>> = &ComponentVTable::construct_external() ;
                            reg.register_external(path.into(), unsafe { VTABLE.erase() }, Default::default());

                            reg.set_primitive_component(path.into(), self.clone())
                        },
                    )*
                }
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
    } };
}

macro_rules! make_primitive_component {
    ($(($value:ident, $type:ty)),*) => {
        make_primitive_component_with_attrs!(
            $(($value, $type, [Debuggable, Networked, Store])),*
        );
    }
}

#[macro_export]
macro_rules! primitive_component_definitions {
    ($macro_to_instantiate:ident) => {
        $macro_to_instantiate!(
            (Empty, ()),
            (Bool, bool),
            (EntityId, EntityId),
            (F32, f32),
            (F64, f64),
            (Mat4, Mat4),
            (I32, i32),
            (Quat, Quat),
            (String, String),
            (U32, u32),
            (U64, u64),
            (Vec2, Vec2),
            (Vec3, Vec3),
            (Vec4, Vec4),
            (ObjectRef, ObjectRef),
            (EntityUid, EntityUid)
        );
    };
}

primitive_component_definitions!(make_primitive_component);
