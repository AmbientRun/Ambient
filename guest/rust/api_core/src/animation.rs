use crate::{
    components::core::{
        animation::animation_graph,
        app::name,
        ecs::{children, parent},
    },
    entity::{add_component, despawn_recursive, get_component},
    prelude::{Entity, EntityId},
};

/// tmp
#[derive(Debug, Clone, Copy)]
pub struct AnimationGraph(pub EntityId);
impl AnimationGraph {
    /// tmp
    pub fn new(root: AnimationNode) -> Self {
        let graph = Entity::new()
            .with_default(animation_graph())
            .with(children(), vec![root.0])
            .with(name(), "Animation graph".to_string())
            .spawn();
        add_component(root.0, parent(), graph);
        Self(graph)
    }
    /// tmp
    pub fn replace_root(&self, new_root: AnimationNode) {
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
pub struct AnimationNode(pub EntityId);
impl AnimationNode {
    /// tmp
    pub fn new_play_clip_from_url(url: impl Into<String>) -> Self {
        Self(
            Entity::new()
                .with(
                    crate::components::core::animation::play_clip_from_url(),
                    url.into(),
                )
                .with(name(), "Play clip from url".to_string())
                .spawn(),
        )
    }
}
