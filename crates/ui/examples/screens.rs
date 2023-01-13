use elements_app::AppBuilder;
use elements_cameras::UICamera;
use elements_core::camera::active_camera;
use elements_ecs::World;
use elements_element::{Element, ElementComponent, ElementComponentExt, Group, Hooks};
use elements_ui::*;

#[derive(Debug, Clone)]
struct RootScreen;
impl ElementComponent for RootScreen {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let (screen, set_screen) = hooks.use_state(None);
        FocusRoot(vec![PageScreen(vec![
            ScreenContainer(screen).el(),
            Text::el("RootScreen"),
            Button::new("Open sub screen", move |_| {
                set_screen(Some(
                    SubScreen {
                        on_back: Cb::new({
                            let set_screen = set_screen.clone();
                            move || {
                                set_screen(None);
                            }
                        }),
                    }
                    .el(),
                ))
            })
            .el(),
        ])
        .el()])
        .el()
    }
}

#[derive(Debug, Clone)]
struct SubScreen {
    on_back: Cb<dyn Fn() + Sync + Send>,
}
impl ElementComponent for SubScreen {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let Self { on_back } = *self;
        let (screen, set_screen) = hooks.use_state(None);
        let (id, _) = hooks.use_state_with(|| friendly_id::create());
        PageScreen(vec![
            ScreenContainer(screen).el(),
            Text::el(format!("SubScreen {id}")),
            Button::new("Back", move |_| on_back()).el(),
            Button::new("Open sub screen", {
                let set_screen = set_screen.clone();
                move |_| {
                    set_screen(Some(
                        SubScreen {
                            on_back: Cb::new({
                                let set_screen = set_screen.clone();
                                move || {
                                    set_screen(None);
                                }
                            }),
                        }
                        .el(),
                    ))
                }
            })
            .el(),
            Button::new("Prompt", {
                let set_screen = set_screen.clone();
                move |_| {
                    set_screen(Some(Prompt::new("Testy", Cb(set_screen.clone()), |_, _| {}).el()));
                }
            })
            .el(),
            Button::new("Editor Prompt", move |_| {
                set_screen(Some(EditorPrompt::new("Testy", "Something".to_string(), Cb(set_screen.clone()), |_, _| {}).el()));
            })
            .el(),
        ])
        .el()
    }
}

fn init(world: &mut World) {
    Group(vec![UICamera.el().set(active_camera(), 0.), RootScreen.el()]).el().spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().run_world(init);
}
