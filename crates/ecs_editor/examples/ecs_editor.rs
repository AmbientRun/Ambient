use ambient_app::AppBuilder;
use ambient_cameras::UICamera;
use ambient_core::async_ecs::async_run;
use ambient_ecs::World;
use ambient_ecs_editor::{ECSEditor, InspectableAsyncWorld};
use ambient_element::{element_component, Element, ElementComponentExt, Group, Hooks};
use ambient_native_std::cb;
use ambient_ui_native::{FocusRoot, ScrollArea, ScrollAreaSizing, WindowSized};
use std::sync::Arc;

#[element_component]
fn ECSEditorUIWorld(hooks: &mut Hooks) -> Element {
    let async_run = hooks.world.resource(async_run()).clone();
    ECSEditor {
        world: Arc::new(InspectableAsyncWorld(cb(move |cb| {
            async_run.run(move |world| cb(world))
        }))),
    }
    .el()
}

fn init(world: &mut World) {
    Group(vec![
        UICamera.el(),
        FocusRoot(vec![WindowSized(vec![ScrollArea::el(
            ScrollAreaSizing::FitChildrenWidth,
            ECSEditorUIWorld.el().memoize_subtree(""),
        )])
        .el()])
        .el(),
    ])
    .el()
    .spawn_interactive(world);
}

#[tokio::main]
async fn main() {
    env_logger::init();
    AppBuilder::simple_ui().run_world(init).await;
}
