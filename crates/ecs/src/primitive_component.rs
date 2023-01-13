use elements_std::asset_url::ObjectRef;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use paste::paste;

use crate::{ComponentDesc, ComponentRegistry, ComponentVTable, EntityId, EntityUid};

macro_rules! make_primitive_component {
    ($(($value:ident, $type:ty)),*) => { paste! {
        #[derive(Debug,  Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct PrimitiveComponent {
            pub ty: PrimitiveComponentType,
            pub desc: ComponentDesc,

            // $($value(Component<$type>)), *,
            // $([<Vec $value>](Component<Vec<$type>>)), *,
            // $([<Option $value>](Component<Option<$type>>)), *
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
            pub(crate) fn register(&self, reg: &mut ComponentRegistry, key: &str, decorating: bool) -> PrimitiveComponent {
                match self {
                    $(
                        ty @ PrimitiveComponentType::$value => reg.register_with_primitive(
                            {
                                static VTABLE: &ComponentVTable<$type> = &ComponentVTable::construct("core", "<unknown>", |_, _| None);
                                unsafe { VTABLE.erase() }
                            },
                            ty.clone()
                        ),
                        ty @ PrimitiveComponentType::[< Vec $value >] => reg.register_with_primitive(
                            {
                                static VTABLE: &ComponentVTable<Vec<$type>> = &ComponentVTable::construct("core", "<unknown>", |_, _| None);
                                unsafe { VTABLE.erase() }
                            },
                            ty.clone(),
                        ),
                        ty @ PrimitiveComponentType::[< Option $value >] => reg.register_with_primitive(
                            {
                                static VTABLE: &ComponentVTable<Option<$type>> = &ComponentVTable::construct("core", "<unknown>", |_, _| None);
                                unsafe { VTABLE.erase() }
                            },
                            ty.clone(),
                        ),
                    )*
                    PrimitiveComponentType::Vec { variants } => match **variants {
                        $(
                            PrimitiveComponentType::$value => reg.register_with_primitive(
                                {
                                    static VTABLE: &ComponentVTable::<Vec<$type>> = &ComponentVTable::construct("core", "<unknown>", |_,_| None);
                                    unsafe {
                                        VTABLE.erase()
                                    }
                                },
                                PrimitiveComponentType::[< Vec $value >]
                            ),
                            // PrimitiveComponentType::$value => reg.register2(key,
                            //     decorating,
                            //     Some(self.clone()),
                            //     Some(PrimitiveComponent::[<Vec $value>](Component::<Vec<$type>>::new_external(0)))
                            // ),
                        )*
                        _ => panic!("Unsupported Vec inner type: {:?}", variants),
                    },
                    PrimitiveComponentType::Option { variants } => match **variants {
                        $(
                            PrimitiveComponentType::$value => reg.register_with_primitive(
                                {
                                    static VTABLE: &ComponentVTable::<Option<$type>> = &ComponentVTable::construct("core", "<unknown>", |_,_| None);
                                    unsafe {
                                        VTABLE.erase()
                                    }
                                },
                                PrimitiveComponentType::[< Option $value >]
                            ),
                            // PrimitiveComponentType::$value => reg.register2(key,
                            //     decorating,
                            //     Some(self.clone()),
                            //     Some(PrimitiveComponent::[<Option $value>](Component::<Option<$type>>::new_external(0)))
                            // ),
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
