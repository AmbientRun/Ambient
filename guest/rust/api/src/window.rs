use crate::internal::wit;

/// Request that the window enters or exits fullscreen mode.
pub fn set_fullscreen(fullscreen: bool) {
    wit::client_window::set_fullscreen(fullscreen)
}
