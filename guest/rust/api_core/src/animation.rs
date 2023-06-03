use std::{cell::RefCell, rc::Rc};

use crate::{
    components::core::{
        animation::{
            animation_graph, apply_base_pose, blend, clip_duration, freeze_at_percentage,
            freeze_at_time, mask_bind_ids, mask_weights, ref_count, retarget_animation_scaled,
            retarget_model_from_url, start_time,
        },
        app::name,
        ecs::{children, parent},
    },
    entity::{
        add_component, despawn_recursive, get_component, mutate_component, remove_component,
        set_component,
    },
    prelude::{block_until, time, Entity, EntityId},
};

/// tmp
#[derive(Debug, Clone, Copy)]
pub struct AnimationGraph(pub EntityId);
impl AnimationGraph {
    /// tmp
    pub fn new(root: impl AsRef<AnimationNode>) -> Self {
        let root: &AnimationNode = root.as_ref();
        let graph = Entity::new()
            .with_default(animation_graph())
            .with(children(), vec![root.0])
            .with(name(), "Animation graph".to_string())
            .spawn();
        add_component(root.0, parent(), graph);
        Self(graph)
    }
    /// tmp
    fn root(&self) -> Option<EntityId> {
        if let Some(children) = get_component(self.0, children()) {
            children.get(0).map(|x| *x)
        } else {
            None
        }
    }
    /// tmp
    pub fn set_root(&self, new_root: impl AsRef<AnimationNode>) {
        if let Some(root) = self.root() {
            remove_component(root, parent());
        }
        let new_root: &AnimationNode = new_root.as_ref();
        add_component(self.0, children(), vec![new_root.0]);
        add_component(new_root.0, parent(), self.0);
    }
    /// tmp
    pub fn set_retargeting(&self, retargeting: AnimationRetargeting) {
        match retargeting {
            AnimationRetargeting::None => {
                remove_component(self.0, retarget_model_from_url());
                return;
            }
            AnimationRetargeting::Skeleton { model_url } => {
                remove_component(self.0, retarget_animation_scaled());
                add_component(self.0, retarget_model_from_url(), model_url);
            }
            AnimationRetargeting::AnimationScaled {
                normalize_hip,
                model_url,
            } => {
                add_component(self.0, retarget_animation_scaled(), normalize_hip);
                add_component(self.0, retarget_model_from_url(), model_url);
            }
        }
    }
}
/// tmp
#[derive(Debug)]
pub struct AnimationNode(pub EntityId);
impl Clone for AnimationNode {
    fn clone(&self) -> Self {
        mutate_component(self.0, ref_count(), |x| *x += 1);
        Self(self.0.clone())
    }
}
impl Drop for AnimationNode {
    fn drop(&mut self) {
        mutate_component(self.0, ref_count(), |x| *x -= 1);
    }
}

/// tmp
#[derive(Debug)]
pub struct PlayClipFromUrlNode(pub AnimationNode);
impl PlayClipFromUrlNode {
    /// tmp
    pub fn new(url: impl Into<String>, looping: bool) -> Self {
        use crate::components::core::animation;
        let node = Entity::new()
            .with(animation::play_clip_from_url(), url.into())
            .with(name(), "Play clip from url".to_string())
            .with(animation::looping(), looping)
            .with(start_time(), time())
            .with(ref_count(), 1)
            .spawn();
        Self(AnimationNode(node))
    }
    /// Freeze the animation at time
    pub fn freeze_at_time(&self, time: f32) {
        add_component(self.0 .0, freeze_at_time(), time);
    }
    /// Freeze the animation at time = percentage * duration
    pub fn freeze_at_percentage(&self, percentage: f32) {
        add_component(self.0 .0, freeze_at_percentage(), percentage);
    }
    /// Returns None if the duration hasn't been loaded yet
    pub fn peek_clip_duration(&self) -> Option<f32> {
        get_component(self.0 .0, clip_duration())
    }
    /// If true, the base pose from the model of the animation clip will be applied to the animation
    ///
    /// Some animations will only work if the base pose of the character is the same as
    /// the animations base pose, so we apply the pose from the animations model to make sure they
    /// correspond
    ///
    /// This exists mostly because some FBX animations have pre-rotations, and to apply them to
    /// character models which don't have the same pre-rotations we need to make sure they're up to sync
    ///
    /// I.e. this is mostly relevant for retargeting
    pub fn apply_base_pose(&self, value: bool) {
        if value {
            add_component(self.0 .0, apply_base_pose(), ());
        } else {
            remove_component(self.0 .0, apply_base_pose());
        }
    }
    /// Returns the duration of this clip. This is async because it needs to wait for the clip to load before the duration can be returned.
    pub async fn clip_duration(&self) -> f32 {
        let res = Rc::new(RefCell::new(0.));
        {
            let res = res.clone();
            block_until(move || match self.peek_clip_duration() {
                Some(val) => {
                    *res.borrow_mut() = val;
                    true
                }
                None => false,
            })
            .await;
        }
        let val: f32 = *res.borrow();
        val
    }
}
impl AsRef<AnimationNode> for PlayClipFromUrlNode {
    fn as_ref(&self) -> &AnimationNode {
        &self.0
    }
}

/// tmp
#[derive(Debug, Clone)]
pub struct BlendNode(pub AnimationNode);
impl BlendNode {
    /// tmp
    pub fn new(
        left: impl AsRef<AnimationNode>,
        right: impl AsRef<AnimationNode>,
        weight: f32,
    ) -> Self {
        use crate::components::core::animation;
        let left: &AnimationNode = left.as_ref();
        let right: &AnimationNode = right.as_ref();
        let node = Entity::new()
            .with(animation::blend(), weight)
            .with(name(), "Blend".to_string())
            .with(children(), vec![left.0, right.0])
            .with(ref_count(), 1)
            .spawn();
        add_component(left.0, parent(), node);
        add_component(right.0, parent(), node);
        Self(AnimationNode(node))
    }
    /// tmp
    pub fn set_weight(&self, weight: f32) {
        set_component(self.0 .0, blend(), weight);
    }
    /// Sets the mask to a list of (bind_id, weights)
    pub fn set_mask(&self, weights: Vec<(String, f32)>) {
        let (bind_ids, weights): (Vec<_>, Vec<_>) = weights.into_iter().unzip();
        add_component(self.0 .0, mask_bind_ids(), bind_ids);
        add_component(self.0 .0, mask_weights(), weights);
    }
}
impl AsRef<AnimationNode> for BlendNode {
    fn as_ref(&self) -> &AnimationNode {
        &self.0
    }
}

/// tmp
#[derive(Debug, Clone)]
pub enum AnimationRetargeting {
    /// Bone Translation comes from the animation data, unchanged.
    None,
    /// Bone Translation comes from the Target Skeleton's bind pose.
    Skeleton {
        /// tmp
        model_url: String,
    },
    /// Bone translation comes from the animation data, but is scaled by the Skeleton's proportions.
    /// This is the ratio between the bone length of the Target Skeleton (the skeleton the animation
    /// is being played on), and the Source Skeleton (the skeleton the animation was authored for).
    AnimationScaled {
        /// Rotates the Hips bone based on the difference between the rotation the animation models root and the retarget animations root
        normalize_hip: bool,
        /// tmp
        model_url: String,
    },
}
impl Default for AnimationRetargeting {
    fn default() -> Self {
        Self::None
    }
}
