use std::sync::Arc;

use elements_app::{App, AppBuilder};
use elements_cameras::UICamera;
use elements_core::{camera::active_camera, hierarchy::children, on_frame, transform::translation};
use elements_ecs::World;
use elements_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use elements_renderer::color;
use elements_std::color::Color;
use elements_ui::{
    layout::{height, width}, Throbber, *
};
use glam::*;

#[derive(Debug, Clone)]
struct WobbleRect;
impl ElementComponent for WobbleRect {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let (state, set_state) = hooks.use_state(0.);
        UIBase
            .el()
            .set(width(), 150.)
            .set(height(), 30. + (state as f32 * 0.01).sin() * 20.)
            .with_background(Color::rgba(1., 0., (state as f32 * 0.01).sin(), 1.))
            .listener(on_frame(), Arc::new(move |_world, _, _| set_state(state + 1.)))
    }
}

#[derive(Clone, Debug)]
struct MyContext(String);

#[derive(Debug, Clone)]
struct ContextUser;
impl ElementComponent for ContextUser {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
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
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
        Element::from(UIBase)
            .init_default(children())
            .children(vec![self.first.set(translation(), vec3(100., 0., 0.)), self.second.set(translation(), vec3(0., 100., 0.))])
    }
}

#[derive(Debug, Clone)]
struct InputTest;
impl ElementComponent for InputTest {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let (value, set_value) = hooks.use_state("".to_string());
        FlowColumn::el([Throbber.el(), TextInput::new(value, Cb(set_value)).el()])
    }
}

#[derive(Debug, Clone)]
struct Example;
impl ElementComponent for Example {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let (count, _set_count) = hooks.use_state(0);
        hooks.provide_context(|| MyContext(format!("context {}", count)));
        println!("Render example {}", count);
        if count < 5 {
            Two {
                first: UIBase.el().set(width(), 150.).set(height(), 30.).with_background(Color::rgba(0.5, 1., 0.5, 1.)),
                second: FlowColumn(vec![
                    InputTest.el(),
                    Text::el(format!("You clicked {} times", count)),
                    UIBase
                        .el()
                        .set(width(), 30. - count as f32 * 2.)
                        .set(height(), 30. + count as f32 * 30.)
                        .with_background(Color::rgba(0.5, 0.5, 0.5, 1.))
                        .with_clickarea(),
                    // .set(on_click(), Arc::new(move |world, _| {
                    //     set_count(count + 1);
                    // }))
                    WobbleRect.into(),
                    UIBase.el().set(width(), 250.).set(height(), 60.).with_background(Color::rgba(0.1, 0.1, 1.0, 1.)),
                    ContextUser.into(),
                ])
                .into(),
            }
            .into()
        } else {
            Text::el("DONE").set(color(), vec4(1., 0., 0., 1.))
        }
    }
}

fn init(world: &mut World) {
    FocusRoot::el([Example.el()]).spawn_interactive(world);
    // ElementNode::from(UIRect {
    //     color: vec3(0.5, 0.5, 0.5),
    //     size: vec2(150., 30.),
    // }).create(world, None);
    world.dump_to_tmp_file();

    UICamera.el().set(active_camera(), 0.).spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().run_world(init);
}
