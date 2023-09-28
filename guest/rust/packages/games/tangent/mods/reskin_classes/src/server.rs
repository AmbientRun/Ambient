use std::collections::HashMap;

use ambient_api::{
    core::{messages::ModuleUnload, model::components::model_from_url},
    ecs::GeneralQuery,
    once_cell::sync::Lazy,
    prelude::*,
};
use packages::tangent_schema::{
    character::def::components::model_url,
    concepts::{CharacterDef, PlayerClass},
};

use crate::packages::tangent_schema::character::components::{def_ref, is_character};

#[main]
pub fn main() {
    let mut def_id_to_old_model_url = HashMap::new();

    let classes = query(PlayerClass::as_query()).build().evaluate();
    for (_id, class) in &classes {
        let Some(def) = CharacterDef::get_spawned(class.def_ref) else {
            continue;
        };

        let new_model_url = match class.name.as_str() {
            "Assault" => packages::this::assets::url("castle_guard_01.fbx"),
            "Tank" => packages::this::assets::url("Mutant.fbx"),
            _ => continue,
        };

        def_id_to_old_model_url.insert(class.def_ref, def.model_url);
        update_def(class.def_ref, &new_model_url);
    }

    ModuleUnload::subscribe(move |_| {
        for (def_id, old_model_url) in &def_id_to_old_model_url {
            update_def(*def_id, old_model_url);
        }
    });
}

fn update_def(def_id: EntityId, new_model_url: &str) {
    static ALL_CHARACTERS_WITH_DEFS: Lazy<GeneralQuery<Component<EntityId>>> =
        Lazy::new(|| query(def_ref()).requires(is_character()).build());

    entity::set_component(def_id, model_url(), new_model_url.to_string());
    for (character_id, character_def_id) in ALL_CHARACTERS_WITH_DEFS.evaluate() {
        if character_def_id == def_id {
            entity::set_component(character_id, model_from_url(), new_model_url.to_string());
        }
    }
}
