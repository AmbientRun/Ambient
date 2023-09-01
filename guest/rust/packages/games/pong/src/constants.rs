#![allow(dead_code)]

use std::f32::consts::PI;

pub const X_BOUNDARY: f32 = 1.;
pub const Y_BOUNDARY: f32 = 1.;

pub const BALL_V_PER_FRAME: f32 = 0.01;
pub const BALL_ACCELERATION: f32 = 0.05;
pub const BALL_SPINNING: f32 = PI / 4.; // radians / ratio of paddle from the center (-0.5 - 0.5)
pub const BALL_RADIUS: f32 = 0.1;
pub const PADDLE_V_PER_FRAME: f32 = BALL_V_PER_FRAME * 2.;
pub const PADDLE_LENGTH: f32 = 0.3;
pub const PADDLE_WIDTH: f32 = 0.1;
pub const SCREEN_PADDING: f32 = 0.2;
