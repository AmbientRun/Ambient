//! A prelude for users of the crate. Imports all the most commonly used types and functions.

pub use crate::{
    button::*, clickarea::*, default_theme::*, dropdown::*, editor::*, layout::*, prompt::*,
    screens::*, scroll_area::*, select::*, tabs::*, text::*, throbber::*, use_focus,
    use_window_logical_resolution, use_window_physical_resolution, with_rect, Focus, FocusRoot,
    Line, Rectangle, UIBase, UIElement, UIExt,
};
pub use ambient_cb::{cb, Cb};
pub use ambient_element::{
    self, element_component, to_owned, Element, ElementComponent, ElementComponentExt, ElementTree,
    Group, Hooks, Memo, Wrap,
};
pub use ambient_guest_bridge::ecs::World;
