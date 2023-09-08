use ambient_ecs::{
    generated::{
        input::components::mouse_over_entity,
        ui::{
            components::{focus, focusable},
            messages::FocusChanged,
        },
    },
    world_events, FnSystem, SystemGroup, WorldEventsExt,
};
use winit::event::{Event, MouseButton, WindowEvent};

pub fn systems() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new(
        "focus",
        vec![Box::new(FnSystem::new(|world, event| {
            if let Event::WindowEvent {
                event: WindowEvent::MouseInput { button, .. },
                ..
            } = event
            {
                if *button == MouseButton::Left {
                    let mouse_over = *world.resource(mouse_over_entity());
                    let focus_id = world
                        .get_cloned(mouse_over, focusable())
                        .unwrap_or_default();
                    let cur_focus = world.resource(focus()).clone();
                    if cur_focus != focus_id {
                        *world.resource_mut(focus()) = focus_id.clone();
                        world
                            .resource_mut(world_events())
                            .add_message(FocusChanged {
                                from_external: false,
                                focus: focus_id,
                            });
                    }
                }
            }
        }))],
    )
}
