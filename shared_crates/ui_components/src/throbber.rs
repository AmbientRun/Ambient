use std::time::Duration;

use ambient_element::ElementComponent;
use ambient_guest_bridge::components::text::font_family;

use crate::text::Text;

#[derive(Debug, Clone)]
/// Shows an animated progress bar to indicate that progress is
/// being made and has not frozen.
pub struct Throbber;

impl ElementComponent for Throbber {
    fn render(self: Box<Self>, hooks: &mut ambient_element::Hooks) -> ambient_element::Element {
        let (index, set_index) = hooks.use_state(0);
        hooks.use_interval_deps(Duration::from_secs_f32(0.1), false, index, move |_| set_index(index + 1));
        let s = match index % 4 {
            0 => "-",
            1 => "\\",
            2 => "|",
            3 => "/",
            _ => "*",
        };
        Text::el(s).set(font_family(), "Code".to_string())
    }
}
