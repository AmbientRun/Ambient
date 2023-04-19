//! Defines several kinds of window-sized screens.

use crate::{
    default_theme::app_background_color,
    layout::{Dock, WindowSized},
    UIBase, UIExt,
};
use ambient_element::{define_el_function_for_vec_element_newtype, Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::{layout::screen, transform::translation};
use glam::vec3;

#[derive(Clone, Debug)]
/// Contains an element with a background color and click area.
pub struct ScreenContainer(pub Option<Element>);
impl ElementComponent for ScreenContainer {
    #[allow(clippy::clone_on_copy)]
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        if let Some(content) = self.0 {
            UIBase.el().with(screen(), ()).children(vec![WindowSized(vec![Dock(vec![content])
                .el()
                .with(translation(), vec3(0., 0., 0.1))])
            .el()
            .with_background(app_background_color().set_a(0.99).clone().into())
            .with_clickarea()
            .el()])
        } else {
            Element::new()
        }
    }
}

#[derive(Clone, Debug)]
/// Contains several elements with a background color and click area.
pub struct PageScreen(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(PageScreen);
impl ElementComponent for PageScreen {
    #[allow(clippy::clone_on_copy)]
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        WindowSized(vec![Dock(self.0).el().with_padding_even(30.)])
            .el()
            .with_background(app_background_color().set_a(0.99).clone().into())
            .with_clickarea()
            .el()
    }
}

#[derive(Clone, Debug)]
/// Contains a single element with a background color and click area.
// TODO: isn't this just [PageScreen] with a single element?
pub struct DialogScreen(pub Element);
impl ElementComponent for DialogScreen {
    #[allow(clippy::clone_on_copy)]
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        WindowSized(vec![Dock(vec![self.0]).el().with_padding_even(30.)])
            .el()
            .with_background(app_background_color().set_a(0.99).clone().into())
            .with_clickarea()
            .el()
    }
}
