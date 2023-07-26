use ambient_api::{core::prefab::components::prefab_from_url, prelude::*};

#[main]
pub fn main() {
    let mut count = 0;
    run_async(async move {
        loop {
            let model = Entity::new()
                .with(prefab_from_url(), asset::url("assets/Teapot.glb").unwrap())
                .spawn();

            sleep(0.01).await;

            entity::despawn_recursive(model);
            count += 1;

            if count % 100 == 0 {
                println!("count: {}", count);
            }
        }
    });
}
