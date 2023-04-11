use ambient_app::{App, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::{hierarchy::children, transform::translation};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_renderer::color;
use ambient_std::color::Color;
use ambient_ui::{
    layout::{height, width},
    Throbber, *,
};
use glam::*;

#[derive(Debug, Clone)]
struct WobbleRect;
impl ElementComponent for WobbleRect {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (state, set_state) = hooks.use_state(0.);
        hooks.use_frame(move |_| set_state(state + 1.));
        UIBase
            .el()
            .with(width(), 150.)
            .with(height(), 30. + (state as f32 * 0.01).sin() * 20.)
            .with_background(Color::rgba(1., 0., (state as f32 * 0.01).sin(), 1.).into())
    }
}

#[derive(Clone, Debug)]
struct MyContext(String);

#[derive(Debug, Clone)]
struct ContextUser;
impl ElementComponent for ContextUser {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (context, _) = hooks.consume_context::<MyContext>().unwrap();
        Text::el(context.0)
    }
}

#[derive(Debug, Clone)]
pub struct Two {
    first: Element,
    second: Element,
}

impl ElementComponent for Two {
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        Element::from(UIBase)
            .init_default(children())
            .children(vec![self.first.with(translation(), vec3(100., 0., 0.)), self.second.with(translation(), vec3(0., 100., 0.))])
    }
}

#[derive(Debug, Clone)]
struct InputTest;
impl ElementComponent for InputTest {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (value, set_value) = hooks.use_state("".to_string());
        FlowColumn::el([Throbber.el(), TextEditor::new(value, set_value).el()])
    }
}

#[derive(Debug, Clone)]
struct Example;
impl ElementComponent for Example {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (count, _set_count) = hooks.use_state(0);
        hooks.provide_context(|| MyContext(format!("context {count}")));
        eprintln!("Render example {count}");
        if count < 5 {
            Two {
                first: UIBase.el().with(width(), 150.).with(height(), 30.).with_background(Color::rgba(0.5, 1., 0.5, 1.).into()),
                second: FlowColumn(vec![
                    InputTest.el(),
                    Text::el(format!("You clicked {count} times")),
                    UIBase
                        .el()
                        .with(width(), 30. - count as f32 * 2.)
                        .with(height(), 30. + count as f32 * 30.)
                        .with_background(Color::rgba(0.5, 0.5, 0.5, 1.).into())
                        .with_clickarea()
                        .el(),
                    // .set(on_click(), Arc::new(move |world, _| {
                    //     set_count(count + 1);
                    // }))
                    WobbleRect.into(),
                    UIBase.el().with(width(), 250.).with(height(), 60.).with_background(Color::rgba(0.1, 0.1, 1.0, 1.).into()),
                    ContextUser.into(),
                ])
                .into(),
            }
            .into()
        } else {
            Text::el("DONE").with(color(), vec4(1., 0., 0., 1.))
        }
    }
}

async fn init(app: &mut App) {
    let world = &mut app.world;

    FocusRoot::el([Example.el()]).spawn_interactive(world);
    // ElementNode::from(UIRect {
    //     color: vec3(0.5, 0.5, 0.5),
    //     size: vec2(150., 30.),
    // }).create(world, None);
    world.dump_to_tmp_file();

    UICamera.el().spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().block_on(init);
}
