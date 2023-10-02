use ambient_api::prelude::*;

use packages::tangent_schema::concepts::{CharacterDef, PlayerClass};

#[main]
pub fn main() {
    PlayerClass {
        is_class: (),

        name: "Tank".to_string(),
        description: "A juggernaut on the battlefield, built to withstand punishment and deal massive damage.".to_string(),
        icon_url: packages::this::assets::url("icon.png"),
        def_ref: CharacterDef {
            max_health: 150.0,
            model_url: packages::this::assets::url("Ch03_nonPBR.fbx"),
            speed: 0.04,
            run_speed_multiplier: 1.2,
            strafe_speed_multiplier: 0.6,
        }
        .spawn(),
    }
    .spawn();
}
