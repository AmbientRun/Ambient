use crate::{
    internal::{self, conversion::FromBindgen},
    prelude::EntityId,
};

#[doc(hidden)]
pub fn get_entity_for_package_id(package_id: String) -> Option<EntityId> {
    internal::wit::package_::get_entity_for_package_id(&package_id).from_bindgen()
}
