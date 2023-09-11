use ambient_ecs::{
    generated::{
        input::components::mouse_over_entity,
        messages::WindowMouseInput,
        ui::{
            components::{focus, focusable},
            messages::FocusChanged,
        },
    },
    read_messages, world_events, FnSystem, SystemGroup, WorldEventReader, WorldEventsExt,
};

pub fn systems() -> SystemGroup {
    let mut reader = WorldEventReader::new();
    SystemGroup::new(
        "focus",
        vec![Box::new(FnSystem::new(move |world, _| {
            for event in
                read_messages::<WindowMouseInput>(&mut reader, world.resource(world_events()))
            {
                if event.button == 0 {
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
