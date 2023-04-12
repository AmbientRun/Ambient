use rustdoc_types::{
    Constant as RdConstant, Crate, Id, Impl as RdImpl, Item, ItemEnum, Path as RdPath,
    Struct as RdStruct, Type as RdType, Variant as RdVariant,
};

use super::{CRATES, PATH_TO_CRATE_AND_ID};

pub trait ItemHelpers {
    fn to_struct(&self) -> Option<&RdStruct>;
    fn to_impl(&self) -> Option<&RdImpl>;
    fn to_constant(&self) -> Option<&RdConstant>;
    fn to_struct_field_type(&self) -> Option<&RdType>;
    fn to_assoc_const(&self) -> Option<&str>;
    fn to_variant(&self) -> Option<&RdVariant>;
}
impl ItemHelpers for ItemEnum {
    fn to_struct(&self) -> Option<&RdStruct> {
        match self {
            ItemEnum::Struct(s) => Some(s),
            _ => None,
        }
    }
    fn to_impl(&self) -> Option<&RdImpl> {
        match self {
            ItemEnum::Impl(s) => Some(s),
            _ => None,
        }
    }
    fn to_constant(&self) -> Option<&RdConstant> {
        match self {
            ItemEnum::Constant(s) => Some(s),
            _ => None,
        }
    }
    fn to_struct_field_type(&self) -> Option<&RdType> {
        match self {
            ItemEnum::StructField(s) => Some(s),
            _ => None,
        }
    }
    fn to_assoc_const(&self) -> Option<&str> {
        match self {
            ItemEnum::AssocConst { default, .. } => default.as_deref(),
            _ => None,
        }
    }
    fn to_variant(&self) -> Option<&RdVariant> {
        match self {
            ItemEnum::Variant(s) => Some(s),
            _ => None,
        }
    }
}

pub trait ItemsHelpers {
    fn find_named_item<'a>(&self, krate: &'a Crate, name: &str) -> Option<&'a Item>;
}
impl ItemsHelpers for [Id] {
    fn find_named_item<'a>(&self, krate: &'a Crate, name: &str) -> Option<&'a Item> {
        self.iter().find_map(|i| {
            let item = i.get(krate);
            if item.name.as_deref() == Some(name) {
                Some(item)
            } else {
                None
            }
        })
    }
}

pub trait IdHelper {
    fn get<'a>(&self, krate: &'a Crate) -> &'a Item;
}
impl IdHelper for Id {
    fn get<'a>(&self, krate: &'a Crate) -> &'a Item {
        match krate.index.get(self) {
            Some(i) => i,
            None => panic!("invalid id: {self:?}"),
        }
    }
}

pub trait PathHelper {
    fn get<'a>(&self, krate: &'a Crate) -> (&'a Crate, &'a Item);
}
impl PathHelper for RdPath {
    fn get<'a>(&self, krate: &'a Crate) -> (&'a Crate, &'a Item) {
        match krate.index.get(&self.id) {
            Some(i) => (krate, i),
            None => {
                let path = krate.paths.get(&self.id).unwrap().path.join("::");
                if let Some((crate_path, id)) = PATH_TO_CRATE_AND_ID.get(&path) {
                    let krate = CRATES.get(crate_path).unwrap();
                    (krate, id.get(krate))
                } else {
                    panic!("invalid path: {self:?} {path:?}")
                }
            }
        }
    }
}
