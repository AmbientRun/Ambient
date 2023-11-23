use ambient_api::{
    element::{use_frame, use_ref_with, use_rerender_signal},
    prelude::*,
};

pub mod packages;

#[main]
pub fn main() {
    PerformanceBar.el().spawn_interactive();
}

#[element_component]
fn PerformanceBar(hooks: &mut Hooks) -> Element {
    let frame_times = use_ref_with(hooks, |_| Vec::new());
    let rerender = use_rerender_signal(hooks);
    use_frame(hooks, {
        to_owned!(frame_times);
        move |_| {
            let mut frame_times = frame_times.lock();
            frame_times.push(delta_time());
            if frame_times.len() > 100 {
                frame_times.remove(0);
            }
            rerender();
        }
    });
    let fps = {
        let frame_times = frame_times.lock();
        let fps = frame_times.len() as f32 / frame_times.iter().sum::<f32>();
        fps
    };
    Text::el(format!("Fps: {fps}"))
}
