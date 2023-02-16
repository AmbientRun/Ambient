use kiwi_app::AppBuilder;
use kiwi_cameras::UICamera;
use kiwi_core::{asset_cache, camera::active_camera, runtime};
use kiwi_ecs::World;
use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use kiwi_gpu::std_assets::PixelTextureViewKey;
use kiwi_std::{asset_cache::SyncAssetKeyExt, color::Color};
use kiwi_ui::{FlowColumn, Image, Text, UIExt};

#[derive(Debug, Clone)]
struct Example;
impl ElementComponent for Example {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (_, set_k) = hooks.use_state(1.0);

        let assets = hooks.world.resource(asset_cache());

        let texture = PixelTextureViewKey::white().get(assets);
        let runtime = hooks.world.resource(runtime()).clone();
        hooks.use_memo_with((), move |_, _| {
            runtime.spawn(async move {
                log::info!("Spawning task");
                use kiwi_std::IntoDuration;
                tokio::time::sleep(5.secs()).await;
                set_k(5.0)
            });
        });

        // After 5 seconds, on rerender, this component crashes the app
        FlowColumn(vec![
            Image { texture: Some(texture) }.el().with_background(Color::rgba(1.0, 1.0, 0.0, 1.0)),
            Text::el("Hello, World!").with_background(Color::rgba(0.5, 0.0, 1.0, 1.0)),
        ])
        .el()
    }
}

fn init(world: &mut World) {
    Example.el().spawn_interactive(world);
    UICamera.el().set(active_camera(), 0.).spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().run_world(init);
}
