use rustdoc_types as rdt;

pub trait ItemHelpers {
    fn to_struct(&self) -> Option<&rdt::Struct>;
    fn to_impl(&self) -> Option<&rdt::Impl>;
    fn to_constant(&self) -> Option<&rdt::Constant>;
    fn to_struct_field_type(&self) -> Option<&rdt::Type>;
    fn to_assoc_const(&self) -> Option<&str>;
    fn to_variant(&self) -> Option<&rdt::Variant>;
}
impl ItemHelpers for rdt::ItemEnum {
    fn to_struct(&self) -> Option<&rdt::Struct> {
        match self {
            rdt::ItemEnum::Struct(s) => Some(s),
            _ => None,
        }
    }
    fn to_impl(&self) -> Option<&rdt::Impl> {
        match self {
            rdt::ItemEnum::Impl(s) => Some(s),
            _ => None,
        }
    }
    fn to_constant(&self) -> Option<&rdt::Constant> {
        match self {
            rdt::ItemEnum::Constant(s) => Some(s),
            _ => None,
        }
    }
    fn to_struct_field_type(&self) -> Option<&rdt::Type> {
        match self {
            rdt::ItemEnum::StructField(s) => Some(s),
            _ => None,
        }
    }
    fn to_assoc_const(&self) -> Option<&str> {
        match self {
            rdt::ItemEnum::AssocConst { default, .. } => default.as_deref(),
            _ => None,
        }
    }
    fn to_variant(&self) -> Option<&rdt::Variant> {
        match self {
            rdt::ItemEnum::Variant(s) => Some(s),
            _ => None,
        }
    }
}
