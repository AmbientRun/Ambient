use ambient_ecs::{components, Debuggable, Description, MaybeResource, Name, Networked, Resource, World};
use ambient_std::math::interpolate;
use glam::{uvec2, vec2, UVec2, Vec2};
use winit::window::{CursorGrabMode, CursorIcon, Window};

components!("app", {

    @[Resource, Name["Window Control"], Description["Allows controlling the window from afar."]]
    window_ctl: flume::Sender<WindowCtl>,

    @[MaybeResource, Debuggable, Networked, Name["Window scale factor"], Description["This number is usually 1, but on for instance retina displays it's 2."]]
    window_scale_factor: f64,
    @[MaybeResource, Debuggable, Networked, Name["Window logical size"], Description["The logical size is the physical size divided by the scale factor."]]
    window_logical_size: UVec2,
    @[MaybeResource, Debuggable, Networked, Name["Window physical size"], Description["The physical size is the actual number of pixels on the screen."]]
    window_physical_size: UVec2,
    @[MaybeResource, Debuggable, Networked, Name["Cursor position"], Description["Absolute mouse cursor position in screen space."]]
    cursor_position: Vec2,
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

pub fn mirror_window_components(src: &mut World, dst: &mut World) {
    let dr = dst.resource_entity();

    dst.set_if_changed(dr, window_physical_size(), *src.resource(window_physical_size())).unwrap();
    dst.set_if_changed(dr, window_logical_size(), *src.resource(window_logical_size())).unwrap();
    dst.set_if_changed(dr, window_scale_factor(), *src.resource(window_scale_factor())).unwrap();

    dst.set_if_changed(dr, cursor_position(), *src.resource(cursor_position())).unwrap();
}

/// Allows controlling the window
#[derive(Debug, Clone)]
pub enum WindowCtl {
    GrabCursor(CursorGrabMode),
    SetCursorIcon(CursorIcon),
    ShowCursor(bool),
}
