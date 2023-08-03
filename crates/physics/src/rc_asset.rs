use std::ops::Deref;

use ambient_native_std::asset_cache::Asset;
use physxx::PxReferenceCounted;

/// This is a wrapper for working with physx reference counted objects in the AssetCache.
///
/// Physx reference counted objects can be aquired in a number of ways from physx. In
/// order to know when to release them from the AssetCache, we simply check to see if
/// the reference count reaches 2; if that happens we know the AssetCache is the only
/// owner left (for internal reasons it's 2 rather than 1). At that point the asset will be
/// dropped.
#[derive(Clone)]
pub struct PxRcAsset<T: PxReferenceCounted>(pub T);
impl<T: PxReferenceCounted> Deref for PxRcAsset<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Clone)]
pub struct PxRcAssetWeak<T: PxReferenceCounted>(pub T);

impl<T: PxReferenceCounted + Sync + Send + Clone> Asset for PxRcAsset<T> {
    type WeakType = PxRcAssetWeak<T>;

    fn to_weak(strong: &Self) -> Self::WeakType {
        PxRcAssetWeak(strong.0.clone())
    }

    fn from_weak(weak: &Self::WeakType) -> Option<Self>
    where
        Self: Sized,
    {
        // Basically, if the asset cache is the only one holding on to this, then
        // we consider it dropped. It's 2 because the check_alive will keep its own
        // reference as well
        if weak.0.get_reference_count() > 2 {
            Some(PxRcAsset(weak.0.clone()))
        } else {
            None
        }
    }
}
