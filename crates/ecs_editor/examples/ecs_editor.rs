use elements_app::{App, AppBuilder};
use elements_cameras::UICamera;
use elements_core::{async_ecs::async_run, camera::active_camera};
use elements_ecs::{EntityData, World};
use elements_ecs_editor::ECSEditor;
use elements_element::{Element, ElementComponent, ElementComponentExt, Group, Hooks};
use elements_std::Cb;
use elements_ui::{FocusRoot, ScrollArea, WindowSized};

#[derive(Debug, Clone)]
struct ECSEditorUIWorld;
impl ElementComponent for ECSEditorUIWorld {
    fn render(self: Box<Self>, world: &mut World, _hooks: &mut Hooks) -> Element {
        let async_run = world.resource(async_run()).clone();
        ECSEditor {
            get_world: Cb::new(move |run| {
                let run = run.clone();
                async_run.run(move |world| run(world));
            }),
            on_change: Cb::new(|world, diff| {
                diff.apply(world, EntityData::new(), false);
            }),
        }
        .el()
    }
}

fn init(world: &mut World) {
    Group(vec![
        UICamera.el().set(active_camera(), 0.),
        FocusRoot(vec![WindowSized(vec![ScrollArea::el(ECSEditorUIWorld.el().memoize_subtree(""))]).el()]).el(),
    ])
    .el()
    .spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().run_world(init);
}
