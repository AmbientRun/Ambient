use ambient_api::prelude::*;

// TODO: Should and could be replaced by a concept.
pub struct PlayerState {
    pub color: Vec4,
    pub ball: EntityId,
    pub ball_strokes: u32,
    pub ball_restore: Vec3,
    pub text: EntityId,
    pub text_container: EntityId,
    pub indicator: EntityId,
    pub indicator_arrow: EntityId,
}
