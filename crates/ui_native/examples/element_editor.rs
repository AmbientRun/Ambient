use ambient_app::{AmbientWindow, AppBuilder};
use ambient_cameras::UICamera;
use ambient_editor_derive::ElementEditor;
use ambient_element::{Element, ElementComponent, ElementComponentExt, Group, Hooks};
use ambient_ui_native::*;
use glam::*;

#[derive(Debug, Clone, Default, ElementEditor)]
pub struct MyStruct {
    a_float: f32,
    one_string: String,
    #[editor(prompt)]
    longer_string: String,
    sub_struct: SubStruct,
    #[editor(prompt = "List")]
    my_list: Vec<f32>,
    #[editor(slider, min = -180., max = 180.)]
    slider: f32,
    #[editor(slider, min = -180, max = 180)]
    int_slider: i32,
}
impl MyStruct {
    fn new() -> Self {
        MyStruct {
            my_list: vec![5., 3., 2.],
            sub_struct: SubStruct {
                my_enum_first: MyEnum::First,
                my_enum_second: MyEnum::Second { testy: 7. },
                my_enum_third: MyEnum::Third(3.),
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, ElementEditor)]
pub enum MyEnum {
    First,
    Second { testy: f32 },
    Third(f32),
}
impl Default for MyEnum {
    fn default() -> Self {
        Self::First
    }
}
#[derive(Debug, Clone, Default, ElementEditor)]
pub struct SubStruct {
    my_enum_first: MyEnum,
    my_enum_second: MyEnum,
    my_enum_third: MyEnum,
}

#[derive(Debug, Clone)]
struct Example;
impl ElementComponent for Example {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (state, set_state) = hooks.use_state(MyStruct::new());
        FocusRoot(vec![ScrollArea::el(
            ScrollAreaSizing::FitChildrenWidth,
            FlowColumn(vec![
                MyStruct::editor(state.clone(), set_state, Default::default()),
                Text::el(format!("{state:#?}")),
            ])
            .el()
            .with(space_between_items(), STREET),
        )])
        .el()
    }
}

async fn init(app: &mut AmbientWindow) {
    let world = &mut app.world;
    Group(vec![UICamera.el(), Example.el()])
        .el()
        .spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().block_on(init);
}
