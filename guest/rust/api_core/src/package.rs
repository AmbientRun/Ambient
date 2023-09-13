use crate::{
    internal::{self, conversion::FromBindgen},
    prelude::EntityId,
};

#[doc(hidden)]
pub fn get_entity_for_package_id(package_id: &str) -> Option<EntityId> {
    internal::wit::ambient_package::get_entity_for_package_id(package_id).from_bindgen()
}

#[cfg(feature = "server")]
/// Load a package from a URL.
pub async fn load(url: &str) -> anyhow::Result<EntityId> {
    use crate::{
        internal::generated::ambient_core::package::messages::{
            PackageLoadFailure, PackageLoadSuccess,
        },
        prelude::wait_for_fallible_runtime_messages,
    };

    internal::wit::server_ambient_package::load(url);
    let res = wait_for_fallible_runtime_messages::<PackageLoadSuccess, PackageLoadFailure>(
        {
            let url = url.to_owned();
            move |s| s.url == url
        },
        {
            let url = url.to_owned();
            move |f| f.url == url
        },
    )
    .await;

    match res {
        Ok(s) => Ok(s.package),
        Err(e) => Err(anyhow::Error::msg(e.reason)),
    }
}
