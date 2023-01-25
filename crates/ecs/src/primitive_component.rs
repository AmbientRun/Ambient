use elements_std::asset_url::ObjectRef;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use paste::paste;

use crate::{
    ComponentDesc, ComponentRegistry, ComponentVTable, Debuggable, EntityId, EntityUid, Networked, PrimitiveAttributeRegistry, Store
};

pub static PRIMITIVE_ATTRIBUTE_REGISTRY: Lazy<RwLock<PrimitiveAttributeRegistry>> = Lazy::new(Default::default);

macro_rules! make_primitive_component {
    ($(($value:ident, $type:ty, [$($attr: ty),*])),*) => { paste! {
        #[derive(Debug,  Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct PrimitiveComponent {
            pub ty: PrimitiveComponentType,
            pub desc: ComponentDesc,
        }

        #[derive(Debug,  Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(tag = "type")]
        pub enum PrimitiveComponentType {
            $($value), *,
            $([< Vec $value >]), *,
            $([< Option$value >]), *,
            Vec { variants: Box<PrimitiveComponentType> },
            Option { variants: Box<PrimitiveComponentType> },
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


            pub(crate) fn register(&self, reg: &mut ComponentRegistry, path: &str) -> Option<PrimitiveComponent> {
                match self {
                    $(
                        ty @ PrimitiveComponentType::$value =>
                        {

                            static VTABLE: &ComponentVTable<$type> = &ComponentVTable::construct_external() ;
                            reg.register_external(path.into(), unsafe { VTABLE.erase() } , Default::default());

                            reg.set_primitive_component(path.into(), ty.clone())
                        },
                        ty @ PrimitiveComponentType::[< Vec $value >] =>

                        {
                            static VTABLE: &ComponentVTable<Vec<$type>> = &ComponentVTable::construct_external() ;
                            reg.register_external(path.into(), unsafe { VTABLE.erase() }, Default::default());

                            reg.set_primitive_component(path.into(), ty.clone())
                        },
                        ty @ PrimitiveComponentType::[< Option $value >] =>
                        {
                            static VTABLE: &ComponentVTable<Option<$type>> = &ComponentVTable::construct_external() ;
                            reg.register_external(path.into(), unsafe { VTABLE.erase() }, Default::default());

                            reg.set_primitive_component(path.into(), ty.clone())
                        },
                    )*
                    PrimitiveComponentType::Vec { variants } => match **variants {
                        $(
                            PrimitiveComponentType::$value =>
                            {
                                static VTABLE: &ComponentVTable<Vec<$type>> = &ComponentVTable::construct_external() ;
                                reg.register_external(path.into(), unsafe { VTABLE.erase() }, Default::default());

                                let ty = PrimitiveComponentType::[< Vec $value >];
                                reg.set_primitive_component(path.into(), ty.clone())
                            },
                        )*
                        _ => panic!("Unsupported Vec inner type: {:?}", variants),
                    },
                    PrimitiveComponentType::Option { variants } => match **variants {
                        $(
                            PrimitiveComponentType::$value =>
                            {
                                static VTABLE: &ComponentVTable<Option<$type>> = &ComponentVTable::construct_external() ;
                                reg.register_external(path.into(), unsafe { VTABLE.erase() }, Default::default());

                                let ty = PrimitiveComponentType::[< Vec $value >];
                                reg.set_primitive_component(path.into(), ty.clone())
                            },
                        )*
                        _ => panic!("Unsupported Option inner type: {:?}", variants),
                    }
                }
            }
        }
        impl PartialEq<PrimitiveComponentType> for PrimitiveComponent {
            fn eq(&self, other: &PrimitiveComponentType) -> bool {
                &self.ty == other
            }
        }
    } };
}

make_primitive_component!(
    (Empty, (), [Debuggable, Networked, Store]),
    (Bool, bool, [Debuggable, Networked, Store]),
    (EntityId, EntityId, [Debuggable, Networked, Store]),
    (F32, f32, [Debuggable, Networked, Store]),
    (F64, f64, [Debuggable, Networked, Store]),
    (Mat4, Mat4, [Debuggable, Networked, Store]),
    (I32, i32, [Debuggable, Networked, Store]),
    (Quat, Quat, [Debuggable, Networked, Store]),
    (String, String, [Debuggable, Networked, Store]),
    (U32, u32, [Debuggable, Networked, Store]),
    (U64, u64, [Debuggable, Networked, Store]),
    (Vec2, Vec2, [Debuggable, Networked, Store]),
    (Vec3, Vec3, [Debuggable, Networked, Store]),
    (Vec4, Vec4, [Debuggable, Networked, Store]),
    (ObjectRef, ObjectRef, [Debuggable, Networked, Store]),
    (EntityUid, EntityUid, [Debuggable, Networked, Store])
);
