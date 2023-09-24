use ambient_api::prelude::*;

use packages::tangent_schema::concepts::PlayerClass;

#[main]
pub fn main() {
    PlayerClass {
        is_class: (),

        name: "Assault".to_string(),
        description: "A versatile choice for those who seek balance in speed, firepower, and maneuverability.".to_string(),
        icon_url: packages::this::assets::url("icon.png"),
    }
    .spawn();
}
