
use ambient_ecs::World;

use ambient_core::{
    async_ecs::async_run,
    runtime,
};

use anyhow::Context;
use crate::shared::wit;

async fn fetch_url(url: &str) -> anyhow::Result<String> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

pub(crate) fn watch(
    world: &World,
    url: String,
) -> anyhow::Result<()> {

    println!("we get this in wit: {}", url);
    let runtime = world.resource(runtime()).clone();
    let async_run = world.resource(async_run()).clone();
    runtime.spawn(async move {
        let mut last_content = String::new();
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let content = reqwest::get(url.clone()).await.unwrap().text().await.unwrap();
            if content != last_content {
                last_content = content.clone();
                async_run.run(move |world| {
                    // println!("{:?}", &content);
                    let engine = rhai::Engine::new();
                    engine.run(&content).unwrap();
                });
            }
        }
    });
    Ok(())
}
