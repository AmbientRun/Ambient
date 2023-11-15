use ambient_api::prelude::*;
use packages::orbit_camera::concepts::OrbitCamera;

pub mod packages;

#[main]
pub fn main() {
    OrbitCamera::suggested().spawn();
}
