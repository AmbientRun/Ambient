use ambient_api::{core::layout::components::space_between_items, element::use_state, prelude::*};

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

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (state, set_state) = use_state(hooks, MyStruct::new());
    FocusRoot(vec![WindowSized::el(vec![ScrollArea::el(
        ScrollAreaSizing::FitParentWidth,
        FlowColumn(vec![
            MyStruct::editor(state.clone(), set_state, Default::default()),
            Text::el(format!("{state:#?}")),
        ])
        .el()
        .with(space_between_items(), STREET),
    )])])
    .el()
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
