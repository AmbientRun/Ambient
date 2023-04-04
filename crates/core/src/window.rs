use ambient_ecs::{components, Debuggable, Description, MaybeResource, Name, Networked, Resource, World};
use ambient_std::math::interpolate;
use glam::{uvec2, vec2, UVec2, Vec2};
use winit::window::{CursorGrabMode, CursorIcon, Window};

components!("app", {
    @[Resource, Name["Window Control"], Description["Allows controlling the window from afar."]]
    window_ctl: flume::Sender<WindowCtl>,

    @[MaybeResource, Debuggable, Networked, Name["Window scale factor"], Description["The DPI/pixel scale factor of the window.\nOn standard displays, this is 1, but it can be higher on high-DPI displays like Apple Retina displays."]]
    window_scale_factor: f64,
    @[MaybeResource, Debuggable, Networked, Name["Window logical size"], Description["The logical size is the physical size divided by the scale factor."]]
    window_logical_size: UVec2,
    @[MaybeResource, Debuggable, Networked, Name["Window physical size"], Description["The physical size is the actual number of pixels on the screen."]]
    window_physical_size: UVec2,
    @[MaybeResource, Debuggable, Networked, Name["Cursor position"], Description["Absolute mouse cursor position in screen-space. This is the _logical_ position. Multiply by the window_scale_factor to get the physical position."]]
    cursor_position: Vec2,
});

pub fn screen_to_clip_space(world: &World, screen_pos: Vec2) -> Vec2 {
    let screen_size = *world.resource(window_logical_size());
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
