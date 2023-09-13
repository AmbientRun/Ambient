use ambient_api::{core::package::components::name, package, prelude::*};

use crate::packages::this::messages::{PackageLoad, PackageLoadFailure, PackageLoadSuccess};

pub fn main() {
    PackageLoad::subscribe(|ctx, msg| {
        let Some(user_id) = ctx.client_user_id() else {
            return;
        };
        let url = msg.url.strip_suffix('/').unwrap_or(&msg.url).to_owned();
        run_async(async move {
            match dbg!(package::load(dbg!(&url)).await) {
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
