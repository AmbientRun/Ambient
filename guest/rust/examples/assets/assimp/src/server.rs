use ambient_api::prelude::*;
use packages::orbit_camera::concepts::OrbitCamera;

#[main]
pub fn main() {
    OrbitCamera::suggested().spawn();
}
