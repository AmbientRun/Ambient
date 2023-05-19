use std::sync::Arc;

use ambient_core::{asset_cache, gpu};
use ambient_ecs::{Entity, SystemGroup};
use ambient_element::{Element, ElementComponent};
use ambient_renderer::RenderTarget;
use ambient_rpc::RpcRegistry;
use ambient_std::Cb;
use glam::{uvec2, uvec4};
use url::Url;

use crate::{
    client::{GameClient, GameClientRenderTarget, LoadedFunc},
    client_game_state::ClientGameState,
    server::RpcArgs,
};

#[derive(Debug, Clone)]
pub struct GameClientView {
    /// The url to connect to
    pub url: Url,
    pub user_id: String,
    pub systems_and_resources: Cb<dyn Fn() -> (SystemGroup, Entity) + Sync + Send>,
    /// Invoked when the game client is loaded
    ///
    /// The returned function is executed when the client is disconnected
    pub on_loaded: LoadedFunc,
    pub create_rpc_registry: Cb<dyn Fn() -> RpcRegistry<RpcArgs> + Sync + Send>,
    pub inner: Element,
}

// Dock(vec![Text::el("Error").header_style(), Text::el(error)]).el()
impl ElementComponent for GameClientView {
    fn render(self: Box<Self>, hooks: &mut ambient_element::Hooks) -> Element {
        let Self {
            url,
            user_id,
            systems_and_resources,
            on_loaded,
            create_rpc_registry,
            inner,
        } = *self;

        let gpu = hooks.world.resource(gpu()).clone();

        hooks.provide_context(|| {
            GameClientRenderTarget(Arc::new(RenderTarget::new(gpu.clone(), uvec2(1, 1), None)))
        });
        let (render_target, _) = hooks.consume_context::<GameClientRenderTarget>().unwrap();

        let assets = hooks.world.resource(asset_cache()).clone();
        let game_state = hooks.use_ref_with(|world| {
            let (systems, resources) = systems_and_resources();

            ClientGameState::new(
                world,
                assets.clone(),
                user_id.clone(),
                render_target.0.clone(),
                systems,
                resources,
            )
        });

        // The game client will be set once a connection establishes
        let (game_client, set_game_client) = hooks.use_state(None as Option<GameClient>);

        todo!()
    }
}
