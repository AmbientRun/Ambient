// use ambient_api::{
//     components::core::{primitives::cube, transform::translation},
//     concepts::make_transformable,
//     physics,
//     prelude::*,
// };
use ambient_api::{
    core::{
        rendering::components::decal_from_url,
        transform::{components::translation, concepts::make_transformable},
    },
    prelude::*,
};

use afps::afps_spraypaint::messages::Spraypaint;

#[main]

pub fn main() {
    println!("Spraypaint server started");
    Spraypaint::subscribe(move |_source, msg| {
        println!("Spray got");
        if let Some(hit) = physics::raycast_first(msg.origin, msg.dir) {
            // println!("hit {:?}", hit.position);
            let decal_url = asset::url("assets/spray/spray/pipeline.toml/0/mat.json").unwrap();

            Entity::new()
                .with_merge(make_transformable())
                .with(translation(), hit.position)
                // .with_default(cube())
                .with(decal_from_url(), decal_url)
                .spawn();
        }
    });
}
