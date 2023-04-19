use ambient_app::AppBuilder;
use ambient_cameras::UICamera;
use ambient_core::async_ecs::async_run;
use ambient_ecs::{Entity, World};
use ambient_ecs_editor::ECSEditor;
use ambient_element::{Element, ElementComponent, ElementComponentExt, Group, Hooks};
use ambient_std::cb;
use ambient_ui_native::{FocusRoot, ScrollArea, WindowSized};

#[derive(Debug, Clone)]
struct ECSEditorUIWorld;
impl ElementComponent for ECSEditorUIWorld {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let async_run = hooks.world.resource(async_run()).clone();
        ECSEditor {
            get_world: cb(move |run| {
                let run = run.clone();
                async_run.run(move |world| run(world));
            }),
            on_change: cb(|world, diff| {
                diff.apply(world, Entity::new(), false);
            }),
        }
        .el()
    }
}

fn init(world: &mut World) {
    Group(vec![UICamera.el(), FocusRoot(vec![WindowSized(vec![ScrollArea::el(ECSEditorUIWorld.el().memoize_subtree(""))]).el()]).el()])
        .el()
        .spawn_interactive(world);
}

#[tokio::main]
async fn main() {
    env_logger::init();
    AppBuilder::simple_ui().run_world(init).await;
}
