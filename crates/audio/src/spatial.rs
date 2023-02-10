use std::f32::consts::E;

use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// The speed of sound in units/s
pub const SPEED_OF_SOUND: f32 = 343.0;
/// The "maximum" speed of the source relative to the listener, in
/// units/block, where block is `block_len / sample_rate ~= BLOCK_DURATION`.
///
/// This is to prevent sharp cuts in the audio volume, direction, and delay if the source
/// moves to far from one block to the next.
pub(crate) const MAX_SPEED: f32 = 5.0;
/// Maximum allowed angular velocity of the emitter around the listener.
///
/// This is to prevent clipping due to IR sphere interpolation
///
/// radians/block
pub(crate) const MAX_ANGULAR_SPEED: f32 = 0.5;

#[derive(Copy, Debug, Clone, serde::Serialize, serde::Deserialize, kiwi_ui::ElementEditor)]
pub struct AudioEmitter {
    pub amplitude: f32,
    pub pos: Vec3,
    pub attenuation: Attenuation,
}

impl Default for AudioEmitter {
    fn default() -> Self {
        Self {
            amplitude: 1.0,
            pos: Default::default(),
            attenuation: Default::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AudioListener {
    /// The position of the right ear
    pub transform: Mat4,
    pub ear_distance: Vec3,
}

impl AudioListener {
    pub fn new(transform: Mat4, ear_distance: Vec3) -> Self {
        Self {
            transform,
            ear_distance,
        }
    }

    /// Returns the position of the left and right ear positions in world space
    pub fn ear_positions(&self) -> (Vec3, Vec3) {
        (
            self.transform.transform_point3(-self.ear_distance / 2.0),
            self.transform.transform_point3(self.ear_distance / 2.0),
        )
    }

    pub fn transform(&self) -> Mat4 {
        self.transform
    }
}

/// Defines the attenuation.
///
/// See:
/// https://www.desmos.com/calculator/mpbzwayz5f
#[derive(Serialize, Deserialize, Debug, Clone, Copy, kiwi_ui::ElementEditor)]
pub enum Attenuation {
    /// 1/(ax^2)
    #[editor("Inverse polynomial 1 / ({a}*x^2 + {b}*x + {c}")]
    InverseQuadratic(f32),
    /// e^(-ax)
    #[editor(slider, min = -10.0, max = 10.0)]
    Exponential(f32),
    /// e^(-(density*x)^gradient)
    #[editor(slider, min = -10.0, max = 10.0)]
    Smoothstep { density: f32, gradient: f32 },
    /// 1 / (1 + lin*x + quad*x^2)
    /// Increasing lin leads to a sharper falloff, while increasing `quad` leads to a bell shaped
    /// smaller area.
    ///
    /// Setting `lin=0` creates a smooth bell shaped falloff that approaches the physically correct
    /// inverse square law, without the infinite peak near 0.
    InversePoly { quad: f32, lin: f32, constant: f32 },
}

impl Default for Attenuation {
    fn default() -> Self {
        Self::InversePoly {
            quad: 0.0,
            lin: 0.0,
            constant: 1.0,
        }
    }
}

impl Attenuation {
    /// Calculates the attenuation factor clamped 0..=1
    pub fn attenuate(&self, dist: f32) -> f32 {
        match *self {
            Attenuation::InverseQuadratic(a) => 1.0 / (a * dist * dist),
            Attenuation::Exponential(a) => E.powf(-a * dist),
            Attenuation::Smoothstep { density, gradient } => {
                E.powf(-(density * dist).powf(gradient))
            }
            Attenuation::InversePoly {
                quad,
                lin,
                constant,
            } => 1.0 / (constant + lin * dist + quad * dist * dist),
        }
        .clamp(0.0, 1.0)
    }

    pub fn inverse(&self, amp: f32) -> f32 {
        let x = match self {
            Self::InverseQuadratic(a) => (1.0 / (amp * a)).sqrt(),
            Self::Exponential(a) => -amp.ln() / a,
            Self::Smoothstep { density, gradient } => (-amp.ln()).powf(1.0 / gradient) / density,
            Self::InversePoly {
                quad,
                lin,
                constant,
            } => {
                let h = lin / (2.0 * quad);
                -h + (h * h - constant / quad + 1.0 / (quad * amp)).sqrt()
            }
        };

        if x.is_normal() {
            x.clamp(0.0, 1e6)
        } else {
            1.0
        }
    }
}
