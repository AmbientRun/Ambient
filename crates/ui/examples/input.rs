use elements_app::AppBuilder;
use elements_cameras::UICamera;
use elements_core::camera::active_camera;
use elements_ecs::World;
use elements_element::{Element, ElementComponent, ElementComponentExt, Group, Hooks};
use elements_ui::*;
use glam::Vec3;
use indexmap::IndexMap;

#[derive(Debug, Clone)]
struct Example;
impl ElementComponent for Example {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let (text, set_text) = hooks.use_state("Enter some text".to_string());
        let (vector3, set_vector3) = hooks.use_state(Vec3::ZERO);
        let (index_map, set_index_map) =
            hooks.use_state(vec![("First".to_string(), "Second".to_string())].into_iter().collect::<IndexMap<String, String>>());
        let (list, set_list) = hooks.use_state(vec!["First".to_string(), "Second".to_string()]);
        let (minimal_list, set_minimal_list) = hooks.use_state(vec!["First".to_string(), "Second".to_string()]);
        let row = |name, editor| FlowRow(vec![Text::el(name).set(min_width(), 110.), editor]).el();
        FocusRoot(vec![FlowColumn(vec![
            row("TextInput", TextInput::new(text, Cb(set_text)).el()),
            // Button::new("Focus test", |_| {}).hotkey(winit::event::VirtualKeyCode::Back).el(),
            row(
                "DropDownSelect",
                DropdownSelect {
                    content: Text::el("Select"),
                    on_select: Cb::new(|_| {}),
                    items: vec![Text::el("First"), Text::el("Second")],
                    inline: false,
                }
                .el(),
            ),
            row("Vec3", Vec3::editor(vector3, Cb(set_vector3), Default::default())),
            row("IndexMap", IndexMap::editor(index_map, Cb(set_index_map), Default::default())),
            row("ListEditor", ListEditor { value: list, on_change: Some(Cb(set_list)) }.el()),
            row(
                "MinimalListEditor",
                MinimalListEditor {
                    value: minimal_list,
                    on_change: Some(Cb(set_minimal_list)),
                    item_opts: Default::default(),
                    add_presets: None,
                    add_title: "Add".to_string(),
                }
                .el(),
            ),
        ])
        .el()
        .set(width(), 200.)
        .set(space_between_items(), STREET)
        .set(padding(), Borders::even(STREET))])
        .el()
    }
}

fn init(world: &mut World) {
    Group(vec![UICamera.el().set(active_camera(), 0.), Example.el()]).el().spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().run_world(init);
}
