use ambient_app::{App, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_element::{Element, ElementComponent, ElementComponentExt, Group, Hooks};
use ambient_ui::*;
use element::element_component;
use glam::Vec3;
use indexmap::IndexMap;

#[element_component]
fn FocusViz(hooks: &mut Hooks) -> Element {
    let (focus, _) = hooks.consume_context::<Focus>().unwrap();
    Text::el(format!("{:?}", focus))
}

#[derive(Debug, Clone)]
struct Example;
impl ElementComponent for Example {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (text, set_text) = hooks.use_state("Enter some text".to_string());
        let (float, set_float) = hooks.use_state(0.0);
        let (vector3, set_vector3) = hooks.use_state(Vec3::ZERO);
        let (index_map, set_index_map) =
            hooks.use_state(vec![("First".to_string(), "Second".to_string())].into_iter().collect::<IndexMap<String, String>>());
        let (list, set_list) = hooks.use_state(vec!["First".to_string(), "Second".to_string()]);
        let (minimal_list, set_minimal_list) = hooks.use_state(vec!["First".to_string(), "Second".to_string()]);
        let row = |name, editor| FlowRow(vec![Text::el(name).set(min_width(), 110.), editor]).el();
        FocusRoot(vec![FlowColumn(vec![
            FocusViz.el(),
            row("TextInput", TextInput::new(text, set_text).el()),
            row("F32Input", F32Input { value: float, on_change: set_float }.el()),
            // Button::new("Focus test", |_| {}).hotkey(winit::event::VirtualKeyCode::Back).el(),
            row(
                "DropDownSelect",
                DropdownSelect {
                    content: Text::el("Select"),
                    on_select: cb(|_| {}),
                    items: vec![Text::el("First"), Text::el("Second")],
                    inline: false,
                }
                .el(),
            ),
            row("Vec3", Vec3::editor(vector3, set_vector3, Default::default())),
            row("IndexMap", IndexMap::editor(index_map, set_index_map, Default::default())),
            row("ListEditor", ListEditor { value: list, on_change: Some(set_list) }.el()),
            row(
                "MinimalListEditor",
                MinimalListEditor {
                    value: minimal_list,
                    on_change: Some(set_minimal_list),
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

async fn init(app: &mut App) {
    let world = &mut app.world;
    Group(vec![UICamera.el().set(active_camera(), 0.), Example.el()]).el().spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().block_on(init);
}
