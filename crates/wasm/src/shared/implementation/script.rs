
use ambient_ecs::{World, Entity};
use ambient_primitives::{cube};
use ambient_core::transform::translation;
use std::sync::Arc;
use parking_lot::Mutex;
use ambient_core::{
    async_ecs::async_run,
    runtime,
};
use glam::{vec3};
use anyhow::Context;
use crate::shared::wit;

async fn fetch_url(url: &str) -> anyhow::Result<String> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

pub(crate) fn watch(
    world: Arc<Mutex<&World>>,
    url: String,
) -> anyhow::Result<()> {

    println!("we get this in wit: {}", url);
    let runtime = world.lock().resource(runtime()).clone();
    let async_run = world.lock().resource(async_run()).clone();
    // let world_arc = Arc::new(Mutex::new(world));
    // let world_arc_clone = Arc::clone(&world_arc);
    let world_arc_clone = Arc::clone(&world);

    // UNSAFE: we are using `std::mem::transmute` to change the lifetime of `world_arc_clone`.
    // This is safe as long as the `World` instance is valid during the execution of the Rhai script.
    let world_arc_clone: Arc<Mutex<&'static mut World>> = unsafe { std::mem::transmute(world_arc_clone) };

    runtime.spawn(async move {
        let mut last_content = String::new();
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let content = reqwest::get(url.clone()).await.unwrap().text().await.unwrap();
            if content != last_content {
                last_content = content.clone();
                let world_arc_clone = Arc::clone(&world_arc_clone);
                async_run.run(move |_world| {
                    let mut engine = rhai::Engine::new();
                    let world_arc_1 = Arc::clone(&world_arc_clone);
                    engine.register_fn("entities", move || {
                        let world = Arc::clone(&world_arc_1);
                        let world = world.lock();
                        println!("{:?}", world.entities());
                    });
                    let world_arc_2 = Arc::clone(&world_arc_clone);
                    engine.register_fn("cube", move |x: f32, y: f32, z: f32| {
                        let world = Arc::clone(&world_arc_2);
                        let mut world = world.lock();
                        let entity = Entity::new().with_default(cube())
                        .with(translation(), vec3(x, y, z))
                        .spawn(&mut *world);
                        // world.add_component(entity_id, component, value);
                        // println!("{:?}", world.entities());
                    });

                    engine.run(&content).unwrap();
                });
            }
        }
    });
    Ok(())
}
