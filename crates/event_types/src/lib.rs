// This is a temporary crate until structured events lands: https://github.com/AmbientRun/Ambient/issues/228

/// Fired each frame.
pub const FRAME: &str = "core/frame";
/// Fired on a collision. Components will contain the `ids` of the objects.
pub const COLLISION: &str = "core/collision";
/// Fired when a collider is loaded. Components will contain the `id` of the object.
pub const COLLIDER_LOAD: &str = "core/collider_load";
/// Fired when the module is loaded.
pub const MODULE_LOAD: &str = "core/module_load";
/// Fired when the module is unloaded.
pub const MODULE_UNLOAD: &str = "core/module_unload";
/// The window gained or lost focus
pub const WINDOW_FOCUSED: &str = "core/window_focused";
/// The window received a character input
pub const WINDOW_RECEIVED_CHARACTER: &str = "core/window_received_character";
/// The window received a character input
pub const WINDOW_MODIFIERS_CHANGED: &str = "core/window_modifiers_changed";
/// The window received a keyboard input
pub const WINDOW_KEYBOARD_INPUT: &str = "core/window_keyboard_input";
/// The window received a mouse button press or release
pub const WINDOW_MOUSE_INPUT: &str = "core/window_mouse_input";
/// The window received a mouse wheel change
pub const WINDOW_MOUSE_WHEEL: &str = "core/window_mouse_wheel";
/// The mouse cursor was moved
pub const WINDOW_MOUSE_MOTION: &str = "core/window_mouse_motion";
