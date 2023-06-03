use crate::{
    components::core::{
        animation::{animation_graph, blend},
        app::name,
        ecs::{children, parent},
    },
    entity::{add_component, despawn_recursive, get_component, set_component},
    prelude::{Entity, EntityId},
};

/// tmp
#[derive(Debug, Clone, Copy)]
pub struct AnimationGraph(pub EntityId);
impl AnimationGraph {
    /// tmp
    pub fn new(root: impl Into<AnimationNode>) -> Self {
        let root: AnimationNode = root.into();
        let graph = Entity::new()
            .with_default(animation_graph())
            .with(children(), vec![root.0])
            .with(name(), "Animation graph".to_string())
            .spawn();
        add_component(root.0, parent(), graph);
        Self(graph)
    }
    /// tmp
    pub fn replace_root(&self, new_root: impl Into<AnimationNode>) {
        let new_root: AnimationNode = new_root.into();
        if let Some(childs) = get_component(self.0, children()) {
            for c in childs {
                despawn_recursive(c);
            }
        }
        add_component(self.0, children(), vec![new_root.0]);
        add_component(new_root.0, parent(), self.0);
    }
}
/// tmp
#[derive(Debug, Clone, Copy)]
pub struct AnimationNode(EntityId);

/// tmp
#[derive(Debug, Clone, Copy)]
pub struct PlayClipFromUrlNode(pub EntityId);
impl PlayClipFromUrlNode {
    /// tmp
    pub fn new(url: impl Into<String>, looping: bool) -> Self {
        use crate::components::core::animation;
        Self(
            Entity::new()
                .with(animation::play_clip_from_url(), url.into())
                .with(name(), "Play clip from url".to_string())
                .with(animation::looping(), looping)
                .spawn(),
        )
    }
}
impl From<PlayClipFromUrlNode> for AnimationNode {
    fn from(value: PlayClipFromUrlNode) -> Self {
        Self(value.0)
    }
}

/// tmp
#[derive(Debug, Clone, Copy)]
pub struct BlendNode(pub EntityId);
impl BlendNode {
    /// tmp
    pub fn new(
        left: impl Into<AnimationNode>,
        right: impl Into<AnimationNode>,
        weight: f32,
    ) -> Self {
        use crate::components::core::animation;
        let left: AnimationNode = left.into();
        let right: AnimationNode = right.into();
        let node = Entity::new()
            .with(animation::blend(), weight)
            .with(name(), "Blend".to_string())
            .with(children(), vec![left.0, right.0])
            .spawn();
        add_component(left.0, parent(), node);
        add_component(right.0, parent(), node);
        Self(node)
    }
    /// tmp
    pub fn set_weight(&self, weight: f32) {
        set_component(self.0, blend(), weight);
    }
}
impl From<BlendNode> for AnimationNode {
    fn from(value: BlendNode) -> Self {
        Self(value.0)
    }
}
