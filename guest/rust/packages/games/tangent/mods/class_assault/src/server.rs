use ambient_api::prelude::*;

use packages::tangent_schema::concepts::{CharacterDef, PlayerClass};

#[main]
pub fn main() {
    PlayerClass {
        is_class: (),

        name: "Assault".to_string(),
        description: "A versatile choice for those who seek balance in speed, firepower, and maneuverability.".to_string(),
        icon_url: packages::this::assets::url("icon.png"),
        def_ref: CharacterDef { max_health: 100.0, model_url: packages::this::assets::url("castle_guard_01.fbx") }.spawn(),
    }
    .spawn();
}
