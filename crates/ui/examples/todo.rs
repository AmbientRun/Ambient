use kiwi_app::AppBuilder;
use kiwi_cameras::UICamera;
use kiwi_core::camera::active_camera;
use kiwi_ecs::World;
use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use kiwi_ui::*;

#[derive(Debug, Clone)]
struct TodoList;
impl ElementComponent for TodoList {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let (dishes, set_dishes) = hooks.use_state(false);
        let (laundry, set_laundry) = hooks.use_state(false);
        FlowColumn(vec![
            FlowRow(vec![Checkbox { value: laundry, on_change: set_laundry }.el(), Text::el("Laundry")]).el(),
            FlowRow(vec![Checkbox { value: dishes, on_change: set_dishes }.el(), Text::el("Dishes")]).el(),
            match (dishes, laundry) {
                (true, true) => Text::el("Yay!"),
                (false, false) => Text::el("Stop watching Netflix dude..."),
                (_, _) => Text::el("One more to go"),
            },
        ])
        .el()
    }
}

fn init(world: &mut World) {
    TodoList.el().spawn_interactive(world);

    UICamera.el().set(active_camera(), 0.).spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().run_world(init);
}
