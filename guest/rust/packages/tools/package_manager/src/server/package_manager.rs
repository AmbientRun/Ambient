use ambient_api::{core::package::components::enabled, prelude::*};

use crate::packages::this::messages::PackageSetEnabled;

pub fn main() {
    PackageSetEnabled::subscribe(|_, msg| {
        entity::set_component(msg.id, enabled(), msg.enabled);
    });
}
