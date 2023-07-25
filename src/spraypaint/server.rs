// use ambient_api::{
//     components::core::{primitives::cube, transform::translation},
//     concepts::make_transformable,
//     physics,
//     prelude::*,
// };
use ambient_api::{
    components::core::{primitives::cube, rendering::decal_from_url, transform::translation},
    concepts::make_transformable,
    prelude::*,
};

#[main]

pub fn main() {
    println!("Spraypaint server started");
    messages::Spraypaint::subscribe(move |source, msg| {
        println!("Spray got");
        if let Some(hit) = physics::raycast_first(msg.origin, msg.dir) {
            println!("hiy {:?}", hit.position);
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
