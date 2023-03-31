use ambient_ecs::{components, Description, Name, Resource, World};
use ambient_std::math::interpolate;
use glam::{uvec2, vec2, UVec2, Vec2};
use winit::window::{CursorGrabMode, CursorIcon, Window};

pub use ambient_ecs::generated::components::core::app::{cursor_position, window_logical_size, window_physical_size, window_scale_factor};

components!("app", {
    @[Resource, Name["Window Control"], Description["Allows controlling the window from afar."]]
    window_ctl: flume::Sender<WindowCtl>,
});

pub fn screen_to_clip_space(world: &World, screen_pos: Vec2) -> Vec2 {
    let screen_size = *world.resource(window_physical_size());
    interpolate(screen_pos, Vec2::ZERO, screen_size.as_vec2(), vec2(-1., 1.), vec2(1., -1.))
}
pub fn get_mouse_clip_space_position(world: &World) -> Vec2 {
    let mouse_position = *world.resource(cursor_position());
    screen_to_clip_space(world, mouse_position)
}

pub fn get_window_sizes(window: &Window) -> (UVec2, UVec2, f64) {
    let size = uvec2(window.inner_size().width, window.inner_size().height);
    let sf = window.scale_factor();
    (size, (size.as_dvec2() / sf).as_uvec2(), sf)
}

/// Allows controlling the window
#[derive(Debug, Clone)]
pub enum WindowCtl {
    GrabCursor(CursorGrabMode),
    SetCursorIcon(CursorIcon),
    ShowCursor(bool),
    SetTitle(String),
}
