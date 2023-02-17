use glam::vec3;
use kiwi_core::{
    hierarchy::children,
    transform::{local_to_world, translation},
};
use kiwi_ecs::{components, query, SystemGroup};
use kiwi_element::{define_el_function_for_vec_element_newtype, Element, ElementComponent, ElementComponentExt, Hooks};

use crate::{app_background_color, padding, Borders, Dock, UIBase, UIExt, WindowSized};

components!("ui", {
    screen: (),
});

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "ui/screens",
        vec![query((local_to_world().changed(), children().changed())).incl(screen()).to_system(|q, world, qs, _| {
            for (_, (ltw, children)) in q.collect_cloned(world, qs) {
                let (_, _, pos) = ltw.to_scale_rotation_translation();
                for c in children {
                    if let Ok(p) = world.get_mut(c, translation()) {
                        p.x = -pos.x;
                        p.y = -pos.y;
                    }
                }
            }
        })],
    )
}

#[derive(Clone, Debug)]
pub struct ScreenContainer(pub Option<Element>);
impl ElementComponent for ScreenContainer {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        if let Some(content) = self.0 {
            UIBase.el().set(screen(), ()).children(vec![WindowSized(vec![Dock(vec![content]).el().set(translation(), vec3(0., 0., 0.1))])
                .el()
                .with_background(*app_background_color().set_a(0.99))
                .with_clickarea()])
        } else {
            Element::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct PageScreen(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(PageScreen);
impl ElementComponent for PageScreen {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        WindowSized(vec![Dock(self.0).el().init(padding(), Borders::even(30.))])
            .el()
            .with_background(*app_background_color().set_a(0.99))
            .with_clickarea()
    }
}

#[derive(Clone, Debug)]
pub struct DialogScreen(pub Element);
impl ElementComponent for DialogScreen {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        WindowSized(vec![Dock(vec![self.0]).el().init(padding(), Borders::even(30.))])
            .el()
            .with_background(*app_background_color().set_a(0.99))
            .with_clickarea()
    }
}
