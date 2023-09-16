use ambient_api::prelude::*;

pub fn spawnpoints() -> [(Vec3, f32); 2] {
    [(vec3(0.0, 0.0, 0.0), 10.0), (vec3(0.0, -100.0, 0.0), 10.0)]
}

pub fn level(pos: Vec2) -> f32 {
    let sdf = spawnpoints()
        .iter()
        .map(|(p, r)| Sdf::translate(Sdf::circle(r + 10.), p.xy()))
        .reduce(|a, b| Sdf::smooth_union(a, b, 5.))
        .unwrap();

    sdf.evaluate(pos)
}

/// A signed-distance function.
pub enum Sdf {
    Circle {
        radius: f32,
    },
    Translate {
        sdf: Box<Sdf>,
        offset: Vec2,
    },
    SmoothUnion {
        sdf1: Box<Sdf>,
        sdf2: Box<Sdf>,
        radius: f32,
    },
}
impl Sdf {
    fn evaluate(&self, pos: Vec2) -> f32 {
        match self {
            Sdf::Circle { radius } => pos.length() - radius,
            Sdf::Translate { sdf, offset } => sdf.evaluate(pos - *offset),
            Sdf::SmoothUnion { sdf1, sdf2, radius } => {
                let d1 = sdf1.evaluate(pos);
                let d2 = sdf2.evaluate(pos);

                let h = (0.5 + 0.5 * (d2 - d1) / radius).clamp(0.0, 1.0);

                lerp(d2, d1, h) - radius * h * (1.0 - h)
            }
        }
    }

    fn circle(radius: f32) -> Self {
        Sdf::Circle { radius }
    }

    fn translate(sdf: Sdf, offset: Vec2) -> Self {
        Sdf::Translate {
            sdf: Box::new(sdf),
            offset,
        }
    }

    fn smooth_union(sdf1: Sdf, sdf2: Sdf, radius: f32) -> Self {
        Sdf::SmoothUnion {
            sdf1: Box::new(sdf1),
            sdf2: Box::new(sdf2),
            radius,
        }
    }
}

fn lerp<T: std::ops::Add + std::ops::Mul<f32>>(
    a: T,
    b: T,
    t: f32,
) -> <<T as std::ops::Mul<f32>>::Output as std::ops::Add>::Output
where
    <T as std::ops::Mul<f32>>::Output: std::ops::Add,
{
    a * (1.0 - t) + b * t
}
