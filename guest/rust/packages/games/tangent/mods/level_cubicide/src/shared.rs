use ambient_api::prelude::*;

pub fn circle_point(radians: f32, radius: f32) -> Vec2 {
    vec2(radians.cos(), radians.sin()) * radius
}

pub fn spawnpoints() -> [(Vec3, f32); 5] {
    [
        (vec3(0.0, 0.0, 0.0), 10.0),
        (vec3(0.0, -100.0, 0.0), 10.0),
        (vec3(0.0, 100.0, 0.0), 10.0),
        (vec3(-100.0, 0.0, 0.0), 10.0),
        (vec3(100.0, 0.0, 0.0), 10.0),
    ]
}

pub fn level(pos: Vec2) -> f32 {
    let spawnpoints = spawnpoints();
    let spawnpoints_sdf = spawnpoints
        .iter()
        .map(|(p, r)| Sdf::translate(Sdf::circle(r + 10.), p.xy()))
        .reduce(|a, b| Sdf::union(a, b))
        .unwrap();

    let spawnpoint_bridges_sdf = spawnpoints
        .iter()
        .map(|p| p.0)
        .flat_map(|a| {
            spawnpoints
                .iter()
                .map(|p| p.0)
                .filter(move |b| *b != a)
                .map(move |b| (a, b))
        })
        .map(|(a, b)| Sdf::oriented_box(a.xy(), b.xy(), 4.))
        .reduce(|a, b| Sdf::union(a, b))
        .unwrap();

    let sdf = Sdf::smooth_union(spawnpoints_sdf, spawnpoint_bridges_sdf, 2.);
    sdf.evaluate(pos)
}

/// A signed-distance function.
pub enum Sdf {
    Circle {
        radius: f32,
    },
    OrientedBox {
        a: Vec2,
        b: Vec2,
        thickness: f32,
    },
    Translate {
        sdf: Box<Sdf>,
        offset: Vec2,
    },
    Union {
        sdf1: Box<Sdf>,
        sdf2: Box<Sdf>,
    },
    SmoothUnion {
        sdf1: Box<Sdf>,
        sdf2: Box<Sdf>,
        radius: f32,
    },
}
impl Sdf {
    // https://iquilezles.org/articles/distfunctions2d/
    fn evaluate(&self, pos: Vec2) -> f32 {
        match self {
            Sdf::Circle { radius } => pos.length() - radius,
            Sdf::OrientedBox { a, b, thickness } => {
                let (a, b, thickness) = (*a, *b, *thickness);

                let l = (b - a).length();
                let d = (b - a) / l;

                let mut q = pos - (a + b) * 0.5;
                q = mat2(vec2(d.x, -d.y), vec2(d.y, d.x)) * q;
                q = q.abs() - vec2(l, thickness) * 0.5;

                return q.max(Vec2::ZERO).length() + f32::min(f32::max(q.x, q.y), 0.0);
            }
            Sdf::Translate { sdf, offset } => sdf.evaluate(pos - *offset),
            Sdf::Union { sdf1, sdf2 } => sdf1.evaluate(pos).min(sdf2.evaluate(pos)),
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

    fn oriented_box(a: Vec2, b: Vec2, thickness: f32) -> Self {
        Sdf::OrientedBox { a, b, thickness }
    }

    fn translate(sdf: Sdf, offset: Vec2) -> Self {
        Sdf::Translate {
            sdf: Box::new(sdf),
            offset,
        }
    }

    fn union(sdf1: Sdf, sdf2: Sdf) -> Self {
        Sdf::Union {
            sdf1: Box::new(sdf1),
            sdf2: Box::new(sdf2),
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
