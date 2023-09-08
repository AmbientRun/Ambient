//! Implements basic tabs.
use std::fmt::Debug;

use ambient_cb::{cb, Cb};
use ambient_element::{to_owned, use_state, Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_guest_bridge::{core::layout::components::space_between_items, ecs::ComponentValue};

use crate::{
    button::Button,
    default_theme::STREET,
    layout::{FlowColumn, FlowRow},
};

#[derive(Clone, Debug)]
/// A header bar of tabs. Does not contain the tab content.
pub struct TabBar<T: ToString + PartialEq + Clone + Debug + Sync + Send + 'static> {
    /// The tabs to display.
    pub tabs: Vec<T>,
    /// The currently selected tab.
    pub value: T,
    /// The callback to call when a tab is selected. Called with the tab value.
    pub on_change: Cb<dyn Fn(T) + Sync + Send>,
}
impl<T: ToString + PartialEq + Clone + Debug + Sync + Send + 'static> ElementComponent
    for TabBar<T>
{
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self {
            tabs,
            value,
            on_change,
        } = *self;
        FlowRow(
            tabs.into_iter()
                .map(|tab| {
                    Button::new(tab.to_string(), {
                        to_owned![on_change, tab];
                        move |_| on_change.0(tab.clone())
                    })
                    .toggled(tab == value)
                    .el()
                })
                .collect(),
        )
        .el()
        .with(space_between_items(), STREET)
    }
}

#[derive(Clone, Debug)]
/// A set of tabs. Contains a `TabBar` and the content of the selected tab.
pub struct Tabs<T: ToString + PartialEq + Default + Clone + Debug + Sync + Send + 'static> {
    /// The tabs to display.
    tabs: Vec<(T, Cb<dyn Fn() -> Element + Sync + Send>)>,
}
impl<T: ToString + PartialEq + Default + Clone + Debug + Sync + Send + 'static> Tabs<T> {
    /// Creates a new `Tabs` with no tabs.
    pub fn new() -> Self {
        Self {
            tabs: Default::default(),
        }
    }

    /// Adds a tab to the `Tabs`. The callback is called when the tab is selected, and should return the content of the tab.
    pub fn with_tab(
        mut self,
        tab: T,
        callback: impl Fn() -> Element + Sync + Send + 'static,
    ) -> Self {
        self.tabs.push((tab, cb(callback)));
        self
    }
}
impl<
        T: ToString + PartialEq + Default + ComponentValue + Clone + Debug + Sync + Send + 'static,
    > ElementComponent for Tabs<T>
{
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (value, set_value) = use_state(hooks, T::default());
        let selected_tab = self
            .tabs
            .iter()
            .find(|it| it.0 == value)
            .map(|it| it.1.clone())
            .unwrap_or(cb(Element::new));
        let key = value.to_string();

        FlowColumn::el([
            TabBar {
                tabs: self.tabs.iter().map(|it| it.0.clone()).collect(),
                value,
                on_change: cb(move |value| set_value(value)),
            }
            .el(),
            selected_tab().key(key),
        ])
        .with(space_between_items(), STREET)
    }
}
