use ambient_api::prelude::*;

use packages::tangent_schema::concepts::{CharacterDef, PlayerClass};

#[main]
pub fn main() {
    PlayerClass {
        is_class: (),

        name: "Scout".to_string(),
        description: "Swift and elusive, ideal for hit-and-run tactics and recon missions."
            .to_string(),
        icon_url: packages::this::assets::url("icon.png"),
        def_ref: CharacterDef {
            max_health: 70.0,
            model_url: packages::this::assets::url("akai_e_espiritu.fbx"),
            speed: 0.12,
            run_speed_multiplier: 1.8,
            strafe_speed_multiplier: 0.9,
        }
        .spawn(),
    }
    .spawn();
}
