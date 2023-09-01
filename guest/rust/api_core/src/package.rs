use crate::{
    internal::{self, conversion::FromBindgen},
    prelude::EntityId,
};

#[doc(hidden)]
pub fn get_entity_for_package_id(package_id: &str) -> Option<EntityId> {
    internal::wit::ambient_package::get_entity_for_package_id(package_id).from_bindgen()
}
