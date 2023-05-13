
use ambient_ecs::{World, Entity, EntityId};
use ambient_primitives::{cube, quad};
use ambient_core::transform::{translation, scale};
use ambient_renderer::color;
use std::sync::Arc;
use parking_lot::Mutex;
use ambient_core::{
    async_ecs::async_run,
    runtime,
};
use glam::{Vec3, Vec4};
// use anyhow::Context;
// use crate::shared::wit;

// async fn fetch_url(url: &str) -> anyhow::Result<String> {
//     let response = reqwest::get(url).await?;
//     let text = response.text().await?;
//     Ok(text)
// }

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
struct RhaiEntityInfo {
    shape: String,
    translation: [f32; 3],
    scale: [f32; 3],
    color: [f32; 4],
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
        let created_entities = Arc::new(Mutex::new(Vec::<EntityId>::new()));
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let content = reqwest::get(url.clone()).await.unwrap().text().await.unwrap();
            if content != last_content {
                last_content = content.clone();
                let world_arc_clone = Arc::clone(&world_arc_clone);
                for entity_id in created_entities.lock().iter() {
                    let mut world = world_arc_clone.lock();
                    world.despawn(*entity_id);
                }
                created_entities.lock().clear();
                let created_entities_clone = Arc::clone(&created_entities);
                async_run.run(move |_world| {
                    let mut engine = rhai::Engine::new();
                    let world_arc_1 = Arc::clone(&world_arc_clone);

                    engine.register_fn("new_entity", move |info: rhai::Map| {
                        println!("entity: {:?}", info);
                        let mut entity = Entity::new();
                        let world = Arc::clone(&world_arc_1);
                        if let Some(shape) = info.get("shape") {
                            println!("shape: {:?}", shape);
                            let shape_str = shape.clone().into_string().unwrap();
                            match shape_str.as_str() {
                                "cube" => {
                                    entity = entity.with_default(cube());
                                },
                                "quad" => {
                                    entity = entity.with_default(quad());
                                },
                                // "ball" => {
                                //     entity = entity.with_default(ball());
                                // },
                                _ => {}
                            }
                        }
                        if let Some(_translation) = info.get("translation") {
                            println!("translation: {:?}", _translation);
                            let pos = _translation.clone().into_typed_array::<f32>().unwrap();
                            entity = entity.with(translation(), Vec3::from_slice(&pos));
                        }
                        if let Some(_scale) = info.get("scale") {
                            println!("scale: {:?}", _scale);
                            let s = _scale.clone().into_typed_array::<f32>().unwrap();
                            entity = entity.with(scale(), Vec3::from_slice(&s));
                        }
                        if let Some(_color) = info.get("color") {
                            println!("color: {:?}", _color);
                            let c = _color.clone().into_typed_array::<f32>().unwrap();
                            entity = entity.with(color(), Vec4::from_slice(&c));
                        }
                        let mut world = world.lock();
                        let id = entity.spawn(&mut *world);
                        created_entities_clone.lock().push(id);
                    });

                    match engine.run(&content) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("error: {:?}", e);
                        }
                    }
                });
            }
        }
    });
    Ok(())
}
