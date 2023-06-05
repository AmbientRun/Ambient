use std::{cell::RefCell, rc::Rc};

use crate::{
    components::core::{
        animation::{
            animation_player, apply_base_pose, blend, clip_duration, freeze_at_percentage,
            freeze_at_time, mask_bind_ids, mask_weights, retarget_animation_scaled,
            retarget_model_from_url, start_time,
        },
        app::{name, ref_count},
        ecs::{children, parent},
    },
    entity::{add_component, get_component, mutate_component, remove_component, set_component},
    prelude::{block_until, time, Entity, EntityId},
};

/// This plays animations, and can handle blending and masking of animations together to create
/// complex effects. A single animation player can be attached to multiple entities in the scene.
#[derive(Debug, Clone, Copy)]
pub struct AnimationPlayer(pub EntityId);
impl AnimationPlayer {
    /// Create a new animation player, with `root` as the currently playing node
    pub fn new(root: impl AsRef<AnimationNode>) -> Self {
        let root: &AnimationNode = root.as_ref();
        let player = Entity::new()
            .with_default(animation_player())
            .with(children(), vec![root.0])
            .with(name(), "Animation player".to_string())
            .spawn();
        add_component(root.0, parent(), player);
        Self(player)
    }
    fn root(&self) -> Option<EntityId> {
        if let Some(children) = get_component(self.0, children()) {
            children.get(0).map(|x| *x)
        } else {
            None
        }
    }
    /// Replaces the current root node of the animation player with a new node
    pub fn play(&self, node: impl AsRef<AnimationNode>) {
        if let Some(root) = self.root() {
            remove_component(root, parent());
        }
        let new_root: &AnimationNode = node.as_ref();
        add_component(self.0, children(), vec![new_root.0]);
        add_component(new_root.0, parent(), self.0);
    }
    /// Set up retargeting
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

/// An animation node. Used in the animation player. It keeps an internal ref count.
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

/// Play clip from url animation node.
/// This is an animation node which can be plugged into an animation player or other animation nodes.
#[derive(Debug)]
pub struct PlayClipFromUrlNode(pub AnimationNode);
impl PlayClipFromUrlNode {
    /// Create a new node.
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
    /// Returns None if the duration hasn't been loaded yet
    pub fn peek_clip_duration(&self) -> Option<f32> {
        get_component(self.0 .0, clip_duration())
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
    /// Wait until the clip has been loaded
    pub async fn wait_until_loaded(&self) {
        self.clip_duration().await;
    }
}
impl AsRef<AnimationNode> for PlayClipFromUrlNode {
    fn as_ref(&self) -> &AnimationNode {
        &self.0
    }
}

/// Blend animation node.
/// This is an animation node which can be plugged into an animation player or other animation nodes.
#[derive(Debug, Clone)]
pub struct BlendNode(pub AnimationNode);
impl BlendNode {
    /// Create a new blend animation node.
    ///
    /// If the weight is 0, only the left animation will play.
    /// If the weight is 1, only the right animation will play.
    /// Values in between blend between the two animations.
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
    /// Set the weight of this blend node.
    ///
    /// If the weight is 0, only the left animation will play.
    /// If the weight is 1, only the right animation will play.
    /// Values in between blend between the two animations.
    pub fn set_weight(&self, weight: f32) {
        set_component(self.0 .0, blend(), weight);
    }
    /// Sets the mask of this blend node.
    ///
    /// For example `blend_node.set_mask(vec![("LeftLeg".to_string(), 1.)])` means
    /// that the LeftLeg is always controlled by the right animation.
    pub fn set_mask(&self, weights: Vec<(String, f32)>) {
        let (bind_ids, weights): (Vec<_>, Vec<_>) = weights.into_iter().unzip();
        add_component(self.0 .0, mask_bind_ids(), bind_ids);
        add_component(self.0 .0, mask_weights(), weights);
    }
    /// Sets a mask value to all bones of a humanoids lower body
    pub fn set_mask_humanoid_lower_body(&self, weight: f32) {
        self.set_mask(
            HUMANOID_LOWER_BODY
                .iter()
                .map(|x| (x.to_string(), weight))
                .collect(),
        );
    }
    /// Sets a mask value to all bones of a humanoids upper body
    pub fn set_mask_humanoid_upper_body(&self, weight: f32) {
        self.set_mask(
            HUMANOID_UPPER_BODY
                .iter()
                .map(|x| (x.to_string(), weight))
                .collect(),
        );
    }
}
impl AsRef<AnimationNode> for BlendNode {
    fn as_ref(&self) -> &AnimationNode {
        &self.0
    }
}

/// Animation retargeting configuration.
#[derive(Debug, Clone)]
pub enum AnimationRetargeting {
    /// Bone Translation comes from the animation data, unchanged.
    None,
    /// Bone Translation comes from the Target Skeleton's bind pose.
    Skeleton {
        /// Model url which the animation will be retargeted to.
        model_url: String,
    },
    /// Bone translation comes from the animation data, but is scaled by the Skeleton's proportions.
    /// This is the ratio between the bone length of the Target Skeleton (the skeleton the animation
    /// is being played on), and the Source Skeleton (the skeleton the animation was authored for).
    AnimationScaled {
        /// Rotates the Hips bone based on the difference between the rotation the animation models root and the retarget animations root
        normalize_hip: bool,
        /// Model url which the animation will be retargeted to.
        model_url: String,
    },
}
impl Default for AnimationRetargeting {
    fn default() -> Self {
        Self::None
    }
}

/// Get the bone entity from the bind_id; for example "LeftFoot"
pub fn get_bone_by_bind_id(entity: EntityId, bind_id: impl AsRef<str>) -> Option<EntityId> {
    if let Some(bid) = get_component(entity, crate::components::core::animation::bind_id()) {
        if &bid == bind_id.as_ref() {
            return Some(entity);
        }
    }
    if let Some(childs) = get_component(entity, children()) {
        for c in childs {
            if let Some(bid) = get_bone_by_bind_id(c, bind_id.as_ref()) {
                return Some(bid);
            }
        }
    }
    None
}

/// Bind-ids for a humanoid's lower body
pub const HUMANOID_LOWER_BODY: [&str; 9] = [
    "Hips",
    "LeftFoot",
    "LeftLeg",
    "LeftToeBase",
    "LeftUpLeg",
    "RightFoot",
    "RightLeg",
    "RightToeBase",
    "RightUpLeg",
];

/// Bind-ids for a humanoid's upper body
pub const HUMANOID_UPPER_BODY: [&str; 43] = [
    "Head",
    "LeftArm",
    "LeftForeArm",
    "LeftHand",
    "LeftHandIndex1",
    "LeftHandIndex2",
    "LeftHandIndex3",
    "LeftHandMiddle1",
    "LeftHandMiddle2",
    "LeftHandMiddle3",
    "LeftHandPinky1",
    "LeftHandPinky2",
    "LeftHandPinky3",
    "LeftHandRing1",
    "LeftHandRing2",
    "LeftHandRing3",
    "LeftHandThumb1",
    "LeftHandThumb2",
    "LeftHandThumb3",
    "LeftShoulder",
    "Neck",
    "RightArm",
    "RightForeArm",
    "RightHand",
    "RightHandIndex1",
    "RightHandIndex2",
    "RightHandIndex3",
    "RightHandMiddle1",
    "RightHandMiddle2",
    "RightHandMiddle3",
    "RightHandPinky1",
    "RightHandPinky2",
    "RightHandPinky3",
    "RightHandRing1",
    "RightHandRing2",
    "RightHandRing3",
    "RightHandThumb1",
    "RightHandThumb2",
    "RightHandThumb3",
    "RightShoulder",
    "Spine",
    "Spine1",
    "Spine2",
];

/// Bind-ids for a humanoid
const HUMANOID_SKELETON: [&str; 52] = [
    // Lower body for convenience
    "Hips",
    "LeftFoot",
    "LeftLeg",
    "LeftToeBase",
    "LeftUpLeg",
    "RightFoot",
    "RightLeg",
    "RightToeBase",
    "RightUpLeg",
    // Upper
    "Head",
    "LeftArm",
    "LeftForeArm",
    "LeftHand",
    "LeftHandIndex1",
    "LeftHandIndex2",
    "LeftHandIndex3",
    "LeftHandMiddle1",
    "LeftHandMiddle2",
    "LeftHandMiddle3",
    "LeftHandPinky1",
    "LeftHandPinky2",
    "LeftHandPinky3",
    "LeftHandRing1",
    "LeftHandRing2",
    "LeftHandRing3",
    "LeftHandThumb1",
    "LeftHandThumb2",
    "LeftHandThumb3",
    "LeftShoulder",
    "Neck",
    "RightArm",
    "RightForeArm",
    "RightHand",
    "RightHandIndex1",
    "RightHandIndex2",
    "RightHandIndex3",
    "RightHandMiddle1",
    "RightHandMiddle2",
    "RightHandMiddle3",
    "RightHandPinky1",
    "RightHandPinky2",
    "RightHandPinky3",
    "RightHandRing1",
    "RightHandRing2",
    "RightHandRing3",
    "RightHandThumb1",
    "RightHandThumb2",
    "RightHandThumb3",
    "RightShoulder",
    "Spine",
    "Spine1",
    "Spine2",
];
