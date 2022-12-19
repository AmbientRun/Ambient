pub use elements_animation as animation;
pub use elements_app as app;
pub use elements_cameras as cameras;
pub use elements_core as core;
pub use elements_ecs as ecs;
pub use elements_editor_derive::*;
pub use elements_model as model;
pub use elements_primitives as primitives;

pub fn init_components() {
    elements_ecs::init_components();
    elements_core::init_all_components();
    elements_element::init_components();
    animation::init_components();
    elements_gizmos::init_components();
    elements_cameras::init_all_components();
    app::init_components();
    elements_renderer::init_all_componets();
    elements_ui::init_all_componets();
    elements_input::init_all_components();
    model::init_components();
    elements_cameras::init_all_components();
}
