use elements_std::asset_url::{ObjectAssetType, ObjectRef, TypedAssetUrl};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use paste::paste;
use serde::{Deserialize, Serialize};

use crate::{Component, ComponentRegistry, EntityId, EntityUid, IComponent};

macro_rules! make_primitive_component {
    ($(($value:ident, $type:ty)),*) => { paste! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum PrimitiveComponent {
            $($value(Component<$type>)), *,
            $([<Vec $value>](Component<Vec<$type>>)), *,
            $([<Option $value>](Component<Option<$type>>)), *
        }

        #[derive(Clone, Serialize, Deserialize, Debug)]
        #[serde(tag = "type")]
        pub enum PrimitiveComponentType {
            $($value), *,
            Vec { variants: Box<PrimitiveComponentType> },
            Option { variants: Box<PrimitiveComponentType> },
        }

        impl PrimitiveComponent {
            fn new(component: &dyn IComponent) -> Option<Self> {
                $(if let Some(comp) = component.downcast_ref::<Component<$type>>() {
                    return Some(PrimitiveComponent::$value(comp.clone()));
                }) *
                $(if let Some(comp) = component.downcast_ref::<Component<Vec<$type>>>() {
                    return Some(PrimitiveComponent::[<Vec $value>](comp.clone()));
                }) *
                $(if let Some(comp) = component.downcast_ref::<Component<Option<$type>>>() {
                    return Some(PrimitiveComponent::[<Option $value>](comp.clone()));
                }) *
                None
            }

            pub fn as_component(&self) -> &dyn IComponent {
                match self {
                  $(Self::$value(c) => c,)*
                  $(Self::[<Vec $value>](c) => c,)*
                  $(Self::[<Option $value>](c) => c,)*
                }
            }
        }
        impl PrimitiveComponentType {
            pub(crate) fn register(&self, reg: &mut ComponentRegistry, key: &str) {
                match self {
                    $(
                        PrimitiveComponentType::$value => reg.register_with_id(key,
                            &mut Component::<$type>::new_external(0),
                            Some(self.clone()),
                            Some(PrimitiveComponent::$value(Component::<$type>::new_external(0)))
                        ),
                    )*
                    PrimitiveComponentType::Vec { variants } => match **variants {
                        $(
                            PrimitiveComponentType::$value => reg.register_with_id(key,
                                &mut Component::<Vec<$type>>::new_external(0),
                                Some(self.clone()),
                                Some(PrimitiveComponent::[<Vec $value>](Component::<Vec<$type>>::new_external(0)))
                            ),
                        )*
                        _ => panic!("Unsuported Vec inner type: {:?}", variants),
                    },
                    PrimitiveComponentType::Option { variants } => match **variants {
                        $(
                            PrimitiveComponentType::$value => reg.register_with_id(key,
                                &mut Component::<Option<$type>>::new_external(0),
                                Some(self.clone()),
                                Some(PrimitiveComponent::[<Option $value>](Component::<Option<$type>>::new_external(0)))
                            ),
                        )*
                        _ => panic!("Unsuported Vec inner type: {:?}", variants),
                    }
                }
            }
        }

        impl PartialEq<PrimitiveComponentType> for PrimitiveComponent {
            fn eq(&self, other: &PrimitiveComponentType) -> bool {
                match (self, other) {
                    $((Self::$value(_), PrimitiveComponentType::$value) => true,)*
                    (pc, PrimitiveComponentType::Vec { variants }) => match (pc, &**variants) {
                        $((Self::[< Vec $value >](_), PrimitiveComponentType::$value) => true,)*
                        _ => false,
                    },
                    (pc, PrimitiveComponentType::Option { variants }) => match (pc, &**variants) {
                        $((Self::[< Option $value >](_), PrimitiveComponentType::$value) => true,)*
                        _ => false,
                    },
                    _ => false,
                }
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
