pub(crate) mod component;
pub(crate) mod conversion;
pub(crate) mod executor;
pub(crate) mod generated;
pub(crate) mod wit;

use crate::internal::{
    component::Entity,
    executor::{FrameState, EXECUTOR},
};

extern "Rust" {
    fn main();
}

struct Guest;
impl wit::guest::Guest for Guest {
    fn init() {
        once_cell::sync::Lazy::force(&EXECUTOR);
        unsafe { main() };
    }

    fn exec(time: f32, event_name: String, components: guest::Entity) {
        let components = Entity(components.into_iter().collect());
        EXECUTOR.execute(FrameState::new(time), event_name.as_str(), &components);
    }
}

use wit::{__link_section, guest};
wit::export_bindings!(Guest);
