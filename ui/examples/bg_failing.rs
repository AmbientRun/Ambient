use elements::{app::App, cameras::UICamera, ecs::World};
use elements_core::{asset_cache, camera::active_camera, runtime};
use elements_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use elements_gpu::std_assets::{PixelTextureKey, PixelTextureViewKey};
use elements_std::{asset_cache::SyncAssetKeyExt, color::Color};
use elements_ui::{FlowColumn, Image, Text, UIExt};

#[derive(Debug, Clone)]
struct Example;
impl ElementComponent for Example {
    fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
        let (_, set_k) = hooks.use_state(1.0);

        let assets = world.resource(asset_cache());

        let texture = PixelTextureViewKey::white().get(assets);
        let runtime = world.resource(runtime());
        hooks.use_memo_with((), move || {
            runtime.spawn(async move {
                log::info!("Spawning task");
                use elements_std::IntoDuration;
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
    App::run_ui(init);
}
