use std::sync::Arc;

use ambient_core::transform::{rotation, translation};
use ambient_editor_derive::ElementEditor;
use ambient_model::{Model, ModelFromUrl};
use ambient_std::{
    asset_cache::{AssetCache, AssetKeepalive, AsyncAssetKey, AsyncAssetKeyExt},
    asset_url::{AnimationAssetType, ModelAssetType, TypedAssetUrl},
    download_asset::AssetError,
};
use anyhow::Context;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{AnimationClip, AnimationClipFromUrl, AnimationOutputs, AnimationTrack};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ElementEditor)]
pub enum AnimationRetargeting {
    /// Bone Translation comes from the animation data, unchanged.
    None,
    /// Bone Translation comes from the Target Skeleton's bind pose.
    Skeleton,
    /// Bone translation comes from the animation data, but is scaled by the Skeleton's proportions.
    /// This is the ratio between the bone length of the Target Skeleton (the skeleton the animation
    /// is being played on), and the Source Skeleton (the skeleton the animation was authored for).
    AnimationScaled {
        /// Rotates the Hips bone based on the difference between the rotation the animation models root and the retarget animations root
        normalize_hip: bool,
    },
}
impl Default for AnimationRetargeting {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ElementEditor)]
pub struct AnimationClipRetargetedFromModel {
    pub clip: TypedAssetUrl<AnimationAssetType>,
    pub translation_retargeting: AnimationRetargeting,
    pub retarget_model: Option<TypedAssetUrl<ModelAssetType>>,
}
#[async_trait]
impl AsyncAssetKey<Result<Arc<AnimationClip>, AssetError>> for AnimationClipRetargetedFromModel {
    fn keepalive(&self) -> AssetKeepalive {
        // TODO(fred): We _could_ have a timeout here, but it looks weird when animations are loading
        // so just keeping them forever for now, and since they are only peeked, the keepalive timeout
        // wouldn't refresh, so once per hour we'd always have that "bug"
        AssetKeepalive::Forever
    }
    async fn load(self, assets: AssetCache) -> Result<Arc<AnimationClip>, AssetError> {
        let clip_url: TypedAssetUrl<AnimationAssetType> =
            self.clip.abs().context(format!("Expected absolute url, got: {}", self.clip))?.into();
        let anim_model =
            ModelFromUrl(clip_url.model_crate().context("Invalid clip url")?.model()).get(&assets).await.context("Failed to load model")?;
        let clip = AnimationClipFromUrl::new(clip_url.unwrap_abs(), true).get(&assets).await.context("No such clip")?;
        match self.translation_retargeting {
            AnimationRetargeting::None => Ok(clip),
            AnimationRetargeting::Skeleton => {
                let mut clip = (*clip).clone();
                clip.tracks.retain(|track| track.outputs.component() != translation());
                Ok(Arc::new(clip))
            }
            AnimationRetargeting::AnimationScaled { normalize_hip } => {
                let retarget_model_url =
                    self.retarget_model.context("No retarget_model specified")?.abs().context(format!("Failed to resolve retarget url"))?;
                let retarget_model = ModelFromUrl(retarget_model_url.into()).get(&assets).await.context("Failed to load retarget model")?;
                let mut clip = (*clip).clone();
                let anim_root = anim_model.roots()[0];
                let _retarget_root = retarget_model.roots()[0];
                let anim_root_rot = anim_model.0.get(anim_root, rotation()).unwrap_or_default();
                let retarget_root_rot = retarget_model.0.get(anim_root, rotation()).unwrap_or_default();
                clip.tracks.retain_mut(|track| {
                    if normalize_hip && track.target.bind_id() == Some("Hips") {
                        let zup = retarget_root_rot.inverse() * anim_root_rot;

                        if track.outputs.component() == rotation() {
                            if let AnimationOutputs::Quat { data, .. } = &mut track.outputs {
                                for v in data {
                                    *v = zup * *v;
                                }
                            }
                        } else if track.outputs.component() == translation() {
                            if let AnimationOutputs::Vec3 { data, .. } = &mut track.outputs {
                                for v in data {
                                    *v = zup * *v;
                                }
                            }
                        }
                    }
                    if track.outputs.component() == translation() {
                        retarget_track(track, &anim_model, &retarget_model).is_some()
                    } else {
                        true
                    }
                });
                Ok(Arc::new(clip))
            }
        }
    }
}
fn retarget_track(track: &mut AnimationTrack, anim_model: &Model, retarget_model: &Model) -> Option<()> {
    let bind_id = track.target.bind_id().unwrap();
    let original = anim_model.get_entity_id_by_bind_id(bind_id).unwrap();
    let target = retarget_model.get_entity_id_by_bind_id(bind_id)?;
    let original = anim_model.0.get(original, translation()).unwrap().length();
    let target = anim_model.0.get(target, translation()).ok()?.length();
    if target == 0. {
        return Some(());
    }
    let scale = target / original;
    match &mut track.outputs {
        AnimationOutputs::Vec3 { data, .. } => {
            for v in data.iter_mut() {
                *v *= scale;
            }
        }
        AnimationOutputs::Quat { .. } => unreachable!(),
        AnimationOutputs::Vec3Field { data, .. } => {
            for v in data.iter_mut() {
                *v *= scale;
            }
        }
    }
    Some(())
}
