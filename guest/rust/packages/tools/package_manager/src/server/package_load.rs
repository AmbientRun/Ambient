use ambient_api::{core::package::components::name, package, prelude::*};

use crate::packages::this::messages::{PackageLoad, PackageLoadFailure, PackageLoadSuccess};

pub fn main() {
    PackageLoad::subscribe(|ctx, msg| {
        let Some(user_id) = ctx.client_user_id() else {
            return;
        };
        let maybe_url = msg.url.strip_suffix('/').unwrap_or(&msg.url).to_owned();
        let url = if !maybe_url.contains("http") {
            ambient_shared_types::urls::deployment_url(&maybe_url)
        } else {
            maybe_url
        };
        let url = if !url.ends_with("ambient.toml") {
            format!("{}/ambient.toml", url)
        } else {
            url
        };

        run_async(async move {
            match package::load(&url).await {
                Ok(id) => {
                    PackageLoadSuccess::new(
                        id,
                        entity::get_component(id, name()).unwrap_or_default(),
                    )
                    .send_client_targeted_reliable(user_id);
                }
                Err(err) => {
                    PackageLoadFailure::new(err.to_string()).send_client_targeted_reliable(user_id);
                }
            };
        });
    });
}
