use std::collections::HashMap;

use crate::internal::host;

// aaaaaaarghhhhhhh
// wit-bindgen generates borrowing types so I need to define my owned types for the borrowing types to borrow from
// really looking forward to not having to do this!

pub(super) enum ComponentListType<'a> {
    TypeEmpty(Vec<()>),
    TypeBool(Vec<bool>),
    TypeEntityId(Vec<host::EntityId>),
    TypeF32(Vec<f32>),
    TypeF64(Vec<f64>),
    TypeMat4(Vec<host::Mat4>),
    TypeI32(Vec<i32>),
    TypeQuat(Vec<host::Quat>),
    TypeString(Vec<&'a str>),
    TypeU32(Vec<u32>),
    TypeU64(Vec<u64>),
    TypeVec2(Vec<host::Vec2>),
    TypeVec3(Vec<host::Vec3>),
    TypeVec4(Vec<host::Vec4>),
    TypeObjectRef(Vec<host::ObjectRefParam<'a>>),
}
impl<'a> ComponentListType<'a> {
    fn as_main(&'a self) -> host::ComponentListTypeParam<'a> {
        match self {
            Self::TypeEmpty(v) => host::ComponentListTypeParam::TypeEmpty(v.as_slice()),
            Self::TypeBool(v) => host::ComponentListTypeParam::TypeBool(v.as_slice()),
            Self::TypeEntityId(v) => host::ComponentListTypeParam::TypeEntityId(v.as_slice()),
            Self::TypeF32(v) => host::ComponentListTypeParam::TypeF32(v.as_slice()),
            Self::TypeF64(v) => host::ComponentListTypeParam::TypeF64(v.as_slice()),
            Self::TypeMat4(v) => host::ComponentListTypeParam::TypeMat4(v.as_slice()),
            Self::TypeI32(v) => host::ComponentListTypeParam::TypeI32(v.as_slice()),
            Self::TypeQuat(v) => host::ComponentListTypeParam::TypeQuat(v.as_slice()),
            Self::TypeString(v) => host::ComponentListTypeParam::TypeString(v.as_slice()),
            Self::TypeU32(v) => host::ComponentListTypeParam::TypeU32(v.as_slice()),
            Self::TypeU64(v) => host::ComponentListTypeParam::TypeU64(v.as_slice()),
            Self::TypeVec2(v) => host::ComponentListTypeParam::TypeVec2(v.as_slice()),
            Self::TypeVec3(v) => host::ComponentListTypeParam::TypeVec3(v.as_slice()),
            Self::TypeVec4(v) => host::ComponentListTypeParam::TypeVec4(v.as_slice()),
            Self::TypeObjectRef(v) => host::ComponentListTypeParam::TypeObjectRef(v.as_slice()),
        }
    }
}

pub(super) enum ComponentOptionType<'a> {
    TypeEmpty(Option<()>),
    TypeBool(Option<bool>),
    TypeEntityId(Option<host::EntityId>),
    TypeF32(Option<f32>),
    TypeF64(Option<f64>),
    TypeMat4(Option<host::Mat4>),
    TypeI32(Option<i32>),
    TypeQuat(Option<host::Quat>),
    TypeString(Option<&'a str>),
    TypeU32(Option<u32>),
    TypeU64(Option<u64>),
    TypeVec2(Option<host::Vec2>),
    TypeVec3(Option<host::Vec3>),
    TypeVec4(Option<host::Vec4>),
    TypeObjectRef(Option<host::ObjectRefParam<'a>>),
}
impl<'a> ComponentOptionType<'a> {
    fn as_main(&self) -> host::ComponentOptionTypeParam<'a> {
        match self {
            Self::TypeEmpty(v) => host::ComponentOptionTypeParam::TypeEmpty(*v),
            Self::TypeBool(v) => host::ComponentOptionTypeParam::TypeBool(*v),
            Self::TypeEntityId(v) => host::ComponentOptionTypeParam::TypeEntityId(*v),
            Self::TypeF32(v) => host::ComponentOptionTypeParam::TypeF32(*v),
            Self::TypeF64(v) => host::ComponentOptionTypeParam::TypeF64(*v),
            Self::TypeMat4(v) => host::ComponentOptionTypeParam::TypeMat4(*v),
            Self::TypeI32(v) => host::ComponentOptionTypeParam::TypeI32(*v),
            Self::TypeQuat(v) => host::ComponentOptionTypeParam::TypeQuat(*v),
            Self::TypeString(v) => host::ComponentOptionTypeParam::TypeString(*v),
            Self::TypeU32(v) => host::ComponentOptionTypeParam::TypeU32(*v),
            Self::TypeU64(v) => host::ComponentOptionTypeParam::TypeU64(*v),
            Self::TypeVec2(v) => host::ComponentOptionTypeParam::TypeVec2(*v),
            Self::TypeVec3(v) => host::ComponentOptionTypeParam::TypeVec3(*v),
            Self::TypeVec4(v) => host::ComponentOptionTypeParam::TypeVec4(*v),
            Self::TypeObjectRef(v) => host::ComponentOptionTypeParam::TypeObjectRef(v.clone()),
        }
    }
}

pub(super) enum ComponentType<'a> {
    TypeEmpty(()),
    TypeBool(bool),
    TypeEntityId(host::EntityId),
    TypeF32(f32),
    TypeF64(f64),
    TypeMat4(host::Mat4),
    TypeI32(i32),
    TypeQuat(host::Quat),
    TypeString(String),
    TypeU32(u32),
    TypeU64(u64),
    TypeVec2(host::Vec2),
    TypeVec3(host::Vec3),
    TypeVec4(host::Vec4),
    TypeObjectRef(host::ObjectRefParam<'a>),
    TypeList(ComponentListType<'a>),
    TypeOption(ComponentOptionType<'a>),
}

pub(super) fn create_owned_types(
    data: &HashMap<u32, host::ComponentTypeResult>,
) -> Vec<(u32, ComponentType)> {
    data.iter()
        .map(|(id, component)| {
            (
                *id,
                match component {
                    host::ComponentTypeResult::TypeEmpty(_) => ComponentType::TypeEmpty(()),
                    host::ComponentTypeResult::TypeBool(v) => ComponentType::TypeBool(*v),
                    host::ComponentTypeResult::TypeEntityId(v) => ComponentType::TypeEntityId(*v),
                    host::ComponentTypeResult::TypeF32(v) => ComponentType::TypeF32(*v),
                    host::ComponentTypeResult::TypeF64(v) => ComponentType::TypeF64(*v),
                    host::ComponentTypeResult::TypeMat4(v) => ComponentType::TypeMat4(*v),
                    host::ComponentTypeResult::TypeI32(v) => ComponentType::TypeI32(*v),
                    host::ComponentTypeResult::TypeQuat(v) => ComponentType::TypeQuat(*v),
                    host::ComponentTypeResult::TypeString(v) => {
                        ComponentType::TypeString(v.clone())
                    }
                    host::ComponentTypeResult::TypeU32(v) => ComponentType::TypeU32(*v),
                    host::ComponentTypeResult::TypeU64(v) => ComponentType::TypeU64(*v),
                    host::ComponentTypeResult::TypeVec2(v) => ComponentType::TypeVec2(*v),
                    host::ComponentTypeResult::TypeVec3(v) => ComponentType::TypeVec3(*v),
                    host::ComponentTypeResult::TypeVec4(v) => ComponentType::TypeVec4(*v),
                    host::ComponentTypeResult::TypeObjectRef(v) => {
                        ComponentType::TypeObjectRef(host::ObjectRefParam { id: &v.id })
                    }
                    host::ComponentTypeResult::TypeList(v) => ComponentType::TypeList(match v {
                        host::ComponentListTypeResult::TypeEmpty(v) => {
                            ComponentListType::TypeEmpty(v.clone())
                        }
                        host::ComponentListTypeResult::TypeBool(v) => {
                            ComponentListType::TypeBool(v.clone())
                        }
                        host::ComponentListTypeResult::TypeEntityId(v) => {
                            ComponentListType::TypeEntityId(v.clone())
                        }
                        host::ComponentListTypeResult::TypeF32(v) => {
                            ComponentListType::TypeF32(v.clone())
                        }
                        host::ComponentListTypeResult::TypeF64(v) => {
                            ComponentListType::TypeF64(v.clone())
                        }
                        host::ComponentListTypeResult::TypeMat4(v) => {
                            ComponentListType::TypeMat4(v.clone())
                        }
                        host::ComponentListTypeResult::TypeI32(v) => {
                            ComponentListType::TypeI32(v.clone())
                        }
                        host::ComponentListTypeResult::TypeQuat(v) => {
                            ComponentListType::TypeQuat(v.clone())
                        }
                        host::ComponentListTypeResult::TypeString(v) => {
                            ComponentListType::TypeString(v.iter().map(|v| v.as_str()).collect())
                        }
                        host::ComponentListTypeResult::TypeU32(v) => {
                            ComponentListType::TypeU32(v.clone())
                        }
                        host::ComponentListTypeResult::TypeU64(v) => {
                            ComponentListType::TypeU64(v.clone())
                        }
                        host::ComponentListTypeResult::TypeVec2(v) => {
                            ComponentListType::TypeVec2(v.clone())
                        }
                        host::ComponentListTypeResult::TypeVec3(v) => {
                            ComponentListType::TypeVec3(v.clone())
                        }
                        host::ComponentListTypeResult::TypeVec4(v) => {
                            ComponentListType::TypeVec4(v.clone())
                        }
                        host::ComponentListTypeResult::TypeObjectRef(v) => {
                            ComponentListType::TypeObjectRef(
                                v.iter()
                                    .map(|v| host::ObjectRefParam { id: &v.id })
                                    .collect(),
                            )
                        }
                    }),
                    host::ComponentTypeResult::TypeOption(v) => {
                        ComponentType::TypeOption(match v {
                            host::ComponentOptionTypeResult::TypeEmpty(v) => {
                                ComponentOptionType::TypeEmpty(*v)
                            }
                            host::ComponentOptionTypeResult::TypeBool(v) => {
                                ComponentOptionType::TypeBool(*v)
                            }
                            host::ComponentOptionTypeResult::TypeEntityId(v) => {
                                ComponentOptionType::TypeEntityId(*v)
                            }
                            host::ComponentOptionTypeResult::TypeF32(v) => {
                                ComponentOptionType::TypeF32(*v)
                            }
                            host::ComponentOptionTypeResult::TypeF64(v) => {
                                ComponentOptionType::TypeF64(*v)
                            }
                            host::ComponentOptionTypeResult::TypeMat4(v) => {
                                ComponentOptionType::TypeMat4(*v)
                            }
                            host::ComponentOptionTypeResult::TypeI32(v) => {
                                ComponentOptionType::TypeI32(*v)
                            }
                            host::ComponentOptionTypeResult::TypeQuat(v) => {
                                ComponentOptionType::TypeQuat(*v)
                            }
                            host::ComponentOptionTypeResult::TypeString(v) => {
                                ComponentOptionType::TypeString(v.as_deref())
                            }
                            host::ComponentOptionTypeResult::TypeU32(v) => {
                                ComponentOptionType::TypeU32(*v)
                            }
                            host::ComponentOptionTypeResult::TypeU64(v) => {
                                ComponentOptionType::TypeU64(*v)
                            }
                            host::ComponentOptionTypeResult::TypeVec2(v) => {
                                ComponentOptionType::TypeVec2(*v)
                            }
                            host::ComponentOptionTypeResult::TypeVec3(v) => {
                                ComponentOptionType::TypeVec3(*v)
                            }
                            host::ComponentOptionTypeResult::TypeVec4(v) => {
                                ComponentOptionType::TypeVec4(*v)
                            }
                            host::ComponentOptionTypeResult::TypeObjectRef(v) => {
                                ComponentOptionType::TypeObjectRef(
                                    v.as_ref().map(|v| host::ObjectRefParam { id: &v.id }),
                                )
                            }
                        })
                    }
                },
            )
        })
        .collect()
}

pub(super) fn create_borrowed_types<'a>(
    data: &'a [(u32, ComponentType)],
) -> Vec<(u32, host::ComponentTypeParam<'a>)> {
    data.iter()
        .map(|(id, ct)| {
            (
                *id,
                match ct {
                    ComponentType::TypeEmpty(_) => host::ComponentTypeParam::TypeEmpty(()),
                    ComponentType::TypeBool(v) => host::ComponentTypeParam::TypeBool(*v),
                    ComponentType::TypeEntityId(v) => host::ComponentTypeParam::TypeEntityId(*v),
                    ComponentType::TypeF32(v) => host::ComponentTypeParam::TypeF32(*v),
                    ComponentType::TypeF64(v) => host::ComponentTypeParam::TypeF64(*v),
                    ComponentType::TypeMat4(v) => host::ComponentTypeParam::TypeMat4(*v),
                    ComponentType::TypeI32(v) => host::ComponentTypeParam::TypeI32(*v),
                    ComponentType::TypeQuat(v) => host::ComponentTypeParam::TypeQuat(*v),
                    ComponentType::TypeString(v) => {
                        host::ComponentTypeParam::TypeString(v.as_str())
                    }
                    ComponentType::TypeU32(v) => host::ComponentTypeParam::TypeU32(*v),
                    ComponentType::TypeU64(v) => host::ComponentTypeParam::TypeU64(*v),
                    ComponentType::TypeVec2(v) => host::ComponentTypeParam::TypeVec2(*v),
                    ComponentType::TypeVec3(v) => host::ComponentTypeParam::TypeVec3(*v),
                    ComponentType::TypeVec4(v) => host::ComponentTypeParam::TypeVec4(*v),
                    ComponentType::TypeObjectRef(v) => {
                        host::ComponentTypeParam::TypeObjectRef(host::ObjectRefParam { id: v.id })
                    }
                    ComponentType::TypeList(v) => host::ComponentTypeParam::TypeList(v.as_main()),
                    ComponentType::TypeOption(v) => {
                        host::ComponentTypeParam::TypeOption(v.as_main())
                    }
                },
            )
        })
        .collect()
}
