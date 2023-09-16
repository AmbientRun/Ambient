use ambient_api::prelude::*;

pub fn level(pos: Vec2) -> f32 {
    let sdf = Sdf::Circle { radius: 20. };

    sdf.evaluate(pos)
}

/// A signed-distance function.
pub enum Sdf {
    Circle { radius: f32 },
    Translate { sdf: Box<Sdf>, offset: Vec2 },
}
impl Sdf {
    fn evaluate(&self, pos: Vec2) -> f32 {
        match self {
            Sdf::Circle { radius } => pos.length() - radius,
            Sdf::Translate { sdf, offset } => sdf.evaluate(pos - *offset),
        }
    }
}
