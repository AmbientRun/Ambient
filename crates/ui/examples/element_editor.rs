use elements_app::App;
use elements_cameras::UICamera;
use elements_core::camera::active_camera;
use elements_ecs::World;
use elements_editor_derive::ElementEditor;
use elements_element::{Element, ElementComponent, ElementComponentExt, Group, Hooks};
use elements_ui::*;
use glam::*;

#[derive(Debug, Clone, Default, ElementEditor)]
pub struct MyStruct {
    a_float: f32,
    one_string: String,
    sub_struct: SubStruct,
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
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let (state, set_state) = hooks.use_state(MyStruct::new());
        FocusRoot(vec![ScrollArea(
            FlowColumn(vec![MyStruct::editor(state.clone(), Some(Cb(set_state)), Default::default()), Text::el(format!("{:#?}", state))])
                .el()
                .set(space_between_items(), STREET),
        )
        .el()])
        .el()
    }
}

fn init(world: &mut World) {
    Group(vec![UICamera.el().set(active_camera(), 0.), Example.el()]).el().spawn_interactive(world);
}

fn main() {
    env_logger::init();
    App::run_ui(init);
}
