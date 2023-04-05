/// A mapping from enum names to Rust types. Instantiate this with a macro that takes `$(($value:ident, $type:ty)),*`.
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
            (U8, u8),
            (U32, u32),
            (U64, u64),
            (Vec2, Vec2),
            (Vec3, Vec3),
            (Vec4, Vec4),
            (Uvec2, UVec2),
            (Uvec3, UVec3),
            (Uvec4, UVec4)
        );
    };
}
