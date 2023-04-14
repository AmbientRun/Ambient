use ambient_core::asset_cache;
use ambient_input::{player_prev_raw_input, player_raw_input};
use ambient_network::client::game_client;
use ambient_std::asset_url::AbsAssetUrl;
use anyhow::Context;

use super::Bindings;
use crate::shared::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::message,
    wit,
};

use ambient_core::{
    camera::{get_active_camera, projection_view},
    main_scene,
    player::local_user_id,
};

use ambient_std::shapes::Ray;
use glam::Mat4;

impl wit::client_message::Host for Bindings {
    fn send(
        &mut self,
        target: wit::client_message::Target,
        name: String,
        data: Vec<u8>,
    ) -> anyhow::Result<()> {
        use wit::client_message::Target;
        let module_id = self.id;
        let world = self.world_mut();

        match target {
            Target::ServerUnreliable | Target::ServerReliable => {
                let connection = world
                    .resource(game_client())
                    .as_ref()
                    .context("no game client")?
                    .connection
                    .clone();

                message::send_networked(
                    world,
                    connection,
                    module_id,
                    &name,
                    &data,
                    matches!(target, Target::ServerReliable),
                )
            }
            Target::LocalBroadcast => message::send_local(world, module_id, None, name, data),
            Target::Local(id) => {
                message::send_local(world, module_id, Some(id.from_bindgen()), name, data)
            }
        }
    }
}
impl wit::client_player::Host for Bindings {
    fn get_raw_input(&mut self) -> anyhow::Result<wit::client_player::RawInput> {
        Ok(self
            .world()
            .resource(player_raw_input())
            .clone()
            .into_bindgen())
    }

    fn get_prev_raw_input(&mut self) -> anyhow::Result<wit::client_player::RawInput> {
        Ok(self
            .world()
            .resource(player_prev_raw_input())
            .clone()
            .into_bindgen())
    }
}

impl wit::asset::Host for Bindings {
    fn url(&mut self, path: String) -> anyhow::Result<Option<String>> {
        let assets = self.world().resource(asset_cache()).clone();
        let asset_url = AbsAssetUrl::from_asset_key(path);
        asset_url.to_download_url(&assets).map(|url| Some(url.to_string()))
    }
}

impl wit::audio::Host for Bindings {
    fn load(&mut self, url: String) -> anyhow::Result<()> {
        crate::shared::implementation::audio::load(self.world_mut(), url)
    }

    fn play(&mut self, name: String, looping: bool, amp: f32) -> anyhow::Result<()> {
        crate::shared::implementation::audio::play(self.world_mut(), name, looping, amp)
    }
}

impl wit::camera::Host for Bindings {
    fn screen_ray(&mut self, clip_space_pos: wit::types::Vec2) -> anyhow::Result<wit::types::Ray> {
        let user_id = self.world().resource_opt(local_user_id()).unwrap();
        let camera = get_active_camera(&self.world(), main_scene(), Some(user_id)).unwrap();
        let proj_view = self.world().get(camera, projection_view()).ok();
        let inv_proj_view = proj_view.unwrap_or(Mat4::IDENTITY).inverse();
        let a = inv_proj_view.project_point3(clip_space_pos.from_bindgen().extend(1.));
        let b = inv_proj_view.project_point3(clip_space_pos.from_bindgen().extend(0.9));
        let origin = a;
        let dir = (b - a).normalize();

        let ray = Ray {
            origin: origin,
            dir: dir,
        };
        Ok(ray.into_bindgen())
    }
}