use std::ops::{Add, Mul, Sub};

use ambient_std::math::interpolate;

pub struct CurvePoint<T> {
    pub input: f32,
    pub output: T,
}
impl<T> CurvePoint<T> {
    pub fn new(input: f32, output: T) -> Self {
        Self { input, output }
    }
}
pub struct Curve<T> {
    pub points: Vec<CurvePoint<T>>,
    pub start: Option<f32>,
    pub end: Option<f32>,
    pub looping: bool,
}
impl<T: Copy> Curve<T>
where
    T: Sub<T, Output = T>,
    T: Add<T, Output = T>,
    f32: Mul<T, Output = T>,
{
    pub fn new_looping(points: Vec<CurvePoint<T>>, end: f32) -> Self {
        Self {
            points,
            start: None,
            end: Some(end),
            looping: true,
        }
    }
    pub fn sample(&self, input: f32) -> Option<T> {
        if self.points.is_empty() {
            return None;
        }
        if self.points.len() == 1 {
            return Some(self.points[0].output);
        }

        // For some reason just multiplying with length below confused the compiler
        fn fmul(a: f32, b: f32) -> f32 {
            a * b
        }
        let start = self.start.unwrap_or(0.);
        let end = self.end.unwrap_or(self.points.last().unwrap().input);
        let length = end - start;
        let first = &self.points[0];
        let last = self.points.last().unwrap();
        let adj = fmul(((input - first.input).abs() / length).ceil() + 1., length);
        let input = (input - start + adj) % length + start; // Make sure we're between start and end

        let right = self.points.iter().position(|x| x.input >= input);
        if let Some(right) = right {
            if right == 0 {
                // We're before the first point
                if self.looping {
                    Some(interpolate(
                        input,
                        last.input - end,
                        first.input,
                        last.output,
                        first.output,
                    ))
                } else {
                    Some(self.points[0].output)
                }
            } else {
                // We're between two points
                let left = right - 1;
                Some(interpolate(
                    input,
                    self.points[left].input,
                    self.points[right].input,
                    self.points[left].output,
                    self.points[right].output,
                ))
            }
        } else {
            // We're past the last point
            if self.looping {
                Some(interpolate(
                    input,
                    last.input,
                    end + first.input,
                    last.output,
                    first.output,
                ))
            } else {
                Some(self.points.last().unwrap().output)
            }
        }
    }
}

#[test]
fn test() {
    use ambient_std::math::Round100;
    use glam::{vec3, Vec3};
    assert_eq!(
        Curve::new_looping(vec![CurvePoint::new(5., Vec3::X)], 24.)
            .sample(0.)
            .unwrap()
            .round100(),
        Vec3::X
    );
    let b = Curve::new_looping(
        vec![CurvePoint::new(6., Vec3::X), CurvePoint::new(18., Vec3::Y)],
        24.,
    );
    assert_eq!(b.sample(0.).unwrap().round100(), vec3(0.5, 0.5, 0.));
    assert_eq!(b.sample(3.).unwrap().round100(), vec3(0.75, 0.25, 0.));
    assert_eq!(b.sample(6.).unwrap().round100(), vec3(1., 0., 0.));
    assert_eq!(b.sample(12.).unwrap().round100(), vec3(0.5, 0.5, 0.));
    assert_eq!(b.sample(18.).unwrap().round100(), vec3(0., 1., 0.));
    assert_eq!(b.sample(21.).unwrap().round100(), vec3(0.25, 0.75, 0.));
    assert_eq!(b.sample(24.).unwrap().round100(), vec3(0.5, 0.5, 0.));

    assert_eq!(
        b.sample(-24. * 40. + 12.).unwrap().round100(),
        vec3(0.5, 0.5, 0.)
    );
    assert_eq!(
        b.sample(24. * 40. + 12.).unwrap().round100(),
        vec3(0.5, 0.5, 0.)
    );
}
