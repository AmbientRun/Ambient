use ambient_api::prelude::*;

use packages::tangent_schema::concepts::PlayerClass;

#[main]
pub fn main() {
    PlayerClass {
        is_class: (),

        name: "Tank".to_string(),
        description: "A juggernaut on the battlefield, built to withstand punishment and deal massive damage."
            .to_string(),
        icon_url: packages::this::assets::url("icon.png"),
    }
    .spawn();
}
