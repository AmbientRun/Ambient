use ambient_core::runtime;
use ambient_element::ElementComponent;

use crate::Text;

#[derive(Debug, Clone)]
/// Shows an animated progress bar to indicate that progress is
/// being made and has not frozen.
pub struct Throbber;

impl ElementComponent for Throbber {
    fn render(self: Box<Self>, hooks: &mut ambient_element::Hooks) -> ambient_element::Element {
        let (status, set_status) = hooks.use_state(String::new());
        let width = 5;

        // Create a . .. ... .. . throbber
        let mut stages = (0..width)
            .map(move |i| ".".repeat(i))
            .map(move |v| format!("{v:<width$}"))
            .chain((0..width).map(move |i| ".".repeat(width - i)).map(move |v| format!("{v:>width$}")))
            .cycle();

        hooks.use_spawn(move |w| {
            let rt = w.resource(runtime());
            let task = rt.spawn(async move {
                use ambient_std::IntoDuration;
                let mut tick = ambient_sys::time::interval(200.ms());
                loop {
                    tick.tick().await;
                    set_status(stages.next().unwrap());
                }
            });

            Box::new(move |_| task.abort())
        });

        Text::el(status)
    }
}
